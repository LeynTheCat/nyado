use super::common::{color, truncate_text_by_width};
use crate::i18n::I18n;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw_statusbar(frame: &mut Frame, area: Rect, message: &str, i18n: &I18n) {
    if area.width == 0 {
        return;
    }
    let style = Style::default().bg(color::STATUSBAR_BG).fg(color::STATUSBAR_FG);
    let block = Block::default().style(style);
    frame.render_widget(block, area);

    let hints = [
        i18n.get("statusbar_hint_wide"),
        i18n.get("statusbar_hint_medium"),
        i18n.get("statusbar_hint_narrow"),
    ];
    let mut chosen_hint = hints[2];
    for &hint in &hints {
        let hint_width = hint.width() as u16;
        if hint_width + 2 <= area.width {
            chosen_hint = hint;
            break;
        }
    }
    let hint_width = chosen_hint.width() as u16;
    let x = area.left() + 2;
    if hint_width + 2 <= area.width {
        frame.render_widget(Paragraph::new(chosen_hint), Rect::new(x, area.top(), hint_width, 1));
    }

    if !message.is_empty() {
        let max_msg_width = (area.width as usize).saturating_sub(2);
        let truncated_msg = truncate_text_by_width(message, max_msg_width);
        let msg_width = truncated_msg.width() as u16;
        if msg_width > 0 && msg_width + 2 <= area.width {
            let msg_x = area.right() - msg_width - 2;
            frame.render_widget(
                Paragraph::new(Span::styled(truncated_msg, Style::default().fg(color::GREEN).add_modifier(Modifier::BOLD))),
                Rect::new(msg_x, area.top(), msg_width, 1),
            );
        }
    }
}