mod common;
pub mod progress_bar;
mod right_panel;
mod search_box;
mod statusbar;
mod todo_list;
mod topbar;

use crate::i18n::I18n;
use crate::storage::Storage;
use progress_bar::ProgressState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Paragraph},
    Frame,
};
use right_panel::draw_right_panel;
use search_box::draw_searchbox;
use statusbar::draw_statusbar;
use todo_list::draw_todo_list;
use topbar::draw_topbar;

fn draw_celebration(frame: &mut Frame, size: Rect, i18n: &I18n) {
    let lines = vec![
        i18n.get("celebration_line1"),
        i18n.get("celebration_line2"),
        i18n.get("celebration_line3"),
        i18n.get("celebration_line4"),
        i18n.get("celebration_line5"),
        i18n.get("celebration_line6"),
    ];
    let num_lines = lines.len();
    let total_height = size.height;
    let padding_top = (total_height.saturating_sub(num_lines as u16)) / 2;
    let mut text = String::new();
    for _ in 0..padding_top {
        text.push('\n');
    }
    for line in lines {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        if line.is_empty() {
            text.push(' ');
        } else {
            text.push_str(line);
        }
    }
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(common::color::GREEN).add_modifier(Modifier::BOLD));
    frame.render_widget(para, size);
}

pub fn draw(
    frame: &mut Frame,
    size: Rect,
    storage: &Storage,
    visible: &[usize],
    selected: usize,
    scroll_state: &mut usize,
    i18n: &I18n,
    message: &str,
    celebrate: u8,
    progress_state: &mut ProgressState,
    search_mode: bool,
    search_buffer: &str,
) {
    if celebrate > 0 {
        draw_celebration(frame, size, i18n);
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
        if search_mode {
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(5)])
                .split(main_area);
            draw_todo_list(frame, left_chunks[0], storage, visible, selected, scroll_state, i18n, progress_state, search_mode, search_buffer);
            draw_searchbox(frame, left_chunks[1], search_buffer);
        } else {
            draw_todo_list(frame, main_area, storage, visible, selected, scroll_state, i18n, progress_state, search_mode, search_buffer);
        }
    } else {
        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(left_width), Constraint::Length(right_width)])
            .split(main_area);
        if search_mode {
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(5)])
                .split(horiz[0]);
            draw_todo_list(frame, left_chunks[0], storage, visible, selected, scroll_state, i18n, progress_state, search_mode, search_buffer);
            draw_searchbox(frame, left_chunks[1], search_buffer);
        } else {
            draw_todo_list(frame, horiz[0], storage, visible, selected, scroll_state, i18n, progress_state, search_mode, search_buffer);
        }
        draw_right_panel(frame, horiz[1], storage, visible, selected, i18n);
    }
}

pub fn draw_toosmall(frame: &mut Frame, size: Rect) {
    let message = vec![
        "oh nyado needs a bigger home",
        "",
        "Minimum size: 30 columns x 10 rows",
        "Please resize your terminal (or pet the cat)",
    ];
    let para = Paragraph::new(message.join("\n"))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    frame.render_widget(para, size);
}