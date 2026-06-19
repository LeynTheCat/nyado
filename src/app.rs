use crate::commands::key_to_command;
use crate::commands::Command;
use crate::config::config;
use crate::handlers::*;
use crate::i18n::I18n;
use crate::storage::{get_data_dir, migrate_old_todos, Storage};
use crate::ui::progress_bar::ProgressState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::Rect;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub(crate) struct App {
    pub storage: Storage,
    pub visible: Vec<(u64, usize)>,
    pub selected: usize,
    pub list_top: usize,
    pub i18n: I18n,
    pub message: String,
    pub message_ttl: u8,
    pub celebrate: u8,
    pub progress_state: ProgressState,
    pub search_mode: bool,
    pub search_buffer: String,
    pub list_area: Option<Rect>,
    pub mouse_dragging: bool,
    pub line_to_task: Vec<usize>,
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
            list_area: None,
            mouse_dragging: false,
            line_to_task: Vec::new(),
        };
        app.sort_and_rebuild();
        app
    }

    pub fn rebuild_visible(&mut self) {
        let filter_tag = &self.storage.filter_tag;
        let search = if self.search_mode { &self.search_buffer } else { &self.storage.search };
        self.visible = self.storage.build_visible_with_offset(filter_tag, search);
        if self.selected >= self.visible.len() {
            self.selected = if self.visible.is_empty() { 0 } else { self.visible.len() - 1 };
        }
        self.storage.dirty_tags = true;
    }

    pub fn sort_and_rebuild(&mut self) {
        self.storage.sort_all();
        self.rebuild_visible();
        self.storage.rebuild_tags();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.message_ttl = 5;
    }

    pub fn check_all_done(&mut self) {
        if self.storage.pending_count() == 0 && self.storage.total_count() > 0 {
            self.celebrate = config().celebration_duration_frames;
            let msg = self.i18n.get("all_done").to_string();
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

    pub fn update_selection_from_mouse(&mut self, mouse: &MouseEvent) {
        if let Some(area) = self.list_area {
            let mouse_row = mouse.row;
            let mouse_col = mouse.column;
            if mouse_row >= area.y && mouse_row < area.y + area.height
                && mouse_col >= area.x && mouse_col < area.x + area.width
            {
                let line_idx = (mouse_row - area.y) as usize;
                if line_idx < self.line_to_task.len() {
                    let task_idx = self.line_to_task[line_idx];
                    let new_idx = self.list_top + task_idx;
                    if new_idx < self.visible.len() {
                        self.selected = new_idx;
                    }
                }
            }
        }
    }

    pub fn handle_mouse_click(&mut self, mouse: MouseEvent) {
        let old_selected = self.selected;
        self.update_selection_from_mouse(&mouse);
        if self.selected == old_selected && self.selected < self.visible.len() {
            let (id, _) = self.visible[self.selected];
            if let Some(todo) = self.storage.get_todo(id) {
                if !todo.children.is_empty() {
                    self.storage.toggle_expand(id);
                    self.rebuild_visible();
                    if self.selected >= self.visible.len() {
                        self.selected = if self.visible.is_empty() { 0 } else { self.visible.len() - 1 };
                    }
                }
            }
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
            Command::NewSubtask => task::new_subtask(self, term),
            Command::EditTask => task::edit(self, term),
            Command::ToggleDone => task::toggle_done(self),
            Command::TogglePin => task::toggle_pin(self),
            Command::ToggleExpand => task::toggle_expand(self),
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
                &mut self.list_area,
                &mut self.line_to_task,
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
            Duration::from_millis(config().celebration_frame_delay_ms)
        } else {
            Duration::from_millis(500)
        };
        if !event::poll(timeout)? {
            continue;
        }

        while let Ok(true) = event::poll(Duration::from_secs(0)) {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let keep = app.handle_input(key.code, &mut terminal, &data_dir);
                    if !keep {
                        running = false;
                        break;
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => navigation::up(&mut app),
                    MouseEventKind::ScrollDown => navigation::down(&mut app),
                    MouseEventKind::Down(btn) if btn == crossterm::event::MouseButton::Left => {
                        app.mouse_dragging = true;
                        app.handle_mouse_click(mouse);
                    }
                    MouseEventKind::Drag(btn) if btn == crossterm::event::MouseButton::Left => {
                        if app.mouse_dragging {
                            app.update_selection_from_mouse(&mouse);
                        }
                    }
                    MouseEventKind::Up(btn) if btn == crossterm::event::MouseButton::Left => {
                        app.mouse_dragging = false;
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    app.draw(&mut terminal)?;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    println!("bye bye~ =^..^=");
    Ok(())
}