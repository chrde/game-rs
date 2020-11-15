use std::collections::HashMap;

use super::*;

#[derive(Copy, Clone, Debug)]
pub struct WorldDiff {
    pub xy: V2,
    pub z: f32,
}

#[derive(Default, Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ChunkIdx {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkIdx {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Position of chunk in the global map
#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
    pub abs: ChunkIdx,

    /// offset from chunk center
    pub offset: V2,
}

impl Default for WorldPosition {
    fn default() -> Self {
        Self {
            abs: ChunkIdx::new(TILE_CHUNK_UNINITIALIZED, 0, 0),
            offset: V2::default(),
        }
    }
}

impl WorldPosition {
    pub fn invalid_offset() -> V2 {
        V2::new(100000.0, 100000.0)
    }

    pub fn origin() -> Self {
        Self {
            abs: ChunkIdx::default(),
            offset: V2::default(),
        }
    }

    // pub fn null_position() -> Self {
    //     Self {
    //         abs: ChunkIdx::new(TILE_CHUNK_UNINITIALIZED, 0, 0),
    //         ..Default::default()
    //     }
    // }

    fn is_valid(&self) -> bool {
        self.abs.x != TILE_CHUNK_UNINITIALIZED
    }
}

#[derive(Debug)]
pub struct World {
    middle: i32,
    pub tile_side: f32,  //in meters
    pub chunk_side: f32, //in meters
    chunks: HashMap<ChunkIdx, Chunk>,
    pub walls: Vec<(i32, i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct Chunk {
    idx: ChunkIdx,
    entities: Vec<usize>,
}

const TILES_PER_CHUNK: u16 = 16;
const TILE_CHUNK_UNINITIALIZED: i32 = std::i32::MAX;

impl Chunk {
    fn new(idx: ChunkIdx) -> Self {
        Self {
            idx,
            entities: vec![],
        }
    }

    fn remove_entity(&mut self, low_entity_idx: usize) -> Option<usize> {
        if let Some(pos) = self.entities.iter().position(|x| *x == low_entity_idx) {
            Some(self.entities.swap_remove(pos))
        } else {
            None
        }
    }

    pub fn entities(&self) -> &[usize] {
        &self.entities
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Tile {
    pub kind: TileKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TileKind {
    Wall,
    Ground,
}

impl World {
    pub fn initial_monster(&self) -> WorldPosition {
        let center = self.position_at_tile(17 / 2 + 1, 9 / 2 + 1, 0);
        self.map_into_chunk_space(center, V2::default())
    }

    pub fn initial_player(&self) -> WorldPosition {
        let center = self.position_at_tile(17 / 2, 9 / 2, 0);
        self.map_into_chunk_space(center, V2::default())
    }

    pub fn initial_camera(&self) -> WorldPosition {
        let center = WorldPosition {
            // abs: ChunkIdx::new(self.middle + 17 / 2, self.middle + 9 / 2, self.middle + 0),
            offset: V2::new(1.5, 1.5),
            abs: ChunkIdx::default(),
        };
        self.map_into_chunk_space(center, V2::default())
    }

    pub fn new() -> Self {
        let tile_side = 1.4;
        let chunk_side = tile_side * TILES_PER_CHUNK as f32;
        let mut result = Self {
            // middle: std::i16::MAX as i32 / (TILES_PER_CHUNK as i32 * 2),
            middle: 0,
            tile_side,
            chunk_side,
            chunks: HashMap::new(),
            walls: vec![],
        };

        result.dummy_world();
        result
    }

    pub fn debug_stuff(&self) {
        if self.chunks.len() != 0 {
            println!("Chunk count: {}", self.chunks.len());
        // for (_, c) in &self.chunks {
        //     println!("\tcount: {}\tchunk:{:?}", c.entities.len(), c);
        // }
        } else {
            println!("empty");
        }
    }

    fn dummy_world(&mut self) {
        let tiles_per_width = 17;
        let tiles_per_height = 9;

        for screen_y in 0..3 {
            for screen_x in 0..3 {
                for tile_y in 0..tiles_per_height {
                    for tile_x in 0..tiles_per_width {
                        let abs_x = screen_x * tiles_per_width + tile_x;
                        let abs_y = screen_y * tiles_per_height + tile_y;

                        let kind = {
                            if tile_x == 0 || tile_x == tiles_per_width - 1 {
                                if tile_y == 4 {
                                    TileKind::Ground
                                } else {
                                    TileKind::Wall
                                }
                            } else if tile_y == 0 || tile_y == tiles_per_height - 1 {
                                if tile_x == 8 {
                                    TileKind::Ground
                                } else {
                                    TileKind::Wall
                                }
                            } else {
                                TileKind::Ground
                            }
                        };
                        let abs_x = abs_x + self.middle;
                        let abs_y = abs_y + self.middle;
                        let abs_z = 0 + self.middle;
                        if kind == TileKind::Wall {
                            self.walls.push((abs_x, abs_y, abs_z));
                        }
                    }
                }
            }
        }
    }

    pub fn chunk(&self, idx: ChunkIdx) -> Option<&Chunk> {
        self.chunks.get(&idx)
    }

    fn is_canonical(&self, rel: f32) -> bool {
        let epsilon = 0.0001;
        let min = -0.5 * self.chunk_side - epsilon;
        let max = 0.5 * self.chunk_side + epsilon;
        let result = rel >= min && rel <= max;
        if !result {
            println!("min: {}, max: {}, current: {}", min, max, rel);
        }
        result
    }

    fn same_chunk(&self, a: WorldPosition, b: WorldPosition) -> bool {
        assert!(self.is_canonical(a.offset.x()));
        assert!(self.is_canonical(a.offset.y()));
        assert!(self.is_canonical(b.offset.x()));
        assert!(self.is_canonical(b.offset.y()));

        a.abs == b.abs
    }

    fn recanonicalize_coord(&self, pos: i32, rel: f32) -> (i32, f32) {
        let offset = (rel / self.chunk_side).round();
        let new_pos = pos + offset as i32;
        let new_rel = rel - offset * self.chunk_side;
        assert!(self.is_canonical(new_rel));
        (new_pos, new_rel)
    }

    pub fn map_into_chunk_space(&self, base_position: WorldPosition, offset: V2) -> WorldPosition {
        let mut result = base_position;
        result.offset += offset;
        let (abs_x, x) = self.recanonicalize_coord(result.abs.x, result.offset.x());
        let (abs_y, y) = self.recanonicalize_coord(result.abs.y, result.offset.y());
        //TODO constructor
        result.abs.x = abs_x;
        result.abs.y = abs_y;
        result.offset = V2::new(x, y);
        result
    }

    pub fn position_at_tile(&self, abs_x: i32, abs_y: i32, abs_z: i32) -> WorldPosition {
        let chunk_idx = ChunkIdx {
            x: abs_x / TILES_PER_CHUNK as i32,
            y: abs_y / TILES_PER_CHUNK as i32,
            z: abs_z / TILES_PER_CHUNK as i32,
        };
        let x = (abs_x - (chunk_idx.x * TILES_PER_CHUNK as i32)) as f32 * self.tile_side;
        let y = (abs_y - (chunk_idx.y * TILES_PER_CHUNK as i32)) as f32 * self.tile_side;
        let pos = WorldPosition {
            abs: chunk_idx,
            offset: V2::new(x, y),
        };
        self.map_into_chunk_space(pos, V2::default())
    }

    pub fn substract(&self, a: WorldPosition, b: WorldPosition) -> WorldDiff {
        let x = a.abs.x - b.abs.x;
        let y = a.abs.y - b.abs.y;
        let xy = V2::new(x as f32, y as f32);
        let z = a.abs.z - b.abs.z;

        WorldDiff {
            xy: self.chunk_side * xy + (a.offset - b.offset),
            z: self.chunk_side * z as f32,
        }
    }

    pub fn change_entity_chunks(
        &mut self, low_entity_idx: usize, entity: &mut LowEntity, new_p: WorldPosition,
    ) {
        let old_p = if entity.p.is_valid() && entity.entity.spatial {
            Some(entity.p)
        } else {
            None
        };

        let new_p = if new_p.is_valid() { Some(new_p) } else { None };

        let same_chunk = match (old_p, new_p) {
            (Some(old), Some(new)) => self.same_chunk(old, new),
            _ => false,
        };

        if !same_chunk {
            if let Some(old) = old_p {
                assert!(old.is_valid());
                self.chunks
                    .get_mut(&old.abs)
                    .and_then(|chunk| chunk.remove_entity(low_entity_idx))
                    .expect("failed to remove from chunk");
            }
            if let Some(new) = new_p {
                assert!(new.is_valid());
                self.chunks
                    .entry(new.abs)
                    .or_insert_with(|| Chunk::new(new.abs))
                    .entities
                    .push(low_entity_idx);
            }
        }

        if let Some(new) = new_p {
            entity.p = new;
            entity.entity.spatial = true;
        } else {
            entity.p = WorldPosition::default();
            entity.entity.spatial = false;
        }
    }
}
