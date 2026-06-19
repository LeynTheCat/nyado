use super::common::color;
use crate::i18n::I18n;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn draw_topbar(frame: &mut Frame, area: Rect, i18n: &I18n) {
    let style = Style::default().bg(color::topbar_bg()).fg(color::topbar_fg());
    let block = Block::default().style(style);
    frame.render_widget(block, area);

    let left_text = format!("[ {} ]", i18n.get("topbar_title"));
    let left_span = Span::styled(left_text, Style::default().bg(color::topbar_bg()).fg(color::topbar_fg()).add_modifier(Modifier::BOLD));
    let left_width = left_span.width();

    let date_str = chrono::Local::now().format(&i18n.get("topbar_date_format")).to_string();
    let right_text = format!("[ {} ]", date_str);
    let right_span = Span::styled(right_text, Style::default().bg(color::topbar_bg()).fg(color::topbar_fg()).add_modifier(Modifier::BOLD));
    let right_width = right_span.width();

    let total_width = left_width + right_width + 4;
    if total_width <= area.width as usize {
        let left_x = area.left() + 2;
        frame.render_widget(Paragraph::new(Line::from(vec![left_span])), Rect::new(left_x, area.top(), left_width as u16, 1));
        let right_x = area.right() - right_width as u16 - 2;
        frame.render_widget(Paragraph::new(Line::from(vec![right_span])), Rect::new(right_x, area.top(), right_width as u16, 1));
    } else {
        let right_x = area.right() - right_width as u16 - 2;
        if right_x > area.left() {
            frame.render_widget(Paragraph::new(Line::from(vec![right_span])), Rect::new(right_x, area.top(), right_width as u16, 1));
        }
    }
}