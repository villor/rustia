use bevy::prelude::*;

use crate::plugins::core::components::*;
use crate::plugins::core::world::tilemap::TileMap;
use crate::types::Position;
use super::protocol::packet::game as packet;

pub fn convert_world_chunk(
    world: &World,
    tilemap: &TileMap,
    pos: Position,
    width: u16,
    height: u16,
) -> Vec<packet::WorldData> {
    let (startz, endz, zstep) = if pos.z > 7 {
        // underground
        (pos.z as i8 - 2, std::cmp::min(15, pos.z + 2) as i8, 1 as i8)
    } else {
        (7, 0, -1)
    };

    let mut out =
        Vec::<packet::WorldData>::with_capacity(
            width as usize * height as usize
            * (startz - endz - zstep).abs() as usize);
    
    let mut nz = startz;
    while nz != endz + zstep {
        get_world_layer(
            world,
            tilemap,
            Position { x: pos.x, y: pos.y, z: nz as u8 }, // todo: offset
            width, height,
            &mut out,
        );
        nz += zstep;
    }

    log::info!("{:#?}", out);

    out
}

pub fn get_world_layer(
    world: &World,
    tilemap: &TileMap,
    pos: Position,
    width: u16, height: u16,
    out: &mut Vec<packet::WorldData>,
) {
    let mut skip = 0;
    for nx in 0..width {
        for ny in 0..height {
            if let Some(tile) = tilemap.get_tile(Position { x: pos.x + nx, y: pos.y + ny, z: pos.z }) {
                if skip > 0 {
                    out.push(packet::WorldData::Empty(skip));
                    skip = 0;
                }

                out.push(packet::WorldData::Tile(convert_tile(world, tile)));
            } else {
                skip += 1;
            }
        }
    }

    if skip > 0 {
        out.push(packet::WorldData::Empty(skip));
    }
}

pub fn convert_tile(world: &World, entity: Entity) -> packet::Tile {
    if let Ok(tile) = world.get::<Tile>(entity) {
        let mut things = tile.things.iter().filter_map(|thing| {
            if let Ok(item) = world.get::<Item>(*thing) {
                Some(packet::Thing::Item(packet::Item {
                    client_id: item.client_id,
                    stack_size: None,
                    fluid: None,
                    animation: None,
                }))
            } else if let Ok(creature) = world.get::<Creature>(*thing) {
                Some(packet::Thing::Creature(packet::Creature {
                    id: creature.id,
                    known: packet::CreatureKnown::No {
                        remove: 0,
                        creature_type: 0, // player
                        creature_name: String::from("Hello"),
                        guild_emblem: 0,
                    },
                    health: 50,
                    direction: 2, //south
                    outfit: packet::Outfit::LookType {
                        look_type: 128, // male citizen
                        head: 0,
                        body: 0,
                        legs: 0,
                        feet: 0,
                        addons: 0,
                        mount: 0,
                    },
                    light: packet::LightInfo { light_level: 0xFF, light_color: 0xFF },
                    speed: 200, 
                    skull: 0, //none
                    shield: 0, // none
                    summon_type: 0,
                    speech_bubble: 0,
                    helpers: 0x00,
                    walk_through: false,
                }))
            } else {
                None
            }
        });

        packet::Tile {
            environmental_effects: 0x00,
            things: [
                things.next(), things.next(),
                things.next(), things.next(),
                things.next(), things.next(),
                things.next(), things.next(),
                things.next(), things.next(),
            ],
        }
    } else {
        packet::Tile {
            environmental_effects: 0x00,
            things: [None, None, None, None, None, None, None, None, None, None],
        }
    }
}
