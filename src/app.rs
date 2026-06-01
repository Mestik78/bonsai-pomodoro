use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;
use chrono::Utc;

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    PostTimerInput {
        title: String,
        description: String,
        focus: u8,
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TimerState {
    New,
    Running,
    Paused,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimerSession {
    #[serde(rename = "start-time")]
    pub start_time: String,
    
    pub duration: u64,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<TimerState>,
    
    #[serde(rename = "time-left")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_left: Option<u64>,

    #[serde(rename = "actual-runtime", skip_serializing_if = "Option::is_none")]
    pub actual_runtime: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct AppState {
    #[serde(default)]
    timers: Vec<TimerSession>,
}

pub struct App {
    pub current_tab: usize,
    pub should_quit: bool,
    pub tab_titles: Vec<&'static str>,
    pub timers: Vec<TimerSession>,
    pub last_tick: Instant,
    pub mode: AppMode,
    pub is_production: bool,
}

impl App {
    fn state_file_path(is_production: bool) -> Option<std::path::PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "Mestik", "BonsaiPomodoro") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = fs::create_dir_all(data_dir);
            }
            if is_production {
                Some(data_dir.join("state.json"))
            } else {
                Some(data_dir.join("state_dev.json"))
            }
        } else {
            None
        }
    }

    pub fn new(is_production: bool) -> Self {
        let mut timers = Vec::new();
        
        if let Some(path) = Self::state_file_path(is_production) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                    timers = state.timers;
                }
            }
        }

        let mut need_new = false;
        let mut new_duration = 50 * 60;
        if let Some(first) = timers.first_mut() {
            if first.state.is_none() {
                need_new = true;
                new_duration = first.duration;
            } else if first.state == Some(TimerState::Running) {
                first.state = Some(TimerState::Paused);
            }
        } else {
            need_new = true;
        }

        if need_new {
            timers.insert(0, TimerSession {
                start_time: Utc::now().to_rfc3339(),
                duration: new_duration,
                state: Some(TimerState::New),
                time_left: Some(new_duration),
                actual_runtime: None,
                title: None,
                description: None,
            });
        }

        Self {
            current_tab: 0,
            should_quit: false,
            tab_titles: vec!["Temporizador", "Bosque"],
            timers,
            last_tick: Instant::now(),
            mode: AppMode::Normal,
            is_production,
        }
    }

    pub fn save_state(&self) {
        if let Some(path) = Self::state_file_path(self.is_production) {
            let state = AppState {
                timers: self.timers.clone(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&state) {
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn active_timer(&self) -> &TimerSession {
        &self.timers[0]
    }

    pub fn active_timer_mut(&mut self) -> &mut TimerSession {
        &mut self.timers[0]
    }

    pub fn toggle_timer(&mut self) {
        let state = self.active_timer().state.clone();
        match state {
            Some(TimerState::New) | Some(TimerState::Paused) => {
                let timer = self.active_timer_mut();
                timer.state = Some(TimerState::Running);
                if state == Some(TimerState::New) {
                    timer.start_time = Utc::now().to_rfc3339();
                }
                self.last_tick = Instant::now();
            },
            Some(TimerState::Running) => {
                let timer = self.active_timer_mut();
                timer.state = Some(TimerState::Paused);
            },
            None => {
                let duration = self.active_timer().duration;
                self.timers.insert(0, TimerSession {
                    start_time: Utc::now().to_rfc3339(),
                    duration,
                    state: Some(TimerState::New),
                    time_left: Some(duration),
                    actual_runtime: None,
                    title: None,
                    description: None,
                });
            }
        }
    }

    pub fn add_minutes(&mut self, minutes: i64) {
        let timer = self.active_timer_mut();
        if timer.state == Some(TimerState::Running) || timer.state.is_none() { return; }
        
        let current_time = timer.time_left.unwrap_or(0);
        let current_minutes = (current_time as i64) / 60;
        let new_minutes = current_minutes + minutes;
        
        let mut new_time = 0;
        if new_minutes > 0 {
            let current_seconds = (current_time as i64) % 60;
            new_time = (new_minutes * 60 + current_seconds) as u64;
        }

        if timer.state == Some(TimerState::New) && new_time < 1 {
            new_time = 1;
        }

        timer.time_left = Some(new_time);
        if timer.state == Some(TimerState::New) {
            timer.duration = new_time;
        }
    }

    pub fn add_seconds(&mut self, seconds: i64) {
        let timer = self.active_timer_mut();
        if timer.state == Some(TimerState::Running) || timer.state.is_none() { return; }
        
        let current_time = timer.time_left.unwrap_or(0);
        let new_time_i = (current_time as i64) + seconds;
        
        let mut new_time = if new_time_i <= 0 { 0 } else { new_time_i as u64 };
        
        if timer.state == Some(TimerState::New) && new_time < 1 {
            new_time = 1;
        }
        
        timer.time_left = Some(new_time);
        if timer.state == Some(TimerState::New) {
            timer.duration = new_time;
        }
    }

    pub fn reset_timer(&mut self) {
        let timer = self.active_timer_mut();
        if timer.state.is_some() {
            timer.time_left = Some(timer.duration);
            timer.state = Some(TimerState::New);
        }
    }

    pub fn finish_early(&mut self) {
        let mut just_finished = false;
        {
            let timer = self.active_timer_mut();
            if timer.state.is_some() {
                let current_time = timer.time_left.unwrap_or(0);
                let actual = timer.duration.saturating_sub(current_time);
                timer.actual_runtime = if actual == timer.duration { None } else { Some(actual) };
                timer.state = None;
                timer.time_left = None;
                just_finished = true;
            }
        }
        if just_finished {
            self.mode = AppMode::PostTimerInput {
                title: String::new(),
                description: String::new(),
                focus: 0,
            };
            self.save_state();
        }
    }

    pub fn on_tick(&mut self) {
        let is_running = self.active_timer().state == Some(TimerState::Running);
        if is_running {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_tick).as_secs();
            
            if elapsed >= 1 {
                let mut just_finished = false;
                {
                    let timer = self.active_timer_mut();
                    let current_time = timer.time_left.unwrap_or(0);
                    if current_time >= elapsed {
                        timer.time_left = Some(current_time - elapsed);
                    } else {
                        timer.time_left = Some(0);
                    }

                    if timer.time_left == Some(0) {
                        timer.actual_runtime = None;
                        timer.state = None;
                        timer.time_left = None;
                        just_finished = true;
                    }
                }
                
                self.last_tick += std::time::Duration::from_secs(elapsed);
                
                if just_finished {
                    self.mode = AppMode::PostTimerInput {
                        title: String::new(),
                        description: String::new(),
                        focus: 0,
                    };
                    self.save_state();
                }
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tab_titles.len();
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = self.tab_titles.len() - 1;
        } else {
            self.current_tab -= 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn input_char(&mut self, c: char) {
        if let AppMode::PostTimerInput { ref mut title, ref mut description, focus } = self.mode {
            if c == '\n' && focus == 0 {
                return;
            }
            if focus == 0 {
                title.push(c);
            } else {
                description.push(c);
            }
        }
    }

    pub fn input_backspace(&mut self) {
        if let AppMode::PostTimerInput { ref mut title, ref mut description, focus } = self.mode {
            if focus == 0 {
                title.pop();
            } else {
                description.pop();
            }
        }
    }

    pub fn input_tab(&mut self, reverse: bool) {
        if let AppMode::PostTimerInput { ref mut focus, .. } = self.mode {
            if reverse {
                *focus = if *focus == 0 { 1 } else { *focus - 1 };
            } else {
                *focus = (*focus + 1) % 2;
            }
        }
    }

    pub fn submit_input(&mut self) {
        let (t, d) = if let AppMode::PostTimerInput { ref title, ref description, .. } = self.mode {
            (title.clone(), description.clone())
        } else {
            return;
        };

        let timer = self.active_timer_mut();
        if !t.is_empty() {
            timer.title = Some(t);
        }
        if !d.is_empty() {
            timer.description = Some(d);
        }
        
        self.save_state();
        self.mode = AppMode::Normal;
    }
}
