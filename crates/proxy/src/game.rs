use bytes::BytesMut;
use protocol::{FrameType, packet::ClientPacket};

use crate::{Origin, ProxyConnection, ProxyEventHandler};

/// Event handler that will detect a game protocol handshake and enable XTEA with the correct key
#[derive(Default)]
pub struct GameHandshaker;

impl GameHandshaker {
    pub fn new() -> Self { Self::default() }
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl ProxyEventHandler for GameHandshaker {
    fn on_new_connection(&self, connection: &mut ProxyConnection) -> anyhow::Result<()> {
        // Because nonce is weird
        connection.set_frame_type(FrameType::LengthPrefixed);
        Ok(())
    }

    fn on_frame(&self, connection: &mut ProxyConnection, from: Origin, frame: BytesMut) -> anyhow::Result<BytesMut> {
        if connection.first_frame() { // Nonce from server
            connection.set_frame_type(FrameType::Raw);
        } else if connection.current_frame_id() == 1 { // First frame from client
            match from {
                Origin::Client => {
                    let mut frame = frame.clone(); // clone really needed? BytesMut is very non-intuitive, Frame-abstraction?
                    match ClientPacket::read_from(&mut frame)? {
                        ClientPacket::GameLogin(login_packet) => {
                            connection.set_frame_type(FrameType::XTEA(login_packet.xtea_key));
                        },
                        packet => {
                            anyhow::bail!(format!("Wrong first packet from client, expected GameLogin, got {:?}", packet));
                        },
                    };
                },
                Origin::Server => {
                    anyhow::bail!("Unexpected frame order, client should send the second frame.");
                }
            }
        }
        Ok(frame)
    }
}
