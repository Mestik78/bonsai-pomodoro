use crate::models::plant::PlantType;
use crate::models::tabs::{TabEvent, EventResult};

pub struct PlantsState {
    pub is_selecting: bool,
    pub selected_index: usize,
    pub cols: usize,
    pub seed: u64,
    pub is_animating: bool,
    pub animation_progress: f32,
    pub last_tick: Option<std::time::Instant>,
    pub plant_types: Vec<(String, PlantType)>,
    pub scroll_y: usize,
}

impl PlantsState {
    pub fn new() -> Self {
        Self {
            is_selecting: false,
            selected_index: 0,
            cols: 3,
            seed: rand::random(),
            is_animating: false,
            animation_progress: 1.0,
            last_tick: None,
            plant_types: PlantType::all().into_iter().map(|pt| (pt.to_string(), pt)).collect(),
            scroll_y: 0,
        }
    }

    pub fn toggle_animation(&mut self) {
        if self.is_animating {
            self.is_animating = false;
            self.animation_progress = 1.0;
            self.last_tick = None;
        } else {
            self.is_animating = true;
            self.animation_progress = 0.0;
            self.last_tick = Some(std::time::Instant::now());
        }
    }

    pub fn tick(&mut self) {
        if self.is_animating {
            if let Some(last) = self.last_tick {
                let now = std::time::Instant::now();
                let delta = now.duration_since(last).as_secs_f32();
                self.last_tick = Some(now);
                
                self.animation_progress += delta / 5.0;
                if self.animation_progress >= 1.0 {
                    self.animation_progress = 1.0;
                    self.is_animating = false;
                    self.last_tick = None;
                }
            } else {
                self.last_tick = Some(std::time::Instant::now());
            }
        }
    }

    pub fn handle_event(&mut self, event: &TabEvent) -> EventResult {
        match event {
            TabEvent::Enter => {
                if !self.is_selecting {
                    self.is_selecting = true;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::ZoomIn { .. } | TabEvent::ZoomOut { .. } => EventResult::Ignored,
            TabEvent::Esc => {
                if self.is_selecting {
                    self.is_selecting = false;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Up { .. } => {
                if self.is_selecting {
                    self.nav_up();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Down { .. } => {
                if self.is_selecting {
                    self.nav_down();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Left => {
                if self.is_selecting {
                    self.nav_left();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            TabEvent::Right => {
                if self.is_selecting {
                    self.nav_right();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            },
            _ => EventResult::Ignored,
        }
    }

    fn nav_left(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.plant_types.len() - 1;
        }
    }

    fn nav_right(&mut self) {
        if self.selected_index + 1 < self.plant_types.len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    fn nav_up(&mut self) {
        let cols = self.cols.max(1);
        if self.selected_index >= cols {
            self.selected_index -= cols;
        } else {
            // If we are at the top row, we could either do nothing or wrap to bottom
            // Wrap to bottom is complex if the last row isn't full, but we'll implement simple wrapping
            let count = self.plant_types.len();
            let mut target = self.selected_index;
            while target + cols < count {
                target += cols;
            }
            self.selected_index = target;
        }
    }

    fn nav_down(&mut self) {
        let cols = self.cols.max(1);
        let count = self.plant_types.len();
        let target = self.selected_index + cols;
        if target < count {
            self.selected_index = target;
        } else {
            self.selected_index %= cols;
        }
    }
}
