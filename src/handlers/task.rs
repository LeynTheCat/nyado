use crate::app::App;
use crate::popup::{popup_with_mode, PopupMode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use crate::config::config;
use crate::popup::PopupReadonlyLayout;
use crate::popup::popup_with_mode_layout;

pub fn new(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if let Ok(Some(text)) = popup_with_mode(
        app.i18n.get("popup_new_title"),
        "",
        PopupMode::Multiline,
        term,
    ) {
        if app.storage.total_count() < config().max_todos {
            let tag = if app.storage.filter_tag.is_empty() {
                String::new()
            } else {
                app.storage.filter_tag.clone()
            };
            let _ = app.storage.add_task(&text, &tag);
            app.sort_and_rebuild();
            let msg = app.i18n.get("task_added").to_string();
            app.set_message(&msg);
        }
    }
}

pub fn new_subtask(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let parent_id = app.visible[app.selected].0;
        if let Ok(Some(text)) = popup_with_mode(
            app.i18n.get("popup_new_subtask_title"),
            "",
            PopupMode::Multiline,
            term,
        ) {
            if app.storage.total_count() < config().max_todos {
                let tag = if app.storage.filter_tag.is_empty() {
                    String::new()
                } else {
                    app.storage.filter_tag.clone()
                };
                let _ = app.storage.add_subtask(parent_id, &text, &tag);
                app.sort_and_rebuild();
            }
        }
    }
}

pub fn edit(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let todo = app.storage.get_todo(id).unwrap();
        let old_text = todo.text.clone();
        if let Ok(Some(new_text)) = popup_with_mode(
            app.i18n.get("popup_edit_title"),
            &old_text,
            PopupMode::Multiline,
            term,
        ) {
            if let Some(todo_mut) = app.storage.find_mut(id) {
                todo_mut.text = new_text;
                app.storage.save();
                app.sort_and_rebuild();
                let msg = app.i18n.get("task_updated").to_string();
                app.set_message(&msg);
            }
        }
    }
}

pub fn toggle_done(app: &mut App) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let _ = app.storage.toggle_task(id);
        app.sort_and_rebuild();
        app.check_all_done();
        let msg = if app.storage.get_todo(id).unwrap().done {
            app.i18n.get("done_msg").to_string()
        } else {
            app.i18n.get("undone_msg").to_string()
        };
        app.set_message(&msg);
    }
}

pub fn toggle_pin(app: &mut App) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let todo = app.storage.get_todo(id).unwrap();
        let msg_key = if todo.pinned { "unpinned_msg" } else { "pinned_msg" };
        if todo.pinned {
            let _ = app.storage.unpin_task(id);
        } else {
            let _ = app.storage.pin_task(id);
        }
        let msg = app.i18n.get(msg_key).to_string();
        app.set_message(&msg);
        app.sort_and_rebuild();
    }
}

pub fn toggle_expand(app: &mut App) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let todo = app.storage.get_todo(id).unwrap();
        if !todo.children.is_empty() {
            app.storage.toggle_expand(id);
            app.rebuild_visible();
        }
    }
}

pub fn set_tag(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let hint_top = if !app.storage.tags_available.is_empty() {
            let existing: Vec<String> = app.storage.tags_available.iter().take(6).map(|(t, _)| format!("#{}", t)).collect();
            format!("{}{}", app.i18n.get("popup_set_tag_hint_existing"), existing.join(" "))
        } else {
            app.i18n.get("popup_set_tag_hint_empty").to_string()
        };
        let placeholder = app.i18n.get("popup_set_tag_hint_empty");
        if let Ok(Some(tag_raw)) = popup_with_mode_layout(
            app.i18n.get("popup_set_tag_title"),
            "",
            PopupMode::Singleline,
            term,
            PopupReadonlyLayout::SingleColumn,
            Some(&hint_top),
            Some(placeholder),
        ) {
            let cleaned: String = tag_raw.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).take(32).collect();
            let msg = if cleaned.is_empty() {
                let _ = app.storage.set_tag(id, "");
                app.i18n.get("tag_cleared").to_string()
            } else {
                let _ = app.storage.set_tag(id, &cleaned);
                app.i18n.get("tag_set").to_string()
            };
            app.set_message(&msg);
            app.storage.dirty_tags = true;
            app.storage.rebuild_tags();
            app.sort_and_rebuild();
        }
    }
}

pub fn delete(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let (id, _depth) = app.visible[app.selected];
        let todo = app.storage.get_todo(id).unwrap();
        let mut text = todo.text.clone();
        const MAX_DISPLAY_LEN: usize = 50;
        if text.chars().count() > MAX_DISPLAY_LEN {
            text = text.chars().take(MAX_DISPLAY_LEN).collect::<String>() + "…";
        }
        let template = app.i18n.get("popup_delete_confirm");
        let prompt = template.replace("{}", &text);
        if let Ok(Some(ans)) = popup_with_mode_layout(
            &prompt,
            "",
            PopupMode::Singleline,
            term,
            PopupReadonlyLayout::SingleColumn,
            None,
            None,
        ) {
            if ans == "y" || ans == "Y" {
                let _ = app.storage.remove_task(id);
                if app.selected >= app.visible.len().saturating_sub(1) && app.selected > 0 {
                    app.selected -= 1;
                }
                app.sort_and_rebuild();
                let msg = app.i18n.get("deleted").to_string();
                app.set_message(&msg);
            }
        }
    }
}

pub fn delete_all(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.storage.todos.is_empty() {
        let template = app.i18n.get("popup_delete_all_confirm");
        let prompt = template.replace("{}", &app.storage.total_count().to_string());
        let warning = app.i18n.get("popup_delete_all_warning");
        if let Ok(Some(ans)) = popup_with_mode_layout(
            &prompt,
            "",
            PopupMode::Singleline,
            term,
            PopupReadonlyLayout::SingleColumn,
            Some(warning),
            None,
        ) {
            if ans == "y" || ans == "Y" {
                app.storage.todos.clear();
                app.storage.expanded.clear();
                app.selected = 0;
                app.sort_and_rebuild();
                app.storage.save();
                let msg = app.i18n.get("all_deleted").to_string();
                app.set_message(&msg);
            }
        }
    }
}