use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
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

        let input_line_width = (popup_width - 8) as usize;
        let full_text: String = chars.iter().collect();
        let cursor_visual = visual_offset(&full_text, cursor_pos);
        if cursor_visual >= scroll_offset + input_line_width {
            scroll_offset = cursor_visual.saturating_sub(input_line_width - 1);
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
                f.render_widget(
                    Paragraph::new(hint).style(Style::default().dim()),
                    Rect::new(inner.left() + 2, inner.top() + 1, inner.width.saturating_sub(4), 1),
                );
            }

            let input_area = Rect::new(inner.left() + 2, inner.top() + 3, inner.width.saturating_sub(4), 1);
            let display_span = Span::styled(format!("> {}", display_text), Style::default().fg(Color::Yellow));
            f.render_widget(Paragraph::new(display_span), input_area);

            let cursor_visual_rel = cursor_visual.saturating_sub(scroll_offset);
            let cursor_x = inner.left() + 2 + 2 + cursor_visual_rel as u16;
            let cursor_y = inner.top() + 3;
            if cursor_visual_rel <= input_line_width {
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
                f.set_cursor(cursor_x_abs, cursor_y_abs);
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
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    lines.push("");
    let hint_lines: Vec<&str> = hint.lines().collect();
    let hint_height = if hint.is_empty() { 0 } else { hint_lines.len() };
    let max_content_len = lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);
    let max_hint_len = hint_lines.iter().map(|l| visual_width(l)).max().unwrap_or(0);
    let needed_width = max_content_len.max(max_hint_len) + 6;

    loop {
        let term_size = term.size()?;
        let popup_width = std::cmp::min(needed_width as u16, term_size.width.saturating_sub(4));
        let needed_height = lines.len() + 4 + hint_height;
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
                let hint_area = Rect::new(inner.left() + 2, inner.top() + 1, inner.width.saturating_sub(4), hint_height as u16);
                let hint_para = Paragraph::new(hint).style(Style::default().dim());
                f.render_widget(hint_para, hint_area);
            }

            let available_width = inner.width as usize - 4;
            let display_lines: Vec<Line> = lines
                .iter()
                .map(|line| {
                    if visual_width(line) > available_width {
                        let truncated = truncate_by_width(line, available_width - 1);
                        format!("{}…", truncated)
                    } else {
                        (*line).to_string()
                    }
                })
                .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Yellow))))
                .collect();
            let para = Paragraph::new(display_lines);
            let content_y = inner.top() + 2 + hint_height as u16;
            let content_height = inner.height.saturating_sub(2 + hint_height as u16);
            let content_area = Rect::new(inner.left() + 2, content_y, inner.width.saturating_sub(4), content_height);
            f.render_widget(para, content_area);
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
    match mode {
        PopupMode::Singleline => popup_singleline(title, hint, initial, terminal, 16),
        PopupMode::Multiline => popup_multiline_editable(title, hint, initial, terminal, 256, 60),
        PopupMode::Readonly => {
            popup_readonly(title, hint, initial, terminal)?;
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
    loop {
        let term_size = terminal.size()?;
        let help_lines = 4;
        let popup_width = std::cmp::min(74, term_size.width.saturating_sub(4));
        let max_list_height = (term_size.height as usize).saturating_sub(4 + help_lines).min(projects.len());
        let rows = (projects.len() + cols - 1) / cols;
        let list_height = max_list_height.min(rows);
        let popup_height = list_height + 4 + help_lines;
        let popup_height = popup_height.min(term_size.height.saturating_sub(4) as usize);
        let popup_x = (term_size.width.saturating_sub(popup_width)) / 2;
        let popup_y = (term_size.height.saturating_sub(popup_height as u16)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height as u16);
        let help_width = 20usize;
        let left_width = (popup_width as usize).saturating_sub(help_width + 3);
        let col_width = left_width / cols;
        let separator = "│";
        if selected < scroll_offset {
            scroll_offset = selected / cols;
        }
        if selected >= (scroll_offset + list_height) * cols {
            scroll_offset = (selected / cols).saturating_sub(list_height - 1);
        }
        terminal.draw(|f| {
            f.render_widget(Clear, popup_area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);
            let mut y = inner.top() + 1;
            for row in scroll_offset..(scroll_offset + list_height) {
                for col in 0..cols {
                    let idx = row * cols + col;
                    if idx >= projects.len() {
                        continue;
                    }
                    let item = &projects[idx];
                    let display = if visual_width(item) > col_width - 2 {
                        truncate_by_width(item, col_width - 3) + "…"
                    } else {
                        item.clone()
                    };
                    let prefix = if idx == selected { "> " } else { "  " };
                    let line = format!("{}{}", prefix, display);
                    let style = if idx == selected {
                        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
                    } else if item == current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    let span = Span::styled(line, style);
                    let x = inner.left() + 2 + col as u16 * (col_width as u16 + 2);
                    f.render_widget(Paragraph::new(span), Rect::new(x, y, col_width as u16, 1));
                }
                y += 1;
            }
            let sep_x = inner.left() + left_width as u16 + 1;
            for i in 0..(popup_height - 2) {
                f.render_widget(
                    Paragraph::new(Span::styled(separator, Style::default().fg(Color::Cyan))),
                    Rect::new(sep_x, inner.top() + i as u16, 1, 1),
                );
            }
            let help_x = inner.left() + left_width as u16 + 3;
            let total_height = popup_height - 2;
            let help_start_y = inner.top() + ((total_height.saturating_sub(help_lines)) / 2) as u16;
            let mut help_y = help_start_y;
            let help_switch_line = format!("{}: {}", hint_enter, help_switch);
            let help_create_line = format!("{}: {}", hint_c, help_create);
            let help_rename_line = format!("{}: {}", hint_r, help_rename);
            let help_delete_line = format!("{}: {}", hint_d, help_delete);
            f.render_widget(
                Paragraph::new(Span::styled(help_switch_line, Style::default().fg(Color::Yellow))),
                Rect::new(help_x, help_y, help_width as u16, 1),
            );
            help_y += 1;
            f.render_widget(
                Paragraph::new(Span::styled(help_create_line, Style::default().fg(Color::Yellow))),
                Rect::new(help_x, help_y, help_width as u16, 1),
            );
            help_y += 1;
            f.render_widget(
                Paragraph::new(Span::styled(help_rename_line, Style::default().fg(Color::Yellow))),
                Rect::new(help_x, help_y, help_width as u16, 1),
            );
            help_y += 1;
            f.render_widget(
                Paragraph::new(Span::styled(help_delete_line, Style::default().fg(Color::Yellow))),
                Rect::new(help_x, help_y, help_width as u16, 1),
            );
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
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('л') => {
                    if selected >= cols {
                        selected -= cols;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('о') => {
                    let next = selected + cols;
                    if next < projects.len() {
                        selected = next;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('р') => {
                    if selected % cols > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('д') => {
                    if selected + 1 < projects.len() && (selected % cols) < cols - 1 {
                        selected += 1;
                    }
                }
                _ => {}
            }
        }
    }
}