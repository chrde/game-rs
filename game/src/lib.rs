#[path = "../../src/host_api.rs"]
mod host_api;
mod render;

use render::Color;
pub use render::OffscreenBuffer;

use host_api::*;

#[no_mangle]
pub extern "C" fn game_init(_host_api: &dyn HostApi) -> *mut GameState {
    let width = 1920;
    let height = 1080;
    let bytes_per_pixel = 4;
    let offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; width * height * bytes_per_pixel],
        width,
        height,
        bytes_per_pixel,
    };
    let game = GameState {
        offscreen_buffer,
        player_x: 0.0,
        player_y: 0.0,
    };
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_update(state: &mut GameState, input: &Input, host_api: &mut dyn HostApi) -> bool {
    let tile_map = [
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    let tile_width = 60.0;
    let tile_height = 60.0;
    let offset_x = 0.0;
    let offset_y = 0.0;
    let ob = &mut state.offscreen_buffer;
    for row in 0..tile_map.len() {
        for column in 0..tile_map[0].len() {
            let tile_id = tile_map[row][column];
            let color = if tile_id == 1 { 0.5 } else { 1.0 };
            let minx = offset_x + column as f32 * tile_width;
            let miny = offset_y + row as f32 * tile_height;
            let maxx = minx + tile_width;
            let maxy = miny + tile_height;
            let color = Color {
                red: color,
                green: color,
                blue: color,
            };
            ob.render_rectangle(minx, miny, maxx, maxy, color);
        }
    }

    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        let player_width = 0.75 * tile_width;
        let player_height = tile_height;
        let mut player_x_offset = 0.0;
        let mut player_y_offset = 0.0;
        if input.new.up {
            player_y_offset -= 1.0
        };
        if input.new.down {
            player_y_offset += 1.0
        };

        if input.new.right {
            player_x_offset += 1.0
        };
        if input.new.left {
            player_x_offset -= 1.0
        };

        state.player_x += input.time_per_frame * 64.0 * player_x_offset;
        state.player_y += input.time_per_frame * 64.0 * player_y_offset;

        let player_left = state.player_x - 0.5 * player_width;
        let player_top = state.player_y - player_height;
        // host_api.println(&format!(
        //     "{} {} {} {}",
        //     player_left,
        //     player_top,
        //     player_left + player_width,
        //     player_top + player_height
        // ));
        ob.render_rectangle(
            player_left,
            player_top,
            player_left + player_width,
            player_top + player_height,
            player_color,
        );
    }

    host_api.update_canvas(&ob.buffer, ob.pitch());
    // host_api.generate_audio();
    true
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub player_x: f32,
    pub player_y: f32,
}
