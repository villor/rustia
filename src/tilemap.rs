use shipyard::EntityId;
use crate::protocol::packet::game::Position;

pub struct TileMap {
    width: u16,
    height: u16,
    tiles: Vec<EntityId>,
}

impl TileMap {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![EntityId::dead(); width as usize * height as usize * 16],
        }
    }

    fn get_index(&self, pos: Position) -> usize {
        pos.x as usize
        + pos.y as usize * self.height as usize
        + pos.z as usize * self.height as usize * self.width as usize
    }

    pub fn clear_tile(&mut self, pos: Position) {
        let index = self.get_index(pos);
        self.tiles[index] = EntityId::dead();
    }

    pub fn set_tile(&mut self, pos: Position, entity: EntityId) {
        let index = self.get_index(pos);
        self.tiles[index] = entity;
    }

    pub fn get_tile(&self, pos: Position) -> EntityId {
        self.tiles[self.get_index(pos)]
    }
}
