#[path="../../src/host_api.rs"]
mod host_api;

use host_api::HostApi;

#[no_mangle]
pub extern "C" fn game_init(host_api: &dyn HostApi) -> *mut GameState {
    let width = 800;
    let pixel_format = 4;
    let offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; width * 600 * pixel_format],
        width,
        pixel_format,
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
            let offset = y * ob.pitch() + ob.pixel_format * x;
            ob.buffer[offset + 0] = (x + game_state.blue_offset) as u8;
            ob.buffer[offset + 1] = (y + game_state.green_offset) as u8;
            ob.buffer[offset + 2] = 0;
            ob.buffer[offset + 3] = 0;
        }
    }
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

#[repr(C)]
pub struct OffscreenBuffer {
    pub buffer: Vec<u8>,
    width: usize,
    pixel_format: usize,
}

impl OffscreenBuffer {
    fn pitch(&self) -> usize {
        self.width * self.pixel_format
    }
}
