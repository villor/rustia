use bevy::prelude::Entity;
use smallvec::SmallVec;
use crate::types::Position;

const FLOOR_COUNT: u8 = 16;
const INLINE_THINGS: usize = 10;

#[derive(Debug, Default, Clone)]
pub struct Tile {
    pub environmental_effects: u8,
    things: SmallVec<[Entity; INLINE_THINGS]>,
}

impl Tile {
    // Returns the stack index of the thing in the tile
    pub fn thing_index(&self, thing: Entity) -> Option<usize> {
        self.things.iter().position(|t| *t == thing)
    }

    /// Returns an iterator of the things in the tile, bottom -> top
    pub fn things_iter(&self) -> impl Iterator<Item = &Entity> {
        self.things.iter()
    }

    /// Adds a thing to the top of the tile
    pub fn push(&mut self, thing: Entity) {
        self.things.push(thing);
    }

    /// Returns true if the tile is empty and has no environmental effects
    pub fn is_dead(&self) -> bool {
        self.environmental_effects == 0x00 && self.things.is_empty()
    }
}

#[derive(Debug)]
pub struct TileMap {
    width: u16,
    height: u16,
    tiles: Vec<Tile>,
}

impl TileMap {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::default(); width as usize * height as usize * FLOOR_COUNT as usize],
        }
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    fn index(&self, pos: Position) -> usize {
        pos.x as usize
        + pos.y as usize * self.height as usize
        + pos.z as usize * self.height as usize * self.width as usize
    }

    pub fn get_tile(&self, pos: Position) -> &Tile {
        &self.tiles[self.index(pos)]
    }

    pub fn get_tile_mut(&mut self, pos: Position) -> &mut Tile {
        let i = self.index(pos);
        &mut self.tiles[i]
    }

    pub fn thing_index(&self, pos: Position, thing: Entity) -> Option<usize> {
        self.tiles[self.index(pos)].thing_index(thing)
    }
}
