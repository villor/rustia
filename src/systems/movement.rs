use shipyard::*;
use crate::unique::TileMap;
use crate::components::*;
use crate::hierarchy::*;
use crate::packet_buffers::PacketBuffer;
use crate::protocol::packet::{client, game};
use crate::constants::Direction;

pub fn packet_player_walk(
    mut moves: ViewMut<Move>,
    packet_north: UniqueViewMut<PacketBuffer<client::WalkNorth>>,
    packet_east: UniqueViewMut<PacketBuffer<client::WalkEast>>,
    packet_south: UniqueViewMut<PacketBuffer<client::WalkSouth>>,
    packet_west: UniqueViewMut<PacketBuffer<client::WalkWest>>,
) {
    while let Some((entity_id, _)) = packet_north.poll() {
        (&mut moves).get(entity_id).next_direction = Direction::North;
    }
    while let Some((entity_id, _)) = packet_east.poll() {
        (&mut moves).get(entity_id).next_direction = Direction::East;
    }
    while let Some((entity_id, _)) = packet_south.poll() {
        (&mut moves).get(entity_id).next_direction = Direction::South;
    }
    while let Some((entity_id, _)) = packet_west.poll() {
        (&mut moves).get(entity_id).next_direction = Direction::West;
    }
}

pub fn movement_system(
    mut moves: ViewMut<Move>,
    mut clients: ViewMut<Client>,
    tiles: View<Tile>,
    mut hierarchy: (EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>),
    items: View<Item>,
    creatures: View<Creature>,
    tilemap: UniqueView<TileMap>,
) {
    for (entity_id, m) in (&mut moves).iter()
        .filter(|m| m.next_direction != Direction::None)
        // TODO: Filter out those who cant move yet (check speed)
        .with_id()
    {
        let old_tile_id = hierarchy.2.get(entity_id).parent;
        let old_tile = tiles.get(old_tile_id);
        let old_pos = old_tile.position;
        let old_stack_index = {
            let mut i = 0;
            let mut child = hierarchy.1.get(old_tile_id).first_child;
            while child != entity_id {
                child = hierarchy.2.get(child).next;
                i += 1;
            }
            i
        };
        
        let new_pos = match m.next_direction {
            Direction::North => game::Position { x: old_pos.x, y: old_pos.y - 1, z: old_pos.z },
            Direction::East  => game::Position { x: old_pos.x + 1, y: old_pos.y, z: old_pos.z },
            Direction::South => game::Position { x: old_pos.x, y: old_pos.y + 1, z: old_pos.z },
            Direction::West  => game::Position { x: old_pos.x - 1, y: old_pos.y, z: old_pos.z },
            _ => panic!("Unhandled direction")
        };
        
        let new_tile_id = tilemap.0.get_tile(new_pos); //todo: make sure not dead

        hierarchy.attach(entity_id, new_tile_id);

        // TODO: Filter to spectators only (18x14)
        for (client_entity_id, client) in (&mut clients).iter().with_id() {
            client.send_packet(game::MoveCreature {
                old_position: old_pos,
                old_stack_index,
                new_position: new_pos,
            });

            if client_entity_id == entity_id {
                // We are moving a player, sync world

                let hierarchy = (&hierarchy.1.as_window(..), &hierarchy.2.as_window(..));
                use super::map::get_world_chunk;

                if old_pos.y > new_pos.y { // north
                    let pos = game::Position { x: old_pos.x - 8, y: new_pos.y - 6, z: new_pos.z };
                    client.send_packet(game::WorldRowNorth {
                        world_chunk: get_world_chunk(hierarchy, &tiles, &items, &creatures, &tilemap, pos, 18, 1).unwrap()
                    });
                } else if old_pos.y < new_pos.y { // south
                    let pos = game::Position { x: old_pos.x - 8, y: new_pos.y + 7, z: new_pos.z };
                    client.send_packet(game::WorldRowSouth {
                        world_chunk: get_world_chunk(hierarchy, &tiles, &items, &creatures, &tilemap, pos, 18, 1).unwrap()
                    });
                }

                if old_pos.x < new_pos.x { // east
                    let pos = game::Position { x: new_pos.x + 9, y: new_pos.y - 6, z: new_pos.z };
                    client.send_packet(game::WorldRowEast {
                        world_chunk: get_world_chunk(hierarchy, &tiles, &items, &creatures, &tilemap, pos, 1, 14).unwrap()
                    });
                } else if old_pos.x > new_pos.x { // west
                    let pos = game::Position { x: new_pos.x - 8, y: new_pos.y - 6, z: new_pos.z };
                    client.send_packet(game::WorldRowWest {
                        world_chunk: get_world_chunk(hierarchy, &tiles, &items, &creatures, &tilemap, pos, 1, 14).unwrap()
                    });
                }
            } // TODO: Else, MoveCreature, AddTileThing, or DeleteTilething
        }

        m.next_direction = Direction::None; // TODO: remove this when we are popping it
    }
}
