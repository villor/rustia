use crate::network::game_listener::{ServerToWorkerMessage, WorkerToServerMessage};

pub struct Tile;

pub struct Creature {
    pub id: u32,
}

pub struct Item {
    pub client_id: u16,
}

pub struct Client {
    pub addr: std::net::SocketAddr,
    pub sender: flume::Sender<ServerToWorkerMessage>,
    pub receiver: flume::Receiver<WorkerToServerMessage>,
}
