use shipyard::*;

use crate::constants::Direction;
use crate::components::*;
use crate::unique::*;
use crate::hierarchy::*;
use crate::network::game_listener::ServerToWorkerMessage;
use crate::protocol::packet::game::{self, Position};
use crate::packet_buffers::PacketTransmitter;

use super::map;

#[allow(clippy::too_many_arguments)]
pub fn player_connect_system(
    mut hierarchy: (EntitiesViewMut, ViewMut<Parent>, ViewMut<Child>),
    mut creatures: ViewMut<Creature>,
    mut moves: ViewMut<Move>,
    mut clients: ViewMut<Client>,
    tiles: View<Tile>,
    items: View<Item>,
    new_client_rx: UniqueViewMut<NewClientRx>,
    creature_id_counter: UniqueView<CreatureIdCounter>,
    tilemap: UniqueViewMut<TileMap>,
    packet_tx_template: UniqueViewMut<PacketTransmitter>,
) {
    while let Ok(new_client) = new_client_rx.0.try_recv() {
        let player_name = new_client.player_name;
        let mut new_client = Client {
            addr: new_client.addr,
            sender: new_client.sender,
            receiver: new_client.receiver,
            dirty: false,
        };

        let player_id = creature_id_counter.0.inc();
        let spawn_position = Position { x: 50, y: 50, z: 7 };
        
        let tile = tilemap.0.get_tile(spawn_position); // TODO: check if tile is dead

        let player = hierarchy.0.add_entity(
            (&mut creatures, &mut moves),
            (Creature { id: player_id }, Move { next_direction: Direction::None, speed: 200 }),
        );
        
        hierarchy.attach(player, tile);

        let _ = new_client.sender.send(
            ServerToWorkerMessage::PacketTransmitter(packet_tx_template.clone_for_client(player))
        );

        new_client.send_packet(make_login_success(player_id));
        new_client.send_packet(game::PendingStateEntered::default());
        new_client.send_packet(game::EnterWorld::default());
        new_client.send_packet(game::FullWorld {
            player_position: Position { x: spawn_position.x, y: spawn_position.y, z: spawn_position.z },
            world_chunk: map::get_world_chunk(
                (&hierarchy.1.as_window(..), &hierarchy.2.as_window(..)),
                &tiles,
                &items,
                &creatures.as_window(..),
                &tilemap,
                Position { x: 50 - 8, y: 50 - 6, z: 7 },
                18,
                14,
            ).unwrap(),
        });
        new_client.send_packet(game::WorldLight {
            light: game::LightInfo { light_level: 0xFF, light_color: 0xD7 }
        });
        new_client.send_packet(game::CreatureLight {
            creature_id: player_id,
            light: game::LightInfo { light_level: 0x00, light_color: 0x00 },
        });
        new_client.send_packet(game::PlayerDataBasic {
            is_premium: false,
            premium_until: 0,
            vocation_id: 0,
            known_spells: vec![],
        });
        new_client.flush_packets();

        hierarchy.0.add_component(&mut clients, new_client, player);

        // TODO: Only send to spectators
        for (client_id, client) in (&mut clients).iter().with_id() {
            if client_id != player {
                client.send_packet(game::AddTileThing {
                    position: spawn_position,
                    stack_index: hierarchy.1.get(tile).num_children as u8 - 1,
                    thing: game::Thing::Creature(game::Creature {
                        id: player_id,
                        known: game::CreatureKnown::No {
                            remove: 0,
                            creature_type: 0, // player
                            creature_name: player_name.clone(),
                            guild_emblem: 0,
                        },
                        health: 50,
                        direction: 2, //south
                        outfit: game::Outfit::LookType {
                            look_type: 128, // male citizen
                            head: 0,
                            body: 0,
                            legs: 0,
                            feet: 0,
                            addons: 0,
                            mount: 0,
                        },
                        light: game::LightInfo { light_level: 0xFF, light_color: 0xFF },
                        speed: 200, 
                        skull: 0, //none
                        shield: 0, // none
                        summon_type: 0,
                        speech_bubble: 0,
                        helpers: 0x00,
                        walk_through: false,
                    }),
                });
            }
        }
    }
}

fn make_login_success(player_id: u32) -> game::LoginSuccess {
    game::LoginSuccess {
        player_id,
        beat_duration: 0x32, // 50 from tfs
        speed_a: 857.36,
        speed_b: 261.29,
        speed_c: -4795.01,
        is_tutor: false,
        pvp_framing: false,
        expert_mode: false,
        store_img_url: String::new(),
        coin_package_size: 25,
    }
}

pub fn packet_flush_system(
    mut clients: ViewMut<Client>,
) {
    for client in (&mut clients).iter() {
        client.flush_packets();
    }
}



