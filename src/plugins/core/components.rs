use bevy::prelude::*;
use smallvec::SmallVec;
use crate::types::Position;

pub struct Player;

#[derive(Debug, Default)]
pub struct Tile {
    pub position: Position,
    pub things: SmallVec<[Entity; 10]>,
}

#[derive(Debug)]
pub struct TileThing {
    pub tile: Entity,
}

pub struct Creature {
    pub id: u32,
}

pub struct Item {
    pub client_id: u16,
}
