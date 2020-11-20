use shipyard::*;

mod player_session;
mod map;

pub fn init(world: &World) {
    map::init(world);

    world.add_workload("main")
        .with_system(system!(player_session::player_connect_system))
        .build();
}
