use crate::storage::get_config_dir;
use crate::ui::common::color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Config {
    pub max_todos: usize,
    pub max_backups: usize,
    pub max_projects: usize,
    pub max_depth: usize,
    pub show_calendar: bool,
    pub celebration_duration_frames: u8,
    pub celebration_blink_interval: u8,
    pub celebration_frame_delay_ms: u64,
}


/// Creates a default configuration with predefined limits and settings.
///
/// # Returns
///
/// A new `Config` instance with default values for:
/// - `max_todos`, `max_backups`, `max_projects`, `max_depth`,
/// - `show_calendar`, and celebration-related timing parameters.
impl Default for Config {
    fn default() -> Self {
        Self {
            max_todos: 2048,
            max_backups: 5,
            max_projects: 64,
            max_depth: 2,
            show_calendar: true,
            celebration_duration_frames: 30,
            celebration_blink_interval: 4,
            celebration_frame_delay_ms: 80,
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        get_config_dir().join("config.yaml")
    }

    pub fn load_or_create() -> Self {
        let path = Self::config_path();
        let default = Config::default();
        let mut cfg = default.clone();

        if let Ok(content) = fs::read_to_string(&path) {
            match serde_yaml::from_str::<Config>(&content) {
                Ok(parsed) => {
                    cfg.max_todos = parsed.max_todos;
                    cfg.max_backups = parsed.max_backups;
                    cfg.max_projects = parsed.max_projects;
                    cfg.max_depth = parsed.max_depth;
                    cfg.show_calendar = parsed.show_calendar;
                    cfg.celebration_duration_frames = parsed.celebration_duration_frames;
                    cfg.celebration_blink_interval = parsed.celebration_blink_interval;
                    cfg.celebration_frame_delay_ms = parsed.celebration_frame_delay_ms;
                }
                Err(e) => {
                    panic!("Failed to parse config file {}: {}\nPlease fix or delete the config file.", path.display(), e);
                }
            }
        }

        let _ = fs::create_dir_all(get_config_dir());
        if let Ok(yaml) = serde_yaml::to_string(&cfg) {
            if let Err(e) = fs::write(&path, yaml) {
                panic!("Failed to write config file {}: {}", path.display(), e);
            }
        } else {
            panic!("Failed to serialize default config");
        }
        cfg
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() {
    color::init_color_mode();
    CONFIG.set(Config::load_or_create()).unwrap();
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}