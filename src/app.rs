use std::time::Instant;

use crate::models::forest::ForestState;
use crate::models::stats::StatsState;

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    PostTimerInput {
        title: String,
        description: String,
        focus: u8,
    }
}

use crate::models::timer::{TimerSession, TimerState};
use crate::models::app_state::AppState;

pub struct App {
    pub current_tab: usize,
    pub should_quit: bool,
    pub tab_titles: Vec<&'static str>,
    pub timers: Vec<TimerSession>,
    pub last_tick: Instant,
    pub mode: AppMode,
    pub is_production: bool,
    pub forest: ForestState,
    pub stats: StatsState,
}

impl App {
    pub fn new(is_production: bool) -> Self {
        let state = AppState::load(is_production);
        let mut timers = state.timers;

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
            timers.insert(0, TimerSession::new(new_duration));
        }

        let finished_count = timers.iter().filter(|t| t.state.is_none()).count();
        let mut forest = ForestState::new();
        if finished_count > 0 {
            forest.list_state.select(Some(0));
            forest.selected_day = 0;
        }

        let stats = StatsState::new();

        Self {
            current_tab: 0,
            should_quit: false,
            tab_titles: vec!["Timer", "Forest", "Stats"],
            timers,
            last_tick: Instant::now(),
            mode: AppMode::Normal,
            is_production,
            forest,
            stats,
        }
    }

    pub fn save_state(&self) {
        let state = AppState {
            timers: self.timers.clone(),
        };
        state.save(self.is_production);
    }

    pub fn active_timer(&self) -> &TimerSession {
        &self.timers[0]
    }

    pub fn active_timer_mut(&mut self) -> &mut TimerSession {
        &mut self.timers[0]
    }

    pub fn toggle_timer(&mut self) {
        if self.timers[0].state.is_none() {
            let duration = self.timers[0].duration;
            self.timers.insert(0, TimerSession::new(duration));
        } else {
            self.timers[0].toggle(&mut self.last_tick);
        }
    }

    pub fn add_minutes(&mut self, minutes: i64) {
        self.timers[0].add_minutes(minutes);
    }

    pub fn add_seconds(&mut self, seconds: i64) {
        self.timers[0].add_seconds(seconds);
    }

    pub fn reset_timer(&mut self) {
        self.timers[0].reset();
    }

    pub fn finish_early(&mut self) {
        if self.timers[0].finish_early() {
            self.mode = AppMode::PostTimerInput {
                title: String::new(),
                description: String::new(),
                focus: 0,
            };
            self.save_state();
        }
    }

    pub fn on_tick(&mut self) {
        if self.timers[0].tick(&mut self.last_tick) {
            self.mode = AppMode::PostTimerInput {
                title: String::new(),
                description: String::new(),
                focus: 0,
            };
            self.save_state();
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

        {
            let timer = self.active_timer_mut();
            if !t.is_empty() {
                timer.title = Some(t);
            }
            if !d.is_empty() {
                timer.description = Some(d);
            }
        }
        
        let new_duration = self.timers[0].duration;
        self.timers.insert(0, TimerSession::new(new_duration));
        
        self.save_state();
        self.mode = AppMode::Normal;
    }

    pub fn forest_next(&mut self) {
        self.forest.next(&self.timers);
    }

    pub fn forest_previous(&mut self) {
        self.forest.previous(&self.timers);
    }

    pub fn forest_enter(&mut self) {
        self.forest.enter(&self.timers);
    }

    pub fn forest_escape(&mut self) {
        self.forest.escape();
    }

    pub fn forest_nav_left(&mut self) {
        self.forest.nav_left(&self.timers);
    }

    pub fn forest_nav_right(&mut self) {
        self.forest.nav_right(&self.timers);
    }

    pub fn forest_nav_up(&mut self) {
        self.forest.nav_up(&self.timers);
    }

    pub fn forest_nav_down(&mut self) {
        self.forest.nav_down(&self.timers);
    }

    pub fn stats_next(&mut self) {
        self.stats.next();
    }

    pub fn stats_previous(&mut self) {
        self.stats.previous();
    }
}
