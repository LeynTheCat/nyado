mod app;
mod commands;
mod i18n;
mod popup;
mod storage;
mod todo;
mod ui;

use anyhow::Result;
use single_instance::SingleInstance;

fn main() -> Result<()> {
    let instance = SingleInstance::new("nyado")?;
    if !instance.is_single() {
        eprintln!("nyado is already running!");
        return Ok(());
    }
    app::run()
}