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

pub fn draw_topbar(frame: &mut Frame, area: Rect, storage: &Storage, i18n: &I18n) {
    let style = Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG);
    let block = Block::default().style(style);
    frame.render_widget(block, area);

    let mut spans = vec![Span::styled(i18n.get("topbar_title"), Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG).add_modifier(Modifier::BOLD))];
    if !storage.filter_tag.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("{}{} ", i18n.get("topbar_filter_prefix"), storage.filter_tag),
            Style::default().fg(color::TAG1).bg(color::TOPBAR_BG).add_modifier(Modifier::BOLD),
        ));
    }
    if !storage.search.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("{}{}/ ", i18n.get("topbar_search_prefix"), storage.search),
            Style::default().fg(color::SEARCH).bg(color::TOPBAR_BG).add_modifier(Modifier::BOLD),
        ));
    }
    let width = spans.iter().map(|s| s.content.len()).sum::<usize>() as u16;
    let now = chrono::Local::now();
    let time_str = now.format(&i18n.get("topbar_date_format")).to_string();
    let time_span = Span::styled(time_str, Style::default().bg(color::TOPBAR_BG).fg(color::TOPBAR_FG).add_modifier(Modifier::BOLD));
    let time_width = time_span.content.len() as u16;
    if width + time_width + 2 < area.width {
        // not used
    }
    let line = Line::from(spans);
    let line_width = line.width() as u16;
    if line_width <= area.width && area.top() < area.bottom() {
        frame.render_widget(Paragraph::new(line), Rect::new(area.left() + 2, area.top(), line_width, 1));
    }
    let time_x = area.right() - time_width - 2;
    if time_width > 0 && time_x >= area.left() && area.top() < area.bottom() {
        frame.render_widget(Paragraph::new(time_span), Rect::new(time_x, area.top(), time_width, 1));
    }
}