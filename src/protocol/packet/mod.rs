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
    fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
        where Self: std::marker::Sized;
}

/// Ability to write an instance of Self to a BytesMut
pub trait PacketWrite {
    /// Writes the packet data to a BytesMut
    fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError>;
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

/// Generates an enum with packet types
/// 
/// The enum implements PacketRead and PacketWrite dispatched by packet id
#[macro_export]
macro_rules! gen_packet_types {
    ($name:ident; $(($var:ident$(($ty:ty))?, $id:literal)),+) => {
        #[derive(Debug)]
        pub enum $name {
            $(
                $var$(($ty))?,
            )+
        }
        
        impl PacketRead for $name {
            fn read_from(data: &mut BytesMut) -> Result<Self, PacketError>
            where Self: std::marker::Sized {
                let id = data.peek_u8();
                match id {
                    $($id => { data.advance(1); Ok($name::$var$((<$ty>::read_from(data)?))?) }),+
                    _ => Err(PacketError::UnknownPacket(id).into()),
                }
            }
        }

        impl PacketWrite for $name {
            fn write_to(&self, out: &mut BytesMut) -> Result<(), PacketError> {
                match self {
                    $(gen_packet_types!(@write_variant ($name, $var$(, $ty, x)?)) => { out.put_u8($id); $(<$ty>::write_to(&x, out)?;)? Ok(()) },)+
                }
            }
        }
    };

    (@write_variant ($name:ident, $var:ident)) => {
        $name::$var
    };

    (@write_variant ($name:ident, $var:ident, $ty:ty, $x:ident)) => {
        $name::$var($x)
    };
}
