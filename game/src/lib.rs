#[path = "../../src/host_api.rs"]
mod host_api;
mod render;
mod tile;

use render::Color;
pub use render::OffscreenBuffer;

use host_api::*;
use tile::*;

#[no_mangle]
pub extern "C" fn game_init(host_api: &dyn HostApi) -> *mut GameState {
    let width = 1920 / 2;
    let height = 1080 / 2;
    let bytes_per_pixel = 4;
    let offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; width * height * bytes_per_pixel],
        width,
        height,
        bytes_per_pixel,
    };
    let player = TileMapPosition::initial_camera();
    let camera = TileMapPosition::initial_camera();
    let game = GameState {
        tile_map: TileMap::new(),
        offscreen_buffer,
        player,
        camera,
        hero_direction: 1,
        backdrop: host_api.load_bmp("assets/bmp/test_background.bmp"),
        hero_bitmaps: load_hero(host_api),
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
    let screen_center_x = 0.5 * (state.offscreen_buffer.width as f32);
    let screen_center_y = 0.5 * (state.offscreen_buffer.height as f32);
    let player_height = 1.4;
    let player_width = 0.75 * player_height;
    state.offscreen_buffer.reset();

    //move player
    {
        let player_offset = {
            let mut player_x_offset = 0.0;
            let mut player_y_offset = 0.0;
            if input.new.up {
                state.hero_direction = 3;
                player_y_offset += 1.0
            };
            if input.new.down {
                state.hero_direction = 2;
                player_y_offset -= 1.0
            };

            if input.new.right {
                state.hero_direction = 1;
                player_x_offset += 1.0
            };
            if input.new.left {
                state.hero_direction = 0;
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
                Some(new_player_pos)
            } else {
                None
            }
        };
        if let Some(new_player_pos) = new_player_pos {
            state.player = new_player_pos;
        }
    }

    // move camera
    {
        state.camera.chunk_position.z = state.player.chunk_position.z;
        let distance_from_camera = state.tile_map.substract(state.player, state.camera);
        if distance_from_camera.x > 9.0 * state.tile_map.tile_size.0 {
            state.camera.chunk_position.x += 17;
        }
        if distance_from_camera.x < -(9.0 * state.tile_map.tile_size.0) {
            state.camera.chunk_position.x -= 17;
        }
        if distance_from_camera.y > 5.0 * state.tile_map.tile_size.0 {
            state.camera.chunk_position.y += 9;
        }
        if distance_from_camera.y < -(5.0 * state.tile_map.tile_size.0) {
            state.camera.chunk_position.y -= 9;
        }
    }

    // host_api.println(&format!("player at {:?}", state.player));
    // host_api.println(&format!("camera at {:?}", state.camera));
    state
        .offscreen_buffer
        .render_bitmap(&state.backdrop, 0.0, 0.0, 0, 0);

    //render map
    {
        for rel_row in -10..10 {
            for rel_column in -20..20 {
                let color = {
                    let column = state.camera.chunk_position.x as i32 + rel_column;
                    let row = state.camera.chunk_position.y as i32 + rel_row;
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
                                TileKind::Ground => continue,
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
                    let center_x = screen_center_x - meters_to_pixels * state.camera.offset.x
                        + (rel_column as f32) * tile_side_in_pixels;
                    let center_y = screen_center_y + meters_to_pixels * state.camera.offset.y
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
    }

    //render player
    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        {
            let distance_from_camera = state.tile_map.substract(state.player, state.camera);
            let player_ground_x = screen_center_x + meters_to_pixels * distance_from_camera.x;
            let player_ground_y = screen_center_y - meters_to_pixels * distance_from_camera.y;
            let player_left = player_ground_x - 0.5 * player_width * meters_to_pixels;
            let player_top = player_ground_y - 0.5 * player_height * meters_to_pixels;

            state.offscreen_buffer.render_rectangle(
                player_left,
                player_top,
                player_left + meters_to_pixels * player_width,
                player_top + meters_to_pixels * player_height,
                player_color,
            );

            let bitmaps = &state.hero_bitmaps[state.hero_direction];
            state.offscreen_buffer.render_bitmap(
                &bitmaps.torso,
                player_ground_x,
                player_ground_y,
                bitmaps.align_x as i32,
                bitmaps.align_y as i32,
            );
            state.offscreen_buffer.render_bitmap(
                &bitmaps.cape,
                player_ground_x,
                player_ground_y,
                bitmaps.align_x as i32,
                bitmaps.align_y as i32,
            );
            state.offscreen_buffer.render_bitmap(
                &bitmaps.head,
                player_ground_x,
                player_ground_y,
                bitmaps.align_x as i32,
                bitmaps.align_y as i32,
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

fn load_hero(host_api: &dyn HostApi) -> Vec<HeroBitmaps> {
    let mut result = Vec::with_capacity(4);
    for dir in ["left", "right", "front", "back"].iter() {
        let hero_bitmaps = HeroBitmaps {
            align_x: 72,
            align_y: 182,
            head: host_api.load_bmp(&format!("assets/bmp/test_hero_{}_head.bmp", dir)),
            cape: host_api.load_bmp(&format!("assets/bmp/test_hero_{}_cape.bmp", dir)),
            torso: host_api.load_bmp(&format!("assets/bmp/test_hero_{}_torso.bmp", dir)),
        };
        result.push(hero_bitmaps);
    }

    result
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub tile_map: TileMap,
    pub player: TileMapPosition,
    pub camera: TileMapPosition,

    backdrop: Bitmap,
    hero_direction: usize,
    hero_bitmaps: Vec<HeroBitmaps>,
}

impl GameState {
    // fn move_player_by(&mut self, offset: Offset) {
    //     let player = &mut self.player;
    //     player.offset.x += offset.x;
    //     player.offset.y += offset.y;
    //     self.tile_map.recanonicalize_position(player);
    // }
}

pub struct HeroBitmaps {
    align_x: u32,
    align_y: u32,
    head: Bitmap,
    cape: Bitmap,
    torso: Bitmap,
}
