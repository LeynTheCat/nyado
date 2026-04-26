use super::common::{CAT_ASCII, color};
use crate::i18n::I18n;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_bongo(frame: &mut Frame, area: Rect, pending: usize, total: usize, i18n: &I18n) {
    if area.width < 40 || area.height < 20 {
        return;
    }
    let cat_width = CAT_ASCII.iter().map(|line| line.chars().count()).max().unwrap_or(20) as u16;
    if area.width <= cat_width {
        return;
    }
    let x = area.right().saturating_sub(cat_width + 1);
    let mut y = area.top();
    for line in CAT_ASCII.iter() {
        let line_len = line.chars().count() as u16;
        if line_len == 0 || y >= area.bottom() - 1 {
            continue;
        }
        if line_len <= area.width {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    *line,
                    Style::default().fg(color::BONGO),
                ))),
                Rect::new(x, y, line_len, 1),
            );
        }
        y += 1;
    }
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
    } else {
        i18n.get("mood_many")
    };
    let mood_span = Span::styled(mood, Style::default().fg(color::BONGO).bg(Color::Reset).add_modifier(Modifier::BOLD));
    let mood_width = mood.chars().count() as u16;
    if mood_width + 2 <= area.width {
        frame.render_widget(
            Paragraph::new(mood_span).alignment(Alignment::Right),
            Rect::new(area.left(), area.bottom() - 1, area.width, 1),
        );
    }
}