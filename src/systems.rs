use shipyard::*;

use crate::{packet_buffers::PacketBuffer, protocol::packet::client};

mod player_session;
mod map;

fn move_system_stub(
    packet_north: UniqueViewMut<PacketBuffer<client::WalkNorth>>,
    packet_east: UniqueViewMut<PacketBuffer<client::WalkEast>>,
    packet_south: UniqueViewMut<PacketBuffer<client::WalkSouth>>,
    packet_west: UniqueViewMut<PacketBuffer<client::WalkWest>>,
) {
    if packet_north.poll().is_some() {
        log::info!("NORTH");
    }
    if packet_east.poll().is_some() {
        log::info!("EAST");
    }
    if packet_south.poll().is_some() {
        log::info!("SOUTH");
    }
    if packet_west.poll().is_some() {
        log::info!("WEST");
    }
}

pub fn init(world: &World) {
    map::init(world);

    world.add_workload("main")
        .with_system(system!(player_session::player_connect_system))
        .with_system(system!(move_system_stub))
        .build();
}
