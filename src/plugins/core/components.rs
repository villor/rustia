//use bevy::prelude::*;
use crate::types::Position;

#[derive(Debug)]
pub struct TileThing {
    pub position: Position,
}

pub struct Creature {
    pub id: u32,
    pub name: String,
}

pub struct Player;
pub struct Monster;
pub struct Npc;

pub struct Item {
    pub client_id: u16,
}
