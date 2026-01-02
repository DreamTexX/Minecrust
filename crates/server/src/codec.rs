use std::fmt::Debug;

use bytes::{Buf, Bytes, BytesMut};
use minecrust_protocol::{Deserialize, Result, Serialize, VersionedDeserialize, datatype::VarInt};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub trait Codec: Debug {
    fn encode<S: Serialize>(&self, packet: S) -> Result<Bytes>;

    fn decode<D: VersionedDeserialize, R: AsyncBufRead + Send + Unpin>(
        &self,
        reader: &mut R,
    ) -> impl Future<Output = Result<D>> + Send;

    /// Method to get the packet length. This method efficiently bridges between the sync
    /// [`Deserialize`] API used in [`VarInt`] and the [`tokio::io::AsyncRead`] used in [`BufReader`].
    ///
    /// Minecraft packets are prefixed with a [`VarInt`] to describe the length of the coming packet.
    /// These kind of data types (as defined in [`minecrust_protocol`]) use a sync [`Deserialize`] API
    /// with a [`std::io::Read`] because normally there is no case and need for some async API. Except
    /// for parsing the packet size, where the length of the incoming byte stream is not yet known.
    ///
    /// To prevent changing the existing API or building a second method only for [`VarInt`] to read
    /// from an async byte stream with no known size we peek into the contents of the [`BufReader`] with
    /// [`BufReader::fill_buf`]. This returns an array of bytes already read, which we can pass down to
    /// the [`VarInt`] [`Deserialize`] Function. After reading the packet size we advance the
    /// [`BufReader`] with the consumed bytes and continue parsing the packet.
    async fn parse_packet_length<R: AsyncBufRead + Unpin>(
        reader: &mut R,
    ) -> minecrust_protocol::Result<usize> {
        let packet_length: VarInt = loop {
            // peek the input stream
            let mut peeked_bytes = reader.fill_buf().await?;
            if peeked_bytes.is_empty() {
                // EOF
                return Ok(0);
            }

            match VarInt::deserialize(&mut peeked_bytes) {
                Ok(value) => {
                    break value;
                }
                Err(err) => match err {
                    minecrust_protocol::Error::Io(err)
                        if std::io::ErrorKind::UnexpectedEof == err.kind() =>
                    {
                        // Not enough bytes read to build var int
                        continue;
                    }
                    _ => return Err(err),
                },
            };
        };

        // remove used bytes for packet length from reader
        reader.consume(packet_length.consumed());

        Ok(*packet_length as usize)
    }

    async fn read_packet<R: AsyncBufRead + Unpin>(
        reader: &mut R,
    ) -> minecrust_protocol::Result<Bytes> {
        let packet_length = Self::parse_packet_length(reader).await?;
        if packet_length == 0 {
            tracing::trace!("no packet received");
            return Err(minecrust_protocol::Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "packet length was zero",
            )));
        }

        let mut packet_bytes = BytesMut::zeroed(packet_length);
        reader.read_exact(&mut packet_bytes).await?;

        Ok(packet_bytes.freeze())
    }
}

#[derive(Debug)]
pub struct PlainCodec {
    pub protocol_version: i32,
}

impl Codec for PlainCodec {
    fn encode<S: Serialize>(&self, packet: S) -> Result<Bytes> {
        let mut packet_bytes = BytesMut::new();
        packet.serialize(&mut packet_bytes);

        let mut bytes = BytesMut::new();
        VarInt::from(packet_bytes.len() as i32).serialize(&mut bytes);
        bytes.extend_from_slice(&packet_bytes);

        Ok(bytes.freeze())
    }

    async fn decode<D: VersionedDeserialize, R: AsyncBufRead + Send + Unpin>(
        &self,
        reader: &mut R,
    ) -> Result<D> {
        let bytes = Self::read_packet(reader).await?;

        D::deserialize(self.protocol_version, &mut bytes.reader())
    }
}
