use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

include!(concat!(env!("OUT_DIR"), "/builtin_langs.rs"));

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

#[derive(Debug, Deserialize, Clone)]
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
    popup_help_title: String,
    popup_help_hint: String,
    help_content: String,
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

impl Localization {
    fn fill_missing(&mut self, default: &Localization) {
        macro_rules! fill_field {
            ($field:ident) => {
                if self.$field.is_empty() {
                    self.$field = default.$field.clone();
                }
            };
        }
        fill_field!(created_prefix);
        fill_field!(done_prefix);
        fill_field!(pinned_marker);
        fill_field!(selected_header);
        fill_field!(tags_header);
        fill_field!(stats_header);
        fill_field!(mood_all_done);
        fill_field!(mood_empty);
        fill_field!(mood_one);
        fill_field!(mood_few);
        fill_field!(mood_several);
        fill_field!(mood_many);
        fill_field!(popup_new_title);
        fill_field!(popup_new_hint);
        fill_field!(popup_edit_title);
        fill_field!(popup_edit_hint);
        fill_field!(popup_set_tag_title);
        fill_field!(popup_set_tag_hint_existing);
        fill_field!(popup_set_tag_hint_empty);
        fill_field!(popup_delete_confirm);
        fill_field!(popup_delete_all_confirm);
        fill_field!(popup_delete_all_warning);
        fill_field!(popup_search_title);
        fill_field!(popup_search_hint);
        fill_field!(popup_help_title);
        fill_field!(popup_help_hint);
        fill_field!(help_content);
        fill_field!(statusbar_hint_wide);
        fill_field!(statusbar_hint_medium);
        fill_field!(statusbar_hint_narrow);
        fill_field!(progress_label);
        fill_field!(column_header);
        fill_field!(scroll_up);
        fill_field!(scroll_down);
        fill_field!(topbar_title);
        fill_field!(topbar_filter_prefix);
        fill_field!(topbar_search_prefix);
        fill_field!(topbar_date_format);
        fill_field!(title);
        fill_field!(right_title);
        fill_field!(celebration_line1);
        fill_field!(celebration_line2);
        fill_field!(celebration_line3);
        fill_field!(celebration_line4);
        fill_field!(celebration_line5);
        fill_field!(celebration_line6);
        fill_field!(empty_list_line1);
        fill_field!(empty_list_line2);
        fill_field!(popup_due_date_title);
        fill_field!(popup_due_date_hint);
        fill_field!(popup_due_time_hint);
        fill_field!(due_date_cleared);
        fill_field!(due_date_set);
        fill_field!(due_date_invalid);

        if self.ui.is_empty() {
            self.ui = default.ui.clone();
        }
        if self.pending.is_empty() {
            self.pending = default.pending.clone();
        }
        if self.done.is_empty() {
            self.done = default.done.clone();
        }
        if self.pinned.is_empty() {
            self.pinned = default.pinned.clone();
        }
        if self.total.is_empty() {
            self.total = default.total.clone();
        }
        if self.messages.is_empty() {
            self.messages = default.messages.clone();
        }
        if self.extra.is_empty() {
            self.extra = default.extra.clone();
        }
    }
}

pub struct I18n {
    languages: Vec<(String, Localization)>,
    current_index: usize,
    default_loc: Localization,
}

impl I18n {
    pub fn new() -> Result<Self> {
        let config_dir = get_config_dir();
        let mut builtin_langs: HashMap<String, Localization> = HashMap::new();
        for (code, content) in BUILTIN_LANGS {
            if let Ok(loc) = toml::from_str::<Localization>(content) {
                builtin_langs.insert(code.to_string(), loc);
            }
        }
        let mut all_langs = builtin_langs.clone();
        if config_dir.exists() && config_dir.is_dir() {
            for entry in fs::read_dir(&config_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.file_name().and_then(|n| n.to_str()).map_or(false, |name| name.starts_with("lang_") && name.ends_with(".toml")) {
                    let name = path.file_name().unwrap().to_str().unwrap();
                    let code = &name[5..name.len()-5];
                    let content = fs::read_to_string(&path)?;
                    if let Ok(loc) = toml::from_str::<Localization>(&content) {
                        all_langs.insert(code.to_string(), loc);
                    }
                }
            }
        }
        let en_loc = all_langs
            .get("en")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("English localization missing"))?;
        let mut languages = Vec::new();
        for (code, mut loc) in all_langs {
            if code != "en" {
                loc.fill_missing(&en_loc);
            }
            languages.push((code, loc));
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
        let default_loc = languages
            .iter()
            .find(|(code, _)| code == "en")
            .map(|(_, loc)| loc.clone())
            .unwrap_or_else(|| languages[0].1.clone());
        Ok(Self {
            languages,
            current_index: 0,
            default_loc,
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

    fn get_from_loc<'a>(&'a self, loc: &'a Localization, key: &str) -> Option<&'a str> {
        if let Some(v) = loc.ui.get(key) {
            return Some(v);
        }
        if let Some(stripped) = key.strip_prefix("pending.") {
            if let Some(v) = loc.pending.get(stripped) {
                return Some(v);
            }
        }
        if let Some(stripped) = key.strip_prefix("done.") {
            if let Some(v) = loc.done.get(stripped) {
                return Some(v);
            }
        }
        if let Some(stripped) = key.strip_prefix("pinned.") {
            if let Some(v) = loc.pinned.get(stripped) {
                return Some(v);
            }
        }
        if let Some(stripped) = key.strip_prefix("total.") {
            if let Some(v) = loc.total.get(stripped) {
                return Some(v);
            }
        }
        if let Some(stripped) = key.strip_prefix("messages.") {
            if let Some(v) = loc.messages.get(stripped) {
                return Some(v);
            }
        }
        if let Some(v) = loc.extra.get(key) {
            return Some(v);
        }
        match key {
            "created_prefix" => Some(&loc.created_prefix),
            "done_prefix" => Some(&loc.done_prefix),
            "pinned_marker" => Some(&loc.pinned_marker),
            "selected_header" => Some(&loc.selected_header),
            "tags_header" => Some(&loc.tags_header),
            "stats_header" => Some(&loc.stats_header),
            "mood_all_done" => Some(&loc.mood_all_done),
            "mood_empty" => Some(&loc.mood_empty),
            "mood_one" => Some(&loc.mood_one),
            "mood_few" => Some(&loc.mood_few),
            "mood_several" => Some(&loc.mood_several),
            "mood_many" => Some(&loc.mood_many),
            "popup_new_title" => Some(&loc.popup_new_title),
            "popup_new_hint" => Some(&loc.popup_new_hint),
            "popup_edit_title" => Some(&loc.popup_edit_title),
            "popup_edit_hint" => Some(&loc.popup_edit_hint),
            "popup_set_tag_title" => Some(&loc.popup_set_tag_title),
            "popup_set_tag_hint_existing" => Some(&loc.popup_set_tag_hint_existing),
            "popup_set_tag_hint_empty" => Some(&loc.popup_set_tag_hint_empty),
            "popup_delete_confirm" => Some(&loc.popup_delete_confirm),
            "popup_delete_all_confirm" => Some(&loc.popup_delete_all_confirm),
            "popup_delete_all_warning" => Some(&loc.popup_delete_all_warning),
            "popup_search_title" => Some(&loc.popup_search_title),
            "popup_search_hint" => Some(&loc.popup_search_hint),
            "popup_help_title" => Some(&loc.popup_help_title),
            "popup_help_hint" => Some(&loc.popup_help_hint),
            "help_content" => Some(&loc.help_content),
            "statusbar_hint_wide" => Some(&loc.statusbar_hint_wide),
            "statusbar_hint_medium" => Some(&loc.statusbar_hint_medium),
            "statusbar_hint_narrow" => Some(&loc.statusbar_hint_narrow),
            "progress_label" => Some(&loc.progress_label),
            "column_header" => Some(&loc.column_header),
            "scroll_up" => Some(&loc.scroll_up),
            "scroll_down" => Some(&loc.scroll_down),
            "topbar_title" => Some(&loc.topbar_title),
            "topbar_filter_prefix" => Some(&loc.topbar_filter_prefix),
            "topbar_search_prefix" => Some(&loc.topbar_search_prefix),
            "topbar_date_format" => Some(&loc.topbar_date_format),
            "title" => Some(&loc.title),
            "right_title" => Some(&loc.right_title),
            "celebration_line1" => Some(&loc.celebration_line1),
            "celebration_line2" => Some(&loc.celebration_line2),
            "celebration_line3" => Some(&loc.celebration_line3),
            "celebration_line4" => Some(&loc.celebration_line4),
            "celebration_line5" => Some(&loc.celebration_line5),
            "celebration_line6" => Some(&loc.celebration_line6),
            "empty_list_line1" => Some(&loc.empty_list_line1),
            "empty_list_line2" => Some(&loc.empty_list_line2),
            "popup_due_date_title" => Some(&loc.popup_due_date_title),
            "popup_due_date_hint" => Some(&loc.popup_due_date_hint),
            "popup_due_time_hint" => Some(&loc.popup_due_time_hint),
            "due_date_cleared" => Some(&loc.due_date_cleared),
            "due_date_set" => Some(&loc.due_date_set),
            "due_date_invalid" => Some(&loc.due_date_invalid),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> &str {
        let current_loc = &self.languages[self.current_index].1;
        if let Some(val) = self.get_from_loc(current_loc, key) {
            return val;
        }
        if let Some(val) = self.get_from_loc(&self.default_loc, key) {
            return val;
        }
        "???"
    }
}