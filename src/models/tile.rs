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

pub struct EmptyTile;

impl Tile for EmptyTile {
    fn render(&self, width: u16, height: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for _ in 0..height {
            lines.push(Line::from(" ".repeat(width as usize)));
        }
        lines
    }
}

pub struct PathTile;

impl Tile for PathTile {
    fn render(&self, width: u16, height: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let pattern = ".  ,   '  .,   ";
        let pat_len = pattern.len();
        
        for y in 0..height {
            let mut span_content = String::with_capacity(width as usize);
            for x in 0..width {
                let idx = (y as usize * 17 + x as usize * 11) % pat_len;
                let ch = pattern.chars().nth(idx).unwrap_or(' ');
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
