use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::app::{App, AppMode};
use crate::models::timer::TimerState;
use crate::bonsai;

pub fn render(frame: &mut Frame, app: &App, inner_area: Rect) {
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
    
    let plant = crate::models::plant::Plant::new(seed, active_timer.plant_type.clone(), progress);
    let mut canvas = crate::models::plant::generate_plant(&plant);
    
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
    
    let plant_frame = bonsai::PlantFrame::new(0);
    let bonsai_lines = plant_frame.render(&canvas, inner_area.width, inner_area.height, None, None);
    
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
}
