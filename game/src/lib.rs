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
    let mut state = GameState {
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
    state.shadow.align_x = 72;
    state.shadow.align_y = 182;
    state.tree.align_x = 40;
    state.tree.align_y = 80;
    state.start();
    Box::into_raw(Box::new(state))
}

#[no_mangle]
pub extern "C" fn game_restart(state: &mut GameState) {
    state.start();
}

#[no_mangle]
pub extern "C" fn game_update(
    state: &mut GameState, input: &Input, host_api: &mut dyn HostApi,
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
        // TODO check for some keypress, and create a sword

        for high in state.entities.high_slice().to_vec().iter() {
            let low = state.entities.low(high.low_entity_idx);

            match low.kind {
                EntityKind::Player => {
                    let entity = state.entities.entity(high.low_entity_idx);
                    let spec = MoveSpec {
                        unit_max_accel_vector: true,
                        speed: 5.0 * 50.0,
                        drag: 8.0,
                    };
                    let entity = move_entity(&state, entity, input.time_per_frame, ddp, spec);
                    state.update_entity(entity);
                }
                EntityKind::Sword => {
                    // TODO
                    // let entity = state.entities.entity(high.low_entity_idx);
                    // let entity = state.update_sword(entity, input.time_per_frame);
                    // state.update_entity(entity);
                }
                EntityKind::Familiar => {
                    let entity = state.entities.entity(high.low_entity_idx);
                    let entity = state.update_familiar(entity, input.time_per_frame);
                    state.update_entity(entity);
                }
                _ => {
                    //TODO
                }
            }
        }
    }

    // move camera
    {
        if let Some(low_idx) = state.entity_focused_by_camera {
            let entity = state.entities.entity(low_idx);
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
        for high in state.entities.high_slice().to_vec().iter() {
            let mut entity_pieces = vec![];
            let low = state.entities.low(high.low_entity_idx);

            let hero_bitmaps = &state.hero_bitmaps[high.facing_direction];
            let magenta = Color {
                red: 1.0,
                green: 0.0,
                blue: 1.0,
            };
            let debug = false;

            match low.kind {
                EntityKind::Player => {
                    let size = V2::new(low.width, low.height);
                    if debug {
                        let offset = 0.5 * size;
                        entity_pieces.push(EntityVisiblePiece::new_rect(
                            magenta, size, offset, 1.0, 1.0,
                        ));
                    }
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(&state.shadow, 1.0, 1.0));
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.torso,
                        1.0,
                        1.0,
                    ));
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.cape,
                        1.0,
                        1.0,
                    ));
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.head,
                        1.0,
                        1.0,
                    ));

                    {
                        let health_dim = V2::new(0.2, 0.2);
                        let spacing_x = 1.5 * health_dim.x();
                        let mut hit_p =
                            V2::new(-0.5 * (low.hit_points.len() - 1) as f32 * spacing_x, -0.25);
                        let dhit_p = V2::new(spacing_x, 0.0);
                        for _ in &low.hit_points {
                            let red = Color {
                                red: 1.0,
                                green: 0.0,
                                blue: 0.0,
                            };

                            entity_pieces.push(EntityVisiblePiece::new_rect(
                                red, health_dim, hit_p, 1.0, 0.0,
                            ));
                            hit_p += dhit_p;
                        }
                    }
                }
                EntityKind::Sword => {}
                EntityKind::Wall => {
                    if debug {
                        let size = V2::new(low.width, low.height);
                        let offset = 0.5 * size;
                        entity_pieces.push(EntityVisiblePiece::new_rect(
                            magenta, size, offset, 1.0, 1.0,
                        ));
                    }
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(&state.tree, 1.0, 1.0));
                }
                EntityKind::Familiar => {
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.head,
                        1.0,
                        1.0,
                    ));
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &state.shadow,
                        1.0,
                        high.t_bob,
                    ));
                }
                EntityKind::Monster => {
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.torso,
                        1.0,
                        1.0,
                    ));
                }
            }

            let entity_ground_x = screen_center_x + meters_to_pixels * high.p.x();
            let bob_offset = 0.3 * (high.t_bob * 3.0).sin();
            let entity_ground_y = screen_center_y - meters_to_pixels * (high.p.y() + bob_offset);
            let entity_ground = V2::new(entity_ground_x, entity_ground_y);
            //bitmap renders with inversed Y (hence top, not bottom)
            for piece in &entity_pieces {
                match piece.kind {
                    PieceKind::Bitmap(bitmap) => {
                        state.offscreen_buffer.render_bitmap(
                            bitmap,
                            entity_ground - piece.offset,
                            piece.alpha,
                        );
                    }
                    PieceKind::Rect(color, size) => {
                        let half = 0.5 * meters_to_pixels * size;
                        let center = entity_ground + meters_to_pixels * piece.offset;
                        let left_top = center - half;
                        state.offscreen_buffer.render_rectangle(
                            left_top,
                            left_top + size * meters_to_pixels * 0.9,
                            color,
                        )
                    }
                }
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

#[derive(Copy, Clone, Debug)]
struct MoveSpec {
    unit_max_accel_vector: bool,
    speed: f32,
    drag: f32,
}

fn move_entity(
    state: &GameState, mut entity: Entity, dt: f32, ddp_orig: V2, spec: MoveSpec,
) -> Entity {
    let mut ddp = ddp_orig;

    if spec.unit_max_accel_vector {
        //normalize (e.g: diagonal)
        if ddp.len_sq() > 1.0 {
            ddp *= 1.0 / ddp.len_sq().sqrt();
        }
    }
    ddp *= spec.speed;

    ddp = ddp - spec.drag * entity.high.dp;

    let mut player_delta: V2 = 0.5 * ddp * dt.powi(2) + entity.high.dp * dt;
    entity.high.dp = ddp * dt + entity.high.dp;

    // collision detection
    {
        for _ in 0..4 {
            let mut t_min = 1.0;
            let desired_position: V2 = entity.high.p + player_delta;
            let mut wall_normal = V2::default();
            let mut collided_idx = None;
            for (high_idx, test_entity) in state.entities.high_slice().iter().enumerate() {
                if entity.low.collides {
                    if test_entity.low_entity_idx == entity.high.low_entity_idx {
                        continue;
                    }
                    let test_low_entity = state.entities.low(test_entity.low_entity_idx);
                    if !test_low_entity.collides {
                        continue;
                    }

                    //minkowski
                    let diameter_w = test_low_entity.width + entity.low.width;
                    let diameter_h = test_low_entity.height + entity.low.height;
                    let corner_x = V2::new(-0.5 * diameter_w, 0.5 * diameter_w);
                    let corner_y = V2::new(-0.5 * diameter_h, 0.5 * diameter_h);

                    let rel: V2 = entity.high.p - test_entity.p;

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
            }

            //move entity
            entity.high.p += t_min * player_delta;
            if let Some(_collided_idx) = collided_idx.take() {
                entity.high.dp = entity.high.dp - entity.high.dp.inner(wall_normal) * wall_normal;
                player_delta = desired_position - entity.high.p;
                player_delta = player_delta - player_delta.inner(wall_normal) * wall_normal;
            //TODO update absZ
            } else {
                break;
            }
        }
    }

    //facing direction
    {
        if entity.high.dp.x() == 0.0 && entity.high.dp.y() == 0.0 {
            //do not change
        } else if entity.high.dp.x().abs() > entity.high.dp.y().abs() {
            if entity.high.dp.x() > 0.0 {
                //right
                entity.high.facing_direction = 1;
            } else {
                //left
                entity.high.facing_direction = 0;
            }
        } else {
            if entity.high.dp.y() > 0.0 {
                //up
                entity.high.facing_direction = 3;
            } else {
                //down
                entity.high.facing_direction = 2;
            }
        }
    }

    entity
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
    fn start(&mut self) {
        self.world = World::new();
        self.camera = self.world.initial_camera();
        self.entities = EntityStorage::new();
        self.entity_focused_by_camera = None;
        let idx = self.add_player(self.world.initial_player());
        self.entity_focused_by_camera = Some(idx);
        self.add_monster(self.world.initial_monster());
        self.add_familiars();
        self.add_walls();
        update_camera(self, self.camera);
    }

    fn tile_side(&self) -> f32 {
        self.world.tile_side
    }

    //TODO inline this in each update_X
    fn update_entity(&mut self, mut entity: Entity) {
        let new_p = self.world.map_into_chunk_space(self.camera, entity.high.p);

        self.world
            .change_entity_chunks(entity.low_idx, Some(entity.low.p), new_p);
        entity.low.p = new_p;

        self.entities.update(entity)
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

    fn add_familiars(&mut self) {
        self.add_familiar(self.world.initial_camera());
    }

    fn update_sword(&mut self, entity: Entity, dt: f32) -> Entity {
        let spec = MoveSpec {
            unit_max_accel_vector: false,
            speed: 0.0,
            drag: 0.0,
        };
        let old_p = entity.high.p;
        let mut entity = move_entity(&self, entity, dt, V2::default(), spec);
        {
            // TODO this should be in the 'main' switch
            let distance_traveled = (entity.high.p - old_p).len();
            entity.low.distance_remaining -= distance_traveled;
            if entity.low.distance_remaining < 0.0 {
                // TODO hide entity
            }
        }
        entity
    }

    fn update_familiar(&mut self, mut entity: Entity, dt: f32) -> Entity {
        let closest_distance = 10.0f32.powi(2);
        let mut closest_hero = None;

        entity.high.t_bob += dt;
        if entity.high.t_bob >= (2.0 * std::f32::consts::PI) {
            entity.high.t_bob -= 2.0 * std::f32::consts::PI;
        }

        for high in self.entities.high_slice() {
            let low = self.entities.low(high.low_entity_idx);
            if low.kind == EntityKind::Player {
                let distance = (high.p - entity.high.p).len_sq();
                if distance < closest_distance && distance > 2.0 {
                    closest_hero = Some(high.p);
                }
            }
        }

        if let Some(hero_p) = closest_hero {
            let acc = 0.5;
            let one_over_length = acc / closest_distance.sqrt();
            let ddp = one_over_length * (hero_p - entity.high.p);
            let spec = MoveSpec {
                unit_max_accel_vector: true,
                speed: 50.0,
                drag: 8.0,
            };
            move_entity(self, entity, dt, ddp, spec)
        } else {
            entity
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
            collides: true,
            ..Default::default()
        };
        let low_entity_idx = self.entities.push_low(entity);
        self.world.change_entity_chunks(low_entity_idx, None, p);
        low_entity_idx
    }

    fn add_monster(&mut self, p: WorldPosition) -> usize {
        let player = LowEntity {
            kind: EntityKind::Monster,
            p,
            width: 1.0,
            height: 0.5,
            collides: true,
            ..Default::default()
        };
        let idx = self.entities.push_low(player);
        self.world.change_entity_chunks(idx, None, p);
        idx
    }

    fn add_familiar(&mut self, p: WorldPosition) -> usize {
        let player = LowEntity {
            kind: EntityKind::Familiar,
            p,
            width: 1.0,
            height: 0.5,
            ..Default::default()
        };
        let idx = self.entities.push_low(player);
        self.world.change_entity_chunks(idx, None, p);
        idx
    }

    fn add_player(&mut self, p: WorldPosition) -> usize {
        let player = LowEntity {
            kind: EntityKind::Player,
            p,
            width: 1.0,
            height: 0.5,
            collides: true,
            hit_points: vec![HitPoint::full(); 3],
            sword: Some(self.add_sword()),
            ..Default::default()
        };
        let idx = self.entities.push_low(player);
        self.world.change_entity_chunks(idx, None, p);
        idx
    }

    fn add_sword(&mut self) -> usize {
        let entity = LowEntity {
            kind: EntityKind::Sword,
            width: 1.0,
            height: 0.5,
            ..Default::default()
        };
        let p = entity.p;
        let idx = self.entities.push_low(entity);
        self.world.change_entity_chunks(idx, None, p);
        idx
    }
}

pub struct HeroBitmaps {
    head: Bitmap,
    cape: Bitmap,
    torso: Bitmap,
}

pub struct EntityVisiblePiece<'a> {
    kind: PieceKind<'a>,
    offset: V2,
    _offset_z: f32,
    alpha: f32,
}

enum PieceKind<'a> {
    Bitmap(&'a Bitmap),
    Rect(Color, V2),
}

impl<'a> EntityVisiblePiece<'a> {
    fn new_rect(color: Color, size: V2, offset: V2, alpha: f32, _offset_z: f32) -> Self {
        Self {
            kind: PieceKind::Rect(color, size),
            offset,
            _offset_z,
            alpha,
        }
    }

    fn new_bitmap(bitmap: &'a Bitmap, alpha: f32, _offset_z: f32) -> Self {
        let offset = V2::new(bitmap.align_x as f32, bitmap.align_y as f32);
        Self {
            kind: PieceKind::Bitmap(bitmap),
            offset,
            _offset_z,
            alpha,
        }
    }
}
