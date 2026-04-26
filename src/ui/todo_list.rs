use super::common::{color, tag_color, truncate_text};
use crate::i18n::I18n;
use crate::storage::Storage;
use super::progress_bar::draw_progress_bar;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_todo_list(
    frame: &mut Frame,
    area: Rect,
    storage: &Storage,
    visible: &[usize],
    selected: usize,
    scroll_state: &mut usize,
    i18n: &I18n,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color::BORDER))
        .title(Span::styled(
            format!("[ {} ]", i18n.get("title")),
            Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let total = storage.todos.len();
    if total > 0 && inner.width > 15 && inner.width >= 2 {
        let done = storage.done_count();
        let prog_area = Rect::new(inner.left() + 1, inner.top(), inner.width - 2, 1);
        if prog_area.width > 0 {
            draw_progress_bar(frame, prog_area, done, total);
        }
    }

    if inner.width > 20 {
        let header = Line::from(vec![Span::raw(i18n.get("column_header"))]);
        let header_width = header.width() as u16;
        if header_width + 2 <= inner.width {
            frame.render_widget(Paragraph::new(header).style(Style::default().dim().add_modifier(Modifier::BOLD)),
                                Rect::new(inner.left() + 1, inner.top() + 2, header_width, 1));
        }
        let line_len = (inner.width - 2) as usize;
        if line_len > 0 {
            let divider = Line::from(vec![Span::raw("─".repeat(line_len))]);
            if line_len as u16 <= inner.width - 2 {
                frame.render_widget(Paragraph::new(divider).style(Style::default().fg(color::BORDER)),
                                    Rect::new(inner.left() + 1, inner.top() + 3, inner.width - 2, 1));
            }
        }
    }

    let list_start_y = inner.top() + 5;
    let list_height = inner.height.saturating_sub(6) as usize;
    if list_height == 0 {
        return;
    }

    let total_items = visible.len();
    if total_items == 0 {
        let msg1 = i18n.get("empty_list_line1");
        let msg2 = i18n.get("empty_list_line2");
        let msg1_len = msg1.chars().count() as u16;
        let msg2_len = msg2.chars().count() as u16;

        // Вычисляем центр области списка
        let center_y = list_start_y + (list_height / 2) as u16;
        let y1 = center_y.saturating_sub(1); // первая строка на одну выше центра
        let y2 = y1 + 1;                   // вторая строка

        if msg1_len + 2 <= inner.width && y1 < inner.bottom() - 1 {
            frame.render_widget(
                Paragraph::new(Span::styled(msg1, Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD))),
                Rect::new(inner.left() + 2, y1, msg1_len, 1),
            );
        }
        if msg2_len + 2 <= inner.width && y2 < inner.bottom() - 1 {
            frame.render_widget(
                Paragraph::new(Span::styled(msg2, Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD))),
                Rect::new(inner.left() + 2, y2, msg2_len, 1),
            );
        }
        return;
    }

    if selected < *scroll_state {
        *scroll_state = selected;
    } else if selected >= *scroll_state + list_height {
        *scroll_state = selected.saturating_sub(list_height - 1);
    }
    if *scroll_state + list_height > total_items {
        *scroll_state = total_items.saturating_sub(list_height);
    }

    if *scroll_state > 0 {
        let up = i18n.get("scroll_up");
        let up_len = up.chars().count() as u16;
        if up_len + 1 <= inner.width {
            frame.render_widget(
                Paragraph::new(Span::styled(up, Style::default().fg(color::SEARCH).bg(Color::Reset).add_modifier(Modifier::BOLD))),
                Rect::new(inner.right() - up_len, list_start_y - 1, up_len, 1),
            );
        }
    }
    if *scroll_state + list_height < total_items {
        let down = i18n.get("scroll_down");
        let down_len = down.chars().count() as u16;
        if down_len + 1 <= inner.width {
            frame.render_widget(
                Paragraph::new(Span::styled(down, Style::default().fg(color::SEARCH).bg(Color::Reset).add_modifier(Modifier::BOLD))),
                Rect::new(inner.right() - down_len, inner.bottom() - 2, down_len, 1),
            );
        }
    }

    for i in 0..list_height {
        let vi = *scroll_state + i;
        if vi >= total_items {
            break;
        }
        let todo_idx = visible[vi];
        let todo = &storage.todos[todo_idx];
        let is_selected = vi == selected;
        let y = list_start_y + i as u16;

        let base_style = if is_selected {
            Style::default().bg(color::SELECTED_BG).fg(color::SELECTED_FG).add_modifier(Modifier::BOLD)
        } else if todo.done {
            Style::default().fg(color::DONE).dim()
        } else if todo.pinned {
            Style::default().fg(color::PIN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD)
        };

        let pin_mark = if todo.pinned && !todo.done { '*' } else { ' ' };
        let done_mark = if todo.done { 'x' } else { ' ' };

        let mut spans = Vec::new();
        spans.push(Span::styled(format!("{} [{}] ", pin_mark, done_mark), base_style));

        if !todo.tag.is_empty() {
            let tag_c = tag_color(&todo.tag);
            let tag_style = if is_selected {
                base_style
            } else {
                Style::default().fg(tag_c).add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled(format!("#{} ", todo.tag), tag_style));
        }

        let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
        let max_width = (inner.width as usize).saturating_sub(used_width + 2);
        let text_display = if todo.text.len() > max_width {
            truncate_text(&todo.text, max_width.saturating_sub(1))
        } else {
            todo.text.clone()
        };
        spans.push(Span::styled(text_display, base_style));

        let line = Line::from(spans);
        let line_len = line.width() as u16;
        if y < inner.bottom() - 1 && line_len + 1 <= inner.width {
            frame.render_widget(Paragraph::new(line), Rect::new(inner.left() + 1, y, line_len, 1));
        }
    }
}