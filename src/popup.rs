use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style, Stylize},
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

fn split_by_width(line: &str, max_width: usize) -> (String, String) {
    let mut acc = 0;
    let mut split_idx = 0;
    for (i, ch) in line.chars().enumerate() {
        let w = ch.width().unwrap_or(1);
        if acc + w > max_width {
            break;
        }
        acc += w;
        split_idx = i + 1;
    }
    if split_idx == 0 {
        split_idx = 1;
    }
    let first = line.chars().take(split_idx).collect();
    let second = line.chars().skip(split_idx).collect();
    (first, second)
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
    Ok(if result.is_empty() { None } else { Some(result) })
}

fn popup_multiline_editable(
    title: &str,
    hint: &str,
    initial: &str,
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
    max_total_chars: usize,
    max_line_width: usize,
) -> io::Result<Option<String>> {
    let mut lines: Vec<String> = initial.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    let mut cursor_x;
    let mut cursor_y;
    let mut scroll_offset: usize = 0;

    let total_chars = |lines: &[String]| -> usize {
        lines.iter().map(|l| l.chars().count()).sum()
    };

    fn enforce_wrap(lines: &mut Vec<String>, max_width: usize) {
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            if visual_width(line) > max_width {
                let (first, second) = split_by_width(line, max_width);
                lines[i] = first;
                lines.insert(i + 1, second);
            } else {
                i += 1;
            }
        }
    }

    enforce_wrap(&mut lines, max_line_width);
    cursor_y = 0;
    cursor_x = 0;

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
        let visual_max_line_len = (popup_width - 8) as usize;
        let effective_max_width = visual_max_line_len.min(max_line_width);

        if cursor_y < scroll_offset {
            scroll_offset = cursor_y;
        }
        if cursor_y >= scroll_offset + input_area_height {
            scroll_offset = cursor_y.saturating_sub(input_area_height - 1);
        }

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
                    if cursor_x > 0 {
                        let line = &lines[cursor_y];
                        let mut chars: Vec<char> = line.chars().collect();
                        chars.remove(cursor_x - 1);
                        lines[cursor_y] = chars.into_iter().collect();
                        cursor_x -= 1;
                        // попытка соединить строки, если текущая стала короткой
                        if cursor_y > 0 {
                            let combined = format!("{}{}", lines[cursor_y - 1], lines[cursor_y]);
                            if visual_width(&combined) <= effective_max_width {
                                lines[cursor_y - 1] = combined;
                                lines.remove(cursor_y);
                                cursor_y -= 1;
                                cursor_x = lines[cursor_y].chars().count();
                            }
                        }
                        // если текущая слишком длинная, разбиваем
                        if visual_width(&lines[cursor_y]) > effective_max_width {
                            let (first, second) = split_by_width(&lines[cursor_y], effective_max_width);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 0;
                        }
                    } else if cursor_y > 0 {
                        let prev_line = lines[cursor_y - 1].clone();
                        let prev_len = prev_line.chars().count();
                        let curr_line = lines.remove(cursor_y);
                        let new_line = format!("{}{}", prev_line, curr_line);
                        lines[cursor_y - 1] = new_line;
                        cursor_y -= 1;
                        cursor_x = prev_len;
                        if visual_width(&lines[cursor_y]) > effective_max_width {
                            let (first, second) = split_by_width(&lines[cursor_y], effective_max_width);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 0;
                        }
                    }
                }
                KeyCode::Delete => {
                    let line = &lines[cursor_y];
                    let mut chars: Vec<char> = line.chars().collect();
                    if cursor_x < chars.len() {
                        chars.remove(cursor_x);
                        lines[cursor_y] = chars.into_iter().collect();
                        if visual_width(&lines[cursor_y]) > effective_max_width {
                            let (first, second) = split_by_width(&lines[cursor_y], effective_max_width);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 0;
                        }
                    } else if cursor_y + 1 < lines.len() {
                        let next_line = lines.remove(cursor_y + 1);
                        lines[cursor_y].push_str(&next_line);
                        if visual_width(&lines[cursor_y]) > effective_max_width {
                            let (first, second) = split_by_width(&lines[cursor_y], effective_max_width);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 0;
                        }
                    }
                }
                KeyCode::Left => {
                    if cursor_x > 0 {
                        cursor_x -= 1;
                    } else if cursor_y > 0 {
                        cursor_y -= 1;
                        cursor_x = lines[cursor_y].chars().count();
                    }
                }
                KeyCode::Right => {
                    let line_len = lines[cursor_y].chars().count();
                    if cursor_x < line_len {
                        cursor_x += 1;
                    } else if cursor_y + 1 < lines.len() {
                        cursor_y += 1;
                        cursor_x = 0;
                    }
                }
                KeyCode::Up => {
                    if cursor_y > 0 {
                        cursor_y -= 1;
                        let new_len = lines[cursor_y].chars().count();
                        if cursor_x > new_len {
                            cursor_x = new_len;
                        }
                    }
                }
                KeyCode::Down => {
                    if cursor_y + 1 < lines.len() {
                        cursor_y += 1;
                        let new_len = lines[cursor_y].chars().count();
                        if cursor_x > new_len {
                            cursor_x = new_len;
                        }
                    }
                }
                KeyCode::Home => cursor_x = 0,
                KeyCode::End => cursor_x = lines[cursor_y].chars().count(),
                KeyCode::PageUp => {
                    cursor_y = cursor_y.saturating_sub(input_area_height);
                    let new_len = lines[cursor_y].chars().count();
                    if cursor_x > new_len {
                        cursor_x = new_len;
                    }
                }
                KeyCode::PageDown => {
                    cursor_y = (cursor_y + input_area_height).min(lines.len() - 1);
                    let new_len = lines[cursor_y].chars().count();
                    if cursor_x > new_len {
                        cursor_x = new_len;
                    }
                }
                KeyCode::Char(c) => {
                    if total_chars(&lines) < max_total_chars {
                        let line = &lines[cursor_y];
                        let mut chars: Vec<char> = line.chars().collect();
                        chars.insert(cursor_x, c);
                        lines[cursor_y] = chars.into_iter().collect();
                        cursor_x += 1;
                        if visual_width(&lines[cursor_y]) > effective_max_width {
                            let (first, second) = split_by_width(&lines[cursor_y], effective_max_width);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 1;
                        }
                    }
                }
                _ => {}
            }
            if cursor_y >= lines.len() {
                cursor_y = lines.len().saturating_sub(1);
            }
            let line_len = lines[cursor_y].chars().count();
            if cursor_x > line_len {
                cursor_x = line_len;
            }
        }
    }
    let result = lines.join("\n");
    Ok(if result.is_empty() { None } else { Some(result) })
}

fn popup_readonly(
    title: &str,
    hint: &str,
    content: &str,
    term: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
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

pub fn popup(
    title: &str,
    hint: &str,
    initial: &str,
    multiline: bool,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<Option<String>> {
    let mode = if multiline {
        PopupMode::Multiline
    } else {
        PopupMode::Singleline
    };
    popup_with_mode(title, hint, initial, mode, terminal)
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