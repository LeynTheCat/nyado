use crate::app::App;
use crate::popup::{popup_with_mode, PopupMode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn show(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let mut help_text = app.i18n.get("help_content").to_string();
    if !help_text.ends_with('\n') {
        help_text.push('\n');
    }
    let title = app.i18n.get("popup_help_title");
    let hint = app.i18n.get("popup_help_hint");
    let _ = popup_with_mode(title, hint, &help_text, PopupMode::Readonly, term);
}