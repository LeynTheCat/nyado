use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::io;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub enum PopupMode {
    Singleline,
    Multiline,
    Readonly,
}

pub enum PopupReadonlyLayout {
    SingleColumn,
    TwoColumns,
}

pub enum ProjectAction {
    Switch(String),
    Create,
    Rename(String),
    Delete(String),
    None,
}

fn visual_width(s: &str) -> usize {
    s.width()
}

fn visual_offset(s: &str, n: usize) -> usize {
    s.chars().take(n).map(|c| c.width().unwrap_or(1)).sum()
}

fn truncate_by_width(s: &str, max_width: usize) -> String {
    let mut res = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let ch_w = ch.width().unwrap_or(1);
        if w + ch_w > max_width {
            if w + 1 <= max_width {
                res.push('…');
            }
            break;
        }
        res.push(ch);
        w += ch_w;
    }
    res
}

fn split_line_by_width(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![line.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for ch in line.chars() {
        let ch_w = ch.width().unwrap_or(1);
        if current_width + ch_w > max_width && !current.is_empty() {
            result.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += ch_w;
    }
    if !current.is_empty() {
        result.push(current);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn total_char_index(lines: &[String], row: usize, col: usize) -> usize {
    let mut idx = 0;
    for (i, line) in lines.iter().enumerate() {
        if i < row {
            idx += line.chars().count();
        } else if i == row {
            idx += col.min(line.chars().count());
            break;
        }
    }
    idx
}

fn find_position_from_index(lines: &[String], mut index: usize) -> (usize, usize) {
    for (i, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        if index <= len {
            return (i, index);
        }
        index -= len;
    }
    if lines.is_empty() {
        (0, 0)
    } else {
        let last = lines.len() - 1;
        (last, lines[last].chars().count())
    }
}

fn popup_singleline(
    title: &str,
    hint: &str,
    initial: &str,
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
    max_len: usize,
) -> io::Result<Option<String>> {
    let mut chars: Vec<char> = initial.chars().collect();
    let mut cursor_pos = chars.len();
    let mut scroll_offset: usize = 0;

    loop {
        let term_size = term.size()?;
        let popup_width = std::cmp::min(74, term_size.width.saturating_sub(4));
        let popup_height = 7;
        if popup_width == 0 || term_size.height < popup_height + 2 {
            return Ok(None);
        }
        let popup_x = (term_size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (term_size.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        let input_line_width = (popup_width.saturating_sub(8)) as usize;
        let full_text: String = chars.iter().collect();
        let cursor_visual = visual_offset(&full_text, cursor_pos);
        if cursor_visual >= scroll_offset + input_line_width {
            scroll_offset = cursor_visual.saturating_sub(input_line_width.saturating_sub(1));
        }
        if cursor_visual < scroll_offset {
            scroll_offset = cursor_visual;
        }

        let visible_text = if scroll_offset == 0 {
            full_text.clone()
        } else {
            let mut acc = 0;
            let mut idx = 0;
            for ch in full_text.chars() {
                let w = ch.width().unwrap_or(1);
                if acc + w <= scroll_offset {
                    acc += w;
                    idx += 1;
                } else {
                    break;
                }
            }
            full_text.chars().skip(idx).collect::<String>()
        };
        let mut display_text = truncate_by_width(&visible_text, input_line_width);
        if visual_width(&full_text) > scroll_offset + input_line_width {
            display_text = truncate_by_width(&visible_text, input_line_width.saturating_sub(1));
            display_text.push('…');
        }

        term.draw(|f| {
            f.render_widget(Clear, popup_area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            if !hint.is_empty() && inner.height >= 1 {
                let hint_rect = Rect::new(
                    inner.left().saturating_add(2),
                    inner.top().saturating_add(1),
                    inner.width.saturating_sub(4),
                    1,
                );
                if hint_rect.width > 0 && hint_rect.height > 0 {
                    f.render_widget(Paragraph::new(hint).style(Style::default().dim()), hint_rect);
                }
            }

            let input_area = Rect::new(
                inner.left().saturating_add(2),
                inner.top().saturating_add(3),
                inner.width.saturating_sub(4),
                1,
            );
            if input_area.width > 0 && input_area.height > 0 {
                let display_span = Span::styled(format!("> {}", display_text), Style::default().fg(Color::Yellow));
                f.render_widget(Paragraph::new(display_span), input_area);
            }

            let cursor_visual_rel = cursor_visual.saturating_sub(scroll_offset);
            let cursor_x = inner.left().saturating_add(2).saturating_add(2).saturating_add(cursor_visual_rel as u16);
            let cursor_y = inner.top().saturating_add(3);
            if cursor_visual_rel <= input_line_width && cursor_x < term_size.width && cursor_y < term_size.height {
                f.set_cursor(cursor_x, cursor_y);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    if cursor_pos > 0 {
                        chars.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < chars.len() {
                        cursor_pos += 1;
                    }
                }
                KeyCode::Char(c) => {
                    if chars.len() < max_len {
                        chars.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                }
                _ => {}
            }
        }
    }
    let result: String = chars.into_iter().collect();
    Ok(Some(result))
}

fn popup_multiline_editable(
    title: &str,
    hint: &str,
    initial: &str,
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
    max_total_chars: usize,
    max_line_width: usize,
) -> io::Result<Option<String>> {
    let mut flat: Vec<char> = initial.chars().collect();
    let mut abs_pos = flat.len();
    let mut scroll_offset: usize = 0;

    loop {
        let term_size = term.size()?;
        let popup_width = std::cmp::min(74, term_size.width.saturating_sub(4));
        let popup_height = 12;
        if popup_width == 0 || term_size.height < popup_height + 2 {
            return Ok(None);
        }
        let popup_x = (term_size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (term_size.height.saturating_sub(popup_height)) / 2;
        let input_area_height = (popup_height - 7) as usize;

        let min_input_width = 20;
        let raw_input_width = (popup_width as i16 - 8).max(min_input_width);
        let visual_max_line_len = raw_input_width as usize;
        let effective_max_width = visual_max_line_len.min(max_line_width).max(1);

        let flat_str: String = flat.iter().collect();
        let lines = split_line_by_width(&flat_str, effective_max_width);
        let (cursor_y, cursor_x) = find_position_from_index(&lines, abs_pos);

        let mut scroll = scroll_offset;
        if cursor_y < scroll {
            scroll = cursor_y;
        }
        if cursor_y >= scroll + input_area_height {
            scroll = cursor_y.saturating_sub(input_area_height - 1);
        }
        scroll_offset = scroll;

        term.draw(|f| {
            let area = Rect::new(0, 0, term_size.width, term_size.height);
            f.render_widget(Clear, area);
            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            if !hint.is_empty() && inner.height >= 1 {
                f.render_widget(
                    Paragraph::new(hint).style(Style::default().dim()),
                    Rect::new(inner.left() + 2, inner.top() + 1, inner.width.saturating_sub(4), 1),
                );
            }

            let input_area = Rect::new(inner.left() + 2, inner.top() + 3, inner.width.saturating_sub(4), inner.height.saturating_sub(5));
            let display_lines: Vec<Line> = lines
                .iter()
                .skip(scroll_offset)
                .take(input_area.height as usize)
                .map(|line| {
                    let mut display = line.clone();
                    if visual_width(&display) > visual_max_line_len {
                        display = truncate_by_width(&display, visual_max_line_len - 1);
                        display.push('…');
                    }
                    Line::from(Span::styled(display, Style::default().fg(Color::Yellow)))
                })
                .collect();
            let para = Paragraph::new(display_lines);
            f.render_widget(para, input_area);

            let current_line = &lines[cursor_y];
            let cursor_visual = visual_offset(current_line, cursor_x);
            let visible_cursor_y = cursor_y.saturating_sub(scroll_offset);
            if visible_cursor_y < input_area.height as usize && cursor_y < lines.len() {
                let cursor_x_abs = inner.left() + 2 + (cursor_visual as u16).min(inner.width.saturating_sub(4));
                let cursor_y_abs = input_area.top() + visible_cursor_y as u16;
                if cursor_x_abs < term_size.width && cursor_y_abs < term_size.height {
                    f.set_cursor(cursor_x_abs, cursor_y_abs);
                } else {
                    f.set_cursor(inner.left() + 2, inner.top() + 3);
                }
            } else {
                f.set_cursor(inner.left() + 2, inner.top() + 3);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Enter {
                break;
            }
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Backspace => {
                    if abs_pos > 0 {
                        flat.remove(abs_pos - 1);
                        abs_pos -= 1;
                    }
                }
                KeyCode::Delete => {
                    if abs_pos < flat.len() {
                        flat.remove(abs_pos);
                    }
                }
                KeyCode::Char(c) => {
                    if flat.len() < max_total_chars {
                        flat.insert(abs_pos, c);
                        abs_pos += 1;
                    }
                }
                KeyCode::Left => {
                    if abs_pos > 0 {
                        abs_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if abs_pos < flat.len() {
                        abs_pos += 1;
                    }
                }
                KeyCode::Up => {
                    let flat_str: String = flat.iter().collect();
                    let lines = split_line_by_width(&flat_str, effective_max_width);
                    let (cy, cx) = find_position_from_index(&lines, abs_pos);
                    if cy > 0 {
                        let new_cy = cy - 1;
                        let new_cx = cx.min(lines[new_cy].chars().count());
                        abs_pos = total_char_index(&lines, new_cy, new_cx);
                    }
                }
                KeyCode::Down => {
                    let flat_str: String = flat.iter().collect();
                    let lines = split_line_by_width(&flat_str, effective_max_width);
                    let (cy, cx) = find_position_from_index(&lines, abs_pos);
                    if cy + 1 < lines.len() {
                        let new_cy = cy + 1;
                        let new_cx = cx.min(lines[new_cy].chars().count());
                        abs_pos = total_char_index(&lines, new_cy, new_cx);
                    }
                }
                KeyCode::PageUp => {
                    let flat_str: String = flat.iter().collect();
                    let lines = split_line_by_width(&flat_str, effective_max_width);
                    let (cy, cx) = find_position_from_index(&lines, abs_pos);
                    let step = input_area_height;
                    let new_cy = cy.saturating_sub(step);
                    let new_cx = cx.min(lines[new_cy].chars().count());
                    abs_pos = total_char_index(&lines, new_cy, new_cx);
                }
                KeyCode::PageDown => {
                    let flat_str: String = flat.iter().collect();
                    let lines = split_line_by_width(&flat_str, effective_max_width);
                    let (cy, cx) = find_position_from_index(&lines, abs_pos);
                    let step = input_area_height;
                    let new_cy = (cy + step).min(lines.len() - 1);
                    let new_cx = cx.min(lines[new_cy].chars().count());
                    abs_pos = total_char_index(&lines, new_cy, new_cx);
                }
                KeyCode::Home => abs_pos = 0,
                KeyCode::End => abs_pos = flat.len(),
                _ => {}
            }
        }
    }
    let result: String = flat.iter().collect();
    Ok(if result.is_empty() { None } else { Some(result) })
}

fn popup_readonly(
    title: &str,
    hint: &str,
    content: &str,
    layout: PopupReadonlyLayout,
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    lines.push("");
    let hint_lines: Vec<&str> = hint.lines().collect();
    let hint_height = if hint.is_empty() { 0 } else { hint_lines.len() };
    let bottom_padding = 1;

    let max_line_len = lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);
    let max_hint_len = hint_lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);

    let (needed_width, needed_height) = match layout {
        PopupReadonlyLayout::SingleColumn => {
            let width = max_line_len.max(max_hint_len) + 6;
            let height = lines.len() + 4 + hint_height + bottom_padding;
            (width, height)
        }
        PopupReadonlyLayout::TwoColumns => {
            let left_len = (lines.len() + 1) / 2;
            let right_len = lines.len() - left_len;
            let height = left_len.max(right_len) + 4 + hint_height + bottom_padding;
            let col_width = max_line_len + 2;
            let gap = 2;
            let width = 2 * col_width + gap + 4;
            (width, height)
        }
    };

    loop {
        let term_size = term.size()?;
        let popup_width = std::cmp::min(needed_width as u16, term_size.width.saturating_sub(4));
        let popup_height = std::cmp::min(needed_height as u16, term_size.height.saturating_sub(4));
        if popup_width == 0 || popup_height == 0 {
            return Ok(());
        }
        let popup_x = (term_size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (term_size.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        term.draw(|f| {
            f.render_widget(Clear, popup_area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            if !hint.is_empty() && inner.height > 0 {
                let hint_rect = Rect::new(
                    inner.left().saturating_add(2),
                    inner.top().saturating_add(1),
                    inner.width.saturating_sub(4),
                    hint_height as u16,
                );
                if hint_rect.width > 0 && hint_rect.height > 0 {
                    f.render_widget(Paragraph::new(hint).style(Style::default().dim()), hint_rect);
                }
            }

            let content_y = inner.top().saturating_add(2).saturating_add(hint_height as u16);
            let content_height = inner.height
                .saturating_sub(2)
                .saturating_sub(hint_height as u16)
                .saturating_sub(bottom_padding as u16);
            let content_area = Rect::new(
                inner.left().saturating_add(2),
                content_y,
                inner.width.saturating_sub(4),
                content_height,
            );
            if content_area.width == 0 || content_area.height == 0 {
                return;
            }

            match layout {
                PopupReadonlyLayout::SingleColumn => {
                    let display_lines: Vec<Line> = lines
                        .iter()
                        .map(|line| {
                            if visual_width(line) > content_area.width as usize {
                                let truncated = truncate_by_width(line, content_area.width as usize - 1);
                                format!("{}…", truncated)
                            } else {
                                (*line).to_string()
                            }
                        })
                        .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Yellow))))
                        .collect();
                    let para = Paragraph::new(display_lines);
                    f.render_widget(para, content_area);
                }
                PopupReadonlyLayout::TwoColumns => {
                    let gap = 2;
                    let left_len = (lines.len() + 1) / 2;
                    let (left_lines, right_lines) = lines.split_at(left_len);
                    let col_width = (content_area.width.saturating_sub(gap)) / 2;
                    if col_width == 0 {
                        return;
                    }
                    let cols = Layout::default()
                        .direction(ratatui::layout::Direction::Horizontal)
                        .constraints([
                            Constraint::Length(col_width),
                            Constraint::Length(gap),
                            Constraint::Length(col_width),
                        ])
                        .split(content_area);
                    if cols.len() < 3 {
                        return;
                    }
                    let left_para = Paragraph::new(
                        left_lines
                            .iter()
                            .map(|line| {
                                if visual_width(line) > col_width as usize {
                                    let truncated = truncate_by_width(line, col_width as usize - 1);
                                    format!("{}…", truncated)
                                } else {
                                    (*line).to_string()
                                }
                            })
                            .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Yellow))))
                            .collect::<Vec<Line>>(),
                    );
                    let right_para = Paragraph::new(
                        right_lines
                            .iter()
                            .map(|line| {
                                if visual_width(line) > col_width as usize {
                                    let truncated = truncate_by_width(line, col_width as usize - 1);
                                    format!("{}…", truncated)
                                } else {
                                    (*line).to_string()
                                }
                            })
                            .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Yellow))))
                            .collect::<Vec<Line>>(),
                    );
                    f.render_widget(left_para, cols[0]);
                    f.render_widget(right_para, cols[2]);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => break,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
pub fn popup_with_mode(
    title: &str,
    hint: &str,
    initial: &str,
    mode: PopupMode,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<Option<String>> {
    popup_with_mode_layout(title, hint, initial, mode, terminal, PopupReadonlyLayout::SingleColumn)
}

pub fn popup_with_mode_layout(
    title: &str,
    hint: &str,
    initial: &str,
    mode: PopupMode,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    readonly_layout: PopupReadonlyLayout,
) -> io::Result<Option<String>> {
    match mode {
        PopupMode::Singleline => popup_singleline(title, hint, initial, terminal, 16),
        PopupMode::Multiline => popup_multiline_editable(title, hint, initial, terminal, 256, 60),
        PopupMode::Readonly => {
            popup_readonly(title, hint, initial, readonly_layout, terminal)?;
            Ok(None)
        }
    }
}

pub fn popup_project_manager(
    title: &str,
    projects: &[String],
    current: &str,
    help_switch: &str,
    help_create: &str,
    help_rename: &str,
    help_delete: &str,
    help_title: &str,
    hint_c: &str,
    hint_r: &str,
    hint_d: &str,
    hint_enter: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<ProjectAction> {
    if projects.is_empty() {
        return Ok(ProjectAction::None);
    }

    let mut selected = 0;
    let mut scroll_offset = 0;
    let cols = 2;

    let left_width = 40;
    let right_width = 24;
    let gap = 2;
    let right_fixed_height = 8;

    loop {
        let term_size = terminal.size()?;
        if term_size.height < 4 || term_size.width < 20 {
            return Ok(ProjectAction::None);
        }

        let rows = (projects.len() + cols - 1) / cols;
        let needed_height = (rows + 4) as u16;
        let left_height = std::cmp::min(needed_height, term_size.height.saturating_sub(4));
        let total_height = std::cmp::max(left_height, right_fixed_height);
        let total_width = left_width + gap + right_width;
        let popup_width = std::cmp::min(total_width, term_size.width.saturating_sub(4));
        let popup_x = (term_size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (term_size.height.saturating_sub(total_height)) / 2;

        let left_area = Rect::new(popup_x, popup_y, left_width, left_height);
        let right_area = Rect::new(popup_x + left_width + gap, popup_y, right_width, right_fixed_height);

        let inner_width = left_width - 2;
        let col_width = (inner_width - 3) / cols as u16;
        let list_height = left_height.saturating_sub(2) as usize;
        if list_height == 0 {
            return Ok(ProjectAction::None);
        }

        let current_row = selected / cols;
        if current_row < scroll_offset {
            scroll_offset = current_row;
        }
        if current_row >= scroll_offset + list_height {
            scroll_offset = current_row.saturating_sub(list_height.saturating_sub(1));
        }

        terminal.draw(|f| {
            let left_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner_left = left_block.inner(left_area);
            f.render_widget(left_block, left_area);

            let top_padding = 1;
            let bottom_padding = 1;
            let list_start_y = inner_left.top().saturating_add(top_padding);
            let list_available_height = inner_left.height.saturating_sub(top_padding + bottom_padding);

            for row in 0..rows {
                let y_offset = row as u16;
                if y_offset >= list_available_height {
                    continue;
                }
                let y = list_start_y.saturating_add(y_offset);
                for col in 0..cols {
                    let idx = row * cols + col;
                    if idx >= projects.len() {
                        continue;
                    }
                    let item = &projects[idx];
                    let display = if visual_width(item) > col_width as usize - 2 {
                        truncate_by_width(item, col_width as usize - 3) + "…"
                    } else {
                        item.clone()
                    };
                    let prefix = if idx == selected { "▶  " } else { "  " };
                    let line = format!("{}{}", prefix, display);
                    let style = if idx == selected {
                        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
                    } else if item == current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    let span = Span::styled(line, style);
                    let x = inner_left.left().saturating_add(2).saturating_add(col as u16 * (col_width + 2));
                    let rect = Rect::new(x, y, col_width, 1);
                    if rect.x < term_size.width && rect.y < term_size.height && rect.width > 0 {
                        f.render_widget(Paragraph::new(span), rect);
                    }
                }
            }

            let right_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(help_title, Style::default().fg(Color::Cyan)));
            let inner_right = right_block.inner(right_area);
            f.render_widget(right_block, right_area);

            let help_lines = vec![
                format!("{}: {}", hint_enter, help_switch),
                format!("{}: {}", hint_c, help_create),
                format!("{}: {}", hint_r, help_rename),
                format!("{}: {}", hint_d, help_delete),
            ];
            let help_style = Style::default().fg(Color::DarkGray).dim();
            let help_top_padding = 1;
            for (idx, line) in help_lines.iter().enumerate() {
                let truncated = if visual_width(line) > inner_right.width as usize - 2 {
                    truncate_by_width(line, inner_right.width as usize - 3) + "…"
                } else {
                    line.clone()
                };
                let span = Span::styled(truncated, help_style);
                let rect = Rect::new(
                    inner_right.left().saturating_add(1),
                    inner_right.top().saturating_add(help_top_padding + idx as u16),
                    inner_right.width.saturating_sub(2),
                    1,
                );
                if rect.width > 0 && rect.height > 0 {
                    f.render_widget(Paragraph::new(span), rect);
                }
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => return Ok(ProjectAction::None),
                KeyCode::Enter => {
                    let name = projects[selected].clone();
                    if name == current {
                        return Ok(ProjectAction::None);
                    }
                    return Ok(ProjectAction::Switch(name));
                }
                KeyCode::Char('c') | KeyCode::Char('с') => return Ok(ProjectAction::Create),
                KeyCode::Char('r') | KeyCode::Char('к') => {
                    let name = projects[selected].clone();
                    if name == "default" {
                        continue;
                    }
                    return Ok(ProjectAction::Rename(name));
                }
                KeyCode::Char('d') | KeyCode::Char('в') => {
                    let name = projects[selected].clone();
                    if name == "default" {
                        continue;
                    }
                    return Ok(ProjectAction::Delete(name));
                }
                KeyCode::Up => {
                    let new_row = selected / cols;
                    if new_row > 0 {
                        selected = selected.saturating_sub(cols);
                    }
                }
                KeyCode::Down => {
                    if selected + cols < projects.len() {
                        selected += cols;
                    }
                }
                KeyCode::Left => {
                    let col = selected % cols;
                    if col > 0 {
                        selected = selected.saturating_sub(1);
                    }
                }
                KeyCode::Right => {
                    let col = selected % cols;
                    if col + 1 < cols && selected + 1 < projects.len() {
                        selected += 1;
                    }
                }
                KeyCode::PageUp => {
                    let step = list_height;
                    let new_row = (selected / cols).saturating_sub(step);
                    selected = new_row * cols;
                    if selected >= projects.len() {
                        selected = projects.len() - 1;
                    }
                }
                KeyCode::PageDown => {
                    let step = list_height;
                    let new_row = (selected / cols) + step;
                    let max_row = (projects.len() - 1) / cols;
                    let target_row = new_row.min(max_row);
                    selected = target_row * cols;
                    if selected >= projects.len() {
                        selected = projects.len() - 1;
                    }
                }
                KeyCode::Home => selected = 0,
                KeyCode::End => selected = projects.len() - 1,
                _ => {}
            }
        }
    }
}