
use bytes::{Buf, BufMut, BytesMut};
use super::{PacketError, BytesMutExt, PacketRead, PacketWrite};

use crate::gen_packet_types;

gen_packet_types!(GameServerPacket;
    ( Nonce(NoncePayload),                     31),
    ( LoginSuccess(LoginSuccessPayload),       23),
    ( PendingStateEntered,                     10),
    ( EnterWorld,                              15),
    ( PlayerDataBasic(PlayerDataBasicPayload), 159),
    ( WorldLight(LightInfo),                   130),
    ( CreatureLight(CreatureLightPayload),     141),
    ( Ping,                                    29),
    ( Pong,                                    30),

    ( FullWorld(FullWorldPayload),             100)
    /*( WorldTopRow(BytesMut),             101),
    ( WorldRightRow(BytesMut),           102),
    ( WorldBottomRow(BytesMut),          103),
    ( WorldLeftRow(BytesMut),            104)*/
);

#[derive(Debug)]
pub struct NoncePayload {
    pub timestamp: u32,
    pub random_number: u8,
}

impl PacketRead for NoncePayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for NoncePayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u32_le(self.timestamp);
        out.put_u8(self.random_number);
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoginSuccessPayload {
    pub player_id: u32,
    pub beat_duration: u16,
    pub speed_a: f64,
    pub speed_b: f64,
    pub speed_c: f64,
    pub is_tutor: bool,
    pub pvp_framing: bool,
    pub expert_mode: bool,
    pub store_img_url: String,
    pub coin_package_size: u16,
}

impl PacketRead for LoginSuccessPayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for LoginSuccessPayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u32_le(self.player_id);
        out.put_u16_le(self.beat_duration);
        out.put_double(self.speed_a, 3);
        out.put_double(self.speed_b, 3);
        out.put_double(self.speed_c, 3);
        out.put_u8(if self.is_tutor { 1 } else { 0 });
        out.put_u8(if self.pvp_framing { 1 } else { 0 });
        out.put_u8(if self.expert_mode { 1 } else { 0 });
        out.put_string(&self.store_img_url);
        out.put_u16_le(self.coin_package_size);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlayerDataBasicPayload {
    pub is_premium: bool,
    pub premium_until: u32,
    pub vocation_id: u8,
    pub known_spells: Vec<u8>,
}

impl PacketRead for PlayerDataBasicPayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for PlayerDataBasicPayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u8(if self.is_premium { 1 } else { 0 });
        out.put_u32_le(self.premium_until);
        out.put_u8(self.vocation_id);
        out.put_u16(self.known_spells.len() as u16);
        for spell_id in self.known_spells.iter() {
            out.put_u8(*spell_id);
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl PacketRead for Position {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Position {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u16_le(self.x);
        out.put_u16_le(self.y);
        out.put_u8(self.z);
        Ok(())
    }
}

#[derive(Debug)]
pub struct LightInfo {
    pub light_level: u8,
    pub light_color: u8,
}

impl PacketRead for LightInfo {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for LightInfo {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u8(self.light_level);
        out.put_u8(self.light_color);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Item {
    pub client_id: u16,
    pub stack_size: Option<u8>,
    pub fluid: Option<u8>,
    pub animation: Option<u8>,
}

impl PacketRead for Item {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Item {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u16_le(self.client_id);
        out.put_u8(0xFF); // MARK_UNMARKED  TODO: (from TFS, dont know what it means)
        
        if let Some(stack_size) = self.stack_size {
            out.put_u8(stack_size);
        }
        else if let Some(fluid) = self.fluid {
            out.put_u8(fluid);
        }

        if let Some(animation) = self.animation {
            out.put_u8(animation);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Outfit {
    LookType {
        look_type: u16,
        head: u8,
        body: u8,
        legs: u8,
        feet: u8,
        addons: u8,
        mount: u16,
    },
    Item {
        client_id: u16,
        mount: u16,
    }
}

impl PacketRead for Outfit {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Outfit {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        match *self {
            Outfit::LookType { look_type, head, body, legs, feet, addons, mount } => {
                out.put_u16_le(look_type);
                out.put_u8(head);
                out.put_u8(body);
                out.put_u8(legs);
                out.put_u8(feet);
                out.put_u8(addons);
                out.put_u16_le(mount);
            },
            Outfit::Item { client_id, mount } => {
                out.put_u16_le(0);
                out.put_u16_le(client_id);
                out.put_u16_le(mount);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum CreatureKnown {
    Yes,
    No {
        remove: u32,
        creature_type: u8,
        creature_name: String,
        guild_emblem: u8,
    },
}

#[derive(Debug)]
pub struct Creature {
    pub id: u32,
    pub known: CreatureKnown,
    pub health: u8,
    pub direction: u8,
    pub outfit: Outfit,
    pub light: LightInfo,
    pub speed: u16,
    pub skull: u8,
    pub shield: u8,
    pub summon_type: u8,
    pub speech_bubble: u8,
    pub helpers: u16,
    pub walk_through: bool,
}

impl PacketRead for Creature {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Creature {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        match self.known {
            CreatureKnown::Yes => {
                out.put_u16_le(0x62);
                out.put_u32_le(self.id);
            },
            CreatureKnown::No { remove, creature_type, ref creature_name, .. } => {
                out.put_u16_le(0x61);
                out.put_u32_le(remove);
                out.put_u32_le(self.id);
                out.put_u8(creature_type);
                out.put_string(creature_name);
            }
        }

        out.put_u8(self.health);
        out.put_u8(self.direction);
        out.put_t(&self.outfit)?;
        out.put_t(&self.light)?;
        out.put_u16_le(self.speed);
        out.put_u8(self.skull);
        out.put_u8(self.shield);
        
        if let CreatureKnown::No { guild_emblem, .. } = self.known {
            out.put_u8(guild_emblem);
        }

        out.put_u8(self.summon_type);
        out.put_u8(self.speech_bubble);
        out.put_u8(0xFF); // MARK_UNMARKED

        out.put_u16_le(self.helpers);
        out.put_u8(if self.walk_through { 1 } else { 0 });

        Ok(())
    }
}

#[derive(Debug)]
pub enum Thing {
    Item(Item),
    Creature(Creature),
}

impl PacketRead for Thing {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Thing {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        match self {
            Thing::Item(item) => out.put_t(item)?,
            Thing::Creature(creature) => out.put_t(creature)?,
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct Tile {
    pub environmental_effects: u16,
    pub things: [Option<Thing>; 10],
}

impl PacketRead for Tile {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Tile {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u16_le(self.environmental_effects);

        for thing in self.things.iter() {
            if let Some(thing) = thing {
                out.put_t(thing)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum WorldData {
    Tile(Tile),
    Empty(usize),
}

impl PacketRead for Vec<WorldData> {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for Vec<WorldData> {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        let mut empty_extra = 0;
        
        for (i, entry) in self.iter().enumerate() {
            let next_is_empty =
                matches!(self.get(i + 1), Some(WorldData::Empty(_)));
            
            match entry {
                WorldData::Tile(tile) => {
                    out.put_t(tile)?;
                    if !next_is_empty {
                        // Tiles has to be followed by a skip, even if its 0
                        out.put_u8(0x00);
                        out.put_u8(0xFF);
                    }
                },
                WorldData::Empty(n) => {
                    if next_is_empty {
                        empty_extra += n;
                        continue;
                    }

                    let mut n = *n + empty_extra;
                    empty_extra = 0;

                    while n > 0xFF {
                        out.put_u8(0xFF);
                        out.put_u8(0xFF);
                        n -= 0xFF;
                    }

                    if n > 0 {
                        out.put_u8(n as u8);
                        out.put_u8(0xFF);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FullWorldPayload {
    pub player_position: Position,
    pub world_chunk: Vec<WorldData>,
}

impl PacketRead for FullWorldPayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for FullWorldPayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_t(&self.player_position)?;
        out.put_t(&self.world_chunk)?; // TODO: Check sum of data? Should always be 2016
        Ok(())
    }
}

#[derive(Debug)]
pub struct CreatureLightPayload {
    pub creature_id: u32,
    pub light: LightInfo,
}

impl PacketRead for CreatureLightPayload {
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        todo!()
    }
}

impl PacketWrite for CreatureLightPayload {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_u32(self.creature_id);
        out.put_t(&self.light)?;
        Ok(())
    }
}
