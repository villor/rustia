use bytes::{BytesMut};
use tokio_util::codec::{Encoder, Decoder};
use super::protocol::TibiaCodec;

impl Decoder for TibiaCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl Encoder<&[u8]> for TibiaCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        self.encode(item, dst)
    }
}
