mod app;
mod commands;
mod i18n;
mod popup;
mod storage;
mod todo;
mod ui;

use anyhow::Result;
use single_instance::SingleInstance;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::panic;
use crate::storage::get_data_dir;

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
            eprintln!("Please include this file when reporting an issue on GitHub");
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
    println!("    nyado [OPTIONS]");
    println!("");
    println!("OPTIONS:");
    println!("    -V, --version    Print version information and exit");
    println!("    -h, --help       Print this help message and exit");
    println!("");
    println!("For key bindings inside the TUI, press ? or h.");
}

fn main() -> Result<()> {
    setup_panic_handler();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
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

    let instance = SingleInstance::new("nyado")?;
    if !instance.is_single() {
        eprintln!("nyado is already running!");
        return Ok(());
    }
    app::run()
}