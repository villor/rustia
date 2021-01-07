use bevy::prelude::*;
/*use crate::{
    components::Client,
    unique::CreatureIdCounter,
};*/

pub mod packet_buffers;
pub mod protocol;
mod components;
mod tokio_runtime;
mod network;
mod packet_conversion;

use network::game_listener::{NewClientInfo, ServerToWorkerMessage};
use packet_buffers::PacketTransmitter;
use protocol::packet::game;
use super::core::{resources::CreatureIdCounter, world::tilemap::TileMap};
use super::core::components::*;
use crate::types::Position;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let (newclient_tx, newclient_rx) = flume::unbounded();

        tokio_runtime::start(async move {
            let login_listener = network::login_listener::listen();
            let game_listener = network::game_listener::listen(newclient_tx);
            futures::join!(login_listener, game_listener)
        });

        packet_buffers::init(app);

        app
            .add_resource(NewClientRx(newclient_rx))
            .add_system(player_connect_system.system());
    }
}

pub struct NewClientRx(pub flume::Receiver<NewClientInfo>);

fn player_connect_system(world: &mut World, resources: &mut Resources) {
    let new_client_rx = resources.get::<NewClientRx>().unwrap();
    while let Ok(new_client) = new_client_rx.0.try_recv() {
        log::info!("Player connected {}", new_client.player_name);

        // Fetch resources
        let creature_id_counter = resources.get::<CreatureIdCounter>().unwrap();
        let tilemap = resources.get::<TileMap>().unwrap();
        let packet_tx = resources.get::<PacketTransmitter>().unwrap();

        let mut new_client = components::Client {
            addr: new_client.addr,
            sender: new_client.sender,
            receiver: new_client.receiver,
            dirty: false,
        };

        let player_id = creature_id_counter.0.inc();
        let spawn_position = Position { x: 50, y: 50, z: 7 };

        let tile = tilemap.get_tile(spawn_position).unwrap(); // TODO: check if tile exists

        let player = world.spawn((
            Player,
            Creature { id: player_id },
            TileThing { tile },
        ));
        world.get_mut::<Tile>(tile).unwrap().things.push(player);

        let _ = new_client.sender.send(
            ServerToWorkerMessage::PacketTransmitter(packet_tx.clone_for_client(player))
        );

        new_client.send_packet(make_login_success(player_id));
        new_client.send_packet(game::PendingStateEntered::default());
        new_client.send_packet(game::EnterWorld::default());
        new_client.send_packet(game::FullWorld {
            player_position: spawn_position,
            world_chunk: packet_conversion::convert_world_chunk(
                world,
                &tilemap,
                Position { x: spawn_position.x - 8, y: spawn_position.y - 6, z: 7 },
                18,
                14,
            ),
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

        world.insert_one(player, new_client).unwrap();
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
