// use std::marker::PhantomData;

// #[repr(C)]
// pub struct GameState<'a> {
//     phantom: PhantomData<&'a i32>,
//     pub offscreen_buffer: OffscreenBuffer,
//     pub blue_offset: usize,
//     pub green_offset: usize,
// }

#[path="../../src/host_api.rs"]
mod host_api;

pub mod game_loop;
pub mod reloader;
