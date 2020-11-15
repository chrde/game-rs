use std::collections::BTreeMap;
use super::*;

const HIT_POINT_SUB_COUNT: u8 = 4;

#[derive(Copy, Clone, Debug)]
pub struct MoveSpec {
    pub unit_max_accel_vector: bool,
    pub speed: f32,
    pub drag: f32,
}

impl MoveSpec {
    pub fn player() -> Self {
        Self {
            unit_max_accel_vector: true,
            speed: 50.0,
            drag: 8.0,
        }
    }

    pub fn sword() -> Self {
        Self {
            unit_max_accel_vector: false,
            speed: 1.0,
            drag: 0.0,
        }
    }
}

impl Default for MoveSpec {
    fn default() -> Self {
        Self {
            unit_max_accel_vector: false,
            speed: 1.0,
            drag: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash, Ord, PartialOrd)]
pub struct StorageIdx(usize);

#[derive(Clone, Debug, Default)]
pub struct SimEntity {
    pub idx: StorageIdx,
    pub updatable: bool,

    //
    pub kind: EntityKind,
    pub spatial: bool,
    pub collides: bool,
    pub simming: bool,

    pub p: V2,
    pub dp: V2,

    pub z: f32,
    pub dz: f32,

    pub chunk_z: i32,
    pub abs_tile_z: i32,

    pub width: f32,
    pub height: f32,

    pub sword: Option<StorageIdx>,
    pub hit_points: Vec<HitPoint>,
    pub facing_direction: usize,
    pub distance_remaining: f32,
    pub t_bob: f32,
}

pub struct SimRegion<'a> {
    storage: &'a mut Storage,
    world: &'a mut World,
    pub origin: WorldPosition,
    bounds: Rect2,
    updatable_bounds: Rect2,

    pub entities: BTreeMap<StorageIdx, SimEntity>,
}

impl<'a> SimRegion<'a> {
    pub fn new(
        storage: &'a mut Storage, world: &'a mut World, origin: WorldPosition,
        updatable_bounds: Rect2,
    ) -> Self {
        let update_range = 1.0;

        let updatable_bounds = updatable_bounds;
        let bounds = updatable_bounds.add_radius(V2::new(update_range, update_range));
        SimRegion {
            storage,
            world,
            origin,
            bounds,
            updatable_bounds,
            entities: BTreeMap::new(),
        }
    }

    pub fn begin(&mut self) {
        let min_chunk = self
            .world
            .map_into_chunk_space(self.origin, self.bounds.min());
        let max_chunk = self
            .world
            .map_into_chunk_space(self.origin, self.bounds.max());

        let mut new_entities = vec![];
        for chunk_y in min_chunk.abs.y..=max_chunk.abs.y {
            for chunk_x in min_chunk.abs.x..=max_chunk.abs.x {
                let chunk_idx = ChunkIdx::new(chunk_x, chunk_y, self.origin.abs.z);
                if let Some(chunk) = self.world.chunk(chunk_idx) {
                    let indices = chunk.entities().iter().map(|idx| StorageIdx(*idx));
                    for idx in indices {
                        let low_entity = self.storage.get(idx);
                        if low_entity.entity.spatial {
                            let p = self.get_sim_space_p(&low_entity);
                            if self.bounds.contains(p) {
                                let entity = self.load_entity(idx);
                                if let Some(sword) = entity.sword {
                                    let entity = self.load_entity(sword);
                                    new_entities.push(entity);
                                }
                                //TODO improve this - `new_entities` makes the borrow_checker happy
                                // after it, we could call `load_entity` instead
                                new_entities.push(entity);
                            }
                        }
                    }
                }
            }
        }
        self.update_entities(new_entities);
    }

    pub fn update_entities(&mut self, new_entities: Vec<SimEntity>) {
        for entity in new_entities {
            self.entities.insert(entity.idx, entity);
        }
    }

    //TODO update old in place
    pub fn move_entity(&self, old: &SimEntity, dt: f32, mut ddp: V2, spec: MoveSpec) -> SimEntity {
        if !old.spatial {
            return old.clone();
        }
        let mut entity = old.clone();
        assert!(entity.spatial);

        if spec.unit_max_accel_vector {
            //normalize (e.g: diagonal)
            if ddp.len_sq() > 1.0 {
                ddp *= 1.0 / ddp.len_sq().sqrt();
            }
        }
        ddp *= spec.speed;

        ddp = ddp - spec.drag * entity.dp;

        let mut player_delta: V2 = 0.5 * ddp * dt.powi(2) + entity.dp * dt;
        entity.dp = ddp * dt + entity.dp;

        // collision detection
        {
            for _ in 0..4 {
                let mut t_min = 1.0;
                let desired_position: V2 = entity.p + player_delta;
                let mut wall_normal = V2::default();
                let mut collided_idx = None;
                let can_collide = entity.collides && entity.spatial;
                if can_collide {
                    for (idx, test_entity) in &self.entities {
                        if test_entity.idx == entity.idx {
                            continue;
                        }
                        if !test_entity.collides || !test_entity.spatial {
                            continue;
                        }

                        //minkowski
                        let diameter_w = test_entity.width + entity.width;
                        let diameter_h = test_entity.height + entity.height;
                        let corner_x = V2::new(-0.5 * diameter_w, 0.5 * diameter_w);
                        let corner_y = V2::new(-0.5 * diameter_h, 0.5 * diameter_h);

                        let rel: V2 = entity.p - test_entity.p;

                        if test_wall(&mut t_min, corner_x.min(), rel, player_delta, corner_y) {
                            wall_normal = V2::new(-1.0, 0.0);
                            collided_idx = Some(idx);
                        }
                        if test_wall(&mut t_min, corner_x.max(), rel, player_delta, corner_y) {
                            wall_normal = V2::new(1.0, 0.0);
                            collided_idx = Some(idx);
                        }

                        if test_wall(
                            &mut t_min,
                            corner_y.min(),
                            rel.rev(),
                            player_delta.rev(),
                            corner_x,
                        ) {
                            wall_normal = V2::new(0.0, -1.0);
                            collided_idx = Some(idx);
                        }

                        if test_wall(
                            &mut t_min,
                            corner_y.max(),
                            rel.rev(),
                            player_delta.rev(),
                            corner_x,
                        ) {
                            wall_normal = V2::new(0.0, 1.0);
                            collided_idx = Some(idx);
                        }
                    }
                }

                // if entity.kind == EntityKind::Sword {
                //     println!("here");
                // }
                //move entity
                entity.p += t_min * player_delta;
                if let Some(_collided_idx) = collided_idx.take() {
                    entity.dp = entity.dp - entity.dp.inner(wall_normal) * wall_normal;
                    player_delta = desired_position - entity.p;
                    player_delta = player_delta - player_delta.inner(wall_normal) * wall_normal;
                //TODO update absZ
                } else {
                    break;
                }
            }
        }

        //facing direction
        {
            if entity.dp.x() == 0.0 && entity.dp.y() == 0.0 {
                //do not change
            } else if entity.dp.x().abs() > entity.dp.y().abs() {
                if entity.dp.x() > 0.0 {
                    //right
                    entity.facing_direction = 1;
                } else {
                    //left
                    entity.facing_direction = 0;
                }
            } else {
                if entity.dp.y() > 0.0 {
                    //up
                    entity.facing_direction = 3;
                } else {
                    //down
                    entity.facing_direction = 2;
                }
            }
        }

        entity
    }

    pub fn end(self, entity_focused_by_camera: Option<StorageIdx>) -> Option<WorldPosition> {
        let mut new_camera = None;
        let len = self.storage.len();
        for (idx, mut entity) in self.entities.into_iter() {
            assert!(entity.simming);
            entity.simming = false;
            let new_p = if entity.spatial {
                self.world.map_into_chunk_space(self.origin, entity.p)
            } else {
                WorldPosition::default()
            };
            let p = self.storage.get_mut(idx).p;
            if entity_focused_by_camera.map_or(false, |e| e == idx) {
                new_camera = Some(p);
            }
            {
                let mut low = LowEntity { entity, p };
                //TODO push & update pos should be combined once world contains storage
                self.world.change_entity_chunks(idx.0, &mut low, new_p);
                self.storage.update(idx, low);
            }
        }
        assert_eq!(len, self.storage.len());
        new_camera
    }

    fn get_sim_space_p(&self, low_entity: &LowEntity) -> V2 {
        if low_entity.entity.spatial {
            self.world.substract(low_entity.p, self.origin).xy
        } else {
            WorldPosition::invalid_offset()
        }
    }

    fn build_entity(&self, low_entity: &LowEntity, idx: StorageIdx, p: V2) -> SimEntity {
        let mut entity = low_entity.entity.clone();
        assert!(!entity.simming);
        entity.idx = idx;
        entity.simming = true;
        entity.p = p;
        entity.updatable = self.updatable_bounds.contains(p);
        assert!(!self.entities.contains_key(&idx));
        // ensure that we dont build it twice
        // assert!(self.entities.insert(idx, entity).is_none());
        entity
    }

    pub fn load_entity(&self, idx: StorageIdx) -> SimEntity {
        // TODO load sword
        let result = if let Some(entity) = self.entities.get(&idx) {
            panic!("already here");
            entity.clone()
        } else {
            let entity = self.storage.get(idx);
            let p = self.get_sim_space_p(&entity);
            let entity = self.build_entity(entity, idx, p);
            let result = entity.clone();
            result
        };
        result
    }
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

#[derive(Copy, Clone, Debug)]
pub struct HitPoint {
    flags: u8,
    filled: u8,
}

impl HitPoint {
    pub fn full() -> Self {
        Self {
            flags: 0,
            filled: HIT_POINT_SUB_COUNT,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Wall,
    Player,
    Familiar,
    Monster,
    Sword,
}

impl Default for EntityKind {
    fn default() -> Self {
        EntityKind::Wall
    }
}

#[derive(Clone, Debug)]
pub struct LowEntity {
    pub p: WorldPosition,
    pub entity: SimEntity,
}

#[derive(Clone, Debug, Default)]
pub struct Storage {
    entities: Vec<LowEntity>,
}

impl Storage {
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn push(&mut self, mut entity: LowEntity) -> StorageIdx {
        entity.entity.idx = StorageIdx(self.entities.len());
        self.entities.push(entity);
        StorageIdx(self.entities.len() - 1)
    }

    fn get(&self, idx: StorageIdx) -> &LowEntity {
        self.entities.get(idx.0).unwrap()
    }

    pub fn update(&mut self, idx: StorageIdx, entity: LowEntity) {
        self.entities[idx.0] = entity;
    }

    fn get_mut(&mut self, idx: StorageIdx) -> &mut LowEntity {
        self.entities.get_mut(idx.0).unwrap()
    }
}
