use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::{decoder::PacketDecoder, encoder::PacketEncoder, packet::RawPacket};

pub(crate) mod crypto;
pub mod decoder;
pub mod encoder;
pub mod packet;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Protocol(#[from] minecrust_protocol::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Compress(#[from] flate2::CompressError),
    #[error(transparent)]
    Decompress(#[from] flate2::DecompressError),
    #[error("packet deflation failed: {0}")]
    Deflate(&'static str),
}

#[derive(Default)]
pub struct PacketCodec {
    encoder: PacketEncoder,
    decoder: PacketDecoder,
}

impl PacketCodec {
    pub fn enable_crypto(&mut self, shared_secret: &[u8; 16]) {
        self.decoder.enable_crypto(shared_secret);
        self.encoder.enable_crypto(shared_secret);
    }

    pub fn enable_compression(&mut self, threshold: usize) {
        self.decoder.enable_compression(threshold);
        self.encoder.enable_compression(threshold);
    }

    pub fn disable_compression(&mut self) {
        self.decoder.disable_compression();
        self.encoder.disable_compression();
    }
}

impl Decoder for PacketCodec {
    type Error = Error;
    type Item = RawPacket;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decoder.decode(src)
    }
}

impl Encoder<RawPacket> for PacketCodec {
    type Error = Error;

    fn encode(&mut self, item: RawPacket, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        self.encoder.encode(item, dst)
    }
}
