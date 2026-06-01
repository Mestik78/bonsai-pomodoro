use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::models::timer::TimerSession;
use chrono::{Datelike, Duration, Local};

pub fn render(frame: &mut Frame, app: &mut App, inner_area: Rect) {
    let prev_width = 20.max((inner_area.width as f32 * 0.45) as u16);
    let list_width = inner_area.width.saturating_sub(prev_width);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(prev_width), Constraint::Length(list_width)])
        .split(inner_area);
    
    let mut items = Vec::new();
    let options = ["Daily", "Weekly", "Monthly", "Heatmap"];
    for (i, opt) in options.iter().enumerate() {
        let is_selected = app.stats.selected() == Some(i);
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
        };
        items.push(ListItem::new(Line::from(Span::styled(format!(" {} ", opt), style))));
    }
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)))
        .style(Style::default().fg(Color::White));
        
    frame.render_stateful_widget(list, chunks[0], &mut app.stats.list_state);
    
    if app.stats.selected() == Some(0) {
        let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
        
        let mut days_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut days_order = Vec::new();
        
        for t in finished_timers {
            let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.format("%m-%d").to_string(), // Short date for bar chart
                Err(_) => t.start_time.chars().take(5).collect(),
            };
            
            if !days_map.contains_key(&date_str) {
                days_order.push(date_str.clone());
                days_map.insert(date_str.clone(), 0);
            }
            
            let duration = t.actual_runtime.unwrap_or(t.duration);
            let mins = duration / 60;
            *days_map.get_mut(&date_str).unwrap() += mins;
        }
        
        days_order.reverse();
        
        let data: Vec<(&str, u64)> = days_order.iter()
            .map(|day| {
                (day.as_str(), *days_map.get(day).unwrap_or(&0))
            })
            .collect();

        let right_block = Block::default()
            .borders(Borders::NONE)
            .title(Span::styled(" Minutes per day ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        
        if data.is_empty() {
            let p = Paragraph::new("No data available.")
                .alignment(Alignment::Center)
                .block(right_block);
            frame.render_widget(p, chunks[1]);
        } else {
            let bars: Vec<Bar> = data.iter().map(|(label, value)| {
                let color = if *value >= 180 {
                    Color::Red
                } else if *value >= 120 {
                    Color::Yellow
                } else if *value >= 60 {
                    Color::LightGreen
                } else if *value >= 30 {
                    Color::Green
                } else {
                    Color::DarkGray
                };
                
                Bar::default()
                    .label(Line::from(label.to_string()))
                    .value(*value)
                    .style(Style::default().fg(color))
                    .value_style(Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD))
            }).collect();

            let bar_chart = BarChart::default()
                .block(right_block)
                .data(BarGroup::default().bars(&bars))
                .bar_width(6)
                .bar_gap(2)
                .bar_set(ratatui::symbols::bar::NINE_LEVELS);
                
            frame.render_widget(bar_chart, chunks[1]);
        }
    } else if app.stats.selected() == Some(3) {
        let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
        
        let mut days_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for t in finished_timers {
            let date_str = match chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                Ok(dt) => dt.with_timezone(&Local).format("%Y-%m-%d").to_string(),
                Err(_) => "".to_string(),
            };
            if !date_str.is_empty() {
                let duration = t.actual_runtime.unwrap_or(t.duration);
                let mins = duration / 60;
                *days_map.entry(date_str).or_insert(0) += mins;
            }
        }
        
        let today = Local::now().date_naive();
        let year = today.year();
        let jan1 = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let dec31 = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
        
        let start_date = jan1 - Duration::days(jan1.weekday().num_days_from_monday() as i64);
        let end_date = dec31 + Duration::days(6 - dec31.weekday().num_days_from_monday() as i64);
        let total_weeks = ((end_date - start_date).num_days() / 7) as usize + 1;
        
        let max_weeks_per_chunk = (chunks[1].width.saturating_sub(4) as usize).max(1);
        let num_chunks = (total_weeks + max_weeks_per_chunk - 1) / max_weeks_per_chunk;
        
        let mut lines = Vec::new();
        lines.push(Line::from("")); // Top padding
        let day_labels = ["M", "T", "W", "T", "F", "S", "S"];
        
        for chunk_idx in 0..num_chunks {
            if chunk_idx > 0 {
                lines.push(Line::from("")); // Gap between chunks
            }
            
            let chunk_start_week = chunk_idx * max_weeks_per_chunk;
            let chunk_end_week = (chunk_start_week + max_weeks_per_chunk).min(total_weeks);
            let chunk_weeks = chunk_end_week - chunk_start_week;
            
            for row in 0..7 {
                let mut spans = Vec::new();
                spans.push(Span::styled(format!("{} ", day_labels[row]), Style::default().fg(Color::DarkGray)));
                
                for col in 0..chunk_weeks {
                    let week_idx = chunk_start_week + col;
                    let cell_date = start_date + Duration::weeks(week_idx as i64) + Duration::days(row as i64);
                    
                    let _is_future = cell_date > today;
                    let is_outside_year = cell_date.year() != year;
                    let is_today = cell_date == today;
                    
                    let color = if is_outside_year {
                        Color::Reset
                    } else {
                        let date_str = cell_date.format("%Y-%m-%d").to_string();
                        let mins = days_map.get(&date_str).unwrap_or(&0);
                        if *mins >= 180 {
                            Color::Red
                        } else if *mins >= 120 {
                            Color::Yellow
                        } else if *mins >= 60 {
                            Color::LightGreen
                        } else if *mins >= 30 {
                            Color::Green
                        } else if *mins > 0 {
                            Color::DarkGray
                        } else {
                            Color::Rgb(30, 30, 30) // Very dark gray for empty days
                        }
                    };
                    
                    if is_outside_year {
                        spans.push(Span::raw(" "));
                    } else {
                        let content = if is_today { "x" } else { " " };
                        spans.push(Span::styled(content, Style::default().bg(color).fg(Color::White).add_modifier(Modifier::BOLD)));
                    }
                }
                lines.push(Line::from(spans));
            }
        }
        
        let right_block = Block::default()
            .borders(Borders::NONE)
            .title(Span::styled(" Activity Heatmap ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            
        let p = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(right_block);
        frame.render_widget(p, chunks[1]);
    } else {
        let p = Paragraph::new("\n\nWork in progress...\nThis view will be implemented soon.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(p, chunks[1]);
    }
}
