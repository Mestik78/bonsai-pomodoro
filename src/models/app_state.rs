use serde::{Deserialize, Serialize};
use super::timer::TimerSession;
use std::fs;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    #[serde(default)]
    pub timers: Vec<TimerSession>,
    #[serde(default = "default_global_seed")]
    pub global_seed: u64,
    #[serde(flatten)]
    pub unknown_fields: std::collections::HashMap<String, serde_json::Value>,
    #[serde(skip)]
    pub raw_unparseable_backup: Option<String>,
}

fn default_global_seed() -> u64 {
    rand::random()
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            timers: Vec::new(),
            global_seed: default_global_seed(),
            unknown_fields: std::collections::HashMap::new(),
            raw_unparseable_backup: None,
        }
    }
}

impl AppState {
    pub fn state_file_path(is_production: bool) -> Option<std::path::PathBuf> {
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

    pub fn load(is_production: bool) -> Self {
        if let Some(path) = Self::state_file_path(is_production) {
            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<AppState>(&content) {
                    Ok(state) => return state,
                    Err(_) => {
                        let mut state = Self::default();
                        state.raw_unparseable_backup = Some(content);
                        return state;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, is_production: bool) {
        if let Some(path) = Self::state_file_path(is_production) {
            if self.raw_unparseable_backup.is_some() && self.timers.is_empty() {
                // No overwrite if file is broken and we haven't done anything
                return;
            }
            
            if let Some(backup) = &self.raw_unparseable_backup {
                let bak_path = path.with_extension("json.bak");
                let _ = fs::write(bak_path, backup);
            }

            if let Ok(json) = serde_json::to_string_pretty(self) {
                let tmp_path = path.with_extension("json.tmp");
                if fs::write(&tmp_path, json).is_ok() {
                    let _ = fs::rename(tmp_path, &path);
                }
            }
        }
    }
}
