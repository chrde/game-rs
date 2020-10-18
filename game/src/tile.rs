use super::*;

#[derive(Copy, Clone, Debug)]
pub struct PositionDiff {
    pub xy: V2,
    pub z: f32,
}

#[derive(Copy, Clone, Debug)]
struct TilePosition {
    x: usize,
    y: usize,
}

//high bits (tile_map.chunk_mask) -> chunk index in tile map
//low bits (tile_map.chunk_shift) -> tile index in chunk
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CompressedPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Copy, Clone, Debug)]
struct Position {
    x: usize,
    y: usize,
    z: usize,
}

/// Position of tile in the global map
#[derive(Copy, Clone, Debug)]
pub struct TileMapPosition {
    pub chunk_position: CompressedPosition,

    /// offset from tile center
    pub offset: V2,
}

impl TileMapPosition {
    pub fn initial_camera() -> Self {
        Self {
            chunk_position: CompressedPosition {
                x: 17 / 2,
                y: 9 / 2,
                z: 0,
            },
            offset: V2::default(),
        }
    }

    pub fn initial_player() -> Self {
        Self {
            chunk_position: CompressedPosition { x: 1, y: 3, z: 0 },
            offset: V2::default(),
        }
    }

    // fn same_tile(&self, other: &Self) -> bool {
    //     self.chunk_position == other.chunk_position
    // }
}

/// Position of tile in a chunk
#[derive(Copy, Clone, Debug)]
pub struct ChunkPosition {
    position_in_map: Position,

    tile_position: TilePosition,
}

#[derive(Debug)]
pub struct TileMap {
    chunk_shift: usize,
    chunk_mask: usize,
    // chunk_dim: u32,
    count_x: usize,
    count_y: usize,
    count_z: usize,

    pub tile_size: Meter,
    chunks: Vec<TileChunk>,
}

#[derive(Clone, Debug)]
pub struct TileChunk {
    chunk_dim: usize,
    tiles: Vec<Tile>,
}

impl TileChunk {
    fn new(chunk_dim: usize) -> Self {
        Self {
            chunk_dim,
            tiles: vec![],
        }
    }

    fn offset(&self, position: TilePosition) -> usize {
        position.y * self.chunk_dim + position.x
    }

    fn tile(&self, position: TilePosition) -> Option<&Tile> {
        self.tiles.get(self.offset(position))
    }

    fn set_tile(&mut self, position: TilePosition, tile: Tile) {
        let offset = self.offset(position);
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

//TODO what is the value of 2
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TileKind {
    Wall,
    Ground,
    Empty,
}

#[derive(Copy, Clone, Debug)]
pub struct Meter(pub f32);

impl TileMap {
    pub fn new() -> Self {
        let chunk_shift = 4;
        let mut result = Self {
            chunk_shift,
            chunk_mask: (1 << chunk_shift) - 1,
            // chunk_dim: (1 << chunk_shift),
            count_x: 128,
            count_y: 128,
            count_z: 2,

            tile_size: Meter(1.4),
            chunks: vec![],
        };

        let size = result.count_x * result.count_y * result.count_z;
        // let size = 2;
        result.chunks = vec![TileChunk::new(1 << chunk_shift); size];

        result.dummy_world();
        // panic!();
        result
        // dbg!(result)
    }

    fn dummy_world(&mut self) {
        let tiles_per_width = 17;
        let tiles_per_height = 9;

        for screen_y in 0..10 {
            for screen_x in 0..10 {
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
                        let position = CompressedPosition {
                            x: abs_x,
                            y: abs_y,
                            z: 0,
                        };
                        let tile = Tile { kind };
                        self.set_tile(position, tile);
                    }
                }
            }
        }
    }

    fn offset_idx(&self, position: Position) -> usize {
        position.z * self.count_y * self.count_x + position.y * self.count_x + position.x
    }

    fn chunk(&self, position: Position) -> Option<&TileChunk> {
        self.chunks.get(self.offset_idx(position))
    }

    fn chunk_mut(&mut self, position: Position) -> Option<&mut TileChunk> {
        let offset = self.offset_idx(position);
        self.chunks.get_mut(offset)
    }

    fn chunk_position(&self, position: CompressedPosition) -> ChunkPosition {
        let position_in_map = Position {
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

    pub fn tile_from_compressed_pos(&self, position: CompressedPosition) -> Option<&Tile> {
        let chunk_pos = self.chunk_position(position);
        self.chunk(chunk_pos.position_in_map)
            .and_then(|c| c.tile(chunk_pos.tile_position))
    }

    fn tile_from_map_pos(&self, position: TileMapPosition) -> Option<&Tile> {
        self.tile_from_compressed_pos(position.chunk_position)
    }

    pub fn is_tile_empty(&self, position: TileMapPosition) -> bool {
        self.tile_from_map_pos(position)
            .map_or(true, |t| t.kind != TileKind::Wall)
    }

    fn set_tile(&mut self, position: CompressedPosition, tile: Tile) -> bool {
        let chunk_pos = self.chunk_position(position);
        let chunk = self.chunk_mut(chunk_pos.position_in_map).unwrap();
        chunk.set_tile(chunk_pos.tile_position, tile);
        true
    }

    fn recanonicalize_coord(&self, pos: usize, rel: f32) -> (usize, f32) {
        let offset = (rel / self.tile_size.0).round();
        let new_pos = pos as f32 + offset;
        let new_rel = rel - offset * self.tile_size.0;
        assert!(new_rel > -0.5001 * self.tile_size.0);
        assert!(new_rel < 0.5001 * self.tile_size.0);
        (new_pos as usize, new_rel)
    }

    fn recanonicalize_position(&self, position: TileMapPosition) -> TileMapPosition {
        let (abs_x, x) = self.recanonicalize_coord(position.chunk_position.x, position.offset.x());
        let (abs_y, y) = self.recanonicalize_coord(position.chunk_position.y, position.offset.y());
        let mut result = position;
        result.chunk_position.x = abs_x;
        result.chunk_position.y = abs_y;
        result.offset = V2::new(x, y);
        result
    }

    pub fn substract(&self, a: TileMapPosition, b: TileMapPosition) -> PositionDiff {
        let x = a.chunk_position.x as f32 - b.chunk_position.x as f32;
        let y = a.chunk_position.y as f32 - b.chunk_position.y as f32;
        let xy = V2::new(x, y);
        let z = a.chunk_position.z as f32 - b.chunk_position.z as f32;

        PositionDiff {
            xy: self.tile_size.0 * xy + (a.offset - b.offset),
            z: self.tile_size.0 * z,
        }
    }

    pub fn centered_tile_point(x: usize, y: usize, z: usize) -> TileMapPosition {
        TileMapPosition {
            chunk_position: CompressedPosition { x, y, z },
            offset: V2::default(),
        }
    }

    pub fn offset(&self, position: TileMapPosition, offset: V2) -> TileMapPosition {
        let mut position = position;
        position.offset += offset;
        self.recanonicalize_position(position)
    }
}
