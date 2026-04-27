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

fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

fn split_chars(s: &str, at_chars: usize) -> (String, String) {
    let left: String = s.chars().take(at_chars).collect();
    let right: String = s.chars().skip(at_chars).collect();
    (left, right)
}

fn remove_char_at(s: &str, at_chars: usize) -> String {
    let mut chars = s.chars();
    let left: String = chars.by_ref().take(at_chars).collect();
    let right: String = chars.skip(1).collect();
    left + &right
}

fn insert_char_at(s: &str, at_chars: usize, c: char) -> String {
    let mut chars = s.chars();
    let left: String = chars.by_ref().take(at_chars).collect();
    let right: String = chars.collect();
    left + &c.to_string() + &right
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn popup_singleline(
    title: &str,
    hint: &str,
    initial: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    max_len: usize,
) -> io::Result<Option<String>> {
    let mut chars: Vec<char> = initial.chars().collect();
    let mut cursor_pos = chars.len();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let area = Rect::new(0, 0, size.width, size.height);
            f.render_widget(Clear, area);
            let popup_width = std::cmp::min(74, size.width - 4);
            let popup_height = 7;
            let popup_x = (size.width - popup_width) / 2;
            let popup_y = (size.height - popup_height) / 2;
            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            if !hint.is_empty() {
                f.render_widget(
                    Paragraph::new(hint).style(Style::default().dim()),
                    Rect::new(inner.left() + 2, inner.top() + 1, inner.width - 4, 1),
                );
            }

            let input_area = Rect::new(inner.left() + 2, inner.top() + 3, inner.width - 4, 1);
            let input_str: String = chars.iter().collect();
            let display = format!("> {}", input_str);
            let mut display_span = Span::styled(display, Style::default().fg(Color::Yellow));
            if cursor_pos == chars.len() {
                display_span = display_span.clone().add_modifier(Modifier::SLOW_BLINK);
            }
            f.render_widget(Paragraph::new(display_span), input_area);
            let cursor_x = inner.left() + 2 + 2 + cursor_pos as u16;
            let cursor_y = inner.top() + 3;
            f.set_cursor(cursor_x, cursor_y);
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
    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

fn popup_multiline_auto(
    title: &str,
    hint: &str,
    initial: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    max_total_len: usize,
    auto_wrap_len: usize,
) -> io::Result<Option<String>> {
    let mut lines: Vec<String> = initial.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    let mut cursor_x: usize = 0;
    let mut cursor_y: usize = 0;
    let mut scroll_offset: usize = 0;

    let total_len = |lines: &[String]| -> usize {
        lines.iter().map(|l| char_len(l)).sum()
    };

    loop {
        let term_size = terminal.size()?;
        let popup_width = std::cmp::min(74, term_size.width - 4);
        let popup_height = 12;
        let popup_x = (term_size.width - popup_width) / 2;
        let popup_y = (term_size.height - popup_height) / 2;
        let input_area_height = (popup_height - 7) as usize;
        let visual_max_line_len = (popup_width - 8) as usize;

        terminal.draw(|f| {
            let area = Rect::new(0, 0, term_size.width, term_size.height);
            f.render_widget(Clear, area);
            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(title, Style::default().fg(Color::Cyan)));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            if !hint.is_empty() {
                f.render_widget(
                    Paragraph::new(hint).style(Style::default().dim()),
                    Rect::new(inner.left() + 2, inner.top() + 1, inner.width - 4, 1),
                );
            }

            let input_area = Rect::new(inner.left() + 2, inner.top() + 3, inner.width - 4, inner.height - 5);
            let display_lines: Vec<Line> = lines
                .iter()
                .skip(scroll_offset)
                .take(input_area.height as usize)
                .map(|line| {
                    if char_len(line) > visual_max_line_len {
                        let truncated = truncate_chars(line, visual_max_line_len - 1);
                        format!("{}…", truncated)
                    } else {
                        line.clone()
                    }
                })
                .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Yellow))))
                .collect();
            let para = Paragraph::new(display_lines);
            f.render_widget(para, input_area);

            let visible_cursor_y = cursor_y.saturating_sub(scroll_offset);
            if visible_cursor_y < input_area.height as usize && cursor_y < lines.len() {
                let cursor_x_abs = inner.left() + 2 + cursor_x as u16;
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
                        let new_line = remove_char_at(line, cursor_x - 1);
                        lines[cursor_y] = new_line;
                        cursor_x -= 1;
                    } else if cursor_y > 0 {
                        let prev_line = lines[cursor_y - 1].clone();
                        let curr_line = lines.remove(cursor_y);
                        let prev_len = char_len(&prev_line);
                        lines[cursor_y - 1] = prev_line + &curr_line;
                        cursor_y -= 1;
                        cursor_x = prev_len;
                        if cursor_y < scroll_offset {
                            scroll_offset = cursor_y;
                        }
                    }
                }
                KeyCode::Delete => {
                    let line = &lines[cursor_y];
                    if cursor_x < char_len(line) {
                        let new_line = remove_char_at(line, cursor_x);
                        lines[cursor_y] = new_line;
                    } else if cursor_y + 1 < lines.len() {
                        let next_line = lines.remove(cursor_y + 1);
                        lines[cursor_y].push_str(&next_line);
                    }
                }
                KeyCode::Left => {
                    if cursor_x > 0 {
                        cursor_x -= 1;
                    } else if cursor_y > 0 {
                        cursor_y -= 1;
                        cursor_x = char_len(&lines[cursor_y]);
                        if cursor_y < scroll_offset {
                            scroll_offset = cursor_y;
                        }
                    }
                }
                KeyCode::Right => {
                    if cursor_x < char_len(&lines[cursor_y]) {
                        cursor_x += 1;
                    } else if cursor_y + 1 < lines.len() {
                        cursor_y += 1;
                        cursor_x = 0;
                        if cursor_y >= scroll_offset + input_area_height {
                            scroll_offset = cursor_y.saturating_sub(input_area_height - 1);
                        }
                    }
                }
                KeyCode::Up => {
                    if cursor_y > 0 {
                        cursor_y -= 1;
                        let new_len = char_len(&lines[cursor_y]);
                        if cursor_x > new_len {
                            cursor_x = new_len;
                        }
                        if cursor_y < scroll_offset {
                            scroll_offset = cursor_y;
                        }
                    }
                }
                KeyCode::Down => {
                    if cursor_y + 1 < lines.len() {
                        cursor_y += 1;
                        let new_len = char_len(&lines[cursor_y]);
                        if cursor_x > new_len {
                            cursor_x = new_len;
                        }
                        if cursor_y >= scroll_offset + input_area_height {
                            scroll_offset = cursor_y.saturating_sub(input_area_height - 1);
                        }
                    }
                }
                KeyCode::Home => cursor_x = 0,
                KeyCode::End => cursor_x = char_len(&lines[cursor_y]),
                KeyCode::PageUp => {
                    cursor_y = cursor_y.saturating_sub(input_area_height);
                    if cursor_y < scroll_offset {
                        scroll_offset = cursor_y;
                    }
                    let new_len = char_len(&lines[cursor_y]);
                    if cursor_x > new_len {
                        cursor_x = new_len;
                    }
                }
                KeyCode::PageDown => {
                    cursor_y = (cursor_y + input_area_height).min(lines.len() - 1);
                    if cursor_y >= scroll_offset + input_area_height {
                        scroll_offset = cursor_y.saturating_sub(input_area_height - 1);
                    }
                    let new_len = char_len(&lines[cursor_y]);
                    if cursor_x > new_len {
                        cursor_x = new_len;
                    }
                }
                KeyCode::Char(c) => {
                    if total_len(&lines) < max_total_len {
                        let line = &lines[cursor_y];
                        let new_line = insert_char_at(line, cursor_x, c);
                        if char_len(&new_line) > auto_wrap_len {
                            let (first, second) = split_chars(&new_line, auto_wrap_len);
                            lines[cursor_y] = first;
                            lines.insert(cursor_y + 1, second);
                            cursor_y += 1;
                            cursor_x = 0;
                            if cursor_y >= scroll_offset + input_area_height {
                                scroll_offset = cursor_y.saturating_sub(input_area_height - 1);
                            }
                        } else {
                            lines[cursor_y] = new_line;
                            cursor_x += 1;
                        }
                    }
                }
                _ => {}
            }
            if cursor_y >= lines.len() {
                cursor_y = lines.len().saturating_sub(1);
            }
            let line_len = char_len(&lines[cursor_y]);
            if cursor_x > line_len {
                cursor_x = line_len;
            }
        }
    }

    let result = lines.join("\n");
    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

pub fn popup(
    title: &str,
    hint: &str,
    initial: &str,
    multiline: bool,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<Option<String>> {
    if !multiline {
        popup_singleline(title, hint, initial, terminal, 16)
    } else {
        popup_multiline_auto(title, hint, initial, terminal, 128, 60)
    }
}