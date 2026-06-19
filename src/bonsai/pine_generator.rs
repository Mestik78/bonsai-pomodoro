use super::canvas::{BonsaiCanvas, BonsaiCell, ElementType};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;
use crate::models::plant::Fruit;

pub fn generate_pine(seed: u64, progress: f32, _fruit: Option<Fruit>, _fruit_quantity: u32) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    if progress <= 0.0 { return canvas; }

    let mut dummy_canvas = BonsaiCanvas::new();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut total_steps = 0;
    pine_tree(&mut dummy_canvas, &mut rng, 0, 0, &mut total_steps, i32::MAX);
    
    if progress > 0.0 {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut steps = 0;
        let max_steps = (progress * total_steps as f32).max(1.0) as i32;
        pine_tree(&mut canvas, &mut rng, 0, 0, &mut steps, max_steps);
    }
    canvas
}

fn pine_tree(canvas: &mut BonsaiCanvas, rng: &mut StdRng, x: i32, mut y: i32, steps: &mut i32, max_steps: i32) {
    let trunk_color = Color::Rgb(139, 69, 19);
    let mut current_life = rng.gen_range(8..=14);
    let life = current_life;
    let needle_color = Color::Rgb(46, 139, 87);
    
    while current_life > 0 {
        if *steps >= max_steps { return; }
        *steps += 1;
        
        let width = if current_life > life - 3 { 3 } else if current_life > life / 2 { 2 } else { 1 };
        for i in 0..width {
            let offset = i - width/2;
            canvas.cells.insert((x + offset, y), BonsaiCell {
                content: "|".to_string(),
                color: trunk_color,
                element_type: ElementType::Trunk,
            });
        }
        
        let height_progress = (life - current_life) as f32 / life as f32; // 0.0 at bottom, 1.0 at top
        if current_life % 2 == 0 {
            // Pine branches are longest at bottom, shorter at top, but never 0
            let max_branch_len = ((life as f32 * (1.1 - height_progress)) / 1.5).max(2.0) as i32;
            
            if rng.gen_bool(0.6) {
                let len = rng.gen_range(max_branch_len/2 + 1..=max_branch_len);
                pine_branch(canvas, rng, x - width/2, y, -1, len, needle_color, steps, max_steps);
            }
            if rng.gen_bool(0.6) {
                let len = rng.gen_range(max_branch_len/2 + 1..=max_branch_len);
                pine_branch(canvas, rng, x + width/2, y, 1, len, needle_color, steps, max_steps);
            }
        }
        
        y -= 1;
        current_life -= 1;
    }
    
    if *steps < max_steps {
        canvas.cells.insert((x, y), BonsaiCell { content: "^".to_string(), color: needle_color, element_type: ElementType::Leaf });
        *steps += 1;
    }
}

fn pine_branch(canvas: &mut BonsaiCanvas, rng: &mut StdRng, mut x: i32, mut y: i32, dir_x: i32, len: i32, color: Color, steps: &mut i32, max_steps: i32) {
    for i in 0..len {
        if *steps >= max_steps { return; }
        *steps += 1;
        
        x += dir_x;
        if i > 0 && i % 3 == 0 {
            y += 1;
        }
        
        let c = if i == len - 1 { if dir_x < 0 { "<" } else { ">" } } else { "~" };
        
        canvas.cells.insert((x, y), BonsaiCell {
            content: c.to_string(),
            color,
            element_type: ElementType::Leaf,
        });
        
        if rng.gen_bool(0.6) && *steps < max_steps {
            *steps += 1;
            canvas.cells.insert((x, y + 1), BonsaiCell {
                content: ",".to_string(),
                color: Color::Rgb(34, 139, 34),
                element_type: ElementType::Leaf,
            });
        }
    }
}
