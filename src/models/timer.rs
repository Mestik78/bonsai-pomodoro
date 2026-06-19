use serde::{Deserialize, Serialize};
use crate::models::plant::PlantType;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TimerState {
    New,
    Starting(String),
    Running,
    Paused,
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(from = "TimerSessionData", into = "TimerSessionData")]
pub struct TimerSession {
    pub start_time: String,
    pub duration: u64,
    pub state: Option<TimerState>,
    pub time_left: Option<u64>,
    pub actual_runtime: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub seed: u64,
    pub plant_type: PlantType,
    pub unknown_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct TimerSessionData {
    #[serde(rename = "start-time")]
    start_time: String,
    duration: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<TimerState>,
    #[serde(rename = "time-left", skip_serializing_if = "Option::is_none")]
    time_left: Option<u64>,
    #[serde(rename = "actual-runtime", skip_serializing_if = "Option::is_none")]
    actual_runtime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing)]
    seed: Option<u64>,
    #[serde(default)]
    plant_type: PlantType,
    #[serde(flatten)]
    unknown_fields: std::collections::HashMap<String, serde_json::Value>,
}

impl From<TimerSessionData> for TimerSession {
    fn from(data: TimerSessionData) -> Self {
        let seed = if let Some(s) = data.seed {
            s
        } else {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            data.start_time.hash(&mut hasher);
            hasher.finish()
        };
        Self {
            start_time: data.start_time,
            duration: data.duration,
            state: data.state,
            time_left: data.time_left,
            actual_runtime: data.actual_runtime,
            title: data.title,
            description: data.description,
            seed,
            plant_type: data.plant_type,
            unknown_fields: data.unknown_fields,
        }
    }
}

impl Into<TimerSessionData> for TimerSession {
    fn into(self) -> TimerSessionData {
        TimerSessionData {
            start_time: self.start_time,
            duration: self.duration,
            state: self.state,
            time_left: self.time_left,
            actual_runtime: self.actual_runtime,
            title: self.title,
            description: self.description,
            seed: Some(self.seed),
            plant_type: self.plant_type,
            unknown_fields: self.unknown_fields,
        }
    }
}

impl TimerSession {
    pub fn new(duration: u64, plant_type: PlantType) -> Self {
        let mut t = Self {
            start_time: chrono::Local::now().to_rfc3339(),
            duration,
            state: Some(TimerState::New),
            time_left: Some(duration),
            actual_runtime: None,
            title: None,
            description: None,
            seed: 0,
            plant_type,
            unknown_fields: std::collections::HashMap::new(),
        };
        
        let hash_seed = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            t.start_time.hash(&mut hasher);
            hasher.finish()
        };
        t.seed = hash_seed;
        t
    }

    pub fn elapsed(&self) -> u64 {
        if self.state.is_none() {
            self.actual_runtime.unwrap_or(self.duration)
        } else {
            self.duration.saturating_sub(self.time_left.unwrap_or(self.duration))
        }
    }

    pub fn progress(&self) -> f32 {
        if self.state == Some(TimerState::New) || matches!(self.state, Some(TimerState::Starting(_))) {
            return 0.0;
        }
        
        let actual = self.elapsed();
        
        let d = (actual as f64).max(0.0);
        let x = (d / 3000.0).clamp(0.0, 1.0);
        (x * x * (3.0 - 2.0 * x)) as f32
    }

    pub fn toggle(&mut self, last_tick: &mut std::time::Instant) {
        let state = self.state.clone();
        match state {
            Some(TimerState::New) => {
                let now = chrono::Local::now().to_rfc3339();
                self.start_time = now.clone();
                self.state = Some(TimerState::Starting(now));
                
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                self.start_time.hash(&mut hasher);
                self.seed = hasher.finish();
                
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

    pub fn tick(&mut self, last_tick: &mut std::time::Instant, is_production: bool) -> bool {
        let state_clone = self.state.clone();
        if let Some(TimerState::Starting(start_time_str)) = state_clone {
            if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&start_time_str) {
                let now = chrono::Local::now();
                if now.signed_duration_since(start_time).num_milliseconds() >= 500 {
                    self.state = Some(TimerState::Running);
                    *last_tick = std::time::Instant::now();
                }
            }
        }

        if self.state == Some(TimerState::Running) {
            let now = std::time::Instant::now();
            let speed_multiplier = if is_production { 1 } else { 600 };
            
            let elapsed_real_ms = now.duration_since(*last_tick).as_millis() as u64;
            let sim_secs = (elapsed_real_ms * speed_multiplier) / 1000;
            
            if sim_secs > 0 {
                let consumed_real_ms = (sim_secs * 1000) / speed_multiplier;
                
                let current_time = self.time_left.unwrap_or(0);
                if current_time >= sim_secs {
                    self.time_left = Some(current_time - sim_secs);
                } else {
                    self.time_left = Some(0);
                }

                *last_tick += std::time::Duration::from_millis(consumed_real_ms);

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
