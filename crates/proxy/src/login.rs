use bytes::{Buf, BytesMut};

use protocol::{FrameType, packet::ClientPacket, packet::LoginServerPacket};

use crate::{Origin, ProxyConnection, ProxyEventHandler};

/// Event handler that will detect a login protocol handshake and enable XTEA with the correct key
#[derive(Default)]
pub struct LoginHandshaker;

impl LoginHandshaker {
    pub fn new() -> Self { Self::default() }
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}

impl ProxyEventHandler for LoginHandshaker {
    fn on_frame(&self, connection: &mut ProxyConnection, from: Origin, frame: BytesMut) -> anyhow::Result<BytesMut> {
        if connection.first_frame() {
            match from {
                Origin::Client => {
                    let mut frame = frame.clone(); // clone really needed? BytesMut is very non-intuitive, Frame-abstraction?
                    match ClientPacket::read_from(&mut frame)? {
                        ClientPacket::AccountLogin(login_packet) => {
                            connection.set_frame_type(FrameType::XTEA(login_packet.xtea_key));
                        },
                        packet => {
                            anyhow::bail!(format!("Wrong first packet, expected AccountLogin, got {:?}", packet));
                        },
                    };
                },
                Origin::Server => {
                    anyhow::bail!("The server sent first, which is wrong on a login connection.");
                }
            }
        }
        Ok(frame)
    }
}

/// Injects the provided server ip/port into all worlds on any CharacterList responses
pub struct GameServerInjector {
    server_ip: String,
    server_port: u16,
}

impl GameServerInjector {
    pub fn new(server_ip: String, server_port: u16) -> Self {
        GameServerInjector {
            server_ip, server_port,
        }
    }

    pub fn new_boxed(server_ip: String, server_port: u16) -> Box<Self> {
        Box::new(GameServerInjector::new(server_ip, server_port))
    }
}

impl ProxyEventHandler for GameServerInjector {
    fn on_frame(&self, connection: &mut ProxyConnection, from: Origin, mut frame: BytesMut) -> anyhow::Result<BytesMut> {
        if connection.current_frame_id() == 1 {
            match from {
                Origin::Client => {
                    anyhow::bail!("The client sent more than one frame, which is wrong on a login connection.");
                },
                Origin::Server => {
                    let mut new_frame = BytesMut::new();
                    while frame.remaining() > 0 {
                        let mut packet = LoginServerPacket::read_from(&mut frame)?;
                        if let LoginServerPacket::CharacterList(ref mut charlist) = packet {
                            for world in charlist.worlds.iter_mut() {
                                world.ip = self.server_ip.clone();
                                world.port = self.server_port;
                            }
                        }
                        packet.write_to(&mut new_frame)?;
                    }
                    return Ok(new_frame)
                }
            }
        }
        Ok(frame)
    }
}

