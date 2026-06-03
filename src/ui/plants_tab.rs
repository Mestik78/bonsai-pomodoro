use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame,
};

use crate::app::App;
use crate::models::plant::{Plant, generate_plant};
use crate::bonsai::PlantFrame;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = area;
    let state = &mut app.plants;

    // --- Pre-calculate bottom panel height ---
    let mut bottom_height = 0;
    let mut rendered_sizes = Vec::new();
    let mut total_sizes_width = 0;
    let mut max_plant_height = 0;
    
    if let Some((_, p_type)) = state.plant_types.get(state.selected_index) {
        let plant = Plant::new(state.seed, p_type.clone(), state.animation_progress);
        let canvas = generate_plant(&plant);

        let scales_count = crate::bonsai::scale::SCALES.len();
        for scale_idx in 0..scales_count {
            let plant_frame = PlantFrame::new(Some(scale_idx), Some(scale_idx));
            let lines = plant_frame.render(&canvas, 100, 100, None, None);
            
            let w = lines.iter().map(|l| l.spans.iter().map(|s| s.content.chars().count()).sum::<usize>()).max().unwrap_or(0) as u16;
            let h = lines.len() as u16;
            
            total_sizes_width += w;
            if h > max_plant_height {
                max_plant_height = h;
            }
            
            rendered_sizes.push((lines, w, h));
        }
        
        // 1 line for the title "Selected: ..."
        bottom_height = max_plant_height + 1;
    }

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(bottom_height)])
        .split(inner_area);

    let top_area = v_chunks[0];
    let bottom_area = v_chunks[1];

    // --- TOP PANEL: Split horizontally (Left New Panel, Right Grid) ---
    let top_h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(top_area);
        
    let top_left_area = top_h_chunks[0];
    let top_right_area = top_h_chunks[1];

    // Left Panel
    let left_block = Block::default()
        .borders(Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let seed_text = vec![
        Line::from(""),
        Line::from(Span::styled("Current Seed:", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(format!("{}", state.seed), Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled("Press 'r' to reroll", Style::default().fg(Color::DarkGray))),
    ];
    let left_inner = left_block.inner(top_left_area);
    frame.render_widget(left_block, top_left_area);
    frame.render_widget(Paragraph::new(seed_text).alignment(Alignment::Center), left_inner);

    // Right Panel (Grid)
    let help_text = if state.is_selecting {
        " [Esc] to exit selection | [Arrows] to navigate "
    } else {
        " [Enter] to enter selection mode "
    };

    let right_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(help_text)
        .title_alignment(Alignment::Center);

    frame.render_widget(right_block.clone(), top_right_area);
    let top_right_inner = right_block.inner(top_right_area);

    // --- Grid Rendering ---
    let available_width = top_right_inner.width as usize;
    let block_width = if available_width < 35 { 15_usize } else { 25_usize };
    let target_tree_height = if available_width < 35 { 7_usize } else { 13_usize };
    
    let cols = (available_width / block_width).max(1);
    state.cols = cols;

    let mut plant_blocks: Vec<Vec<Line>> = Vec::new();

    for (i, (_, p_type)) in state.plant_types.iter().enumerate() {
        let is_selected = i == state.selected_index;
        let pot_color = if is_selected && state.is_selecting {
            Some(Color::Yellow)
        } else {
            None
        };

        let plant = Plant::new(state.seed, p_type.clone(), state.animation_progress);
        let canvas = generate_plant(&plant);
        let plant_frame = PlantFrame::new(Some(1), None);
        let mini_lines = plant_frame.render(&canvas, top_right_inner.width, target_tree_height as u16, None, pot_color);

        let mut block_lines = Vec::new();
        let pad_count = target_tree_height.saturating_sub(mini_lines.len());
        for _ in 0..pad_count {
            block_lines.push(Line::from(""));
        }
        for line in mini_lines {
            block_lines.push(line);
        }
        plant_blocks.push(block_lines);
    }

    let mut list_items = Vec::new();
    let max_grid_height = plant_blocks.iter().map(|b| b.len()).max().unwrap_or(0);

    for row_chunk in plant_blocks.chunks(cols) {
        for i in 0..max_grid_height {
            let mut combined_spans = Vec::new();
            for block in row_chunk.iter() {
                if i < block.len() {
                    let line_len: usize = block[i].spans.iter().map(|s| s.content.chars().count()).sum();
                    for span in block[i].spans.clone() {
                        combined_spans.push(span);
                    }
                    let padding = block_width.saturating_sub(line_len);
                    combined_spans.push(Span::raw(" ".repeat(padding)));
                } else {
                    combined_spans.push(Span::raw(" ".repeat(block_width)));
                }
            }
            list_items.push(ListItem::new(Line::from(combined_spans)));
        }
        list_items.push(ListItem::new(Line::from(""))); // Spacer between rows
    }

    let list = List::new(list_items)
        .style(Style::default().fg(Color::White));
    frame.render_widget(list, top_right_inner);


    // --- BOTTOM PANEL: Detail of selected plant ---
    if let Some((name, _)) = state.plant_types.get(state.selected_index) {
        let bottom_block = Block::default()
            .borders(Borders::NONE)
            .title(format!(" Selected: {} ", name))
            .title_alignment(Alignment::Center);
        
        frame.render_widget(bottom_block.clone(), bottom_area);
        let bottom_inner = bottom_block.inner(bottom_area);

        // Margin between sizes
        let margin = 4;
        let num_sizes = rendered_sizes.len() as u16;
        total_sizes_width += margin * (num_sizes.saturating_sub(1)); // margin between items

        // Calculate center layout
        let padding_left = bottom_inner.width.saturating_sub(total_sizes_width) / 2;
        let padding_right = bottom_inner.width.saturating_sub(padding_left + total_sizes_width);

        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(padding_left),
                Constraint::Length(total_sizes_width),
                Constraint::Length(padding_right),
            ])
            .split(bottom_inner);

        // Vertical split to align to bottom
        let v_align_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(max_plant_height)])
            .split(h_chunks[1]); // use the center chunk

        let sizes_area = v_align_chunks[1];

        // Split the sizes area into the items with margins
        let mut constraints = Vec::new();
        for (i, &(_, w, _)) in rendered_sizes.iter().enumerate() {
            constraints.push(Constraint::Length(w));
            if i < rendered_sizes.len() - 1 {
                constraints.push(Constraint::Length(margin));
            }
        }
        
        let items_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(sizes_area);

        for (idx, (lines, _w, h)) in rendered_sizes.into_iter().enumerate() {
            let target_chunk = items_chunks[idx * 2]; // 0, 2, 4
            let pad_count = max_plant_height.saturating_sub(h);
            let mut padded_lines = Vec::new();
            for _ in 0..pad_count {
                padded_lines.push(Line::from(""));
            }
            for line in lines {
                padded_lines.push(line);
            }
            frame.render_widget(Paragraph::new(padded_lines).alignment(Alignment::Center), target_chunk);
        }
    }
}
