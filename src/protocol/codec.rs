use bytes::{Buf, BufMut, BytesMut};
use std::io;
use adler32::adler32;

use super::util::xtea;

const HEADER_SIZE: usize = 2;
const CHECKSUM_SIZE: usize = 4;
const MAX_FRAME_SIZE: usize = 24590;
const MAX_DATA_SIZE: usize = MAX_FRAME_SIZE - HEADER_SIZE;

#[derive(Debug)]
enum DecodeState {
    Head,
    Data(usize),
}

#[derive(Debug)]
pub enum FrameType {
    Raw,
    LengthPrefixed, // Nonce
    XTEA([std::num::Wrapping<u32>; 4]),
}

#[derive(Debug)]
pub struct TibiaCodec {
    state: DecodeState,
    frame_type: FrameType,
}

impl TibiaCodec {
    pub fn new() -> Self {
        Self {
            state: DecodeState::Head,
            frame_type: FrameType::Raw,
        }
    }

    pub fn set_frame_type(&mut self, frame_type: FrameType) {
        self.frame_type = frame_type;
    }

    fn decode_head(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < HEADER_SIZE {
            return Ok(None);
        }

        let n = src.get_u16_le() as usize;

        if n > MAX_DATA_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame size over limit"
            ));
        }

        src.reserve(n);

        Ok(Some(n))
    }

    fn decode_data(&self, n: usize, src: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        if src.len() < n {
            return Ok(None);
        }

        let mut data = src.split_to(n);

        let recv_checksum = data.split_to(CHECKSUM_SIZE).get_u32_le();
        let checksum = match data.remaining() {
            0 => 0,
            _ => adler32(data.bytes())?,
        };

        if recv_checksum != checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, 
                "checksum check failed"
            ));
        }

        if let FrameType::XTEA(key) = self.frame_type {
            xtea::decrypt(&mut data[..], &key);
        }

        if let FrameType::LengthPrefixed | FrameType::XTEA(_) = self.frame_type {
            let length = data.split_to(HEADER_SIZE).get_u16_le() as usize;
            if data.len() < length {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput, 
                    "not enough data"
                ));
            }
            data.truncate(length);
        }

        Ok(Some(data))
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        let n = match self.state {
            DecodeState::Head => match self.decode_head(src)? {
                Some(n) => {
                    self.state = DecodeState::Data(n);
                    n
                }
                None => return Ok(None),
            },
            DecodeState::Data(n) => n
        };
        
        match self.decode_data(n, src)? {
            Some(data) => {
                self.state = DecodeState::Head;
                src.reserve(HEADER_SIZE);
                Ok(Some(data))
            },
            None => Ok(None),
        }
    }

    pub fn encode(&mut self, packet_data: BytesMut, dst: &mut BytesMut) -> Result<(), io::Error> {
        // Calculate data size and padding
        let (n, padding) = match self.frame_type {
            FrameType::Raw => (CHECKSUM_SIZE + packet_data.len(), 0),
            FrameType::LengthPrefixed => (CHECKSUM_SIZE + HEADER_SIZE + packet_data.len(), 0),
            FrameType::XTEA(_) => {
                let n = HEADER_SIZE + packet_data.len();
                let mut padding = 0;
                while (n + padding) % 8 != 0 { padding += 1; }
                (CHECKSUM_SIZE + n + padding, padding)
            }
        };
        
        if n > MAX_DATA_SIZE as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input size over limit",
            ));
        }

        // Reserve and split to write checksum later
        dst.reserve(HEADER_SIZE + n);
        let mut data_dst = dst.split_off(HEADER_SIZE + CHECKSUM_SIZE);

        // Write size
        dst.put_u16_le(n as u16);

        // Add LengthPrefixed(nonce)/XTEA header with packet length
        if let FrameType::LengthPrefixed | FrameType::XTEA(_) = self.frame_type {
            data_dst.put_u16_le(packet_data.len() as u16);
        }

        // Copy packet data
        data_dst.extend_from_slice(&packet_data[..]);

        // Add padding
        for _ in 0..padding {
            data_dst.put_u8(0x33);
        }

        // XTEA encrypt
        if let FrameType::XTEA(key) = self.frame_type {
            xtea::encrypt(&mut data_dst[..n - CHECKSUM_SIZE], &key);
        }

        // Write checksum
        let checksum = adler32(&data_dst[..n - CHECKSUM_SIZE])?;
        dst.put_u32_le(checksum);

        dst.unsplit(data_dst);

        Ok(())
    }
}

impl Default for TibiaCodec {
    fn default() -> Self {
        Self::new()
    }
}
