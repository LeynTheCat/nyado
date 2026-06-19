use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub const CAT_ASCII_BIG: [&str; 16] = [
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

pub const CAT_BIG_HEIGHT: usize = CAT_ASCII_BIG.len();

pub const CAT_ASCII_SMALL: [&str; 11] = [
    "⠀⠀  ⠀⠀⣠⠀⠀⠀⠀⠀ ⣄⠀⠀⠀⠀ ⠀",
    "⠀⠀  ⢠⣾⣿⣇⠀⠀ ⠀⣼⣿⣷⡀⠀   ",
    " ⠀ ⢀⣿⣿⣿⣿⡄⠀ ⢰⣿⣿⣿⣷⡀⠀⠀ ",
    "   ⣾⣿⣿⣿⣿⣧  ⣾⣿⣿⣿⣿⣧⠀  ",
    "  ⢸⣿⣿⣿⣿⣿⣿⣄⣠⣿⣿⣿⣿⣿⣿⡆⠀ ",
    "  ⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣇⠀ ",
    "  ⣿⡟⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⢻⣿  ",
    "  ⣿⡇⣿⣿⡿⣿⣿⣿⣿⣿⣿⣿⣿⡟⢸⣿  ",
    "⠀ ⡸⢿⣮⣭⣴⣾⣿⣿⣿⣿⣷⣬⣥⣶⡿⢇⠀ ",
    " ⠈ ⠔⠙⠛⠛⠛⠛⠛⠛⠛⠛⠛⠛⠋⠢ ⠁⠀",
    "                    ",
];

pub const CAT_SMALL_HEIGHT: usize = CAT_ASCII_SMALL.len();

fn draw_bongo_generic(frame: &mut Frame, area: Rect, cat_lines: &[&str], cat_height: usize, centered: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let cat_width = cat_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(20) as u16;
    if area.width < cat_width || area.height < cat_height as u16 {
        return;
    }
    let x = if centered {
        area.left() + (area.width - cat_width) / 2
    } else {
        area.right().saturating_sub(cat_width + 1)
    };
    let mut y = area.top();
    for &line in cat_lines.iter() {
        let line_len = line.chars().count() as u16;
        if line_len == 0 || y >= area.bottom() - 1 {
            continue;
        }
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line,
                Style::default().fg(color::bongo()),
            ))),
            Rect::new(x, y, line_len, 1),
        );
        y += 1;
    }
}

pub fn draw_bongo_big(frame: &mut Frame, area: Rect) {
    draw_bongo_generic(frame, area, &CAT_ASCII_BIG, CAT_BIG_HEIGHT, false);
}

pub fn draw_bongo_small(frame: &mut Frame, area: Rect) {
    draw_bongo_generic(frame, area, &CAT_ASCII_SMALL, CAT_SMALL_HEIGHT, true);
}

pub mod color {
    use ratatui::style::Color;
    use std::sync::OnceLock;

    static TRUE_COLOR_SUPPORTED: OnceLock<bool> = OnceLock::new();
    pub fn init_color_mode() {
        let supported = std::env::var("COLORTERM")
            .map(|v| v == "truecolor" || v == "24bit")
            .unwrap_or(false)
            || std::env::var("TERM")
                .map(|v| v.contains("truecolor") || v.contains("24bit"))
                .unwrap_or(false);

        TRUE_COLOR_SUPPORTED.set(supported).ok();
    }

    fn true_color_supported() -> bool {
        *TRUE_COLOR_SUPPORTED.get().unwrap_or(&false)
    }

    fn choose_rgb(rgb: Color, ansi: Color) -> Color {
        if true_color_supported() {
            rgb
        } else {
            ansi
        }
    }

    pub fn cyan() -> Color {
        choose_rgb(Color::Rgb(0x94, 0xe2, 0xd5), Color::Cyan)
    }
    pub fn magenta() -> Color {
        choose_rgb(Color::Rgb(0xf5, 0xc2, 0xe7), Color::Magenta)
    }
    pub fn green() -> Color {
        choose_rgb(Color::Rgb(0xa6, 0xe3, 0xa1), Color::Green)
    }
    pub fn yellow() -> Color {
        choose_rgb(Color::Rgb(0xf9, 0xe2, 0xaf), Color::Yellow)
    }
    pub fn red() -> Color {
        choose_rgb(Color::Rgb(0xf3, 0x8b, 0xa8), Color::Red)
    }
    pub fn blue() -> Color {
        choose_rgb(Color::Rgb(0x89, 0xb4, 0xfa), Color::Blue)
    }
    pub fn black() -> Color {
        choose_rgb(Color::Rgb(0x45, 0x47, 0x5a), Color::Black)
    }
    pub fn white() -> Color {
        choose_rgb(Color::Rgb(0xba, 0xc2, 0xde), Color::White)
    }

    pub fn celebrate_color1() -> Color {
        cyan()
    }
    pub fn celebrate_color2() -> Color {
        magenta()
    }
    pub fn border() -> Color {
        cyan()
    }
    pub fn topbar_bg() -> Color {
        magenta()
    }
    pub fn topbar_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn selected_bg() -> Color {
        cyan()
    }
    pub fn selected_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn done() -> Color {
        white()
    }
    pub fn pending() -> Color {
        yellow()
    }
    pub fn bongo() -> Color {
        magenta()
    }
    pub fn statusbar_bg() -> Color {
        blue()
    }
    pub fn statusbar_fg() -> Color {
        black()
    }
    pub fn header() -> Color {
        cyan()
    }
    pub fn search() -> Color {
        yellow()
    }
    pub fn pin() -> Color {
        red()
    }
    pub fn stats_done() -> Color {
        green()
    }
    pub fn stats_pending() -> Color {
        yellow()
    }
    pub fn stats_project() -> Color {
        white()
    }
    pub fn stats_pinned() -> Color {
        magenta()
    }
    pub fn stats_overdue() -> Color {
        red()
    }
    pub fn tag1() -> Color {
        green()
    }
    pub fn tag2() -> Color {
        yellow()
    }
    pub fn tag3() -> Color {
        cyan()
    }
    pub fn tag4() -> Color {
        magenta()
    }
    pub fn tag5() -> Color {
        red()
    }
    pub fn tag6() -> Color {
        blue()
    }
    pub fn tag7() -> Color {
        white()
    }
    pub fn tag8() -> Color {
        choose_rgb(Color::Rgb(0xa6, 0xe3, 0xa1), Color::LightGreen)
    }
    pub fn tag9() -> Color {
        choose_rgb(Color::Rgb(0xf9, 0xe2, 0xaf), Color::LightYellow)
    }
    pub fn tag10() -> Color {
        choose_rgb(Color::Rgb(0x94, 0xe2, 0xd5), Color::LightCyan)
    }
    pub fn tag11() -> Color {
        choose_rgb(Color::Rgb(0xf5, 0xc2, 0xe7), Color::LightMagenta)
    }
    pub fn tag12() -> Color {
        choose_rgb(Color::Rgb(0xf3, 0x8b, 0xa8), Color::LightRed)
    }
    pub fn tag13() -> Color {
        blue()
    }

    pub fn calendar_today_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn calendar_today_bg() -> Color {
        green()
    }
    pub fn calendar_uncompleted_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn calendar_uncompleted_bg() -> Color {
        red()
    }
    pub fn calendar_completed_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn calendar_completed_bg() -> Color {
        magenta()
    }
    pub fn calendar_weekend_fg() -> Color {
        choose_rgb(Color::Rgb(0x58, 0x5b, 0x70), Color::DarkGray)
    }
    pub fn calendar_weekend_bg() -> Color {
        yellow()
    }
    pub fn calendar_normal_day() -> Color {
        choose_rgb(Color::Rgb(0x94, 0xe2, 0xd5), Color::LightCyan)
    }
    pub fn calendar_weekday_header() -> Color {
        choose_rgb(Color::Rgb(0x6c, 0x70, 0x86), Color::Gray)
    }

    pub fn due_overdue() -> Color {
        red()
    }
    pub fn due_future() -> Color {
        green()
    }
    pub fn pinned_marker() -> Color {
        magenta()
    }
    pub fn popup_border() -> Color {
        cyan()
    }
    pub fn popup_title() -> Color {
        cyan()
    }
    pub fn popup_input_fg() -> Color {
        yellow()
    }
    pub fn popup_selected_bg() -> Color {
        cyan()
    }
    pub fn popup_selected_fg() -> Color {
        choose_rgb(Color::Rgb(0x1e, 0x1e, 0x2e), Color::Black)
    }
    pub fn popup_current_project() -> Color {
        green()
    }
    pub fn popup_normal_project() -> Color {
        yellow()
    }
    pub fn popup_help_fg() -> Color {
        choose_rgb(Color::Rgb(0x58, 0x5b, 0x70), Color::DarkGray)
    }
}

pub fn tag_color(tag: &str) -> Color {
    let mut h = 0u64;
    for ch in tag.chars() {
        h = h.wrapping_mul(31).wrapping_add(ch as u64);
    }
    let colors: [fn() -> Color; 13] = [
        color::tag1, color::tag2, color::tag3, color::tag4, color::tag5, color::tag6,
        color::tag7, color::tag8, color::tag9, color::tag10, color::tag11, color::tag12,
        color::tag13,
    ];
    colors[(h % 13) as usize]()
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