use crate::app::App;
use crate::popup::{popup_with_mode, PopupMode};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn set(app: &mut App, term: &mut Terminal<CrosstermBackend<io::Stdout>>) {
    if !app.visible.is_empty() {
        let idx = app.visible[app.selected];
        let date_title = app.i18n.get("popup_due_date_title");
        let date_hint = app.i18n.get("popup_due_date_hint");
        if let Ok(Some(date_str)) = popup_with_mode(date_title, date_hint, "", PopupMode::Singleline, term) {
            let trimmed_date = date_str.trim();
            if trimmed_date.is_empty() {
                app.storage.todos[idx].due_date = 0;
                let msg = app.i18n.get("due_date_cleared").to_string();
                app.set_message(&msg);
                app.sort_todos();
                app.storage.save();
                return;
            }
            let time_hint = app.i18n.get("popup_due_time_hint");
            let time_res = popup_with_mode("", time_hint, "", PopupMode::Singleline, term);
            let time_str = match time_res {
                Ok(Some(t)) => t.trim().to_string(),
                Ok(None) => return,
                Err(_) => {
                    let msg = app.i18n.get("due_date_invalid").to_string();
                    app.set_message(&msg);
                    return;
                }
            };
            if let Some(timestamp) = parse_datetime(trimmed_date, &time_str) {
                app.storage.todos[idx].due_date = timestamp;
                let display = if time_str.is_empty() {
                    format!("{} {}", app.i18n.get("due_date_set"), trimmed_date)
                } else {
                    format!("{} {} {}", app.i18n.get("due_date_set"), trimmed_date, time_str)
                };
                app.set_message(&display);
            } else {
                let msg = app.i18n.get("due_date_invalid").to_string();
                app.set_message(&msg);
                return;
            }
            app.sort_todos();
            app.storage.save();
        }
    }
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