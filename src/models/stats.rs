use ratatui::widgets::ListState;
use crate::models::tabs::{TabEvent, EventResult};

pub struct StatsState {
    pub list_state: ListState,
    pub scroll_offset: usize,
    pub is_movement_mode: bool,
}

impl StatsState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state, scroll_offset: 0, is_movement_mode: false }
    }

    pub fn handle_event(&mut self, event: &TabEvent) -> EventResult {
        match event {
            TabEvent::Enter => {
                if self.selected() != Some(3) && self.selected() != Some(4) {
                    self.is_movement_mode = !self.is_movement_mode;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Esc => {
                if self.is_movement_mode {
                    self.is_movement_mode = false;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Up { .. } => {
                self.previous();
                self.scroll_offset = 0;
                self.is_movement_mode = false;
                EventResult::Consumed
            },
            TabEvent::Down { .. } => {
                self.next();
                self.scroll_offset = 0;
                self.is_movement_mode = false;
                EventResult::Consumed
            },
            TabEvent::Left => {
                if self.is_movement_mode && self.selected() != Some(3) && self.selected() != Some(4) {
                    self.scroll_offset += 1;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Right => {
                if self.is_movement_mode && self.selected() != Some(3) && self.selected() != Some(4) {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            _ => EventResult::Ignored
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= 4 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { 4 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
