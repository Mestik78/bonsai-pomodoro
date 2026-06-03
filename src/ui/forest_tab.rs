use ratatui::{
    layout::{Alignment, Rect, Layout, Constraint, Direction},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::app::App;
use crate::models::tile::{Tile, PlantTile, EmptyTile, PathTile};

fn slice_line(line: &Line, skip: usize, take: usize) -> Line<'static> {
    let mut skipped = 0;
    let mut taken = 0;
    let mut new_spans = Vec::new();
    
    for span in &line.spans {
        if taken >= take {
            break;
        }
        let chars_count = span.content.chars().count();
        if skipped + chars_count <= skip {
            skipped += chars_count;
            continue;
        }
        
        let start_char = if skipped < skip { skip - skipped } else { 0 };
        let available = chars_count - start_char;
        let needed = take - taken;
        let end_char = start_char + available.min(needed);
        
        let sub_str: String = span.content.chars().skip(start_char).take(end_char - start_char).collect();
        new_spans.push(Span::styled(sub_str, span.style));
        
        skipped += if skipped < skip { start_char } else { 0 };
        taken += end_char - start_char;
    }
    if taken < take {
        new_spans.push(Span::raw(" ".repeat(take - taken)));
    }
    
    Line::from(new_spans)
}

pub fn render(frame: &mut Frame, app: &App, inner_area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner_area);
        
    let info_area = chunks[0];
    let map_area = chunks[1];

    let state = &app.forest;
    let map = &state.tilemap;
    
    let zoom_level = &map.zoom_levels[map.current_zoom];
    let tile_w = zoom_level.tile_width;
    let tile_h = zoom_level.tile_height;
    
    let screen_width = map_area.width as usize;
    let screen_height = map_area.height as usize;
    
    if screen_width == 0 || screen_height == 0 {
        return;
    }
    
    let center_col = (map.camera_x + (tile_w as i32) / 2).div_euclid(tile_w as i32);
    let center_row = (map.camera_y + (tile_h as i32) / 2).div_euclid(tile_h as i32);
    
    let selected_idx = match state.forest_map.grid.get(&(center_col, center_row)) {
        Some(crate::models::forest_map::MapElement::Plant(idx)) => Some(*idx),
        _ => None,
    };
    
    let mut info_text = " Center a plant to view details ".to_string();
    if state.is_searching {
        info_text = format!(" /{}_ ", state.search_query);
    } else if !state.search_matches.is_empty() {
        info_text = format!(" Match {}/{} for '{}' (n to jump, Esc to clear) ", state.search_index + 1, state.search_matches.len(), state.search_query);
    } else if let Some(idx) = selected_idx {
        let selected_timer = &app.timers[idx];
        let title = selected_timer.title.as_deref().unwrap_or("Unnamed Session");
        let date = match chrono::DateTime::parse_from_rfc3339(&selected_timer.start_time) {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => selected_timer.start_time.clone(),
        };
        
        let mut total_duration = 0;
        for t in &app.timers {
            if t.state.is_none() {
                let d = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                    Ok(dt) => dt.format("%Y-%m-%d").to_string(),
                    Err(_) => t.start_time.clone(),
                };
                if d == date {
                    total_duration += t.duration;
                }
            }
        }
        
        info_text = format!(" Plant: {} │ Date: {} │ Day Total: {:02}:{:02} ", title, date, total_duration / 60, total_duration % 60);
    }

    let info_p = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)))
        .alignment(Alignment::Center);
    frame.render_widget(info_p, info_area);

    let start_x = map.camera_x - (screen_width as i32) / 2;
    let start_y = map.camera_y + (screen_height as i32) / 2;
    
    let logical_start_x = start_x + (tile_w as i32) / 2;
    let logical_start_y = start_y + (tile_h as i32) / 2;
    
    let start_col = logical_start_x.div_euclid(tile_w as i32);
    let end_col = (logical_start_x + screen_width as i32 - 1).div_euclid(tile_w as i32);
    
    let min_row = (logical_start_y - screen_height as i32 + 1).div_euclid(tile_h as i32);
    let max_row = logical_start_y.div_euclid(tile_h as i32);
    
    let _plant = crate::models::plant::Plant::new(1234, crate::models::plant::PlantType::Bonsai, 1.0);
    let _canvas = crate::models::plant::generate_plant(&_plant);
    let plant_frame = crate::bonsai::PlantFrame::new(Some(map.current_zoom), Some(map.current_zoom));

    let mut tile_cache = HashMap::new();
    
    for row in min_row..=max_row {
        for col in start_col..=end_col {
            let is_center = col == (map.camera_x + (tile_w as i32) / 2).div_euclid(tile_w as i32)
                         && row == (map.camera_y + (tile_h as i32) / 2).div_euclid(tile_h as i32);
                         
            let tile_lines = match state.forest_map.grid.get(&(col, row)).copied() {
                Some(crate::models::forest_map::MapElement::Plant(idx)) => {
                    let t = &app.timers[idx];
                    let canvas = crate::models::plant::generate_plant(&crate::models::plant::Plant::new(t.seed, t.plant_type.clone(), 1.0));
                    
                    let pot_color = if is_center {
                        Some(ratatui::style::Color::Yellow)
                    } else {
                        None
                    };

                    let tile = PlantTile {
                        canvas: &canvas,
                        frame: plant_frame.clone(),
                        label: Some(format!("{:02}:{:02}", t.duration / 60, t.duration % 60)),
                        pot_color,
                    };
                    tile.render(tile_w, tile_h)
                },
                Some(crate::models::forest_map::MapElement::Path) => {
                    PathTile.render(tile_w, tile_h)
                },
                _ => {
                    EmptyTile.render(tile_w, tile_h)
                }
            };
            tile_cache.insert((row, col), tile_lines);
        }
    }
    
    let mut all_lines = Vec::new();
    
    for screen_y in 0..screen_height {
        let abs_y = logical_start_y - screen_y as i32;
        let map_row = abs_y.div_euclid(tile_h as i32);
        
        let tile_y = (((map_row + 1) * (tile_h as i32) - 1 - abs_y) as usize).min(tile_h as usize - 1);
        
        let mut combined_spans = Vec::new();
        
        for col in start_col..=end_col {
            if let Some(tile_lines) = tile_cache.get(&(map_row, col)) {
                if tile_y < tile_lines.len() {
                    let line = &tile_lines[tile_y];
                    
                    let x_skip = if col == start_col {
                        logical_start_x.rem_euclid(tile_w as i32) as usize
                    } else {
                        0
                    };
                    
                    let abs_x_end = logical_start_x + screen_width as i32 - 1;
                    let x_take = if col == end_col {
                        let tile_end_x = abs_x_end.rem_euclid(tile_w as i32) as usize;
                        tile_end_x - x_skip + 1
                    } else {
                        (tile_w as usize) - x_skip
                    };
                    
                    let sliced = slice_line(line, x_skip, x_take);
                    for span in sliced.spans {
                        combined_spans.push(span);
                    }
                } else {
                    let x_skip = if col == start_col { logical_start_x.rem_euclid(tile_w as i32) as usize } else { 0 };
                    let abs_x_end = logical_start_x + screen_width as i32 - 1;
                    let x_take = if col == end_col {
                        let tile_end_x = abs_x_end.rem_euclid(tile_w as i32) as usize;
                        tile_end_x - x_skip + 1
                    } else {
                        (tile_w as usize) - x_skip
                    };
                    combined_spans.push(Span::raw(" ".repeat(x_take)));
                }
            }
        }
        all_lines.push(Line::from(combined_spans));
    }
    
    let p = Paragraph::new(all_lines)
        .block(Block::default().borders(Borders::NONE));
        
    frame.render_widget(p, map_area);
}
