use std::collections::HashMap;

use super::*;

#[derive(Copy, Clone, Debug)]
pub struct PositionDiff {
    pub xy: V2,
    pub z: f32,
}

#[derive(Copy, Clone, Debug)]
struct TilePosition {
    x: i32,
    y: i32,
}

//high bits (tile_map.chunk_mask) -> chunk index in tile map
//low bits (tile_map.chunk_shift) -> tile index in chunk
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AbsPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

//lower 8 bits are zero
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct ChunkIdx {
    x: i32,
    y: i32,
    z: i32,
}

/// Position of tile in the global map
#[derive(Copy, Clone, Debug)]
pub struct TileMapPosition {
    pub abs: AbsPosition,

    /// offset from tile center
    pub offset: V2,
}

/// Position of tile in a chunk
#[derive(Copy, Clone, Debug)]
pub struct ChunkPosition {
    position_in_map: ChunkIdx,

    tile_position: TilePosition,
}

#[derive(Debug)]
pub struct TileMap {
    middle: i32,
    chunk_shift: i32,
    chunk_mask: i32,
    chunk_dim: i32,
    pub tile_size: Meter,
    chunks: HashMap<ChunkIdx, TileChunk>,
    pub walls: Vec<AbsPosition>,
}

#[derive(Clone, Debug)]
pub struct TileChunk {
    chunk_dim: i32,
    tiles: Vec<Tile>,
}

impl TileChunk {
    fn new(chunk_dim: i32) -> Self {
        Self {
            chunk_dim,
            tiles: vec![],
        }
    }

    fn offset(&self, position: TilePosition) -> i32 {
        position.y * self.chunk_dim + position.x
    }

    fn tile(&self, position: TilePosition) -> Option<&Tile> {
        self.tiles.get(self.offset(position) as usize)
    }

    fn set_tile(&mut self, position: TilePosition, tile: Tile) {
        let offset = self.offset(position) as usize;
        while self.tiles.len() <= offset {
            self.tiles.push(Tile {
                kind: TileKind::Empty,
            });
        }
        self.tiles[offset] = tile;
        // self.tiles.push(tile);
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
    Empty,
}

#[derive(Copy, Clone, Debug)]
pub struct Meter(pub f32);

impl TileMap {
    pub fn initial_player(&self) -> TileMapPosition {
        TileMapPosition {
            abs: AbsPosition {
                x: self.middle + 1,
                y: self.middle + 3,
                z: self.middle + 0,
            },
            offset: V2::default(),
        }
    }

    pub fn initial_camera(&self) -> TileMapPosition {
        TileMapPosition {
            abs: AbsPosition {
                x: self.middle + 17 / 2,
                y: self.middle + 9 / 2,
                z: self.middle + 0,
            },
            offset: V2::default(),
        }
    }

    pub fn new() -> Self {
        let chunk_shift = 4;
        let mut result = Self {
            middle: std::i16::MAX as i32 / 32,
            chunk_shift,
            chunk_mask: (1 << chunk_shift) - 1,
            chunk_dim: (1 << chunk_shift),
            tile_size: Meter(1.4),
            chunks: HashMap::new(),
            walls: vec![],
        };

        result.dummy_world();
        result
    }

    fn dummy_world(&mut self) {
        let tiles_per_width = 17;
        let tiles_per_height = 9;

        for screen_y in 0..1 {
            for screen_x in 0..1 {
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
                        let position = AbsPosition {
                            x: abs_x,
                            y: abs_y,
                            z: abs_z,
                        };
                        println!("wall {:X?}", position);
                        let tile = Tile { kind };
                        if kind == TileKind::Wall {
                            self.walls.push(position);
                        }
                        // self.set_tile(position, tile);
                    }
                }
            }
        }
    }

    fn chunk(&self, chunk: ChunkIdx) -> Option<&TileChunk> {
        self.chunks.get(&chunk)
    }

    fn chunk_mut(&mut self, position: ChunkIdx) -> Option<&mut TileChunk> {
        self.chunks.get_mut(&position)
    }

    fn abs(&self, position: AbsPosition) -> ChunkPosition {
        let position_in_map = ChunkIdx {
            x: position.x >> self.chunk_shift,
            y: position.y >> self.chunk_shift,
            z: position.z,
        };
        let tile_position = TilePosition {
            x: position.x & self.chunk_mask,
            y: position.y & self.chunk_mask,
        };
        ChunkPosition {
            position_in_map,
            tile_position,
        }
    }

    pub fn tile_from_compressed_pos(&self, position: AbsPosition) -> Option<&Tile> {
        let chunk_pos = self.abs(position);
        self.chunk(chunk_pos.position_in_map)
            .and_then(|c| c.tile(chunk_pos.tile_position))
    }

    fn tile_from_map_pos(&self, position: TileMapPosition) -> Option<&Tile> {
        self.tile_from_compressed_pos(position.abs)
    }

    pub fn is_tile_empty(&self, position: TileMapPosition) -> bool {
        self.tile_from_map_pos(position)
            .map_or(true, |t| t.kind != TileKind::Wall)
    }

    fn set_tile(&mut self, position: AbsPosition, tile: Tile) -> bool {
        let chunk_pos = self.abs(position);
        let chunk = self.chunk_mut(chunk_pos.position_in_map).unwrap();
        chunk.set_tile(chunk_pos.tile_position, tile);
        true
    }

    fn recanonicalize_coord(&self, pos: i32, rel: f32) -> (i32, f32) {
        let offset = (rel / self.tile_size.0).round();
        let new_pos = pos as f32 + offset;
        let new_rel = rel - offset * self.tile_size.0;
        assert!(new_rel > -0.5 * self.tile_size.0);
        assert!(new_rel < 0.5 * self.tile_size.0);
        (new_pos as i32, new_rel)
    }

    pub fn map_into_tile_space(
        &self,
        base_position: TileMapPosition,
        offset: V2,
    ) -> TileMapPosition {
        let mut result = base_position;
        result.offset += offset;
        let (abs_x, x) = self.recanonicalize_coord(result.abs.x, result.offset.x());
        let (abs_y, y) = self.recanonicalize_coord(result.abs.y, result.offset.y());
        result.abs.x = abs_x;
        result.abs.y = abs_y;
        result.offset = V2::new(x, y);
        result
    }

    pub fn substract(&self, a: TileMapPosition, b: TileMapPosition) -> PositionDiff {
        let x = a.abs.x as f32 - b.abs.x as f32;
        let y = a.abs.y as f32 - b.abs.y as f32;
        let xy = V2::new(x, y);
        let z = a.abs.z as f32 - b.abs.z as f32;

        PositionDiff {
            xy: self.tile_size.0 * xy + (a.offset - b.offset),
            z: self.tile_size.0 * z,
        }
    }

    pub fn centered_tile_point(x: i32, y: i32, z: i32) -> TileMapPosition {
        TileMapPosition {
            abs: AbsPosition { x, y, z },
            offset: V2::default(),
        }
    }
}
