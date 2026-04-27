use super::common::{CAT_ASCII, CAT_HEIGHT, color};
use ratatui::{
    layout::Rect,
    style::{Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_bongo(frame: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let cat_width = CAT_ASCII.iter().map(|line| line.chars().count()).max().unwrap_or(20) as u16;
    if area.width < cat_width || area.height < CAT_HEIGHT as u16 {
        return;
    }
    let x = area.right().saturating_sub(cat_width + 1);
    let mut y = area.top();
    for line in CAT_ASCII.iter() {
        let line_len = line.chars().count() as u16;
        if line_len == 0 || y >= area.bottom() - 1 {
            continue;
        }
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                *line,
                Style::default().fg(color::BONGO),
            ))),
            Rect::new(x, y, line_len, 1),
        );
        y += 1;
    }
}