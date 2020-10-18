#[path = "../../src/host_api.rs"]
mod host_api;
mod math;
mod render;
mod tile;

pub use math::V2;
pub use render::OffscreenBuffer;

use host_api::*;
use render::Color;
use std::cmp;
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
    let camera = TileMapPosition::initial_camera();
    let mut game = GameState {
        tile_map: TileMap::new(),
        offscreen_buffer,
        entities: Vec::new(),
        camera,
        backdrop: host_api.load_bmp("assets/bmp/test_background.bmp"),
        hero_bitmaps: load_hero(host_api),
    };
    game.add_player();
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
    state.offscreen_buffer.reset();

    //move player
    {
        let ddp = {
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

            V2::new(player_x_offset, player_y_offset)
        };

        let player = &mut state.entities[0];
        let tile_map = &state.tile_map;
        move_player(tile_map, player, input.time_per_frame, ddp)

    }

    // move camera
    {
        state.camera.chunk_position.z = state.player().p.chunk_position.z;
        let distance_from_camera = state.offset_camera_player();
        if distance_from_camera.xy.x() > 9.0 * state.tile_size_in_meters() {
            state.camera.chunk_position.x += 17;
        }
        if distance_from_camera.xy.x() < -(9.0 * state.tile_size_in_meters()) {
            state.camera.chunk_position.x -= 17;
        }
        if distance_from_camera.xy.y() > 5.0 * state.tile_size_in_meters() {
            state.camera.chunk_position.y += 9;
        }
        if distance_from_camera.xy.y() < -(5.0 * state.tile_size_in_meters()) {
            state.camera.chunk_position.y -= 9;
        }
    }

    state
        .offscreen_buffer
        .render_bitmap(&state.backdrop, V2::default(), 0, 0);

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
                        let color = if pos == state.player().p.chunk_position {
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
                    let tile_side = 0.5 * V2::new(tile_side_in_pixels, tile_side_in_pixels);
                    let center = {
                        let x = screen_center_x - meters_to_pixels * state.camera.offset.x()
                            + (rel_column as f32) * tile_side_in_pixels;
                        let y = screen_center_y + meters_to_pixels * state.camera.offset.y()
                            - (rel_row as f32) * tile_side_in_pixels;

                        V2::new(x, y)
                    };

                    // let min = center - 0.9 * tile_side;
                    // let max = center + 0.9 * tile_side;

                    let min = center - tile_side;
                    let max = center + tile_side;

                    state.offscreen_buffer.render_rectangle(min, max, color);
                }
            }
        }
    }

    // render player
    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        {
            let distance_from_camera = state.offset_camera_player();
            let player_ground_x = screen_center_x + meters_to_pixels * distance_from_camera.xy.x();
            let player_ground_y = screen_center_y - meters_to_pixels * distance_from_camera.xy.y();
            let player_ground = V2::new(player_ground_x, player_ground_y);
            let width_height = V2::new(state.player().width, state.player().height);
            //bitmap renders with inversed Y (hence top, not bottom)
            let player_left_top = player_ground - 0.5 * meters_to_pixels * width_height;

            state.offscreen_buffer.render_rectangle(
                player_left_top,
                player_left_top + meters_to_pixels * width_height,
                player_color,
            );

            let bitmaps = &state.hero_bitmaps[state.player().facing_direction];
            state.offscreen_buffer.render_bitmap(
                &bitmaps.torso,
                player_ground,
                bitmaps.align_x as i32,
                bitmaps.align_y as i32,
            );
            state.offscreen_buffer.render_bitmap(
                &bitmaps.cape,
                player_ground,
                bitmaps.align_x as i32,
                bitmaps.align_y as i32,
            );
            state.offscreen_buffer.render_bitmap(
                &bitmaps.head,
                player_ground,
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

fn test_wall(t_min: &mut f32, wall_x: f32, rel: V2, player_delta: V2, y_range: V2) -> bool {
    let epsilon = 0.0001;
    if player_delta.x() != 0.0 {
        let t_result = (wall_x - rel.x()) / player_delta.x();
        let y = rel.y() + t_result * player_delta.y();
        if t_result >= 0.0 && *t_min > t_result {
            if y >= y_range.min() && y <= y_range.max() {
                *t_min = {
                    let new = t_result - epsilon;
                    if new < 0.0 {
                        0.0
                    } else {
                        new
                    }
                };
                return true;
            }
        }
    }

    false
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

fn move_player(
    tile_map: &TileMap,
    player: &mut Entity,
    dt: f32,
    ddp_orig: V2,
) {
    let mut ddp = ddp_orig;

    //normalize (e.g: diagonal)
    if ddp.len() > 1.0 {
        ddp *= 1.0 / ddp.len().sqrt();
    }
    let player_speed = 50.0; // m/s^2
    ddp *= player_speed;

    let gravity = -8.0;
    ddp += gravity * player.dp;

    let old_pos = player.p;
    let mut player_delta: V2 = 0.5 * ddp * dt.powi(2) + player.dp * dt;
    player.dp = ddp * dt + player.dp;
    let new_pos = tile_map.offset(old_pos, player_delta);

    assert_eq!(old_pos.chunk_position.z, new_pos.chunk_position.z);
    let z_pos = old_pos.chunk_position.z;

    //collision detection
    {
        let player_width = (player.width / tile_map.tile_size.0).ceil() as usize;
        let player_height = (player.height / tile_map.tile_size.0).ceil() as usize;

        let min_x = {
            let min = cmp::min(old_pos.chunk_position.x, new_pos.chunk_position.x);
            min - player_width
        };
        let max_x = {
            let max = cmp::max(old_pos.chunk_position.x, new_pos.chunk_position.x);
            max + player_width
        };
        let min_y = {
            let min = cmp::min(old_pos.chunk_position.y, new_pos.chunk_position.y);
            min - player_height
        };
        let max_y = {
            let max = cmp::max(old_pos.chunk_position.y, new_pos.chunk_position.y);
            max + player_height
        };

        let mut t_remaining = 1.0;
        for _ in 0..4 {
            let mut t_min = 1.0;

            assert!((max_x - min_x) < 32);
            assert!((max_y - min_y) < 32);

            let mut wall_normal = V2::default();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let tile_pos = TileMap::centered_tile_point(x, y, z_pos);
                    if !tile_map.is_tile_empty(tile_pos) {
                        //minkowski
                        let diameter_w = tile_map.tile_size.0 + player.width;
                        let diameter_h = tile_map.tile_size.0 + player.height;
                        let corner_x = V2::new(-0.5 * diameter_w, 0.5 * diameter_w);
                        let corner_y = V2::new(-0.5 * diameter_h, 0.5 * diameter_h);

                        let rel = tile_map.substract(player.p, tile_pos);

                        if test_wall(&mut t_min, corner_x.min(), rel.xy, player_delta, corner_y) {
                            wall_normal = V2::new(-1.0, 0.0);
                        }
                        if test_wall(&mut t_min, corner_x.max(), rel.xy, player_delta, corner_y) {
                            wall_normal = V2::new(1.0, 0.0);
                        }

                        if test_wall(&mut t_min, corner_y.min(), rel.xy.rev(), player_delta.rev(), corner_x) {
                            wall_normal = V2::new(0.0, -1.0);
                        }

                        if test_wall(&mut t_min, corner_y.max(), rel.xy.rev(), player_delta.rev(), corner_x) {
                            wall_normal = V2::new(0.0, 1.0);
                        }

                    }
                }
            }

            //move player
            player.p = tile_map.offset(player.p, t_min * player_delta);
            player.dp = player.dp - player.dp.inner(wall_normal) * wall_normal;
            player_delta = player_delta - player_delta.inner(wall_normal) * wall_normal;
            t_remaining -= t_min * t_remaining;

            if t_remaining <= 0.0 {
                break;
            }
        }
    }

    //facing direction
    {
        if player.dp.x() == 0.0 && player.dp.y() == 0.0 {
            //do not change
        } else if player.dp.x().abs() > player.dp.y().abs() {
            if player.dp.x() > 0.0 {
                //right
                player.facing_direction = 1;
            } else {
                //left
                player.facing_direction = 0;
            }
        } else {
            if player.dp.y() > 0.0 {
                //up
                player.facing_direction = 3;
            } else {
                //down
                player.facing_direction = 2;
            }
        }
    }
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub tile_map: TileMap,
    pub camera: TileMapPosition,

    entities: Vec<Entity>,
    backdrop: Bitmap,
    hero_bitmaps: Vec<HeroBitmaps>,
}

impl GameState {
    fn offset_camera_player(&self) -> PositionDiff {
        let player_pos = self.player().p;
        self.tile_map.substract(player_pos, self.camera)
    }

    // fn entity(&self, idx: usize) -> &Entity {
    //     &self.entities[idx]
    // }

    fn tile_size_in_meters(&self) -> f32 {
        self.tile_map.tile_size.0
    }

    // fn entity_mut(&mut self, idx: usize) -> &mut Entity {
    //     &mut self.entities[idx]
    // }

    fn player(&self) -> &Entity {
        &self.entities[0]
    }

    // fn player_mut(&mut self) -> &mut Entity {
    //     &mut self.entities[0]
    // }

    fn add_player(&mut self) {
        assert!(self.entities.is_empty());
        let player = Entity {
            exists: true,
            p: TileMapPosition::initial_player(),
            dp: V2::default(),
            facing_direction: 0,
            width: 1.0,
            height: 0.5,
        };
        self.entities.push(player);
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    exists: bool,
    p: TileMapPosition,
    dp: V2,
    facing_direction: usize,
    width: f32,
    height: f32,
}

pub struct HeroBitmaps {
    align_x: u32,
    align_y: u32,
    head: Bitmap,
    cape: Bitmap,
    torso: Bitmap,
}
