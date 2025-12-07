use std::{io::ErrorKind, net::SocketAddr};

use bytes::{Buf, Bytes, BytesMut, buf::Reader};
use minecrust_protocol::{
    Deserialize, Serialize,
    datatype::{Intent, VarInt},
    packets::v773::HandshakingIncoming,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use crate::handler::{Handler, StatusHandler};

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
async fn parse_packet_length<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
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

async fn read_packet_from_reader(
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Reader<Bytes>, minecrust_protocol::Error> {
    let packet_length = parse_packet_length(reader).await?;
    if packet_length == 0 {
        tracing::trace!("handshake failed, no packet received");
        return Err(minecrust_protocol::Error::Io(std::io::Error::new(
            ErrorKind::UnexpectedEof,
            "packet length was zero",
        )));
    }

    let mut packet_bytes = BytesMut::zeroed(packet_length);
    reader.read_exact(&mut packet_bytes).await?;

    Ok(packet_bytes.freeze().reader())
}

#[derive(Debug)]
pub struct Connection {
    pub id: usize,
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
    pub client_address: SocketAddr,
    pub reader: BufReader<OwnedReadHalf>,
    pub writer: BufWriter<OwnedWriteHalf>,
}

impl Connection {
    pub async fn new(
        id: usize,
        stream: TcpStream,
        client_address: SocketAddr,
    ) -> minecrust_protocol::Result<Self> {
        tracing::info!(?client_address, "client connecting");
        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let writer = BufWriter::new(write_half);

        let mut packet = read_packet_from_reader(&mut reader).await?;
        match HandshakingIncoming::deserialize(&mut packet)? {
            HandshakingIncoming::Intention(packet) => {
                tracing::debug!(?packet, "client handshake done");
                tracing::info!("client connected");

                Ok(Self {
                    id,
                    protocol_version: *packet.protocol_version,
                    server_address: packet.server_address,
                    server_port: packet.server_port,
                    intent: packet.intent,
                    client_address,
                    reader,
                    writer,
                })
            }
        }
    }

    pub fn next_packet(
        &mut self,
    ) -> impl Future<Output = minecrust_protocol::Result<Reader<Bytes>>> + Send {
        read_packet_from_reader(&mut self.reader)
    }

    pub async fn send_packet(&mut self, packet: impl Serialize) -> minecrust_protocol::Result<()> {
        let mut response_package_bytes = Vec::new();
        packet.serialize(&mut response_package_bytes)?;
        self.writer.write_all(&response_package_bytes).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub fn into_handler(self) -> impl Handler {
        match self.intent {
            Intent::Status => StatusHandler::new(self),
            Intent::Login => unimplemented!(),
            Intent::Transfer => unimplemented!(),
        }
    }
}
