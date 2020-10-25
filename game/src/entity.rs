use super::*;

//TODO use this
#[derive(Copy, Clone, Debug)]
pub struct EntityIdx(usize);

#[derive(Clone, Debug)]
pub struct HighEntity {
    pub t_bob: f32,
    //relative to camera
    pub p: V2,
    pub dp: V2,
    pub chunk_z: i32,
    pub facing_direction: usize,
    pub z: f32,
    pub dz: f32,
    pub low_entity_idx: usize,
}

impl HighEntity {
    fn new(p: V2, chunk_z: i32, low_entity_idx: usize) -> Self {
        Self {
            t_bob: 0.0,
            p,
            dp: V2::default(),
            chunk_z,
            facing_direction: 0,
            z: 0.0,
            dz: 0.0,
            low_entity_idx,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LowEntity {
    pub kind: EntityKind,
    pub p: WorldPosition,
    pub width: f32,
    pub height: f32,
    pub abs_tile_z: i32,
    pub collides: bool,
    pub high_entity_idx: Option<usize>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Wall,
    Player,
    Familiar,
    Monster
}

//TODO remove the clones, extract entities from world
#[derive(Clone, Debug)]
pub struct Entity {
    pub low_idx: usize,
    pub low: LowEntity,
    pub high: HighEntity,
}

#[derive(Clone, Debug)]
pub struct EntityStorage {
    low_entities: Vec<LowEntity>,
    high_entities: Vec<HighEntity>,
}

impl EntityStorage {
    pub fn new() -> Self {
        Self {
            low_entities: vec![],
            high_entities: vec![],
        }
    }

    pub fn validate_entities(&self) -> bool {
        let mut valid = true;
        for high_idx in 0..self.high_entities.len() {
            let high = &self.high_entities[high_idx];
            valid =
                valid && self.low_entities[high.low_entity_idx].high_entity_idx == Some(high_idx);
        }

        valid
    }

    pub fn push_low(&mut self, low_entity: LowEntity) -> usize {
        self.low_entities.push(low_entity);
        self.low_entities.len() - 1
    }

    pub fn low(&self, low_entity_idx: usize) -> &LowEntity {
        &self.low_entities[low_entity_idx]
    }

    pub fn high_mut(&mut self, idx: usize) -> &mut HighEntity {
        &mut self.high_entities[idx]
    }

    pub fn high_slice(&self) -> &[HighEntity] {
        &self.high_entities
    }

    pub fn make_low(&mut self, low_entity_idx: usize) {
        let high_idx = self
            .low_entities
            .get_mut(low_entity_idx)
            .and_then(|l| l.high_entity_idx.take());
        if let Some(high_idx) = high_idx {
            let last = self.high_entities.last().unwrap().low_entity_idx;
            self.high_entities.swap_remove(high_idx);
            if high_idx != self.high_entities.len() {
                self.low_entities[last].high_entity_idx = Some(high_idx);
            }
        }
    }

    pub fn high_idx(&self, low_entity_idx: usize) -> Option<usize> {
        self.low_entities
            .get(low_entity_idx)
            .and_then(|l| l.high_entity_idx)
    }

    pub fn new_high(&mut self, low_entity_idx: usize, p: V2) -> usize {
        let low = &mut self.low_entities[low_entity_idx];
        let new_high_idx = self.high_entities.len();
        let new_high = HighEntity::new(p, low.abs_tile_z, low_entity_idx);
        low.high_entity_idx = Some(new_high_idx);
        self.high_entities.push(new_high);
        new_high_idx
    }

    pub fn entity(&self, low_idx: usize) -> Entity {
        let low = self.low_entities[low_idx].clone();
        let high = self.high_entities[low
            .high_entity_idx
            .expect("Should call `make_high_entity` before")]
        .clone();
        Entity { low_idx, low, high }
    }

    pub fn update(&mut self, entity: Entity) {
        let idx = entity.high.low_entity_idx;
        self.high_entities[entity.low.high_entity_idx.unwrap()] = entity.high;
        self.low_entities[idx] = entity.low;
    }
}
