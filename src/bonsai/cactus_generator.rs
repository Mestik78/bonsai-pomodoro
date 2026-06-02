use super::canvas::{BonsaiCanvas, BonsaiCell};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::models::plant::Fruit;

fn hash_2d(seed: u64, x: i32, y: i32) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    x.hash(&mut hasher);
    y.hash(&mut hasher);
    hasher.finish()
}

pub fn generate_cactus(seed: u64, progress: f32, fruit: Option<Fruit>, fruit_quantity: u32) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    canvas.trunk_parts = None;
    if progress <= 0.0 {
        return canvas;
    }
    
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Main body properties
    let h = rng.gen_range(5..=9) as i32;
    let w = rng.gen_range(8..=14) as i32; // half-width
    
    let current_h = (h as f32 * progress).ceil() as i32;
    let current_w = (w as f32 * progress).ceil() as i32;
    
    // Pups (hijos)
    let num_pups = rng.gen_range(0..=2);
    let mut pups = Vec::new();
    for _ in 0..num_pups {
        let pup_h = rng.gen_range(2..=4);
        let pup_w = rng.gen_range(3..=6);
        let pup_x = if rng.gen_bool(0.5) { rng.gen_range(-w..-w/2) } else { rng.gen_range(w/2..w) };
        pups.push((pup_x, pup_h, pup_w));
    }
    
    if current_h == 0 || current_w == 0 {
        return canvas;
    }
    
    // Draw pups first
    for &(pup_x, pup_h, pup_w) in &pups {
        let pup_current_h = (pup_h as f32 * progress).ceil() as i32;
        let pup_current_w = (pup_w as f32 * progress).ceil() as i32;
        let pup_current_x = (pup_x as f32 * progress).round() as i32;
        for y in 0..=pup_current_h {
            draw_ellipse_row(&mut canvas, seed, pup_current_x, y, pup_current_h, pup_current_w, &fruit, fruit_quantity);
        }
    }
    
    // Draw main body
    for y in 0..=current_h {
        draw_ellipse_row(&mut canvas, seed, 0, y, current_h, current_w, &fruit, fruit_quantity);
    }
    
    canvas
}

fn draw_ellipse_row(canvas: &mut BonsaiCanvas, seed: u64, offset_x: i32, y: i32, h: i32, w: i32, fruit: &Option<Fruit>, fruit_quantity: u32) {
    if h <= 0 || w <= 0 { return; }
    
    // Adding +1.0 to the divisor ensures it doesn't get infinitely sharp at the top/bottom
    // It creates a nice flat top and bottom, simulating the ground and the crown of the cactus.
    let y_factor = (y as f32 - h as f32 / 2.0) / (h as f32 / 2.0 + 1.0); 
    let inner_sqrt = (1.0 - y_factor * y_factor).max(0.0);
    let mut x_max = (w as f32 * inner_sqrt.sqrt()).round() as i32;
    
    // Organic noise to edges
    if y > 0 && y < h {
        let row_hash = hash_2d(seed, offset_x, y);
        let noise = (row_hash % 3) as i32 - 1;
        x_max += noise;
    }
    x_max = x_max.max(1);
    
    for x in -x_max..=x_max {
        let is_edge = x == -x_max || x == x_max || y == h || y == 0;
        
        let cell_hash = hash_2d(seed, offset_x + x, y);
        let color_dice = cell_hash % 100;
        let mut color = if color_dice < 30 { Color::LightGreen } else if color_dice < 40 { Color::DarkGray } else { Color::Green };
        
        let s = if is_edge {
            if x == -x_max || x == x_max {
                if (cell_hash / 100) % 10 < 4 { "." } else { "|" }
            } else if y == h {
                if (cell_hash / 100) % 10 < 4 { "." } else { "_" }
            } else {
                "_"
            }
        } else {
            let chars = ["*", "&", "#", "%", "@", ":"];
            chars[((cell_hash / 100) % chars.len() as u64) as usize]
        };
        
        let mut final_s = s.to_string();
        
        if let Some(f) = fruit {
            if hash_2d(seed.wrapping_add(1), offset_x + x, y) % 100 < fruit_quantity as u64 {
                final_s = f.character.to_string();
                color = f.color;
            }
        }
        
        // We negate y because in the canvas, negative y goes UP.
        canvas.cells.insert((offset_x + x, -y), BonsaiCell {
            content: final_s,
            color,
        });
    }
}
