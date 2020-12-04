use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder};
use futures::prelude::*;
use bytes::{Buf, BytesMut};
use anyhow::{Result, bail};
use log::{error, info};

use super::protocol::{TibiaCodec, FrameType};
use super::protocol::packet::*;

pub async fn listen() {
    let listener = TcpListener::bind("127.0.0.1:7171").await
        .expect("failed to bind login port 7171");

    info!("Login server listening on 127.0.0.1:7171");
    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => { tokio::spawn(async move {
                socket.set_nodelay(true).expect("failed to set nodelay on tcp socket");
    
                if let Err(e) = handle_login(socket).await {
                    error!("Something went wrong when handling the login");
                    error!("error: {}", e);
                }
            }); },
            Err(e) => { error!("login: couldn't get client: {:?}", e); }
        }
        tokio::task::yield_now().await;
    }
}

async fn handle_login(socket: TcpStream) -> Result<()> {
    let peer_addr = socket.peer_addr().unwrap();
    info!("Login connection from {:?}", peer_addr);
    let mut framed = TibiaCodec::new().framed(socket);

    let data = if let Some(packet) = framed.next().await {
        if let ClientPacket::AccountLogin(data) = ClientPacket::read_from(&mut packet?)? {
            data
        } else {
            bail!("invalid login packet");
        }
    } else {
        bail!("unexpected eof");
    };

    framed.codec_mut().set_frame_type(FrameType::XTEA(data.xtea_key));
    let mut output = BytesMut::with_capacity(24590);

    LoginServerPacket::Motd(login::Motd(String::from("0\nHello world!"))).write_to(&mut output)?;

    let ticks= SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / 30;
    let session_key = format!("{}\n{}\n{}\n{}", data.account_name, data.password, data.auth_token, ticks);
    LoginServerPacket::SessionKey(login::SessionKey(session_key)).write_to(&mut output)?;

    LoginServerPacket::CharacterList(login::CharacterList {
        worlds: vec![login::World {
            id: 0,
            name: String::from("World"),
            ip: String::from("127.0.0.1"),
            port: 7172,
        }],
        characters: vec![login::Character {
            world_id: 0,
            name: String::from("Hello"),
        }],
        has_premium: true,
        premium_days_left: 0,
    }).write_to(&mut output)?;
    
    framed.send(output.bytes()).await?;

    info!("Character list sent to {:?}", peer_addr);

    Ok(())
}
