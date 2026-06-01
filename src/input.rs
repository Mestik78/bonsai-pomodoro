use std::{io, time::Duration};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, AppMode};
use crate::models::forest::ForestLevel;

pub fn handle_event(app: &mut App, tick_rate: Duration) -> io::Result<bool> {
    if event::poll(tick_rate)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    AppMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Left => {
                                if app.current_tab == 1 && app.forest.level == ForestLevel::Bonsai {
                                    app.forest_nav_left();
                                } else {
                                    app.previous_tab();
                                }
                            },
                            KeyCode::Right => {
                                if app.current_tab == 1 && app.forest.level == ForestLevel::Bonsai {
                                    app.forest_nav_right();
                                } else {
                                    app.next_tab();
                                }
                            },
                            KeyCode::Tab => app.next_tab(),
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
                                    if app.forest.level == ForestLevel::Bonsai {
                                        app.forest_nav_up();
                                    } else {
                                        app.forest_previous();
                                    }
                                } else if app.current_tab == 2 {
                                    app.stats_previous();
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
                                    if app.forest.level == ForestLevel::Bonsai {
                                        app.forest_nav_down();
                                    } else {
                                        app.forest_next();
                                    }
                                } else if app.current_tab == 2 {
                                    app.stats_next();
                                }
                            },
                            KeyCode::Enter => {
                                if app.current_tab == 1 {
                                    app.forest_enter();
                                }
                            },
                            KeyCode::Esc => {
                                if app.current_tab == 1 {
                                    app.forest_escape();
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

    app.on_tick();

    if app.should_quit {
        app.save_state();
        return Ok(true);
    }

    Ok(false)
}
