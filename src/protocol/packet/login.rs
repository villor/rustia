use bytes::{Buf, BufMut, BytesMut};
use super::{PacketError, BytesMutExt, PacketRead, PacketWrite, PacketPayload};

use crate::gen_packet_types;

gen_packet_types!(LoginServerPacket; LoginServerPacketKind;
    ( Error,          10  ),
    ( Error2,         11  ),
    ( Motd,           20  ),
    ( SessionKey,     40  ),
    ( CharacterList,  100 )
);

#[derive(Debug, Default)]
pub struct Error(pub String);
impl PacketRead for Error {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized + Default {
        Ok(Self(data.get_string()?))
    }
}
impl PacketWrite for Error {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_string(&self.0);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Error2(pub String);
impl PacketRead for Error2 {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized + Default {
        Ok(Self(data.get_string()?))
    }
}
impl PacketWrite for Error2 {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_string(&self.0);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Motd(pub String);
impl PacketRead for Motd {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized + Default {
        Ok(Self(data.get_string()?))
    }
}
impl PacketWrite for Motd {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_string(&self.0);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SessionKey(pub String);
impl PacketRead for SessionKey {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized + Default {
        Ok(Self(data.get_string()?))
    }
}
impl PacketWrite for SessionKey {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_string(&self.0);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct World {
    pub id: u8,
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Default)]
pub struct Character {
    pub world_id: u8,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct CharacterList {
    pub worlds: Vec<World>,
    pub characters: Vec<Character>,
    pub has_premium: bool,
    pub premium_days_left: u32,
}

impl PacketRead for CharacterList {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for CharacterList {
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
