use crate::app::App;

pub fn start_search(app: &mut App) {
    app.search_mode = true;
    app.search_buffer.clear();
    app.rebuild_visible();
}

pub fn clear(app: &mut App) {
    app.storage.search.clear();
    app.storage.filter_tag.clear();
    app.search_mode = false;
    app.search_buffer.clear();
    app.selected = 0;
    app.rebuild_visible();
    let msg = app.i18n.get("filters_cleared").to_string();
    app.set_message(&msg);
}

pub fn by_tag(app: &mut App, idx: usize) {
    if idx < app.storage.tags_available.len() {
        let tag = app.storage.tags_available[idx].0.clone();
        if app.storage.filter_tag == tag {
            app.storage.filter_tag.clear();
        } else {
            app.storage.filter_tag = tag;
        }
        app.selected = 0;
        app.rebuild_visible();
    }
}