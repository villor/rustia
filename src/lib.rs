
use shipyard::*;
use spin_sleep::LoopHelper;

pub mod network;
pub mod protocol; // TODO: make a crate
pub mod unique;
pub mod systems;
pub mod components;
pub mod util;
pub mod tilemap;
pub mod hierarchy;

struct GameServer {
    world: World,
}

pub async fn main() {
    let world = World::new();
    let (newclient_tx, newclient_rx) = flume::unbounded();

    world.add_unique(unique::NewClientRx(newclient_rx));
    world.add_unique(unique::CreatureIdCounter(util::atomic_counter::AtomicU32Counter::new(1)));

    systems::init(&world);

    tokio::spawn(network::login_listener::listen());
    tokio::spawn(network::game_listener::listen(newclient_tx));

    let state = GameServer {
        world,
    };
    let mut _state = run_game_thread(state).await;
}

async fn run_game_thread(state: GameServer) {
    let (_tx, rx) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name(String::from("game_thread"))
        .spawn(move || {
            let mut loop_helper = LoopHelper::builder().build_with_target_rate(20.0);
            loop {
                loop_helper.loop_start();
                state.world.run_default();
                loop_helper.loop_sleep();
            }

            // tx.send(state).ok().expect("failed to exit game thread");
        })
        .expect("failed to spawn ticker thread");
    
    rx.await.unwrap()
}
