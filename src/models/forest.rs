use ratatui::widgets::ListState;
use crate::models::timer::TimerSession;
use crate::models::tabs::{TabEvent, EventResult};

#[derive(PartialEq)]
pub enum ForestLevel {
    Day,
    Bonsai,
}

#[derive(PartialEq)]
pub enum NavDir {
    Up,
    Down,
}

pub struct ForestState {
    pub list_state: ListState,
    pub level: ForestLevel,
    pub selected_day: usize,
    pub selected_bonsai: usize,
    pub cols: usize,
    pub last_nav_dir: NavDir,
    pub cached_days: Vec<String>,
}

impl ForestState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
            level: ForestLevel::Day,
            selected_day: 0,
            selected_bonsai: 0,
            cols: 1,
            last_nav_dir: NavDir::Down,
            cached_days: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: &TabEvent, timers: &[TimerSession]) -> EventResult {
        match event {
            TabEvent::Up { .. } => {
                if self.level == ForestLevel::Bonsai {
                    self.nav_up(timers);
                } else {
                    self.previous();
                }
                EventResult::Consumed
            },
            TabEvent::Down { .. } => {
                if self.level == ForestLevel::Bonsai {
                    self.nav_down(timers);
                } else {
                    self.next();
                }
                EventResult::Consumed
            },
            TabEvent::Left => {
                if self.level == ForestLevel::Bonsai {
                    self.nav_left(timers);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Right => {
                if self.level == ForestLevel::Bonsai {
                    self.nav_right(timers);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Enter => {
                self.enter();
                EventResult::Consumed
            },
            TabEvent::Esc => {
                self.escape();
                EventResult::Consumed
            },
            TabEvent::Tab => EventResult::Ignored,
        }
    }

    pub fn update_cache(&mut self, timers: &[TimerSession]) {
        self.cached_days = Self::get_unique_days(timers);
    }

    pub fn get_unique_days(timers: &[TimerSession]) -> Vec<String> {
        let mut days = Vec::new();
        for t in timers.iter().filter(|t| t.state.is_none()) {
            let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                Err(_) => t.start_time.clone(),
            };
            if !days.contains(&date_str) {
                days.push(date_str);
            }
        }
        days
    }

    pub fn next(&mut self) {
        let days_count = self.cached_days.len();
        if days_count == 0 { return; }
        if self.selected_day >= days_count - 1 {
            self.selected_day = 0;
            self.last_nav_dir = NavDir::Up;
        } else {
            self.selected_day += 1;
            self.last_nav_dir = NavDir::Down;
        }
    }

    pub fn previous(&mut self) {
        let days_count = self.cached_days.len();
        if days_count == 0 { return; }
        if self.selected_day == 0 {
            self.selected_day = days_count - 1;
            self.last_nav_dir = NavDir::Down;
        } else {
            self.selected_day -= 1;
            self.last_nav_dir = NavDir::Up;
        }
    }

    pub fn enter(&mut self) {
        if self.cached_days.is_empty() { return; }
        self.level = ForestLevel::Bonsai;
        self.selected_bonsai = 0;
    }

    pub fn escape(&mut self) {
        self.level = ForestLevel::Day;
    }

    pub fn get_bonsai_count_for_selected_day(&self, timers: &[TimerSession]) -> usize {
        if self.cached_days.is_empty() { return 0; }
        
        let selected_idx = self.selected_day;
        if selected_idx >= self.cached_days.len() { return 0; }
        let date_str = &self.cached_days[selected_idx];
        
        timers.iter().filter(|t| t.state.is_none() && {
            let t_date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                Err(_) => t.start_time.clone(),
            };
            &t_date_str == date_str
        }).count()
    }

    pub fn nav_left(&mut self, timers: &[TimerSession]) {
        if self.level != ForestLevel::Bonsai { return; }
        let count = self.get_bonsai_count_for_selected_day(timers);
        if count == 0 { return; }
        self.last_nav_dir = NavDir::Up;
        if self.selected_bonsai > 0 {
            self.selected_bonsai -= 1;
        } else {
            self.previous();
            let new_count = self.get_bonsai_count_for_selected_day(timers);
            self.selected_bonsai = new_count.saturating_sub(1);
        }
    }

    pub fn nav_right(&mut self, timers: &[TimerSession]) {
        if self.level != ForestLevel::Bonsai { return; }
        let count = self.get_bonsai_count_for_selected_day(timers);
        if count == 0 { return; }
        self.last_nav_dir = NavDir::Down;
        if self.selected_bonsai + 1 < count {
            self.selected_bonsai += 1;
        } else {
            self.next();
            self.selected_bonsai = 0;
        }
    }

    pub fn nav_up(&mut self, timers: &[TimerSession]) {
        if self.level != ForestLevel::Bonsai { return; }
        let count = self.get_bonsai_count_for_selected_day(timers);
        if count == 0 { return; }
        self.last_nav_dir = NavDir::Up;
        let cols = self.cols.max(1);
        if self.selected_bonsai >= cols {
            self.selected_bonsai -= cols;
        } else {
            let col = self.selected_bonsai % cols;
            self.previous();
            let new_count = self.get_bonsai_count_for_selected_day(timers);
            if new_count == 0 {
                self.selected_bonsai = 0;
            } else {
                let rem = new_count % cols;
                let last_row_start = new_count - rem;
                let target = last_row_start + col;
                if target >= new_count {
                    self.selected_bonsai = target.saturating_sub(cols);
                } else {
                    self.selected_bonsai = target;
                }
            }
        }
    }

    pub fn nav_down(&mut self, timers: &[TimerSession]) {
        if self.level != ForestLevel::Bonsai { return; }
        let count = self.get_bonsai_count_for_selected_day(timers);
        if count == 0 { return; }
        self.last_nav_dir = NavDir::Down;
        let cols = self.cols.max(1);
        
        let target = self.selected_bonsai + cols;
        if target < count {
            self.selected_bonsai = target;
        } else {
            let col = self.selected_bonsai % cols;
            self.next();
            let new_count = self.get_bonsai_count_for_selected_day(timers);
            if new_count == 0 {
                self.selected_bonsai = 0;
            } else {
                self.selected_bonsai = col.min(new_count.saturating_sub(1));
            }
        }
    }
}
