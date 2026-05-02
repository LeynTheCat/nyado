use super::common::{color, truncate_text};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};
use std::sync::Mutex;

struct ProgressState {
    animated_filled: usize,
    last_done: usize,
    last_total: usize,
    last_bar_len: usize,
}

static PROGRESS_STATE: Mutex<Option<ProgressState>> = Mutex::new(None);

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
    let target_filled = (bar_len * done) / total;

    let mut state_guard = PROGRESS_STATE.lock().unwrap();
    let state = state_guard.get_or_insert_with(|| ProgressState {
        animated_filled: target_filled,
        last_done: done,
        last_total: total,
        last_bar_len: bar_len,
    });

    let need_animation = (done != state.last_done) || (total != state.last_total);
    if need_animation {
        state.last_done = done;
        state.last_total = total;
        state.last_bar_len = bar_len;
        if state.animated_filled < target_filled {
            state.animated_filled = (state.animated_filled + 1).min(target_filled);
        } else if state.animated_filled > target_filled {
            state.animated_filled = state.animated_filled.saturating_sub(1).max(target_filled);
        }
    } else {
        if bar_len != state.last_bar_len {
            state.last_bar_len = bar_len;
        }
        state.animated_filled = target_filled;
    }

    let bar: String = (0..bar_len)
        .map(|i| if i < state.animated_filled { '█' } else { '░' })
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