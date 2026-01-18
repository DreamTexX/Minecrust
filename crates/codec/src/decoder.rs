use std::io::copy;

use bytes::{Buf, BufMut, BytesMut};
use flate2::read::ZlibDecoder;
use minecrust_protocol::datatype::var_int;
use tokio_util::codec::Decoder;

use crate::{Error, crypto::Cfb8Cipher, packet::RawPacket};

#[derive(Default)]
pub struct PacketDecoder {
    cipher: Option<Cfb8Cipher>,
    cipher_cursor: usize,
    pending_frame_length: Option<usize>,
    threshold: Option<usize>,
}

impl PacketDecoder {
    fn decrypt(&mut self, src: &mut BytesMut) {
        if let Some(cipher) = &mut self.cipher {
            let buffer = &mut src[self.cipher_cursor..];
            self.cipher_cursor += buffer.len();
            for b in buffer {
                *b = cipher.decrypt_byte(*b);
            }
        }
    }

    fn get_frame_size(&mut self, src: &mut BytesMut) -> Result<Option<usize>, Error> {
        let buffer_before_size = src.remaining();
        match self.pending_frame_length {
            Some(length) => Ok(Some(length)),
            None => match var_int::deserialize(src) {
                // Deserialize only consumes on success
                Ok(var_int) => {
                    let var_int = var_int as usize;
                    self.pending_frame_length = Some(var_int);

                    self.cipher_cursor = self
                        .cipher_cursor
                        .saturating_sub(buffer_before_size - src.remaining());

                    Ok(Some(var_int))
                }
                Err(minecrust_protocol::Error::TryGetError(_))
                | Err(minecrust_protocol::Error::UnexpectedEof) => Ok(None), // this is not an error, we just wait for more bytes
                Err(err) => Err(err.into()),
            },
        }
    }

    fn inflate(&mut self, frame: &mut BytesMut) -> Result<(), Error> {
        if self.threshold.is_some() {
            let data_length = var_int::deserialize(frame)?;

            if data_length > 0 {
                tracing::trace!("received packet must be inflated");

                let deflated_buffer = frame.split();
                let mut writer = frame.writer();
                let mut decoder = ZlibDecoder::new(&*deflated_buffer);
                copy(&mut decoder, &mut writer)?;
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
        self.threshold = Some(threshold)
    }

    pub fn disable_compression(&mut self) {
        self.threshold = None;
    }
}

impl Decoder for PacketDecoder {
    type Error = Error;
    type Item = RawPacket;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decrypt(src);

        let Some(frame_size) = self.get_frame_size(src)? else {
            return Ok(None);
        };

        if src.remaining() < frame_size {
            src.reserve(frame_size - src.remaining());
            return Ok(None); // waiting for more bytes
        }

        let mut frame = src.split_to(frame_size);
        self.cipher_cursor = self.cipher_cursor.saturating_sub(frame_size);
        self.pending_frame_length = None;

        self.inflate(&mut frame)?;
        let raw_packet = RawPacket::try_from(frame)?;

        tracing::trace!(?raw_packet, "decode packet");
        Ok(Some(raw_packet))
    }
}
