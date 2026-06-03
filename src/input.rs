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
                        if app.current_tab == crate::models::tabs::Tab::Forest && app.forest.is_searching {
                            match key.code {
                                KeyCode::Char(c) => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::Char(c)); },
                                KeyCode::Backspace => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::Backspace); },
                                KeyCode::Enter => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::Enter); },
                                KeyCode::Esc => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::Esc); },
                                _ => {}
                            }
                        } else {
                            let mut is_up = false;
                            let mut is_down = false;
                            let mut is_left = false;
                            let mut is_right = false;
                            let mut is_accept = false;
                            let mut is_back = false;

                            match key.code {
                                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => is_up = true,
                                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => is_down = true,
                                KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => is_left = true,
                                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => is_right = true,
                                KeyCode::Enter | KeyCode::Char(' ') => is_accept = true,
                                KeyCode::Esc | KeyCode::Char('q') => is_back = true,
                                _ => {}
                            }

                            if is_left {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.previous_tab();
                                } else if matches!(app.dispatch_event(crate::models::tabs::TabEvent::Left), crate::models::tabs::EventResult::Ignored) {
                                    app.previous_tab();
                                }
                            } else if is_right {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.next_tab();
                                } else if matches!(app.dispatch_event(crate::models::tabs::TabEvent::Right), crate::models::tabs::EventResult::Ignored) {
                                    app.next_tab();
                                }
                            } else if is_up {
                                let _ = app.dispatch_event(crate::models::tabs::TabEvent::Up { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) });
                            } else if is_down {
                                let _ = app.dispatch_event(crate::models::tabs::TabEvent::Down { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) });
                            } else if is_accept {
                                let res = app.dispatch_event(crate::models::tabs::TabEvent::Enter);
                                match res {
                                    crate::models::tabs::EventResult::Ignored => {
                                        if app.current_tab == crate::models::tabs::Tab::Plants {
                                            app.plants.toggle_animation();
                                        } else if app.current_tab != crate::models::tabs::Tab::Timer || !app.is_selecting_plant {
                                            app.toggle_timer();
                                        }
                                    },
                                    crate::models::tabs::EventResult::JumpToForest(idx) => {
                                        app.history.escape();
                                        app.current_tab = crate::models::tabs::Tab::Forest;
                                        app.forest.center_on_timer(idx);
                                    },
                                    crate::models::tabs::EventResult::JumpToHistory(idx) => {
                                        app.current_tab = crate::models::tabs::Tab::History;
                                        app.history.select_timer(idx, &app.timers);
                                    },
                                    _ => {}
                                }
                            } else if is_back {
                                if matches!(app.dispatch_event(crate::models::tabs::TabEvent::Esc), crate::models::tabs::EventResult::Ignored) {
                                    app.quit();
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char('[') => app.previous_tab(),
                                    KeyCode::Char(']') => app.next_tab(),
                                    KeyCode::Char(c @ '1'..='5') => {
                                        let idx = c.to_digit(10).unwrap() as usize - 1;
                                        app.set_tab(idx);
                                    },
                                    KeyCode::Char('p') => {
                                        if app.current_tab == crate::models::tabs::Tab::Timer {
                                            if app.timers[0].state == Some(crate::models::timer::TimerState::New) {
                                                app.is_selecting_plant = true;
                                                app.timers[0].seed = rand::random();
                                            }
                                        }
                                    },
                                    KeyCode::Char('r') => {
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
                                    KeyCode::Char('f') => {
                                        if app.current_tab == crate::models::tabs::Tab::Timer && !app.is_selecting_plant {
                                            app.finish_early();
                                        }
                                    },
                                    KeyCode::Char('+') => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::ZoomIn); },
                                    KeyCode::Char('-') => { let _ = app.dispatch_event(crate::models::tabs::TabEvent::ZoomOut); },
                                    KeyCode::Char('/') => {
                                        if app.current_tab == crate::models::tabs::Tab::Forest {
                                            let _ = app.dispatch_event(crate::models::tabs::TabEvent::SearchStart);
                                        }
                                    },
                                    KeyCode::Char('n') => {
                                        if app.current_tab == crate::models::tabs::Tab::Forest {
                                            let _ = app.dispatch_event(crate::models::tabs::TabEvent::SearchNext);
                                        }
                                    },
                                    _ => {}
                                }
                            }
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
