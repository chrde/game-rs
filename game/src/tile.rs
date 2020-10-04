#[derive(Copy, Clone)]
struct TileRelPosition {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone)]
struct TilePosition {
    x: usize,
    y: usize,
}

//high bits (tile_map.chunk_mask) -> chunk index in tile map
//low bits (tile_map.chunk_shift) -> tile index in chunk
#[derive(Copy, Clone, Eq, PartialEq)]
struct CompressedPosition {
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Copy, Clone)]
struct Position {
    x: usize,
    y: usize,
    z: usize,
}

/// Position of tile in the global map
#[derive(Copy, Clone)]
pub struct TileMapPosition {
    chunk_position: CompressedPosition,

    /// offset from tile center
    rel_position: TileRelPosition,
}

impl TileMapPosition {
    fn same_tile(&self, other: &Self) -> bool {
        self.chunk_position == other.chunk_position
    }
}

/// Position of tile in a chunk
#[derive(Copy, Clone)]
pub struct ChunkPosition {
    position_in_map: Position,

    tile_position: TilePosition,
}

pub struct TileMap {
    chunk_shift: usize,
    chunk_mask: usize,
    chunk_dim: u32,

    count_x: usize,
    count_y: usize,
    count_z: usize,

    tile_side_in_meters: Meter,
    chunks: Vec<TileChunk>,
}

pub struct TileChunk {
    chunk_dim: usize,
    tiles: Vec<Tile>,
}

impl TileChunk {
    fn offset(&self, position: TilePosition) -> usize {
        position.y * self.chunk_dim + position.x
    }

    fn tile(&self, position: TilePosition) -> Option<&Tile> {
        self.tiles.get(self.offset(position))
    }

    fn set_tile(&mut self, position: TilePosition, tile: Tile) {
        let offset = self.offset(position);
        self.tiles[offset] = tile;
    }
}

pub struct Tile {
    kind: TileKind,
}

//TODO what is the value of 2
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TileKind {
    Wall,
    Ground,
    Empty,
}

pub struct Meter(f32);

impl TileMap {
    fn offset(&self, position: Position) -> usize {
        position.z * self.count_y * self.count_x + position.y * self.count_x + position.x
    }

    fn chunk(&self, position: Position) -> Option<&TileChunk> {
        self.chunks.get(self.offset(position))
    }

    fn chunk_mut(&mut self, position: Position) -> Option<&mut TileChunk> {
        let offset = self.offset(position);
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

    fn tile_from_compressed_pos(&self, position: CompressedPosition) -> Option<&Tile> {
        let chunk_pos = self.chunk_position(position);
        self.chunk(chunk_pos.position_in_map)
            .and_then(|c| c.tile(chunk_pos.tile_position))
    }

    fn tile_from_map_pos(&self, position: TileMapPosition) -> Option<&Tile> {
        self.tile_from_compressed_pos(position.chunk_position)
    }

    fn is_tile_empty(&self, position: TileMapPosition) -> bool {
        self.tile_from_map_pos(position)
            .map_or(true, |t| t.kind == TileKind::Empty)
    }

    fn set_tile(&mut self, position: CompressedPosition, tile: Tile) -> bool {
        let chunk_pos = self.chunk_position(position);
        self.chunk_mut(chunk_pos.position_in_map)
            .map(|c| c.set_tile(chunk_pos.tile_position, tile))
            .is_some()
    }

    fn recanonicalize_coord(&self, pos: &mut usize, rel: &mut f32) {
        //TODO only this fn is missing
        todo!()
    }

    fn recanonicalize_position(&self, position: TileMapPosition) -> TileMapPosition {
        let mut new = position;
        self.recanonicalize_coord(&mut new.chunk_position.x, &mut new.rel_position.x);
        self.recanonicalize_coord(&mut new.chunk_position.y, &mut new.rel_position.y);
        new
    }
}
