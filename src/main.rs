mod app;
mod ui;

use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run application loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    // Definimos un tick rate máximo de 250ms para que la UI sea muy responsiva
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // event::poll espera hasta `tick_rate` a ver si hay un evento de teclado.
        // Si no hay evento, devuelve false y el loop continúa.
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Left => app.previous_tab(),
                        KeyCode::Right | KeyCode::Tab => app.next_tab(),
                        KeyCode::Char(' ') => app.toggle_timer(),
                        KeyCode::Up => {
                            if app.current_tab == 0 {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.add_seconds(1);
                                } else {
                                    app.add_minutes(1);
                                }
                            }
                        },
                        KeyCode::Down => {
                            if app.current_tab == 0 {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.add_seconds(-1);
                                } else {
                                    app.add_minutes(-1);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        // Llamamos a on_tick para que la app descuente el tiempo si procede
        app.on_tick();

        if app.should_quit {
            app.save_state();
            return Ok(());
        }
    }
}
