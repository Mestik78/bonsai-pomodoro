mod app;
mod ui;
mod models;
mod bonsai;
mod input;

use std::{io, time::Duration};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let is_production = !args.contains(&"--dev".to_string());
    
    if args.contains(&"--bonsai".to_string()) {
        let seed = args.iter().position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| rand::random());
            
        let zoom = args.iter().position(|a| a == "--zoom")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
            
        let canvas = bonsai::generate_bonsai(seed, 1.0);
        let lines = canvas.render(zoom, None, None);
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
    let tick_rate = Duration::from_millis(33);

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if input::handle_event(app, tick_rate)? {
            break;
        }
    }
    
    Ok(())
}
