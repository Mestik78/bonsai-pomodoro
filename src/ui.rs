use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    // Dividimos la pantalla en 2 verticalmente: Pestañas (3 líneas) y Contenido (el resto)
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(frame.area());

    // Renderizamos las Pestañas
    let titles = app.tab_titles.iter().map(|t| {
        Line::from(Span::styled(*t, Style::default().fg(Color::Green)))
    }).collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Bonsai Pomodoro "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, chunks[0]);

    // Renderizamos el contenido dependiendo de la pestaña
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let content = match app.current_tab {
        0 => "Contenido del Temporizador. Usa las flechas <- y -> para cambiar de pestaña. Presiona 'q' para salir.",
        1 => "Contenido del Bosque. Aquí crecerán tus árboles. Usa las flechas <- y -> para cambiar de pestaña. Presiona 'q' para salir.",
        _ => unreachable!(),
    };

    let paragraph = Paragraph::new(content)
        .block(inner_block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, chunks[1]);
}
