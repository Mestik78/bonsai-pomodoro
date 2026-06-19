use super::canvas::{BonsaiCanvas, BonsaiCell, ElementType};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use crate::models::plant::Fruit;

pub fn generate_bamboo(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    if progress <= 0.0 { return canvas; }
    
    let mut rng = StdRng::seed_from_u64(seed);
    
    let num_stalks = rng.gen_range(4..=9);
    let mut all_pixels = Vec::new();
    
    for _ in 0..num_stalks {
        let x = rng.gen_range(-5..=5);
        let life = rng.gen_range(8..=16);
        bamboo_stalk(&mut all_pixels, &mut rng, x, 0, life);
    }
    
    // Sort by Y (which is negative, so greater Y is closer to ground). 
    // We want to draw from ground up, so sort by Y descending
    all_pixels.sort_by_key(|p| -p.1);
    
    let total_pixels = all_pixels.len();
    let pixels_to_draw = (total_pixels as f32 * progress) as usize;
    
    for i in 0..pixels_to_draw {
        let p = &all_pixels[i];
        canvas.cells.insert((p.0, p.1), BonsaiCell {
            content: p.2.clone(),
            color: p.3,
            element_type: p.4.clone(),
        });
    }
    
    canvas
}

fn bamboo_stalk(pixels: &mut Vec<(i32, i32, String, Color, ElementType)>, rng: &mut StdRng, mut x: i32, mut y: i32, life: i32) {
    let green1 = Color::Rgb(34, 139, 34);
    let green2 = Color::Rgb(0, 100, 0);
    
    for i in 0..life {
        let color = if i % 2 == 0 { green1 } else { green2 };
        
        let c = if i > 0 && i % 3 == 0 { "=" } else { "|" };
        pixels.push((x, y, c.to_string(), color, ElementType::Trunk));
        
        if i > 3 && rng.gen_bool(0.3) {
            let dir = if rng.gen_bool(0.5) { 1 } else { -1 };
            let leaf_x = x + dir;
            let leaf_y = y - 1;
            let leaf_c = if dir == 1 { "~" } else { "-" };
            pixels.push((leaf_x, leaf_y, leaf_c.to_string(), Color::Rgb(50, 205, 50), ElementType::Leaf));
        }
        
        if i > life / 2 && rng.gen_bool(0.1) {
            x += if rng.gen_bool(0.5) { 1 } else { -1 };
        }
        
        y -= 1;
    }
}
