use bevy::prelude::*;

use crate::util;

use super::ClientPlugin;

pub mod resources;
pub mod components;
pub mod world;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(world::load_map.system())
            .add_resource(resources::CreatureIdCounter(util::atomic_counter::AtomicU32Counter::new(1)))
            .add_plugin(ClientPlugin);
    }
}

