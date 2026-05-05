use super::common::color;
use crate::i18n::I18n;
use crate::storage::Storage;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw_topbar(frame: &mut Frame, area: Rect, storage: &Storage, i18n: &I18n) {
    let style = Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG);
    let block = Block::default().style(style);
    frame.render_widget(block, area);

    let left_text = i18n.get("topbar_title");
    let left_span = Span::styled(left_text, Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG).add_modifier(Modifier::BOLD));
    let left_width = left_span.content.width();

    let project_name = &storage.current_project;
    let center_str = format!("< {} >", project_name);
    let center_span = Span::styled(center_str, Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG).add_modifier(Modifier::BOLD));
    let center_width = center_span.content.width();

    let right_text = chrono::Local::now().format(&i18n.get("topbar_date_format")).to_string();
    let right_span = Span::styled(right_text, Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG).add_modifier(Modifier::BOLD));
    let right_width = right_span.content.width();

    let total_width = left_width + center_width + right_width + 6;
    if total_width <= area.width as usize {
        let left_x = area.left() + 2;
        frame.render_widget(Paragraph::new(Line::from(vec![left_span])), Rect::new(left_x, area.top(), left_width as u16, 1));
        let center_x = area.left() + (area.width - center_width as u16) / 2;
        frame.render_widget(Paragraph::new(Line::from(vec![center_span])), Rect::new(center_x, area.top(), center_width as u16, 1));
        let right_x = area.right() - right_width as u16 - 2;
        frame.render_widget(Paragraph::new(Line::from(vec![right_span])), Rect::new(right_x, area.top(), right_width as u16, 1));
    } else {
        let center_x = area.left() + (area.width - center_width as u16) / 2;
        frame.render_widget(Paragraph::new(Line::from(vec![center_span])), Rect::new(center_x, area.top(), center_width as u16, 1));
        let right_x = area.right() - right_width as u16 - 2;
        if right_x > area.left() {
            frame.render_widget(Paragraph::new(Line::from(vec![right_span])), Rect::new(right_x, area.top(), right_width as u16, 1));
        }
    }
}