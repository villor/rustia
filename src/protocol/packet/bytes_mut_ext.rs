use bytes::{Buf, BufMut, BytesMut};
use std::num::Wrapping;
use super::{PacketError, PacketRead, PacketWrite};
//use std::convert::TryInto;

pub trait BytesMutExt {
    fn peek_u8(&mut self) -> u8;
    //fn peek_u16_le(&mut self) -> u16;
    fn get_string(&mut self) -> Result<String, PacketError>;
    fn put_string(&mut self, s: &str);

    fn get_double(&mut self) -> f64;
    fn put_double(&mut self, value: f64, precision: u8);

    fn get_t<T: PacketRead + Default>(&mut self) -> Result<T, PacketError>;
    fn put_t<T: PacketWrite>(&mut self, writable: &T) -> Result<(), PacketError>;
}

impl BytesMutExt for BytesMut {
    fn peek_u8(&mut self) -> u8 {
        *self.bytes().first().unwrap()
    }

    /*fn peek_u16_le(&mut self) -> u16 {
        u16::from_le_bytes(self[..2].try_into().unwrap())
    }*/

    fn get_string(&mut self) -> Result<String, PacketError> {
        let len = self.get_u16_le() as usize;
        let result = match String::from_utf8(self[..len].to_vec()) {
            Ok(s) => Ok(s),
            Err(_) => Err(PacketError::InvalidString),
        };
        self.advance(len);
        result
    }

    fn put_string(&mut self, s: &str) {
        self.put_u16_le(s.len() as u16);
        self.put(s.as_bytes());
    }

    fn get_double(&mut self) -> f64 {
        let precision = self.get_u8();
        let v = Wrapping(self.get_u32_le() as i32) - Wrapping(std::i32::MAX);
        v.0 as f64 / 10f64.powi(precision as i32)
    }

    fn put_double(&mut self, value: f64, precision: u8) {
        self.put_u8(precision);
        self.put_u32_le(((value * 10f64.powi(precision as i32)) + std::i32::MAX as f64) as u32);
    }

    fn get_t<T: PacketRead + Default>(&mut self) -> Result<T, PacketError> {
        T::read_from(self)
    }

    fn put_t<T: PacketWrite>(&mut self, writable: &T) -> Result<(), PacketError> {
        writable.write_to(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_roundtrip() {
        let s = "Hello World!";
        let mut b = BytesMut::with_capacity(2 + s.len());
        b.put_string(s);
        let result = b.get_string().expect("failed to get string");
        assert_eq!(result, s);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_double_roundtrip() {
        let tests = [857.36, 261.29, -4795.01];
        for test in tests.iter() {
            let mut b = BytesMut::with_capacity(5);
            b.put_double(*test, 3);
            assert_eq!(*test, b.get_double());
        }
    }

    #[test]
    fn test_double_exact() {
        let mut b = BytesMut::with_capacity(5);
        b.put_double(-4795.01, 3);
        assert_eq!(3, b.get_u8());
        assert_eq!(2142688637, b.get_u32_le());
    }
}
