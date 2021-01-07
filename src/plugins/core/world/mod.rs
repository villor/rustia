use bevy::prelude::*;
use rand::Rng;
use smallvec::SmallVec;

use super::components::*;
use crate::types::Position;

pub mod tilemap;

pub fn load_map(world: &mut World, resources: &mut Resources) {
    log::info!("Loading/generating map...");

    let mut rng = rand::thread_rng();
    
    let mut tilemap = tilemap::TileMap::new(100, 100);

    for x in 0..100 {
        for y in 0..100 {
            let position = Position { x, y, z: 7 };
            
            let tile = world.reserve_entity();

            let ground_item = world.spawn((
                Item { client_id: rng.gen_range(351..356) },
                TileThing { tile },
            ));

            world.insert_one(tile, Tile {
                position,
                things: SmallVec::from_slice(&[ground_item]),
            }).unwrap();

            tilemap.set_tile(position, tile);
        }
    }

    resources.insert(tilemap);
}
