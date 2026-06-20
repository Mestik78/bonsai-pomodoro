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

fn get_gradient_color(value: u64, max_val: u64) -> Color {
    if value == 0 {
        return Color::Rgb(22, 27, 34); // Github Dark empty
    }
    
    // Restauramos el degradado simple lineal (sin curva logarítmica)
    let raw_t = (value as f32 / max_val as f32).clamp(0.0, 1.0);
    
    // Discretize into 15 levels for values > 0 (16 colors total including empty)
    let steps = 14.0;
    let t = (raw_t * steps).round() / steps;
    
    let stops = [
        (0.00, (14, 68, 41)),      // Lowest non-zero (Github Low)
        (0.33, (0, 109, 50)),      // Medium-low
        (0.66, (38, 166, 65)),     // Medium-high
        (1.00, (57, 211, 83)),     // High (Github High)
    ];
    
    let mut c1 = stops[0];
    let mut c2 = stops[stops.len()-1];
    
    for i in 0..stops.len()-1 {
        if t >= stops[i].0 && t <= stops[i+1].0 {
            c1 = stops[i];
            c2 = stops[i+1];
            break;
        }
    }
    
    let range = c2.0 - c1.0;
    let local_t = if range > 0.0 { (t - c1.0) / range } else { 0.0 };
    
    // Función de ayuda para interpolar con corrección gamma aproximada (2.2)
    let linearize = |c: u8| -> f32 { (c as f32 / 255.0).powf(2.2) };
    let delinearize = |l: f32| -> u8 { (l.powf(1.0 / 2.2) * 255.0) as u8 };
    
    let lr1 = linearize(c1.1.0); let lg1 = linearize(c1.1.1); let lb1 = linearize(c1.1.2);
    let lr2 = linearize(c2.1.0); let lg2 = linearize(c2.1.1); let lb2 = linearize(c2.1.2);
    
    let lr = lr1 * (1.0 - local_t) + lr2 * local_t;
    let lg = lg1 * (1.0 - local_t) + lg2 * local_t;
    let lb = lb1 * (1.0 - local_t) + lb2 * local_t;
    
    let r = delinearize(lr);
    let g = delinearize(lg);
    let b = delinearize(lb);
    
    Color::Rgb(r, g, b)
}

pub fn render(frame: &mut Frame, app: &mut App, inner_area: Rect) {
    let prev_width = 12;
    let list_width = inner_area.width.saturating_sub(prev_width);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(prev_width), Constraint::Length(list_width)])
        .split(inner_area);
    
    let list_rect = chunks[0];
    if let Some((mx, my)) = app.mouse_click_pos {
        if mx >= list_rect.x && mx < list_rect.x + list_rect.width &&
           my >= list_rect.y && my < list_rect.y + list_rect.height {
            let rel_y = my as usize - list_rect.y as usize + app.stats.list_state.offset();
            if rel_y < 4 { // 4 options
                app.stats.list_state.select(Some(rel_y));
            }
        }
    }

    let mut items = Vec::new();
    let options = ["Daily", "Weekly", "Monthly", "Heatmap", "Averages"];
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
    
    let selected = app.stats.selected();
    if selected == Some(0) || selected == Some(1) || selected == Some(2) {
        let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
        
        let mut group_map: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();
        let today = chrono::Local::now().date_naive();
        let mut min_date = today;
        
        let mut daily_sums: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
        
        for t in &finished_timers {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                let dt_local = dt.with_timezone(&chrono::Local);
                let date_naive = dt_local.date_naive();
                if date_naive < min_date {
                    min_date = date_naive;
                }
                
                let duration = t.actual_runtime.unwrap_or(t.duration);
                let mins = duration / 60;
                
                if selected == Some(0) {
                    let date_str = dt_local.format("%Y-%m-%d").to_string();
                    group_map.entry(date_str).or_insert_with(Vec::new).push(mins);
                } else if selected == Some(1) {
                    let date_str = dt_local.format("%Y-%m-%d").to_string();
                    *daily_sums.entry(date_str).or_insert(0) += mins;
                } else {
                    let date_str = dt_local.format("%y-%m").to_string();
                    *daily_sums.entry(date_str).or_insert(0) += mins;
                }
            }
        }
        
        if selected == Some(1) {
            for (date_str, sum) in daily_sums {
                let dt = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
                let week_str = format!("{:02}W{:02}", dt.iso_week().year() % 100, dt.iso_week().week());
                group_map.entry(week_str).or_insert_with(Vec::new).push(sum);
            }
        } else if selected == Some(2) {
            for (date_str, sum) in daily_sums {
                group_map.entry(date_str).or_insert_with(Vec::new).push(sum);
            }
        }
        
        // Generate continuous timeline from min_date to today
        let mut group_order = Vec::new();
        let mut curr = min_date;
        while curr <= today {
            let key = if selected == Some(0) {
                curr.format("%Y-%m-%d").to_string()
            } else if selected == Some(1) {
                format!("{:02}W{:02}", curr.iso_week().year() % 100, curr.iso_week().week())
            } else {
                curr.format("%y-%m").to_string()
            };
            
            if group_order.last() != Some(&key) {
                group_order.push(key.clone());
            }
            if !group_map.contains_key(&key) {
                group_map.insert(key, Vec::new());
            }
            curr += chrono::Duration::days(1);
        }
        
        let inner_width = chunks[1].width.saturating_sub(2);
        let max_bars = if selected == Some(0) {
            let weeks = inner_width / 60;
            let rem = inner_width % 60;
            let extra = rem / 8;
            (weeks * 7 + extra) as usize
        } else {
            (inner_width / 8) as usize
        };
        
        let max_offset = group_order.len().saturating_sub(max_bars);
        if app.stats.scroll_offset > max_offset {
            app.stats.scroll_offset = max_offset;
        }
        let offset = app.stats.scroll_offset;
        
        let end_idx = group_order.len().saturating_sub(offset);
        let start_idx = end_idx.saturating_sub(max_bars);
        
        let mut chart_max = 0;
        let sigma = 4.0_f64; // Distancia de suavizado (desviación típica)
        
        for (i, key) in group_order.iter().enumerate() {
            let val = group_map.get(key).map(|v| v.iter().sum::<u64>()).unwrap_or(0);
            if val == 0 { continue; }
            
            let dist = if i < start_idx {
                start_idx - i
            } else if i >= end_idx {
                i - end_idx + 1
            } else {
                0
            };
            
            let weight = if dist == 0 {
                1.0
            } else {
                // Decaimiento Gaussiano: e^(-x^2 / 2*sigma^2)
                (-((dist as f64).powi(2)) / (2.0 * sigma.powi(2))).exp()
            };
            
            let effective_val = (val as f64 * weight) as u64;
            
            if effective_val > chart_max {
                chart_max = effective_val;
            }
        }
        let chart_max = chart_max.max(1);
        
        let visible_groups = &group_order[start_idx..end_idx];
        
        let data: Vec<(&str, u64)> = visible_groups.iter()
            .map(|key| {
                let sum = group_map.get(key).map(|v| v.iter().sum()).unwrap_or(0);
                (key.as_str(), sum)
            })
            .collect();

        let mut title_str = match selected {
            Some(0) => " Minutes per day ".to_string(),
            Some(1) => " Minutes per week ".to_string(),
            Some(2) => " Minutes per month ".to_string(),
            _ => " Minutes ".to_string(),
        };
        
        if app.stats.is_movement_mode {
            title_str = format!("{} (Scroll Mode) ", title_str.trim());
        }

        let right_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(title_str, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        
        if data.is_empty() {
            let p = Paragraph::new("No data available.")
                .alignment(Alignment::Center)
                .block(right_block);
            frame.render_widget(p, chunks[1]);
        } else {
            let mut bars: Vec<Bar> = Vec::new();
            if data.len() < max_bars {
                for _ in 0..(max_bars - data.len()) {
                    bars.push(Bar::default().label(Line::from("")).value(0).style(Style::default()));
                }
            }
            
            let mut global_max = group_map.values().map(|v| v.iter().sum::<u64>()).max().unwrap_or(0);
            if global_max < 60 {
                global_max = 60; // Mínimo absoluto para no exagerar días de 5 mins
            }

            let mut group_vecs: Vec<Vec<Bar>> = Vec::new();
            let mut current_bars: Vec<Bar> = Vec::new();
            let mut last_iso_week = None;

            for (label, value) in &data {
                let color = get_gradient_color(*value, global_max);
                
                let mut display_label = label.to_string();
                if selected == Some(0) {
                    if let Ok(dt) = chrono::NaiveDate::parse_from_str(label, "%Y-%m-%d") {
                        let iso_week = dt.iso_week().week();
                        if let Some(last_week) = last_iso_week {
                            if last_week != iso_week {
                                group_vecs.push(current_bars);
                                current_bars = Vec::new();
                            }
                        }
                        last_iso_week = Some(iso_week);
                        
                        let w_char = match dt.weekday() {
                            chrono::Weekday::Mon => "M",
                            chrono::Weekday::Tue => "T",
                            chrono::Weekday::Wed => "W",
                            chrono::Weekday::Thu => "T",
                            chrono::Weekday::Fri => "F",
                            chrono::Weekday::Sat => "S",
                            chrono::Weekday::Sun => "S",
                        };
                        
                        if dt.day() == 1 {
                            display_label = format!("{} 1/{:02}", w_char, dt.month()); // e.g. "M 1/06"
                        } else {
                            display_label = format!("{} {:02}", w_char, dt.day());
                        }
                    }
                }
                
                current_bars.push(Bar::default()
                    .label(Line::from(display_label))
                    .value(*value)
                    .style(Style::default().fg(color))
                    .value_style(Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)));
            }
            if !current_bars.is_empty() {
                group_vecs.push(current_bars);
            }
            
            let groups: Vec<BarGroup> = group_vecs.iter()
                .map(|vec| BarGroup::default().bars(vec))
                .collect();

            let mut bar_chart = BarChart::default()
                .block(right_block)
                .bar_width(6)
                .bar_gap(2)
                .max(chart_max)
                .bar_set(ratatui::symbols::bar::NINE_LEVELS);
                
            for group in groups {
                bar_chart = bar_chart.data(group);
            }
                
            if selected == Some(0) {
                bar_chart = bar_chart.group_gap(4);
            }
                
            frame.render_widget(bar_chart, chunks[1]);
            
            // Draw horizontal separators inside the bars!
            if chart_max > 0 && (selected == Some(0) || selected == Some(1)) {
                let inner_area = chunks[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
                let available_height = inner_area.height.saturating_sub(1);
                let bar_bottom_y = inner_area.bottom().saturating_sub(2);
                
                if available_height > 0 {
                    let mut bar_x = inner_area.left();
                    let mut last_iso_week = None;
                    
                    for (label, _total_val) in &data {
                        if selected == Some(0) {
                            if let Ok(dt) = chrono::NaiveDate::parse_from_str(label, "%Y-%m-%d") {
                                let iso_week = dt.iso_week().week();
                                if let Some(last_week) = last_iso_week {
                                    if last_week != iso_week {
                                        bar_x += 4; // group_gap
                                    }
                                }
                                last_iso_week = Some(iso_week);
                            }
                        }
                        
                        if let Some(parts) = group_map.get(*label) {
                            let total_val: u64 = parts.iter().sum();
                            if total_val > 0 {
                                let color = get_gradient_color(total_val, global_max);
                                let mut cumulative = 0;
                                for p in parts.iter().take(parts.len().saturating_sub(1)) {
                                    cumulative += p;
                                    let height_in_eighths = (cumulative * available_height as u64 * 8) / chart_max;
                                    let line_y = bar_bottom_y.saturating_sub((height_in_eighths / 8) as u16);
                                    
                                    if line_y >= inner_area.top() && line_y <= bar_bottom_y {
                                        for dx in 0..6 {
                                            let cx = bar_x + dx;
                                            if cx < inner_area.right() {
                                                if let Some(cell) = frame.buffer_mut().cell_mut((cx, line_y)) {
                                                    cell.set_char('─').set_bg(color).set_fg(Color::Black);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        bar_x += 8; // bar_width + bar_gap
                    }
                }
            }
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
        
        let mut global_max = *days_map.values().max().unwrap_or(&0);
        if global_max < 60 {
            global_max = 60;
        }

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
                        get_gradient_color(*mins, global_max)
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
    } else if selected == Some(4) {
        let finished_timers: Vec<&TimerSession> = app.timers.iter().filter(|t| t.state.is_none()).collect();
        
        let today = chrono::Local::now().date_naive();
        let mut min_date = today;
        for t in &finished_timers {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                let dt_local = dt.with_timezone(&chrono::Local);
                if dt_local.date_naive() < min_date {
                    min_date = dt_local.date_naive();
                }
            }
        }
        
        let mut curr = min_date;
        let mut day_counts = [0; 7];
        while curr <= today {
            let wd = curr.weekday().num_days_from_monday() as usize;
            day_counts[wd] += 1;
            curr += chrono::Duration::days(1);
        }
        
        let total_days = (today - min_date).num_days() as u64 + 1;
        
        let mut day_sums = [0; 7];
        let mut hour_sums = [0; 24];
        
        use chrono::Timelike;
        
        for t in &finished_timers {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                let mut curr_dt = dt.with_timezone(&chrono::Local);
                let duration = t.actual_runtime.unwrap_or(t.duration);
                let mut remaining_secs = duration;
                
                while remaining_secs > 0 {
                    let next_hour = (curr_dt + chrono::Duration::hours(1)).with_minute(0).unwrap().with_second(0).unwrap();
                    let secs_to_next_hour = (next_hour - curr_dt).num_seconds().max(0) as u64;
                    let secs_in_this_hour = std::cmp::min(remaining_secs, secs_to_next_hour);
                    
                    let hour = curr_dt.hour() as usize;
                    hour_sums[hour] += secs_in_this_hour;
                    
                    let wd = curr_dt.weekday().num_days_from_monday() as usize;
                    day_sums[wd] += secs_in_this_hour;
                    
                    remaining_secs -= secs_in_this_hour;
                    curr_dt = next_hour;
                }
            }
        }
        
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let mut day_bars = Vec::new();
        let mut day_max = 60;
        let mut day_vals = [0; 7];
        
        for i in 0..7 {
            let avg = if day_counts[i] > 0 { (day_sums[i] / day_counts[i]) / 60 } else { 0 };
            day_vals[i] = avg;
            if avg > day_max { day_max = avg; }
        }
        for i in 0..7 {
            let c = get_gradient_color(day_vals[i], day_max);
            day_bars.push(ratatui::widgets::Bar::default().label(Line::from(days[i])).value(day_vals[i])
                .style(Style::default().fg(c)).value_style(Style::default().fg(Color::Black).bg(c).add_modifier(Modifier::BOLD)));
        }
        
        let mut hour_bars = Vec::new();
        let mut hour_max = 10;
        let mut hour_vals = [0; 24];
        
        for i in 0..24 {
            let avg = if total_days > 0 { (hour_sums[i] / total_days) / 60 } else { 0 };
            hour_vals[i] = avg;
            if avg > hour_max { hour_max = avg; }
        }
        for i in 0..24 {
            let c = get_gradient_color(hour_vals[i], hour_max);
            let label = if i % 2 == 0 { format!("{:02}", i) } else { "".to_string() };
            hour_bars.push(ratatui::widgets::Bar::default().label(Line::from(label)).value(hour_vals[i]).text_value("".to_string())
                .style(Style::default().fg(c)).value_style(Style::default().fg(c).bg(c)));
        }
        
        let mut daily_sessions: std::collections::HashMap<String, Vec<&TimerSession>> = std::collections::HashMap::new();
        for t in &finished_timers {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&t.start_time) {
                let date_str = dt.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
                daily_sessions.entry(date_str).or_insert_with(Vec::new).push(t);
            }
        }
        
        let mut rest_sums_secs = [0; 10];
        let mut rest_counts = [0; 10];
        
        let mut weekly_breaks: std::collections::BTreeMap<String, ([u64; 10], [u64; 10])> = std::collections::BTreeMap::new();
        
        for (date_str, mut sessions) in daily_sessions {
            let dt = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
            let week_key = format!("{:02}W{:02}", dt.iso_week().year() % 100, dt.iso_week().week());
            let week_entry = weekly_breaks.entry(week_key).or_insert(([0; 10], [0; 10]));
            
            sessions.sort_by_key(|t| t.start_time.clone());
            
            let mut valid_sessions = Vec::new();
            for i in 0..sessions.len() {
                let curr_t = sessions[i];
                if let Ok(curr_dt) = chrono::DateTime::parse_from_rfc3339(&curr_t.start_time) {
                    let mut actual = curr_t.actual_runtime.unwrap_or(curr_t.duration) as i64;
                    if i + 1 < sessions.len() {
                        if let Ok(next_dt) = chrono::DateTime::parse_from_rfc3339(&sessions[i+1].start_time) {
                            let max_actual = (next_dt - curr_dt).num_seconds().max(0);
                            actual = actual.min(max_actual);
                        }
                    }
                    
                    // Filtrar pomodoros abortados: debe durar al menos el 50% de su objetivo o 1500s (25m)
                    let min_valid = (curr_t.duration / 2).min(1500) as i64;
                    if actual >= min_valid { 
                        valid_sessions.push((curr_dt, actual));
                    }
                }
            }
            
            let mut break_index = 0;
            for i in 0..valid_sessions.len() {
                if i + 1 < valid_sessions.len() {
                    let (curr_dt, actual_run) = valid_sessions[i];
                    let (next_dt, _) = valid_sessions[i+1];
                    
                    let end_of_curr = curr_dt + chrono::Duration::seconds(actual_run);
                    let rest_secs = (next_dt - end_of_curr).num_seconds();
                    
                    // Si el descanso es mayor a 60 mins (3600s), asumimos nueva sesión de mañana/tarde
                    if rest_secs > 3600 {
                        break_index = 0;
                    } else if rest_secs >= 0 {
                        if break_index < 10 {
                            rest_sums_secs[break_index] += rest_secs as u64;
                            rest_counts[break_index] += 1;
                            
                            week_entry.0[break_index] += rest_secs as u64;
                            week_entry.1[break_index] += 1;
                        }
                        break_index += 1;
                    }
                }
            }
        }
        
        let mut fatigue_bars = Vec::new();
        let mut fatigue_max = 30; // max around 30 mins for breaks
        let mut fatigue_vals = [0; 10];
        
        for i in 0..10 {
            let avg_secs = if rest_counts[i] > 0 { rest_sums_secs[i] / rest_counts[i] } else { 0 };
            fatigue_vals[i] = avg_secs / 60;
            if fatigue_vals[i] > fatigue_max { fatigue_max = fatigue_vals[i]; }
        }
        
        // Find the last index that has a > 0 value, or at least count > 0 if we want to show 0m breaks
        let last_valid = (0..10).rev().find(|&i| rest_counts[i] > 0).unwrap_or(0);
        let display_count = (last_valid + 1).max(1); // Mínimo 1 barra
        
        for i in 0..display_count {
            let c = get_gradient_color(fatigue_vals[i], fatigue_max);
            let suffix = match i { 0 => "st", 1 => "nd", 2 => "rd", _ => "th" };
            let label = format!("{}{}", i + 1, suffix);
            
            // To prevent the bar from looking empty (bug-like) if the break was < 1 minute
            let display_val = if rest_counts[i] > 0 { fatigue_vals[i].max(1) } else { 0 };
            
            fatigue_bars.push(ratatui::widgets::Bar::default().label(Line::from(label))
                .value(display_val)
                .text_value(format!("{}m", fatigue_vals[i]))
                .style(Style::default().fg(c))
                .value_style(Style::default().fg(Color::Black).bg(c).add_modifier(Modifier::BOLD)));
        }
        
        let sub_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([ratatui::layout::Constraint::Percentage(50), ratatui::layout::Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);
        let top_row = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(50), ratatui::layout::Constraint::Percentage(50)].as_ref())
            .split(sub_chunks[0]);
            
        let bottom_row = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(50), ratatui::layout::Constraint::Percentage(50)].as_ref())
            .split(sub_chunks[1]);
            
        let day_chart = ratatui::widgets::BarChart::default()
            .block(Block::default().title(" Average by Day (mins) ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .bar_width(3)
            .bar_gap(1)
            .max(day_max)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS)
            .data(ratatui::widgets::BarGroup::default().bars(&day_bars));
        frame.render_widget(day_chart, top_row[0]);
        
        let fatigue_chart = ratatui::widgets::BarChart::default()
            .block(Block::default().title(" Global Fatigue Curve (mins) ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .bar_width(7)
            .bar_gap(1)
            .max(fatigue_max)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS)
            .data(ratatui::widgets::BarGroup::default().bars(&fatigue_bars));
        frame.render_widget(fatigue_chart, top_row[1]);
        
        let hour_chart = ratatui::widgets::BarChart::default()
            .block(Block::default().title(" Average by Hour (mins) ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .bar_width(2)
            .bar_gap(0)
            .max(hour_max)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS)
            .data(ratatui::widgets::BarGroup::default().bars(&hour_bars));
        frame.render_widget(hour_chart, bottom_row[0]);
        
        // --- 4th Chart: Weekly Fatigue ---
        let mut weekly_data: Vec<(String, Vec<u64>)> = Vec::new();
        for (week_key, (sums, counts)) in &weekly_breaks {
            let mut parts = Vec::new();
            for i in 0..10 {
                if counts[i] > 0 {
                    let avg = sums[i] / counts[i] / 60;
                    parts.push(avg.max(1));
                } else {
                    break;
                }
            }
            if !parts.is_empty() {
                weekly_data.push((week_key.clone(), parts));
            }
        }
        
        let available_bars = (bottom_row[1].width.saturating_sub(2) / 6) as usize;
        let start_idx = weekly_data.len().saturating_sub(available_bars);
        let visible_weekly = if weekly_data.is_empty() { &[] } else { &weekly_data[start_idx..] };
        
        let mut weekly_chart_max = 1;
        for (_, parts) in visible_weekly {
            let total: u64 = parts.iter().sum();
            if total > weekly_chart_max {
                weekly_chart_max = total;
            }
        }
        
        let mut weekly_bars = Vec::new();
        for (label, parts) in visible_weekly {
            let total_val: u64 = parts.iter().sum();
            let c = get_gradient_color(total_val, weekly_chart_max);
            
            let display_label = if label.len() == 5 { &label[2..] } else { label };
            
            weekly_bars.push(ratatui::widgets::Bar::default()
                .label(Line::from(display_label.to_string()))
                .value(total_val)
                .text_value("") // Hide total average from the top
                .style(Style::default().fg(c))
                .value_style(Style::default().fg(Color::Black).bg(c).add_modifier(Modifier::BOLD)));
        }
        
        let weekly_chart = ratatui::widgets::BarChart::default()
            .block(Block::default().title(" Weekly Fatigue (mins) ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .bar_width(4)
            .bar_gap(2)
            .max(weekly_chart_max)
            .bar_set(ratatui::symbols::bar::NINE_LEVELS)
            .data(ratatui::widgets::BarGroup::default().bars(&weekly_bars));
            
        frame.render_widget(weekly_chart, bottom_row[1]);
        
        // Draw horizontal separators and segment values for Weekly Fatigue
        if weekly_chart_max > 0 && !visible_weekly.is_empty() {
            let inner_area = bottom_row[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
            let available_height = inner_area.height.saturating_sub(1);
            let bar_bottom_y = inner_area.bottom().saturating_sub(2);
            
            if available_height > 0 {
                let mut bar_x = inner_area.left();
                
                for (_label, parts) in visible_weekly {
                    let total_val: u64 = parts.iter().sum();
                    if total_val > 0 {
                        let color = get_gradient_color(total_val, weekly_chart_max);
                        let mut cumulative = 0;
                        
                        for (idx, &p) in parts.iter().enumerate() {
                            let start_eighths = (cumulative * available_height as u64 * 8) / weekly_chart_max;
                            let start_y = bar_bottom_y.saturating_sub((start_eighths / 8) as u16);
                            
                            cumulative += p;
                            
                            let end_eighths = (cumulative * available_height as u64 * 8) / weekly_chart_max;
                            let end_y = bar_bottom_y.saturating_sub((end_eighths / 8) as u16);
                            
                            let text_y = if idx == 0 { start_y } else { start_y.saturating_sub(1) };
                            let has_top_sep = idx < parts.len() - 1;
                            let min_y = if has_top_sep { end_y + 1 } else { end_y };
                            
                            if text_y >= min_y && text_y >= inner_area.top() && text_y <= bar_bottom_y {
                                let text = format!("{:^4}", p);
                                frame.buffer_mut().set_string(bar_x, text_y, text, Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD));
                            }
                            
                            if has_top_sep {
                                let line_y = end_y;
                                if line_y >= inner_area.top() && line_y <= bar_bottom_y {
                                    for dx in 0..4 {
                                        let cx = bar_x + dx;
                                        if cx < inner_area.right() {
                                            if let Some(cell) = frame.buffer_mut().cell_mut((cx, line_y)) {
                                                cell.set_char('─').set_bg(color).set_fg(Color::Black);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    bar_x += 6;
                }
            }
        }
    } else {
        let p = Paragraph::new("\n\nWork in progress...\nThis view will be implemented soon.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(p, chunks[1]);
    }
}
