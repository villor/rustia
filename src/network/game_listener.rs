use std::{net::SocketAddr, time::{SystemTime, UNIX_EPOCH}};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Decoder, Framed};
use futures::prelude::*;
use futures::future;
use bytes::{BytesMut, Buf};
use anyhow::{Result, bail};
use log::{error, info};

use crate::packet_buffers::PacketTransmitter;
use super::protocol::{TibiaCodec, FrameType};
use super::protocol::packet::*;

pub enum ServerToWorkerMessage {
    PacketTransmitter(PacketTransmitter),
    SendPacket(GameServerPacket),
    SendPacketBoxed(Box<GameServerPacket>),
    FlushPackets,
    Disconnect,
}

pub enum WorkerToServerMessage {
    Disconnected,
}

pub struct NewClientInfo {
    pub addr: SocketAddr,
    pub player_name: String,

    pub sender: flume::Sender<ServerToWorkerMessage>,
    pub receiver: flume::Receiver<WorkerToServerMessage>,
}

pub async fn listen(tx: flume::Sender<NewClientInfo>) {
    let listener = TcpListener::bind("127.0.0.1:7172").await
        .expect("failed to bind game port 7172");

    info!("Game server listening on 127.0.0.1:7172");
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                let worker_tx = tx.clone();
                tokio::spawn(async move {
                    info!("Game connection from {:?}", addr);

                    socket.set_nodelay(true).expect("failed to set nodelay on tcp socket");

                    if let Err(e) = handle_connection(socket, addr, worker_tx).await {
                        error!("Something went wrong when handling the game connection");
                        error!("error: {}", e);
                    }
                }
            ); },
            Err(e) => { error!("game: couldn't get client: {:?}", e); }
        }
        tokio::task::yield_now().await;
    }
}

async fn flush(buffer: &mut BytesMut, framed: &mut Framed<TcpStream, TibiaCodec>) -> Result<()> {
    framed.send(buffer.bytes()).await?;
    framed.flush().await?;
    buffer.clear();
    Ok(())
}

async fn handle_connection(socket: TcpStream, addr: SocketAddr, newclient_tx: flume::Sender<NewClientInfo>) -> Result<()> {
    let mut framed = TibiaCodec::new().framed(socket);
    // TODO: think about how big this buffer needs to be
    let mut out_buf = BytesMut::with_capacity(TibiaCodec::MAX_BODY_LENGTH * 2);

    // Send nonce command
    framed.codec_mut().set_frame_type(FrameType::LengthPrefixed);
    let nonce = GameServerPacket::Nonce(game::Nonce {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,
        random_number: 23u8, // TODO: Actual random
    });
    nonce.write_to(&mut out_buf)?;
    flush(&mut out_buf, &mut framed).await?;
    
    framed.codec_mut().set_frame_type(FrameType::Raw);

    // Receive GameLogin command
    let login_data = if let Some(packet) = framed.next().await {
        if let ClientPacket::GameLogin(data) = ClientPacket::read_from(&mut packet?)? {
            data
        } else {
            bail!("invalid game login packet");
        }
    } else {
        bail!("unexpected eof");
    };

    // Enable XTEA
    framed.codec_mut().set_frame_type(FrameType::XTEA(login_data.xtea_key));

    // TODO: Check nonce, account stuff against db etc

    let (server_tx, rx) = flume::unbounded();
    let (_tx, server_rx) = flume::unbounded();
    let _ = newclient_tx.send(NewClientInfo {
        addr,
        player_name: login_data.character_name,
        sender: server_tx,
        receiver: server_rx,
    });

    let mut rx_stream = rx.stream();

    let packet_tx =
        if let Some(ServerToWorkerMessage::PacketTransmitter(packet_tx)) = rx_stream.next().await {
            packet_tx
        } else {
            bail!("did not receive packet transmitter from main thread");
        };

    let pong_buf = {
        let mut buf = BytesMut::with_capacity(1);
        GameServerPacket::Pong(game::Pong::default()).write_to(&mut buf)?;
        buf
    };

    loop {
        match futures::future::select(
            rx_stream.next(), 
            framed.next()
        ).await {
            future::Either::Left((msg, _)) => {
                if let Some(msg) = msg {
                    match msg {
                        ServerToWorkerMessage::SendPacket(packet) => {
                            // TODO: refactor
                            let len_before = out_buf.len();
                            packet.write_to(&mut out_buf)?;
                            if out_buf.len() > TibiaCodec::MAX_BODY_LENGTH {
                                let last_packet = out_buf.split_off(len_before);
                                flush(&mut out_buf, &mut framed).await?;
                                out_buf.extend_from_slice(last_packet.as_ref());
                            }
                        },
                        ServerToWorkerMessage::SendPacketBoxed(packet) => {
                            // TODO: refactor
                            let len_before = out_buf.len();
                            packet.write_to(&mut out_buf)?;
                            if out_buf.len() > TibiaCodec::MAX_BODY_LENGTH {
                                let last_packet = out_buf.split_off(len_before);
                                flush(&mut out_buf, &mut framed).await?;
                                out_buf.extend_from_slice(last_packet.as_ref());
                            }
                        },
                        ServerToWorkerMessage::FlushPackets => {
                            flush(&mut out_buf, &mut framed).await?;
                        }
                        ServerToWorkerMessage::Disconnect => {
                            bail!("disconnect by server");
                        },
                        _ => {},
                    }
                }
            },
            future::Either::Right((packets, _)) => {
                if let Some(packets) = packets {
                    let mut packets = packets?;

                    while !packets.is_empty() {
                        match ClientPacket::read_from(&mut packets) {
                            Ok(packet) => match packet {
                                ClientPacket::Ping(_) => {
                                    log::trace!("Ping received from {:?}, sending Pong!", addr);
                                    framed.send(pong_buf.bytes()).await?;
                                },
                                ClientPacket::Pong(_) => todo!(),
                                packet => {
                                    packet_tx.send(packet)?;
                                },
                            },
                            Err(e) => match e {
                                PacketError::UnknownPacket(id) => {
                                    error!("Received unknown packet with id {}, skipping rest of frame", id);
                                    break;
                                },
                                e => return Err(e.into())
                            }
                        }
                    }
                } else {
                    bail!("client disconnected");
                }
            },
        }

        tokio::task::yield_now().await;
    }
}
