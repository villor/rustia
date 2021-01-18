use bevy::prelude::*;
/*use crate::{
    components::Client,
    unique::CreatureIdCounter,
};*/

pub mod packet_buffers;
pub mod protocol;
pub mod components;
mod tokio_runtime;
mod network;
mod packet_conversion;

use network::game_listener::{NewClientInfo, ServerToWorkerMessage};
use packet_buffers::{PacketBuffer, PacketTransmitter};
use packet_conversion::*;
use protocol::packet::{self, game};
use components::*;
use super::core::world::tilemap::TileMap;
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
            .add_system_to_stage(stage::PRE_UPDATE, receive_movement.system())
            .add_system_to_stage(stage::POST_UPDATE, new_client_system.system())
            .add_system_to_stage(stage::POST_UPDATE, send_login_success.system())
            .add_system_to_stage(stage::POST_UPDATE, send_add_tile_thing.system());
    }
}

pub struct NewClientRx(pub flume::Receiver<NewClientInfo>);

/// Receives new clients from the listener thread and spawns entities
fn new_client_system(world: &mut World, resources: &mut Resources) {
    let new_client_rx = resources.get::<NewClientRx>().unwrap();
    while let Ok(new_client) = new_client_rx.0.try_recv() {
        log::debug!("New client connected. Spawning entity...");
        world.spawn((Client::new(new_client),));
    }
}

fn receive_movement(
    north: Res<PacketBuffer<packet::client::WalkNorth>>,
    west: Res<PacketBuffer<packet::client::WalkWest>>,
    south: Res<PacketBuffer<packet::client::WalkSouth>>,
    east: Res<PacketBuffer<packet::client::WalkEast>>,
) {
    
}

/// Send login success and initial data when a new player is added to an entity with a Client component
fn send_login_success(world: &mut World, resources: &mut Resources) {
    let tilemap = resources.get::<TileMap>().unwrap();
    let packet_tx = resources.get::<PacketTransmitter>().unwrap();
    
    for (entity, creature, tile_thing, client) in world.query_filtered::<(Entity, &Creature, &TileThing, &Client), Added<Player>>() {
        log::debug!("Player spawned. Sending login success to client...");

        let _ = client.sender.send(
            ServerToWorkerMessage::PacketTransmitter(packet_tx.clone_for_client(entity))
        );

        client.send_packet(game::LoginSuccess {
            player_id: creature.id,
            beat_duration: 0x32, // 50 from tfs
            speed_a: 857.36,
            speed_b: 261.29,
            speed_c: -4795.01,
            is_tutor: false,
            pvp_framing: false,
            expert_mode: false,
            store_img_url: String::new(),
            coin_package_size: 25,
        });
        client.send_packet(game::PendingStateEntered::default());
        client.send_packet(game::EnterWorld::default());
        client.send_packet(game::FullWorld {
            player_position: tile_thing.position,
            world_chunk: packet_conversion::convert_world_chunk(
                world,
                &tilemap,
                Position { x: tile_thing.position.x - 8, y: tile_thing.position.y - 6, z: 7 },
                18,
                14,
            ),
        });
        client.send_packet(game::WorldLight {
            light: game::LightInfo { light_level: 0xFF, light_color: 0xD7 }
        });
        client.send_packet(game::CreatureLight {
            creature_id: creature.id,
            light: game::LightInfo { light_level: 0x00, light_color: 0x00 },
        });
        client.send_packet(game::PlayerDataBasic {
            is_premium: false,
            premium_until: 0,
            vocation_id: 0,
            known_spells: vec![],
        });
        client.flush_packets();
    }
}

fn send_add_tile_thing(world: &mut World, resources: &mut Resources) {
    let tilemap = resources.get::<TileMap>().unwrap();
    // TODO: Make sure this doesnt run for tile things added during map load
    for (entity, tile_thing) in world.query_filtered::<(Entity, &TileThing), Added<TileThing>>() {
        log::debug!("Tile thing spawned. Sending AddTileThing to clients...");

        let tile = tilemap.get_tile(tile_thing.position);

        let payload = game::AddTileThing {
            position: tile_thing.position,
            stack_index: tile.thing_index(entity).unwrap() as u8,
            thing: convert_tile_thing(world, entity).unwrap(),
        };

        // TODO: Limit spectators
        for (spectator, client) in world.query::<(Entity, &Client)>() {
            if spectator != entity {
                client.send_packet(payload.clone());
            }
        }
    }
}
