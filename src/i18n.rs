use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use anyhow::Result;
use crate::storage::get_config_dir;

include!(concat!(env!("OUT_DIR"), "/builtin_langs.rs"));

#[derive(Debug, Deserialize, Clone)]
struct Localization {
    celebration_line1: String,
    celebration_line2: String,
    celebration_line3: String,
    celebration_line4: String,
    celebration_line5: String,
    celebration_line6: String,
    created_prefix: String,
    done_prefix: String,
    pinned_marker: String,
    selected_header: String,
    tags_header: String,
    stats_header: String,
    help_content: String,
    popup_new_subtask_title: String,
    popup_new_subtask_hint: String,
    popup_help_title: String,
    popup_help_hint: String,
    mood_all_done: String,
    mood_empty: String,
    mood_one: String,
    mood_few: String,
    mood_several: String,
    mood_lots: String,
    mood_heap: String,
    mood_pile: String,
    mood_overwhelming: String,
    mood_hectic: String,
    mood_crazy: String,
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
    popup_due_date_title: String,
    popup_due_date_hint: String,
    popup_due_time_hint: String,
    due_date_cleared: String,
    due_date_set: String,
    due_date_invalid: String,
    statusbar_hint_wide: String,
    statusbar_hint_medium: String,
    statusbar_hint_narrow: String,
    column_header: String,
    scroll_up: String,
    scroll_down: String,
    topbar_title: String,
    topbar_date_format: String,
    title: String,
    right_title: String,
    empty_list_line1: String,
    empty_list_line2: String,
    calendar_title: String,
    calendar_mon: String,
    calendar_tue: String,
    calendar_wed: String,
    calendar_thu: String,
    calendar_fri: String,
    calendar_sat: String,
    calendar_sun: String,
    project_menu_title: String,
    project_menu_help_title: String,
    project_menu_help_switch: String,
    project_menu_help_create: String,
    project_menu_help_rename: String,
    project_menu_help_delete: String,
    project_menu_hint_c: String,
    project_menu_hint_r: String,
    project_menu_hint_d: String,
    project_menu_hint_enter: String,
    only_one_project: String,
    project_switched: String,
    project_created: String,
    project_renamed: String,
    project_deleted: String,
    project_invalid_name: String,
    project_already_exists: String,
    rename_failed: String,
    delete_failed: String,
    popup_delete_project_confirm: String,
    project_create_title: String,
    project_rename_title: String,
    project_limit_reached: String,
    overdue_prefix: String,
    project_prefix: String,
    pending_prefix: String,
    done_prefix_stat: String,
    pinned_prefix: String,
    total_prefix: String,
    task_added: String,
    task_updated: String,
    done_msg: String,
    undone_msg: String,
    pinned_msg: String,
    unpinned_msg: String,
    tag_set: String,
    tag_cleared: String,
    deleted: String,
    all_deleted: String,
    filters_cleared: String,
    all_done: String,
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
        fill_field!(celebration_line1);
        fill_field!(celebration_line2);
        fill_field!(celebration_line3);
        fill_field!(celebration_line4);
        fill_field!(celebration_line5);
        fill_field!(celebration_line6);
        fill_field!(created_prefix);
        fill_field!(done_prefix);
        fill_field!(pinned_marker);
        fill_field!(selected_header);
        fill_field!(tags_header);
        fill_field!(stats_header);
        fill_field!(help_content);
        fill_field!(popup_new_subtask_title);
        fill_field!(popup_new_subtask_hint);
        fill_field!(popup_help_title);
        fill_field!(popup_help_hint);
        fill_field!(mood_all_done);
        fill_field!(mood_empty);
        fill_field!(mood_one);
        fill_field!(mood_few);
        fill_field!(mood_several);
        fill_field!(mood_lots);
        fill_field!(mood_heap);
        fill_field!(mood_pile);
        fill_field!(mood_overwhelming);
        fill_field!(mood_hectic);
        fill_field!(mood_crazy);
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
        fill_field!(popup_due_date_title);
        fill_field!(popup_due_date_hint);
        fill_field!(popup_due_time_hint);
        fill_field!(due_date_cleared);
        fill_field!(due_date_set);
        fill_field!(due_date_invalid);
        fill_field!(statusbar_hint_wide);
        fill_field!(statusbar_hint_medium);
        fill_field!(statusbar_hint_narrow);
        fill_field!(column_header);
        fill_field!(scroll_up);
        fill_field!(scroll_down);
        fill_field!(topbar_title);
        fill_field!(topbar_date_format);
        fill_field!(title);
        fill_field!(right_title);
        fill_field!(empty_list_line1);
        fill_field!(empty_list_line2);
        fill_field!(calendar_title);
        fill_field!(calendar_mon);
        fill_field!(calendar_tue);
        fill_field!(calendar_wed);
        fill_field!(calendar_thu);
        fill_field!(calendar_fri);
        fill_field!(calendar_sat);
        fill_field!(calendar_sun);
        fill_field!(project_menu_title);
        fill_field!(project_menu_help_title);
        fill_field!(project_menu_help_switch);
        fill_field!(project_menu_help_create);
        fill_field!(project_menu_help_rename);
        fill_field!(project_menu_help_delete);
        fill_field!(project_menu_hint_c);
        fill_field!(project_menu_hint_r);
        fill_field!(project_menu_hint_d);
        fill_field!(project_menu_hint_enter);
        fill_field!(only_one_project);
        fill_field!(project_switched);
        fill_field!(project_created);
        fill_field!(project_renamed);
        fill_field!(project_deleted);
        fill_field!(project_invalid_name);
        fill_field!(project_already_exists);
        fill_field!(rename_failed);
        fill_field!(delete_failed);
        fill_field!(popup_delete_project_confirm);
        fill_field!(project_create_title);
        fill_field!(project_rename_title);
        fill_field!(project_limit_reached);
        fill_field!(overdue_prefix);
        fill_field!(project_prefix);
        fill_field!(pending_prefix);
        fill_field!(done_prefix_stat);
        fill_field!(pinned_prefix);
        fill_field!(total_prefix);
        fill_field!(task_added);
        fill_field!(task_updated);
        fill_field!(done_msg);
        fill_field!(undone_msg);
        fill_field!(pinned_msg);
        fill_field!(unpinned_msg);
        fill_field!(tag_set);
        fill_field!(tag_cleared);
        fill_field!(deleted);
        fill_field!(all_deleted);
        fill_field!(filters_cleared);
        fill_field!(all_done);
        for (k, v) in default.extra.iter() {
            if !self.extra.contains_key(k) {
                self.extra.insert(k.clone(), v.clone());
            }
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
            match toml::from_str::<Localization>(content) {
                Ok(loc) => {
                    builtin_langs.insert(code.to_string(), loc);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to parse builtin localization for '{}': {}", code, e);
                }
            }
        }
        let mut all_langs = builtin_langs.clone();
        let mut en_parse_error: Option<anyhow::Error> = None;
        if config_dir.exists() && config_dir.is_dir() {
            for entry in fs::read_dir(&config_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.file_name().and_then(|n| n.to_str()).map_or(false, |name| name.starts_with("lang_") && name.ends_with(".toml")) {
                    let name = path.file_name().unwrap().to_str().unwrap();
                    let code = &name[5..name.len()-5];
                    let content = fs::read_to_string(&path)?;
                    match toml::from_str::<Localization>(&content) {
                        Ok(loc) => {
                            all_langs.insert(code.to_string(), loc);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to parse external localization file '{}' for code '{}': {}", path.display(), code, e);
                            if code == "en" {
                                en_parse_error = Some(anyhow::anyhow!(e));
                            }
                        }
                    }
                }
            }
        }
        let en_loc = if let Some(loc) = all_langs.get("en").cloned() {
            loc
        } else {
            eprintln!("[FATAL] English localization not found in loaded languages.");
            eprintln!("Loaded languages: {:?}", all_langs.keys().collect::<Vec<_>>());
            if let Some(err) = en_parse_error {
                eprintln!("Parse error for lang_en.toml: {}", err);
            }
            anyhow::bail!("English localization missing");
        };
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
        let val = match key {
            "celebration_line1" => Some(&loc.celebration_line1),
            "celebration_line2" => Some(&loc.celebration_line2),
            "celebration_line3" => Some(&loc.celebration_line3),
            "celebration_line4" => Some(&loc.celebration_line4),
            "celebration_line5" => Some(&loc.celebration_line5),
            "celebration_line6" => Some(&loc.celebration_line6),
            "created_prefix" => Some(&loc.created_prefix),
            "done_prefix" => Some(&loc.done_prefix),
            "pinned_marker" => Some(&loc.pinned_marker),
            "selected_header" => Some(&loc.selected_header),
            "tags_header" => Some(&loc.tags_header),
            "stats_header" => Some(&loc.stats_header),
            "help_content" => Some(&loc.help_content),
            "popup_new_subtask_title" => Some(&loc.popup_new_subtask_title),
            "popup_new_subtask_hint" => Some(&loc.popup_new_subtask_hint),
            "popup_help_title" => Some(&loc.popup_help_title),
            "popup_help_hint" => Some(&loc.popup_help_hint),
            "mood_all_done" => Some(&loc.mood_all_done),
            "mood_empty" => Some(&loc.mood_empty),
            "mood_one" => Some(&loc.mood_one),
            "mood_few" => Some(&loc.mood_few),
            "mood_several" => Some(&loc.mood_several),
            "mood_lots" => Some(&loc.mood_lots),
            "mood_heap" => Some(&loc.mood_heap),
            "mood_pile" => Some(&loc.mood_pile),
            "mood_overwhelming" => Some(&loc.mood_overwhelming),
            "mood_hectic" => Some(&loc.mood_hectic),
            "mood_crazy" => Some(&loc.mood_crazy),
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
            "popup_due_date_title" => Some(&loc.popup_due_date_title),
            "popup_due_date_hint" => Some(&loc.popup_due_date_hint),
            "popup_due_time_hint" => Some(&loc.popup_due_time_hint),
            "due_date_cleared" => Some(&loc.due_date_cleared),
            "due_date_set" => Some(&loc.due_date_set),
            "due_date_invalid" => Some(&loc.due_date_invalid),
            "statusbar_hint_wide" => Some(&loc.statusbar_hint_wide),
            "statusbar_hint_medium" => Some(&loc.statusbar_hint_medium),
            "statusbar_hint_narrow" => Some(&loc.statusbar_hint_narrow),
            "column_header" => Some(&loc.column_header),
            "scroll_up" => Some(&loc.scroll_up),
            "scroll_down" => Some(&loc.scroll_down),
            "topbar_title" => Some(&loc.topbar_title),
            "topbar_date_format" => Some(&loc.topbar_date_format),
            "title" => Some(&loc.title),
            "right_title" => Some(&loc.right_title),
            "empty_list_line1" => Some(&loc.empty_list_line1),
            "empty_list_line2" => Some(&loc.empty_list_line2),
            "calendar_title" => Some(&loc.calendar_title),
            "calendar_mon" => Some(&loc.calendar_mon),
            "calendar_tue" => Some(&loc.calendar_tue),
            "calendar_wed" => Some(&loc.calendar_wed),
            "calendar_thu" => Some(&loc.calendar_thu),
            "calendar_fri" => Some(&loc.calendar_fri),
            "calendar_sat" => Some(&loc.calendar_sat),
            "calendar_sun" => Some(&loc.calendar_sun),
            "project_menu_title" => Some(&loc.project_menu_title),
            "project_menu_help_title" => Some(&loc.project_menu_help_title),
            "project_menu_help_switch" => Some(&loc.project_menu_help_switch),
            "project_menu_help_create" => Some(&loc.project_menu_help_create),
            "project_menu_help_rename" => Some(&loc.project_menu_help_rename),
            "project_menu_help_delete" => Some(&loc.project_menu_help_delete),
            "project_menu_hint_c" => Some(&loc.project_menu_hint_c),
            "project_menu_hint_r" => Some(&loc.project_menu_hint_r),
            "project_menu_hint_d" => Some(&loc.project_menu_hint_d),
            "project_menu_hint_enter" => Some(&loc.project_menu_hint_enter),
            "only_one_project" => Some(&loc.only_one_project),
            "project_switched" => Some(&loc.project_switched),
            "project_created" => Some(&loc.project_created),
            "project_renamed" => Some(&loc.project_renamed),
            "project_deleted" => Some(&loc.project_deleted),
            "project_invalid_name" => Some(&loc.project_invalid_name),
            "project_already_exists" => Some(&loc.project_already_exists),
            "rename_failed" => Some(&loc.rename_failed),
            "delete_failed" => Some(&loc.delete_failed),
            "popup_delete_project_confirm" => Some(&loc.popup_delete_project_confirm),
            "project_create_title" => Some(&loc.project_create_title),
            "project_rename_title" => Some(&loc.project_rename_title),
            "project_limit_reached" => Some(&loc.project_limit_reached),
            "overdue_prefix" => Some(&loc.overdue_prefix),
            "project_prefix" => Some(&loc.project_prefix),
            "pending_prefix" => Some(&loc.pending_prefix),
            "done_prefix_stat" => Some(&loc.done_prefix_stat),
            "pinned_prefix" => Some(&loc.pinned_prefix),
            "total_prefix" => Some(&loc.total_prefix),
            "task_added" => Some(&loc.task_added),
            "task_updated" => Some(&loc.task_updated),
            "done_msg" => Some(&loc.done_msg),
            "undone_msg" => Some(&loc.undone_msg),
            "pinned_msg" => Some(&loc.pinned_msg),
            "unpinned_msg" => Some(&loc.unpinned_msg),
            "tag_set" => Some(&loc.tag_set),
            "tag_cleared" => Some(&loc.tag_cleared),
            "deleted" => Some(&loc.deleted),
            "all_deleted" => Some(&loc.all_deleted),
            "filters_cleared" => Some(&loc.filters_cleared),
            "all_done" => Some(&loc.all_done),
            _ => loc.extra.get(key),
        };
        val.map(|s| s.as_str())
    }

    pub fn get(&self, key: &str) -> &str {
        let current_loc = &self.languages[self.current_index].1;
        if let Some(val) = self.get_from_loc(current_loc, key) {
            // eprintln!("[DEBUG i18n] key='{}' → found in current ({}), value='{}'", key, self.current_code(), val);
            return val;
        }
        if let Some(val) = self.get_from_loc(&self.default_loc, key) {
            // eprintln!("[DEBUG i18n] key='{}' not in current, but found in default (en), value='{}'", key, val);
            return val;
        }
        // eprintln!("[DEBUG i18n] key='{}' NOT FOUND in any language, returning '???'", key);
        "???"
    }
}