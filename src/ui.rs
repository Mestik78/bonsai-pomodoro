use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
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

    match app.current_tab {
        0 => {
            // Pestaña Temporizador
            let minutes = app.time_left / 60;
            let seconds = app.time_left % 60;
            let timer_text = format!("{:02}:{:02}", minutes, seconds);

            // Dividimos verticalmente para centrar el BigText y los mensajes
            let vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),      // padding superior
                    Constraint::Length(8),   // espacio para BigText
                    Constraint::Length(2),   // espacio vacío
                    Constraint::Length(1),   // [ CORRIENDO ]
                    Constraint::Length(2),   // espacio vacío
                    Constraint::Length(1),   // instrucciones
                    Constraint::Min(0),      // padding inferior
                ])
                .split(inner_area);

            // BigText ocupa todo el ancho y se ajusta a la izquierda por defecto.
            // Para centrarlo horizontalmente, hacemos otra división en la fila del BigText:
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(39),  // Ajustado al ancho de "MM:SS"
                    Constraint::Min(0),
                ])
                .split(vert_chunks[1]);

            let big_text = BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::default().fg(Color::White))
                .lines(vec![timer_text.into()])
                .build();

            frame.render_widget(big_text, horiz_chunks[1]);

            // Indicador de estado
            let status_text = if app.is_running {
                "[ CORRIENDO ]"
            } else {
                "[ PAUSADO ]"
            };
            let status_color = if app.is_running { Color::Green } else { Color::Yellow };
            let status_p = Paragraph::new(Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)))
                .alignment(Alignment::Center);
            frame.render_widget(status_p, vert_chunks[3]);

            // Instrucciones
            let help_text = "Espacio: Pausar/Reanudar  |  Arr/Aba: Ajustar Minuto  |  Izq/Der: Cambiar Pestaña";
            let help_p = Paragraph::new(Span::styled(help_text, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center);
            frame.render_widget(help_p, vert_chunks[5]);
        },
        1 => {
            // Pestaña Bosque
            let vert_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner_area);
                
            let help_text = "El bosque está descansando. Aquí crecerán tus árboles en futuras versiones.";
            let paragraph = Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
                
            frame.render_widget(paragraph, vert_chunks[1]);
        },
        _ => unreachable!(),
    }
}
