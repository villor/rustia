use shipyard::*;
use crate::components::*;
use crate::hierarchy::*;
use crate::unique::*;
use crate::protocol::packet::game as packet;
use crate::protocol::packet::game::Position;
use rand::Rng;
use anyhow::Result;

pub fn init(world: &World) {
    log::info!("Loading/generating map...");
    let mut rng = rand::thread_rng();

    let mut tilemap = crate::tilemap::TileMap::new(100, 100);

    world.run(
        |mut hierarchy: (EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>),
         mut tiles: ViewMut<Tile>,
         mut items: ViewMut<Item>| {
            for x in 0..100 {
                for y in 0..100 {
                    let tile = hierarchy.0.add_entity(&mut tiles, Tile {
                        position: Position { x, y, z: 7 }
                    });
                    let ground = hierarchy.0.add_entity(&mut items, Item {
                        client_id: rng.gen_range(351..356)
                    });

                    hierarchy.attach(ground, tile);

                    tilemap.set_tile(Position { x: x as u16, y: y as u16, z: 7 }, tile);
                }
            }
        }
    );

    world.add_unique(crate::unique::TileMap(tilemap));
}

/// Gets a world chunk from the ECS and converts it into WorldData for packets
#[allow(clippy::too_many_arguments)]
pub fn get_world_chunk(
    hierarchy: (&Window<Parent>, &Window<Child>),
    tiles: &Window<Tile>,
    items: &Window<Item>,
    creatures: &Window<Creature>,
    tilemap: &TileMap,
    pos: Position,
    width: u16,
    height: u16,
) -> Result<Vec<packet::WorldData>> {
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
            hierarchy,
            tiles,
            items,
            creatures,
            tilemap,
            Position { x: pos.x, y: pos.y, z: nz as u8 }, // todo: offset
            width, height,
            &mut out,
        )?;
        nz += zstep;
    }

    Ok(out)
}

/// Gets a world layer from the ECS and converts it into WorldData for packets
/// Expects a pre-allocated vector for output
#[allow(clippy::too_many_arguments)]
pub fn get_world_layer(
    hierarchy: (&Window<Parent>, &Window<Child>),
    tiles: &Window<Tile>,
    items: &Window<Item>,
    creatures: &Window<Creature>,
    tilemap: &TileMap,
    pos: Position,
    width: u16, height: u16,
    out: &mut Vec<packet::WorldData>,
) -> Result<()> {
    let mut skip = 0;
    for nx in 0..width {
        for ny in 0..height {
            let tile = tilemap.0.get_tile(Position { x: pos.x + nx, y: pos.y + ny, z: pos.z });
            if tile != EntityId::dead() {
                if skip > 0 {
                    out.push(packet::WorldData::Empty(skip));
                    skip = 0;
                }

                out.push(packet::WorldData::Tile(get_tile(
                    tile,
                    hierarchy,
                    tiles,
                    items,
                    creatures,
                )?));
            } else {
                skip += 1;
            }
        }
    }

    if skip > 0 {
        out.push(packet::WorldData::Empty(skip));
    }

    Ok(())
}

/// Gets a tile from the ECS and converts it to a packet Tile for use in WorldData
#[allow(clippy::too_many_arguments)]
pub fn get_tile(
    tile: EntityId,
    hierarchy: (&Window<Parent>, &Window<Child>),
    _tiles: &Window<Tile>,
    items: &Window<Item>,
    creatures: &Window<Creature>,
) -> Result<packet::Tile> {
    
    let mut things = hierarchy.descendants(tile)
        .filter_map(|id| {
            if let Ok(item) = items.try_get(id) {
                Some(packet::Thing::Item(packet::Item {
                    client_id: item.client_id,
                    stack_size: None,
                    fluid: None,
                    animation: None,
                }))
            } else if let Ok(creature) = creatures.try_get(id) {
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

    Ok(packet::Tile {
        environmental_effects: 0x00,
        things: [
            things.next(), things.next(),
            things.next(), things.next(),
            things.next(), things.next(),
            things.next(), things.next(),
            things.next(), things.next(),
        ],
    })
}


