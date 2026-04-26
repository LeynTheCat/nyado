use super::common::{color, truncate_text};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

pub fn draw_progress_bar(frame: &mut Frame, area: Rect, done: usize, total: usize) {
    if total == 0 || area.width == 0 {
        return;
    }
    let width = area.width as usize;
    if width < 10 {
        return;
    }
    let percent = (done * 100) / total;
    let bar_len = width.saturating_sub(12);
    if bar_len < 5 {
        return;
    }
    let filled = (bar_len * done) / total;
    let bar: String = (0..bar_len)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect();
    let label = format!("{}% {}/{}", percent, done, total);
    let line = format!("[{}] {}", bar, label);
    let display = truncate_text(&line, width);
    let line_len = display.chars().count() as u16;
    if line_len <= area.width {
        frame.render_widget(
            Paragraph::new(Span::styled(display, Style::default().fg(color::GREEN).add_modifier(Modifier::BOLD))),
            Rect::new(area.left(), area.top(), line_len, 1),
        );
    }
}