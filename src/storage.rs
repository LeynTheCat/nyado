use crate::todo::Todo;
use crate::todo::now_secs;
use crate::config::config;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::PathBuf;
use anyhow::{Result, bail};

pub fn get_data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("nyado")
}

pub fn get_config_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("nyado");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn migrate_txt_to_yaml(txt_path: &PathBuf, yaml_path: &PathBuf) -> Result<()> {
    if !txt_path.exists() || yaml_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(txt_path)?;
    let mut todos = Vec::new();
    for line in content.lines() {
        if let Some(todo) = Todo::from_line_legacy(line) {
            todos.push(todo);
        }
    }
    let yaml = serde_yaml::to_string(&todos)?;
    fs::write(yaml_path, yaml)?;
    let backup_txt = txt_path.with_extension("txt.migrated");
    fs::rename(txt_path, backup_txt)?;
    Ok(())
}

pub fn migrate_old_todos(data_dir: &PathBuf, projects_dir: &PathBuf) -> Result<()> {
    let old_todos = data_dir.join("todos.txt");
    let old_default_txt = projects_dir.join("default.txt");
    let new_default_yaml = projects_dir.join("default.yaml");

    if old_todos.exists() && !new_default_yaml.exists() {
        migrate_txt_to_yaml(&old_todos, &new_default_yaml)?;
    }
    if old_default_txt.exists() && !new_default_yaml.exists() {
        migrate_txt_to_yaml(&old_default_txt, &new_default_yaml)?;
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
    pub expanded: HashSet<u64>,
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
            expanded: HashSet::new(),
        }
    }

    fn project_file(&self) -> PathBuf {
        self.projects_dir.join(format!("{}.yaml", self.current_project))
    }

    fn backup_dir(&self) -> PathBuf {
        self.projects_dir.join(".backups").join(&self.current_project)
    }

    fn rebuild_depth_in_todos(todos: &mut [Todo]) {
        for todo in todos {
            todo.rebuild_depth(0);
        }
    }

    pub fn set_project(&mut self, name: &str) -> bool {
        let project_file = self.projects_dir.join(format!("{}.yaml", name));
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
        let path = self.project_file();
        if !path.exists() {
            self.todos.clear();
            self.dirty_tags = true;
            self.expanded.clear();
            self.save();
            return;
        }
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_yaml::from_str::<Vec<Todo>>(&content) {
                    Ok(mut todos) => {
                        Self::rebuild_depth_in_todos(&mut todos);
                        self.todos = todos;
                    }
                    Err(e) => {
                        panic!("Failed to parse YAML file {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to read file {}: {}", path.display(), e);
            }
        }
        self.dirty_tags = true;
        self.sort_all();
    }

    pub fn save(&self) {
        self.create_backup();
        let path = self.project_file();
        if let Ok(yaml) = serde_yaml::to_string(&self.todos) {
            let _ = fs::write(&path, yaml);
        }
    }

    fn create_backup(&self) {
        let path = self.project_file();
        if !path.exists() {
            return;
        }
        let backup_dir = self.backup_dir();
        let _ = fs::create_dir_all(&backup_dir);
        let backup_name = |n: usize| format!("{:02}.bak", n);
        let max_backups = config().max_backups;
        for i in (0..max_backups-1).rev() {
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
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yaml") {
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
        if projects.len() >= config().max_projects {
            return false;
        }
        let path = self.projects_dir.join(format!("{}.yaml", name));
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
        let path = self.projects_dir.join(format!("{}.yaml", name));
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
        let old_path = self.projects_dir.join(format!("{}.yaml", old_name));
        let new_path = self.projects_dir.join(format!("{}.yaml", new_name));
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
            todo.collect_tags(&mut counts);
        }
        let mut tags: Vec<_> = counts.into_iter().collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        self.tags_available = tags;
        self.dirty_tags = false;
    }

    pub fn total_count(&self) -> usize {
        self.todos.iter().map(|t| t.total_count()).sum()
    }
    pub fn pending_count(&self) -> usize {
        self.todos.iter().map(|t| t.pending_count()).sum()
    }
    pub fn done_count(&self) -> usize {
        self.todos.iter().map(|t| t.done_count()).sum()
    }
    pub fn pinned_count(&self) -> usize {
        self.todos.iter().map(|t| t.pinned_count()).sum()
    }

    pub fn find_mut(&mut self, id: u64) -> Option<&mut Todo> {
        for todo in &mut self.todos {
            if let Some(found) = todo.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn add_task(&mut self, text: &str, tag: &str) -> Result<()> {
        let todo = Todo::new(text, tag);
        self.todos.push(todo);
        self.sort_all();
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn add_subtask(&mut self, parent_id: u64, text: &str, tag: &str) -> Result<()> {
        let parent = self.find_mut(parent_id).ok_or_else(|| anyhow::anyhow!("Parent not found"))?;
        if parent.depth >= config().max_depth {
            bail!("Maximum nesting depth reached (max {})", config().max_depth);
        }
        let parent_tag = parent.tag.clone();
        let subtag = if !parent_tag.is_empty() { parent_tag } else { tag.to_string() };
        let subtask = Todo::new_subtask(text, &subtag, parent_id, parent.depth);
        parent.children.push(subtask);
        self.sort_all();
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    fn propagate_done_recursive(todo: &mut Todo, done: bool, done_at: u64) {
        todo.done = done;
        todo.done_at = done_at;
        for child in &mut todo.children {
            Self::propagate_done_recursive(child, done, done_at);
        }
    }

    pub fn toggle_task(&mut self, id: u64) -> Result<()> {
        let todo = self.find_mut(id).ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        let new_done = !todo.done;
        let done_at = if new_done { crate::todo::now_secs() } else { 0 };
        Self::propagate_done_recursive(todo, new_done, done_at);
        self.dirty_tags = true;
        self.sort_all();
        self.save();
        Ok(())
    }

    pub fn pin_task(&mut self, id: u64) -> Result<()> {
        let todo = self.find_mut(id).ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        todo.pinned = true;
        self.sort_all();
        self.save();
        Ok(())
    }

    pub fn unpin_task(&mut self, id: u64) -> Result<()> {
        let todo = self.find_mut(id).ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        todo.pinned = false;
        self.sort_all();
        self.save();
        Ok(())
    }

    pub fn set_due_date(&mut self, id: u64, due: u64) -> Result<()> {
        let todo = self.find_mut(id).ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        todo.due_date = due;
        self.sort_all();
        self.save();
        Ok(())
    }

    pub fn set_tag(&mut self, id: u64, tag: &str) -> Result<()> {
        let todo = self.find_mut(id).ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        todo.tag = tag.to_string();
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn toggle_expand(&mut self, id: u64) {
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
    }

    pub fn is_expanded(&self, id: u64) -> bool {
        self.expanded.contains(&id)
    }

    pub fn build_visible_with_offset(&self, filter_tag: &str, search: &str) -> Vec<(u64, usize)> {
        let search_lower = search.to_lowercase();
        let mut visible = Vec::new();
        for todo in &self.todos {
            self.visit_for_visible(todo, &mut visible, filter_tag, &search_lower);
        }
        visible
    }

    fn visit_for_visible(&self, todo: &Todo, out: &mut Vec<(u64, usize)>, filter_tag: &str, search_lower: &str) {
        let tag_match = filter_tag.is_empty() || todo.tag == filter_tag;
        if !tag_match {
            return;
        }
        let search_match = search_lower.is_empty() || todo.text.to_lowercase().contains(&search_lower);
        if search_match {
            let offset = todo.depth * 4;
            out.push((todo.id, offset));
        }
        if self.is_expanded(todo.id) && !todo.children.is_empty() {
            for child in &todo.children {
                self.visit_for_visible(child, out, filter_tag, search_lower);
            }
        }
    }

    pub fn get_stats(&self) -> (usize, usize, usize, usize) {
        let total = self.total_count();
        let done = self.done_count();
        let pending = self.pending_count();
        let pinned = self.pinned_count();
        (total, done, pending, pinned)
    }

    pub fn get_todo(&self, id: u64) -> Option<&Todo> {
        for todo in &self.todos {
            if let Some(found) = todo.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn sort_all(&mut self) {
        let now = now_secs();
        self.todos.sort_by(|a, b| {
            if a.done != b.done {
                return a.done.cmp(&b.done);
            }
            if a.done {
                return b.done_at.cmp(&a.done_at);
            }
            if a.pinned != b.pinned {
                return b.pinned.cmp(&a.pinned);
            }
            let a_urg = if a.due_date > 0 {
                if a.due_date < now { 0 } else { a.due_date }
            } else {
                u64::MAX
            };
            let b_urg = if b.due_date > 0 {
                if b.due_date < now { 0 } else { b.due_date }
            } else {
                u64::MAX
            };
            if a_urg != b_urg {
                return a_urg.cmp(&b_urg);
            }
            b.created_at.cmp(&a.created_at)
        });
        for todo in &mut self.todos {
            todo.sort();
        }
    }

    pub fn done_percent(&self) -> f64 {
        let total = self.total_count();
        if total == 0 { 0.0 } else { (self.done_count() as f64 / total as f64) * 100.0 }
    }

    pub fn remove_task(&mut self, id: u64) -> Result<()> {
        if let Some(pos) = self.todos.iter().position(|t| t.id == id) {
            self.todos.remove(pos);
            self.dirty_tags = true;
            self.save();
            return Ok(());
        }
        fn remove_recursive(children: &mut Vec<Todo>, id: u64) -> Result<()> {
            if let Some(pos) = children.iter().position(|c| c.id == id) {
                children.remove(pos);
                return Ok(());
            }
            for child in children.iter_mut() {
                if let Some(_) = child.find_mut(id) {
                    return remove_recursive(&mut child.children, id);
                }
            }
            bail!("Task not found")
        }
        remove_recursive(&mut self.todos, id)?;
        self.sort_all();
        self.dirty_tags = true;
        self.save();
        Ok(())
    }

    pub fn list_tasks_flat(&self, filter_done: Option<bool>, filter_pinned: Option<bool>, filter_tag: Option<&str>) -> Vec<(usize, &Todo)> {
        let mut result = Vec::new();
        let mut index = 1;
        for todo in &self.todos {
            self.visit_flat(todo, &mut result, &mut index, filter_done, filter_pinned, filter_tag);
        }
        result
    }

    fn visit_flat<'a>(&self, todo: &'a Todo, out: &mut Vec<(usize, &'a Todo)>, idx: &mut usize,
                      filter_done: Option<bool>, filter_pinned: Option<bool>, filter_tag: Option<&str>) {
        let mut matches = true;
        if let Some(done) = filter_done {
            if todo.done != done { matches = false; }
        }
        if matches {
            if let Some(pinned) = filter_pinned {
                if todo.pinned != pinned { matches = false; }
            }
        }
        if matches {
            if let Some(tag) = filter_tag {
                if todo.tag != tag { matches = false; }
            }
        }
        if matches {
            out.push((*idx, todo));
            *idx += 1;
        }
        for child in &todo.children {
            self.visit_flat(child, out, idx, filter_done, filter_pinned, filter_tag);
        }
    }
}

impl Todo {
    pub fn from_line_legacy(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 6 {
            return None;
        }
        let pinned = parts[0] == "P";
        let done = parts[1] == "x";
        let tag = if parts[2] == "none" { String::new() } else { parts[2].to_string() };
        let text = parts[3].to_string();
        let created_at = parts[4].parse().unwrap_or(0);
        let done_at = parts[5].parse().unwrap_or(0);
        let due_date = if parts.len() >= 7 { parts[6].parse().unwrap_or(0) } else { 0 };
        Some(Todo {
            id: crate::todo::generate_id(),
            parent_id: None,
            depth: 0,
            done,
            pinned,
            tag,
            text,
            created_at,
            done_at,
            due_date,
            children: Vec::new(),
        })
    }
}