use crate::app::App;
use crate::popup::{popup_with_mode, PopupMode};
use crate::todo::Todo;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const MAX_TODOS: usize = 2048;

pub fn new(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if let Ok(Some(text)) = popup_with_mode(
        app.i18n.get("popup_new_title"),
        app.i18n.get("popup_new_hint"),
        "",
        PopupMode::Multiline,
        term,
    ) {
        if app.storage.todos.len() < MAX_TODOS {
            let tag = if app.storage.filter_tag.is_empty() {
                String::new()
            } else {
                app.storage.filter_tag.clone()
            };
            app.storage.todos.push(Todo::new(&text, &tag));
            app.sort_todos();
            app.storage.save();
            let msg = app.i18n.get("messages.task_added").to_string();
            app.set_message(&msg);
        }
    }
}

pub fn edit(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let idx = app.visible[app.selected];
        let old_text = app.storage.todos[idx].text.clone();
        if let Ok(Some(new_text)) = popup_with_mode(
            app.i18n.get("popup_edit_title"),
            app.i18n.get("popup_edit_hint"),
            &old_text,
            PopupMode::Multiline,
            term,
        ) {
            app.storage.todos[idx].text = new_text;
            app.storage.save();
            app.rebuild_visible();
            let msg = app.i18n.get("messages.task_updated").to_string();
            app.set_message(&msg);
        }
    }
}

pub fn toggle_done(app: &mut App) {
    if !app.visible.is_empty() {
        let idx = app.visible[app.selected];
        let done = !app.storage.todos[idx].done;
        let done_at = if done { crate::todo::now_secs() } else { 0 };
        app.storage.todos[idx].done = done;
        app.storage.todos[idx].done_at = done_at;
        app.sort_todos();
        app.storage.save();
        app.check_all_done();
        let msg = if done {
            app.i18n.get("messages.done").to_string()
        } else {
            app.i18n.get("messages.undone").to_string()
        };
        app.set_message(&msg);
    }
}

pub fn toggle_pin(app: &mut App) {
    if !app.visible.is_empty() {
        let idx = app.visible[app.selected];
        app.storage.todos[idx].pinned = !app.storage.todos[idx].pinned;
        app.sort_todos();
        app.storage.save();
        let msg = if app.storage.todos[idx].pinned {
            app.i18n.get("messages.pinned").to_string()
        } else {
            app.i18n.get("messages.unpinned").to_string()
        };
        app.set_message(&msg);
    }
}

pub fn set_tag(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let hint = if !app.storage.tags_available.is_empty() {
            let existing: Vec<String> = app.storage.tags_available.iter().take(6).map(|(t, _)| format!("#{}", t)).collect();
            format!("{}{}", app.i18n.get("popup_set_tag_hint_existing"), existing.join(" "))
        } else {
            app.i18n.get("popup_set_tag_hint_empty").to_string()
        };
        if let Ok(Some(tag_raw)) = popup_with_mode(
            app.i18n.get("popup_set_tag_title"),
            &hint,
            "",
            PopupMode::Singleline,
            term,
        ) {
            let cleaned: String = tag_raw.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).take(32).collect();
            let idx = app.visible[app.selected];
            let msg = if cleaned.is_empty() {
                app.storage.todos[idx].tag.clear();
                app.i18n.get("messages.tag_cleared").to_string()
            } else {
                app.storage.todos[idx].tag = cleaned;
                app.i18n.get("messages.tag_set").to_string()
            };
            app.set_message(&msg);
            app.storage.dirty_tags = true;
            app.storage.rebuild_tags();
            app.sort_todos();
            app.storage.save();
        }
    }
}

pub fn delete(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let idx = app.visible[app.selected];
        let text = &app.storage.todos[idx].text;
        let template = app.i18n.get("popup_delete_confirm");
        let prompt = template.replace("{}", text);
        if let Ok(Some(ans)) = popup_with_mode(&prompt, "", "", PopupMode::Singleline, term) {
            if ans == "y" || ans == "Y" {
                app.storage.todos.remove(idx);
                if app.selected >= app.visible.len().saturating_sub(1) && app.selected > 0 {
                    app.selected -= 1;
                }
                app.sort_todos();
                app.storage.save();
                let msg = app.i18n.get("messages.deleted").to_string();
                app.set_message(&msg);
            }
        }
    }
}

pub fn delete_all(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.storage.todos.is_empty() {
        let template = app.i18n.get("popup_delete_all_confirm");
        let prompt = template.replace("{}", &app.storage.todos.len().to_string());
        if let Ok(Some(ans)) = popup_with_mode(&prompt, app.i18n.get("popup_delete_all_warning"), "", PopupMode::Singleline, term) {
            if ans == "y" || ans == "Y" {
                app.storage.todos.clear();
                app.selected = 0;
                app.sort_todos();
                app.storage.save();
                let msg = app.i18n.get("messages.all_deleted").to_string();
                app.set_message(&msg);
            }
        }
    }
}