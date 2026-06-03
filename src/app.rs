use std::time::Instant;

use crate::models::history::HistoryState;
use crate::models::forest::ForestState;
use crate::models::stats::StatsState;
use crate::models::plants::PlantsState;
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
    pub timer_plant_list_state: ratatui::widgets::ListState,
    pub is_selecting_plant: bool,
    pub last_tick: Instant,
    pub mode: AppMode,
    pub is_production: bool,
    pub global_seed: u64,
    pub history: HistoryState,
    pub forest: ForestState,
    pub stats: StatsState,
    pub plants: PlantsState,
}

impl App {
    pub fn new(is_production: bool) -> Self {
        let state = AppState::load(is_production);
        let global_seed = state.global_seed;
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
            timers.insert(0, TimerSession::new(new_duration, crate::models::plant::PlantType::Bonsai));
        }

        let finished_count = timers.iter().filter(|t| t.state.is_none()).count();
        let mut history = HistoryState::new();
        history.update_cache(&timers);
        if finished_count > 0 {
            history.list_state.select(Some(0));
            history.selected_day = 0;
        }

        let stats = StatsState::new();
        let plants = PlantsState::new();

        let mut timer_plant_list_state = ratatui::widgets::ListState::default();
        timer_plant_list_state.select(Some(0));

        let mut tab_titles = vec!["Timer", "History", "Forest", "Stats"];
        if !is_production {
            tab_titles.push("Plants");
        }
        
        let mut forest = ForestState::new();
        forest.rebuild_map(&timers, global_seed);

        Self {
            current_tab: Tab::Timer,
            should_quit: false,
            tab_titles,
            timers,
            timer_plant_list_state,
            is_selecting_plant: false,
            last_tick: Instant::now(),
            mode: AppMode::Normal,
            is_production,
            global_seed,
            history,
            forest,
            stats,
            plants,
        }
    }

    pub fn save_state(&self) {
        let state = AppState {
            timers: self.timers.clone(),
            global_seed: self.global_seed,
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
            let selected_idx = self.timer_plant_list_state.selected().unwrap_or(0);
            let selected_plant = crate::models::plant::PlantType::all()[selected_idx].clone();
            self.timers.insert(0, TimerSession::new(duration, selected_plant));
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
        let selected_idx = self.timer_plant_list_state.selected().unwrap_or(0);
        let selected_plant = crate::models::plant::PlantType::all()[selected_idx].clone();
        self.timers[0].plant_type = selected_plant;
        self.timers[0].seed = rand::random();
    }

    pub fn finish_early(&mut self) {
        if self.timers[0].finish_early() {
            self.mode = AppMode::PostTimerInput {
                title: String::new(),
                description: String::new(),
                focus: 0,
            };
            self.save_state();
            self.history.update_cache(&self.timers);
            self.forest.rebuild_map(&self.timers, self.global_seed);
        }
    }

    pub fn on_tick(&mut self) {
        self.plants.tick();
        if self.timers[0].tick(&mut self.last_tick, self.is_production) {
            self.mode = AppMode::PostTimerInput {
                title: String::new(),
                description: String::new(),
                focus: 0,
            };
            self.save_state();
            self.history.update_cache(&self.timers);
            self.forest.rebuild_map(&self.timers, self.global_seed);
        }
    }

    pub fn next_tab(&mut self) {
        if self.current_tab == crate::models::tabs::Tab::History {
            self.history.escape();
        }
        self.current_tab = self.current_tab.next(self.is_production);
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == crate::models::tabs::Tab::History {
            self.history.escape();
        }
        self.current_tab = self.current_tab.previous(self.is_production);
    }

    pub fn set_tab(&mut self, index: usize) {
        let num_tabs = if self.is_production { 4 } else { 5 };
        if index < num_tabs {
            let next_tab = match index {
                0 => crate::models::tabs::Tab::Timer,
                1 => crate::models::tabs::Tab::History,
                2 => crate::models::tabs::Tab::Forest,
                3 => crate::models::tabs::Tab::Stats,
                4 => crate::models::tabs::Tab::Plants,
                _ => self.current_tab,
            };
            if self.current_tab == crate::models::tabs::Tab::History && next_tab != crate::models::tabs::Tab::History {
                self.history.escape();
            }
            self.current_tab = next_tab;
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
        let selected_idx = self.timer_plant_list_state.selected().unwrap_or(0);
        let selected_plant = crate::models::plant::PlantType::all()[selected_idx].clone();
        self.timers.insert(0, TimerSession::new(new_duration, selected_plant));
        
        self.save_state();
        self.mode = AppMode::Normal;
    }


    // Dead code removed

    pub fn handle_timer_event(&mut self, event: &crate::models::tabs::TabEvent) -> crate::models::tabs::EventResult {
        use crate::models::tabs::{TabEvent, EventResult};
        match event {
            TabEvent::Enter | TabEvent::Esc => {
                if self.is_selecting_plant {
                    self.is_selecting_plant = false;
                    return EventResult::Consumed;
                }
                EventResult::Ignored
            },
            TabEvent::Up { is_ctrl } => {
                if !self.is_selecting_plant {
                    if *is_ctrl {
                        self.add_seconds(1);
                    } else {
                        self.add_minutes(1);
                    }
                } else {
                    let i = match self.timer_plant_list_state.selected() {
                        Some(i) => if i == 0 { crate::models::plant::PlantType::all().len() - 1 } else { i - 1 },
                        None => 0,
                    };
                    self.timer_plant_list_state.select(Some(i));
                    
                    if self.timers[0].state == Some(crate::models::timer::TimerState::New) {
                        self.timers[0].plant_type = crate::models::plant::PlantType::all()[i].clone();
                        self.timers[0].seed = rand::random();
                    }
                }
                EventResult::Consumed
            },
            TabEvent::Down { is_ctrl } => {
                if !self.is_selecting_plant {
                    if *is_ctrl {
                        self.add_seconds(-1);
                    } else {
                        self.add_minutes(-1);
                    }
                } else {
                    let i = match self.timer_plant_list_state.selected() {
                        Some(i) => if i >= crate::models::plant::PlantType::all().len() - 1 { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.timer_plant_list_state.select(Some(i));
                    
                    if self.timers[0].state == Some(crate::models::timer::TimerState::New) {
                        self.timers[0].plant_type = crate::models::plant::PlantType::all()[i].clone();
                        self.timers[0].seed = rand::random();
                    }
                }
                EventResult::Consumed
            },
            _ => EventResult::Ignored
        }
    }

    pub fn dispatch_event(&mut self, event: crate::models::tabs::TabEvent) -> crate::models::tabs::EventResult {
        use crate::models::tabs::{Tab, EventResult};
        match self.current_tab {
            Tab::Timer => self.handle_timer_event(&event),
            Tab::History => self.history.handle_event(&event, &self.timers),
            Tab::Forest => self.forest.handle_event(&event),
            Tab::Stats => self.stats.handle_event(&event),
            Tab::Plants => self.plants.handle_event(&event),
        }
    }
}
