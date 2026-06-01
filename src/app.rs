use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize)]
struct AppState {
    time_left: u64,
}


pub struct App {
    pub current_tab: usize,
    pub should_quit: bool,
    pub tab_titles: Vec<&'static str>,
    pub time_left: u64, // Segundos restantes
    pub is_running: bool,
    pub last_tick: Instant,
}

impl App {
    fn state_file_path() -> Option<std::path::PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "Mestik", "BonsaiPomodoro") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = fs::create_dir_all(data_dir);
            }
            Some(data_dir.join("state.json"))
        } else {
            None
        }
    }

    pub fn new() -> Self {
        let mut time_left = 50 * 60; // 50 minutos iniciales
        
        if let Some(path) = Self::state_file_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                    time_left = state.time_left;
                }
            }
        }

        Self {
            current_tab: 0,
            should_quit: false,
            tab_titles: vec!["Temporizador", "Bosque"],
            time_left,
            is_running: false,
            last_tick: Instant::now(),
        }
    }

    pub fn save_state(&self) {
        if let Some(path) = Self::state_file_path() {
            let state = AppState {
                time_left: self.time_left,
            };
            if let Ok(json) = serde_json::to_string(&state) {
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn toggle_timer(&mut self) {
        self.is_running = !self.is_running;
        if self.is_running {
            // Al reanudar, reiniciamos last_tick para no restar el tiempo de pausa
            self.last_tick = Instant::now();
        }
    }

    pub fn add_minutes(&mut self, minutes: i64) {
        let current_minutes = (self.time_left as i64) / 60;
        let new_minutes = current_minutes + minutes;
        
        if new_minutes <= 0 {
            self.time_left = 0;
            self.is_running = false;
        } else {
            // Mantenemos los segundos actuales y solo modificamos los minutos
            let current_seconds = (self.time_left as i64) % 60;
            self.time_left = (new_minutes * 60 + current_seconds) as u64;
        }
    }

    pub fn add_seconds(&mut self, seconds: i64) {
        let new_time = (self.time_left as i64) + seconds;
        
        if new_time <= 0 {
            self.time_left = 0;
            self.is_running = false;
        } else {
            self.time_left = new_time as u64;
        }
    }

    pub fn on_tick(&mut self) {
        if self.is_running {
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_tick).as_secs();
            
            if elapsed >= 1 {
                if self.time_left >= elapsed {
                    self.time_left -= elapsed;
                } else {
                    self.time_left = 0;
                    self.is_running = false;
                }
                self.last_tick += std::time::Duration::from_secs(elapsed);
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
}
