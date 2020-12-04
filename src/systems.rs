use shipyard::*;

mod player_session;
mod map;
mod movement;

pub fn init(world: &World) {
    map::init(world);

    world.add_workload("main")
        .with_system(system!(player_session::player_connect_system))
        .with_system(system!(movement::packet_player_walk))
        .with_system(system!(movement::movement_system))
        .with_system(system!(player_session::packet_flush_system))
        .build();
}
