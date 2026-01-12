use bytes::{Buf, BufMut, BytesMut};
use flate2::{Compression, read::ZlibEncoder};
use minecrust_protocol::{Serialize, datatype::VarInt};
use std::io::copy;
use tokio_util::codec::Encoder;

use crate::{Error, crypto::Cfb8Cipher, packet::RawPacket};

#[derive(Default)]
pub struct PacketEncoder {
    cipher: Option<Cfb8Cipher>,
    threshold: Option<usize>,
}

impl PacketEncoder {
    fn encrypt(&mut self, buffer: &mut [u8]) {
        if let Some(cipher) = &mut self.cipher {
            for b in buffer {
                *b = cipher.encrypt_byte(*b);
            }
        }
    }

    fn deflate(&mut self, frame: &mut BytesMut) -> Result<(), Error> {
        if let Some(threshold) = &mut self.threshold {
            let packet_size = frame.remaining();

            if packet_size >= *threshold {
                let inflated_bytes = frame.split();
                VarInt::from(packet_size as i32).serialize(frame);
                let mut writer = frame.writer();
                let mut encoder = ZlibEncoder::new(&*inflated_bytes, Compression::default());
                copy(&mut encoder, &mut writer)?;

                tracing::trace!(
                    original_size = packet_size,
                    compressed_size = frame.remaining(),
                    "compression results"
                );
            } else {
                frame.reserve(1);
                frame.copy_within(0.., 1);
                frame[0] = 0x00;
            }
        }

        Ok(())
    }

    pub fn enable_crypto(&mut self, shared_secret: &[u8; 16]) {
        if self.cipher.is_none() {
            self.cipher = Some(Cfb8Cipher::new(shared_secret));
        }
    }

    pub fn enable_compression(&mut self, threshold: usize) {
        tracing::trace!(threshold, "enabling compression");
        self.threshold = Some(threshold)
    }

    pub fn disable_compression(&mut self) {
        self.threshold = None;
    }
}

impl Encoder<RawPacket> for PacketEncoder {
    type Error = Error;

    fn encode(&mut self, raw_packet: RawPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        tracing::trace!(?raw_packet, "encoding packet");
        let mut frame = BytesMut::new();
        raw_packet.id.serialize(&mut frame);
        raw_packet.data.serialize(&mut frame);

        self.deflate(&mut frame)?;
        let finished_frame = frame.split();
        VarInt::from(finished_frame.remaining() as i32).serialize(&mut frame);
        frame.unsplit(finished_frame);

        self.encrypt(&mut frame);

        dst.unsplit(frame);
        Ok(())
    }
}
