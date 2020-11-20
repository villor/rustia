use bytes::{Buf, BufMut, BytesMut};
use std::num::Wrapping;
use super::{PacketError, BytesMutExt, util, PacketRead, PacketWrite};

use crate::gen_packet_types;

gen_packet_types!(ClientPacket;
    ( AccountLogin(AccountLoginPayload), 0x01 ),
    ( GameLogin(GameLoginPayload),       0x0A ),
    ( Ping,                              0x1D ),
    ( Pong,                              0x1E )
);

#[derive(Debug)]
pub struct AccountLoginPayload {
    pub client_os: u16,
    pub client_version: u16,
    pub protocol_version: u32,
    pub content_revision: u32,
    pub spr_signature: u32,
    pub pic_signature: u32,
    pub game_preview_state: u8,
    pub xtea_key: [Wrapping<u32>; 4],
    pub account_name: String,
    pub password: String,
    pub auth_token: String,
}

impl PacketRead for AccountLoginPayload {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        let client_os = data.get_u16_le();
        let client_version = data.get_u16_le();
        let protocol_version = data.get_u32_le();
        let content_revision = data.get_u32_le();
        let spr_signature = data.get_u32_le();
        let pic_signature = data.get_u32_le();
        let game_preview_state = data.get_u8();

        util::rsa::rsa_decrypt(&mut data[..128]);
        if data.get_u8() != 0 {
            return Err(PacketError::RsaCheckFailed);
        }

        let xtea_key =  [
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
        ];

        let account_name = data.get_string()?;
        let password = data.get_string()?;

        data.advance(data.remaining() - 128);
        util::rsa::rsa_decrypt(&mut data[..128]);
        if data.get_u8() != 0 {
            return Err(PacketError::RsaCheckFailed);
        }

        let auth_token = data.get_string()?;

        Ok(AccountLoginPayload {
            client_os,
            client_version,
            protocol_version,
            content_revision,
            spr_signature,
            pic_signature,
            game_preview_state,
            xtea_key,
            account_name,
            password,
            auth_token,
        })
    }
}

impl PacketWrite for AccountLoginPayload {
    fn write_to(&self, _out: &mut BytesMut) -> Result<(), PacketError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct GameLoginPayload {
    pub client_os: u16,
    pub client_version: u16,
    pub protocol_version: u32,
    pub client_type: u8,
    pub dat_revision: u16,
    pub xtea_key: [Wrapping<u32>; 4],
    pub gm_flag: u8,
    pub session_key: String,
    pub character_name: String,
    pub challenge_timestamp: u32,
    pub challenge_rand_num: u8,
}

impl PacketRead for GameLoginPayload {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        let client_os = data.get_u16_le();
        let client_version = data.get_u16_le();
        let protocol_version = data.get_u32_le();
        let client_type = data.get_u8();
        let dat_revision = data.get_u16_le();

        util::rsa::rsa_decrypt(&mut data[..128]);
        if data.get_u8() != 0 {
            return Err(PacketError::RsaCheckFailed);
        }

        let xtea_key =  [
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
            Wrapping(data.get_u32_le()),
        ];

        let gm_flag = data.get_u8();
        let session_key = data.get_string()?;
        let character_name = data.get_string()?;
        let challenge_timestamp = data.get_u32_le();
        let challenge_rand_num = data.get_u8();

        Ok(GameLoginPayload {
            client_os,
            client_version,
            protocol_version,
            client_type,
            dat_revision,
            xtea_key,
            gm_flag,
            session_key,
            character_name,
            challenge_timestamp,
            challenge_rand_num,
        })
    }
}

impl PacketWrite for GameLoginPayload {
    fn write_to(&self, _out: &mut BytesMut) -> Result<(), PacketError> {
        todo!()
    }
}
