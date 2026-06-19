use ratatui::widgets::ListState;
use crate::models::timer::TimerSession;
use crate::models::tabs::{TabEvent, EventResult};

#[derive(PartialEq)]
pub enum HistoryLevel {
    Day,
    Bonsai,
}

#[derive(PartialEq)]
pub enum NavDir {
    Up,
    Down,
}

pub struct HistoryState {
    pub list_state: ListState,
    pub level: HistoryLevel,
    pub selected_day: usize,
    pub selected_bonsai: usize,
    pub cols: usize,
    pub last_nav_dir: NavDir,
    pub cached_days: Vec<String>,
    pub is_searching: bool,
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>, // (day_idx, bonsai_idx)
    pub search_index: usize,
}

impl HistoryState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
            level: HistoryLevel::Day,
            selected_day: 0,
            selected_bonsai: 0,
            cols: 1,
            last_nav_dir: NavDir::Down,
            cached_days: Vec::new(),
            is_searching: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_index: 0,
        }
    }

    pub fn handle_event(&mut self, event: &TabEvent, timers: &[TimerSession]) -> EventResult {
        if self.is_searching {
            match event {
                TabEvent::Char(c) => {
                    self.search_query.push(*c);
                    EventResult::Consumed
                },
                TabEvent::Backspace => {
                    self.search_query.pop();
                    EventResult::Consumed
                },
                TabEvent::Enter => {
                    self.search_matches.clear();
                    self.search_index = 0;
                    if !self.search_query.is_empty() {
                        let query = self.search_query.to_lowercase();
                        
                        let mut days_map: std::collections::HashMap<String, Vec<&TimerSession>> = std::collections::HashMap::new();
                        let finished_timers: Vec<&TimerSession> = timers.iter().filter(|t| t.state.is_none()).collect();
                        for t in finished_timers {
                            let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                                Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                                Err(_) => t.start_time.clone(),
                            };
                            if !days_map.contains_key(&date_str) {
                                days_map.insert(date_str.clone(), Vec::new());
                            }
                            days_map.get_mut(&date_str).unwrap().push(t);
                        }

                        for (day_idx, date_str) in self.cached_days.iter().enumerate() {
                            let mut day_matches = false;
                            if date_str.to_lowercase().contains(&query) {
                                day_matches = true;
                            }
                            
                            if let Some(timers_for_day) = days_map.get(date_str) {
                                for (bonsai_idx, t) in timers_for_day.iter().enumerate() {
                                    let mut timer_matches = day_matches;
                                    if !timer_matches {
                                        if let Some(ref title) = t.title {
                                            if title.to_lowercase().contains(&query) {
                                                timer_matches = true;
                                            }
                                        }
                                        if let Some(ref desc) = t.description {
                                            if desc.to_lowercase().contains(&query) {
                                                timer_matches = true;
                                            }
                                        }
                                    }
                                    if timer_matches {
                                        self.search_matches.push((day_idx, bonsai_idx));
                                    }
                                }
                            }
                        }
                        
                        if !self.search_matches.is_empty() {
                            let (d, b) = self.search_matches[0];
                            self.selected_day = d;
                            self.selected_bonsai = b;
                            self.level = HistoryLevel::Bonsai;
                        }
                    }
                    self.is_searching = false;
                    EventResult::Consumed
                },
                TabEvent::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                    self.search_matches.clear();
                    EventResult::Consumed
                },
                _ => EventResult::Ignored,
            }
        } else {
            match event {
                TabEvent::Up { .. } => {
                    if self.level == HistoryLevel::Bonsai {
                        self.nav_up(timers);
                    } else {
                        self.previous();
                    }
                    EventResult::Consumed
                },
                TabEvent::Down { .. } => {
                    if self.level == HistoryLevel::Bonsai {
                        self.nav_down(timers);
                    } else {
                        self.next();
                    }
                    EventResult::Consumed
                },
                TabEvent::Left => {
                    if self.level == HistoryLevel::Bonsai {
                        self.nav_left(timers);
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                },
                TabEvent::Right => {
                    if self.level == HistoryLevel::Bonsai {
                        self.nav_right(timers);
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                },
                TabEvent::Enter => {
                    if self.level == HistoryLevel::Day {
                        self.enter();
                        EventResult::Consumed
                    } else {
                        if let Some(idx) = self.get_selected_timer_index(timers) {
                            EventResult::JumpToForest(idx)
                        } else {
                            EventResult::Consumed
                        }
                    }
                },
                TabEvent::SearchStart => {
                    self.is_searching = true;
                    self.search_query.clear();
                    self.search_matches.clear();
                    self.level = HistoryLevel::Day;
                    EventResult::Consumed
                },
                TabEvent::SearchNext => {
                    if !self.search_matches.is_empty() {
                        self.search_index = (self.search_index + 1) % self.search_matches.len();
                        let (d, b) = self.search_matches[self.search_index];
                        self.selected_day = d;
                        self.selected_bonsai = b;
                        self.level = HistoryLevel::Bonsai;
                    }
                    EventResult::Consumed
                },
                TabEvent::Esc => {
                    let mut consumed = false;
                    if !self.search_matches.is_empty() {
                        self.search_matches.clear();
                        consumed = true;
                    }
                    if self.level == HistoryLevel::Bonsai {
                        self.escape();
                        consumed = true;
                    }
                    if consumed { EventResult::Consumed } else { EventResult::Ignored }
                },
                TabEvent::ZoomIn | TabEvent::ZoomOut => EventResult::Ignored,
                _ => EventResult::Ignored,
            }
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
        self.level = HistoryLevel::Bonsai;
        self.selected_bonsai = 0;
    }

    pub fn escape(&mut self) {
        self.level = HistoryLevel::Day;
    }

    pub fn get_selected_timer_index(&self, timers: &[TimerSession]) -> Option<usize> {
        if self.cached_days.is_empty() { return None; }
        let selected_idx = self.selected_day;
        if selected_idx >= self.cached_days.len() { return None; }
        let date_str = &self.cached_days[selected_idx];
        
        let mut count = 0;
        for (i, t) in timers.iter().enumerate() {
            if t.state.is_none() {
                let t_date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                    Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                    Err(_) => t.start_time.clone(),
                };
                if t_date_str == *date_str {
                    if count == self.selected_bonsai {
                        return Some(i);
                    }
                    count += 1;
                }
            }
        }
        None
    }

    pub fn select_timer(&mut self, timer_idx: usize, timers: &[TimerSession]) {
        if timer_idx >= timers.len() { return; }
        let t = &timers[timer_idx];
        let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => t.start_time.clone(),
        };
        
        if let Some(day_pos) = self.cached_days.iter().position(|d| d == &date_str) {
            self.selected_day = day_pos;
            self.level = HistoryLevel::Bonsai;
            
            let mut count = 0;
            for (i, timer) in timers.iter().enumerate() {
                if timer.state.is_none() {
                    let d = match chrono::DateTime::parse_from_rfc3339(&timer.start_time) {
                        Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                        Err(_) => timer.start_time.clone(),
                    };
                    if d == date_str {
                        if i == timer_idx {
                            self.selected_bonsai = count;
                            break;
                        }
                        count += 1;
                    }
                }
            }
        }
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
        if self.level != HistoryLevel::Bonsai { return; }
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
        if self.level != HistoryLevel::Bonsai { return; }
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
        if self.level != HistoryLevel::Bonsai { return; }
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
        if self.level != HistoryLevel::Bonsai { return; }
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
