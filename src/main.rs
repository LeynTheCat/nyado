mod app;
mod commands;
mod i18n;
mod popup;
mod storage;
mod todo;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}