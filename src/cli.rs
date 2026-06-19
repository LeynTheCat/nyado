use anyhow::{anyhow, bail, Result};
use std::env;
use std::path::PathBuf;

use crate::storage::{get_data_dir, Storage};
use crate::todo::now_secs;

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
    println!("    --export <project> <format> [--output <path>]  Export tasks to CSV or JSON");
    println!("    --random-task [project]          Show a random pending task from project");
    println!("    --today [project]                Show today's plan (due today, overdue, etc.)");
    println!("    --help, -h                       Show this help");
    println!("    --version, -V                    Show version");
    println!("");
    println!("Examples:");
    println!("    nyado --create-project work");
    println!("    nyado --project-name work --create-task \"Write report\" --tag important");
    println!("    nyado --list-tasks --done");
    println!("    nyado --export default csv --output /tmp/tasks.csv");
    println!("    nyado --random-task work");
    println!("    nyado --today");
}

pub fn run_cli(args: &[String]) -> Result<()> {
    if args.len() >= 2 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            _ => {}
        }
    }

    let mut current_project = "default".to_string();
    let mut command: Option<&str> = None;
    let mut tag = String::new();
    let mut filter_done = None;
    let mut filter_pending = None;
    let mut filter_pinned = None;

    let mut export_project = String::new();
    let mut export_format = String::new();
    let mut export_output: Option<String> = None;
    let mut random_project: Option<String> = None;
    let mut today_project: Option<String> = None;

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
            "--export" => {
                command = Some("--export");
                if i + 2 < tokens.len() && !tokens[i+1].starts_with('-') && !tokens[i+2].starts_with('-') {
                    export_project = tokens[i+1].to_string();
                    export_format = tokens[i+2].to_string().to_lowercase();
                    i += 3;
                    while i < tokens.len() && tokens[i] == "--output" {
                        if i + 1 < tokens.len() {
                            export_output = Some(tokens[i+1].to_string());
                            i += 2;
                        } else {
                            bail!("--output requires a path");
                        }
                    }
                } else {
                    bail!("--export requires project and format (csv|json)");
                }
            }
            "--random-task" => {
                command = Some("--random-task");
                if i + 1 < tokens.len() && !tokens[i+1].starts_with('-') {
                    random_project = Some(tokens[i+1].to_string());
                    i += 2;
                } else {
                    random_project = None;
                    i += 1;
                }
            }
            "--today" => {
                command = Some("--today");
                if i + 1 < tokens.len() && !tokens[i+1].starts_with('-') {
                    today_project = Some(tokens[i+1].to_string());
                    i += 2;
                } else {
                    today_project = None;
                    i += 1;
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

    let command = match command {
        Some(cmd) => cmd,
        None => {
            eprintln!("No command specified. Use --help for usage.");
            std::process::exit(1);
        }
    };

    match command {
        "--export" => {
            if !storage.list_projects().contains(&export_project) {
                eprintln!("Project '{}' does not exist.", export_project);
                std::process::exit(1);
            }
            storage.set_project(&export_project);
            storage.load_current();
            let tasks = storage.list_tasks_flat(None, None, None);
            let output_path = if let Some(path) = export_output {
                PathBuf::from(&path)
            } else {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                data_dir.join(format!("export_{}_{}.{}", export_project, timestamp, export_format))
            };
            let result = match export_format.as_str() {
                "csv" => export_to_csv(&tasks, &output_path),
                "json" => export_to_json(&tasks, &output_path),
                _ => {
                    eprintln!("Unsupported format: {}. Use csv or json.", export_format);
                    std::process::exit(1);
                }
            };
            if let Err(e) = result {
                eprintln!("Export failed: {}", e);
                std::process::exit(1);
            }
            println!("Exported to: {}", output_path.display());
        }
        "--random-task" => {
            let project_name = random_project.as_deref().unwrap_or(&current_project);
            if !storage.list_projects().contains(&project_name.to_string()) {
                eprintln!("Project '{}' does not exist.", project_name);
                std::process::exit(1);
            }
            storage.set_project(project_name);
            storage.load_current();
            let pending_tasks: Vec<_> = storage.list_tasks_flat(Some(false), None, None).into_iter().map(|(_, t)| t).collect();
            if pending_tasks.is_empty() {
                println!("No pending tasks in project '{}'.", project_name);
                return Ok(());
            }
            use rand::Rng;
            let idx = rand::thread_rng().gen_range(0..pending_tasks.len());
            let task = pending_tasks[idx];
            let tag_str = if task.tag.is_empty() { "".to_string() } else { format!("#{} ", task.tag) };
            let due_str = if task.due_date > 0 {
                let dt = chrono::DateTime::from_timestamp(task.due_date as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "?".to_string());
                format!(" (due: {})", dt)
            } else {
                String::new()
            };
            println!("Random task from '{}':", project_name);
            println!("  {}{}{}", tag_str, task.text, due_str);
        }
        "--today" => {
            let project_name = today_project.as_deref().unwrap_or(&current_project);
            if !storage.list_projects().contains(&project_name.to_string()) {
                eprintln!("Project '{}' does not exist.", project_name);
                std::process::exit(1);
            }
            storage.set_project(project_name);
            storage.load_current();
            let now = now_secs();
            let start_of_today = now - (now % 86400);
            let end_of_today = start_of_today + 86400;
            let all_tasks = storage.list_tasks_flat(None, None, None);
            let mut due_today = Vec::new();
            let mut overdue = Vec::new();
            let mut pending_count = 0;
            for (_, task) in all_tasks {
                if !task.done {
                    pending_count += 1;
                    if task.due_date > 0 {
                        if task.due_date < start_of_today {
                            overdue.push(task);
                        } else if task.due_date < end_of_today {
                            due_today.push(task);
                        }
                    }
                }
            }
            println!("=== Today's plan for project '{}' ===", project_name);
            println!("Pending tasks total: {}", pending_count);
            if !overdue.is_empty() {
                println!("\n[OVERDUE]");
                for (i, task) in overdue.iter().enumerate() {
                    let due_date = chrono::DateTime::from_timestamp(task.due_date as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "?".to_string());
                    let tag_str = if task.tag.is_empty() { "".to_string() } else { format!("#{} ", task.tag) };
                    println!("  {}. {}{} (due: {})", i+1, tag_str, task.text, due_date);
                }
            }
            if !due_today.is_empty() {
                println!("\n[DUE TODAY]");
                for (i, task) in due_today.iter().enumerate() {
                    let tag_str = if task.tag.is_empty() { "".to_string() } else { format!("#{} ", task.tag) };
                    println!("  {}. {}{}", i+1, tag_str, task.text);
                }
            }
            if overdue.is_empty() && due_today.is_empty() {
                println!("\nNo tasks due today or overdue. Enjoy your day!");
            }
        }
        _ => {
            if !storage.set_project(&current_project) {
                eprintln!(
                    "Project '{}' does not exist. Use --list-projects to see available projects.",
                    current_project
                );
                std::process::exit(1);
            }
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
                    let text = tokens
                        .iter()
                        .skip(1)
                        .find(|&&t| t != "--tag" && !t.starts_with('-'))
                        .ok_or_else(|| anyhow!("Missing task text"))?;
                    let task_text = *text;
                    let duplicate = storage.list_tasks_flat(None, None, None).iter().any(|(_, t)| {
                        t.text == task_text && (tag.is_empty() && t.tag.is_empty() || t.tag == tag)
                    });
                    if duplicate {
                        eprintln!("Warning: A task with the same text and tag already exists.");
                    }
                    storage.add_task(task_text, &tag)?;
                    println!("Task added: {}", task_text);
                }
                "--delete-task" => {
                    let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
                    let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
                    let flat = storage.list_tasks_flat(None, None, None);
                    if idx == 0 || idx > flat.len() {
                        eprintln!("Invalid index. Task numbers are 1..{}", flat.len());
                        std::process::exit(1);
                    }
                    let (_, task) = flat[idx - 1];
                    let task_id = task.id;
                    let task_text = task.text.clone();
                    storage.remove_task(task_id)?;
                    println!("Task {} (\"{}\") deleted.", idx, task_text);
                }
                "--toggle-task" => {
                    let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
                    let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
                    let flat = storage.list_tasks_flat(None, None, None);
                    if idx == 0 || idx > flat.len() {
                        eprintln!("Invalid index. Task numbers are 1..{}", flat.len());
                        std::process::exit(1);
                    }
                    let (_, task) = flat[idx - 1];
                    let task_id = task.id;
                    let new_state = !task.done;
                    storage.toggle_task(task_id)?;
                    println!(
                        "Task {} toggled to {}.",
                        idx,
                        if new_state { "done" } else { "pending" }
                    );
                }
                "--pin-task" => {
                    let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
                    let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
                    let flat = storage.list_tasks_flat(None, None, None);
                    if idx == 0 || idx > flat.len() {
                        eprintln!("Invalid index. Task numbers are 1..{}", flat.len());
                        std::process::exit(1);
                    }
                    let (_, task) = flat[idx - 1];
                    let task_id = task.id;
                    if task.pinned {
                        println!("Task {} is already pinned.", idx);
                    } else {
                        storage.pin_task(task_id)?;
                        println!("Task {} pinned.", idx);
                    }
                }
                "--unpin-task" => {
                    let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
                    let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
                    let flat = storage.list_tasks_flat(None, None, None);
                    if idx == 0 || idx > flat.len() {
                        eprintln!("Invalid index. Task numbers are 1..{}", flat.len());
                        std::process::exit(1);
                    }
                    let (_, task) = flat[idx - 1];
                    let task_id = task.id;
                    if !task.pinned {
                        println!("Task {} is not pinned.", idx);
                    } else {
                        storage.unpin_task(task_id)?;
                        println!("Task {} unpinned.", idx);
                    }
                }
                "--set-due" => {
                    let idx_str = tokens.get(1).ok_or_else(|| anyhow!("Missing task index"))?;
                    let idx: usize = idx_str.parse().map_err(|_| anyhow!("Invalid index"))?;
                    let date_str = tokens.get(2).ok_or_else(|| anyhow!("Missing date (YYYY-MM-DD)"))?;
                    let flat = storage.list_tasks_flat(None, None, None);
                    if idx == 0 || idx > flat.len() {
                        eprintln!("Invalid index. Task numbers are 1..{}", flat.len());
                        std::process::exit(1);
                    }
                    let (_, task) = flat[idx - 1];
                    let task_id = task.id;
                    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .map_err(|_| anyhow!("Invalid date format, use YYYY-MM-DD"))?;
                    let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u64;
                    storage.set_due_date(task_id, timestamp)?;
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
                    let tasks = storage.list_tasks_flat(filter_done_flag, filter_pinned, filter_tag_str);
                    if tasks.is_empty() {
                        println!("No tasks match the filters.");
                    } else {
                        for (i, todo) in tasks {
                            let status = if todo.done { "[✓]" } else { "[ ]" };
                            let pin = if todo.pinned { "* " } else { "" };
                            let tag_str = if todo.tag.is_empty() {
                                "".to_string()
                            } else {
                                format!(" #{}", todo.tag)
                            };
                            println!("{}. {}{} {}{}", i, pin, status, todo.text, tag_str);
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
        }
    }

    Ok(())
}

fn export_to_csv(tasks: &[(usize, &crate::todo::Todo)], path: &PathBuf) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&["id", "done", "pinned", "tag", "text", "created_at", "done_at", "due_date"])?;
    for (_, t) in tasks {
        wtr.write_record(&[
            t.id.to_string(),
            t.done.to_string(),
            t.pinned.to_string(),
            t.tag.clone(),
            t.text.clone(),
            t.created_at.to_string(),
            t.done_at.to_string(),
            t.due_date.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn export_to_json(tasks: &[(usize, &crate::todo::Todo)], path: &PathBuf) -> Result<()> {
    let tasks_vec: Vec<&crate::todo::Todo> = tasks.iter().map(|(_, t)| *t).collect();
    let json = serde_json::to_string_pretty(&tasks_vec)?;
    std::fs::write(path, json)?;
    Ok(())
}