use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::io;

pub fn popup(title: &str, hint: &str, initial: &str, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<Option<String>> {
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
                f.render_widget(Paragraph::new(hint).style(Style::default().dim()), Rect::new(inner.left() + 2, inner.top() + 1, inner.width - 4, 1));
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
            if key.kind == KeyEventKind::Press {
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
                        chars.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                    _ => {}
                }
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