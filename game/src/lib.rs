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
        storage: Storage::default(),
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
    let player_ddp = {
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
    // TODO multiplayer

    let updatable_bounds = {
        let tile_span_x = 17 * 3;
        let tile_span_y = 9 * 3;
        let camera_size = state.tile_side() * V2::new(tile_span_x as f32, tile_span_y as f32);

        Rect2::new_center_dim(V2::default(), camera_size)
    };
    let mut sim_region = SimRegion::new(
        &mut state.storage,
        &mut state.world,
        state.camera,
        updatable_bounds,
    );
    sim_region.begin();

    // clear bg with magenta
    {
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
    }

    {
        let mut new_entities = vec![];
        for (_, entity) in sim_region.entities.iter() {
            if !entity.updatable {
                continue;
            }
            let mut entity_pieces = vec![];
            let hero_bitmaps = &state.hero_bitmaps[entity.facing_direction];

            let debug = true;
            match entity.kind {
                EntityKind::Player => {
                    if input.new.sword {
                        let sword_idx = entity.sword.unwrap();
                        let sword = sim_region.entities.get(&sword_idx).unwrap();
                        if !sword.spatial {
                            let mut sword = sword.clone();
                            sword.spatial = true;
                            sword.distance_remaining = 5.0;
                            sword.p = entity.p;
                            sword.dp = 2.0 * V2::new(1.0, 1.0);
                            new_entities.push(sword);
                        }
                    }
                    let size = V2::new(entity.width, entity.height);
                    if debug {
                        let offset = 0.5 * size;
                        entity_pieces.push(EntityVisiblePiece::new_rect(
                            Color::magenta(),
                            size,
                            offset,
                            1.0,
                            1.0,
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

                    // health points
                    {
                        let health_dim = V2::new(0.2, 0.2);
                        let spacing_x = 1.5 * health_dim.x();
                        let mut hit_p = V2::new(
                            -0.5 * (entity.hit_points.len() - 1) as f32 * spacing_x,
                            -0.25,
                        );
                        let dhit_p = V2::new(spacing_x, 0.0);
                        for _ in &entity.hit_points {
                            entity_pieces.push(EntityVisiblePiece::new_rect(
                                Color::red(),
                                health_dim,
                                hit_p,
                                1.0,
                                0.0,
                            ));
                            hit_p += dhit_p;
                        }
                    }
                    let new_entity = sim_region.move_entity(
                        &entity,
                        input.time_per_frame,
                        player_ddp,
                        MoveSpec::player(),
                    );
                    new_entities.push(new_entity);
                }
                EntityKind::Sword => {
                    for new in &new_entities {
                        assert!(new.kind != EntityKind::Sword);
                    }
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(&state.shadow, 1.0, 1.0));
                    let ddp = V2::new(0.0, 0.0);
                    let new_entity = update_sword(&sim_region, entity, input.time_per_frame, ddp);
                    new_entities.push(new_entity);
                }
                EntityKind::Wall => {
                    if debug {
                        let size = V2::new(entity.width, entity.height);
                        let offset = 0.5 * size;
                        entity_pieces.push(EntityVisiblePiece::new_rect(
                            Color::magenta(),
                            size,
                            offset,
                            1.0,
                            1.0,
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
                        entity.t_bob,
                    ));
                    let new_entity = update_familiar(&sim_region, entity, input.time_per_frame);
                    new_entities.push(new_entity);
                }
                EntityKind::Monster => {
                    entity_pieces.push(EntityVisiblePiece::new_bitmap(
                        &hero_bitmaps.torso,
                        1.0,
                        1.0,
                    ));
                }
            }

            // render
            let entity_ground_x = screen_center_x + meters_to_pixels * entity.p.x();
            let bob_offset = 0.3 * ((1.0 - entity.t_bob) * 3.0).sin();
            let entity_ground_y = screen_center_y - meters_to_pixels * (entity.p.y() + bob_offset);
            let entity_ground = V2::new(entity_ground_x, entity_ground_y);
            if entity.kind == EntityKind::Player {
                println!("{:?}", entity_ground);
            }
            for piece in entity_pieces {
                match piece.kind {
                    //bitmap renders with inversed Y (hence top, not bottom)
                    PieceKind::Bitmap(bitmap) => {
                        if debug {
                            state.offscreen_buffer.render_bitmap(
                                bitmap,
                                entity_ground - piece.offset,
                                piece.alpha,
                            );
                        }
                    }
                    PieceKind::Rect(color, size) => {
                        let half = 0.5 * meters_to_pixels * size;
                        let center = entity_ground + meters_to_pixels * piece.offset;
                        let left_top = entity_ground - half;
                        state.offscreen_buffer.render_rectangle(
                            left_top,
                            left_top + size * meters_to_pixels * 0.9,
                            color,
                        )
                    }
                }
            }
        }
        sim_region.update_entities(new_entities);
    }

    let origin = WorldPosition::origin();
    let sim_origin = sim_region.origin;

    if let Some(new_camera) = sim_region.end(state.entity_focused_by_camera) {
        state.camera = new_camera;
    }

    let diff = state.world.substract(origin, sim_origin);
    state
        .offscreen_buffer
        .render_rectangle(diff.xy, V2::new(10.0, 10.0), Color::magenta());

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

fn update_sword(sim_region: &SimRegion, entity: &SimEntity, dt: f32, ddp: V2) -> SimEntity {
    let mut new_entity = sim_region.move_entity(entity, dt, ddp, MoveSpec::sword());
    let distance_traveled = (entity.p - new_entity.p).len();
    new_entity.distance_remaining -= distance_traveled;
    if new_entity.distance_remaining < 0.0 {
        new_entity.spatial = false;
        new_entity.p = WorldPosition::invalid_offset();
    }
    new_entity
}

fn update_familiar(sim_region: &SimRegion, entity: &SimEntity, dt: f32) -> SimEntity {
    let mut entity = entity.clone();
    let closest_distance = 10.0f32.powi(2);
    let mut closest_hero = None;

    entity.t_bob += dt;
    if entity.t_bob >= (2.0 * std::f32::consts::PI) {
        entity.t_bob -= 2.0 * std::f32::consts::PI;
    }

    for (_, other) in &sim_region.entities {
        if other.kind == EntityKind::Player {
            let distance = (other.p - entity.p).len_sq();
            if distance < closest_distance && distance > 2.0 {
                closest_hero = Some(other.p);
            }
        }
    }

    if let Some(hero_p) = closest_hero {
        let acc = 0.5;
        let one_over_length = acc / closest_distance.sqrt();
        let ddp = one_over_length * (hero_p - entity.p);
        sim_region.move_entity(&entity, dt, ddp, MoveSpec::player())
    } else {
        entity
    }
}

#[repr(C)]
pub struct GameState {
    pub offscreen_buffer: OffscreenBuffer,
    pub world: World,
    pub camera: WorldPosition,
    entity_focused_by_camera: Option<StorageIdx>,

    storage: Storage,

    backdrop: Bitmap,
    shadow: Bitmap,
    tree: Bitmap,
    hero_bitmaps: Vec<HeroBitmaps>,
}

impl GameState {
    fn start(&mut self) {
        self.world = World::new();
        self.camera = self.world.initial_camera();
        self.storage = Storage::default();
        self.entity_focused_by_camera = None;
        let idx = self.add_player(self.world.initial_player());
        self.entity_focused_by_camera = Some(idx);
        self.add_monster(self.world.initial_monster());
        self.add_familiars();
        self.add_walls();
    }

    fn tile_side(&self) -> f32 {
        self.world.tile_side
    }

    fn add_familiars(&mut self) {
        self.add_familiar(self.world.initial_camera());
    }

    fn add_walls(&mut self) {
        let walls = self.world.walls.clone();
        for abs in walls {
            let p = self.world.position_at_tile(abs.0, abs.1, abs.2);
            self.add_wall(p);
        }
    }

    fn add_low(&mut self, p: WorldPosition, entity: SimEntity) -> StorageIdx {
        //TODO push & update pos should be combined once world contains storage
        let low_entity_idx = self.storage.len();
        let mut low = LowEntity {
            entity,
            p: WorldPosition::default(),
        };
        self.world.change_entity_chunks(low_entity_idx, &mut low, p);
        self.world.debug_stuff();
        self.storage.push(low)
    }

    fn add_wall(&mut self, p: WorldPosition) -> StorageIdx {
        let entity = SimEntity {
            kind: EntityKind::Wall,
            width: self.tile_side(),
            height: self.tile_side(),
            collides: true,
            spatial: true,
            ..Default::default()
        };
        self.add_low(p, entity)
    }

    fn add_monster(&mut self, p: WorldPosition) -> StorageIdx {
        let entity = SimEntity {
            kind: EntityKind::Monster,
            width: 1.0,
            height: 0.5,
            collides: true,
            spatial: true,
            ..Default::default()
        };
        self.add_low(p, entity)
    }

    fn add_familiar(&mut self, p: WorldPosition) -> StorageIdx {
        let entity = SimEntity {
            kind: EntityKind::Familiar,
            width: 1.0,
            height: 0.5,
            spatial: true,
            ..Default::default()
        };
        self.add_low(p, entity)
    }

    fn add_player(&mut self, p: WorldPosition) -> StorageIdx {
        let entity = SimEntity {
            kind: EntityKind::Player,
            width: 1.0,
            height: 0.5,
            collides: true,
            spatial: true,
            hit_points: vec![HitPoint::full(); 3],
            sword: Some(self.add_sword()),
            ..Default::default()
        };
        self.add_low(p, entity)
    }

    fn add_sword(&mut self) -> StorageIdx {
        let entity = SimEntity {
            kind: EntityKind::Sword,
            width: 1.0,
            height: 0.5,
            ..Default::default()
        };
        self.add_low(WorldPosition::default(), entity)
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
