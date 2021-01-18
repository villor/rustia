use std::sync::atomic::{AtomicBool, Ordering};

use super::network::game_listener::{NewClientInfo, ServerToWorkerMessage, WorkerToServerMessage};
use super::protocol::packet::GameServerPacket;

pub struct Client {
    pub player_name: String,
    pub addr: std::net::SocketAddr,
    pub sender: flume::Sender<ServerToWorkerMessage>,
    pub receiver: flume::Receiver<WorkerToServerMessage>,
    pub dirty: AtomicBool,
}

impl Client {
    pub fn new(new_client: NewClientInfo) -> Self {
        Self {
            player_name: new_client.player_name,
            addr: new_client.addr,
            sender: new_client.sender,
            receiver: new_client.receiver,
            dirty: AtomicBool::new(false),
        }
    }

    pub fn send_packet<T>(&self, payload: T)
    where T: Into<GameServerPacket> {
        let _ = self.sender.send(ServerToWorkerMessage::SendPacket(payload.into()));
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn flush_packets(&self) {
        if self.dirty.load(Ordering::Relaxed) {
            let _ = self.sender.send(ServerToWorkerMessage::FlushPackets);
            self.dirty.store(false, Ordering::Relaxed);
        }        
    }
}
