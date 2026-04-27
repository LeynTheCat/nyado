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
    empty_list_line1: String,
    empty_list_line2: String,
    popup_due_date_title: String,
    popup_due_date_hint: String,
    popup_due_time_hint: String,
    due_date_cleared: String,
    due_date_set: String,
    due_date_invalid: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
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

    fn get_en_loc(&self) -> Option<&Localization> {
        for (code, loc) in &self.languages {
            if code == "en" {
                return Some(loc);
            }
        }
        None
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        let loc = &self.languages[self.current_index].1;
        if let Some(v) = loc.ui.get(key) {
            return v;
        }
        if let Some(stripped) = key.strip_prefix("pending.") {
            if let Some(v) = loc.pending.get(stripped) {
                return v;
            }
        }
        if let Some(stripped) = key.strip_prefix("done.") {
            if let Some(v) = loc.done.get(stripped) {
                return v;
            }
        }
        if let Some(stripped) = key.strip_prefix("pinned.") {
            if let Some(v) = loc.pinned.get(stripped) {
                return v;
            }
        }
        if let Some(stripped) = key.strip_prefix("total.") {
            if let Some(v) = loc.total.get(stripped) {
                return v;
            }
        }
        if let Some(stripped) = key.strip_prefix("messages.") {
            if let Some(v) = loc.messages.get(stripped) {
                return v;
            }
        }
        if let Some(v) = loc.extra.get(key) {
            return v;
        }
        let val = match key {
            "created_prefix" => &loc.created_prefix,
            "done_prefix" => &loc.done_prefix,
            "pinned_marker" => &loc.pinned_marker,
            "selected_header" => &loc.selected_header,
            "tags_header" => &loc.tags_header,
            "stats_header" => &loc.stats_header,
            "mood_all_done" => &loc.mood_all_done,
            "mood_empty" => &loc.mood_empty,
            "mood_one" => &loc.mood_one,
            "mood_few" => &loc.mood_few,
            "mood_several" => &loc.mood_several,
            "mood_many" => &loc.mood_many,
            "popup_new_title" => &loc.popup_new_title,
            "popup_new_hint" => &loc.popup_new_hint,
            "popup_edit_title" => &loc.popup_edit_title,
            "popup_edit_hint" => &loc.popup_edit_hint,
            "popup_set_tag_title" => &loc.popup_set_tag_title,
            "popup_set_tag_hint_existing" => &loc.popup_set_tag_hint_existing,
            "popup_set_tag_hint_empty" => &loc.popup_set_tag_hint_empty,
            "popup_delete_confirm" => &loc.popup_delete_confirm,
            "popup_delete_all_confirm" => &loc.popup_delete_all_confirm,
            "popup_delete_all_warning" => &loc.popup_delete_all_warning,
            "popup_search_title" => &loc.popup_search_title,
            "popup_search_hint" => &loc.popup_search_hint,
            "statusbar_hint_wide" => &loc.statusbar_hint_wide,
            "statusbar_hint_medium" => &loc.statusbar_hint_medium,
            "statusbar_hint_narrow" => &loc.statusbar_hint_narrow,
            "progress_label" => &loc.progress_label,
            "column_header" => &loc.column_header,
            "scroll_up" => &loc.scroll_up,
            "scroll_down" => &loc.scroll_down,
            "topbar_title" => &loc.topbar_title,
            "topbar_filter_prefix" => &loc.topbar_filter_prefix,
            "topbar_search_prefix" => &loc.topbar_search_prefix,
            "topbar_date_format" => &loc.topbar_date_format,
            "title" => &loc.title,
            "right_title" => &loc.right_title,
            "celebration_line1" => &loc.celebration_line1,
            "celebration_line2" => &loc.celebration_line2,
            "celebration_line3" => &loc.celebration_line3,
            "celebration_line4" => &loc.celebration_line4,
            "celebration_line5" => &loc.celebration_line5,
            "celebration_line6" => &loc.celebration_line6,
            "empty_list_line1" => &loc.empty_list_line1,
            "empty_list_line2" => &loc.empty_list_line2,
            "popup_due_date_title" => &loc.popup_due_date_title,
            "popup_due_date_hint" => &loc.popup_due_date_hint,
            "popup_due_time_hint" => &loc.popup_due_time_hint,
            "due_date_cleared" => &loc.due_date_cleared,
            "due_date_set" => &loc.due_date_set,
            "due_date_invalid" => &loc.due_date_invalid,
            _ => {
                if let Some(en_loc) = self.get_en_loc() {
                    if let Some(v) = en_loc.extra.get(key) {
                        return v;
                    }
                    match key {
                        _ => key,
                    }
                } else {
                    key
                }
            }
        };
        val
    }
}