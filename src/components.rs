use crate::network::game_listener::{ServerToWorkerMessage, WorkerToServerMessage};
use crate::protocol::packet::GameServerPacket;
use crate::protocol::packet::game::Position;
use crate::constants::Direction;

pub struct Tile {
    pub position: Position,
}

pub struct Creature {
    pub id: u32,
}

pub struct Item {
    pub client_id: u16,
}

pub struct Move {
    pub next_direction: Direction,
    pub speed: u16,
}

pub struct Client {
    pub addr: std::net::SocketAddr,
    pub sender: flume::Sender<ServerToWorkerMessage>,
    pub receiver: flume::Receiver<WorkerToServerMessage>,
    pub dirty: bool,
}

impl Client {
    pub fn send_packet<T>(&mut self, payload: T)
    where T: Into<GameServerPacket> {
        let _ = self.sender.send(ServerToWorkerMessage::SendPacket(payload.into()));
        self.dirty = true;
    }

    /*pub fn send_packet_boxed<T>(&self, payload: Box<T>)
    where T: Into<GameServerPacket> {
        let _ = self.sender.send(ServerToWorkerMessage::SendPacketBoxed());
    }*/

    pub fn flush_packets(&mut self) {
        if self.dirty {
            let _ = self.sender.send(ServerToWorkerMessage::FlushPackets);
            self.dirty = false;
        }        
    }
}
