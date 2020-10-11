fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rx, _watcher) = engine::reloader::run()?;

    engine::game_loop::main(rx)?;

    Ok(())
}

