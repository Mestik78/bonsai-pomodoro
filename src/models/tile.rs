use crate::bonsai::canvas::BonsaiCanvas;
use crate::bonsai::PlantFrame;
use ratatui::style::Color;
use ratatui::text::Line;

pub trait Tile {
    fn render(&self, width: u16, height: u16) -> Vec<Line<'static>>;
}

pub struct PlantTile<'a> {
    pub canvas: &'a BonsaiCanvas,
    pub frame: PlantFrame,
    pub label: Option<String>,
    pub pot_color: Option<Color>,
}

impl<'a> Tile for PlantTile<'a> {
    fn render(&self, width: u16, height: u16) -> Vec<Line<'static>> {
        let mut lines = self.frame.render(self.canvas, width, height, self.label.clone(), self.pot_color);
        
        for line in &mut lines {
            let line_width: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
            if (line_width as u16) < width {
                let pad_total = width - line_width as u16;
                let pad_left = pad_total / 2;
                let pad_right = pad_total - pad_left;
                
                let mut new_spans = vec![ratatui::text::Span::raw(" ".repeat(pad_left as usize))];
                new_spans.extend(line.spans.clone());
                new_spans.push(ratatui::text::Span::raw(" ".repeat(pad_right as usize)));
                *line = ratatui::text::Line::from(new_spans);
            }
        }
        
        let actual_height = lines.len() as u16;
        if actual_height < height {
            let pad_top = height - actual_height;
            let mut padded_lines = Vec::new();
            for _ in 0..pad_top {
                padded_lines.push(ratatui::text::Line::from(" ".repeat(width as usize)));
            }
            padded_lines.extend(lines);
            lines = padded_lines;
        } else if actual_height > height {
            let skip = actual_height - height;
            lines = lines.into_iter().skip(skip as usize).collect();
        }
        
        lines
    }
}

pub struct TerrainTile {
    pub neighbors: [[bool; 3]; 3],
}

impl Tile for TerrainTile {
    fn render(&self, width: u16, height: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        
        let get_val = |x: i32, y: i32| -> f32 {
            let nx = (x + 1).clamp(0, 2) as usize;
            let ny = (y + 1).clamp(0, 2) as usize;
            if self.neighbors[ny][nx] { 1.0 } else { 0.0 }
        };

        for r in 0..height {
            let mut span_content = String::with_capacity(width as usize);
            for c in 0..width {
                // map character center to [-0.5, 0.5]
                let mx = -0.5 + (c as f32 + 0.5) / (width as f32);
                let my = -0.5 + (r as f32 + 0.5) / (height as f32);
                
                let evaluate = |px: f32, py: f32| -> bool {
                    let x0 = px.floor();
                    let x1 = x0 + 1.0;
                    let y0 = py.floor();
                    let y1 = y0 + 1.0;
                    
                    let v00 = get_val(x0 as i32, y0 as i32);
                    let v10 = get_val(x1 as i32, y0 as i32);
                    let v01 = get_val(x0 as i32, y1 as i32);
                    let v11 = get_val(x1 as i32, y1 as i32);
                    
                    let tx = px - x0;
                    let ty = py - y0;
                    
                    let v = v00 * (1.0 - tx) * (1.0 - ty)
                          + v10 * tx * (1.0 - ty)
                          + v01 * (1.0 - tx) * ty
                          + v11 * tx * ty;
                          
                    v >= 0.35 // threshold for organic shapes
                };
                
                let ch = if evaluate(mx, my) {
                    let pattern = ".  ,   '  .,   ";
                    let idx = (r as usize * 17 + c as usize * 11) % pattern.len();
                    pattern.chars().nth(idx).unwrap_or(' ')
                } else {
                    ' '
                };
                span_content.push(ch);
            }
            
            lines.push(Line::from(ratatui::text::Span::styled(
                span_content,
                ratatui::style::Style::default().fg(Color::Rgb(34, 139, 34)) // Forest Green
            )));
        }
        lines
    }
}

// Removed PathTile in favor of TerrainTile
