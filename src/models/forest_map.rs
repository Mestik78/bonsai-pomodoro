use std::collections::HashMap;
use crate::models::timer::TimerSession;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[derive(Clone, Copy, PartialEq)]
pub enum MapElement {
    Empty,
    Path,
    Plant(usize), // Almacenamos el índice del timer en el vector original
}

#[derive(Clone, Default)]
pub struct ForestMap {
    pub grid: HashMap<(i32, i32), MapElement>,
}

struct BlobInfo {
    cx: i32,
    cy: i32,
    r: i32,
}

impl ForestMap {
    pub fn build(timers: &[TimerSession], global_seed: u64) -> Self {
        let mut map = ForestMap {
            grid: HashMap::new(),
        };

        let mut days: Vec<(String, Vec<usize>)> = Vec::new();
        for (i, t) in timers.iter().enumerate() {
            if t.state.is_none() {
                let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                    Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                    Err(_) => t.start_time.clone(),
                };
                if let Some(pos) = days.iter().position(|(d, _)| *d == date_str) {
                    days[pos].1.push(i);
                } else {
                    days.push((date_str, vec![i]));
                }
            }
        }
        
        days.sort_by(|a, b| a.0.cmp(&b.0));
        if days.is_empty() { return map; }
        
        let mut rng = StdRng::seed_from_u64(global_seed);
        let mut placed_blobs: Vec<BlobInfo> = Vec::new();
        
        for (day_idx, (_, timer_indices)) in days.iter().enumerate() {
            let num_plants = timer_indices.len() as f32;
            let r_new = num_plants.sqrt().ceil() as i32 + 2;
            
            let mut current_blob_x = 0;
            let mut current_blob_y = 0;
            
            if day_idx > 0 {
                let mut placed = false;
                let mut tries = 0;
                let mut extra_distance = 0;
                
                let mut parent_cx = 0;
                let mut parent_cy = 0;
                
                while !placed {
                    let parent_idx = rng.gen_range(0..day_idx);
                    let parent = &placed_blobs[parent_idx];
                    
                    let theta: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                    let d = rng.gen_range(2..=5) + extra_distance;
                    let target_r = parent.r + r_new + d;
                    
                    let test_cx = parent.cx + (target_r as f32 * theta.cos()).round() as i32;
                    let test_cy = parent.cy + (target_r as f32 * theta.sin()).round() as i32;
                    
                    let mut collides = false;
                    for blob in &placed_blobs {
                        let dist_sq = (test_cx - blob.cx).pow(2) + (test_cy - blob.cy).pow(2);
                        let min_dist = blob.r + r_new;
                        if dist_sq <= min_dist.pow(2) {
                            collides = true;
                            break;
                        }
                    }
                    
                    if !collides {
                        current_blob_x = test_cx;
                        current_blob_y = test_cy;
                        parent_cx = parent.cx;
                        parent_cy = parent.cy;
                        placed = true;
                    } else {
                        tries += 1;
                        if tries > 20 {
                            extra_distance += 2;
                            tries = 0;
                        }
                    }
                }
                
                // Bresenham path
                let mut path_x = parent_cx;
                let mut path_y = parent_cy;
                let dx = (current_blob_x - parent_cx).abs();
                let sx = if parent_cx < current_blob_x { 1 } else { -1 };
                let dy = -(current_blob_y - parent_cy).abs();
                let sy = if parent_cy < current_blob_y { 1 } else { -1 };
                let mut err = dx + dy;
                
                loop {
                    map.grid.entry((path_x, path_y)).or_insert(MapElement::Path);
                    if path_x == current_blob_x && path_y == current_blob_y { break; }
                    let e2 = 2 * err;
                    if e2 >= dy {
                        err += dy;
                        path_x += sx;
                    }
                    if e2 <= dx {
                        err += dx;
                        path_y += sy;
                    }
                }
            }
            
            placed_blobs.push(BlobInfo { cx: current_blob_x, cy: current_blob_y, r: r_new });
            
            let mut dx = 0;
            let mut dy = -1;
            let mut rx = 0;
            let mut ry = 0;
            
            let mut min_x = current_blob_x;
            let mut max_x = current_blob_x;
            let mut min_y = current_blob_y;
            let mut max_y = current_blob_y;
            
            for (idx, &timer_idx) in timer_indices.iter().enumerate() {
                if idx > 0 {
                    if rx == ry || (rx < 0 && rx == -ry) || (rx > 0 && rx == 1 - ry) {
                        let t = dx;
                        dx = -dy;
                        dy = t;
                    }
                    rx += dx;
                    ry += dy;
                }
                let px = current_blob_x + rx;
                let py = current_blob_y + ry;
                map.grid.insert((px, py), MapElement::Plant(timer_idx));
                min_x = min_x.min(px);
                max_x = max_x.max(px);
                min_y = min_y.min(py);
                max_y = max_y.max(py);
            }
            
            for x in (min_x - 1)..=(max_x + 1) {
                for y in (min_y - 1)..=(max_y + 1) {
                    map.grid.entry((x, y)).or_insert(MapElement::Path);
                }
            }
        }

        map
    }
}
