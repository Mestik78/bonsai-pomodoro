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
        self.frame.render(self.canvas, width, height, self.label.clone(), self.pot_color)
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
        let pattern = ".,  .. , .  ,  . ";
        let pat_len = pattern.len();
        
        for y in 0..height {
            let mut span_content = String::with_capacity(width as usize);
            for x in 0..width {
                let idx = (y as usize * 13 + x as usize * 7) % pat_len;
                let ch = pattern.chars().nth(idx).unwrap_or(' ');
                span_content.push(ch);
            }
            lines.push(Line::from(ratatui::text::Span::styled(
                span_content,
                ratatui::style::Style::default().fg(Color::Rgb(139, 69, 19)) // Brown
            )));
        }
        lines
    }
}
