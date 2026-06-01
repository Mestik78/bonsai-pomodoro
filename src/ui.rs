use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::app::{App, TimerState, AppMode};

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
            // Pestaña Temporizador
            let timer = app.active_timer();
            let time_left = timer.time_left.unwrap_or(0);
            let minutes = time_left / 60;
            let seconds = time_left % 60;
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
            let status_text = match timer.state {
                Some(TimerState::New) => "[ NUEVO ]",
                Some(TimerState::Running) => "[ CORRIENDO ]",
                Some(TimerState::Paused) => "[ PAUSADO ]",
                None => "[ FINALIZADO ]",
            };
            let status_color = match timer.state {
                Some(TimerState::New) => Color::Cyan,
                Some(TimerState::Running) => Color::Green,
                Some(TimerState::Paused) => Color::Yellow,
                None => Color::Red,
            };
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
