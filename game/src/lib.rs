mod entity;
#[path = "../../src/host_api.rs"]
mod host_api;
mod math;
mod render;
mod world;

pub use math::*;
pub use render::OffscreenBuffer;

use entity::*;
use host_api::*;
use render::Color;
use world::*;

#[no_mangle]
pub extern "C" fn game_init(host_api: &dyn HostApi) -> *mut GameState {
    println!("hello");
    let width = 1920 / 2;
    let height = 1080 / 2;
    let bytes_per_pixel = 4;
    let offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; width * height * bytes_per_pixel],
        width,
        height,
        bytes_per_pixel,
    };
    let world = World::new();
    let camera = world.initial_camera();
    let mut game = GameState {
        offscreen_buffer,
        world,
        camera,
        entity_focused_by_camera: None,
        entities: EntityStorage::new(),
        backdrop: host_api.load_bmp("assets/test/test_background.bmp"),
        shadow: host_api.load_bmp("assets/test/test_hero_shadow.bmp"),
        tree: host_api.load_bmp("assets/test2/tree00.bmp"),
        hero_bitmaps: load_hero(host_api),
    };
    game.tree.align_x = 40;
    game.tree.align_y = 80;
    let idx = game.add_player(camera);
    game.entity_focused_by_camera = Some(idx);
    game.add_walls();
    update_camera(&mut game, camera);
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_restart(state: &mut GameState) {
    state.world = World::new();
    state.camera = state.world.initial_camera();
    state.entities = EntityStorage::new();
    state.entity_focused_by_camera = None;
    let idx = state.add_player(state.camera);
    state.entity_focused_by_camera = Some(idx);
    state.add_walls();
    update_camera(state, state.camera);
}

#[no_mangle]
pub extern "C" fn game_update(
    state: &mut GameState,
    input: &Input,
    host_api: &mut dyn HostApi,
) -> bool {
    let tile_side_in_pixels: f32 = 60.0;
    let meters_to_pixels = tile_side_in_pixels as f32 / state.tile_side();
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
        let player = move_player(state, player, input.time_per_frame, ddp);
        state.update_player(player);
    }

    // move camera
    {
        if let Some(low_idx) = state.entity_focused_by_camera {
            let entity = state.entity(low_idx);
            let new_camera = entity.low.p;
            // let mut new_camera = state.camera;

            //TODO update camera z based on entity focused

            // if entity.high.p.x() > 9.0 * state.tile_side() {
            //     println!("moving right");
            //     new_camera.abs.x += 17;
            // }
            // if entity.high.p.x() < -(9.0 * state.tile_side()) {
            //     println!("moving left");
            //     new_camera.abs.x -= 17;
            // }
            // if entity.high.p.y() > 5.0 * state.tile_side() {
            //     println!("moving up");
            //     new_camera.abs.y += 9;
            // }
            // if entity.high.p.y() < -(5.0 * state.tile_side()) {
            //     println!("moving down");
            //     new_camera.abs.y -= 9;
            // }

            update_camera(state, new_camera);
        }
    }

    let grey = Color {
        red: 0.5,
        green: 0.5,
        blue: 0.5,
    };
    state.offscreen_buffer.render_rectangle(
        V2::default(),
        V2::new(
            state.offscreen_buffer.width as f32,
            state.offscreen_buffer.height as f32,
        ),
        grey,
    );

    // render entities
    {
        let player_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
        };

        for high in state.entities.high_slice().iter() {
            let low = &state.entities.low(high.low_entity_idx);
            let player_ground_x = screen_center_x + meters_to_pixels * high.p.x();
            let player_ground_y = screen_center_y - meters_to_pixels * high.p.y();
            let player_ground = V2::new(player_ground_x, player_ground_y);
            let width_height = V2::new(low.width, low.height);
            //bitmap renders with inversed Y (hence top, not bottom)
            let player_left_top1 = player_ground - 0.5 * meters_to_pixels * width_height;

            if low.kind == EntityKind::Player {
                let bitmaps = &state.hero_bitmaps[high.facing_direction];

                state.offscreen_buffer.render_bitmap(
                    &bitmaps.torso,
                    player_ground,
                    bitmaps.torso.align_x as i32,
                    bitmaps.torso.align_y as i32,
                    1.0,
                );
                state.offscreen_buffer.render_bitmap(
                    &bitmaps.cape,
                    player_ground,
                    bitmaps.cape.align_x as i32,
                    bitmaps.cape.align_y as i32,
                    1.0,
                );
                state.offscreen_buffer.render_bitmap(
                    &bitmaps.head,
                    player_ground,
                    bitmaps.head.align_x as i32,
                    bitmaps.head.align_y as i32,
                    1.0,
                );
            } else {
                state.offscreen_buffer.render_bitmap(
                    &state.tree,
                    player_ground,
                    state.tree.align_x as i32,
                    state.tree.align_y as i32,
                    1.0,
                );

                state.offscreen_buffer.render_rectangle(
                    player_left_top1,
                    player_left_top1 + meters_to_pixels * width_height * 0.9,
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
        let mut hero_bitmaps = HeroBitmaps {
            head: host_api.load_bmp(&format!("assets/test/test_hero_{}_head.bmp", dir)),
            cape: host_api.load_bmp(&format!("assets/test/test_hero_{}_cape.bmp", dir)),
            torso: host_api.load_bmp(&format!("assets/test/test_hero_{}_torso.bmp", dir)),
        };
        hero_bitmaps.head.align_x = 72;
        hero_bitmaps.head.align_y = 182;
        hero_bitmaps.cape.align_x = 72;
        hero_bitmaps.cape.align_y = 182;
        hero_bitmaps.torso.align_x = 72;
        hero_bitmaps.torso.align_y = 182;
        result.push(hero_bitmaps);
    }

    result
}

fn move_player(state: &mut GameState, mut player: Entity, dt: f32, ddp_orig: V2) -> Entity {
    let mut ddp = ddp_orig;

    //normalize (e.g: diagonal)
    if ddp.len() > 1.0 {
        ddp *= 1.0 / ddp.len().sqrt();
    }
    let player_speed = 5.0 * 50.0; // m/s^2
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
            for (high_idx, test_entity) in state.entities.high_slice().iter().enumerate() {
                if test_entity.low_entity_idx == player.high.low_entity_idx {
                    continue;
                }
                let test_low_entity = state.entities.low(test_entity.low_entity_idx);
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

    let new_p = state
        .world
        .map_into_chunk_space(state.camera, player.high.p);

    state
        .world
        .change_entity_chunks(player.low_idx, Some(player.low.p), new_p);
    player.low.p = new_p;

    player
}

fn update_camera(state: &mut GameState, new_camera: WorldPosition) {
    assert!(state.entities.validate_entities());

    let offset = state.camera_offset(new_camera);
    state.camera = new_camera;

    let tile_span_x = 17 * 3;
    let tile_span_y = 9 * 3;
    let camera_size = state.tile_side() * V2::new(tile_span_x as f32, tile_span_y as f32);

    let camera_bounds = Rect2::new_center_dim(V2::default(), camera_size);

    state.update_entities(-offset, camera_bounds);
    assert!(state.entities.validate_entities());

    let min_chunk = state
        .world
        .map_into_chunk_space(new_camera, camera_bounds.min());
    let max_chunk = state
        .world
        .map_into_chunk_space(new_camera, camera_bounds.max());

    for chunk_y in min_chunk.abs.y..=max_chunk.abs.y {
        for chunk_x in min_chunk.abs.x..=max_chunk.abs.x {
            let chunk_idx = ChunkIdx::new(chunk_x, chunk_y, new_camera.abs.z);
            if let Some(chunk) = state.world.chunk(chunk_idx) {
                //TODO here
                let entities: Vec<_> = chunk.entities().to_vec();
                for low_entity_idx in entities {
                    let low_entity = state.entities.low(low_entity_idx);
                    if low_entity.high_entity_idx.is_none() {
                        let entity_pos = state.camera_offset(low_entity.p);
                        if camera_bounds.contains(entity_pos) {
                            state.make_high(low_entity_idx);
                        }
                    }
                }
            }
        }
    }
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub world: World,
    pub camera: WorldPosition,
    entity_focused_by_camera: Option<usize>,

    entities: EntityStorage,

    backdrop: Bitmap,
    shadow: Bitmap,
    tree: Bitmap,
    hero_bitmaps: Vec<HeroBitmaps>,
}

impl GameState {
    fn tile_side(&self) -> f32 {
        self.world.tile_side
    }

    fn update_player(&mut self, entity: Entity) {
        self.entities.update(entity)
    }

    fn player(&mut self) -> Entity {
        self.entity(self.entity_focused_by_camera.unwrap())
    }

    fn entity(&mut self, low_idx: usize) -> Entity {
        self.make_high(low_idx);
        self.entities.entity(low_idx)
    }

    //promote
    fn make_high(&mut self, low_idx: usize) -> usize {
        if let Some(high_idx) = self.entities.high_idx(low_idx) {
            high_idx
        } else {
            let low_p = self.entities.low(low_idx).p;
            let p = self.camera_offset(low_p);
            self.entities.new_high(low_idx, p)
        }
    }

    fn camera_offset(&self, p: WorldPosition) -> V2 {
        let diff = self.world.substract(p, self.camera);
        diff.xy
    }

    fn update_entities(&mut self, offset: V2, camera_bounds: Rect2) {
        let mut idx = 0;
        while idx < self.entities.high_slice().len() {
            let high = &mut self.entities.high_mut(idx);
            high.p += offset;
            if camera_bounds.contains(high.p) {
                idx += 1;
            } else {
                let demoted = high.low_entity_idx;
                drop(high);
                self.entities.make_low(demoted)
            }
        }
    }

    fn add_walls(&mut self) {
        let walls = self.world.walls.clone();
        for abs in walls {
            let p = self.world.position_at_tile(abs.0, abs.1, abs.2);
            self.add_wall(p);
        }
    }

    fn add_wall(&mut self, p: WorldPosition) -> usize {
        let entity = LowEntity {
            kind: EntityKind::Wall,
            p,
            width: self.tile_side(),
            height: self.tile_side(),
            abs_tile_z: 0,
            collides: true,
            high_entity_idx: None,
        };
        let low_entity_idx = self.entities.push_low(entity);
        self.world.change_entity_chunks(low_entity_idx, None, p);
        low_entity_idx
    }

    fn add_player(&mut self, p: WorldPosition) -> usize {
        let player = LowEntity {
            kind: EntityKind::Player,
            p,
            width: 1.0,
            height: 0.5,
            abs_tile_z: 0,
            collides: true,
            high_entity_idx: None,
        };
        let idx = self.entities.push_low(player);
        self.world.change_entity_chunks(idx, None, p);
        idx
    }
}

pub struct HeroBitmaps {
    head: Bitmap,
    cape: Bitmap,
    torso: Bitmap,
}
