use std::time::Instant;

use crate::models::forest::ForestState;
use crate::models::stats::StatsState;
use crate::models::tabs::Tab;

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
    pub current_tab: Tab,
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
        forest.update_cache(&timers);
        if finished_count > 0 {
            forest.list_state.select(Some(0));
            forest.selected_day = 0;
        }

        let stats = StatsState::new();

        Self {
            current_tab: Tab::Timer,
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
        let timers = self.timers.clone();
        let is_production = self.is_production;
        std::thread::spawn(move || {
            let state = AppState { timers };
            state.save(is_production);
        });
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
            self.forest.update_cache(&self.timers);
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
            self.forest.update_cache(&self.timers);
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
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


    // Dead code removed

    pub fn handle_timer_event(&mut self, event: &crate::models::tabs::TabEvent) -> crate::models::tabs::EventResult {
        use crate::models::tabs::{TabEvent, EventResult};
        match event {
            TabEvent::Up { is_ctrl } => {
                if *is_ctrl {
                    self.add_seconds(1);
                } else {
                    self.add_minutes(1);
                }
                EventResult::Consumed
            },
            TabEvent::Down { is_ctrl } => {
                if *is_ctrl {
                    self.add_seconds(-1);
                } else {
                    self.add_minutes(-1);
                }
                EventResult::Consumed
            },
            _ => EventResult::Ignored
        }
    }

    pub fn dispatch_event(&mut self, event: crate::models::tabs::TabEvent) {
        use crate::models::tabs::{Tab, EventResult};
        let result = match self.current_tab {
            Tab::Timer => self.handle_timer_event(&event),
            Tab::Forest => self.forest.handle_event(&event, &self.timers),
            Tab::Stats => self.stats.handle_event(&event),
        };

        if let EventResult::Ignored = result {
            match event {
                crate::models::tabs::TabEvent::Left => self.previous_tab(),
                crate::models::tabs::TabEvent::Right => self.next_tab(),
                _ => {}
            }
        }
    }
}
