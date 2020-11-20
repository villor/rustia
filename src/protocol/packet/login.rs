use bytes::{Buf, BufMut, BytesMut};
use super::{PacketError, BytesMutExt, PacketRead, PacketWrite};

use crate::gen_packet_types;

gen_packet_types!(LoginServerPacket;
    ( Error(String),                        0x0A ),
    ( Error2(String),                       0x0B ),
    ( Motd(String),                         0x14 ),
    ( SessionKey(String),                   0x28 ),
    ( CharacterList(CharacterListPayload),  0x64 )
);

#[derive(Debug)]
pub struct World {
    pub id: u8,
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct Character {
    pub world_id: u8,
    pub name: String,
}

#[derive(Debug)]
pub struct CharacterListPayload {
    pub worlds: Vec<World>,
    pub characters: Vec<Character>,
    pub has_premium: bool,
    pub premium_days_left: u32,
}

impl PacketRead for CharacterListPayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for CharacterListPayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u8(self.worlds.len() as u8);
        for world in self.worlds.iter() {
            out.put_u8(world.id);
            out.put_string(&world.name);
            out.put_string(&world.ip);
            out.put_u16_le(world.port);
            out.put_u8(0); // why?
        }

        out.put_u8(self.characters.len() as u8);
        for c in self.characters.iter() {
            out.put_u8(c.world_id);
            out.put_string(&c.name);
        }

        out.put_u8(0);
        out.put_u8(if self.has_premium { 1 } else { 0 });
        out.put_u32_le(self.premium_days_left);

        Ok(())
    }
}
