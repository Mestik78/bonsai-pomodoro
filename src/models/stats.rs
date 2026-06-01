use ratatui::widgets::ListState;
use crate::models::tabs::{TabEvent, EventResult};

pub struct StatsState {
    pub list_state: ListState,
}

impl StatsState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }

    pub fn handle_event(&mut self, event: &TabEvent) -> EventResult {
        match event {
            TabEvent::Up { .. } => {
                self.previous();
                EventResult::Consumed
            },
            TabEvent::Down { .. } => {
                self.next();
                EventResult::Consumed
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
                if i >= 3 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { 3 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
