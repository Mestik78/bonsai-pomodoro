use super::canvas::{BonsaiCanvas, BonsaiCell};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use ratatui::style::Color;

pub fn generate_cactus(seed: u64, progress: f32) -> BonsaiCanvas {
    let mut canvas = BonsaiCanvas::new();
    if progress <= 0.0 {
        return canvas;
    }
    
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Main body properties
    let h = rng.gen_range(5..=9) as i32;
    let w = rng.gen_range(8..=14) as i32; // half-width
    
    let current_h = (h as f32 * progress).ceil() as i32;
    
    // Pups (hijos)
    let num_pups = rng.gen_range(0..=2);
    let mut pups = Vec::new();
    for _ in 0..num_pups {
        let pup_h = rng.gen_range(2..=4);
        let pup_w = rng.gen_range(3..=6);
        let pup_x = if rng.gen_bool(0.5) { rng.gen_range(-w..-w/2) } else { rng.gen_range(w/2..w) };
        pups.push((pup_x, pup_h, pup_w));
    }
    
    // Draw pups first
    for &(pup_x, pup_h, pup_w) in &pups {
        let pup_current_h = (pup_h as f32 * progress).ceil() as i32;
        for y in 0..=pup_current_h {
            draw_ellipse_row(&mut canvas, &mut rng, pup_x, y, pup_h, pup_w);
        }
    }
    
    // Draw main body
    for y in 0..=current_h {
        draw_ellipse_row(&mut canvas, &mut rng, 0, y, h, w);
        
        // Flower on top
        if y == h && progress >= 0.95 {
            if rng.gen_bool(0.8) {
                let f_colors = [Color::Magenta, Color::Red, Color::Yellow, Color::LightRed];
                let f_color = f_colors[rng.gen_range(0..f_colors.len())];
                canvas.cells.insert((0, -y - 1), BonsaiCell {
                    content: "✿".to_string(),
                    color: f_color,
                });
            }
        }
    }
    
    canvas
}

fn draw_ellipse_row(canvas: &mut BonsaiCanvas, rng: &mut StdRng, offset_x: i32, y: i32, h: i32, w: i32) {
    // Adding +1.0 to the divisor ensures it doesn't get infinitely sharp at the top/bottom
    // It creates a nice flat top and bottom, simulating the ground and the crown of the cactus.
    let y_factor = (y as f32 - h as f32 / 2.0) / (h as f32 / 2.0 + 1.0); 
    let inner_sqrt = (1.0 - y_factor * y_factor).max(0.0);
    let mut x_max = (w as f32 * inner_sqrt.sqrt()).round() as i32;
    
    // Organic noise to edges
    if y > 0 && y < h {
        x_max += rng.gen_range(-1..=1);
    }
    x_max = x_max.max(1);
    
    for x in -x_max..=x_max {
        let is_edge = x == -x_max || x == x_max || y == h || y == 0;
        
        let color = if rng.gen_bool(0.3) { Color::LightGreen } else if rng.gen_bool(0.1) { Color::DarkGray } else { Color::Green };
        
        let s = if is_edge {
            if x == -x_max || x == x_max {
                if rng.gen_bool(0.4) { "." } else { "|" }
            } else if y == h {
                if rng.gen_bool(0.4) { "." } else { "_" }
            } else {
                "_"
            }
        } else {
            let chars = ["*", "&", "#", "%", "@", ":"];
            chars[rng.gen_range(0..chars.len())]
        };
        
        // We negate y because in the canvas, negative y goes UP.
        canvas.cells.insert((offset_x + x, -y), BonsaiCell {
            content: s.to_string(),
            color,
        });
    }
}
