#[derive(Clone)]
pub struct ZoomLevel {
    pub tile_width: u16,
    pub tile_height: u16,
}

#[derive(Clone)]
pub struct Tilemap {
    pub zoom_levels: Vec<ZoomLevel>,
    pub current_zoom: usize,
    pub camera_x: i32,
    pub camera_y: i32,
}

impl Default for Tilemap {
    fn default() -> Self {
        Self::new()
    }
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            zoom_levels: vec![
                ZoomLevel { tile_width: 43, tile_height: 20 },
                ZoomLevel { tile_width: 20, tile_height: 10 },
                ZoomLevel { tile_width: 10, tile_height: 5 },
            ],
            current_zoom: 1, // Medium
            camera_x: 0,
            camera_y: 0,
        }
    }

    pub fn zoom_in(&mut self) {
        if self.current_zoom > 0 {
            let old_tw = self.zoom_levels[self.current_zoom].tile_width as f32;
            let old_th = self.zoom_levels[self.current_zoom].tile_height as f32;
            self.current_zoom -= 1;
            let new_tw = self.zoom_levels[self.current_zoom].tile_width as f32;
            let new_th = self.zoom_levels[self.current_zoom].tile_height as f32;
            self.camera_x = ((self.camera_x as f32 / old_tw) * new_tw).round() as i32;
            self.camera_y = ((self.camera_y as f32 / old_th) * new_th).round() as i32;
        }
    }

    pub fn zoom_out(&mut self) {
        if self.current_zoom < self.zoom_levels.len() - 1 {
            let old_tw = self.zoom_levels[self.current_zoom].tile_width as f32;
            let old_th = self.zoom_levels[self.current_zoom].tile_height as f32;
            self.current_zoom += 1;
            let new_tw = self.zoom_levels[self.current_zoom].tile_width as f32;
            let new_th = self.zoom_levels[self.current_zoom].tile_height as f32;
            self.camera_x = ((self.camera_x as f32 / old_tw) * new_tw).round() as i32;
            self.camera_y = ((self.camera_y as f32 / old_th) * new_th).round() as i32;
        }
    }

    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.camera_x += dx;
        self.camera_y += dy;
    }
}
