use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug, Clone)]
pub struct Todo {
    pub done: bool,
    pub pinned: bool,
    pub tag: String,
    pub text: String,
    pub created_at: u64,
    pub done_at: u64,
}

impl Todo {
    pub fn new(text: &str, tag: &str) -> Self {
        Self {
            done: false,
            pinned: false,
            tag: tag.to_string(),
            text: text.to_string(),
            created_at: now_secs(),
            done_at: 0,
        }
    }

    pub fn to_line(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}\n",
            if self.pinned { 'P' } else { '-' },
            if self.done { 'x' } else { ' ' },
            if self.tag.is_empty() { "none" } else { &self.tag },
            self.text,
            self.created_at,
            self.done_at
        )
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 6 {
            return None;
        }
        let pinned = parts[0] == "P";
        let done = parts[1] == "x";
        let tag = if parts[2] == "none" {
            String::new()
        } else {
            parts[2].to_string()
        };
        let text = parts[3].to_string();
        let created_at = parts[4].parse().unwrap_or(0);
        let done_at = parts[5].parse().unwrap_or(0);
        Some(Self {
            done,
            pinned,
            tag,
            text,
            created_at,
            done_at,
        })
    }
}