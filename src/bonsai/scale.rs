use crate::bonsai::canvas::BonsaiCanvas;
use ratatui::style::Color;
use ratatui::text::Line;

pub type RenderFn = fn(&BonsaiCanvas, Option<String>, Option<Color>) -> Vec<Line<'static>>;

pub struct Scale {
    pub min_width: u16,
    pub min_height: u16,
    pub render_fn: RenderFn,
}

pub const SCALES: [Scale; 4] = [
    Scale {
        min_width: 52,
        min_height: 24,
        render_fn: BonsaiCanvas::render_full,
    },
    Scale {
        min_width: 24,
        min_height: 12,
        render_fn: BonsaiCanvas::render_medium,
    },
    Scale {
        min_width: 12,
        min_height: 6,
        render_fn: BonsaiCanvas::render_small,
    },
    Scale {
        min_width: 2,
        min_height: 1,
        render_fn: BonsaiCanvas::render_micro,
    },
];

#[derive(Clone, Copy)]
pub struct PlantFrame {
    pub max_scale: usize,
    pub min_scale: usize,
}

impl PlantFrame {
    pub fn new(max_scale: Option<usize>, min_scale: Option<usize>) -> Self {
        Self {
            max_scale: max_scale.unwrap_or(0),
            min_scale: min_scale.unwrap_or(SCALES.len() - 1),
        }
    }

    pub fn get_best_scale(&self, width: u16, height: u16) -> &Scale {
        // Find the largest scale (lowest index) that fits, starting from max_scale to min_scale
        for i in self.max_scale..=self.min_scale {
            let scale = &SCALES[i];
            if width >= scale.min_width && height >= scale.min_height {
                return scale;
            }
        }
        // If none fit, return the min_scale (smallest allowed)
        &SCALES[self.min_scale]
    }

    pub fn render(
        &self,
        canvas: &BonsaiCanvas,
        width: u16,
        height: u16,
        label: Option<String>,
        pot_color: Option<Color>,
    ) -> Vec<Line<'static>> {
        let scale = self.get_best_scale(width, height);
        (scale.render_fn)(canvas, label, pot_color)
    }
}
