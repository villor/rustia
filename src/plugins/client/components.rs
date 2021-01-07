use super::network::game_listener::{ServerToWorkerMessage, WorkerToServerMessage};
use super::protocol::packet::GameServerPacket;

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
