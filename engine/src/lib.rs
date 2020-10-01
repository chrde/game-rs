// use std::marker::PhantomData;

// #[repr(C)]
// pub struct GameState<'a> {
//     phantom: PhantomData<&'a i32>,
//     pub offscreen_buffer: OffscreenBuffer,
//     pub blue_offset: usize,
//     pub green_offset: usize,
// }

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub blue_offset: usize,
    pub green_offset: usize,
}

#[repr(C)]
pub struct OffscreenBuffer {
    pub buffer: Vec<u8>,
}
