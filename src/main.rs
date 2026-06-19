mod app;
mod handlers;
mod commands;
mod i18n;
mod popup;
mod storage;
mod todo;
mod ui;
mod cli;
mod config;

use anyhow::Result;
use single_instance::SingleInstance;
use std::env;
use std::fs::File;
use std::io::Write;
use std::panic;
use storage::get_data_dir;

fn setup_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
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
        log.push_str(&format!(
            "Panic occurred at: {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        log.push_str("Panic details:\n");

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            log.push_str(&format!("  message: {}\n", s));
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            log.push_str(&format!("  message: {}\n", s));
        } else {
            log.push_str("  message: (unknown)\n");
        }

        if let Some(location) = panic_info.location() {
            log.push_str(&format!(
                "  location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        }

        let backtrace = std::backtrace::Backtrace::force_capture();
        log.push_str(&format!("  backtrace:\n{:?}\n", backtrace));

        if let Some(file) = &mut file {
            let _ = file.write_all(log.as_bytes());
            eprintln!("nyado crashed! Crash log saved to: {}", log_path.display());
        } else {
            eprintln!(
                "nyado crashed! Could not write crash log to {}",
                log_path.display()
            );
            eprintln!("{}", log);
        }

        original_hook(panic_info);
    }));
}

fn main() -> Result<()> {
    setup_panic_handler();
    config::init_config();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        return cli::run_cli(&args);
    }

    let instance = SingleInstance::new("nyado")?;
    if !instance.is_single() {
        eprintln!("nyado is already running!");
        return Ok(());
    }
    app::run()
}