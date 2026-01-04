use bytes::BytesMut;
use minecrust_protocol::{Error, packet::RawPacket};

use crate::{
    crypto::{Cfb8Cipher, Decryption},
    frame::FrameDecoder,
    packet::PacketDecoder,
};

#[derive(Debug, Default)]
pub struct InboundPipeline {
    decryption: Option<Decryption>,
    frame_decoder: FrameDecoder,
    packet_decoder: PacketDecoder,
}

impl InboundPipeline {
    pub fn process(&mut self, mut input: BytesMut) -> Result<Vec<RawPacket>, Error> {
        if let Some(decryption) = &mut self.decryption {
            decryption.process(&mut input);
        }

        let mut packets = vec![];
        for frame in self.frame_decoder.decode(input.freeze())? {
            packets.push(self.packet_decoder.decode(frame)?);
        }
        Ok(packets)
    }

    pub fn enable_crypto(&mut self, shared_secret: &[u8; 16]) {
        let cipher = Cfb8Cipher::new(shared_secret);
        let decryption_layer = Decryption::new(cipher);
        self.decryption = Some(decryption_layer)
    }

    pub fn enable_compression(&mut self) {
        unimplemented!()
        // self.layer.push();
    }
}
