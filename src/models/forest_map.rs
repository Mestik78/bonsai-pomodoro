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

impl ForestMap {
    pub fn build(timers: &[TimerSession], global_seed: u64) -> Self {
        let mut map = ForestMap {
            grid: HashMap::new(),
        };

        // Agrupar timers por día
        // Vec<(fecha, Vec<index_en_timers>)>
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
        
        // Ordenar los días por fecha (alfabético en YYYY-MM-DD funciona)
        days.sort_by(|a, b| a.0.cmp(&b.0));
        
        if days.is_empty() {
            return map; // Mapa vacío
        }
        
        let mut rng = StdRng::seed_from_u64(global_seed);
        
        let mut current_blob_x = 0;
        let mut current_blob_y = 0;
        
        for (day_idx, (_, timer_indices)) in days.iter().enumerate() {
            // Posicionar las plantas del día actual en espiral alrededor de current_blob_x, current_blob_y
            // Espiral simple: 0,0 luego derecha, arriba, izquierda, izquierda, abajo, abajo...
            let mut dx = 0;
            let mut dy = -1;
            let mut rx = 0;
            let mut ry = 0;
            
            // Para poder dibujar caminos alrededor del blob, guardamos los límites
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
            
            // Rodear el blob actual con caminos
            for x in (min_x - 1)..=(max_x + 1) {
                for y in (min_y - 1)..=(max_y + 1) {
                    map.grid.entry((x, y)).or_insert(MapElement::Path);
                }
            }
            
            // Decidir la posición del siguiente blob (Random Walk)
            if day_idx < days.len() - 1 {
                let direction = rng.gen_range(0..4);
                let jump_distance = rng.gen_range(5..=10);
                
                let next_x = match direction {
                    0 => current_blob_x + jump_distance, // Este
                    1 => current_blob_x - jump_distance, // Oeste
                    _ => current_blob_x,
                };
                
                let next_y = match direction {
                    2 => current_blob_y + jump_distance, // Norte
                    3 => current_blob_y - jump_distance, // Sur
                    _ => current_blob_y,
                };
                
                // Trazar un camino Manhattan desde current al next
                let mut path_x = current_blob_x;
                let mut path_y = current_blob_y;
                
                while path_x != next_x {
                    path_x += if next_x > path_x { 1 } else { -1 };
                    map.grid.entry((path_x, path_y)).or_insert(MapElement::Path);
                }
                while path_y != next_y {
                    path_y += if next_y > path_y { 1 } else { -1 };
                    map.grid.entry((path_x, path_y)).or_insert(MapElement::Path);
                }
                
                current_blob_x = next_x;
                current_blob_y = next_y;
            }
        }

        map
    }
}
