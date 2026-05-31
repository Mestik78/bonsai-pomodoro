use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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

    let (content, style) = match app.current_tab {
        0 => {
            let mut text = Text::default();
            for line in app.bonsai_ansi.lines() {
                let mut spans = vec![];
                for c in line.chars() {
                    let style = match c {
                        '&' | '~' | 'v' | '*' => Style::default().fg(Color::LightGreen),
                        '|' | '\\' | '/' | '_' | '(' | ')' | '<' | '>' => Style::default().fg(Color::Rgb(139, 69, 19)), // Marrón
                        ':' | '.' | '-' | '[' | ']' => Style::default().fg(Color::DarkGray),
                        _ => Style::default().fg(Color::White),
                    };
                    spans.push(Span::styled(c.to_string(), style));
                }
                text.lines.push(Line::from(spans));
            }
            (text, Style::default())
        },
        1 => (
            Text::raw("Contenido del Bosque. Aquí crecerán tus árboles. Usa <- y -> para cambiar de pestaña. Presiona 'q' para salir."),
            Style::default().fg(Color::White)
        ),
        _ => unreachable!(),
    };

    let paragraph = Paragraph::new(content)
        .block(inner_block)
        .style(style);

    frame.render_widget(paragraph, chunks[1]);
}
