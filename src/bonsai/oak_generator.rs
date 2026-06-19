use super::canvas::{BonsaiCanvas, BonsaiCell, ElementType};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use crate::models::plant::Fruit;

pub fn generate_oak(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let leaves_color = vec![Color::Rgb(34, 139, 34), Color::Rgb(0, 100, 0), Color::Rgb(85, 107, 47)];
    let chars = vec!["&", "*", "%", "#", "@"];
    let trunk_color = Color::Rgb(120, 80, 50);
    let trunk_chars = vec!["|", "|", "|"];
    generate_tree_with_params(seed, progress, leaves_color, chars, trunk_color, trunk_chars, 7, 7)
}

pub fn generate_red_maple(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let leaves_color = vec![Color::Rgb(220, 20, 60), Color::Rgb(255, 69, 0), Color::Rgb(139, 0, 0), Color::Rgb(255, 140, 0)];
    let chars = vec!["*", "&", "v", "#"];
    let trunk_color = Color::Rgb(101, 67, 33);
    let trunk_chars = vec!["|", "|", "|"];
    generate_tree_with_params(seed, progress, leaves_color, chars, trunk_color, trunk_chars, 7, 7)
}

pub fn generate_white_birch(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let leaves_color = vec![Color::Rgb(154, 205, 50), Color::Rgb(255, 215, 0), Color::Rgb(173, 255, 47)];
    let chars = vec!["~", "*", "v"];
    let trunk_color = Color::Rgb(220, 220, 220); // Off-white
    let trunk_chars = vec!["|", "¦", "|"]; // Birch texture
    generate_tree_with_params(seed, progress, leaves_color, chars, trunk_color, trunk_chars, 7, 7)
}

fn generate_tree_with_params(seed: u64, progress: f32, leaves_color: Vec<Color>, chars: Vec<&str>, trunk_color: Color, trunk_chars: Vec<&str>, life: i32, radius: i32) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    if progress <= 0.0 { return canvas; }
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut color_rng = StdRng::seed_from_u64(seed.wrapping_add(1));
    
    // Smooth pushing up effect: the whole tree grows visually up
    // Generate full tree at target size, but shift it down
    let shift_y = (life as f32 * (1.0 - progress)).round() as i32;
    
    let mut all_pixels = Vec::new();
    oak_tree(&mut all_pixels, &mut rng, &mut color_rng, 0, shift_y, life, radius, &leaves_color, &chars, trunk_color, &trunk_chars);
    
    // Sort canopy pixels by distance from center to grow outwards, trunk pixels by Y to grow upwards
    // Trunk elements get a negative distance so they draw first
    all_pixels.sort_by_key(|p| p.4);
    
    let total_pixels = all_pixels.len();
    let pixels_to_draw = (total_pixels as f32 * progress) as usize;
    
    for i in 0..pixels_to_draw {
        let p = &all_pixels[i];
        canvas.cells.insert((p.0, p.1), BonsaiCell {
            content: p.2.clone(),
            color: p.3,
            element_type: p.5.clone()
        });
    }
    
    canvas
}

fn oak_tree(pixels: &mut Vec<(i32, i32, String, Color, i32, ElementType)>, rng: &mut StdRng, color_rng: &mut StdRng, x: i32, mut y: i32, life: i32, radius: i32, leaves_color: &[Color], chars: &[&str], trunk_color: Color, trunk_chars: &[&str]) {
    for i in 0..life {
        let mut cur_x = x - (trunk_chars.len() as i32) / 2;
        for c in trunk_chars {
            pixels.push((cur_x, y, c.to_string(), trunk_color, -1000 + i, ElementType::Trunk));
            cur_x += 1;
        }
        y -= 1;
    }
    
    oak_canopy(pixels, rng, color_rng, x, y, radius, leaves_color, chars);
}

fn oak_canopy(pixels: &mut Vec<(i32, i32, String, Color, i32, ElementType)>, rng: &mut StdRng, color_rng: &mut StdRng, cx: i32, cy: i32, radius: i32, leaves_color: &[Color], chars: &[&str]) {
    let mut occupied = std::collections::HashSet::new();
    for r in 0..=radius {
        for _ in 0..(r * 8 + 1) {
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let dist = if r > 0 { rng.gen_range(0.0..r as f32) } else { 0.0 };
            let px = cx + (angle.cos() * dist * 1.5) as i32;
            let py = cy + (angle.sin() * dist) as i32;
            
            if !occupied.contains(&(px, py)) {
                occupied.insert((px, py));
                let dist_sq = ((px - cx)*(px - cx) + (py - cy)*(py - cy)) as i32;
                pixels.push((
                    px, py, 
                    chars[color_rng.gen_range(0..chars.len())].to_string(),
                    leaves_color[color_rng.gen_range(0..leaves_color.len())],
                    dist_sq,
                    ElementType::Leaf
                ));
            }
        }
    }
}
