mod common;
mod progress_bar;
mod right_panel;
mod statusbar;
mod todo_list;
mod topbar;

use crate::i18n::I18n;
use crate::storage::Storage;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};
use right_panel::draw_right_panel;
use statusbar::draw_statusbar;
use todo_list::draw_todo_list;
use topbar::draw_topbar;

pub fn draw(
    frame: &mut Frame,
    size: Rect,
    storage: &Storage,
    visible: &[usize],
    selected: usize,
    scroll_state: &mut usize,
    i18n: &I18n,
    message: &str,
    celebrate: bool,
) {
    if celebrate {
        frame.render_widget(Clear, size);
        let lines = vec![
            i18n.get("celebration_line1"),
            i18n.get("celebration_line2"),
            i18n.get("celebration_line3"),
            i18n.get("celebration_line4"),
            i18n.get("celebration_line5"),
            i18n.get("celebration_line6"),
        ];
        let mut spans = Vec::new();
        for line in lines {
            if line.is_empty() {
                spans.push(Line::from(""));
            } else {
                spans.push(Line::from(vec![Span::styled(
                    line,
                    Style::default().fg(common::color::GREEN).add_modifier(Modifier::BOLD),
                )]));
            }
        }
        let para = Paragraph::new(spans).alignment(Alignment::Center);
        frame.render_widget(para, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(size);
    draw_topbar(frame, chunks[0], storage, i18n);
    draw_statusbar(frame, chunks[2], message, i18n);

    let main_area = chunks[1];
    let left_width = if main_area.width > 80 {
        (main_area.width * 58 / 100).max(20)
    } else {
        main_area.width
    };
    let right_width = main_area.width.saturating_sub(left_width);

    if right_width < 20 {
        draw_todo_list(frame, main_area, storage, visible, selected, scroll_state, i18n);
    } else {
        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(left_width), Constraint::Length(right_width)])
            .split(main_area);
        draw_todo_list(frame, horiz[0], storage, visible, selected, scroll_state, i18n);
        draw_right_panel(frame, horiz[1], storage, visible, selected, i18n);
    }
}

pub fn draw_toosmall(frame: &mut Frame, size: Rect) {
    let message = vec![
        "Terminal too small.",
        "Minimum required: 30 columns x 10 rows.",
        "Please resize your terminal and try again.",
    ];
    let para = Paragraph::new(message.join("\n"))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    frame.render_widget(para, size);
}