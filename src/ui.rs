use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::app::{App, TimerState, AppMode, TimerSession};
use crate::bonsai;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let titles = app.tab_titles.iter().map(|t| {
        Line::from(Span::styled(*t, Style::default().fg(Color::Green)))
    }).collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Bonsai Pomodoro "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, chunks[0]);

    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    // Render the general inner frame
    frame.render_widget(inner_block.clone(), chunks[1]);
    let inner_area = inner_block.inner(chunks[1]);

    if let AppMode::PostTimerInput { title, description, focus } = &app.mode {
        let horiz_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area);
            
        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(1), // Help text
                Constraint::Length(3), // Title
                Constraint::Length(6), // Description
                Constraint::Min(0),
            ])
            .split(horiz_chunks[0]);
        
        let help_p = Paragraph::new("Pomodoro finished! Press Tab to switch fields, Enter to save.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(help_p, input_chunks[0]);

        let title_style = if *focus == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let title_p = Paragraph::new(title.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Title "))
            .style(title_style);
        frame.render_widget(title_p, input_chunks[1]);

        let desc_style = if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let desc_p = Paragraph::new(description.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .style(desc_style)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(desc_p, input_chunks[2]);
        
        // Draw Bonsai in right half
        let active_timer = app.active_timer();
        let seed = active_timer.seed;
        let actual = active_timer.actual_runtime.unwrap_or(active_timer.duration);
        let d = (actual as f64).max(0.0);
        let x = (d / 3000.0).clamp(0.0, 1.0);
        let progress = (x * x * (3.0 - 2.0 * x)) as f32;
        let canvas = bonsai::generate_bonsai(seed, progress);
        
        let zoom = if inner_area.width < 21 {
            0.25
        } else if inner_area.width < 31 {
            0.5
        } else {
            1.0
        };
        let bonsai_lines = canvas.render(zoom, None, None);
        
        let bonsai_p = Paragraph::new(bonsai_lines.clone())
            .alignment(Alignment::Center);
            
        let bonsai_height = bonsai_lines.len() as u16;
        let bonsai_vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(bonsai_height),
            ])
            .split(horiz_chunks[1]);
            
        frame.render_widget(bonsai_p, bonsai_vert_chunks[1]);

        return;
    }

    match app.current_tab {
        0 => {
            let active_timer = app.active_timer();
            let state_str = match active_timer.state {
                Some(TimerState::New) => "New",
                Some(TimerState::Starting(_)) => "Starting...",
                Some(TimerState::Running) => "Running",
                Some(TimerState::Paused) => "Paused",
                None => "Finished",
            };

            // Calculate formatted time
            let time_to_show = if active_timer.state == Some(TimerState::Running) {
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(app.last_tick).as_secs();
                active_timer.time_left.unwrap_or(active_timer.duration).saturating_sub(elapsed)
            } else {
                active_timer.time_left.unwrap_or(active_timer.duration)
            };
            
            let minutes = time_to_show / 60;
            let seconds = time_to_show % 60;
            let time_str = format!("{:02}:{:02}", minutes, seconds);

            let big_text = BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::default().fg(Color::Green))
                .lines(vec![time_str.clone().into()])
                .build();

            let status_text = match app.mode {
                AppMode::Normal => state_str,
                _ => "Entering data...",
            };
            let status_color = match app.mode {
                AppMode::Normal => match active_timer.state {
                    Some(TimerState::New) => Color::Cyan,
                    Some(TimerState::Starting(_)) => Color::LightYellow,
                    Some(TimerState::Running) => Color::Green,
                    Some(TimerState::Paused) => Color::Yellow,
                    None => Color::Red,
                },
                _ => Color::Yellow,
            };

            // 1. Draw Bonsai Fullscreen
            let seed = active_timer.seed;
            let progress = if active_timer.state == Some(TimerState::New) || matches!(active_timer.state, Some(TimerState::Starting(_))) {
                0.0
            } else {
                let actual = active_timer.duration.saturating_sub(time_to_show);
                let d = (actual as f64).max(0.0);
                let x = (d / 3000.0).clamp(0.0, 1.0);
                (x * x * (3.0 - 2.0 * x)) as f32
            };
            
            let mut canvas = bonsai::generate_bonsai(seed, progress);
            
            if let Some(TimerState::Starting(ref start_time_str)) = active_timer.state {
                if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_time_str) {
                    let now = chrono::Utc::now();
                    let elapsed_f = now.signed_duration_since(start_time).num_milliseconds() as f32 / 1000.0;
                    let animation_progress = (elapsed_f / 0.5).clamp(0.0, 1.0);
                    let y = -10.0 + (10.0 * animation_progress);
                    canvas.cells.insert((0, y.round() as i32), bonsai::BonsaiCell {
                        content: "*".to_string(),
                        color: Color::Yellow,
                    });
                }
            }
            
            let zoom = if inner_area.width < 21 {
                0.25
            } else if inner_area.width < 31 {
                0.5
            } else {
                1.0
            };
            let bonsai_lines = canvas.render(zoom, None, None);
            
            let bonsai_p = Paragraph::new(bonsai_lines.clone())
                .alignment(Alignment::Center);
                
            let bonsai_height = bonsai_lines.len() as u16;
            let bonsai_vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(bonsai_height),
                    Constraint::Length(1),
                ])
                .split(inner_area);
                
            frame.render_widget(bonsai_p, bonsai_vert_chunks[1]);
            
            // 2. Overlay Timer Widget
            let use_compact_timer = inner_area.height < 25 || inner_area.width < 52;

            if use_compact_timer {
                let timer_width = 16.max(status_text.len() as u16 + 4);
                let timer_height = 3;
                let offset_x = 2;
                let offset_y = 1;
                
                let timer_area = ratatui::layout::Rect {
                    x: inner_area.x + offset_x,
                    y: inner_area.y + offset_y,
                    width: timer_width.min(inner_area.width.saturating_sub(offset_x)),
                    height: timer_height.min(inner_area.height.saturating_sub(offset_y)),
                };
                
                let timer_block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(format!(" {} ", status_text), Style::default().fg(status_color).add_modifier(ratatui::style::Modifier::BOLD)));
                    
                frame.render_widget(ratatui::widgets::Clear, timer_area);
                
                let timer_p = Paragraph::new(Span::styled(time_str, Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)))
                    .alignment(Alignment::Center)
                    .block(timer_block);
                    
                frame.render_widget(timer_p, timer_area);
            } else {
                let timer_width = 45; // 39 text + 2 borders + 4 padding
                let timer_height = 11; // 8 text + 2 borders + 1 top padding
                let offset_x = 2;
                let offset_y = 1;
                
                let timer_area = ratatui::layout::Rect {
                    x: inner_area.x + offset_x,
                    y: inner_area.y + offset_y,
                    width: timer_width.min(inner_area.width.saturating_sub(offset_x)),
                    height: timer_height.min(inner_area.height.saturating_sub(offset_y)),
                };
                
                let timer_block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(format!(" {} ", status_text), Style::default().fg(status_color).add_modifier(ratatui::style::Modifier::BOLD)));
                    
                frame.render_widget(ratatui::widgets::Clear, timer_area);
                frame.render_widget(timer_block, timer_area);
                
                let inner_timer_area = ratatui::layout::Rect {
                    x: timer_area.x + 3,
                    y: timer_area.y + 2, // 1 for border + 1 for padding top
                    width: timer_area.width.saturating_sub(6),
                    height: 8, // exact BigText height
                };
                frame.render_widget(big_text, inner_timer_area);
            }
            
            // 3. Overlay Help Text
            let help_text = if inner_area.width < 31 {
                "Space: Pause/Resume"
            } else if inner_area.width < 52 {
                "Space: Pause/Resume  |  Up/Down: Adjust Minute"
            } else {
                "Space: Pause/Resume  |  Up/Down: Adjust Minute  |  Left/Right: Switch Tab"
            };
            let help_p = Paragraph::new(Span::styled(help_text, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center);
                
            let help_area = ratatui::layout::Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(1),
                width: inner_area.width,
                height: 1,
            };
            frame.render_widget(ratatui::widgets::Clear, help_area);
            frame.render_widget(help_p, help_area);
        },
        1 => {
            let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
                     if finished_timers.is_empty() {
                let p = Paragraph::new("You have no finished sessions yet.")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(p, inner_area);
            } else {
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
                
                let selected_idx = app.forest_selected_day;
                
                let mut items = Vec::new();
                let mut target_line_idx = 0;
                let mut current_line: usize = 0;
                
                let available_width = if app.forest_level == crate::app::ForestLevel::Bonsai {
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
                        ratatui::text::Span::styled(format!(" {} ", date_str), title_style),
                    ];
                    let header_len = date_str.len() + 2;
                    let dashes_len = available_width.saturating_sub(header_len);
                    title_spans.push(ratatui::text::Span::styled("─".repeat(dashes_len), title_style));
                    
                    items.push(ListItem::new(ratatui::text::Line::from(title_spans)));
                    current_line += 1;
                    
                    items.push(ListItem::new(ratatui::text::Line::from(""))); // Spacer
                    current_line += 1;
                    
                    let timers_for_day = days_map.get(date_str).unwrap();
                    
                    let mut timer_blocks: Vec<Vec<ratatui::text::Line>> = Vec::new();
                    
                    let list_zoom = if available_width < 35 { 0.25 } else { 0.5 };
                    let block_width = if list_zoom == 0.25 { 15_usize } else { 25_usize };
                    let target_tree_height = if list_zoom == 0.25 { 7_usize } else { 13_usize };
                    
                    for (t_idx, t) in timers_for_day.iter().enumerate() {
                        let time_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
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
                        let is_bonsai_selected = is_selected && app.forest_level == crate::app::ForestLevel::Bonsai && t_idx == app.forest_selected_bonsai;
                        let pot_color = if is_bonsai_selected { Some(Color::Yellow) } else { None };
                        let mini_lines = mini_canvas.render(list_zoom, Some(duration_str), pot_color);
                        
                        let mut block_lines = Vec::new();
                        
                        let pad_count = target_tree_height.saturating_sub(mini_lines.len());
                        for _ in 0..pad_count {
                            block_lines.push(ratatui::text::Line::from(""));
                        }
                        
                        for line in mini_lines {
                            block_lines.push(line);
                        }
                        timer_blocks.push(block_lines);
                    }
                    
                    let max_height = timer_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
                    let cols = (available_width as usize / block_width).max(1);
                    
                    if is_selected {
                        app.forest_cols = cols;
                    }
                    
                    for (row_idx, row_chunk) in timer_blocks.chunks(cols).enumerate() {
                        if is_selected && app.forest_level == crate::app::ForestLevel::Bonsai {
                            let selected_row = app.forest_selected_bonsai / cols;
                            if row_idx == selected_row {
                                if app.forest_last_nav_dir == crate::app::NavDir::Down {
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
                            for (block_idx, block) in row_chunk.iter().enumerate() {
                                if i < block.len() {
                                    let line_len: usize = block[i].spans.iter().map(|s| s.content.chars().count()).sum();
                                    for mut span in block[i].spans.clone() {
                                        combined_spans.push(span);
                                    }
                                    let padding = block_width.saturating_sub(line_len);
                                    let pad_span = ratatui::text::Span::raw(" ".repeat(padding));
                                    combined_spans.push(pad_span);
                                } else {
                                    let pad_span = ratatui::text::Span::raw(" ".repeat(block_width));
                                    combined_spans.push(pad_span);
                                }
                            }
                            items.push(ListItem::new(ratatui::text::Line::from(combined_spans)));
                            current_line += 1;
                        }
                        items.push(ListItem::new(ratatui::text::Line::from(""))); // Spacer between rows
                        current_line += 1;
                    }
                    
                    if is_selected && app.forest_level == crate::app::ForestLevel::Day {
                        if app.forest_last_nav_dir == crate::app::NavDir::Down {
                            target_line_idx = current_line.saturating_sub(1_usize);
                        } else {
                            target_line_idx = day_start_line;
                        }
                    }
                }
                
                app.forest_state.select(Some(target_line_idx));
                let list = List::new(items)
                    .block(Block::default())
                    .style(Style::default().fg(Color::White));
                    
                if app.forest_level == crate::app::ForestLevel::Bonsai {
                    let prev_width = 20.max((inner_area.width as f32 * 0.45) as u16);
                    let list_width = inner_area.width.saturating_sub(prev_width);
                    
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(list_width), Constraint::Length(prev_width)])
                        .split(inner_area);
                        
                    frame.render_stateful_widget(list, chunks[0], &mut app.forest_state);
                    
                    let details_block = Block::default()
                        .borders(Borders::LEFT)
                        .border_style(Style::default().fg(Color::DarkGray));
                    
                    let details_area = details_block.inner(chunks[1]);
                    frame.render_widget(details_block, chunks[1]);
                    
                    if selected_idx < days_order.len() {
                        let selected_day_str = &days_order[selected_idx];
                        if let Some(timers_for_day) = days_map.get(selected_day_str) {
                            if app.forest_selected_bonsai < timers_for_day.len() {
                                let selected_bonsai = timers_for_day[app.forest_selected_bonsai];
                                
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
                                    ratatui::text::Line::from(ratatui::text::Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD))),
                                ];
                                
                                if !desc.is_empty() {
                                    details_text.push(ratatui::text::Line::from(""));
                                    details_text.push(ratatui::text::Line::from(desc));
                                }
                                
                                details_text.push(ratatui::text::Line::from(""));
                                details_text.push(ratatui::text::Line::from(ratatui::text::Span::styled(format!("{} {}", date_str, time_str), Style::default().fg(Color::DarkGray))));
                                
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
                    frame.render_stateful_widget(list, inner_area, &mut app.forest_state);
                }
            }
        },
        2 => {
            let prev_width = 20.max((inner_area.width as f32 * 0.45) as u16);
            let list_width = inner_area.width.saturating_sub(prev_width);
            
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(prev_width), Constraint::Length(list_width)])
                .split(inner_area);
            
            let mut items = Vec::new();
            let options = ["Daily", "Weekly", "Monthly", "Heatmap"];
            for (i, opt) in options.iter().enumerate() {
                let is_selected = app.stats_state.selected() == Some(i);
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray).add_modifier(ratatui::style::Modifier::BOLD)
                };
                items.push(ListItem::new(ratatui::text::Line::from(ratatui::text::Span::styled(format!(" {} ", opt), style))));
            }
            
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(Color::DarkGray)))
                .style(Style::default().fg(Color::White));
                
            frame.render_stateful_widget(list, chunks[0], &mut app.stats_state);
            
            if app.stats_state.selected() == Some(0) {
                let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
                
                let mut days_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
                let mut days_order = Vec::new();
                
                for t in finished_timers {
                    let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                        Ok(dt) => dt.format("%m-%d").to_string(), // Short date for bar chart
                        Err(_) => t.start_time.chars().take(5).collect(),
                    };
                    
                    if !days_map.contains_key(&date_str) {
                        days_order.push(date_str.clone());
                        days_map.insert(date_str.clone(), 0);
                    }
                    
                    let duration = t.actual_runtime.unwrap_or(t.duration);
                    let mins = duration / 60;
                    *days_map.get_mut(&date_str).unwrap() += mins;
                }
                
                days_order.reverse();
                
                let data: Vec<(&str, u64)> = days_order.iter()
                    .map(|day| {
                        (day.as_str(), *days_map.get(day).unwrap_or(&0))
                    })
                    .collect();

                let right_block = Block::default()
                    .borders(Borders::NONE)
                    .title(Span::styled(" Minutes per day ", Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)));
                
                if data.is_empty() {
                    let p = Paragraph::new("No data available.")
                        .alignment(Alignment::Center)
                        .block(right_block);
                    frame.render_widget(p, chunks[1]);
                } else {
                    let bars: Vec<Bar> = data.iter().map(|(label, value)| {
                        let color = if *value >= 180 {
                            Color::Red
                        } else if *value >= 120 {
                            Color::Yellow
                        } else if *value >= 60 {
                            Color::LightGreen
                        } else if *value >= 30 {
                            Color::Green
                        } else {
                            Color::DarkGray
                        };
                        
                        Bar::default()
                            .label(Line::from(label.to_string()))
                            .value(*value)
                            .style(Style::default().fg(color))
                            .value_style(Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD))
                    }).collect();

                    let bar_chart = BarChart::default()
                        .block(right_block)
                        .data(BarGroup::default().bars(&bars))
                        .bar_width(6)
                        .bar_gap(2)
                        .bar_set(ratatui::symbols::bar::NINE_LEVELS);
                        
                    frame.render_widget(bar_chart, chunks[1]);
                }
            } else if app.stats_state.selected() == Some(3) {
                use chrono::{Datelike, TimeZone, Utc, Duration, Local};
                let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
                
                let mut days_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
                for t in finished_timers {
                    let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                        Ok(dt) => dt.with_timezone(&Local).format("%Y-%m-%d").to_string(),
                        Err(_) => "".to_string(),
                    };
                    if !date_str.is_empty() {
                        let duration = t.actual_runtime.unwrap_or(t.duration);
                        let mins = duration / 60;
                        *days_map.entry(date_str).or_insert(0) += mins;
                    }
                }
                
                let today = Local::now().date_naive();
                let year = today.year();
                let jan1 = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let dec31 = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
                
                let start_date = jan1 - Duration::days(jan1.weekday().num_days_from_monday() as i64);
                let end_date = dec31 + Duration::days(6 - dec31.weekday().num_days_from_monday() as i64);
                let total_weeks = ((end_date - start_date).num_days() / 7) as usize + 1;
                
                let max_weeks_per_chunk = (chunks[1].width.saturating_sub(4) as usize).max(1);
                let num_chunks = (total_weeks + max_weeks_per_chunk - 1) / max_weeks_per_chunk;
                
                let mut lines = Vec::new();
                lines.push(ratatui::text::Line::from("")); // Top padding
                let day_labels = ["M", "T", "W", "T", "F", "S", "S"];
                
                for chunk_idx in 0..num_chunks {
                    if chunk_idx > 0 {
                        lines.push(ratatui::text::Line::from("")); // Gap between chunks
                    }
                    
                    let chunk_start_week = chunk_idx * max_weeks_per_chunk;
                    let chunk_end_week = (chunk_start_week + max_weeks_per_chunk).min(total_weeks);
                    let chunk_weeks = chunk_end_week - chunk_start_week;
                    
                    for row in 0..7 {
                        let mut spans = Vec::new();
                        spans.push(ratatui::text::Span::styled(format!("{} ", day_labels[row]), Style::default().fg(Color::DarkGray)));
                        
                        for col in 0..chunk_weeks {
                            let week_idx = chunk_start_week + col;
                            let cell_date = start_date + Duration::weeks(week_idx as i64) + Duration::days(row as i64);
                            
                            let is_future = cell_date > today;
                            let is_outside_year = cell_date.year() != year;
                            let is_today = cell_date == today;
                            
                            let color = if is_outside_year {
                                Color::Reset
                            } else {
                                let date_str = cell_date.format("%Y-%m-%d").to_string();
                                let mins = days_map.get(&date_str).unwrap_or(&0);
                                if *mins >= 180 {
                                    Color::Red
                                } else if *mins >= 120 {
                                    Color::Yellow
                                } else if *mins >= 60 {
                                    Color::LightGreen
                                } else if *mins >= 30 {
                                    Color::Green
                                } else if *mins > 0 {
                                    Color::DarkGray
                                } else {
                                    Color::Rgb(30, 30, 30) // Very dark gray for empty days
                                }
                            };
                            
                            if is_outside_year {
                                spans.push(ratatui::text::Span::raw(" "));
                            } else {
                                let content = if is_today { "x" } else { " " };
                                spans.push(ratatui::text::Span::styled(content, Style::default().bg(color).fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD)));
                            }
                        }
                        lines.push(ratatui::text::Line::from(spans));
                    }
                }
                
                let right_block = Block::default()
                    .borders(Borders::NONE)
                    .title(Span::styled(" Activity Heatmap ", Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)));
                    
                let p = Paragraph::new(lines)
                    .alignment(Alignment::Center)
                    .block(right_block);
                frame.render_widget(p, chunks[1]);
            } else {
                let p = Paragraph::new("\n\nWork in progress...\nThis view will be implemented soon.")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::NONE));
                frame.render_widget(p, chunks[1]);
            }
        },
        _ => unreachable!(),
    }
}
