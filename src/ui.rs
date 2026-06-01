use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
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

    // Renderizamos el marco interno general
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
        
        let help_p = Paragraph::new("¡Pomodoro finalizado! Presiona Tab para cambiar de campo, Enter para guardar.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(help_p, input_chunks[0]);

        let title_style = if *focus == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let title_p = Paragraph::new(title.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Título "))
            .style(title_style);
        frame.render_widget(title_p, input_chunks[1]);

        let desc_style = if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let desc_p = Paragraph::new(description.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Descripción "))
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
        let bonsai_lines = canvas.render(zoom);
        
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
                Some(TimerState::New) => "Nuevo",
                Some(TimerState::Starting(_)) => "Iniciando...",
                Some(TimerState::Running) => "Corriendo",
                Some(TimerState::Paused) => "Pausado",
                None => "Finalizado",
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
                _ => "Introduciendo datos...",
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
            let bonsai_lines = canvas.render(zoom);
            
            let bonsai_p = Paragraph::new(bonsai_lines.clone())
                .alignment(Alignment::Center);
                
            let bonsai_height = bonsai_lines.len() as u16;
            let bonsai_vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(bonsai_height),
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
                "Espacio: Pausar/Reanudar"
            } else if inner_area.width < 52 {
                "Espacio: Pausar/Reanudar  |  Arr/Aba: Ajustar Minuto"
            } else {
                "Espacio: Pausar/Reanudar  |  Arr/Aba: Ajustar Minuto  |  Izq/Der: Cambiar Pestaña"
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
                let p = Paragraph::new("Aún no tienes sesiones finalizadas.")
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
                
                let selected_idx = app.bosque_selected_day;
                
                let mut items = Vec::new();
                let mut target_line_idx = 0;
                let mut current_line = 0;
                
                for (day_idx, date_str) in days_order.into_iter().enumerate() {
                    let is_selected = day_idx == selected_idx;
                    let title_style = if is_selected {
                        if app.bosque_level == crate::app::BosqueLevel::Bonsai {
                            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(ratatui::style::Modifier::BOLD)
                        }
                    } else {
                        Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)
                    };
                    
                    if is_selected && app.bosque_level == crate::app::BosqueLevel::Day {
                        target_line_idx = current_line;
                    }
                    
                    // Title item
                    items.push(ListItem::new(ratatui::text::Line::from(ratatui::text::Span::styled(format!(" {} ", date_str), title_style))));
                    current_line += 1;
                    
                    items.push(ListItem::new(ratatui::text::Line::from(""))); // Spacer
                    current_line += 1;
                    
                    let timers_for_day = days_map.get(&date_str).unwrap();
                    
                    let mut timer_blocks: Vec<Vec<ratatui::text::Line>> = Vec::new();
                    for (t_idx, t) in timers_for_day.iter().enumerate() {
                        let time_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                            Ok(dt) => dt.format("%H:%M").to_string(),
                            Err(_) => "".to_string(),
                        };
                        let duration = t.actual_runtime.unwrap_or(t.duration);
                        let mins = duration / 60;
                        let secs = duration % 60;
                        
                        let header1 = format!("{}  {:02}:{:02}", time_str, mins, secs);
                        
                        let is_bonsai_selected = is_selected && app.bosque_level == crate::app::BosqueLevel::Bonsai && t_idx == app.bosque_selected_bonsai;
                        let header_style = if is_bonsai_selected {
                            Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(ratatui::style::Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Cyan)
                        };
                        
                        let d = (duration as f64).max(0.0);
                        let x = (d / 3000.0).clamp(0.0, 1.0);
                        let progress = (x * x * (3.0 - 2.0 * x)) as f32;
                        let mini_canvas = bonsai::generate_bonsai(t.seed, progress);
                        let mini_lines = mini_canvas.render(0.25);
                        
                        let mut block_lines = Vec::new();
                        block_lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(header1, header_style)));
                        for line in mini_lines {
                            block_lines.push(line);
                        }
                        timer_blocks.push(block_lines);
                    }
                    
                    let block_width = 25;
                    let max_height = timer_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
                    let available_width = inner_area.width as usize;
                    let cols = (available_width / block_width).max(1);
                    
                    if is_selected {
                        app.bosque_cols = cols;
                    }
                    
                    for (row_idx, row_chunk) in timer_blocks.chunks(cols).enumerate() {
                        if is_selected && app.bosque_level == crate::app::BosqueLevel::Bonsai {
                            let selected_row = app.bosque_selected_bonsai / cols;
                            if row_idx == selected_row {
                                target_line_idx = current_line;
                            }
                        }
                        
                        for i in 0..max_height {
                            let mut combined_spans = Vec::new();
                            for block in row_chunk {
                                if i < block.len() {
                                    let line_len: usize = block[i].spans.iter().map(|s| s.content.chars().count()).sum();
                                    combined_spans.extend(block[i].spans.clone());
                                    let padding = block_width.saturating_sub(line_len);
                                    combined_spans.push(ratatui::text::Span::raw(" ".repeat(padding)));
                                } else {
                                    combined_spans.push(ratatui::text::Span::raw(" ".repeat(block_width)));
                                }
                            }
                            items.push(ListItem::new(ratatui::text::Line::from(combined_spans)));
                            current_line += 1;
                        }
                        items.push(ListItem::new(ratatui::text::Line::from(""))); // Spacer between rows
                        current_line += 1;
                    }
                }
                
                app.bosque_state.select(Some(target_line_idx));
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(" Bosque "))
                    .style(Style::default().fg(Color::White));
                    
                frame.render_stateful_widget(list, inner_area, &mut app.bosque_state);
            }
        },
        _ => unreachable!(),
    }
}
