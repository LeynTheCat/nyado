use crate::app::App;
use std::path::PathBuf;

pub fn toggle(app: &mut App, data_dir: &PathBuf) {
    app.i18n.toggle_language();
    let lang_code = app.i18n.current_code();
    app.set_message(&format!("Language: {}", lang_code));
    app.save_current_language(data_dir);
}