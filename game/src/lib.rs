#[path="../../src/host_api.rs"]
mod host_api;
mod render;

pub use render::OffscreenBuffer;

use host_api::HostApi;

#[no_mangle]
pub extern "C" fn game_init(host_api: &dyn HostApi) -> *mut GameState {
    let width = 800;
    let height = 600;
    let bytes_per_pixel = 4;
    let offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; width * height * bytes_per_pixel],
        width,
        height,
        bytes_per_pixel,
    };
    host_api.print("hola");
    let game = GameState {
        offscreen_buffer,
        blue_offset: 0,
        green_offset: 0,
    };
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_update(game_state: &mut GameState, host_api: &mut dyn HostApi) -> bool {
    game_state.blue_offset+=1;
    game_state.green_offset+=1;
    let ob = &mut game_state.offscreen_buffer;
    let height = ob.buffer.len() / 3200;
    for y in 0..height {
        for x in 0..ob.width {
            // B G R A
            let offset = y * ob.pitch() + ob.bytes_per_pixel * x;
            // ob.buffer[offset + 0] = (x + game_state.blue_offset) as u8;
            // ob.buffer[offset + 1] = (y + game_state.green_offset) as u8;
            ob.buffer[offset + 0] = 0;
            ob.buffer[offset + 1] = 0;
            ob.buffer[offset + 2] = 0;
            ob.buffer[offset + 3] = 0;
        }
    }
    ob.render_rectangle(10.0, 10.0, 100.0, 100.0, true);
    ob.render_rectangle(10.0, 101.0, 100.0, 200.0, false);
    host_api.update_canvas(&ob.buffer, ob.pitch());
    // host_api.generate_audio();
    true
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub blue_offset: usize,
    pub green_offset: usize,
}
