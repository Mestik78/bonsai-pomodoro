use serde::{Deserialize, Serialize};
use super::timer::TimerSession;
use std::fs;
use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    #[serde(default)]
    pub timers: Vec<TimerSession>,
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
                if let Ok(state) = serde_json::from_str::<AppState>(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, is_production: bool) {
        if let Some(path) = Self::state_file_path(is_production) {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let tmp_path = path.with_extension("json.tmp");
                if fs::write(&tmp_path, json).is_ok() {
                    let _ = fs::rename(tmp_path, path);
                }
            }
        }
    }
}
