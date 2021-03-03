use bytes::BytesMut;

use crate::{DisconnectReason, Origin, ProxyConnection, ProxyEventHandler};

/// Simple proxy event handler that prints all events to the console
pub struct DebugEventHandler {
    label: String,
}

impl Default for DebugEventHandler {
    fn default() -> Self {
        Self { label: "Debug".to_string() }
    }
}

impl DebugEventHandler {
    pub fn new(label: String) -> Self {
        Self { label }
    }

    pub fn new_boxed(label: String) -> Box<Self> {
        Box::new(Self::new(label))
    }
}

impl ProxyEventHandler for DebugEventHandler {
    fn on_new_connection(&self, connection: &mut ProxyConnection) -> anyhow::Result<()> {
        println!("{}:{} New connection from {}", self.label, connection.id(), connection.client_addr());
        Ok(())
    }

    fn on_ready(&self, connection: &mut ProxyConnection) -> anyhow::Result<()> {
        println!("{}:{} Connected to server {}", self.label, connection.id(), connection.server_addr());
        Ok(())
    }

    fn on_disconnect(&self, connection: &mut ProxyConnection, reason: &DisconnectReason) {
        println!("{}:{} Disconnected, reason:", self.label, connection.id());
        match reason {
            DisconnectReason::DisconnectedBy(origin) => println!("  Disconnect by {:?}", origin),
            DisconnectReason::Error(err) => println!("  Error {:?}", err),
        };
    }

    fn on_frame(&self, connection: &mut ProxyConnection, from: Origin, frame: BytesMut) -> anyhow::Result<BytesMut> {
        println!("{}:{} {:?} says:", self.label, connection.id(), from);
        println!("{:?}", frame.as_ref());
        Ok(frame)
    }
}

