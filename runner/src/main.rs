use libloading as lib;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rx, _watcher) = runner::reloader::run()?;

    runner::game_loop::main(rx)?;

    Ok(())
}
