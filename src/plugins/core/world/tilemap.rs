use bevy::prelude::Entity;
use crate::types::Position;

const FLOOR_COUNT: u8 = 16;

pub struct TileMap {
    width: u16,
    height: u16,
    tiles: Vec<Option<Entity>>,
}

impl TileMap {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![None; width as usize * height as usize * FLOOR_COUNT as usize],
        }
    }

    pub fn width(&self) -> u16 { self.width }
    pub fn height(&self) -> u16 { self.height }

    fn get_index(&self, pos: Position) -> usize {
        pos.x as usize
        + pos.y as usize * self.height as usize
        + pos.z as usize * self.height as usize * self.width as usize
    }

    pub fn clear_tile(&mut self, pos: Position) {
        let index = self.get_index(pos);
        self.tiles[index] = None;
    }

    pub fn set_tile(&mut self, pos: Position, entity: Entity) {
        let index = self.get_index(pos);
        self.tiles[index] = Some(entity);
    }

    pub fn get_tile(&self, pos: Position) -> Option<Entity> {
        self.tiles[self.get_index(pos)]
    }
}
