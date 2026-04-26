mod app;
mod todo;
mod ui;
mod storage;
mod popup;
mod i18n;
mod commands;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}