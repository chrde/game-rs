// use engine::*;
use libloading as lib;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use super::host_api::*;
// use std::time::Duration;

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

pub struct GameLib {
    lib: lib::Library,
}

impl GameLib {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let lib = lib::Library::new(LIBGAME)?;
        Ok(Self { lib })
    }

    pub fn reload(self) -> Result<Self, Box<dyn std::error::Error>> {
        std::mem::drop(self);
        Ok(Self {
            lib: lib::Library::new(LIBGAME)?,
        })
    }

    pub fn api(&mut self) -> Result<GameApi<'_>, Box<dyn std::error::Error>> {
        unsafe {
            let init = self.lib.get(b"game_init")?;
            let update = self.lib.get(b"game_update")?;
            Ok(GameApi { init, update })
        }
    }
}

#[repr(C)]
pub struct GameState {
    _private: [u8; 0],
}

pub struct GameApi<'lib> {
    /// Called on game start
    pub init: lib::Symbol<'lib, fn(&dyn HostApi) -> *mut GameState>,

    /// Called on game loop. Returns `true` if the game continues running
    pub update: lib::Symbol<'lib, fn(*mut GameState, &Input, &mut dyn HostApi) -> bool>,
    // // Called on game exit
    // pub shutdown: lib::Symbol<'lib, fn(*mut GameState)>,

    // // Called on game unload
    // pub unload: lib::Symbol<'lib, fn(*mut GameState)>,

    // // Called on game reload
    // pub reload: lib::Symbol<'lib, fn(*mut GameState)>,
}

