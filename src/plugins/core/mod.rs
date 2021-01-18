use bevy::prelude::*;

use crate::{types::Position, util};

use super::ClientPlugin;
use super::client::components::Client;

pub mod resources;
pub mod components;
pub mod world;

use components::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(world::load_map.system())
            .add_resource(resources::CreatureIdCounter(util::atomic_counter::AtomicU32Counter::new(1)))
            .add_system(player_spawn.system())
            .add_plugin(ClientPlugin);
    }
}

fn player_spawn(
    added_clients: Query<(Entity, &Client), Without<Player>>,
    id_counter: Res<resources::CreatureIdCounter>,
    mut tilemap: ResMut<world::tilemap::TileMap>,
    commands: &mut Commands,
) {
    for (entity, client) in added_clients.iter() {
        log::debug!("Client added. Spawning player...");

        let spawn_position = Position { x: 50, y: 50, z: 7 };

        let tile = tilemap.get_tile_mut(spawn_position);
        tile.push(entity);

        commands.insert(entity, (
            Player,
            Creature { id: id_counter.0.inc(), name: client.player_name.clone() },
            TileThing { position: spawn_position },
        ));
    }
}

