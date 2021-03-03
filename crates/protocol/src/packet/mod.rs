use bytes::BytesMut;
use thiserror::Error;

use super::util;

mod bytes_mut_ext;
pub mod login;
pub mod client;
pub mod game;

pub use bytes_mut_ext::*;
pub use client::ClientPacket;
pub use login::LoginServerPacket;
pub use game::GameServerPacket;

#[derive(Clone, Copy, Error, Debug)]
pub enum PacketError {
    #[error("unknown packet id {0}")]
    UnknownPacket(u8),
    #[error("invalid string in packet")]
    InvalidString,
    #[error("RSA zero check failed")]
    RsaCheckFailed,
}

/// Ability to read an instance of Self from a BytesMut
pub trait PacketRead {
    /// Reads the packet data from a BytesMut.
    fn read_from(_data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized + Default {
        Ok(Self::default())
    }
}

/// Ability to write an instance of Self to a BytesMut
pub trait PacketWrite {
    /// Writes the packet data to a BytesMut
    fn write_to(&self, _out: &mut BytesMut) -> Result<(), PacketError> {
        Ok(())
    }
}

impl PacketRead for String {
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
    where Self: std::marker::Sized {
        data.get_string()
    }
}

impl PacketWrite for String {
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
        out.put_string(self);
        Ok(())
    }
}

pub trait PacketPayload<T> {
    fn index() -> usize;
    fn kind() -> T;
}

/// Generates an enum with packet types
/// 
/// The enum implements PacketRead and PacketWrite dispatched by packet id
#[macro_export]
macro_rules! gen_packet_types {
    ($name:ident; $name_kind:ident; $(($var:ident, $id:literal)),+) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $var($var),
            )+
        }

        #[derive(Debug)]
        pub enum $name_kind {
            $(
                $var,
            )+
            __CountKindsLast,
        }
        
        impl $name {
            #[allow(dead_code)]
            pub const COUNT: usize = $name_kind::__CountKindsLast as usize;

            pub fn read_from(data: &mut BytesMut) -> Result<Self, PacketError> {
                let id = data.peek_u8();
                match id {
                    $($id => { data.advance(1); Ok($name::$var(<$var>::read_from(data)?)) }),+
                    _ => Err(PacketError::UnknownPacket(id).into()),
                }
            }

            pub fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
                match self {
                    $($name::$var(x) => { out.put_u8($id); <$var>::write_to(&x, out)?; Ok(()) },)+
                }
            }

            pub fn index(&self) -> usize {
                match self {
                    $($name::$var(_) => $name_kind::$var as usize),+
                }
            }
        }

        use std::convert::TryFrom;
        $(
            impl TryFrom<$name> for $var {
                type Error = &'static str;

                fn try_from(packet: $name) -> Result<Self, Self::Error> {
                    if let $name::$var(packet) = packet {
                        return Ok(packet);
                    }
                    Err("variant and struct mismatch")
                }
            }

            impl Into<$name> for $var {
                fn into(self) -> $name {
                    $name::$var(self)
                }
            }

            impl PacketPayload<$name_kind> for $var {
                fn index() -> usize {
                    Self::kind() as usize
                }

                fn kind() -> $name_kind {
                    $name_kind::$var
                }
            }
        )+
    };
}
