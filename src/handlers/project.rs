use crate::app::App;
use crate::popup::{popup_project_manager, popup_with_mode, PopupMode, ProjectAction};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn menu(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let projects = app.storage.list_projects();
    let current = app.storage.current_project.clone();
    let title = app.i18n.get("project_menu_title");
    let help_switch = app.i18n.get("project_menu_help_switch");
    let help_create = app.i18n.get("project_menu_help_create");
    let help_rename = app.i18n.get("project_menu_help_rename");
    let help_delete = app.i18n.get("project_menu_help_delete");
    let help_title = app.i18n.get("project_menu_help_title");
    let hint_c = app.i18n.get("project_menu_hint_c");
    let hint_r = app.i18n.get("project_menu_hint_r");
    let hint_d = app.i18n.get("project_menu_hint_d");
    let hint_enter = app.i18n.get("project_menu_hint_enter");
    match popup_project_manager(
        title, &projects, &current,
        help_switch, help_create, help_rename, help_delete,
        help_title,
        hint_c, hint_r, hint_d, hint_enter,
        term,
    ) {
        Ok(ProjectAction::Switch(name)) => {
            if name != app.storage.current_project {
                app.storage.set_project(&name);
                app.rebuild_visible();
                let msg = app.i18n.get("project_switched").replace("{}", &name);
                app.set_message(&msg);
            }
        }
        Ok(ProjectAction::Create) => create(app, term),
        Ok(ProjectAction::Rename(old)) => rename(app, term, &old),
        Ok(ProjectAction::Delete(proj)) => delete(app, term, &proj),
        _ => {}
    }
}

pub fn create(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    let projects = app.storage.list_projects();
    if projects.len() >= 64 {
        let msg = app.i18n.get("project_limit_reached").to_string();
        app.set_message(&msg);
        return;
    }
    let title = app.i18n.get("project_create_title");
    let hint = app.i18n.get("popup_esc_hint");
    if let Ok(Some(name)) = popup_with_mode(title, hint, "", PopupMode::Singleline, term) {
        if name.is_empty() || name.contains('.') || name.contains('/') || name.contains('\\') {
            let msg = app.i18n.get("project_invalid_name").to_string();
            app.set_message(&msg);
            return;
        }
        if app.storage.create_project(&name) {
            let msg = app.i18n.get("project_created").replace("{}", &name);
            app.set_message(&msg);
            switch(app, &name);
        } else {
            let msg = app.i18n.get("project_already_exists").to_string();
            app.set_message(&msg);
        }
    }
}

pub fn rename(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>, old: &str) {
    let title = app.i18n.get("project_rename_title");
    let hint = app.i18n.get("popup_esc_hint");
    if let Ok(Some(new)) = popup_with_mode(title, hint, old, PopupMode::Singleline, term) {
        if new.is_empty() || new.contains('.') || new.contains('/') || new.contains('\\') {
            let msg = app.i18n.get("project_invalid_name").to_string();
            app.set_message(&msg);
            return;
        }
        if app.storage.rename_project(old, &new) {
            let msg = app.i18n.get("project_renamed").replace("{}", old).replace("{}", &new);
            app.set_message(&msg);
        } else {
            let msg = app.i18n.get("rename_failed").to_string();
            app.set_message(&msg);
        }
    }
}

pub fn delete(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>, proj: &str) {
    if proj == "default" {
        return;
    }
    let template = app.i18n.get("popup_delete_project_confirm");
    let prompt = template.replace("{}", proj);
    let hint = app.i18n.get("popup_esc_hint");
    if let Ok(Some(ans)) = popup_with_mode(&prompt, hint, "", PopupMode::Singleline, term) {
        if ans == "y" || ans == "Y" {
            if app.storage.delete_project(proj) {
                let msg = app.i18n.get("project_deleted").replace("{}", proj);
                app.set_message(&msg);
                if app.storage.current_project == proj {
                    switch(app, "default");
                }
            } else {
                let msg = app.i18n.get("delete_failed").to_string();
                app.set_message(&msg);
            }
        }
    }
}

pub fn prev(app: &mut App) {
    let projects = app.storage.list_projects();
    if projects.len() <= 1 {
        let msg = app.i18n.get("only_one_project").to_string();
        app.set_message(&msg);
        return;
    }
    let current = app.storage.current_project.clone();
    let pos = projects.iter().position(|p| *p == current).unwrap_or(0);
    let new_pos = if pos == 0 { projects.len() - 1 } else { pos - 1 };
    let new_project = projects[new_pos].clone();
    switch(app, &new_project);
}

pub fn next(app: &mut App) {
    let projects = app.storage.list_projects();
    if projects.len() <= 1 {
        let msg = app.i18n.get("only_one_project").to_string();
        app.set_message(&msg);
        return;
    }
    let current = app.storage.current_project.clone();
    let pos = projects.iter().position(|p| *p == current).unwrap_or(0);
    let new_pos = (pos + 1) % projects.len();
    let new_project = projects[new_pos].clone();
    switch(app, &new_project);
}

fn switch(app: &mut App, name: &str) {
    if name != app.storage.current_project {
        app.storage.set_project(name);
        app.rebuild_visible();
        let msg = app.i18n.get("project_switched").replace("{}", name);
        app.set_message(&msg);
    }
}