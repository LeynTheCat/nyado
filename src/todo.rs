use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn generate_id() -> u64 {
    let now = now_secs();
    let random: u64 = rand::random();
    now ^ random
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub parent_id: Option<u64>,
    #[serde(default)]
    pub depth: usize,
    pub done: bool,
    pub pinned: bool,
    pub tag: String,
    pub text: String,
    pub created_at: u64,
    pub done_at: u64,
    pub due_date: u64,
    pub children: Vec<Todo>,
}

impl Todo {
    pub fn new(text: &str, tag: &str) -> Self {
        Self {
            id: generate_id(),
            parent_id: None,
            depth: 0,
            done: false,
            pinned: false,
            tag: tag.to_string(),
            text: text.to_string(),
            created_at: now_secs(),
            done_at: 0,
            due_date: 0,
            children: Vec::new(),
        }
    }

    pub fn new_subtask(text: &str, tag: &str, parent_id: u64, parent_depth: usize) -> Self {
        let mut subtask = Self::new(text, tag);
        subtask.parent_id = Some(parent_id);
        subtask.depth = parent_depth + 1;
        subtask
    }

    pub fn find_mut(&mut self, id: u64) -> Option<&mut Self> {
        if self.id == id {
            Some(self)
        } else {
            for child in &mut self.children {
                if let Some(found) = child.find_mut(id) {
                    return Some(found);
                }
            }
            None
        }
    }

    pub fn find(&self, id: u64) -> Option<&Self> {
        if self.id == id {
            Some(self)
        } else {
            for child in &self.children {
                if let Some(found) = child.find(id) {
                    return Some(found);
                }
            }
            None
        }
    }

    pub fn total_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_count()).sum::<usize>()
    }

    pub fn pending_count(&self) -> usize {
        let me = if !self.done { 1 } else { 0 };
        me + self.children.iter().map(|c| c.pending_count()).sum::<usize>()
    }

    pub fn done_count(&self) -> usize {
        let me = if self.done { 1 } else { 0 };
        me + self.children.iter().map(|c| c.done_count()).sum::<usize>()
    }

    pub fn pinned_count(&self) -> usize {
        let me = if self.pinned && !self.done { 1 } else { 0 };
        me + self.children.iter().map(|c| c.pinned_count()).sum::<usize>()
    }

    pub fn collect_tags(&self, map: &mut std::collections::HashMap<String, usize>) {
        if !self.tag.is_empty() {
            *map.entry(self.tag.clone()).or_insert(0) += 1;
        }
        for child in &self.children {
            child.collect_tags(map);
        }
    }

    pub fn sort(&mut self) {
        let now = now_secs();
        self.children.sort_by(|a, b| {
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
        for child in &mut self.children {
            child.sort();
        }
    }

    pub fn rebuild_depth(&mut self, current_depth: usize) {
        self.depth = current_depth;
        for child in &mut self.children {
            child.rebuild_depth(current_depth + 1);
        }
    }
}