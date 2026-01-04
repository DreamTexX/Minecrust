use bytes::{Bytes, BytesMut};
use minecrust_protocol::{Deserialize, Error, datatype::VarInt};

#[derive(Debug, Clone, Default)]
pub(crate) struct FrameDecoder {
    pending_packet_length: Option<usize>,
    buffer: BytesMut,
}

impl FrameDecoder {
    pub(crate) fn decode(&mut self, input: Bytes) -> Result<Vec<Bytes>, Error> {
        self.buffer.extend_from_slice(&input);

        let mut packets = vec![];
        while let Some(packet) = self.try_decode()? {
            packets.push(packet);
        }
        Ok(packets)
    }

    fn try_decode(&mut self) -> Result<Option<Bytes>, Error> {
        let packet_length = match self.pending_packet_length {
            Some(length) => length,
            None => match VarInt::deserialize(&mut self.buffer) {
                Ok(var_int) => {
                    let var_int = *var_int as usize;
                    self.pending_packet_length = Some(var_int);

                    var_int
                }
                Err(Error::TryGetError(_)) | Err(Error::UnexpectedEof) => {
                    return Ok(None);
                } // this is not an error, we just wait for more bytes
                Err(err) => return Err(err), // Anything else is a real error that should not happen...
            },
        };
        if self.buffer.len() < packet_length {
            return Ok(None); // waiting for more bytes
        }

        let data = self.buffer.split_to(packet_length).freeze();
        self.pending_packet_length = None;
        Ok(Some(data))
    }
}

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};
    use minecrust_protocol::{Serialize, datatype::VarInt};

    use super::FrameDecoder;

    #[test]
    fn test_incomplete() {
        let mut stream = BytesMut::new();
        VarInt::from(10).serialize(&mut stream);
        stream.put_slice(&[0x00, 0x01]);

        let mut decoder = FrameDecoder::default();
        let packets = decoder.decode(stream.freeze()).unwrap();
        assert_eq!(packets.len(), 0);

        assert_eq!(decoder.buffer.len(), 2);
    }

    #[test]
    fn test_single() {
        let mut stream = BytesMut::new();
        VarInt::from(3).serialize(&mut stream); // packet length
        stream.put_slice(&[0xff, 0xff, 0xff]);

        let mut decoder = FrameDecoder::default();
        let packets = decoder.decode(stream.freeze()).unwrap();
        let packet = packets.first().unwrap();
        assert_eq!(packet[..], [255u8, 255u8, 255u8]);

        assert_eq!(decoder.buffer.len(), 0);
    }

    #[test]
    fn test_multi() {
        let mut stream = BytesMut::new();
        VarInt::from(3).serialize(&mut stream); // packet length
        stream.put_slice(&[0xff, 0xff, 0xff]);
        VarInt::from(1).serialize(&mut stream); // packet length
        stream.put_slice(&[0xff, 0xff]); // one excessive byte

        let mut decoder = FrameDecoder::default();
        let packets = decoder.decode(stream.freeze()).unwrap();
        let packet = packets.get(0).unwrap();
        assert_eq!(packet[..], [255u8, 255u8, 255u8]);
        let packet = packets.get(1).unwrap();
        assert_eq!(packet[..], [255u8]);

        assert_eq!(decoder.buffer.len(), 1);
    }

    #[test]
    fn test_chunked() {
        let mut stream = BytesMut::new();
        VarInt::from(3).serialize(&mut stream); // packet length

        let mut decoder = FrameDecoder::default();

        let packets = decoder.decode(stream.freeze()).unwrap();
        assert_eq!(packets.len(), 0);

        let mut stream = BytesMut::new();
        stream.put_slice(&[0xff, 0xff, 0xff]);
        let packets = decoder.decode(stream.freeze()).unwrap();

        assert_eq!(packets.len(), 1);
        let packet = packets.first().unwrap();
        assert_eq!(packet[..], [255u8, 255u8, 255u8]);

        assert_eq!(decoder.buffer.len(), 0);
    }
}
