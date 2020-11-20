use bytes::BytesMut;
use tokio_util::codec::{Encoder, Decoder};
use super::protocol::TibiaCodec;

impl Decoder for TibiaCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl Encoder<BytesMut> for TibiaCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        self.encode(item, dst)
    }
}
