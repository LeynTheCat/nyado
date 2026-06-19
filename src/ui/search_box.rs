use crate::ui::common::color;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw_searchbox(frame: &mut Frame, area: Rect, search_buffer: &str) {
    if area.width < 4 || area.height < 5 {
        return;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color::border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let text = format!("   {}", search_buffer);
    let text_width = text.width();
    if text_width + 4 > inner.width as usize {
        return;
    }
    let inner_height = inner.height;
    if inner_height < 3 {
        return;
    }
    let padding_top = (inner_height - 1) / 2;
    let padding_bottom = inner_height - 1 - padding_top;
    let mut padded_text = String::new();
    for _ in 0..padding_top {
        padded_text.push('\n');
    }
    padded_text.push_str(&text);
    for _ in 0..padding_bottom {
        padded_text.push('\n');
    }
    let para = Paragraph::new(padded_text).alignment(Alignment::Left);
    frame.render_widget(para, inner);
}