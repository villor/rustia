use std::{fmt::{self, Display, Formatter}, sync::Arc};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use ahash::AHashMap;
use smallvec::{SmallVec};

use base::Position;

const MAX_LAYERS: usize = 16;

const CHUNK_BITS: u16 = 3;
const CHUNK_SIZE: u16 = 1 << CHUNK_BITS;
const CHUNK_MASK: u16 = CHUNK_SIZE - 1;

pub type ThingId = usize; // change

#[derive(Debug, Default, Clone)]
pub struct Tile {
    things: SmallVec<[ThingId; 10]>,
}

impl Tile {
    // Returns the stack index of the thing in the tile
    pub fn thing_index(&self, thing: ThingId) -> Option<usize> {
        self.things.iter().position(|t| *t == thing)
    }

    /// Returns an iterator of the things in the tile, bottom -> top
    pub fn things_iter(&self) -> impl Iterator<Item = &ThingId> {
        self.things.iter()
    }

    /// Adds a thing to the top of the tile
    pub fn push(&mut self, thing: ThingId) {
        self.things.push(thing);
    }

    /// Remove and return thing at index
    pub fn remove(&mut self, thing_index: usize) -> ThingId {
        self.things.remove(thing_index)
    }

    /// Swap two things in the stack
    pub fn swap(&mut self, a: usize, b: usize) {
        self.things.swap(a, b)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ChunkPosition {
    pub x: u16,
    pub y: u16,
}

impl ChunkPosition {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl From<Position> for ChunkPosition {
    fn from(pos: Position) -> Self {
        Self {
            x: pos.x >> CHUNK_BITS,
            y: pos.y >> CHUNK_BITS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TilePosition {
    x: usize,
    y: usize,
    z: usize,
}

impl From<Position> for TilePosition {
    fn from(pos: Position) -> Self {
        Self {
            x: (pos.x & CHUNK_MASK) as usize,
            y: (pos.y & CHUNK_MASK) as usize,
            z: pos.z as usize
        }
    }
}

impl Display for ChunkPosition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub struct Chunk {
    layers: [Option<Vec<Tile>>; MAX_LAYERS],
    position: ChunkPosition,
}

impl Chunk {
    /// Creates a new empty chunk with the specified position
    fn new(position: ChunkPosition) -> Self {
        Self {
            layers: [
                None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, 
            ],
            position
        }
    }

    /// Gets the position of this chunk
    pub fn position(&self) -> ChunkPosition {
        self.position
    }

    /// Get a reference to the tile at pos
    pub fn tile_at(&self, pos: TilePosition) -> Option<&Tile> {
        match &self.layers[pos.z] {
            Some(tiles) => Some(&tiles[pos.x + pos.y * CHUNK_SIZE as usize]),
            None => None,
        }
    }

    /// Get a mutable reference to the tile at pos
    pub fn tile_at_mut(&mut self, pos: TilePosition) -> Option<&mut Tile> {
        match &mut self.layers[pos.z] {
            Some(tiles) => Some(&mut tiles[pos.x + pos.y * CHUNK_SIZE as usize]),
            None => None,
        }
    }

    pub fn set_tile(&mut self, pos: TilePosition, tile: Tile) {
        if self.layers[pos.z].is_none() {
            self.layers[pos.z] = Some(Vec::with_capacity(CHUNK_SIZE as usize ^ 2));
        }

        if let Some(layer) = &mut self.layers[pos.z] {
            layer[pos.x + pos.y * CHUNK_SIZE as usize] = tile;
        }
    }
}

#[derive(Default)]
pub struct Map {
    width: u16,
    height: u16,
    chunks: AHashMap<ChunkPosition, Arc<RwLock<Chunk>>>,
}

impl Map {
    /// Creates a new empty map with specified width and height
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width, height,
            ..Self::default()
        }
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position(), Arc::new(RwLock::new(chunk)));
    }

    pub fn ensure_chunk(&mut self, pos: ChunkPosition) {
        if !self.chunks.contains_key(&pos) {
            self.insert_chunk(Chunk::new(pos));
        }
    }

    pub fn chunk_at(&self, pos: ChunkPosition) -> Option<RwLockReadGuard<Chunk>> {
        self.chunks.get(&pos).map(|lock| lock.read())
    }

    pub fn chunk_at_mut(&self, pos: ChunkPosition) -> Option<RwLockWriteGuard<Chunk>> {
        self.chunks.get(&pos).map(|lock| lock.write())
    }
}
