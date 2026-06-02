use serde::{Deserialize, Serialize};
use crate::models::plant::PlantType;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TimerState {
    New,
    Starting(String),
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

    pub seed: u64,

    #[serde(default)]
    pub plant_type: PlantType,
}

impl TimerSession {
    pub fn new(duration: u64) -> Self {
        Self {
            start_time: chrono::Utc::now().to_rfc3339(),
            duration,
            state: Some(TimerState::New),
            time_left: Some(duration),
            actual_runtime: None,
            title: None,
            description: None,
            seed: rand::random(),
            plant_type: PlantType::Bonsai,
        }
    }

    pub fn toggle(&mut self, last_tick: &mut std::time::Instant) {
        let state = self.state.clone();
        match state {
            Some(TimerState::New) => {
                self.state = Some(TimerState::Starting(chrono::Utc::now().to_rfc3339()));
                *last_tick = std::time::Instant::now();
            },
            Some(TimerState::Paused) => {
                self.state = Some(TimerState::Running);
                *last_tick = std::time::Instant::now();
            },
            Some(TimerState::Running) => {
                self.state = Some(TimerState::Paused);
            },
            _ => {}
        }
    }

    pub fn add_minutes(&mut self, minutes: i64) {
        if self.state != Some(TimerState::New) { return; }
        
        let current_time = self.time_left.unwrap_or(0);
        let current_minutes = (current_time as i64) / 60;
        let new_minutes = current_minutes + minutes;
        
        let mut new_time = 0;
        if new_minutes > 0 {
            let current_seconds = (current_time as i64) % 60;
            new_time = (new_minutes * 60 + current_seconds) as u64;
        }

        if new_time < 1 { new_time = 1; }

        self.time_left = Some(new_time);
        self.duration = new_time;
    }

    pub fn add_seconds(&mut self, seconds: i64) {
        if self.state != Some(TimerState::New) { return; }
        
        let current_time = self.time_left.unwrap_or(0);
        let new_time_i = (current_time as i64) + seconds;
        
        let mut new_time = if new_time_i <= 0 { 0 } else { new_time_i as u64 };
        if new_time < 1 { new_time = 1; }
        
        self.time_left = Some(new_time);
        self.duration = new_time;
    }

    pub fn reset(&mut self) {
        if self.state.is_some() {
            self.time_left = Some(self.duration);
            self.state = Some(TimerState::New);
        }
    }

    pub fn finish_early(&mut self) -> bool {
        if self.state.is_some() {
            let current_time = self.time_left.unwrap_or(0);
            let actual = self.duration.saturating_sub(current_time);
            self.actual_runtime = if actual == self.duration { None } else { Some(actual) };
            self.state = None;
            self.time_left = None;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, last_tick: &mut std::time::Instant) -> bool {
        let state_clone = self.state.clone();
        if let Some(TimerState::Starting(start_time_str)) = state_clone {
            if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&start_time_str) {
                let now = chrono::Utc::now();
                if now.signed_duration_since(start_time).num_milliseconds() >= 500 {
                    self.state = Some(TimerState::Running);
                    self.start_time = now.to_rfc3339();
                    *last_tick = std::time::Instant::now();
                }
            }
        }

        if self.state == Some(TimerState::Running) {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(*last_tick).as_secs();
            
            if elapsed >= 1 {
                let current_time = self.time_left.unwrap_or(0);
                if current_time >= elapsed {
                    self.time_left = Some(current_time - elapsed);
                } else {
                    self.time_left = Some(0);
                }

                *last_tick += std::time::Duration::from_secs(elapsed);

                if self.time_left == Some(0) {
                    self.actual_runtime = None;
                    self.state = None;
                    self.time_left = None;
                    return true;
                }
            }
        }
        false
    }
}
