pub mod timer_tab;
pub mod forest_tab;
pub mod stats_tab;
pub mod plants_tab;
pub mod post_timer;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::{App, AppMode};
use crate::models::tabs::Tab;

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

    let tab_index = match app.current_tab {
        Tab::Timer => 0,
        Tab::Forest => 1,
        Tab::Stats => 2,
        Tab::Plants => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Bonsai Pomodoro "))
        .select(tab_index)
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
