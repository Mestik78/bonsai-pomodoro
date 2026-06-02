use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;


pub fn render(frame: &mut Frame, app: &App, inner_area: Rect, title: &str, description: &str, focus: u8) {
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

    let title_style = if focus == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
    let title_p = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).title(" Title "))
        .style(title_style);
    frame.render_widget(title_p, input_chunks[1]);

    let desc_style = if focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) };
    let desc_p = Paragraph::new(description)
        .block(Block::default().borders(Borders::ALL).title(" Description "))
        .style(desc_style)
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(desc_p, input_chunks[2]);
    
    // Draw Bonsai in right half
    let active_timer = app.active_timer();
    let plant = crate::models::plant::Plant::from_timer(active_timer);
    let canvas = crate::models::plant::generate_plant(&plant);
    
    let plant_frame = crate::bonsai::PlantFrame::new(Some(0), None);
    let bonsai_lines = plant_frame.render(&canvas, horiz_chunks[1].width, horiz_chunks[1].height, None, None);
    
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
}
