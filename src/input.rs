use std::{io, time::Duration};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, AppMode};

pub fn handle_event(app: &mut App, tick_rate: Duration) -> io::Result<bool> {
    if event::poll(tick_rate)? {
        let ev = event::read()?;
        if let Event::Key(key) = ev {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    AppMode::Normal => {
                        match key.code {
                            KeyCode::Left => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.previous_tab();
                                } else {
                                    app.dispatch_event(crate::models::tabs::TabEvent::Left);
                                }
                            },
                            KeyCode::Right => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.next_tab();
                                } else {
                                    app.dispatch_event(crate::models::tabs::TabEvent::Right);
                                }
                            },
                            KeyCode::Up => app.dispatch_event(crate::models::tabs::TabEvent::Up { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) }),
                            KeyCode::Down => app.dispatch_event(crate::models::tabs::TabEvent::Down { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) }),
                            KeyCode::Enter => app.dispatch_event(crate::models::tabs::TabEvent::Enter),
                            KeyCode::Esc => app.dispatch_event(crate::models::tabs::TabEvent::Esc),
                            KeyCode::Backspace => {
                                if app.current_tab == crate::models::tabs::Tab::Forest && app.forest.is_searching {
                                    app.dispatch_event(crate::models::tabs::TabEvent::Backspace);
                                }
                            },
                            KeyCode::Char(c) => {
                                if app.current_tab == crate::models::tabs::Tab::Forest && app.forest.is_searching {
                                    app.dispatch_event(crate::models::tabs::TabEvent::Char(c));
                                } else {
                                    match c {
                                        '[' => app.previous_tab(),
                                        ']' => app.next_tab(),
                                        '1'..='5' => {
                                            let idx = c.to_digit(10).unwrap() as usize - 1;
                                            app.set_tab(idx);
                                        },
                                        'q' => app.quit(),
                                        's' => {
                                            if app.current_tab == crate::models::tabs::Tab::Timer {
                                                if app.timers[0].state == Some(crate::models::timer::TimerState::New) {
                                                    app.is_selecting_plant = true;
                                                    app.timers[0].seed = rand::random();
                                                }
                                            }
                                        },
                                        ' ' => {
                                            if app.current_tab == crate::models::tabs::Tab::Plants {
                                                app.plants.toggle_animation();
                                            } else if app.current_tab != crate::models::tabs::Tab::Timer || !app.is_selecting_plant {
                                                app.toggle_timer();
                                            }
                                        },
                                        'r' => {
                                            if app.current_tab == crate::models::tabs::Tab::Timer {
                                                if app.is_selecting_plant {
                                                    app.timers[0].seed = rand::random();
                                                } else {
                                                    app.reset_timer();
                                                }
                                            } else if app.current_tab == crate::models::tabs::Tab::Plants {
                                                app.plants.seed = rand::random();
                                            }
                                        },
                                        'f' => {
                                            if app.current_tab == crate::models::tabs::Tab::Timer && !app.is_selecting_plant {
                                                app.finish_early();
                                            }
                                        },
                                        '+' => app.dispatch_event(crate::models::tabs::TabEvent::ZoomIn),
                                        '-' => app.dispatch_event(crate::models::tabs::TabEvent::ZoomOut),
                                        '/' => {
                                            if app.current_tab == crate::models::tabs::Tab::Forest {
                                                app.dispatch_event(crate::models::tabs::TabEvent::SearchStart);
                                            }
                                        },
                                        'n' => {
                                            if app.current_tab == crate::models::tabs::Tab::Forest {
                                                app.dispatch_event(crate::models::tabs::TabEvent::SearchNext);
                                            }
                                        },
                                        _ => {}
                                    }
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
        } else if let Event::Mouse(mouse_event) = ev {
            if mouse_event.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                if mouse_event.row == 0 {
                    let mut current_x = 0;
                    for (i, title) in app.tab_titles.iter().enumerate() {
                        let width = title.chars().count() as u16 + 2;
                        if mouse_event.column >= current_x && mouse_event.column < current_x + width {
                            app.set_tab(i);
                            break;
                        }
                        current_x += width + 1;
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
