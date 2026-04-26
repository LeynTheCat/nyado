use crate::todo::Todo;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

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
        let mut file = File::create(&self.path).unwrap();
        for todo in &self.todos {
            write!(file, "{}", todo.to_line()).unwrap();
        }
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
}