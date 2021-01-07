use std::convert::TryFrom;
use bevy::{ecs::Entity, prelude::AppBuilder};

use super::protocol::packet::PacketPayload;
use super::protocol::packet::client::{self, ClientPacket};

pub struct PacketBuffer<T>
where T: TryFrom<ClientPacket> {
    phantom: std::marker::PhantomData<T>,
    tx: flume::Sender<(Entity, ClientPacket)>,
    rx: flume::Receiver<(Entity, ClientPacket)>,
}

impl<T> Default for PacketBuffer<T>
where T: TryFrom<ClientPacket> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PacketBuffer<T>
where T: TryFrom<ClientPacket> {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            phantom: std::marker::PhantomData,
            tx, rx
        }
    }

    pub fn poll(&self) -> Option<(Entity, T)> {
        if let Ok((entity_id, client_packet)) = self.rx.try_recv() {
            if let Ok(packet) = T::try_from(client_packet) {
                return Some((entity_id, packet))
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
enum PacketTransmitterKind {
    Template,
    Client(Entity),
}

#[derive(Clone)]
pub struct PacketTransmitter {
    tx: Vec<Option<flume::Sender<(Entity, ClientPacket)>>>,
    kind: PacketTransmitterKind,
}

impl Default for PacketTransmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketTransmitter {
    fn new() -> Self {
        Self {
            tx: vec![None; ClientPacket::COUNT],
            kind: PacketTransmitterKind::Template,
        }
    }

    pub fn clone_for_client(&self, entity_id: Entity) -> Self {
        let mut pt = self.clone();
        pt.kind = PacketTransmitterKind::Client(entity_id);
        pt
    }

    pub fn send(&self, packet: ClientPacket) -> anyhow::Result<()> {
        match self.kind {
            PacketTransmitterKind::Template => panic!("attempt to send from template"),
            PacketTransmitterKind::Client(entity_id) => {
                if let Some(tx) = &self.tx[packet.index()] {
                    tx.send((entity_id, packet))?;
                    Ok(())
                } else {
                    panic!("attempt to access non-initialized packet buffer");
                }
            }
        }
    }
}

macro_rules! add_packet_buffer {
    ($packet_payload:ty, $packet_tx_template:expr, $app:expr) => {
        let packet_buffer = PacketBuffer::<$packet_payload>::new();
        $packet_tx_template.tx[<$packet_payload>::index()] = Some(packet_buffer.tx.clone());
        $app.add_resource(packet_buffer);
    }
}

pub fn init(app: &mut AppBuilder) {
    let mut packet_tx_template = PacketTransmitter::new();

    add_packet_buffer!(client::WalkNorth, packet_tx_template, app);
    add_packet_buffer!(client::WalkEast, packet_tx_template, app);
    add_packet_buffer!(client::WalkSouth, packet_tx_template, app);
    add_packet_buffer!(client::WalkWest, packet_tx_template, app);

    app.add_resource(packet_tx_template);
}
