#[path = "../../src/host_api.rs"]
mod host_api;
mod render;
mod tile;

use render::Color;
pub use render::OffscreenBuffer;

// use tile::*;
use host_api::*;
use tile::*;

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
    let player = TileMapPosition::initial_player();
    let game = GameState {
        tile_map: TileMap::new(),
        offscreen_buffer,
        player,
    };
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_update(
    state: &mut GameState,
    input: &Input,
    host_api: &mut dyn HostApi,
) -> bool {
    let tile_side_in_pixels: f32 = 60.0;
    let meters_to_pixels = tile_side_in_pixels as f32 / state.tile_map.tile_size.0;

    let lower_x = -(tile_side_in_pixels as f32 / 2.0);
    let lower_y = state.offscreen_buffer.height;
    let screen_center_x = 0.5 * (state.offscreen_buffer.width as f32);
    let screen_center_y = 0.5 * (state.offscreen_buffer.height as f32);
    let player_height = 1.4;
    let player_width = 0.75 * player_height;
    state.offscreen_buffer.reset();

    // let tile_width = 60.0;
    // let tile_height = 60.0;
    // let offset_x = 0.0;
    // let offset_y = 0.0;
    // let ob = &mut state.offscreen_buffer;
    // for row in 0..tile_map.len() {
    //     for column in 0..tile_map[0].len() {
    //         let tile_id = tile_map[row][column];
    //         let color = if tile_id == 1 { 0.5 } else { 1.0 };
    //         let minx = offset_x + column as f32 * tile_width;
    //         let miny = offset_y + row as f32 * tile_height;
    //         let maxx = minx + tile_width;
    //         let maxy = miny + tile_height;
    //         let color = Color {
    //             red: color,
    //             green: color,
    //             blue: color,
    //         };
    //         ob.render_rectangle(minx, miny, maxx, maxy, color);
    //     }
    // }

    //move player
    {
        let player_offset = {
            let mut player_x_offset = 0.0;
            let mut player_y_offset = 0.0;
            if input.new.up {
                player_y_offset += 1.0
            };
            if input.new.down {
                player_y_offset -= 1.0
            };

            if input.new.right {
                player_x_offset += 1.0
            };
            if input.new.left {
                player_x_offset -= 1.0
            };

            let player_speed = 10.0;

            Offset {
                x: input.time_per_frame * player_x_offset * player_speed,
                y: input.time_per_frame * player_y_offset * player_speed,
            }
        };

        let new_player_pos = {
            let mut new_player_pos = state.player.clone();
            new_player_pos.offset.x += player_offset.x;
            new_player_pos.offset.y += player_offset.y;
            state.tile_map.recanonicalize_position(&mut new_player_pos);

            let mut new_left = new_player_pos.clone();
            new_left.offset.x -= 0.5 * player_width;
            state.tile_map.recanonicalize_position(&mut new_left);

            let mut new_right = new_player_pos.clone();
            new_right.offset.x += 0.5 * player_width;
            state.tile_map.recanonicalize_position(&mut new_right);

            if state.tile_map.is_tile_empty(new_player_pos)
                && state.tile_map.is_tile_empty(new_left)
                && state.tile_map.is_tile_empty(new_right)
            {
                // host_api.println(&format!("{:?}", new_player_pos));
                Some(new_player_pos)
            } else {
                None
            }
        };
        // host_api.println(&format!(
        //     "{} {} {} {}",
        //     player_left,
        //     player_top,
        //     player_left + player_width,
        //     player_top + player_height
        // ));
        // ob.render_rectangle(
        //     player_left,
        //     player_top,
        //     player_left + player_width,
        //     player_top + player_height,
        //     player_color,
        // );

        if let Some(new_player_pos) = new_player_pos {
            state.player = new_player_pos;
        }
    }

    //render map
    {
        for rel_row in -10..10 {
            for rel_column in -20..20 {
                let color = {
                    let column = state.player.chunk_position.x as i32 + rel_column;
                    let row = state.player.chunk_position.y as i32 + rel_row;
                    let pos = CompressedPosition {
                        x: column as usize,
                        y: row as usize,
                        z: 0,
                    };
                    if let Some(t) = state.tile_map.tile_from_compressed_pos(pos) {
                        let color = if pos == state.player.chunk_position {
                            0.25
                        } else {
                            match t.kind {
                                TileKind::Wall => 1.0,
                                TileKind::Ground => 0.5,
                                TileKind::Empty => continue,
                            }
                        };
                        let color = Color {
                            red: color,
                            green: color,
                            blue: color,
                        };
                        Some(color)
                    } else {
                        None
                    }
                };
                if let Some(color) = color {
                    let center_x = screen_center_x - meters_to_pixels * state.player.offset.x
                        + (rel_column as f32) * tile_side_in_pixels;
                    let center_y = screen_center_y + meters_to_pixels * state.player.offset.y
                        - (rel_row as f32) * tile_side_in_pixels;

                    let minx = center_x - 0.5 * tile_side_in_pixels;
                    let miny = center_y - 0.5 * tile_side_in_pixels;
                    let maxx = center_x + 0.5 * tile_side_in_pixels;
                    let maxy = center_y + 0.5 * tile_side_in_pixels;

                    state
                        .offscreen_buffer
                        .render_rectangle(minx, miny, maxx, maxy, color);
                }
            }
        }
        // panic!();
    }

    //render player
    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        // let ob = &mut state.offscreen_buffer;
        {
            let player_left = screen_center_x - 0.5 * player_width * meters_to_pixels;
            let player_top = screen_center_y - 0.5 * player_height * meters_to_pixels;

            state.offscreen_buffer.render_rectangle(
                player_left,
                player_top,
                player_left + meters_to_pixels * player_width,
                player_top + meters_to_pixels * player_height,
                player_color,
            );
        }
    }

    host_api.update_canvas(
        &state.offscreen_buffer.buffer,
        state.offscreen_buffer.pitch(),
    );
    // host_api.generate_audio();
    true
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub tile_map: TileMap,
    pub player: TileMapPosition,
}

impl GameState {
    // fn move_player_by(&mut self, offset: Offset) {
    //     let player = &mut self.player;
    //     player.offset.x += offset.x;
    //     player.offset.y += offset.y;
    //     self.tile_map.recanonicalize_position(player);
    // }
}
