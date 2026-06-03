use serde::{Deserialize, Serialize};
use crate::models::tilemap::Tilemap;
use crate::models::forest_map::ForestMap;
use crate::models::tabs::{TabEvent, EventResult};

#[derive(Serialize, Deserialize, Clone)]
pub struct ForestState {
    #[serde(skip)]
    pub tilemap: Tilemap,
    #[serde(skip)]
    pub forest_map: ForestMap,
    #[serde(skip)]
    pub is_searching: bool,
    #[serde(skip)]
    pub search_query: String,
    #[serde(skip)]
    pub search_matches: Vec<(i32, i32)>,
    #[serde(skip)]
    pub search_index: usize,
}

impl ForestState {
    pub fn new() -> Self {
        Self {
            tilemap: Tilemap::new(),
            forest_map: ForestMap::default(),
            is_searching: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_index: 0,
        }
    }
    
    pub fn rebuild_map(&mut self, timers: &[crate::models::timer::TimerSession], global_seed: u64) {
        self.forest_map = ForestMap::build(timers, global_seed);
    }

    pub fn handle_event(&mut self, event: &TabEvent) -> EventResult {
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
                        for (date, cx, cy) in &self.forest_map.day_blobs {
                            if date.contains(&self.search_query) {
                                self.search_matches.push((*cx, *cy));
                            }
                        }
                        if !self.search_matches.is_empty() {
                            let (cx, cy) = self.search_matches[0];
                            self.tilemap.camera_x = cx * (self.tilemap.zoom_levels[self.tilemap.current_zoom].tile_width as i32);
                            self.tilemap.camera_y = cy * (self.tilemap.zoom_levels[self.tilemap.current_zoom].tile_height as i32);
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
                TabEvent::Up { .. } => { self.tilemap.pan(0, 1); EventResult::Consumed },
                TabEvent::Down { .. } => { self.tilemap.pan(0, -1); EventResult::Consumed },
                TabEvent::Left => { self.tilemap.pan(-2, 0); EventResult::Consumed },
                TabEvent::Right => { self.tilemap.pan(2, 0); EventResult::Consumed },
                TabEvent::ZoomIn => { self.tilemap.zoom_in(); EventResult::Consumed },
                TabEvent::ZoomOut => { self.tilemap.zoom_out(); EventResult::Consumed },
                TabEvent::Esc => { 
                    self.search_matches.clear();
                    EventResult::Consumed 
                },
                TabEvent::SearchStart => {
                    self.is_searching = true;
                    self.search_query.clear();
                    self.search_matches.clear();
                    EventResult::Consumed
                },
                TabEvent::SearchNext => {
                    if !self.search_matches.is_empty() {
                        self.search_index = (self.search_index + 1) % self.search_matches.len();
                        let (cx, cy) = self.search_matches[self.search_index];
                        self.tilemap.camera_x = cx * (self.tilemap.zoom_levels[self.tilemap.current_zoom].tile_width as i32);
                        self.tilemap.camera_y = cy * (self.tilemap.zoom_levels[self.tilemap.current_zoom].tile_height as i32);
                    }
                    EventResult::Consumed
                },
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
