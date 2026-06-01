use std::{io, time::Duration};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, AppMode};

pub fn handle_event(app: &mut App, tick_rate: Duration) -> io::Result<bool> {
    if event::poll(tick_rate)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    AppMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Left => app.dispatch_event(crate::models::tabs::TabEvent::Left),
                            KeyCode::Right => app.dispatch_event(crate::models::tabs::TabEvent::Right),
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::Char(' ') => app.toggle_timer(),
                            KeyCode::Char('r') => {
                                if app.current_tab == crate::models::tabs::Tab::Timer {
                                    app.reset_timer();
                                }
                            },
                            KeyCode::Char('f') => {
                                if app.current_tab == crate::models::tabs::Tab::Timer {
                                    app.finish_early();
                                }
                            },
                            KeyCode::Up => app.dispatch_event(crate::models::tabs::TabEvent::Up { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) }),
                            KeyCode::Down => app.dispatch_event(crate::models::tabs::TabEvent::Down { is_ctrl: key.modifiers.contains(KeyModifiers::CONTROL) }),
                            KeyCode::Enter => app.dispatch_event(crate::models::tabs::TabEvent::Enter),
                            KeyCode::Esc => app.dispatch_event(crate::models::tabs::TabEvent::Esc),
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
