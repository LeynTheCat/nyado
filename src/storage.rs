use crate::todo::Todo;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

const MAX_BACKUPS: usize = 5;

pub struct Storage {
    pub todos: Vec<Todo>,
    pub search: String,
    pub filter_tag: String,
    pub tags_available: Vec<(String, usize)>,
    pub dirty_tags: bool,
    path: PathBuf,
}

impl Storage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            todos: Vec::new(),
            search: String::new(),
            filter_tag: String::new(),
            tags_available: Vec::new(),
            dirty_tags: true,
            path,
        }
    }

    pub fn load(&mut self) {
        if !self.path.exists() {
            self.todos.clear();
            self.dirty_tags = true;
            return;
        }
        let file = File::open(&self.path).unwrap();
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
        let mut file = File::create(&self.path).unwrap();
        for todo in &self.todos {
            write!(file, "{}", todo.to_line()).unwrap();
        }
    }

    fn create_backup(&self) {
        if !self.path.exists() {
            return;
        }
        let default_dir = PathBuf::from(".");
        let dir = self.path.parent().unwrap_or(&default_dir);
        let stem = self.path.file_stem().unwrap().to_str().unwrap();
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let backup_name = |n: usize| {
            if ext.is_empty() {
                format!("{}.bak.{}", stem, n)
            } else {
                format!("{}.{}.bak.{}", stem, ext, n)
            }
        };

        for i in (0..MAX_BACKUPS-1).rev() {
            let old = dir.join(backup_name(i));
            let new = dir.join(backup_name(i+1));
            if old.exists() {
                let _ = fs::rename(&old, &new);
            }
        }
        let backup_path = dir.join(backup_name(0));
        let _ = fs::copy(&self.path, &backup_path);
    }

    /*
    pub fn restore_latest_backup(&mut self) -> bool {
        let default_dir = PathBuf::from(".");
        let dir = self.path.parent().unwrap_or(&default_dir);
        let stem = self.path.file_stem().unwrap().to_str().unwrap();
        let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let backup_name = if ext.is_empty() {
            format!("{}.bak.0", stem)
        } else {
            format!("{}.{}.bak.0", stem, ext)
        };
        let backup_path = dir.join(backup_name);
        if backup_path.exists() {
            let _ = fs::copy(&backup_path, &self.path);
            self.load();
            true
        } else {
            false
        }
    }
    */
    

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
}