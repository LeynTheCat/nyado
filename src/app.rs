use crate::commands::{key_to_command, Command};
use crate::i18n::I18n;
use crate::popup::popup;
use crate::storage::Storage;
use crate::todo::Todo;
use crate::ui::{draw, draw_toosmall};
use crate::todo::now_secs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::fs;
use std::io;
use std::path::PathBuf;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

const MAX_TODOS: usize = 1024;
const LANG_PREF_FILE: &str = "lang_pref.txt";

pub struct App {
    storage: Storage,
    pub visible: Vec<usize>,
    selected: usize,
    list_top: usize,
    i18n: I18n,
    message: String,
    message_ttl: u8,
    celebrate: u8,
}

fn parse_datetime(date_str: &str, time_str: &str) -> Option<u64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    let naive_date = NaiveDate::from_ymd_opt(year, month, day)?;
    let (hour, minute) = if time_str.is_empty() {
        (0, 0)
    } else {
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 2 {
            return None;
        }
        let h = time_parts[0].parse::<u32>().ok()?;
        let m = time_parts[1].parse::<u32>().ok()?;
        (h, m)
    };
    let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)?;
    let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
    Some(naive_datetime.and_utc().timestamp() as u64)
}

impl App {
    pub fn new(storage: Storage, i18n: I18n) -> Self {
        let mut app = Self {
            storage,
            selected: 0,
            visible: Vec::new(),
            list_top: 0,
            i18n,
            message: String::new(),
            message_ttl: 0,
            celebrate: 0,
        };
        app.rebuild_visible();
        app
    }

    pub fn rebuild_visible(&mut self) {
        self.visible.clear();
        let search_lower = self.storage.search.to_lowercase();
        for (idx, todo) in self.storage.todos.iter().enumerate() {
            if !self.storage.filter_tag.is_empty() && todo.tag != self.storage.filter_tag {
                continue;
            }
            if !self.storage.search.is_empty() && !todo.text.to_lowercase().contains(&search_lower) {
                continue;
            }
            self.visible.push(idx);
        }
        if self.selected >= self.visible.len() {
            self.selected = if self.visible.is_empty() { 0 } else { self.visible.len() - 1 };
        }
        self.storage.dirty_tags = true;
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.message_ttl = 5;
    }

    fn check_all_done(&mut self) {
        if self.storage.pending_count() == 0 && !self.storage.todos.is_empty() {
            self.celebrate = 10;
            let msg = self.i18n.get("messages.all_done").to_string();
            self.set_message(&msg);
        }
    }

    fn sort_todos(&mut self) {
        self.storage.todos.sort_by(|a, b| {
            if a.pinned != b.pinned {
                return b.pinned.cmp(&a.pinned);
            }
            if a.done != b.done {
                return a.done.cmp(&b.done);
            }
            let now = now_secs();
            let a_overdue = a.due_date > 0 && a.due_date < now;
            let b_overdue = b.due_date > 0 && b.due_date < now;
            if a_overdue != b_overdue {
                return b_overdue.cmp(&a_overdue);
            }
            let a_has_due = a.due_date > 0;
            let b_has_due = b.due_date > 0;
            if a_has_due != b_has_due {
                return b_has_due.cmp(&a_has_due);
            }
            if a_has_due && b_has_due {
                return a.due_date.cmp(&b.due_date);
            }
            std::cmp::Ordering::Equal
        });
        self.storage.dirty_tags = true;
        self.rebuild_visible();
    }

    fn save_current_language(&self, data_dir: &PathBuf) {
        let code = self.i18n.current_code();
        let path = data_dir.join(LANG_PREF_FILE);
        let _ = fs::write(path, code);
    }

    fn load_saved_language(i18n: &mut I18n, data_dir: &PathBuf) {
        let path = data_dir.join(LANG_PREF_FILE);
        if let Ok(content) = fs::read_to_string(&path) {
            let code = content.trim();
            i18n.set_language_by_code(code);
        }
    }

    fn cmd_language(&mut self, data_dir: &PathBuf) {
        self.i18n.toggle_language();
        let lang_code = self.i18n.current_code();
        self.set_message(&format!("Language: {}", lang_code));
        self.save_current_language(data_dir);
    }

    fn cmd_up(&mut self) { if self.selected > 0 { self.selected -= 1; } }
    fn cmd_down(&mut self) { if self.selected + 1 < self.visible.len() { self.selected += 1; } }
    fn cmd_top(&mut self) { self.selected = 0; }
    fn cmd_bottom(&mut self) { self.selected = self.visible.len().saturating_sub(1); }
    fn cmd_page_up(&mut self, term: &Terminal<CrosstermBackend<io::Stdout>>) {
        let step = (term.size().unwrap().height as usize).saturating_sub(5);
        self.selected = self.selected.saturating_sub(step);
    }
    fn cmd_page_down(&mut self, term: &Terminal<CrosstermBackend<io::Stdout>>) {
        let step = (term.size().unwrap().height as usize).saturating_sub(5);
        self.selected = (self.selected + step).min(self.visible.len().saturating_sub(1));
    }

    fn cmd_new_task(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if let Ok(Some(text)) = popup(&self.i18n.get("popup_new_title"), &self.i18n.get("popup_new_hint"), "", true, term) {
            if self.storage.todos.len() < MAX_TODOS {
                let tag = if self.storage.filter_tag.is_empty() { String::new() } else { self.storage.filter_tag.clone() };
                self.storage.todos.push(Todo::new(&text, &tag));
                self.sort_todos();
                self.storage.save();
                let msg = self.i18n.get("messages.task_added").to_string();
                self.set_message(&msg);
            }
        }
    }

    fn cmd_edit_task(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if !self.visible.is_empty() {
            let idx = self.visible[self.selected];
            let old_text = self.storage.todos[idx].text.clone();
            if let Ok(Some(new_text)) = popup(&self.i18n.get("popup_edit_title"), &self.i18n.get("popup_edit_hint"), &old_text, true, term) {
                self.storage.todos[idx].text = new_text;
                self.storage.save();
                self.rebuild_visible();
                let msg = self.i18n.get("messages.task_updated").to_string();
                self.set_message(&msg);
            }
        }
    }

    fn cmd_toggle_done(&mut self) {
        if !self.visible.is_empty() {
            let idx = self.visible[self.selected];
            let done = !self.storage.todos[idx].done;
            let done_at = if done { now_secs() } else { 0 };
            self.storage.todos[idx].done = done;
            self.storage.todos[idx].done_at = done_at;
            self.sort_todos();
            self.storage.save();
            self.check_all_done();
            let msg = if done { self.i18n.get("messages.done").to_string() } else { self.i18n.get("messages.undone").to_string() };
            self.set_message(&msg);
        }
    }

    fn cmd_toggle_pin(&mut self) {
        if !self.visible.is_empty() {
            let idx = self.visible[self.selected];
            self.storage.todos[idx].pinned = !self.storage.todos[idx].pinned;
            self.sort_todos();
            self.storage.save();
            let msg = if self.storage.todos[idx].pinned { self.i18n.get("messages.pinned").to_string() } else { self.i18n.get("messages.unpinned").to_string() };
            self.set_message(&msg);
        }
    }

    fn cmd_set_tag(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if !self.visible.is_empty() {
            let hint = if !self.storage.tags_available.is_empty() {
                let existing: Vec<String> = self.storage.tags_available.iter().take(6).map(|(t,_)| format!("#{}", t)).collect();
                format!("{}{}", self.i18n.get("popup_set_tag_hint_existing"), existing.join(" "))
            } else {
                self.i18n.get("popup_set_tag_hint_empty").to_string()
            };
            if let Ok(Some(tag_raw)) = popup(&self.i18n.get("popup_set_tag_title"), &hint, "", false, term) {
                let cleaned: String = tag_raw.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).take(32).collect();
                let idx = self.visible[self.selected];
                if cleaned.is_empty() {
                    self.storage.todos[idx].tag.clear();
                    let msg = self.i18n.get("messages.tag_cleared").to_string();
                    self.set_message(&msg);
                } else {
                    self.storage.todos[idx].tag = cleaned;
                    let msg = self.i18n.get("messages.tag_set").to_string();
                    self.set_message(&msg);
                }
                self.storage.dirty_tags = true;
                self.storage.rebuild_tags();
                self.sort_todos();
                self.storage.save();
            }
        }
    }

    fn cmd_delete_task(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if !self.visible.is_empty() {
            let idx = self.visible[self.selected];
            let text = &self.storage.todos[idx].text;
            let template = self.i18n.get("popup_delete_confirm");
            let prompt = template.replace("{}", text);
            if let Ok(Some(ans)) = popup(&prompt, "", "", false, term) {
                if ans == "y" || ans == "Y" {
                    self.storage.todos.remove(idx);
                    if self.selected >= self.visible.len().saturating_sub(1) && self.selected > 0 {
                        self.selected -= 1;
                    }
                    self.sort_todos();
                    self.storage.save();
                    let msg = self.i18n.get("messages.deleted").to_string();
                    self.set_message(&msg);
                }
            }
        }
    }

    fn cmd_delete_all(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if !self.storage.todos.is_empty() {
            let template = self.i18n.get("popup_delete_all_confirm");
            let prompt = template.replace("{}", &self.storage.todos.len().to_string());
            if let Ok(Some(ans)) = popup(&prompt, &self.i18n.get("popup_delete_all_warning"), "", false, term) {
                if ans == "y" || ans == "Y" {
                    self.storage.todos.clear();
                    self.selected = 0;
                    self.sort_todos();
                    self.storage.save();
                    let msg = self.i18n.get("messages.all_deleted").to_string();
                    self.set_message(&msg);
                }
            }
        }
    }

    fn cmd_search(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if let Ok(Some(q)) = popup(&self.i18n.get("popup_search_title"), &self.i18n.get("popup_search_hint"), &self.storage.search, true, term) {
            self.storage.search = q;
        } else {
            self.storage.search.clear();
        }
        self.selected = 0;
        self.rebuild_visible();
    }

    fn cmd_clear_filters(&mut self) {
        self.storage.search.clear();
        self.storage.filter_tag.clear();
        self.selected = 0;
        self.rebuild_visible();
        let msg = self.i18n.get("messages.filters_cleared").to_string();
        self.set_message(&msg);
    }

    fn cmd_filter_tag(&mut self, idx: usize) {
        if idx < self.storage.tags_available.len() {
            let tag = self.storage.tags_available[idx].0.clone();
            if self.storage.filter_tag == tag {
                self.storage.filter_tag.clear();
            } else {
                self.storage.filter_tag = tag;
            }
            self.selected = 0;
            self.rebuild_visible();
            // let filter_msg = self.i18n.get("messages.filter_format").to_string().replace("{}", &self.storage.filter_tag);
            // self.set_message(&filter_msg);
        }
    }

    fn cmd_set_due_date(&mut self, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        if !self.visible.is_empty() {
            let idx = self.visible[self.selected];
            let date_title = self.i18n.get("popup_due_date_title").to_string();
            let date_hint = self.i18n.get("popup_due_date_hint").to_string();
            if let Ok(Some(date_str)) = popup(&date_title, &date_hint, "", false, term) {
                let trimmed_date = date_str.trim();
                if trimmed_date.is_empty() {
                    self.storage.todos[idx].due_date = 0;
                    let msg = self.i18n.get("due_date_cleared").to_string();
                    self.set_message(&msg);
                    self.sort_todos();
                    self.storage.save();
                    return;
                }
                let time_hint = self.i18n.get("popup_due_time_hint").to_string();
                let time_res = popup("", &time_hint, "", false, term);
                let time_str = match time_res {
                    Ok(Some(t)) => t.trim().to_string(),
                    Ok(None) => "".to_string(),
                    Err(_) => "".to_string(),
                };
                if let Some(timestamp) = parse_datetime(trimmed_date, &time_str) {
                    self.storage.todos[idx].due_date = timestamp;
                    let display = if time_str.is_empty() {
                        format!("{} {}", self.i18n.get("due_date_set"), trimmed_date)
                    } else {
                        format!("{} {} {}", self.i18n.get("due_date_set"), trimmed_date, time_str)
                    };
                    self.set_message(&display);
                } else {
                    let msg = self.i18n.get("due_date_invalid").to_string();
                    self.set_message(&msg);
                    return;
                }
                self.sort_todos();
                self.storage.save();
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyCode, term: &mut Terminal<CrosstermBackend<io::Stdout>>, data_dir: &PathBuf) -> bool {
        match key_to_command(key) {
            Command::Quit => return false,
            Command::Language => self.cmd_language(data_dir),
            Command::Up => self.cmd_up(),
            Command::Down => self.cmd_down(),
            Command::Top => self.cmd_top(),
            Command::Bottom => self.cmd_bottom(),
            Command::PageUp => self.cmd_page_up(term),
            Command::PageDown => self.cmd_page_down(term),
            Command::NewTask => self.cmd_new_task(term),
            Command::EditTask => self.cmd_edit_task(term),
            Command::ToggleDone => self.cmd_toggle_done(),
            Command::TogglePin => self.cmd_toggle_pin(),
            Command::SetTag => self.cmd_set_tag(term),
            Command::DeleteTask => self.cmd_delete_task(term),
            Command::DeleteAll => self.cmd_delete_all(term),
            Command::Search => self.cmd_search(term),
            Command::ClearFilters => self.cmd_clear_filters(),
            Command::FilterTag(idx) => self.cmd_filter_tag(idx),
            Command::SetDueDate => self.cmd_set_due_date(term),
            Command::None => {}
        }
        true
    }

    pub fn tick(&mut self) {
        if self.celebrate > 0 {
            self.celebrate -= 1;
        }
        if self.message_ttl > 0 {
            self.message_ttl -= 1;
        } else {
            self.message.clear();
        }
    }

    pub fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), anyhow::Error> {
        terminal.draw(|f| {
            let size = f.size();
            if size.height < 10 || size.width < 30 {
                draw_toosmall(f, size);
                return;
            }
            draw(
                f, size,
                &self.storage,
                &self.visible,
                self.selected,
                &mut self.list_top,
                &self.i18n,
                &self.message,
                self.celebrate > 0,
            );
        })?;
        Ok(())
    }
}

pub fn run() -> anyhow::Result<()> {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("nyado");
    std::fs::create_dir_all(&data_dir)?;
    let todos_path = data_dir.join("todos.txt");

    let i18n = I18n::new()?;
    let mut storage = Storage::new(todos_path);
    storage.load();
    storage.rebuild_tags();

    let mut app = App::new(storage, i18n);
    App::load_saved_language(&mut app.i18n, &data_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut running = true;
    while running {
        app.tick();
        app.draw(&mut terminal)?;

        let timeout = if app.celebrate > 0 {
            std::time::Duration::from_millis(150)
        } else {
            std::time::Duration::from_millis(100)
        };
        if !event::poll(timeout)? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let keep = app.handle_input(key.code, &mut terminal, &data_dir);
                if !keep {
                    running = false;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    println!("bye bye~ =^..^=");
    Ok(())
}