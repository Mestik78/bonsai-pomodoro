use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use tui_big_text::{BigText, PixelSize};
use chrono::DateTime;

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
        let canvas = bonsai::generate_bonsai(seed, 1.0);
        let bonsai_lines = canvas.render(1.0);
        
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
                .lines(vec![time_str.into()])
                .build();

            let status_text = match app.mode {
                AppMode::Normal => state_str,
                _ => "Introduciendo datos...",
            };
            let status_color = match app.mode {
                AppMode::Normal => match active_timer.state {
                    Some(TimerState::New) => Color::Cyan,
                    Some(TimerState::Running) => Color::Green,
                    Some(TimerState::Paused) => Color::Yellow,
                    None => Color::Red,
                },
                _ => Color::Yellow,
            };

            // 1. Draw Bonsai Fullscreen
            let seed = active_timer.seed;
            let progress = 1.0 - (time_to_show as f32 / active_timer.duration as f32).clamp(0.0, 1.0);
            
            let canvas = bonsai::generate_bonsai(seed, progress);
            let bonsai_lines = canvas.render(1.0);
            
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
            
            // 3. Overlay Help Text
            let help_text = "Espacio: Pausar/Reanudar  |  Arr/Aba: Ajustar Minuto  |  Izq/Der: Cambiar Pestaña";
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
                let items: Vec<ListItem> = finished_timers.into_iter().map(|t| {
                    let date_str = match DateTime::parse_from_rfc3339(&t.start_time) {
                        Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
                        Err(_) => t.start_time.clone(),
                    };
                    
                    let duration = t.actual_runtime.unwrap_or(t.duration);
                    let minutes = duration / 60;
                    let seconds = duration % 60;
                    let title = t.title.as_deref().unwrap_or("Sin Título");
                    
                    let header = format!("{} | {} | {:02}:{:02}", title, date_str, minutes, seconds);
                    
                    let mut text_lines = vec![
                        ratatui::text::Line::from(ratatui::text::Span::styled(
                            header,
                            Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)
                        ))
                    ];
                    
                    if let Some(desc) = &t.description {
                        text_lines.push(ratatui::text::Line::from(desc.as_str()));
                    }
                    
                    let mini_canvas = bonsai::generate_bonsai(t.seed, 1.0);
                    let mini_lines = mini_canvas.render(0.5);
                    for line in mini_lines {
                        text_lines.push(line);
                    }
                    
                    ListItem::new(text_lines)
                        .style(Style::default().fg(Color::White))
                }).collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(" Historial del Bosque "))
                    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(list, inner_area, &mut app.bosque_state);
            }
        },
        _ => unreachable!(),
    }
}
