use super::common::{color, tag_color};
use crate::i18n::I18n;
use crate::storage::Storage;
use super::bongo::draw_bongo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::todo::now_secs;

pub fn draw_right_panel(frame: &mut Frame, area: Rect, storage: &Storage, visible: &[usize], selected: usize, i18n: &I18n) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color::BORDER))
        .title(Span::styled(
            format!("[ {} ]", i18n.get("right_title")),
            Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 20 {
        return;
    }

    let stats_header = i18n.get("stats_header");
    let header_line = Line::from(Span::styled(
        format!("[ {} ]", stats_header),
        Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
    ));
    let header_width = header_line.width() as u16;
    let mut y = inner.top() + 1;
    if header_width + 4 <= inner.width {
        frame.render_widget(Paragraph::new(header_line), Rect::new(inner.left() + 2, y, header_width, 1));
    } else {
        let short = format!("[{}]", stats_header.chars().take(inner.width as usize - 4).collect::<String>());
        let short_line = Line::from(Span::styled(short, Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD)));
        let short_width = short_line.width() as u16;
        frame.render_widget(Paragraph::new(short_line), Rect::new(inner.left() + 2, y, short_width, 1));
    }
    y += 1;

    let pending = storage.pending_count();
    let done = storage.done_count();
    let pinned = storage.pinned_count();
    let total = storage.todos.len();

    let pending_prefix = format!("{} ", i18n.get("pending.prefix"));
    let done_prefix = format!("{} ", i18n.get("done.prefix"));
    let pinned_prefix = format!("{} ", i18n.get("pinned.prefix"));
    let total_prefix = format!("{} ", i18n.get("total.prefix"));

    let stats = vec![
        Line::from(vec![
            Span::styled(format!("{:<18}", pending_prefix), Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD)),
            Span::raw(pending.to_string()),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", done_prefix), Style::default().fg(color::GREEN).add_modifier(Modifier::BOLD)),
            Span::raw(done.to_string()),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", pinned_prefix), Style::default().fg(color::PIN).add_modifier(Modifier::BOLD)),
            Span::raw(pinned.to_string()),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", total_prefix), Style::default().dim().add_modifier(Modifier::BOLD)),
            Span::raw(total.to_string()),
        ]),
    ];

    for stat in stats {
        let stat_width = stat.width() as u16;
        if stat_width + 4 <= inner.width {
            frame.render_widget(Paragraph::new(stat), Rect::new(inner.left() + 2, y, stat_width, 1));
        }
        y += 1;
        if y + 2 >= inner.bottom() {
            break;
        }
    }
    y += 1;

    if !storage.tags_available.is_empty() && y < inner.bottom() - 4 {
        let header = Line::from(Span::styled(
            format!("[ {} ]", i18n.get("tags_header")),
            Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
        ));
        let header_w = header.width() as u16;
        if header_w + 4 <= inner.width {
            frame.render_widget(Paragraph::new(header), Rect::new(inner.left() + 2, y, header_w, 1));
        }
        y += 1;
        for (i, (tag, cnt)) in storage.tags_available.iter().take(9).enumerate() {
            let tag_c = tag_color(tag);
            let is_active = !storage.filter_tag.is_empty() && storage.filter_tag == *tag;
            let style = if is_active {
                Style::default().fg(color::SELECTED_FG).bg(color::SELECTED_BG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(tag_c).add_modifier(Modifier::BOLD)
            };
            let line = format!(" {}  #{:<8} ({})", i + 1, tag, cnt);
            let line_width = line.chars().count() as u16;
            if line_width + 4 <= inner.width {
                frame.render_widget(Paragraph::new(Span::styled(line, style)), Rect::new(inner.left() + 2, y, line_width, 1));
            }
            y += 1;
            if y >= inner.bottom() - 4 {
                break;
            }
        }
        y += 1;
    }

    if selected < visible.len() && y < inner.bottom() - 4 {
        let todo = &storage.todos[visible[selected]];
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            format!("[ {} ]", i18n.get("selected_header")),
            Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
        )]));
        for line in todo.text.split('\n').take((inner.bottom() - y - 2) as usize) {
            lines.push(Line::from(Span::styled(line, Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD))));
        }
        if todo.pinned {
            lines.push(Line::from(Span::styled(i18n.get("pinned_marker"), Style::default().fg(color::PIN).add_modifier(Modifier::BOLD))));
        }
        let created = chrono::DateTime::from_timestamp(todo.created_at as i64, 0)
            .map(|dt| dt.format("%d %b %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(vec![Span::raw(i18n.get("created_prefix")), Span::styled(created, Style::default().dim().add_modifier(Modifier::BOLD))]));
        if todo.done && todo.done_at != 0 {
            let done_at = chrono::DateTime::from_timestamp(todo.done_at as i64, 0)
                .map(|dt| dt.format("%d %b %H:%M").to_string())
                .unwrap();
            lines.push(Line::from(vec![Span::raw(i18n.get("done_prefix")), Span::styled(done_at, Style::default().dim().add_modifier(Modifier::BOLD))]));
        }
        if todo.due_date > 0 {
            let now = now_secs();
            let dt = chrono::DateTime::from_timestamp(todo.due_date as i64, 0);
            let due_str = dt.map(|dt| dt.format("%d %b %H:%M").to_string()).unwrap_or_else(|| "?? ??? ??:??".to_string());
            let overdue = todo.due_date < now;
            let style = if overdue {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color::GREEN).add_modifier(Modifier::BOLD)
            };
            lines.push(Line::from(vec![Span::styled("> ", style), Span::styled(due_str, style)]));
        }
        let details_area = Rect::new(inner.left() + 2, y, inner.width - 4, inner.bottom() - y - 2);
        if details_area.width > 0 && details_area.height > 0 {
            let details = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(details, details_area);
        }
    }

    draw_bongo(frame, inner, pending, total, i18n);
}