use serde::{Deserialize, Serialize};
use crate::models::tilemap::Tilemap;
use crate::models::forest_map::ForestMap;
use crate::models::tabs::{TabEvent, EventResult};

#[derive(Serialize, Deserialize, Clone)]
pub struct ForestState {
    #[serde(skip)]
    pub tilemap: Tilemap,
    #[serde(skip)]
    pub is_moving: bool,
    #[serde(skip)]
    pub forest_map: ForestMap,
}

impl ForestState {
    pub fn new() -> Self {
        Self {
            tilemap: Tilemap::new(),
            is_moving: false,
            forest_map: ForestMap::default(),
        }
    }
    
    pub fn rebuild_map(&mut self, timers: &[crate::models::timer::TimerSession], global_seed: u64) {
        self.forest_map = ForestMap::build(timers, global_seed);
    }

    pub fn handle_event(&mut self, event: &TabEvent) -> EventResult {
        if self.is_moving {
            match event {
                TabEvent::Up { .. } => { self.tilemap.pan(0, 1); EventResult::Consumed },
                TabEvent::Down { .. } => { self.tilemap.pan(0, -1); EventResult::Consumed },
                TabEvent::Left => { self.tilemap.pan(-2, 0); EventResult::Consumed },
                TabEvent::Right => { self.tilemap.pan(2, 0); EventResult::Consumed },
                TabEvent::ZoomIn => { self.tilemap.zoom_in(); EventResult::Consumed },
                TabEvent::ZoomOut => { self.tilemap.zoom_out(); EventResult::Consumed },
                TabEvent::Esc => { self.is_moving = false; EventResult::Consumed },
                _ => EventResult::Ignored,
            }
        } else {
            match event {
                TabEvent::Enter => { self.is_moving = true; EventResult::Consumed },
                TabEvent::ZoomIn => { self.tilemap.zoom_in(); EventResult::Consumed },
                TabEvent::ZoomOut => { self.tilemap.zoom_out(); EventResult::Consumed },
                _ => EventResult::Ignored,
            }
        }
    }
}

impl Default for ForestState {
    fn default() -> Self {
        Self::new()
    }
}
