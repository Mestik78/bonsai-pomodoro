use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::timer::TimerSession;
use crate::models::history::{HistoryLevel, NavDir};
use crate::bonsai;


pub fn render(frame: &mut Frame, app: &mut App, inner_area: Rect) {
    let mut title_text = " History ".to_string();
    if app.history.is_searching {
        title_text = format!(" /{}_ ", app.history.search_query);
    } else if !app.history.search_matches.is_empty() {
        title_text = format!(" Match {}/{} for '{}' (n to jump, Esc to clear) ", app.history.search_index + 1, app.history.search_matches.len(), app.history.search_query);
    }
    
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.history.is_searching { Color::Yellow } else { Color::DarkGray }))
        .title(title_text);
        
    let history_area = main_block.inner(inner_area);
    frame.render_widget(main_block, inner_area);
    
    let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
    if finished_timers.is_empty() {
        let p = Paragraph::new("You have no finished sessions yet.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, history_area);
        return;
    }

    let days_order = &app.history.cached_days;
    let mut days_map: std::collections::HashMap<String, Vec<&TimerSession>> = std::collections::HashMap::new();
    
    for t in finished_timers {
        let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => t.start_time.clone(),
        };
        if !days_map.contains_key(&date_str) {
            days_map.insert(date_str.clone(), Vec::new());
        }
        days_map.get_mut(&date_str).unwrap().push(t);
    }
    
    let selected_idx = app.history.selected_day;
    
    let mut items = Vec::new();
    let mut target_line_idx = 0;
    let mut current_line: usize = 0;
    let mut hovered_selection = None;
    let hovered_line = if app.mouse_moved_this_frame {
        if let Some((mx, my)) = app.mouse_pos {
            if mx >= history_area.x && mx < history_area.x + history_area.width && my >= history_area.y && my < history_area.y + history_area.height {
                Some(my as usize - history_area.y as usize + app.history.list_state.offset())
            } else { None }
        } else { None }
    } else { None };
    let mx = app.mouse_pos.map(|p| p.0).unwrap_or(0);

    let available_width = if app.history.level == HistoryLevel::Bonsai {
        let base_width = (history_area.width as f32 * 0.50) as u16;
        let prev_width = base_width.clamp(10, crate::bonsai::scale::SCALES[0].min_width + 10);
        history_area.width.saturating_sub(prev_width) as usize
    } else {
        history_area.width as usize
    };

    for (day_idx, date_str) in days_order.iter().enumerate() {
        let is_selected = day_idx == selected_idx;
        let title_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(ratatui::style::Modifier::BOLD)
        };
        
        let day_start_line = current_line;
        
        let timers_for_day = days_map.get(date_str).unwrap();
        
        let total_secs_day: u64 = timers_for_day.iter().map(|t| t.actual_runtime.unwrap_or(t.duration)).sum();
        let total_mins = total_secs_day / 60;
        let total_hours = total_mins / 60;
        let rem_mins = total_mins % 60;
        
        let time_str = if total_hours > 0 {
            format!("─ {}h {:02}m ", total_hours, rem_mins)
        } else {
            format!("─ {}m ", total_mins)
        };
        
        let left_dashes = 3;
        let title_text = format!(" {} ", date_str);
        
        // Title item
        let mut title_spans = vec![
            Span::styled("─".repeat(left_dashes), title_style),
            Span::styled(title_text.clone(), title_style),
            Span::styled(time_str.clone(), title_style),
        ];
        let header_len = left_dashes + title_text.chars().count() + time_str.chars().count();
        let dashes_len = available_width.saturating_sub(header_len);
        title_spans.push(Span::styled("─".repeat(dashes_len), title_style));
        if hovered_line == Some(current_line) {
            hovered_selection = Some((day_idx, None));
        }
        items.push(ListItem::new(Line::from(title_spans)));
        current_line += 1;
        
        items.push(ListItem::new(Line::from(""))); // Spacer
        current_line += 1;
        

        
        let mut timer_blocks: Vec<Vec<Line>> = Vec::new();
        
        let block_width = if available_width < 35 { 15_usize } else { 25_usize };
        let target_tree_height = if available_width < 35 { 7_usize } else { 13_usize };
        
        for (t_idx, t) in timers_for_day.iter().enumerate() {
            let _time_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.format("%H:%M").to_string(),
                Err(_) => "".to_string(),
            };
            let duration = t.actual_runtime.unwrap_or(t.duration);
            let mins = duration / 60;
            let secs = duration % 60;
            
            let duration_str = format!("{:02}:{:02}", mins, secs);
            
            let plant = crate::models::plant::Plant::from_timer(t);
            let mini_canvas = crate::models::plant::generate_plant(&plant);
            let is_bonsai_selected = is_selected && app.history.level == HistoryLevel::Bonsai && t_idx == app.history.selected_bonsai;
            let pot_color = if is_bonsai_selected { Some(Color::Yellow) } else { None };
            
            let plant_frame = bonsai::PlantFrame::new(Some(1), None); // Max scale 0.5 (index 1)
            let mini_lines = plant_frame.render(&mini_canvas, available_width as u16, target_tree_height as u16, Some(duration_str.to_string()), pot_color);
            
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
            app.history.cols = cols;
        }
        
        for (row_idx, row_chunk) in timer_blocks.chunks(cols).enumerate() {
            if is_selected && app.history.level == HistoryLevel::Bonsai {
                let selected_row = app.history.selected_bonsai / cols;
                if row_idx == selected_row {
                    let list_height = history_area.height as usize;
                    let block_start = current_line;
                    let block_end = current_line + max_height.saturating_sub(1);
                    
                    let offset = app.history.list_state.offset();
                    
                    if block_end >= offset + list_height {
                        target_line_idx = block_end;
                    } else if block_start < offset {
                        target_line_idx = block_start;
                    } else {
                        target_line_idx = block_start;
                    }
                }
            }
            
            let block_start = current_line;
            let block_end = current_line + max_height.saturating_sub(1);
            if let Some(hl) = hovered_line {
                if hl >= block_start && hl <= block_end {
                    let rel_x = mx.saturating_sub(history_area.x) as usize;
                    let col_idx = rel_x / block_width;
                    if col_idx < row_chunk.len() {
                        hovered_selection = Some((day_idx, Some(row_idx * cols + col_idx)));
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
        
        if is_selected && app.history.level == HistoryLevel::Day {
            let list_height = history_area.height as usize;
            if app.history.last_nav_dir == NavDir::Down {
                let day_height = current_line - day_start_line;
                if day_height <= list_height {
                    target_line_idx = current_line.saturating_sub(1);
                } else {
                    target_line_idx = day_start_line + list_height.saturating_sub(1);
                }
            } else {
                target_line_idx = day_start_line;
            }
        }
    }
    
    app.history.list_state.select(Some(target_line_idx));
    if let Some((day_idx, bonsai_idx_opt)) = hovered_selection {
        app.history.selected_day = day_idx;
        if let Some(b_idx) = bonsai_idx_opt {
            app.history.level = HistoryLevel::Bonsai;
            app.history.selected_bonsai = b_idx;
        } else {
            app.history.level = HistoryLevel::Day;
        }
    }

    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().fg(Color::White));
        
    if app.history.level == HistoryLevel::Bonsai {
        let base_width = (history_area.width as f32 * 0.50) as u16;
        let prev_width = base_width.clamp(10, crate::bonsai::scale::SCALES[0].min_width + 10);
        let list_width = history_area.width.saturating_sub(prev_width);
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(list_width), Constraint::Length(prev_width)])
            .split(history_area);
            
        frame.render_stateful_widget(list, chunks[0], &mut app.history.list_state);
        
        let details_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray));
        
        let details_area = details_block.inner(chunks[1]);
        frame.render_widget(details_block, chunks[1]);
        
        if selected_idx < days_order.len() {
            let selected_day_str = &days_order[selected_idx];
            if let Some(timers_for_day) = days_map.get(selected_day_str) {
                if app.history.selected_bonsai < timers_for_day.len() {
                    let selected_bonsai = timers_for_day[app.history.selected_bonsai];
                    
                    let title = selected_bonsai.title.as_deref().unwrap_or("Untitled");
                    let desc = selected_bonsai.description.as_deref().unwrap_or("");
                    
                    let start_dt = chrono::DateTime::parse_from_rfc3339(&selected_bonsai.start_time).ok();
                    let date_str = start_dt.map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default();
                    let time_str = start_dt.map(|dt| dt.format("%H:%M").to_string()).unwrap_or_default();
                    
                    let duration = selected_bonsai.actual_runtime.unwrap_or(selected_bonsai.duration);
                    let mins = duration / 60;
                    let secs = duration % 60;
                    let duration_str = format!("{:02}:{:02}", mins, secs);

                    let plant = crate::models::plant::Plant::from_timer(selected_bonsai);
                    let canvas = crate::models::plant::generate_plant(&plant);
                    
                    let plant_frame = bonsai::PlantFrame::new(Some(0), None);
                    let bonsai_lines = plant_frame.render(&canvas, details_area.width, details_area.height, Some(duration_str.clone()), None);
                    
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
        frame.render_stateful_widget(list, history_area, &mut app.history.list_state);
    }
}
