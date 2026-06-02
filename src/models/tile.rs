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
