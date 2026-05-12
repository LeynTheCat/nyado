mod app;
mod handlers;
mod commands;
mod i18n;
mod popup;
mod storage;
mod todo;
mod ui;

use anyhow::{Result, bail, anyhow};
use single_instance::SingleInstance;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::panic;
use storage::{Storage, get_data_dir};

fn setup_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        let _ = crossterm::terminal::disable_raw_mode();

        let version = env!("CARGO_PKG_VERSION");
        let data_dir = get_data_dir();
        let log_path = data_dir.join("nyado_crash.log");
        let _ = std::fs::create_dir_all(&data_dir);
        let mut file = File::create(&log_path).ok();
        let mut log = String::new();

        log.push_str(&format!("nyado version: {}\n", version));
        log.push_str(&format!("Panic occurred at: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        log.push_str("Panic details:\n");

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            log.push_str(&format!("  message: {}\n", s));
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            log.push_str(&format!("  message: {}\n", s));
        } else {
            log.push_str("  message: (unknown)\n");
        }

        if let Some(location) = panic_info.location() {
            log.push_str(&format!("  location: {}:{}:{}\n", location.file(), location.line(), location.column()));
        }

        let backtrace = std::backtrace::Backtrace::force_capture();
        log.push_str(&format!("  backtrace:\n{:?}\n", backtrace));

        if let Some(file) = &mut file {
            let _ = file.write_all(log.as_bytes());
            eprintln!("nyado crashed! Crash log saved to: {}", log_path.display());
        } else {
            eprintln!("nyado crashed! Could not write crash log to {}", log_path.display());
            eprintln!("{}", log);
        }

        original_hook(panic_info);
    }));
}

fn print_version() {
    println!("nyado {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    println!("nyado - a TUI todo-list manager");
    println!("");
    println!("USAGE:");
    println!("    nyado [COMMAND] [OPTIONS]");
    println!("    nyado                (start TUI)");
    println!("");
    println!("COMMANDS (CLI):");
    println!("    --create-project <name>          Create new project");
    println!("    --delete-project <name>          Delete project (except default)");
    println!("    --rename-project <old> <new>     Rename project");
    println!("    --list-projects                  List all projects");
    println!("    --project-name <name>            Set current project for subsequent task commands (default: default)");
    println!("    --create-task <text> [--tag <tag>]  Add a task");
    println!("    --delete-task <index>            Delete task by number (1-based)");
    println!("    --toggle-task <index>            Mark task as done/undone");
    println!("    --pin-task <index>               Pin task");
    println!("    --unpin-task <index>             Unpin task");
    println!("    --set-due <index> <YYYY-MM-DD>   Set due date");
    println!("    --list-tasks                     Show all tasks of current project");
    println!("       [--done] [--pending] [--pinned] [--tag <tag>]  Filter tasks");
    println!("    --stats                          Show task statistics (total/done/pending/pinned)");
    println!("    --done-percents                  Show percentage of completed tasks");
    println!("    --help, -h                       Show this help");
    println!("    --version, -V                    Show version");
    println!("");
    println!("Examples:");
    println!("    nyado --create-project work");
    println!("    nyado --project-name work --create-task \"Write report\" --tag important");
    println!("    nyado --list-tasks --done");
}

fn main() -> Result<()> {
    setup_panic_handler();

    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 2 {
        match args[1].as_str() {
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }
    
    if args.len() == 1 {
        let instance = SingleInstance::new("nyado")?;
        if !instance.is_single() {
            eprintln!("nyado is already running!");
            return Ok(());
        }
        return app::run();
    }

    run_cli(&args)
}

fn run_cli(args: &[String]) -> Result<()> {
    let mut current_project = "default".to_string();
    let mut command: Option<&str> = None;
    let mut tag = String::new();
    let mut filter_done = None;
    let mut filter_pending = None;
    let mut filter_pinned = None;

    let tokens: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "--project-name" => {
                if i + 1 < tokens.len() {
                    current_project = tokens[i + 1].to_string();
                    i += 2;
                } else {
                    bail!("--project-name requires argument");
                }
            }
            "--create-project" | "--delete-project" | "--rename-project" | "--list-projects"
            | "--create-task" | "--delete-task" | "--toggle-task" | "--pin-task" | "--unpin-task"
            | "--set-due" | "--list-tasks" | "--stats" | "--done-percents" => {
                if command.is_none() {
                    command = Some(tokens[i]);
                    i += 1;
                } else {
                    bail!("Only one command allowed at a time");
                }
            }
            "--tag" => {
                if i + 1 < tokens.len() {
                    tag = tokens[i + 1].to_string();
                    i += 2;
                } else {
                    bail!("--tag requires argument");
                }
            }
            "--done" => {
                filter_done = Some(true);
                i += 1;
            }
            "--pending" => {
                filter_pending = Some(true);
                i += 1;
            }
            "--pinned" => {
                filter_pinned = Some(true);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    let data_dir = get_data_dir();
    let projects_dir = data_dir.join("projects");
    std::fs::create_dir_all(&projects_dir)?;

    let mut storage = Storage::new(projects_dir.clone());
    if !storage.set_project(&current_project) {
        eprintln!("Project '{}' does not exist. Use --list-projects to see available projects.", current_project);
        std::process::exit(1);
    }

    let command = match command {
        Some(cmd) => cmd,
        None => {
            eprintln!("No command specified. Use --help for usage.");
            std::process::exit(1);
        }
    };

    match command {
        "--create-project" => {
            let name = tokens.get(1).ok_or_else(|| anyhow!("Missing project name"))?;
            if storage.list_projects().contains(&name.to_string()) {
                eprintln!("Project '{}' already exists.", name);
                std::process::exit(1);
            }
            if storage.create_project(name) {
                println!("Project '{}' created.", name);
            } else {
                eprintln!("Failed to create project '{}' (maybe limit reached).", name);
                std::process::exit(1);
            }
        }
        "--delete-project" => {
            let name = tokens.get(1).ok_or_else(|| anyhow!("Missing project name"))?;
            if !storage.list_projects().contains(&name.to_string()) {
                eprintln!("Project '{}' does not exist.", name);
                std::process::exit(1);
            }
            if storage.delete_project(name) {
                println!("Project '{}' deleted.", name);
            } else {
                eprintln!("Cannot delete project '{}' (default or already deleted).", name);
                std::process::exit(1);
            }
        }
        "--rename-project" => {
            let old = tokens.get(1).ok_or_else(|| anyhow!("Missing old name"))?;
            let new = tokens.get(2).ok_or_else(|| anyhow!("Missing new name"))?;
            if old == new {
                eprintln!("Old and new names are the same.");
                std::process::exit(1);
            }
            let projects = storage.list_projects();
            if !projects.contains(&old.to_string()) {
                eprintln!("Project '{}' does not exist.", old);
                std::process::exit(1);
            }
            if projects.contains(&new.to_string()) {
                eprintln!("Project '{}' already exists.", new);
                std::process::exit(1);
            }
            let mut proj_storage = Storage::new(projects_dir);
            if proj_storage.rename_project(old, new) {
                println!("Project renamed from '{}' to '{}'.", old, new);
            } else {
                eprintln!("Rename failed.");
                std::process::exit(1);
            }
        }
        "--list-projects" => {
            let projects = storage.list_projects();
            println!("Projects:");
            for p in projects {
                println!("  {}", p);
            }
        }
        "--create-task" => {
            let text = tokens.iter().skip(1).find(|&&t| t != "--tag" && !t.starts_with('-')).ok_or_else(|| anyhow!("Missing task text"))?;
            let task_text = *text;
            let duplicate = storage.todos.iter().any(|t| t.text == task_text && (tag.is_empty() && t.tag.is_empty() || t.tag == tag));
            if duplicate {
                eprintln!("Warning: A task with the same text and tag already exists.");
            }
            storage.add_task(task_text, &tag)?;
            println!("Task added: {}", task_text);
        }
        "--delete-task" => {
            let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
            let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
            if idx == 0 || idx > storage.todos.len() {
                eprintln!("Invalid index. Task numbers are 1..{}", storage.todos.len());
                std::process::exit(1);
            }
            let task_text = storage.todos[idx-1].text.clone();
            storage.remove_task(idx)?;
            println!("Task {} (\"{}\") deleted.", idx, task_text);
        }
        "--toggle-task" => {
            let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
            let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
            if idx == 0 || idx > storage.todos.len() {
                eprintln!("Invalid index. Task numbers are 1..{}", storage.todos.len());
                std::process::exit(1);
            }
            let new_state = !storage.todos[idx-1].done;
            storage.toggle_task(idx)?;
            println!("Task {} toggled to {}.", idx, if new_state { "done" } else { "pending" });
        }
        "--pin-task" => {
            let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
            let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
            if idx == 0 || idx > storage.todos.len() {
                eprintln!("Invalid index. Task numbers are 1..{}", storage.todos.len());
                std::process::exit(1);
            }
            if storage.todos[idx-1].pinned {
                println!("Task {} is already pinned.", idx);
            } else {
                storage.pin_task(idx)?;
                println!("Task {} pinned.", idx);
            }
        }
        "--unpin-task" => {
            let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
            let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
            if idx == 0 || idx > storage.todos.len() {
                eprintln!("Invalid index. Task numbers are 1..{}", storage.todos.len());
                std::process::exit(1);
            }
            if !storage.todos[idx-1].pinned {
                println!("Task {} is not pinned.", idx);
            } else {
                storage.unpin_task(idx)?;
                println!("Task {} unpinned.", idx);
            }
        }
        "--set-due" => {
            let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
            let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
            let date_str = tokens.get(2).ok_or_else(|| anyhow!("Missing date (YYYY-MM-DD)"))?;
            if idx == 0 || idx > storage.todos.len() {
                eprintln!("Invalid index. Task numbers are 1..{}", storage.todos.len());
                std::process::exit(1);
            }
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| anyhow!("Invalid date format, use YYYY-MM-DD"))?;
            let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64;
            storage.set_due_date(idx, timestamp)?;
            println!("Due date for task {} set to {}.", idx, date_str);
        }
        "--list-tasks" => {
            let filter_done_flag = if filter_done == Some(true) {
                Some(true)
            } else if filter_pending == Some(true) {
                Some(false)
            } else {
                None
            };
            let filter_tag_str = if !tag.is_empty() { Some(tag.as_str()) } else { None };
            let tasks = storage.list_tasks_filtered(filter_done_flag, filter_pinned, filter_tag_str);
            if tasks.is_empty() {
                println!("No tasks match the filters.");
            } else {
                for (i, todo) in tasks {
                    let status = if todo.done { "[✓]" } else { "[ ]" };
                    let pin = if todo.pinned { "📌 " } else { "" };
                    let tag_str = if todo.tag.is_empty() { "" } else { &format!(" #{}", todo.tag) };
                    println!("{}. {}{} {}{}", i+1, pin, status, todo.text, tag_str);
                }
            }
        }
        "--stats" => {
            let (total, done, pending, pinned) = storage.get_stats();
            println!("Project: {}", storage.current_project);
            println!("Total tasks:   {}", total);
            println!("Done:          {}", done);
            println!("Pending:       {}", pending);
            println!("Pinned:        {}", pinned);
        }
        "--done-percents" => {
            let percent = storage.done_percent();
            println!("Completed: {:.1}%", percent);
        }
        _ => {
            eprintln!("Unknown command. Use --help.");
            std::process::exit(1);
        }
    }

    Ok(())
}