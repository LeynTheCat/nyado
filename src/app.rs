use crate::commands::key_to_command;
use crate::commands::Command;
use crate::handlers::*;
use crate::i18n::I18n;
use crate::storage::{get_data_dir, migrate_old_todos, Storage};
use crate::ui::progress_bar::ProgressState;
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

pub(crate) struct App {
    pub storage: Storage,
    pub visible: Vec<usize>,
    pub selected: usize,
    pub list_top: usize,
    pub i18n: I18n,
    pub message: String,
    pub message_ttl: u8,
    pub celebrate: u8,
    pub progress_state: ProgressState,
    pub search_mode: bool,
    pub search_buffer: String,
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
            progress_state: ProgressState::new(),
            search_mode: false,
            search_buffer: String::new(),
        };
        app.rebuild_visible();
        app
    }

    pub fn rebuild_visible(&mut self) {
        self.visible.clear();
        let filter_tag = &self.storage.filter_tag;
        let search_lower = if self.search_mode {
            self.search_buffer.to_lowercase()
        } else {
            self.storage.search.to_lowercase()
        };
        for (idx, todo) in self.storage.todos.iter().enumerate() {
            if !filter_tag.is_empty() && todo.tag != *filter_tag {
                continue;
            }
            if !search_lower.is_empty() && !todo.text.to_lowercase().contains(&search_lower) {
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

    pub fn sort_todos(&mut self) {
        self.storage.todos.sort_by(|a, b| {
            if a.pinned != b.pinned {
                return b.pinned.cmp(&a.pinned);
            }
            if a.done != b.done {
                return a.done.cmp(&b.done);
            }
            let now = crate::todo::now_secs();
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

    pub fn check_all_done(&mut self) {
        if self.storage.pending_count() == 0 && !self.storage.todos.is_empty() {
            self.celebrate = 15;
            let msg = self.i18n.get("messages.all_done").to_string();
            self.set_message(&msg);
        }
    }

    pub(crate) fn save_current_language(&self, data_dir: &PathBuf) {
        let code = self.i18n.current_code();
        let path = data_dir.join("lang_pref.txt");
        let _ = fs::write(path, code);
    }

    fn load_saved_language(i18n: &mut I18n, data_dir: &PathBuf) {
        let path = data_dir.join("lang_pref.txt");
        if let Ok(content) = fs::read_to_string(&path) {
            let code = content.trim();
            i18n.set_language_by_code(code);
        }
    }

    pub fn handle_input(&mut self, key: KeyCode, term: &mut Terminal<CrosstermBackend<io::Stdout>>, data_dir: &PathBuf) -> bool {
        if self.search_mode {
            match key {
                KeyCode::Char(c) => {
                    if self.search_buffer.len() < 64 {
                        self.search_buffer.push(c);
                        self.selected = 0;
                        self.rebuild_visible();
                    }
                }
                KeyCode::Backspace => {
                    self.search_buffer.pop();
                    self.selected = 0;
                    self.rebuild_visible();
                }
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_buffer.clear();
                    self.rebuild_visible();
                }
                KeyCode::Enter => {
                    self.storage.search = self.search_buffer.clone();
                    self.search_mode = false;
                    self.search_buffer.clear();
                    self.rebuild_visible();
                    self.set_message(&format!("Search: {}", self.storage.search));
                }
                _ => {}
            }
            return true;
        }

        match key_to_command(key) {
            Command::Quit => return false,
            Command::Language => language::toggle(self, data_dir),
            Command::Up => navigation::up(self),
            Command::Down => navigation::down(self),
            Command::Top => navigation::top(self),
            Command::Bottom => navigation::bottom(self),
            Command::PageUp => navigation::page_up(self, term),
            Command::PageDown => navigation::page_down(self, term),
            Command::NewTask => task::new(self, term),
            Command::EditTask => task::edit(self, term),
            Command::ToggleDone => task::toggle_done(self),
            Command::TogglePin => task::toggle_pin(self),
            Command::SetTag => task::set_tag(self, term),
            Command::DeleteTask => task::delete(self, term),
            Command::DeleteAll => task::delete_all(self, term),
            Command::Search => filter::start_search(self),
            Command::ClearFilters => filter::clear(self),
            Command::FilterTag(idx) => filter::by_tag(self, idx),
            Command::SetDueDate => due_date::set(self, term),
            Command::Help => help::show(self, term),
            Command::SwitchProject => project::menu(self, term),
            Command::PrevProject => project::prev(self),
            Command::NextProject => project::next(self),
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
                crate::ui::draw_toosmall(f, size);
                return;
            }
            crate::ui::draw(
                f, size,
                &self.storage,
                &self.visible,
                self.selected,
                &mut self.list_top,
                &self.i18n,
                &self.message,
                self.celebrate,
                &mut self.progress_state,
                self.search_mode,
                &self.search_buffer,
            );
        })?;
        Ok(())
    }
}

pub fn run() -> anyhow::Result<()> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let projects_dir = data_dir.join("projects");
    std::fs::create_dir_all(&projects_dir)?;

    migrate_old_todos(&data_dir, &projects_dir)?;

    let mut storage = Storage::new(projects_dir);
    if !storage.list_projects().contains(&"default".to_string()) {
        storage.create_project("default");
        storage.set_project("default");
    } else {
        storage.load_current();
    }
    storage.rebuild_tags();

    let i18n = I18n::new()?;
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