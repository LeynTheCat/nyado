use crate::todo::{Todo, now_secs};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use anyhow::{Result, bail};

const MAX_BACKUPS: usize = 5;
const MAX_PROJECTS: usize = 64;

pub fn get_data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("nyado")
}

pub fn get_config_dir() -> PathBuf {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("nyado");
        if dir.exists() && dir.is_dir() {
            return dir;
        }
    }
    if PathBuf::from("config").exists() && PathBuf::from("config").is_dir() {
        return PathBuf::from("config");
    }
    let mut fallback = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    fallback.push("nyado");
    fallback
}

pub fn migrate_old_todos(data_dir: &PathBuf, projects_dir: &PathBuf) -> Result<()> {
    let old_todos = data_dir.join("todos.txt");
    let old_default = projects_dir.join("default.txt");
    if old_todos.exists() {
        let should_migrate = if !old_default.exists() {
            true
        } else {
            let default_size = fs::metadata(&old_default).map(|m| m.len()).unwrap_or(0);
            default_size == 0
        };
        if should_migrate {
            if let Ok(content) = fs::read_to_string(&old_todos) {
                let mut file = File::create(&old_default)?;
                write!(file, "{}", content)?;
            }
            let _ = fs::remove_file(&old_todos);
        }
    }
    Ok(())
}

pub struct Storage {
    pub todos: Vec<Todo>,
    pub search: String,
    pub filter_tag: String,
    pub tags_available: Vec<(String, usize)>,
    pub dirty_tags: bool,
    projects_dir: PathBuf,
    pub current_project: String,
}

impl Storage {
    pub fn new(projects_dir: PathBuf) -> Self {
        Self {
            todos: Vec::new(),
            search: String::new(),
            filter_tag: String::new(),
            tags_available: Vec::new(),
            dirty_tags: true,
            projects_dir,
            current_project: "default".to_string(),
        }
    }

    fn backup_dir(&self) -> PathBuf {
        self.projects_dir.join(".backups").join(&self.current_project)
    }

    pub fn set_project(&mut self, name: &str) -> bool {
        let project_file = self.projects_dir.join(format!("{}.txt", name));
        if !project_file.exists() && name != "default" {
            return false;
        }
        self.current_project = name.to_string();
        self.load_current();
        self.filter_tag.clear();
        self.search.clear();
        self.rebuild_tags();
        true
    }

    pub fn load_current(&mut self) {
        let path = self.projects_dir.join(format!("{}.txt", self.current_project));
        if !path.exists() {
            self.todos.clear();
            self.dirty_tags = true;
            self.save();
            return;
        }
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        self.todos.clear();
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(todo) = Todo::from_line(&line) {
                    self.todos.push(todo);
                }
            }
        }
        self.dirty_tags = true;
    }

    pub fn save(&self) {
        self.create_backup();
        let path = self.projects_dir.join(format!("{}.txt", self.current_project));
        let mut file = File::create(&path).unwrap();
        for todo in &self.todos {
            write!(file, "{}", todo.to_line()).unwrap();
        }
    }

    fn create_backup(&self) {
        let path = self.projects_dir.join(format!("{}.txt", self.current_project));
        if !path.exists() {
            return;
        }
        let backup_dir = self.backup_dir();
        let _ = fs::create_dir_all(&backup_dir);
        let backup_name = |n: usize| format!("{:02}.bak", n);
        for i in (0..MAX_BACKUPS-1).rev() {
            let old = backup_dir.join(backup_name(i));
            let new = backup_dir.join(backup_name(i+1));
            if old.exists() {
                let _ = fs::rename(&old, &new);
            }
        }
        let backup_path = backup_dir.join(backup_name(0));
        let _ = fs::copy(&path, &backup_path);
    }

    pub fn list_projects(&self) -> Vec<String> {
        let mut projects = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("txt") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        projects.push(name.to_string());
                    }
                }
            }
        }
        if projects.is_empty() {
            projects.push("default".to_string());
        }
        projects.sort();
        projects
    }

    pub fn create_project(&self, name: &str) -> bool {
        let projects = self.list_projects();
        if projects.len() >= MAX_PROJECTS {
            return false;
        }
        let path = self.projects_dir.join(format!("{}.txt", name));
        if path.exists() {
            return false;
        }
        File::create(&path).ok();
        let backup_dir = self.projects_dir.join(".backups").join(name);
        let _ = fs::create_dir_all(&backup_dir);
        true
    }

    pub fn delete_project(&self, name: &str) -> bool {
        if name == "default" {
            return false;
        }
        let path = self.projects_dir.join(format!("{}.txt", name));
        if path.exists() {
            fs::remove_file(&path).ok();
            let backup_dir = self.projects_dir.join(".backups").join(name);
            let _ = fs::remove_dir_all(&backup_dir);
            true
        } else {
            false
        }
    }

    pub fn rename_project(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name == "default" || new_name.is_empty() {
            return false;
        }
        let old_path = self.projects_dir.join(format!("{}.txt", old_name));
        let new_path = self.projects_dir.join(format!("{}.txt", new_name));
        if !old_path.exists() || new_path.exists() {
            return false;
        }
        fs::rename(&old_path, &new_path).ok();
        let old_backup = self.projects_dir.join(".backups").join(old_name);
        let new_backup = self.projects_dir.join(".backups").join(new_name);
        if old_backup.exists() {
            let _ = fs::rename(&old_backup, &new_backup);
        }
        if self.current_project == old_name {
            self.current_project = new_name.to_string();
            self.load_current();
            self.filter_tag.clear();
            self.search.clear();
            self.rebuild_tags();
        }
        true
    }

    pub fn rebuild_tags(&mut self) {
        if !self.dirty_tags {
            return;
        }
        let mut counts = HashMap::new();
        for todo in &self.todos {
            if !todo.tag.is_empty() {
                *counts.entry(todo.tag.clone()).or_insert(0) += 1;
            }
        }
        let mut tags: Vec<_> = counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        self.tags_available = tags;
        self.dirty_tags = false;
    }

    pub fn pending_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.done).count()
    }

    pub fn done_count(&self) -> usize {
        self.todos.iter().filter(|t| t.done).count()
    }

    pub fn pinned_count(&self) -> usize {
        self.todos.iter().filter(|t| t.pinned).count()
    }

    pub fn add_task(&mut self, text: &str, tag: &str) -> Result<()> {
        let todo = Todo::new(text, tag);
        self.todos.push(todo);
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn remove_task(&mut self, index: usize) -> Result<()> {
        if index == 0 || index > self.todos.len() {
            bail!("Invalid task index (1..{})", self.todos.len());
        }
        self.todos.remove(index - 1);
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn toggle_task(&mut self, index: usize) -> Result<()> {
        if index == 0 || index > self.todos.len() {
            bail!("Invalid task index");
        }
        let todo = &mut self.todos[index - 1];
        todo.done = !todo.done;
        todo.done_at = if todo.done { now_secs() } else { 0 };
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn pin_task(&mut self, index: usize) -> Result<()> {
        if index == 0 || index > self.todos.len() {
            bail!("Invalid task index");
        }
        self.todos[index - 1].pinned = true;
        self.save();
        Ok(())
    }

    pub fn unpin_task(&mut self, index: usize) -> Result<()> {
        if index == 0 || index > self.todos.len() {
            bail!("Invalid task index");
        }
        self.todos[index - 1].pinned = false;
        self.save();
        Ok(())
    }

    pub fn set_due_date(&mut self, index: usize, due: u64) -> Result<()> {
        if index == 0 || index > self.todos.len() {
            bail!("Invalid task index");
        }
        self.todos[index - 1].due_date = due;
        self.save();
        Ok(())
    }

    pub fn list_tasks_filtered(&self, filter_done: Option<bool>, filter_pinned: Option<bool>, filter_tag: Option<&str>) -> Vec<(usize, &Todo)> {
        self.todos
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                if let Some(done) = filter_done {
                    if t.done != done {
                        return false;
                    }
                }
                if let Some(pinned) = filter_pinned {
                    if t.pinned != pinned {
                        return false;
                    }
                }
                if let Some(tag) = filter_tag {
                    if t.tag != tag {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn get_stats(&self) -> (usize, usize, usize, usize) {
        let total = self.todos.len();
        let done = self.todos.iter().filter(|t| t.done).count();
        let pending = total - done;
        let pinned = self.todos.iter().filter(|t| t.pinned).count();
        (total, done, pending, pinned)
    }

    pub fn done_percent(&self) -> f64 {
        let (total, done, _, _) = self.get_stats();
        if total == 0 {
            0.0
        } else {
            (done as f64 / total as f64) * 100.0
        }
    }
}