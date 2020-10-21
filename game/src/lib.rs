#[path = "../../src/host_api.rs"]
mod host_api;
mod math;
mod render;
mod tile;

pub use math::*;
pub use render::OffscreenBuffer;

use host_api::*;
use render::Color;
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
    let tile_map = TileMap::new();
    let camera = tile_map.initial_camera();
    let mut game = GameState {
        offscreen_buffer,
        tile_map,
        camera,
        entity_focused_by_camera: None,
        low_entities: Vec::new(),
        high_entities: Vec::new(),
        backdrop: host_api.load_bmp("assets/bmp/test_background.bmp"),
        shadow: host_api.load_bmp("assets/bmp/test_hero_shadow.bmp"),
        hero_bitmaps: load_hero(host_api),
    };
    game.add_player(camera);
    game.add_walls();
    update_camera(&mut game, camera);
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_update(
    state: &mut GameState,
    input: &Input,
    host_api: &mut dyn HostApi,
) -> bool {
    let tile_side_in_pixels: f32 = 60.0;
    let meters_to_pixels = tile_side_in_pixels as f32 / state.tile_size_in_meters();
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

        let player = state.player();
        let new_player = move_player(&state, player, input.time_per_frame, ddp);
        state.update_player(new_player);
    }

    // move camera
    {
        if let Some(low_idx) = state.entity_focused_by_camera {
            let entity = state.high_entity(low_idx);
            let mut new_camera = state.camera;

            //TODO update camera z based on entity focused

            if entity.high.p.x() > 9.0 * state.tile_size_in_meters() {
                new_camera.abs.x += 17;
            }
            if entity.high.p.x() < -(9.0 * state.tile_size_in_meters()) {
                new_camera.abs.x -= 17;
            }
            if entity.high.p.y() > 5.0 * state.tile_size_in_meters() {
                new_camera.abs.y += 9;
            }
            if entity.high.p.y() < -(5.0 * state.tile_size_in_meters()) {
                new_camera.abs.y -= 9;
            }

            update_camera(state, new_camera);
        }
    }

    state
        .offscreen_buffer
        .render_bitmap(&state.backdrop, V2::default(), 0, 0, 1.0);

    // render entities
    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        for high in state.high_entities.iter().rev() {
            let low = &state.low_entities[high.low_entity_idx];
            let player_ground_x = screen_center_x + meters_to_pixels * high.p.x();
            let player_ground_y = screen_center_y - meters_to_pixels * high.p.y();
            let player_ground = V2::new(player_ground_x, player_ground_y);
            let width_height = V2::new(low.width, low.height);
            //bitmap renders with inversed Y (hence top, not bottom)
            let player_left_top = player_ground - 0.5 * meters_to_pixels * width_height;

            if low.kind == EntityKind::Player {
                let bitmaps = &state.hero_bitmaps[high.facing_direction];
                state.offscreen_buffer.render_bitmap(
                    &bitmaps.torso,
                    player_ground,
                    bitmaps.align_x as i32,
                    bitmaps.align_y as i32,
                    1.0,
                );
                state.offscreen_buffer.render_bitmap(
                    &bitmaps.cape,
                    player_ground,
                    bitmaps.align_x as i32,
                    bitmaps.align_y as i32,
                    1.0,
                );
                state.offscreen_buffer.render_bitmap(
                    &bitmaps.head,
                    player_ground,
                    bitmaps.align_x as i32,
                    bitmaps.align_y as i32,
                    1.0,
                );
            } else {
                state.offscreen_buffer.render_rectangle(
                    player_left_top,
                    player_left_top + meters_to_pixels * width_height,
                    player_color,
                );
            }
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
    let epsilon = 0.001;
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
    state: &GameState,
    mut player: Entity,
    dt: f32,
    ddp_orig: V2,
) -> Entity {
    let mut ddp = ddp_orig;

    //normalize (e.g: diagonal)
    if ddp.len() > 1.0 {
        ddp *= 1.0 / ddp.len().sqrt();
    }
    let player_speed = 50.0; // m/s^2
    ddp *= player_speed;

    let friction = -8.0;
    ddp += friction * player.high.dp;

    let mut player_delta: V2 = 0.5 * ddp * dt.powi(2) + player.high.dp * dt;
    player.high.dp = ddp * dt + player.high.dp;

    // collision detection
    {
        for _ in 0..4 {
            let mut t_min = 1.0;
            let desired_position: V2 = player.high.p + player_delta;
            let mut wall_normal = V2::default();
            let mut collided_idx = None;
            for (high_idx, test_entity) in state.high_entities.iter().enumerate() {
                if test_entity.low_entity_idx == player.high.low_entity_idx {
                    continue;
                }
                let test_low_entity = state.low_entity(test_entity.low_entity_idx);
                if !test_low_entity.collides {
                    continue;
                }

                //minkowski
                let diameter_w = test_low_entity.width + player.low.width;
                let diameter_h = test_low_entity.height + player.low.height;
                let corner_x = V2::new(-0.5 * diameter_w, 0.5 * diameter_w);
                let corner_y = V2::new(-0.5 * diameter_h, 0.5 * diameter_h);

                let rel: V2 = player.high.p - test_entity.p;

                if test_wall(&mut t_min, corner_x.min(), rel, player_delta, corner_y) {
                    wall_normal = V2::new(-1.0, 0.0);
                    collided_idx = Some(high_idx);
                }
                if test_wall(&mut t_min, corner_x.max(), rel, player_delta, corner_y) {
                    wall_normal = V2::new(1.0, 0.0);
                    collided_idx = Some(high_idx);
                }

                if test_wall(
                    &mut t_min,
                    corner_y.min(),
                    rel.rev(),
                    player_delta.rev(),
                    corner_x,
                ) {
                    wall_normal = V2::new(0.0, -1.0);
                    collided_idx = Some(high_idx);
                }

                if test_wall(
                    &mut t_min,
                    corner_y.max(),
                    rel.rev(),
                    player_delta.rev(),
                    corner_x,
                ) {
                    wall_normal = V2::new(0.0, 1.0);
                    collided_idx = Some(high_idx);
                }
            }

            //move player
            player.high.p += t_min * player_delta;
            if let Some(_collided_idx) = collided_idx.take() {
                player.high.dp = player.high.dp - player.high.dp.inner(wall_normal) * wall_normal;
                player_delta = desired_position - player.high.p;
                player_delta = player_delta - player_delta.inner(wall_normal) * wall_normal;
            //TODO update absZ
            } else {
                break;
            }
        }
    }

    //facing direction
    {
        if player.high.dp.x() == 0.0 && player.high.dp.y() == 0.0 {
            //do not change
        } else if player.high.dp.x().abs() > player.high.dp.y().abs() {
            if player.high.dp.x() > 0.0 {
                //right
                player.high.facing_direction = 1;
            } else {
                //left
                player.high.facing_direction = 0;
            }
        } else {
            if player.high.dp.y() > 0.0 {
                //up
                player.high.facing_direction = 3;
            } else {
                //down
                player.high.facing_direction = 2;
            }
        }
    }

    player.low.p = state
        .tile_map
        .map_into_tile_space(state.camera, player.high.p);
    player
}

fn update_camera(state: &mut GameState, new_position: TileMapPosition) {
    let diff = state.tile_map.substract(new_position, state.camera);

    state.camera = new_position;

    let tile_span_x = 17 * 3;
    let tile_span_y = 9 * 3;

    let camera_size = state.tile_size_in_meters() * V2::new(tile_span_x as f32, tile_span_y as f32);

    let camera_bounds = Rect2::new_center_dim(V2::default(), camera_size);

    state.offset_and_check_frequency_area(-diff.xy, camera_bounds);

    // state.camera.abs.z = state.player().p.abs.z;
    let min_tile_x = new_position.abs.x.saturating_sub(tile_span_x / 2);
    let max_tile_x = new_position.abs.x + tile_span_x / 2;
    let min_tile_y = new_position.abs.y.saturating_sub(tile_span_y / 2);
    let max_tile_y = new_position.abs.y + tile_span_y / 2;


    for low_idx in 0..state.low_entities.len() {
        let low_pos = state.low_entity(low_idx).p;
        //TODO z
        if low_pos.abs.x >= min_tile_x
            && low_pos.abs.x <= max_tile_x
            && low_pos.abs.y >= min_tile_y
            && low_pos.abs.y <= max_tile_y
        {
            state.make_high_entity(low_idx);
        }
    }
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub tile_map: TileMap,
    pub camera: TileMapPosition,
    entity_focused_by_camera: Option<usize>,

    low_entities: Vec<LowEntity>,
    high_entities: Vec<HighEntity>,

    backdrop: Bitmap,
    shadow: Bitmap,
    hero_bitmaps: Vec<HeroBitmaps>,
}

impl GameState {
    fn tile_size_in_meters(&self) -> f32 {
        self.tile_map.tile_size.0
    }

    fn update_player(&mut self, entity: Entity) {
        let idx = entity.high.low_entity_idx;
        self.high_entities[entity.low.high_entity_idx.unwrap()] = entity.high;
        self.low_entities[idx] = entity.low;
    }

    fn player(&mut self) -> Entity {
        self.high_entity(self.entity_focused_by_camera.unwrap())
    }

    fn low_entity_mut(&mut self, low_idx: usize) -> &mut LowEntity {
        &mut self.low_entities[low_idx]
    }

    fn low_entity(&self, low_idx: usize) -> &LowEntity {
        &self.low_entities[low_idx]
    }

    fn high_entity(&mut self, low_idx: usize) -> Entity {
        let high_entity_idx = self.make_high_entity(low_idx);
        let low = self.low_entities[low_idx].clone();
        let high = self.high_entities[high_entity_idx].clone();
        Entity { low_idx, low, high }
    }

    //demote
    fn make_low_entity(&mut self, low_idx: usize) {
        if let Some(high_idx) = self.low_entity_mut(low_idx).high_entity_idx.take() {
            let last = self.high_entities.last().unwrap().low_entity_idx;
            self.high_entities.swap_remove(high_idx);
            if high_idx != self.high_entities.len() {
                self.low_entity_mut(last).high_entity_idx = Some(high_idx);
            }
        }
    }

    //promote
    fn make_high_entity(&mut self, low_idx: usize) -> usize {
        let camera = self.camera;
        let low = &mut self.low_entities[low_idx];
        if let Some(high_idx) = low.high_entity_idx {
            high_idx
        } else {
            let new_high_idx = self.high_entities.len();
            let new_high = {
                let p = self.tile_map.substract(low.p, camera);
                HighEntity::new(p.xy, low.abs_tile_z, low_idx)
            };
            low.high_entity_idx = Some(new_high_idx);
            self.high_entities.push(new_high);
            new_high_idx
        }
    }

    fn offset_and_check_frequency_area(&mut self, offset: V2, camera_bounds: Rect2) {
        let mut idx = 0;
        while idx < self.high_entities.len() {
            let high = &mut self.high_entities[idx];
            high.p += offset;
            if camera_bounds.contains(high.p) {
                idx += 1;
            } else {
                let demoted = high.low_entity_idx;
                drop(high);
                self.make_low_entity(demoted)
            }
        }
    }

    fn add_walls(&mut self) {
        let walls = self.tile_map.walls.clone();
        for abs in walls {
            let p = TileMapPosition {
                abs,
                offset: V2::default(),
            };
            self.add_wall(p);
        }
    }

    fn add_wall(&mut self, p: TileMapPosition) -> usize {
        let entity = LowEntity {
            kind: EntityKind::Wall,
            p,
            width: self.tile_size_in_meters(),
            height: self.tile_size_in_meters(),
            abs_tile_z: 0,
            collides: true,
            high_entity_idx: None,
        };
        self.low_entities.push(entity);
        self.low_entities.len() - 1
    }

    fn add_player(&mut self, p: TileMapPosition) -> usize {
        let player = LowEntity {
            kind: EntityKind::Player,
            p,
            width: 1.0,
            height: 0.5,
            abs_tile_z: 0,
            collides: true,
            high_entity_idx: None,
        };
        self.low_entities.push(player);
        let idx = self.low_entities.len() - 1;
        if self.entity_focused_by_camera.is_none() {
            self.entity_focused_by_camera = Some(idx);
        }
        idx
    }
}

//TODO remove the clones, extract entities from world
#[derive(Clone, Debug)]
pub struct Entity {
    low_idx: usize,
    low: LowEntity,
    high: HighEntity,
}

#[derive(Clone, Debug)]
pub struct LowEntity {
    kind: EntityKind,
    p: TileMapPosition,
    width: f32,
    height: f32,
    abs_tile_z: i32,
    collides: bool,
    high_entity_idx: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct HighEntity {
    //relative to camera
    p: V2,
    dp: V2,
    abs_tile_z: i32,
    facing_direction: usize,
    z: f32,
    dz: f32,
    low_entity_idx: usize,
}

impl HighEntity {
    fn new(p: V2, abs_tile_z: i32, low_entity_idx: usize) -> Self {
        Self {
            p,
            dp: V2::default(),
            abs_tile_z,
            facing_direction: 0,
            z: 0.0,
            dz: 0.0,
            low_entity_idx,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Wall,
    Player,
}

pub struct HeroBitmaps {
    align_x: u32,
    align_y: u32,
    head: Bitmap,
    cape: Bitmap,
    torso: Bitmap,
}
