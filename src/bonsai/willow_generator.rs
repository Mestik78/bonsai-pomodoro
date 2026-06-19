use super::canvas::{BonsaiCanvas, BonsaiCell, ElementType};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use crate::models::plant::Fruit;

pub fn generate_willow(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let mut dummy_canvas = BonsaiCanvas::new();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut total_steps = 0;
    willow_tree(&mut dummy_canvas, &mut rng, 0, 0, 11, &mut total_steps, i32::MAX);
    
    let mut canvas = BonsaiCanvas::new();
    if progress > 0.0 {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut steps = 0;
        let max_steps = (progress * total_steps as f32).max(1.0) as i32;
        willow_tree(&mut canvas, &mut rng, 0, 0, 11, &mut steps, max_steps);
    }
    canvas
}

fn willow_tree(canvas: &mut BonsaiCanvas, rng: &mut StdRng, mut x: i32, mut y: i32, life: i32, steps: &mut i32, max_steps: i32) {
    let trunk_color = Color::Rgb(139, 115, 85);
    for _ in 0..life {
        if *steps >= max_steps { return; }
        *steps += 1;
        
        canvas.cells.insert((x, y), BonsaiCell { content: "|".to_string(), color: trunk_color, element_type: ElementType::Trunk });
        canvas.cells.insert((x-1, y), BonsaiCell { content: "|".to_string(), color: trunk_color, element_type: ElementType::Trunk });
        
        if rng.gen_bool(0.3) { x += if rng.gen_bool(0.5) { 1 } else { -1 }; }
        
        if rng.gen_bool(0.15) {
            let dir = if rng.gen_bool(0.5) { 1 } else { -1 };
            let len = rng.gen_range(5..=10);
            willow_branch(canvas, rng, x, y, dir, len, steps, max_steps);
        }
        y -= 1;
    }
    
    for _ in 0..2 {
        let dir = if rng.gen_bool(0.5) { 1 } else { -1 };
        let len = rng.gen_range(6..=12);
        willow_branch(canvas, rng, x, y, dir, len, steps, max_steps);
    }
}

fn willow_branch(canvas: &mut BonsaiCanvas, rng: &mut StdRng, mut x: i32, mut y: i32, dir_x: i32, life: i32, steps: &mut i32, max_steps: i32) {
    let trunk_color = Color::Rgb(139, 115, 85);
    for i in 0..life {
        if *steps >= max_steps { return; }
        *steps += 1;
        
        // Ramas más a los lados: dir_x en vez de sumar 1, puede sumar 2
        x += dir_x;
        if rng.gen_bool(0.3) { x += dir_x; }
        
        if i > life / 3 {
            y += if rng.gen_bool(0.7) { 1 } else { 0 };
        } else {
            y -= if rng.gen_bool(0.3) { 1 } else { 0 };
        }
        
        let c = if i > life / 2 { "\\" } else { "_" };
        canvas.cells.insert((x, y), BonsaiCell { content: c.to_string(), color: trunk_color, element_type: ElementType::Trunk });
        
        if rng.gen_bool(0.3) {
            let vine_len = rng.gen_range(4..=12);
            willow_vine(canvas, rng, x, y, vine_len, steps, max_steps);
        }
    }
}

fn willow_vine(canvas: &mut BonsaiCanvas, rng: &mut StdRng, x: i32, mut y: i32, len: i32, steps: &mut i32, max_steps: i32) {
    let vine_color = Color::Rgb(154, 205, 50);
    for _ in 0..len {
        if *steps >= max_steps { return; }
        *steps += 1;
        y += 1;
        let c = if rng.gen_bool(0.5) { "|" } else { ":" };
        canvas.cells.insert((x, y), BonsaiCell { content: c.to_string(), color: vine_color, element_type: ElementType::Leaf });
    }
}
