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

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Bonsai Pomodoro - Command line options:\n");
        println!("  -h, --help       Show this help message and exit");
        println!("  --dev            Start the application in development mode");
        println!("  --bonsai         Draw a bonsai in the terminal and exit");
        println!("  --seed <number>  Seed for generating the bonsai (used with --bonsai)");
        println!("  --zoom <number>  Zoom level for the bonsai (used with --bonsai, default 1.0)");
        return Ok(());
    }

    let is_production = !args.contains(&"--dev".to_string());
    
    if args.contains(&"--bonsai".to_string()) {
        let seed = args.iter().position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| rand::random());
            
        let zoom_arg = args.iter().position(|a| a == "--zoom")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok());

        let zooms = if let Some(z) = zoom_arg {
            vec![z]
        } else {
            vec![1.0, 0.5, 0.25]
        };
            
        let canvas = bonsai::generate_bonsai(seed, 1.0);
        let mut all_renders = Vec::new();
        for &z in &zooms {
            all_renders.push(canvas.render(z, None, None));
        }

        let max_height = all_renders.iter().map(|r| r.len()).max().unwrap_or(0);

        for y in 0..max_height {
            for (i, lines) in all_renders.iter().enumerate() {
                let height = lines.len();
                let width = lines.first()
                    .map(|l| l.spans.iter().map(|s| s.content.chars().count()).sum::<usize>())
                    .unwrap_or(0);
                
                let pad_top = max_height.saturating_sub(height);
                
                if y < pad_top {
                    print!("{}", " ".repeat(width));
                } else {
                    let orig_y = y - pad_top;
                    if let Some(line) = lines.get(orig_y) {
                        for span in &line.spans {
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
                    } else {
                        print!("{}", " ".repeat(width));
                    }
                }
                
                if i < all_renders.len() - 1 {
                    print!("    "); // 4 espacios de margen entre árboles
                }
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
