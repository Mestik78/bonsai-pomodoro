use std::collections::{HashMap, HashSet};
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
    pub day_blobs: Vec<(String, i32, i32)>,
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
            day_blobs: Vec::new(),
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
        
        let mut rng_topo = StdRng::seed_from_u64(global_seed ^ 0x123456789ABCDEF0);
        let mut rng_blob = StdRng::seed_from_u64(global_seed ^ 0x0FEDCBA987654321);
        let mut rng_path = StdRng::seed_from_u64(global_seed ^ 0x0102030405060708);
        
        let mut placed_blobs: Vec<BlobInfo> = Vec::new();
        
        for (day_idx, (_, timer_indices)) in days.iter().enumerate() {
            let num_plants = timer_indices.len();
            
            // First, let's pre-calculate the shape of the blob starting at (0,0).
            // We use a Random BFS (growth)
            let mut blob_cells = Vec::new();
            blob_cells.push((0, 0));
            
            let mut frontier = Vec::new();
            let mut visited = HashSet::new();
            visited.insert((0, 0));
            
            let add_to_frontier = |x: i32, y: i32, v: &mut HashSet<(i32, i32)>, f: &mut Vec<(i32, i32)>| {
                for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if !v.contains(&(nx, ny)) {
                        v.insert((nx, ny));
                        f.push((nx, ny));
                    }
                }
            };
            
            add_to_frontier(0, 0, &mut visited, &mut frontier);
            
            for _ in 1..num_plants {
                if frontier.is_empty() { break; }
                let f_idx = rng_blob.gen_range(0..frontier.len());
                let cell = frontier.swap_remove(f_idx);
                blob_cells.push(cell);
                add_to_frontier(cell.0, cell.1, &mut visited, &mut frontier);
            }
            
            let mut max_r_sq = 0;
            for &(bx, by) in &blob_cells {
                let r_sq = bx*bx + by*by;
                if r_sq > max_r_sq { max_r_sq = r_sq; }
            }
            let r_new = (max_r_sq as f32).sqrt().ceil() as i32; // Sin padding para mayor compactación
            
            let mut current_blob_x = 0;
            let mut current_blob_y = 0;
            
            if day_idx > 0 {
                let mut placed = false;
                let mut tries = 0;
                let mut extra_distance = 0;
                
                let mut parent_cx = 0;
                let mut parent_cy = 0;
                
                while !placed {
                    let parent_idx = rng_topo.gen_range(0..day_idx);
                    let parent = &placed_blobs[parent_idx];
                    
                    let theta: f32 = rng_topo.gen_range(0.0..std::f32::consts::TAU);
                    let d = 1 + extra_distance;
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
                            extra_distance += 1;
                            tries = 0;
                        }
                    }
                }
                
                // Bresenham over Bezier path
                let p0 = (parent_cx as f32, parent_cy as f32);
                let p3 = (current_blob_x as f32, current_blob_y as f32);
                
                let dx = p3.0 - p0.0;
                let dy = p3.1 - p0.1;
                
                let len = (dx*dx + dy*dy).sqrt().max(1.0);
                let nx = -dy / len;
                let ny = dx / len;
                
                let mag1 = rng_path.gen_range(-15.0..15.0);
                let mag2 = rng_path.gen_range(-15.0..15.0);
                
                let p1 = (p0.0 + dx * 0.33 + nx * mag1, p0.1 + dy * 0.33 + ny * mag1);
                let p2 = (p0.0 + dx * 0.66 + nx * mag2, p0.1 + dy * 0.66 + ny * mag2);
                
                let mut last_px = parent_cx;
                let mut last_py = parent_cy;
                
                let segments = (len * 2.0) as usize; 
                
                for step in 1..=segments {
                    let t = step as f32 / segments as f32;
                    let u = 1.0 - t;
                    let tt = t * t;
                    let uu = u * u;
                    let uuu = uu * u;
                    let ttt = tt * t;
                    
                    let qx = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
                    let qy = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;
                    
                    let target_px = qx.round() as i32;
                    let target_py = qy.round() as i32;
                    
                    let mut bx = last_px;
                    let mut by = last_py;
                    let dx_b = (target_px - bx).abs();
                    let sx_b = if bx < target_px { 1 } else { -1 };
                    let dy_b = -(target_py - by).abs();
                    let sy_b = if by < target_py { 1 } else { -1 };
                    let mut err = dx_b + dy_b;
                    
                    loop {
                        map.grid.entry((bx, by)).or_insert(MapElement::Path);
                        if bx == target_px && by == target_py { break; }
                        let e2 = 2 * err;
                        if e2 >= dy_b {
                            err += dy_b;
                            bx += sx_b;
                        }
                        if e2 <= dx_b {
                            err += dx_b;
                            by += sy_b;
                        }
                    }
                    
                    last_px = target_px;
                    last_py = target_py;
                }
            }
            
            placed_blobs.push(BlobInfo { cx: current_blob_x, cy: current_blob_y, r: r_new });
            map.day_blobs.push((days[day_idx].0.clone(), current_blob_x, current_blob_y));
            
            for (idx, &timer_idx) in timer_indices.iter().enumerate() {
                let cell = blob_cells[idx];
                let px = current_blob_x + cell.0;
                let py = current_blob_y + cell.1;
                map.grid.insert((px, py), MapElement::Plant(timer_idx));
            }
            
            // Rodear las plantas con caminos usando un borde ceñido
            let mut paths_to_add = Vec::new();
            for (idx, _) in timer_indices.iter().enumerate() {
                let cell = blob_cells[idx];
                let px = current_blob_x + cell.0;
                let py = current_blob_y + cell.1;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        paths_to_add.push((px + dx, py + dy));
                    }
                }
            }
            for (px, py) in paths_to_add {
                map.grid.entry((px, py)).or_insert(MapElement::Path);
            }
        }

        map
    }
}
