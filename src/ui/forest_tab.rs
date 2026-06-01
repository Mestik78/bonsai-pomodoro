use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::timer::TimerSession;
use crate::models::forest::{ForestLevel, NavDir};
use crate::bonsai;

pub fn render(frame: &mut Frame, app: &mut App, inner_area: Rect) {
    let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
    if finished_timers.is_empty() {
        let p = Paragraph::new("You have no finished sessions yet.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, inner_area);
        return;
    }

    let mut days_order = Vec::new();
    let mut days_map: std::collections::HashMap<String, Vec<&TimerSession>> = std::collections::HashMap::new();
    
    for t in finished_timers {
        let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => t.start_time.clone(),
        };
        if !days_map.contains_key(&date_str) {
            days_order.push(date_str.clone());
            days_map.insert(date_str.clone(), Vec::new());
        }
        days_map.get_mut(&date_str).unwrap().push(t);
    }
    
    let selected_idx = app.forest.selected_day;
    
    let mut items = Vec::new();
    let mut target_line_idx = 0;
    let mut current_line: usize = 0;
    
    let available_width = if app.forest.level == ForestLevel::Bonsai {
        let prev_width = 20.max((inner_area.width as f32 * 0.45) as u16);
        inner_area.width.saturating_sub(prev_width) as usize
    } else {
        inner_area.width as usize
    };

    for (day_idx, date_str) in days_order.iter().enumerate() {
        let is_selected = day_idx == selected_idx;
        let title_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(ratatui::style::Modifier::BOLD)
        };
        
        let day_start_line = current_line;
        
        // Title item
        let mut title_spans = vec![
            Span::styled(format!(" {} ", date_str), title_style),
        ];
        let header_len = date_str.len() + 2;
        let dashes_len = available_width.saturating_sub(header_len);
        title_spans.push(Span::styled("─".repeat(dashes_len), title_style));
        
        items.push(ListItem::new(Line::from(title_spans)));
        current_line += 1;
        
        items.push(ListItem::new(Line::from(""))); // Spacer
        current_line += 1;
        
        let timers_for_day = days_map.get(date_str).unwrap();
        
        let mut timer_blocks: Vec<Vec<Line>> = Vec::new();
        
        let list_zoom = if available_width < 35 { 0.25 } else { 0.5 };
        let block_width = if list_zoom == 0.25 { 15_usize } else { 25_usize };
        let target_tree_height = if list_zoom == 0.25 { 7_usize } else { 13_usize };
        
        for (t_idx, t) in timers_for_day.iter().enumerate() {
            let _time_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.format("%H:%M").to_string(),
                Err(_) => "".to_string(),
            };
            let duration = t.actual_runtime.unwrap_or(t.duration);
            let mins = duration / 60;
            let secs = duration % 60;
            
            let duration_str = format!("{:02}:{:02}", mins, secs);
            
            let d = (duration as f64).max(0.0);
            let x = (d / 3000.0).clamp(0.0, 1.0);
            let progress = (x * x * (3.0 - 2.0 * x)) as f32;
            let mini_canvas = bonsai::generate_bonsai(t.seed, progress);
            let is_bonsai_selected = is_selected && app.forest.level == ForestLevel::Bonsai && t_idx == app.forest.selected_bonsai;
            let pot_color = if is_bonsai_selected { Some(Color::Yellow) } else { None };
            let mini_lines = mini_canvas.render(list_zoom, Some(duration_str), pot_color);
            
            let mut block_lines = Vec::new();
            
            let pad_count = target_tree_height.saturating_sub(mini_lines.len());
            for _ in 0..pad_count {
                block_lines.push(Line::from(""));
            }
            
            for line in mini_lines {
                block_lines.push(line);
            }
            timer_blocks.push(block_lines);
        }
        
        let max_height = timer_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
        let cols = (available_width as usize / block_width).max(1);
        
        if is_selected {
            app.forest.cols = cols;
        }
        
        for (row_idx, row_chunk) in timer_blocks.chunks(cols).enumerate() {
            if is_selected && app.forest.level == ForestLevel::Bonsai {
                let selected_row = app.forest.selected_bonsai / cols;
                if row_idx == selected_row {
                    if app.forest.last_nav_dir == NavDir::Down {
                        target_line_idx = current_line + max_height.saturating_sub(1);
                    } else {
                        if selected_row == 0 {
                            target_line_idx = day_start_line;
                        } else {
                            target_line_idx = current_line;
                        }
                    }
                }
            }
            
            for i in 0..max_height {
                let mut combined_spans = Vec::new();
                for block in row_chunk.iter() {
                    if i < block.len() {
                        let line_len: usize = block[i].spans.iter().map(|s| s.content.chars().count()).sum();
                        for span in block[i].spans.clone() {
                            combined_spans.push(span);
                        }
                        let padding = block_width.saturating_sub(line_len);
                        let pad_span = Span::raw(" ".repeat(padding));
                        combined_spans.push(pad_span);
                    } else {
                        let pad_span = Span::raw(" ".repeat(block_width));
                        combined_spans.push(pad_span);
                    }
                }
                items.push(ListItem::new(Line::from(combined_spans)));
                current_line += 1;
            }
            items.push(ListItem::new(Line::from(""))); // Spacer between rows
            current_line += 1;
        }
        
        if is_selected && app.forest.level == ForestLevel::Day {
            if app.forest.last_nav_dir == NavDir::Down {
                target_line_idx = current_line.saturating_sub(1_usize);
            } else {
                target_line_idx = day_start_line;
            }
        }
    }
    
    app.forest.list_state.select(Some(target_line_idx));
    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().fg(Color::White));
        
    if app.forest.level == ForestLevel::Bonsai {
        let prev_width = 20.max((inner_area.width as f32 * 0.45) as u16);
        let list_width = inner_area.width.saturating_sub(prev_width);
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(list_width), Constraint::Length(prev_width)])
            .split(inner_area);
            
        frame.render_stateful_widget(list, chunks[0], &mut app.forest.list_state);
        
        let details_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray));
        
        let details_area = details_block.inner(chunks[1]);
        frame.render_widget(details_block, chunks[1]);
        
        if selected_idx < days_order.len() {
            let selected_day_str = &days_order[selected_idx];
            if let Some(timers_for_day) = days_map.get(selected_day_str) {
                if app.forest.selected_bonsai < timers_for_day.len() {
                    let selected_bonsai = timers_for_day[app.forest.selected_bonsai];
                    
                    let title = selected_bonsai.title.as_deref().unwrap_or("Untitled");
                    let desc = selected_bonsai.description.as_deref().unwrap_or("");
                    
                    let start_dt = chrono::DateTime::parse_from_rfc3339(&selected_bonsai.start_time).ok();
                    let date_str = start_dt.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default();
                    let time_str = start_dt.map(|dt| dt.format("%H:%M").to_string()).unwrap_or_default();
                    
                    let duration = selected_bonsai.actual_runtime.unwrap_or(selected_bonsai.duration);
                    let mins = duration / 60;
                    let secs = duration % 60;
                    let duration_str = format!("{:02}:{:02}", mins, secs);

                    let d = (duration as f64).max(0.0);
                    let x = (d / 3000.0).clamp(0.0, 1.0);
                    let progress = (x * x * (3.0 - 2.0 * x)) as f32;
                    let canvas = bonsai::generate_bonsai(selected_bonsai.seed, progress);
                    
                    let preview_zoom: f32 = if details_area.width < 20 {
                        0.25
                    } else if details_area.width < 40 {
                        0.5
                    } else {
                        1.0
                    };
                    let list_zoom_val: f32 = if available_width < 35 { 0.25 } else { 0.5 };
                    let zoom = preview_zoom.max(list_zoom_val);
                    let bonsai_lines = canvas.render(zoom, Some(duration_str.clone()), None);
                    
                    let bonsai_height = bonsai_lines.len() as u16;
                    let bottom_height = bonsai_height + 4;
                    
                    let mut details_text = vec![
                        Line::from(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD))),
                    ];
                    
                    if !desc.is_empty() {
                        details_text.push(Line::from(""));
                        details_text.push(Line::from(desc));
                    }
                    
                    details_text.push(Line::from(""));
                    details_text.push(Line::from(Span::styled(format!("{} {}", date_str, time_str), Style::default().fg(Color::DarkGray))));
                    
                    let top_p = Paragraph::new(details_text)
                        .alignment(Alignment::Center)
                        .wrap(ratatui::widgets::Wrap { trim: false });
                        
                    let mut bottom_text = Vec::new();
                    for line in bonsai_lines {
                        bottom_text.push(line);
                    }
                    
                    let bottom_p = Paragraph::new(bottom_text)
                        .alignment(Alignment::Center);
                        
                    let details_vert = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(bottom_height),
                        ])
                        .split(details_area);
                        
                    frame.render_widget(top_p, details_vert[0]);
                    frame.render_widget(bottom_p, details_vert[1]);
                }
            }
        }
    } else {
        frame.render_stateful_widget(list, inner_area, &mut app.forest.list_state);
    }
}
