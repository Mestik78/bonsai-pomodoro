mod app;
mod ui;
mod bonsai;

use std::{io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, AppMode};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let is_production = args.contains(&"--user".to_string());
    
    if args.contains(&"--bonsai".to_string()) {
        let seed = args.iter().position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| rand::random());
            
        let zoom = args.iter().position(|a| a == "--zoom")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
            
        let canvas = bonsai::generate_bonsai(seed, 32, 5);
        let lines = canvas.render(zoom);
        for line in lines {
            for span in line.spans {
                if let Some(c) = span.style.fg {
                    match c {
                        ratatui::style::Color::Green | ratatui::style::Color::LightGreen => print!("\x1b[32m"),
                        ratatui::style::Color::DarkGray => print!("\x1b[90m"),
                        ratatui::style::Color::Rgb(r, g, b) => print!("\x1b[38;2;{};{};{}m", r, g, b),
                        _ => print!("\x1b[0m"),
                    }
                }
                print!("{}", span.content);
                print!("\x1b[0m");
            }
            println!();
        }
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(is_production);

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
                    match app.mode {
                        AppMode::Normal => {
                            match key.code {
                                KeyCode::Char('q') => app.quit(),
                                KeyCode::Left => app.previous_tab(),
                                KeyCode::Right | KeyCode::Tab => app.next_tab(),
                                KeyCode::Char(' ') => app.toggle_timer(),
                                KeyCode::Char('r') => {
                                    if app.current_tab == 0 {
                                        app.reset_timer();
                                    }
                                },
                                KeyCode::Char('f') => {
                                    if app.current_tab == 0 {
                                        app.finish_early();
                                    }
                                },
                                KeyCode::Up => {
                                    if app.current_tab == 0 {
                                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                                            app.add_seconds(1);
                                        } else {
                                            app.add_minutes(1);
                                        }
                                    } else if app.current_tab == 1 {
                                        app.bosque_previous();
                                    }
                                },
                                KeyCode::Down => {
                                    if app.current_tab == 0 {
                                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                                            app.add_seconds(-1);
                                        } else {
                                            app.add_minutes(-1);
                                        }
                                    } else if app.current_tab == 1 {
                                        app.bosque_next();
                                    }
                                },
                                _ => {}
                            }
                        },
                        AppMode::PostTimerInput { .. } => {
                            match key.code {
                                KeyCode::Char(c) => app.input_char(c),
                                KeyCode::Backspace => app.input_backspace(),
                                KeyCode::BackTab => app.input_tab(true),
                                KeyCode::Tab => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.input_tab(true);
                                    } else {
                                        app.input_tab(false);
                                    }
                                },
                                KeyCode::Enter => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        app.input_char('\n');
                                    } else {
                                        app.submit_input();
                                    }
                                },
                                KeyCode::Esc => app.submit_input(),
                                _ => {}
                            }
                        }
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
