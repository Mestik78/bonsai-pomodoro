use ratatui::style::Color;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum BranchType {
    Trunk,
    ShootLeft,
    ShootRight,
    Dying,
    Dead,
}

#[derive(Clone)]
pub struct BonsaiCell {
    pub content: String,
    pub color: Color,
}

pub struct BonsaiCanvas {
    pub cells: HashMap<(i32, i32), BonsaiCell>,
}

impl BonsaiCanvas {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }
    
    // Renders as a list of lines for ratatui
    pub fn render(&self, zoom: f32) -> Vec<ratatui::text::Line<'static>> {
        let mut lines = Vec::new();
        
        let min_x = self.cells.keys().map(|k| k.0).min().unwrap_or(0);
        let max_x = self.cells.keys().map(|k| k.0).max().unwrap_or(0);
        let min_y = self.cells.keys().map(|k| k.1).min().unwrap_or(0);
        let max_y = 0; // Trunk ends at y=0
        
        if zoom >= 1.0 {
            for y in min_y..=max_y {
                let mut spans = Vec::new();
                for x in min_x..=max_x {
                    if let Some(cell) = self.cells.get(&(x, y)) {
                        spans.push(ratatui::text::Span::styled(
                            cell.content.clone(),
                            ratatui::style::Style::default().fg(cell.color)
                        ));
                    } else {
                        spans.push(ratatui::text::Span::raw(" "));
                    }
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            
            // Draw Full Pot
            let color_text = Color::DarkGray;
            let color_leaf = Color::Green;
            let color_wood = Color::Rgb(160, 82, 45);
            
            let center_idx = (0 - min_x).max(0) as usize;
            
            let has_tree = self.cells.len() > 1;
            
            let pot_lines = if !has_tree {
                vec![
                    vec![
                        (":", color_text), ("_____________________________", color_leaf), (":", color_text)
                    ],
                    vec![(" \\                           / ", color_text)],
                    vec![("  \\_________________________/  ", color_text)],
                    vec![("  (_)                     (_)  ", color_text)]
                ]
            } else {
                vec![
                    vec![
                        (":", color_text), ("___________", color_leaf), (".", color_wood), ("/", color_wood), 
                        ("~~~", color_wood), ("\\", color_wood), (".", color_wood), ("___________", color_leaf), (":", color_text)
                    ],
                    vec![(" \\                           / ", color_text)],
                    vec![("  \\_________________________/  ", color_text)],
                    vec![("  (_)                     (_)  ", color_text)]
                ]
            };
            
            for line_parts in pot_lines {
                let mut spans = Vec::new();
                let pad = center_idx.saturating_sub(15);
                spans.push(ratatui::text::Span::raw(" ".repeat(pad)));
                for (s, c) in line_parts {
                    spans.push(ratatui::text::Span::styled(s.to_string(), ratatui::style::Style::default().fg(c)));
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            
        } else {
            // Zoom out logic using Braille
            let step_x = (1.0 / zoom).round().max(1.0) as i32;
            let step_y = (1.0 / zoom).round().max(1.0) as i32;
            
            for y in (min_y..=max_y).step_by(step_y as usize) {
                let mut spans = Vec::new();
                for x in (min_x..=max_x).step_by(step_x as usize) {
                    let mut braille_code = 0;
                    let mut last_color = Color::Green;
                    let mut has_cell = false;
                    
                    // Braille is 2x4 dots. We map the 2x4 grid to the source step_x * step_y region.
                    for by in 0..4 {
                        for bx in 0..2 {
                            let src_dx = (bx * step_x) / 2;
                            let src_dy = (by * step_y) / 4;
                            
                            if let Some(cell) = self.cells.get(&(x + src_dx, y + src_dy)) {
                                has_cell = true;
                                last_color = cell.color;
                                
                                let dot = match (bx, by) {
                                    (0, 0) => 0x01,
                                    (0, 1) => 0x02,
                                    (0, 2) => 0x04,
                                    (0, 3) => 0x40,
                                    (1, 0) => 0x08,
                                    (1, 1) => 0x10,
                                    (1, 2) => 0x20,
                                    (1, 3) => 0x80,
                                    _ => 0,
                                };
                                braille_code |= dot;
                            }
                        }
                    }
                    
                    if has_cell {
                        let ch = std::char::from_u32(0x2800 + braille_code).unwrap_or(' ');
                        spans.push(ratatui::text::Span::styled(
                            ch.to_string(),
                            ratatui::style::Style::default().fg(last_color)
                        ));
                    } else {
                        spans.push(ratatui::text::Span::raw(" "));
                    }
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            
            // Draw Mini Pot based on zoom
            let center_idx = ((0 - min_x) / step_x).max(0) as usize;
            let color_text = Color::DarkGray;
            
            let pot_lines = if zoom >= 0.5 {
                vec![
                    " (___________) ",
                    "  \\_________/  ",
                    "    (_) (_)    ",
                ]
            } else {
                vec![
                    " \\___/ ",
                    "  (_)  ",
                ]
            };
            
            for s in pot_lines {
                let mut spans = Vec::new();
                let pad = center_idx.saturating_sub(s.len() / 2);
                spans.push(ratatui::text::Span::raw(" ".repeat(pad)));
                spans.push(ratatui::text::Span::styled(s.to_string(), ratatui::style::Style::default().fg(color_text)));
                lines.push(ratatui::text::Line::from(spans));
            }
        }
        
        lines
    }
}

pub fn generate_bonsai(seed: u64, progress: f32) -> BonsaiCanvas {
    let life = 32;
    let multiplier = 5;
    
    // First pass: find total steps
    let mut dummy_canvas = BonsaiCanvas::new();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut total_steps = 0;
    branch(&mut dummy_canvas, &mut rng, 0, 0, BranchType::Trunk, life, multiplier, &mut total_steps, i32::MAX);
    
    // Second pass: generate actual tree
    let mut canvas = BonsaiCanvas::new();
    if progress > 0.0 {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut steps = 0;
        let max_steps = (progress * total_steps as f32).max(1.0) as i32;
        branch(&mut canvas, &mut rng, 0, 0, BranchType::Trunk, life, multiplier, &mut steps, max_steps);
    }
    
    canvas
}

fn set_deltas(rng: &mut StdRng, btype: BranchType, life: i32, age: i32, multiplier: i32) -> (i32, i32) {
    let dx;
    let mut dy = 0;
    
    match btype {
        BranchType::Trunk => {
            if age <= 2 || life < 4 {
                dy = 0;
                dx = rng.gen_range(-1..=1);
            } else if age < (multiplier * 3) {
                if age % (multiplier as f32 * 0.5) as i32 == 0 { dy = -1; }
                let dice = rng.gen_range(0..=9);
                if dice == 0 { dx = -2; }
                else if dice <= 3 { dx = -1; }
                else if dice <= 5 { dx = 0; }
                else if dice <= 8 { dx = 1; }
                else { dx = 2; }
            } else {
                let dice = rng.gen_range(0..=9);
                if dice > 2 { dy = -1; }
                dx = rng.gen_range(-1..=1);
            }
        },
        BranchType::ShootLeft => {
            let dice = rng.gen_range(0..=9);
            if dice <= 1 { dy = -1; }
            else if dice <= 7 { dy = 0; }
            else { dy = 1; }
            
            let dice = rng.gen_range(0..=9);
            if dice <= 1 { dx = -2; }
            else if dice <= 5 { dx = -1; }
            else if dice <= 8 { dx = 0; }
            else { dx = 1; }
        },
        BranchType::ShootRight => {
            let dice = rng.gen_range(0..=9);
            if dice <= 1 { dy = -1; }
            else if dice <= 7 { dy = 0; }
            else { dy = 1; }
            
            let dice = rng.gen_range(0..=9);
            if dice <= 1 { dx = 2; }
            else if dice <= 5 { dx = 1; }
            else if dice <= 8 { dx = 0; }
            else { dx = -1; }
        },
        BranchType::Dying => {
            let dice = rng.gen_range(0..=9);
            if dice <= 1 { dy = -1; }
            else if dice <= 8 { dy = 0; }
            else { dy = 1; }
            
            let dice = rng.gen_range(0..=14);
            if dice == 0 { dx = -3; }
            else if dice <= 2 { dx = -2; }
            else if dice <= 5 { dx = -1; }
            else if dice <= 8 { dx = 0; }
            else if dice <= 11 { dx = 1; }
            else if dice <= 13 { dx = 2; }
            else { dx = 3; }
        },
        BranchType::Dead => {
            let dice = rng.gen_range(0..=9);
            if dice <= 2 { dy = -1; }
            else if dice <= 6 { dy = 0; }
            else { dy = 1; }
            dx = rng.gen_range(-1..=1);
        }
    }
    (dx, dy)
}

fn choose_color(rng: &mut StdRng, btype: BranchType) -> Color {
    match btype {
        BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
            if rng.gen_bool(0.5) { Color::Rgb(160, 82, 45) } else { Color::Rgb(101, 67, 33) }
        },
        BranchType::Dying => {
            if rng.gen_bool(0.1) { Color::LightGreen } else { Color::Green }
        },
        BranchType::Dead => {
            if rng.gen_bool(0.3) { Color::DarkGray } else { Color::Green }
        }
    }
}

fn choose_string(rng: &mut StdRng, mut btype: BranchType, life: i32, dx: i32, dy: i32) -> String {
    if life < 4 { btype = BranchType::Dying; }
    
    match btype {
        BranchType::Trunk => {
            if dy == 0 { "/~".to_string() }
            else if dx < 0 { "\\|".to_string() }
            else if dx == 0 { "/|\\".to_string() }
            else { "|/".to_string() }
        },
        BranchType::ShootLeft => {
            if dy > 0 { "\\".to_string() }
            else if dy == 0 { "\\_".to_string() }
            else if dx < 0 { "\\|".to_string() }
            else if dx == 0 { "/|".to_string() }
            else { "/".to_string() }
        },
        BranchType::ShootRight => {
            if dy > 0 { "/".to_string() }
            else if dy == 0 { "_/".to_string() }
            else if dx < 0 { "\\|".to_string() }
            else if dx == 0 { "/|".to_string() }
            else { "/".to_string() }
        },
        BranchType::Dying | BranchType::Dead => {
            let leaves = ["&", "*"];
            leaves[rng.gen_range(0..leaves.len())].to_string()
        }
    }
}

fn branch(canvas: &mut BonsaiCanvas, rng: &mut StdRng, mut y: i32, mut x: i32, btype: BranchType, mut life: i32, multiplier: i32, steps: &mut i32, max_steps: i32) {
    let mut shoot_cooldown = multiplier;
    let life_start = life;
    
    while life > 0 {
        if *steps >= max_steps {
            return;
        }
        *steps += 1;
        
        life -= 1;
        let age = life_start - life;
        let (dx, dy) = set_deltas(rng, btype, life, age, multiplier);
        
        if life < 3 {
            branch(canvas, rng, y, x, BranchType::Dead, life, multiplier, steps, max_steps);
        } else if btype == BranchType::Trunk && life < multiplier + 2 {
            branch(canvas, rng, y, x, BranchType::Dying, life, multiplier, steps, max_steps);
        } else if (btype == BranchType::ShootLeft || btype == BranchType::ShootRight) && life < multiplier + 2 {
            branch(canvas, rng, y, x, BranchType::Dying, life, multiplier, steps, max_steps);
        } else if btype == BranchType::Trunk && (rng.gen_range(0..3) == 0 || life % multiplier == 0) {
            if rng.gen_range(0..8) == 0 && life > 7 {
                shoot_cooldown = multiplier * 2;
                let added_life = rng.gen_range(-2..=2);
                branch(canvas, rng, y, x, BranchType::Trunk, life + added_life, multiplier, steps, max_steps);
            } else if shoot_cooldown <= 0 {
                shoot_cooldown = multiplier * 2;
                let shoot_life = life + multiplier;
                let shoot_type = if rng.gen_bool(0.5) { BranchType::ShootLeft } else { BranchType::ShootRight };
                branch(canvas, rng, y, x, shoot_type, shoot_life, multiplier, steps, max_steps);
            }
        }
        shoot_cooldown -= 1;
        
        x += dx;
        y += dy;
        
        let color = choose_color(rng, btype);
        let s = choose_string(rng, btype, life, dx, dy);
        
        // Add to canvas
        for (i, c) in s.chars().enumerate() {
            canvas.cells.insert((x + i as i32, y), BonsaiCell {
                content: c.to_string(),
                color,
            });
        }
    }
}
