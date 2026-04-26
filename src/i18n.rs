use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn get_config_dir() -> PathBuf {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("nyado");
        if dir.exists() && dir.is_dir() {
            return dir;
        }
    }
    if Path::new("config").exists() && Path::new("config").is_dir() {
        return PathBuf::from("config");
    }
    let mut fallback = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    fallback.push("nyado");
    fallback
}

#[derive(Debug, Deserialize)]
struct Localization {
    ui: HashMap<String, String>,
    pending: HashMap<String, String>,
    done: HashMap<String, String>,
    pinned: HashMap<String, String>,
    total: HashMap<String, String>,
    messages: HashMap<String, String>,
    created_prefix: String,
    done_prefix: String,
    pinned_marker: String,
    selected_header: String,
    tags_header: String,
    stats_header: String,
    mood_all_done: String,
    mood_empty: String,
    mood_one: String,
    mood_few: String,
    mood_several: String,
    mood_many: String,
    popup_new_title: String,
    popup_new_hint: String,
    popup_edit_title: String,
    popup_edit_hint: String,
    popup_set_tag_title: String,
    popup_set_tag_hint_existing: String,
    popup_set_tag_hint_empty: String,
    popup_delete_confirm: String,
    popup_delete_all_confirm: String,
    popup_delete_all_warning: String,
    popup_search_title: String,
    popup_search_hint: String,
    statusbar_hint_wide: String,
    statusbar_hint_medium: String,
    statusbar_hint_narrow: String,
    progress_label: String,
    column_header: String,
    scroll_up: String,
    scroll_down: String,
    topbar_title: String,
    topbar_filter_prefix: String,
    topbar_search_prefix: String,
    topbar_date_format: String,
    title: String,
    right_title: String,
    celebration_line1: String,
    celebration_line2: String,
    celebration_line3: String,
    celebration_line4: String,
    celebration_line5: String,
    celebration_line6: String,
}

pub struct I18n {
    languages: Vec<(String, Localization)>,
    current_index: usize,
}

impl I18n {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = get_config_dir();
        let mut languages = Vec::new();
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.file_name().and_then(|n| n.to_str()).map_or(false, |name| name.starts_with("lang_") && name.ends_with(".toml")) {
                let name = path.file_name().unwrap().to_str().unwrap();
                let code = &name[5..name.len()-5];
                let content = fs::read_to_string(&path)?;
                let loc: Localization = toml::from_str(&content)?;
                languages.push((code.to_string(), loc));
            }
        }
        if languages.is_empty() {
            anyhow::bail!("No language files found in config/ directory (searched in {:?})", config_dir);
        }
        languages.sort_by(|(code_a, _), (code_b, _)| {
            if code_a == "en" {
                std::cmp::Ordering::Less
            } else if code_b == "en" {
                std::cmp::Ordering::Greater
            } else {
                code_a.cmp(code_b)
            }
        });
        Ok(Self {
            languages,
            current_index: 0,
        })
    }

    pub fn current_code(&self) -> &str {
        &self.languages[self.current_index].0
    }

    pub fn set_language_by_code(&mut self, code: &str) -> bool {
        for (i, (c, _)) in self.languages.iter().enumerate() {
            if c == code {
                self.current_index = i;
                return true;
            }
        }
        false
    }

    pub fn toggle_language(&mut self) {
        self.current_index = (self.current_index + 1) % self.languages.len();
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        let loc = &self.languages[self.current_index].1;
        if let Some(v) = loc.ui.get(key) {
            return v;
        }
        match key {
            "pending.prefix" => return &loc.pending["prefix"],
            "done.prefix" => return &loc.done["prefix"],
            "pinned.prefix" => return &loc.pinned["prefix"],
            "total.prefix" => return &loc.total["prefix"],
            "created_prefix" => return &loc.created_prefix,
            "done_prefix" => return &loc.done_prefix,
            "pinned_marker" => return &loc.pinned_marker,
            "selected_header" => return &loc.selected_header,
            "tags_header" => return &loc.tags_header,
            "stats_header" => return &loc.stats_header,
            "mood_all_done" => return &loc.mood_all_done,
            "mood_empty" => return &loc.mood_empty,
            "mood_one" => return &loc.mood_one,
            "mood_few" => return &loc.mood_few,
            "mood_several" => return &loc.mood_several,
            "mood_many" => return &loc.mood_many,
            "popup_new_title" => return &loc.popup_new_title,
            "popup_new_hint" => return &loc.popup_new_hint,
            "popup_edit_title" => return &loc.popup_edit_title,
            "popup_edit_hint" => return &loc.popup_edit_hint,
            "popup_set_tag_title" => return &loc.popup_set_tag_title,
            "popup_set_tag_hint_existing" => return &loc.popup_set_tag_hint_existing,
            "popup_set_tag_hint_empty" => return &loc.popup_set_tag_hint_empty,
            "popup_delete_confirm" => return &loc.popup_delete_confirm,
            "popup_delete_all_confirm" => return &loc.popup_delete_all_confirm,
            "popup_delete_all_warning" => return &loc.popup_delete_all_warning,
            "popup_search_title" => return &loc.popup_search_title,
            "popup_search_hint" => return &loc.popup_search_hint,
            "statusbar_hint_wide" => return &loc.statusbar_hint_wide,
            "statusbar_hint_medium" => return &loc.statusbar_hint_medium,
            "statusbar_hint_narrow" => return &loc.statusbar_hint_narrow,
            "progress_label" => return &loc.progress_label,
            "column_header" => return &loc.column_header,
            "scroll_up" => return &loc.scroll_up,
            "scroll_down" => return &loc.scroll_down,
            "topbar_title" => return &loc.topbar_title,
            "topbar_filter_prefix" => return &loc.topbar_filter_prefix,
            "topbar_search_prefix" => return &loc.topbar_search_prefix,
            "topbar_date_format" => return &loc.topbar_date_format,
            "title" => return &loc.title,
            "right_title" => return &loc.right_title,
            "celebration_line1" => return &loc.celebration_line1,
            "celebration_line2" => return &loc.celebration_line2,
            "celebration_line3" => return &loc.celebration_line3,
            "celebration_line4" => return &loc.celebration_line4,
            "celebration_line5" => return &loc.celebration_line5,
            "celebration_line6" => return &loc.celebration_line6,
            _ => {}
        }
        if let Some(stripped) = key.strip_prefix("messages.") {
            if let Some(v) = loc.messages.get(stripped) {
                return v;
            }
        }
        key
    }
}