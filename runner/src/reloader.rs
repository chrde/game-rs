use engine::*;
use libloading as lib;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::Duration;

const LIBGAME: &str = "./target/release/libgame.so";

pub fn run() -> Result<(Receiver<()>, RecommendedWatcher), Box<dyn std::error::Error>> {
    let libgame = Path::new(LIBGAME).canonicalize().unwrap();
    let path = libgame.parent().unwrap().to_owned();

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new_immediate(move |res: Result<notify::Event, _>| match res {
            Ok(event) => {
                if let notify::EventKind::Create(_) = event.kind {
                    if event.paths.iter().any(|x| x == &libgame) {
                        // signal that we need to reload
                        tx.send(()).unwrap();
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        })?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    Ok((rx, watcher))
}

pub struct Game {
    pub api: GameApi,
    lib: lib::Library,
}

impl Game {
    pub fn reload() -> Result<Self, Box<dyn std::error::Error>> {
        let lib = lib::Library::new(LIBGAME)?;
        let api: Result<_, Box<dyn std::error::Error>> = unsafe {
            let init = *(lib.get(b"game_init")?);
            let update = *(lib.get(b"game_update")?);
            Ok(GameApi { init, update })
        };
        Ok(Self { api: api?, lib })
    }
}

// #[repr(C)]
// pub struct GameState {
//     _private: [u8; 0],
// }

pub struct GameApi {
    /// Called on game start
    pub init: unsafe extern "C" fn(u8) -> *mut engine::GameState,

    /// Called on game loop. Returns `true` if the game continues running
    pub update: unsafe extern "C" fn(*mut GameState) -> bool,
    // // Called on game exit
    // pub shutdown: lib::Symbol<'lib, fn(*mut GameState)>,

    // // Called on game unload
    // pub unload: lib::Symbol<'lib, fn(*mut GameState)>,

    // // Called on game reload
    // pub reload: lib::Symbol<'lib, fn(*mut GameState)>,
}
