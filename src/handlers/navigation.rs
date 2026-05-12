use crate::app::App;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn up(app: &mut App) {
    if app.selected > 0 {
        app.selected -= 1;
    }
}

pub fn down(app: &mut App) {
    if app.selected + 1 < app.visible.len() {
        app.selected += 1;
    }
}

pub fn top(app: &mut App) {
    app.selected = 0;
}

pub fn bottom(app: &mut App) {
    app.selected = app.visible.len().saturating_sub(1);
}

pub fn page_up(app: &mut App, term: &Terminal<CrosstermBackend<io::Stdout>>) {
    let step = (term.size().unwrap().height as usize).saturating_sub(5);
    app.selected = app.selected.saturating_sub(step);
}

pub fn page_down(app: &mut App, term: &Terminal<CrosstermBackend<io::Stdout>>) {
    let step = (term.size().unwrap().height as usize).saturating_sub(5);
    app.selected = (app.selected + step).min(app.visible.len().saturating_sub(1));
}