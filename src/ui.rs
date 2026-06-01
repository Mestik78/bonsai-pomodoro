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
        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(1), // Help text
                Constraint::Length(3), // Title
                Constraint::Length(6), // Description
                Constraint::Min(0),
            ])
            .split(inner_area);
        
        let help_p = Paragraph::new("¡Pomodoro finalizado! Presiona Tab para cambiar de campo, Enter para guardar.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(help_p, input_chunks[0]);

        let title_style = if *focus == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let title_cursor = if *focus == 0 { "_" } else { "" };
        let title_p = Paragraph::new(format!("{}{}", title, title_cursor))
            .block(Block::default().borders(Borders::ALL).title(" Título "))
            .style(title_style);
        frame.render_widget(title_p, input_chunks[1]);

        let desc_style = if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
        let desc_cursor = if *focus == 1 { "_" } else { "" };
        let desc_p = Paragraph::new(format!("{}{}", description, desc_cursor))
            .block(Block::default().borders(Borders::ALL).title(" Descripción "))
            .style(desc_style)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(desc_p, input_chunks[2]);

        return;
    }

    match app.current_tab {
        0 => {
            // Split main area horizontally: left for timer, right for bonsai
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(inner_area);
                
            let vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(8), // Timer
                    Constraint::Length(1), // Spacer
                    Constraint::Length(1), // Status
                    Constraint::Min(0),
                    Constraint::Length(1), // Help
                ])
                .split(horiz_chunks[0]);

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

            frame.render_widget(big_text, vert_chunks[1]);

            let status_text = match app.mode {
                AppMode::Normal => state_str,
                _ => "Introduciendo datos...",
            };
            let status_color = match app.mode {
                AppMode::Normal => Color::White,
                _ => Color::Yellow,
            };
            let status_p = Paragraph::new(Span::styled(status_text, Style::default().fg(status_color).add_modifier(ratatui::style::Modifier::BOLD)))
                .alignment(Alignment::Center);
            frame.render_widget(status_p, vert_chunks[3]);

            let help_text = "Espacio: Pausar/Reanudar  |  Arr/Aba: Ajustar Minuto  |  Izq/Der: Cambiar Pestaña";
            let help_p = Paragraph::new(Span::styled(help_text, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center);
            frame.render_widget(help_p, vert_chunks[5]);
            
            // Draw Bonsai in right chunk
            let seed = active_timer.seed;
            // The active tree grows based on time spent. 
            // If duration is 25 min, max life is 32. 
            // So life = (1.0 - time_left / duration) * 32
            // Wait, just grow it fully if we want. Let's make it grow over time!
            let progress = 1.0 - (time_to_show as f32 / active_timer.duration as f32).clamp(0.0, 1.0);
            let life = (progress * 32.0) as i32;
            let life = life.max(4); // minimum life
            
            let canvas = bonsai::generate_bonsai(seed, life, 5);
            let bonsai_lines = canvas.render(1.0);
            
            let bonsai_p = Paragraph::new(bonsai_lines.clone())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Tu Bonsái "));
                
            let right_height = horiz_chunks[1].height;
            let bonsai_height = (bonsai_lines.len() as u16 + 2).min(right_height);
            let right_vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(bonsai_height),
                ])
                .split(horiz_chunks[1]);
                
            frame.render_widget(bonsai_p, right_vert_chunks[1]);
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
                    
                    // Render mini bonsai
                    let mini_canvas = bonsai::generate_bonsai(t.seed, 32, 5);
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
