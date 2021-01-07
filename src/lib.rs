use bevy::prelude::*;

pub mod util;
pub mod plugins;
pub mod constants;
pub mod types;

pub fn main() {
    App::build()
        .add_plugins(MinimalPlugins)
        .add_plugin(plugins::CorePlugin)
        .run();
}
