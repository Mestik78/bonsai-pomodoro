pub mod timer_tab;
pub mod forest_tab;
pub mod stats_tab;
pub mod plants_tab;
pub mod post_timer;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::{App, AppMode};
use crate::models::tabs::Tab;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let tab_index = match app.current_tab {
        Tab::Timer => 0,
        Tab::Forest => 1,
        Tab::Stats => 2,
        Tab::Plants => 3,
    };

    let mut tab_spans = Vec::new();
    let mut border_spans = Vec::new();

    for (i, title) in app.tab_titles.iter().enumerate() {
        let is_selected = i == tab_index;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        let title_str = format!(" {} ", title);
        let width = title_str.chars().count();
        
        tab_spans.push(Span::styled(title_str, style));
        border_spans.push(Span::styled("─".repeat(width), Style::default().fg(Color::DarkGray)));
        
        if i < app.tab_titles.len() - 1 {
            tab_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
            border_spans.push(Span::styled("┴", Style::default().fg(Color::DarkGray)));
        }
    }
    
    border_spans.push(Span::styled("─".repeat(200), Style::default().fg(Color::DarkGray)));

    let tabs_widget = ratatui::widgets::Paragraph::new(Line::from(tab_spans));
    let border_widget = ratatui::widgets::Paragraph::new(Line::from(border_spans));

    frame.render_widget(tabs_widget, chunks[0]);
    frame.render_widget(border_widget, chunks[1]);

    let inner_block = Block::default()
        .borders(Borders::NONE);

    // Render the general inner frame
    frame.render_widget(inner_block.clone(), chunks[2]);
    let inner_area = inner_block.inner(chunks[2]);

    if let AppMode::PostTimerInput { title, description, focus } = &app.mode {
        post_timer::render(frame, app, inner_area, title.as_str(), description.as_str(), *focus);
        return;
    }

    match app.current_tab {
        Tab::Timer => timer_tab::render(frame, app, inner_area),
        Tab::Forest => forest_tab::render(frame, app, inner_area),
        Tab::Stats => stats_tab::render(frame, app, inner_area),
        Tab::Plants => plants_tab::render(frame, app, inner_area),
    }
}
