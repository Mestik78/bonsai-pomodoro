mod app;
mod ui;
mod models;
mod bonsai;
mod input;

use std::{io, time::Duration};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableMouseCapture, DisableMouseCapture},
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
        println!("  --all-plants     Draw all available plants in the terminal and exit");
        println!("  --plant <name>   Draw a specific plant in the terminal and exit (e.g. 'Oak Tree')");
        println!("  --seed <number>  Seed for generating the bonsai (used with --bonsai/--all-plants)");
        println!("  --zoom <number>  Zoom level for the bonsai (used with --bonsai, default 1.0)");
        println!("  --animated       Animate the growth of the plants");
        return Ok(());
    }

    let is_production = !args.contains(&"--dev".to_string());
    
        let is_bonsai_mode = args.contains(&"--bonsai".to_string()) 
            || args.contains(&"--all-plants".to_string()) 
            || args.iter().any(|a| a == "--plant");

    if is_bonsai_mode {
        let seed = args.iter().position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| rand::random());
            
        let zoom_arg = args.iter().position(|a| a == "--zoom")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<f32>().ok());

        let animated = args.contains(&"--animated".to_string());

        let selected_scales: Vec<&crate::bonsai::Scale> = if let Some(z) = zoom_arg {
            if z >= 1.0 { vec![&crate::bonsai::SCALES[0]] }
            else if z >= 0.5 { vec![&crate::bonsai::SCALES[1]] }
            else { vec![&crate::bonsai::SCALES[2]] }
        } else {
            crate::bonsai::SCALES.iter().collect()
        };
            
        let mut plant_types = Vec::new();
        if args.contains(&"--all-plants".to_string()) {
            plant_types = crate::models::plant::PlantType::all();
        } else if let Some(idx) = args.iter().position(|a| a == "--plant") {
            if let Some(name) = args.get(idx + 1) {
                let name_lower = name.to_lowercase();
                plant_types = crate::models::plant::PlantType::all()
                    .into_iter()
                    .filter(|p| p.to_string().to_lowercase() == name_lower)
                    .collect();
                if plant_types.is_empty() {
                    println!("Plant type '{}' not found. Available plants:", name);
                    for p in crate::models::plant::PlantType::all() {
                        println!("  - {}", p.to_string());
                    }
                    return Ok(());
                }
            }
        } else {
            plant_types = vec![
                crate::models::plant::PlantType::Bonsai,
                crate::models::plant::PlantType::Cactus,
                crate::models::plant::PlantType::LemonTree,
            ];
        }

        // Precalcular altura máxima para anclar la maceta
        let mut global_max_height = 0;
        for p_type in &plant_types {
            let plant = crate::models::plant::Plant::new(seed, p_type.clone(), 1.0);
            let canvas = crate::models::plant::generate_plant(&plant);
            let h = canvas.render_full(None, None).len();
            if h > global_max_height {
                global_max_height = h;
            }
        }

        let start_time = std::time::Instant::now();
        let animation_duration = 25.0; // 25 mins a x60 = 25s

        loop {
            let elapsed = start_time.elapsed().as_secs_f32();
            let progress = if animated {
                (elapsed / animation_duration).clamp(0.0, 1.0)
            } else {
                1.0
            };

            // Smoothstep
            let x = progress as f64;
            let visual_progress = (x * x * (3.0 - 2.0 * x)) as f32;

            if animated {
                print!("\x1B[2J\x1B[1;1H");
            }

            for p_type in &plant_types {
                let plant = crate::models::plant::Plant::new(seed, p_type.clone(), visual_progress);
                let canvas = crate::models::plant::generate_plant(&plant);
                let mut all_renders = Vec::new();
                for scale in &selected_scales {
                    all_renders.push((scale.render_fn)(&canvas, None, None));
                }

                // Usamos la altura máxima global precalculada para que la maceta no se mueva
                let max_height = if animated { global_max_height } else { all_renders.iter().map(|r| r.len()).max().unwrap_or(0) };

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
                                            ratatui::style::Color::Black => print!("\x1b[30m"),
                                            ratatui::style::Color::Rgb(r, g, b) => print!("\x1b[38;2;{};{};{}m", r, g, b),
                                            ratatui::style::Color::Red | ratatui::style::Color::LightRed | ratatui::style::Color::Magenta => print!("\x1b[31m"),
                                            ratatui::style::Color::Yellow => print!("\x1b[33m"),
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
                println!("\n");
            }
            
            if progress >= 1.0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    let _ = std::fs::remove_file("/tmp/bonsai_status");

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
