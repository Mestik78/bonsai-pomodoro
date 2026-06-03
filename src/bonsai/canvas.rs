use ratatui::style::Color;
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElementType {
    Trunk,
    Leaf,
    Fruit,
}

#[derive(Clone)]
pub struct BonsaiCell {
    pub content: String,
    pub color: Color,
    pub element_type: ElementType,
}

pub struct BonsaiCanvas {
    pub cells: HashMap<(i32, i32), BonsaiCell>,
    pub trunk_parts: Option<Vec<(String, Color)>>,
}

impl BonsaiCanvas {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            trunk_parts: Some(vec![
                (".".to_string(), Color::Rgb(160, 82, 45)),
                ("/".to_string(), Color::Rgb(160, 82, 45)),
                ("~~~".to_string(), Color::Rgb(160, 82, 45)),
                ("\\".to_string(), Color::Rgb(160, 82, 45)),
                (".".to_string(), Color::Rgb(160, 82, 45)),
            ]),
        }
    }
    
    // Renders as a list of lines for ratatui
    pub fn render(&self, zoom: f32, label: Option<String>, pot_color: Option<Color>) -> Vec<ratatui::text::Line<'static>> {
        let mut lines = Vec::new();
        
        let mut min_x = self.cells.keys().map(|k| k.0).min().unwrap_or(0);
        let mut max_x = self.cells.keys().map(|k| k.0).max().unwrap_or(0);
        let min_y = self.cells.keys().map(|k| k.1).min().unwrap_or(0);
        let max_y = 0; // Trunk ends at y=0
        
        if zoom >= 1.0 {
            let required_radius = 15;
            min_x = min_x.min(-required_radius);
            max_x = max_x.max(required_radius);
        } else {
            let step_x = (1.0 / zoom).round().max(1.0) as i32;
            let required_radius = if zoom >= 0.5 { 7 * step_x } else { 4 * step_x };
            min_x = min_x.min(-required_radius);
            max_x = max_x.max(required_radius);
        }
        
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
            let color_text = pot_color.unwrap_or(Color::DarkGray);
            let color_leaf = Color::Green;
            let _color_wood = Color::Rgb(160, 82, 45);
            
            let center_idx = (0 - min_x).max(0) as usize;
            
            let has_tree = self.cells.len() > 1;
            
            let mut pot_lines: Vec<Vec<(String, Color)>> = Vec::new();
            
            if !has_tree {
                pot_lines.push(vec![
                    (":".to_string(), color_text), ("_____________________________".to_string(), color_leaf), (":".to_string(), color_text)
                ]);
            } else if let Some(ref tp) = self.trunk_parts {
                let mut line = vec![(":".to_string(), color_text)];
                let tp_len: usize = tp.iter().map(|(s, _)| s.chars().count()).sum();
                let side_len = (29_usize.saturating_sub(tp_len)) / 2;
                let right_len = 29 - tp_len - side_len;
                line.push(("_".repeat(side_len), color_leaf));
                line.extend(tp.clone());
                line.push(("_".repeat(right_len), color_leaf));
                line.push((":".to_string(), color_text));
                pot_lines.push(line);
            } else {
                pot_lines.push(vec![
                    (":".to_string(), color_text), ("_____________________________".to_string(), color_leaf), (":".to_string(), color_text)
                ]);
            }
            
            let mut line2 = " \\                           / ".to_string();
            if let Some(ref lbl) = label {
                let lbl_len = lbl.chars().count();
                let spaces = 27;
                if lbl_len <= spaces {
                    let pad_left = (spaces - lbl_len) / 2;
                    let mut new_line2 = String::from(" \\");
                    new_line2.push_str(&" ".repeat(pad_left));
                    new_line2.push_str(lbl);
                    new_line2.push_str(&" ".repeat(spaces - pad_left - lbl_len));
                    new_line2.push_str("/ ");
                    line2 = new_line2;
                }
            }
            pot_lines.push(vec![(line2, color_text)]);
            pot_lines.push(vec![("  \\_________________________/  ".to_string(), color_text)]);
            pot_lines.push(vec![("  (_)                     (_)  ".to_string(), color_text)]);
            
            let total_cols = (min_x..=max_x).count();
            for line_parts in pot_lines {
                let mut spans = Vec::new();
                let pad = center_idx.saturating_sub(15);
                spans.push(ratatui::text::Span::raw(" ".repeat(pad)));
                let mut current_len = pad;
                for (s, c) in line_parts {
                    current_len += s.chars().count();
                    spans.push(ratatui::text::Span::styled(s, ratatui::style::Style::default().fg(c)));
                }
                if current_len < total_cols {
                    spans.push(ratatui::text::Span::raw(" ".repeat(total_cols - current_len)));
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            
        } else {
            // Zoom out logic using Braille
            let step_x = (1.0 / zoom).round().max(1.0) as i32;
            let step_y = (1.0 / zoom).round().max(1.0) as i32;
            
            for y in (min_y..=max_y).step_by(step_y as usize) {
                let is_last_line = y + step_y > max_y;
                let mut spans = Vec::new();
                let mut col_idx = 0;
                for x in (min_x..=max_x).step_by(step_x as usize) {
                    let mut braille_code = 0;
                    let mut last_color = Color::Green;
                    let mut max_element = ElementType::Trunk;
                    let mut has_cell = false;
                    
                    // Braille is 2x4 dots. We map the 2x4 grid to the source step_x * step_y region.
                    for by in 0..4 {
                        for bx in 0..2 {
                            let src_dx = (bx * step_x) / 2;
                            let src_dy = (by * step_y) / 4;
                            
                            if let Some(cell) = self.cells.get(&(x + src_dx, y + src_dy)) {
                                has_cell = true;
                                if cell.element_type >= max_element {
                                    max_element = cell.element_type;
                                    last_color = cell.color;
                                }
                                
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
                    
                    let center_col = ((0 - min_x) / step_x).max(0) as i32;
                    let dist = (col_idx as i32 - center_col).abs();
                    
                    if has_cell {
                        let ch = std::char::from_u32(0x2800 + braille_code).unwrap_or(' ');
                        spans.push(ratatui::text::Span::styled(
                            ch.to_string(),
                            ratatui::style::Style::default().fg(last_color)
                        ));
                    } else {
                        if is_last_line {
                            let pc = pot_color.unwrap_or(Color::DarkGray);
                            if zoom >= 0.5 {
                                if dist <= 5 {
                                    spans.push(ratatui::text::Span::styled("_", ratatui::style::Style::default().fg(pc)));
                                } else {
                                    spans.push(ratatui::text::Span::raw(" "));
                                }
                            } else {
                                if dist <= 1 {
                                    spans.push(ratatui::text::Span::styled("_", ratatui::style::Style::default().fg(pc)));
                                } else {
                                    spans.push(ratatui::text::Span::raw(" "));
                                }
                            }
                        } else {
                            spans.push(ratatui::text::Span::raw(" "));
                        }
                    }
                    col_idx += 1;
                }
                lines.push(ratatui::text::Line::from(spans));
            }
            
            // Draw Mini Pot based on zoom
            let center_idx = ((0 - min_x) / step_x).max(0) as usize;
            
            let pot_lines = if zoom >= 0.5 {
                let mut l1 = "  \\_________/  ".to_string();
                if let Some(ref lbl) = label {
                    let lbl_len = lbl.chars().count();
                    let spaces = 9;
                    if lbl_len <= spaces {
                        let pad_left = (spaces - lbl_len) / 2;
                        let mut new_l1 = String::from("  \\");
                        new_l1.push_str(&"_".repeat(pad_left));
                        new_l1.push_str(lbl);
                        new_l1.push_str(&"_".repeat(spaces - pad_left - lbl_len));
                        new_l1.push_str("/  ");
                        l1 = new_l1;
                    }
                }
                vec![l1]
            } else {
                vec![
                    "  \\___/  ".to_string(),
                ]
            };
            
            let total_cols = (min_x..=max_x).step_by(step_x as usize).count();
            let pot_c = pot_color.unwrap_or(Color::DarkGray);
            for s in pot_lines {
                let mut spans = Vec::new();
                let pad = center_idx.saturating_sub(s.chars().count() / 2);
                spans.push(ratatui::text::Span::raw(" ".repeat(pad)));
                spans.push(ratatui::text::Span::styled(s.to_string(), ratatui::style::Style::default().fg(pot_c)));
                
                let current_len = pad + s.chars().count();
                if current_len < total_cols {
                    spans.push(ratatui::text::Span::raw(" ".repeat(total_cols - current_len)));
                }
                
                lines.push(ratatui::text::Line::from(spans));
            }
        }
        
        lines
    }

    pub fn render_full(&self, label: Option<String>, pot_color: Option<Color>) -> Vec<ratatui::text::Line<'static>> {
        self.render(1.0, label, pot_color)
    }

    pub fn render_medium(&self, label: Option<String>, pot_color: Option<Color>) -> Vec<ratatui::text::Line<'static>> {
        self.render(0.5, label, pot_color)
    }

    pub fn render_small(&self, label: Option<String>, pot_color: Option<Color>) -> Vec<ratatui::text::Line<'static>> {
        self.render(0.25, label, pot_color)
    }

    pub fn render_micro(&self, _label: Option<String>, _pot_color: Option<Color>) -> Vec<ratatui::text::Line<'static>> {
        let mut best_cell: Option<(&(i32, i32), &BonsaiCell)> = None;
        
        for (pos, cell) in self.cells.iter() {
            if let Some((best_pos, best_c)) = best_cell {
                if cell.element_type > best_c.element_type {
                    best_cell = Some((pos, cell));
                } else if cell.element_type == best_c.element_type {
                    if pos > best_pos {
                        best_cell = Some((pos, cell));
                    }
                }
            } else {
                best_cell = Some((pos, cell));
            }
        }
        
        let best_color = best_cell.map(|(_, c)| c.color).unwrap_or(Color::Green);
        
        vec![
            ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("♣", ratatui::style::Style::default().fg(best_color)),
                ratatui::text::Span::raw(" "),
            ])
        ]
    }
}
