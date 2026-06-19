use super::canvas::{BonsaiCanvas, BonsaiCell, ElementType};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use crate::models::plant::Fruit;

pub fn generate_mushroom(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let cap_color = Color::Rgb(220, 20, 60); // Crimson red
    let spot_color = Color::White;
    generate_mushroom_with_params(seed, progress, 4..=7, 7..=11, cap_color, spot_color, 0.12)
}

pub fn generate_tall_mushroom(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let cap_color = Color::Rgb(210, 180, 140); // Tan
    let spot_color = Color::Rgb(139, 69, 19); // SaddleBrown spots
    generate_mushroom_with_params(seed, progress, 8..=13, 4..=6, cap_color, spot_color, 0.05)
}

pub fn generate_brown_mushroom(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let cap_color = Color::Rgb(139, 69, 19); // SaddleBrown
    let spot_color = Color::Rgb(205, 133, 63); // Peru
    generate_mushroom_with_params(seed, progress, 3..=5, 6..=9, cap_color, spot_color, 0.2)
}

fn generate_mushroom_with_params(seed: u64, progress: f32, stem_range: std::ops::RangeInclusive<i32>, radius_range: std::ops::RangeInclusive<i32>, cap_color: Color, spot_color: Color, spot_chance: f64) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    if progress <= 0.0 { return canvas; }
    
    let mut rng = StdRng::seed_from_u64(seed);
    
    let base_stem = rng.gen_range(stem_range);
    let base_radius = rng.gen_range(radius_range);
    
    let current_stem = (base_stem as f32 * progress).round() as i32;
    let current_radius = (base_radius as f32 * progress).round() as i32;
    
    if current_stem == 0 && current_radius == 0 { return canvas; }
    
    mushroom_tree(&mut canvas, seed, 0, 0, current_stem, current_radius, cap_color, spot_color, spot_chance);
    
    canvas
}

fn mushroom_tree(canvas: &mut BonsaiCanvas, seed: u64, x: i32, mut y: i32, stem_height: i32, cap_radius: i32, cap_color: Color, spot_color: Color, spot_chance: f64) {
    let stem_color = Color::Rgb(245, 245, 220); // Beige
    
    // Stem
    for i in 0..stem_height {
        canvas.cells.insert((x, y - i), BonsaiCell { content: "|".to_string(), color: stem_color, element_type: ElementType::Trunk });
        canvas.cells.insert((x+1, y - i), BonsaiCell { content: "|".to_string(), color: stem_color, element_type: ElementType::Trunk });
    }
    
    y -= stem_height;
    
    // Cap
    let mut cap_pixels = Vec::new();
    for dy in 0..=cap_radius/2 {
        let dy_f = dy as f32;
        let cap_f = cap_radius as f32;
        let w = ((cap_f * cap_f) - (dy_f * 2.0 * dy_f * 2.0)).max(0.0).sqrt() as i32;
        
        for dx in -w..=w+1 {
            let hash = seed.wrapping_add((dx as u64).wrapping_mul(131)).wrapping_add((dy as u64).wrapping_mul(71));
            let is_spot = (hash % 100) < (spot_chance * 100.0) as u64;
            let color = if is_spot { spot_color } else { cap_color };
            let ch = if is_spot { "o" } else { "@" };
            let dist_sq = dx * dx + dy * dy;
            cap_pixels.push((x + dx, y - dy, ch, color, dist_sq));
        }
    }
    
    cap_pixels.sort_by_key(|p| p.4);
    
    for (px, py, ch, color, _) in cap_pixels {
        canvas.cells.insert((px, py), BonsaiCell { content: ch.to_string(), color, element_type: ElementType::Leaf });
    }
}
