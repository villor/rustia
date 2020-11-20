use shipyard::*;

use crate::components::*;
use crate::unique::*;
use crate::hierarchy::*;
use crate::network::game_listener::ServerToWorkerMessage;
use crate::protocol::packet::{GameServerPacket, game, game::Position};

use super::map;

#[allow(clippy::too_many_arguments)]
pub fn player_connect_system(
    entities: EntitiesViewMut,
    mut creatures: ViewMut<Creature>,
    mut clients: ViewMut<Client>,
    parents: ViewMut<Parent>,
    children: ViewMut<Child>,
    tiles: ViewMut<Tile>,
    items: View<Item>,
    new_client_rx: UniqueViewMut<NewClientRx>,
    creature_id_counter: UniqueView<CreatureIdCounter>,
    tilemap: UniqueViewMut<TileMap>,
) {
    let mut hierarchy = (entities, parents, children);
    while let Ok(new_client) = new_client_rx.0.try_recv() {
        let player_id = creature_id_counter.0.inc();
        let spawn_position = Position { x: 50, y: 50, z: 7 };
        
        let tile = tilemap.0.get_tile(spawn_position); // TODO: check if dead

        let player = hierarchy.0.add_entity(
            &mut creatures,
            Creature { id: player_id },
        );
        
        hierarchy.attach(player, tile);

        let _ = new_client.sender.send(ServerToWorkerMessage::SendPacketsTogether(vec![
            make_login_success(player_id),
    
            GameServerPacket::PendingStateEntered,
            GameServerPacket::EnterWorld,
        
            GameServerPacket::FullWorld(game::FullWorldPayload {
                player_position: Position { x: spawn_position.x, y: spawn_position.y, z: spawn_position.z },
                world_chunk: map::get_world_chunk(
                    (&hierarchy.1.as_window(..), &hierarchy.2.as_window(..)),
                    &tiles.as_window(..),
                    &items,
                    &creatures.as_window(..),
                    &tilemap,
                    Position { x: 50 - 8, y: 50 - 6, z: 7 },
                    18,
                    14,
                ).unwrap(),
            }),
        
            GameServerPacket::WorldLight(game::LightInfo { light_level: 0xFF, light_color: 0xD7 }),
            GameServerPacket::CreatureLight(game::CreatureLightPayload {
                creature_id: 1,
                light: game::LightInfo { light_level: 0x00, light_color: 0x00 },
            }),
        
            GameServerPacket::PlayerDataBasic(game::PlayerDataBasicPayload {
                is_premium: false,
                premium_until: 0,
                vocation_id: 0,
                known_spells: vec![],
            }),
        ]));

        hierarchy.0.add_component(&mut clients, Client {
            addr: new_client.addr,
            sender: new_client.sender,
            receiver: new_client.receiver,
        }, player);
    }
}

fn make_login_success(player_id: u32) -> GameServerPacket {
    GameServerPacket::LoginSuccess(game::LoginSuccessPayload {
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
    })
}



