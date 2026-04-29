use super::common::{color, tag_color, draw_bongo, CAT_HEIGHT};
use crate::i18n::I18n;
use crate::storage::Storage;
use crate::todo::now_secs;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn draw_content(
    frame: &mut Frame,
    storage: &Storage,
    visible: &[usize],
    selected: usize,
    i18n: &I18n,
    inner: Rect,
    content_x: u16,
    content_width: u16,
    start_y: u16,
) -> u16 {
    let mut y = start_y;
    let stats_header = i18n.get("stats_header");
    let header_line = Line::from(Span::styled(
        format!("[ {} ]", stats_header),
        Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
    ));
    let header_width = header_line.width() as u16;
    if header_width + 4 <= content_width {
        frame.render_widget(Paragraph::new(header_line), Rect::new(content_x, y, header_width, 1));
    } else if content_width >= 4 {
        let short = format!("[{}]", stats_header.chars().take((content_width as usize).saturating_sub(4)).collect::<String>());
        let short_line = Line::from(Span::styled(short, Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD)));
        let short_width = short_line.width() as u16;
        frame.render_widget(Paragraph::new(short_line), Rect::new(content_x, y, short_width, 1));
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
        if stat_width + 4 <= content_width {
            frame.render_widget(Paragraph::new(stat), Rect::new(content_x, y, stat_width, 1));
        }
        y += 1;
        if y + 2 >= inner.bottom() {
            return y;
        }
    }
    y += 1;

    if !storage.tags_available.is_empty() && y < inner.bottom() - 4 {
        let header = Line::from(Span::styled(
            format!("[ {} ]", i18n.get("tags_header")),
            Style::default().fg(color::HEADER).add_modifier(Modifier::BOLD),
        ));
        let header_w = header.width() as u16;
        if header_w + 4 <= content_width {
            frame.render_widget(Paragraph::new(header), Rect::new(content_x, y, header_w, 1));
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
            if line_width + 4 <= content_width {
                frame.render_widget(Paragraph::new(Span::styled(line, style)), Rect::new(content_x, y, line_width, 1));
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
        for line in todo.text.split('\n') {
            let mut remaining = line;
            while !remaining.is_empty() {
                let split_at = if remaining.chars().count() > content_width as usize {
                    let end = remaining
                        .char_indices()
                        .nth(content_width as usize)
                        .map(|(i, _)| i)
                        .unwrap_or(remaining.len());
                    let (first, rest) = remaining.split_at(end);
                    remaining = rest;
                    first
                } else {
                    let all = remaining;
                    remaining = "";
                    all
                };
                lines.push(Line::from(Span::styled(
                    split_at,
                    Style::default().fg(color::PENDING).add_modifier(Modifier::BOLD),
                )));
            }
        }
        if todo.pinned {
            lines.push(Line::from(Span::styled(
                i18n.get("pinned_marker"),
                Style::default().fg(color::PIN).add_modifier(Modifier::BOLD),
            )));
        }

        let created = DateTime::from_timestamp(todo.created_at as i64, 0)
            .map(|dt| dt.with_timezone(&Local).format("%d %b %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(vec![
            Span::raw(i18n.get("created_prefix")),
            Span::styled(created, Style::default().dim().add_modifier(Modifier::BOLD)),
        ]));

        if todo.done && todo.done_at != 0 {
            let done_at = DateTime::from_timestamp(todo.done_at as i64, 0)
                .map(|dt| dt.with_timezone(&Local).format("%d %b %H:%M").to_string())
                .unwrap();
            lines.push(Line::from(vec![
                Span::raw(i18n.get("done_prefix")),
                Span::styled(done_at, Style::default().dim().add_modifier(Modifier::BOLD)),
            ]));
        }

        if todo.due_date > 0 {
            let now = now_secs();
            let dt = DateTime::from_timestamp(todo.due_date as i64, 0);
            if let Some(dt) = dt {
                let local_dt = dt.with_timezone(&Local);
                let due_str = local_dt.format("%d %b %H:%M").to_string();
                let overdue = todo.due_date < now;
                let due_style = if overdue {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color::GREEN).add_modifier(Modifier::BOLD)
                };
                lines.push(Line::from(vec![
                    Span::styled("> ", due_style),
                    Span::styled(due_str, due_style),
                ]));
            }
        }

        let available_height = inner.bottom().saturating_sub(y).saturating_sub(2);
        if content_width > 0 && available_height > 0 {
            let details_area = Rect::new(content_x, y, content_width, available_height);
            let details = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(details, details_area);
        }
    }
    y
}

fn draw_mood(frame: &mut Frame, area: Rect, pending: usize, total: usize, i18n: &I18n) {
    let mood = if pending == 0 && total > 0 {
        i18n.get("mood_all_done")
    } else if pending == 0 {
        i18n.get("mood_empty")
    } else if pending == 1 {
        i18n.get("mood_one")
    } else if pending <= 3 {
        i18n.get("mood_few")
    } else if pending <= 7 {
        i18n.get("mood_several")
    } else {
        i18n.get("mood_many")
    };
    let mood_span = Span::styled(mood, Style::default().fg(color::BONGO).bg(Color::Reset).add_modifier(Modifier::BOLD));
    let mood_width = mood.chars().count() as u16;
    if mood_width + 2 <= area.width {
        frame.render_widget(
            Paragraph::new(mood_span).alignment(Alignment::Right),
            Rect::new(area.left(), area.bottom() - 1, area.width, 1),
        );
    }
}

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

    let cat_width = super::common::CAT_ASCII.iter().map(|l| l.chars().count()).max().unwrap_or(20) as u16;
    let cat_height = CAT_HEIGHT as u16;
    let min_content_width = 30;
    let min_content_height = 15;

    let can_horizontal = inner.width >= cat_width + min_content_width + 4 && inner.height >= cat_height + 2;
    let can_vertical = inner.height >= cat_height + min_content_height + 2 && inner.width >= cat_width + 2;

    if can_horizontal {
        let content_width = inner.width - cat_width - 4;
        let content_x = inner.left() + 2;
        let content_rect = Rect::new(content_x, inner.top(), content_width, inner.height);
        draw_content(frame, storage, visible, selected, i18n, content_rect, content_x, content_width, content_rect.y);
        let cat_x = inner.right() - cat_width - 1;
        let cat_area = Rect::new(cat_x, inner.top(), cat_width, inner.height);
        draw_bongo(frame, cat_area);
        draw_mood(frame, cat_area, storage.pending_count(), storage.todos.len(), i18n);
    } else if can_vertical {
        let cat_area = Rect::new(inner.left(), inner.top(), inner.width, cat_height);
        draw_bongo(frame, cat_area);
        let content_y = inner.top() + cat_height;
        let content_height = inner.height - cat_height;
        if content_height > 0 {
            let content_rect = Rect::new(inner.left() + 2, content_y, inner.width - 4, content_height);
            draw_content(frame, storage, visible, selected, i18n, content_rect, content_rect.x, content_rect.width, content_rect.y);
        }
        draw_mood(frame, inner, storage.pending_count(), storage.todos.len(), i18n);
    } else {
        draw_content(frame, storage, visible, selected, i18n, inner, inner.left() + 2, inner.width - 4, inner.top() + 1);
        draw_mood(frame, inner, storage.pending_count(), storage.todos.len(), i18n);
    }
}