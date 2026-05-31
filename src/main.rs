mod app;
mod ui;

use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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
    loop {
        // Dibujamos la pantalla en cada iteración
        terminal.draw(|f| ui::render(f, app))?;

        // Atrapamos eventos del teclado
        if let Event::Key(key) = event::read()? {
            // Solo reaccionamos si es presionar tecla, no soltarla (importante en Windows/algunas TTY)
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Right | KeyCode::Tab => app.next_tab(),
                    KeyCode::Left | KeyCode::BackTab => app.previous_tab(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
