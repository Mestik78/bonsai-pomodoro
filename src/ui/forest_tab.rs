use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::tile::{Tile, PlantTile, EmptyTile};

pub fn render(frame: &mut Frame, app: &App, inner_area: Rect) {
    let state = &app.forest;
    let map = &state.tilemap;
    
    let zoom_level = &map.zoom_levels[map.current_zoom];
    let tile_w = zoom_level.tile_width;
    let tile_h = zoom_level.tile_height;
    
    let cols = (inner_area.width / tile_w) as i32;
    let rows = (inner_area.height / tile_h) as i32;
    
    let mut all_lines = Vec::new();
    
    // Plant base parameters for the test
    let plant = crate::models::plant::Plant::new(1234, crate::models::plant::PlantType::Bonsai, 1.0);
    let canvas = crate::models::plant::generate_plant(&plant);
    let plant_frame = crate::bonsai::PlantFrame::new(Some(map.current_zoom), Some(map.current_zoom));

    for row in 0..rows {
        let mut row_tiles_lines = Vec::new();
        for col in 0..cols {
            let map_x = map.camera_x + col;
            let map_y = map.camera_y + row;
            
            let tile_lines = if map_x == 0 && (map_y == 0 || map_y == 1) {
                let tile = PlantTile {
                    canvas: &canvas,
                    frame: plant_frame,
                    label: None,
                    pot_color: None,
                };
                tile.render(tile_w, tile_h)
            } else {
                EmptyTile.render(tile_w, tile_h)
            };
            row_tiles_lines.push(tile_lines);
        }
        
        for i in 0..tile_h as usize {
            let mut combined_spans = Vec::new();
            for col in 0..cols as usize {
                if i < row_tiles_lines[col].len() {
                    for span in &row_tiles_lines[col][i].spans {
                        combined_spans.push(span.clone());
                    }
                }
            }
            all_lines.push(Line::from(combined_spans));
        }
    }
    
    let status_color = if state.is_moving { Color::Green } else { Color::DarkGray };
    let status_text = if state.is_moving {
        " MOVEMENT MODE (Arrows to pan, +/- to zoom, Esc to exit) "
    } else {
        " VIEW MODE (Enter to move, +/- to zoom) "
    };

    let p = Paragraph::new(all_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(status_color)).title(Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD))));
        
    frame.render_widget(p, inner_area);
}
