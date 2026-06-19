use super::common::{
    color, tag_color, draw_bongo_big, draw_bongo_small,
    CAT_BIG_HEIGHT, CAT_SMALL_HEIGHT, visual_width,
};
use crate::i18n::I18n;
use crate::storage::Storage;
use crate::todo::now_secs;
use crate::config::config;
use chrono::{DateTime, Local, Datelike, NaiveDate, Duration};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashSet;

fn draw_content(
    frame: &mut Frame,
    storage: &Storage,
    visible: &[(u64, usize)],
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
        Style::default().fg(color::header()).add_modifier(Modifier::BOLD),
    ));
    let header_width = header_line.width() as u16;
    if header_width + 4 <= content_width {
        frame.render_widget(Paragraph::new(header_line), Rect::new(content_x, y, header_width, 1));
    } else if content_width >= 4 {
        let short = format!("[{}]", stats_header.chars().take((content_width as usize).saturating_sub(4)).collect::<String>());
        let short_line = Line::from(Span::styled(short, Style::default().fg(color::header()).add_modifier(Modifier::BOLD)));
        let short_width = short_line.width() as u16;
        frame.render_widget(Paragraph::new(short_line), Rect::new(content_x, y, short_width, 1));
    }
    y += 1;

    let project_label_raw = i18n.get("project_prefix").trim_end();
    let pending_label_raw = i18n.get("pending_prefix").trim_end();
    let done_label_raw = i18n.get("done_prefix_stat").trim_end();
    let pinned_label_raw = i18n.get("pinned_prefix").trim_end();
    let total_label_raw = i18n.get("total_prefix").trim_end();
    let overdue_label_raw = i18n.get("overdue_prefix").trim_end();

    let labels = vec![
        project_label_raw,
        pending_label_raw,
        done_label_raw,
        pinned_label_raw,
        total_label_raw,
        overdue_label_raw,
    ];

    let max_label_len = labels.iter().map(|s| s.chars().count()).max().unwrap_or(0);

    let mut label_width = max_label_len;
    let min_label_width = 10;
    if label_width < min_label_width {
        label_width = min_label_width;
    }
    let available_for_label = (content_width as usize).saturating_sub(6);
    if label_width > available_for_label {
        label_width = available_for_label;
    }

    let project_label = project_label_raw.to_string();
    let pending_label = pending_label_raw.to_string();
    let done_label = done_label_raw.to_string();
    let pinned_label = pinned_label_raw.to_string();
    let total_label = total_label_raw.to_string();
    let overdue_label = overdue_label_raw.to_string();

    let project_line = Line::from(vec![
        Span::styled(format!("{:<1$}", project_label, label_width), Style::default().fg(color::stats_project()).add_modifier(Modifier::BOLD)),
        Span::styled(": ", Style::default().fg(color::stats_project()).add_modifier(Modifier::BOLD)),
        Span::styled(storage.current_project.clone(), Style::default().fg(color::header()).add_modifier(Modifier::BOLD)),
    ]);
    let project_width = project_line.width() as u16;
    if project_width + 4 <= content_width {
        frame.render_widget(Paragraph::new(project_line), Rect::new(content_x, y, project_width, 1));
    } else if content_width >= 4 {
        let remaining_width = (content_width as usize).saturating_sub(project_label.len() + 4);
        let short_project = if storage.current_project.len() > remaining_width {
            let max_len = remaining_width.saturating_sub(1);
            format!("{}…", &storage.current_project[..max_len])
        } else {
            storage.current_project.clone()
        };
        let short_line = Line::from(vec![
            Span::styled(format!("{:<1$}", project_label, label_width), Style::default().fg(color::stats_pending()).add_modifier(Modifier::BOLD)),
            Span::styled(": ", Style::default().fg(color::stats_pending()).add_modifier(Modifier::BOLD)),
            Span::styled(short_project, Style::default().fg(color::header()).add_modifier(Modifier::BOLD)),
        ]);
        let short_width = short_line.width() as u16;
        frame.render_widget(Paragraph::new(short_line), Rect::new(content_x, y, short_width, 1));
    }
    y += 1;

    let pending = storage.pending_count();
    let done = storage.done_count();
    let pinned = storage.pinned_count();
    let total = storage.total_count();
    let now = now_secs();
    let overdue = storage.todos.iter().filter(|t| !t.done && t.due_date > 0 && t.due_date < now).count();

    let mut stats = Vec::new();
    stats.push(Line::from(vec![
        Span::styled(format!("{:<1$}", pending_label, label_width), Style::default().fg(color::stats_pending()).add_modifier(Modifier::BOLD)),
        Span::styled(": ", Style::default().fg(color::stats_pending()).add_modifier(Modifier::BOLD)),
        Span::raw(pending.to_string()),
    ]));
    stats.push(Line::from(vec![
        Span::styled(format!("{:<1$}", done_label, label_width), Style::default().fg(color::stats_done()).add_modifier(Modifier::BOLD)),
        Span::styled(": ", Style::default().fg(color::stats_done()).add_modifier(Modifier::BOLD)),
        Span::raw(done.to_string()),
    ]));
    stats.push(Line::from(vec![
        Span::styled(format!("{:<1$}", pinned_label, label_width), Style::default().fg(color::stats_pinned()).add_modifier(Modifier::BOLD)),
        Span::styled(": ", Style::default().fg(color::stats_pinned()).add_modifier(Modifier::BOLD)),
        Span::raw(pinned.to_string()),
    ]));
    stats.push(Line::from(vec![
        Span::styled(format!("{:<1$}", total_label, label_width), Style::default().dim().add_modifier(Modifier::BOLD)),
        Span::styled(": ", Style::default().dim().add_modifier(Modifier::BOLD)),
        Span::raw(total.to_string()),
    ]));
    if overdue > 0 {
        stats.push(Line::from(vec![
            Span::styled(format!("{:<1$}", overdue_label, label_width), Style::default().fg(color::stats_overdue()).add_modifier(Modifier::BOLD)),
            Span::styled(": ", Style::default().fg(color::stats_overdue()).add_modifier(Modifier::BOLD)),
            Span::raw(overdue.to_string()),
        ]));
    }

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
            Style::default().fg(color::header()).add_modifier(Modifier::BOLD),
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
                Style::default().fg(color::selected_fg()).bg(color::selected_bg()).add_modifier(Modifier::BOLD)
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
    }

    if config().show_calendar {
        let now_local = Local::now();
        let current_year = now_local.year();
        let current_month = now_local.month();
        let first_day_of_month = NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
        let days_in_month = (NaiveDate::from_ymd_opt(current_year, current_month + 1, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(current_year + 1, 1, 1).unwrap())
            - Duration::days(1))
            .day();
        let week_start = first_day_of_month.weekday().num_days_from_monday();

        let mut uncompleted_due_days = HashSet::new();
        let mut completed_due_days = HashSet::new();
        for todo in &storage.todos {
            if todo.due_date > 0 {
                if let Some(dt) = DateTime::from_timestamp(todo.due_date as i64, 0) {
                    let date = dt.with_timezone(&Local).date_naive();
                    if date.year() == current_year && date.month() == current_month {
                        let day = date.day();
                        if todo.done {
                            completed_due_days.insert(day);
                        } else {
                            uncompleted_due_days.insert(day);
                        }
                    }
                }
            }
        }

        const CALENDAR_MIN_HEIGHT: u16 = 8;
        const CALENDAR_MIN_WIDTH: u16 = 24;

        if y + CALENDAR_MIN_HEIGHT + 1 <= inner.bottom() && content_width >= CALENDAR_MIN_WIDTH {
            y += 1;

            let calendar_title = format!("[ {} ]", i18n.get("calendar_title"));
            let title_line = Line::from(Span::styled(calendar_title, Style::default().fg(color::header()).add_modifier(Modifier::BOLD)));
            let title_width = title_line.width() as u16;
            if title_width + 4 <= content_width {
                frame.render_widget(Paragraph::new(title_line), Rect::new(content_x, y, title_width, 1));
            }
            y += 1;

            let weekdays = [
                i18n.get("calendar_mon"),
                i18n.get("calendar_tue"),
                i18n.get("calendar_wed"),
                i18n.get("calendar_thu"),
                i18n.get("calendar_fri"),
                i18n.get("calendar_sat"),
                i18n.get("calendar_sun"),
            ];
            let mut weekday_spans = Vec::new();
            for &wd in &weekdays {
                let visual_len = visual_width(wd);
                let pad = 3usize.saturating_sub(visual_len);
                let formatted = format!("{}{} ", wd, " ".repeat(pad));
                weekday_spans.push(Span::styled(formatted, Style::default().fg(color::calendar_weekday_header()).add_modifier(Modifier::BOLD)));
            }
            let weekday_line = Line::from(weekday_spans);
            let weekday_width = weekday_line.width() as u16;
            if weekday_width + 4 <= content_width {
                frame.render_widget(Paragraph::new(weekday_line), Rect::new(content_x + 1, y, weekday_width, 1));
            }
            y += 1;

            let mut day = 1;
            let mut row = 0;
            while day <= days_in_month && y < inner.bottom() - 1 {
                let mut day_spans = Vec::new();
                for col in 0..7 {
                    if row == 0 && col < week_start {
                        day_spans.push(Span::raw("   "));
                    } else if day <= days_in_month {
                        let is_today = day == now_local.day() && current_month == now_local.month() && current_year == now_local.year();
                        let has_uncompleted = uncompleted_due_days.contains(&day);
                        let has_completed = completed_due_days.contains(&day);
                        let is_weekend = col == 5 || col == 6;
                        
                        let day_str = format!("{:>2}", day);
                        
                        let style = if is_today {
                            Style::default().fg(color::calendar_today_fg()).bg(color::calendar_today_bg()).add_modifier(Modifier::BOLD)
                        } else if has_uncompleted {
                            Style::default().fg(color::calendar_uncompleted_fg()).bg(color::calendar_uncompleted_bg()).add_modifier(Modifier::BOLD)
                        } else if has_completed {
                            Style::default().fg(color::calendar_completed_fg()).bg(color::calendar_completed_bg()).add_modifier(Modifier::BOLD)
                        } else if is_weekend {
                            Style::default().fg(color::calendar_weekend_fg()).bg(color::calendar_weekend_bg()).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(color::calendar_normal_day())
                        };
                        
                        day_spans.push(Span::raw(" "));
                        day_spans.push(Span::styled(day_str, style));
                        day += 1;
                    } else {
                        day_spans.push(Span::raw("   "));
                    }
                    if col < 6 {
                        day_spans.push(Span::raw(" "));
                    }
                }
                let day_line = Line::from(day_spans);
                let line_width = day_line.width() as u16;
                if line_width + 4 <= content_width {
                    frame.render_widget(Paragraph::new(day_line), Rect::new(content_x, y, line_width, 1));
                }
                y += 1;
                row += 1;
            }
        }
    }

    if selected < visible.len() && y < inner.bottom() - 4 {
        y += 1;
        let (id, _depth) = visible[selected];
        if let Some(todo) = storage.get_todo(id) {
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(vec![Span::styled(
                format!("[ {} ]", i18n.get("selected_header")),
                Style::default().fg(color::header()).add_modifier(Modifier::BOLD),
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
                        Style::default().fg(color::pending()).add_modifier(Modifier::BOLD),
                    )));
                }
            }
            if todo.pinned {
                lines.push(Line::from(Span::styled(
                    i18n.get("pinned_marker"),
                    Style::default().fg(color::pinned_marker()).add_modifier(Modifier::BOLD),
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
                let now_sec = now_secs();
                let dt = DateTime::from_timestamp(todo.due_date as i64, 0);
                if let Some(dt) = dt {
                    let local_dt = dt.with_timezone(&Local);
                    let due_str = local_dt.format("%d %b %H:%M").to_string();
                    let overdue_flag = todo.due_date < now_sec;
                    let due_style = if overdue_flag {
                        Style::default().fg(color::due_overdue()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color::due_future()).add_modifier(Modifier::BOLD)
                    };
                    lines.push(Line::from(vec![
                        Span::styled("> ", due_style),
                        Span::styled(due_str, due_style),
                    ]));
                }
            }

            let available_height = inner.bottom().saturating_sub(y).saturating_sub(2);
            if content_width > 0 && available_height > 0 {
                let lines_len = lines.len() as u16;
                let details_area = Rect::new(content_x, y, content_width, available_height);
                let details = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
                frame.render_widget(details, details_area);
                y += lines_len;
            }
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
    } else if pending <= 15 {
        i18n.get("mood_lots")
    } else if pending <= 31 {
        i18n.get("mood_heap")
    } else if pending <= 63 {
        i18n.get("mood_pile")
    } else if pending <= 127 {
        i18n.get("mood_overwhelming")
    } else if pending <= 255 {
        i18n.get("mood_hectic")
    } else {
        i18n.get("mood_crazy")
    };
    let mood_span = Span::styled(mood, Style::default().fg(color::bongo()).bg(Color::Reset).add_modifier(Modifier::BOLD));
    let mood_width = mood.chars().count() as u16;
    if mood_width + 2 <= area.width {
        frame.render_widget(
            Paragraph::new(mood_span).alignment(Alignment::Right),
            Rect::new(area.left(), area.bottom() - 1, area.width, 1),
        );
    }
}

pub fn draw_right_panel(
    frame: &mut Frame,
    area: Rect,
    storage: &Storage,
    visible: &[(u64, usize)],
    selected: usize,
    i18n: &I18n,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color::border()))
        .title(Span::styled(
            format!("[ {} ]", i18n.get("right_title")),
            Style::default().fg(color::header()).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 20 {
        return;
    }

    let big_cat_width = super::common::CAT_ASCII_BIG
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20) as u16;
    let big_cat_height = CAT_BIG_HEIGHT as u16;
    let small_cat_width = super::common::CAT_ASCII_SMALL
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20) as u16;
    let small_cat_height = CAT_SMALL_HEIGHT as u16;

    let min_content_width = 30;
    let min_content_height = 15;

    let can_horizontal = inner.width >= big_cat_width + min_content_width + 4
        && inner.height >= big_cat_height + 2;
    let can_vertical = inner.height >= small_cat_height + min_content_height + 2
        && inner.width >= small_cat_width + 2;

    if can_horizontal {
        let content_width = inner.width - big_cat_width - 4;
        let content_x = inner.left() + 2;
        let content_rect = Rect::new(content_x, inner.top(), content_width, inner.height);
        draw_content(
            frame,
            storage,
            visible,
            selected,
            i18n,
            content_rect,
            content_x,
            content_width,
            content_rect.y,
        );
        let cat_x = inner.right() - big_cat_width - 1;
        let cat_area = Rect::new(cat_x, inner.top(), big_cat_width, inner.height);
        draw_bongo_big(frame, cat_area);
    } else if can_vertical {
        let cat_area = Rect::new(inner.left(), inner.top(), inner.width, small_cat_height);
        draw_bongo_small(frame, cat_area);
        let content_y = inner.top() + small_cat_height;
        let content_height = inner.height - small_cat_height;
        if content_height > 0 {
            let content_rect = Rect::new(inner.left() + 2, content_y, inner.width - 4, content_height);
            draw_content(
                frame,
                storage,
                visible,
                selected,
                i18n,
                content_rect,
                content_rect.x,
                content_rect.width,
                content_rect.y,
            );
        }
    } else {
        draw_content(
            frame,
            storage,
            visible,
            selected,
            i18n,
            inner,
            inner.left() + 2,
            inner.width - 4,
            inner.top() + 1,
        );
    }

    draw_mood(frame, inner, storage.pending_count(), storage.total_count(), i18n);
}