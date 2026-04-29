use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub const CAT_ASCII: [&str; 16] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣇⠀⠀⠀⠀⠀⠀⠀⠀⣼⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⣄⠀⠀⠀⠀⠀⠀⣰⣿⣿⣿⣆⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⡀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣇⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⡆⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⣿⣿⣿⣿⣿⣿⣿⣿⡀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⠀⢰⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀",
    "⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀",
    "⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀",
    "⠀⠀⠀⣿⣿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⣿⣿⣿⠀⠀⠀",
    "⠀⠀⠀⣿⡟⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀⣿⣿⠀⠀⠀",
    "⠀⠀⠀⢿⣇⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⣿⣿⠀⠀⠀",
    "⠀⠀⠀⠘⣿⣄⡙⠟⠿⠋⢀⣿⣿⣿⣿⣿⣿⣿⣿⡘⠛⠋⠋⣀⣾⡿⠃⠀⠀⠀",
    "⠀⠀⠀⠀⠘⢻⣿⣷⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠁⠀⠀⠀⠀",
    "⠀⠀⠀ ⠔⠁⡠⠋⠛⡿⠿⠿⠿⠿⠿⠿⠿⠿⠿⠿⢿⠋⠙⢄⠈⠂⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠈  ⠀⠀⠀⠀⠀⠀⠀⠀⠀ ⠁ ⠀⠀⠀⠀⠀⠀⠀",
];

pub const CAT_HEIGHT: usize = CAT_ASCII.len();

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

pub mod color {
    use ratatui::style::Color;
    pub const BORDER: Color = Color::Cyan;
    pub const TOPBAR_BG: Color = Color::Magenta;
    pub const TOPBAR_FG: Color = Color::Black;
    pub const SELECTED_BG: Color = Color::Cyan;
    pub const SELECTED_FG: Color = Color::Black;
    pub const DONE: Color = Color::White;
    pub const PENDING: Color = Color::Yellow;
    pub const BONGO: Color = Color::Magenta;
    pub const STATUSBAR_BG: Color = Color::Blue;
    pub const STATUSBAR_FG: Color = Color::Black;
    pub const HEADER: Color = Color::Cyan;
    pub const GREEN: Color = Color::Green;
    pub const PIN: Color = Color::Red;
    pub const TAG1: Color = Color::Green;
    pub const TAG2: Color = Color::Yellow;
    pub const TAG3: Color = Color::Cyan;
    pub const TAG4: Color = Color::Magenta;
    pub const TAG5: Color = Color::Red;
    pub const TAG6: Color = Color::Blue;
    pub const TAG7: Color = Color::White;
    pub const TAG8: Color = Color::Green;
    pub const SEARCH: Color = Color::Yellow;
}

pub fn tag_color(tag: &str) -> Color {
    let mut h = 0u64;
    for ch in tag.chars() {
        h = h.wrapping_mul(31).wrapping_add(ch as u64);
    }
    let colors = [
        color::TAG1, color::TAG2, color::TAG3, color::TAG4,
        color::TAG5, color::TAG6, color::TAG7, color::TAG8,
    ];
    colors[(h % 8) as usize]
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut chars = text.chars();
    let mut result = String::with_capacity(max_chars);
    let mut count = 0;
    for ch in chars.by_ref() {
        if count + 1 > max_chars {
            result.push('…');
            return result;
        }
        result.push(ch);
        count += 1;
    }
    result
}

pub fn visual_width(s: &str) -> usize {
    s.width()
}

pub fn truncate_text_by_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut current_width = 0;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(1);
        if current_width + ch_width > max_width {
            if current_width + 1 <= max_width {
                result.push('…');
            }
            return result;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result
}