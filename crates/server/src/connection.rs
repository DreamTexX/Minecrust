use std::net::SocketAddr;

use minecrust_protocol::{
    Serialize, VersionedDeserialize,
    datatype::Intent,
    packets::incoming::handshaking::{HandshakingPacket, Intention},
};
use tokio::{
    io::{AsyncWriteExt, BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use crate::codec::{Codec, PlainCodec};

#[derive(Debug)]
pub struct Connection<C: Codec> {
    #[allow(unused)]
    pub id: usize,
    #[allow(unused)]
    pub client_address: SocketAddr,
    pub reader: BufReader<OwnedReadHalf>,
    pub writer: BufWriter<OwnedWriteHalf>,
    pub codec: C,
    #[allow(unused)]
    pub server_address: String,
    #[allow(unused)]
    pub server_port: u16,
    pub intent: Intent,
}

pub async fn handshake(
    id: usize,
    stream: TcpStream,
    client_address: SocketAddr,
) -> minecrust_protocol::Result<Connection<PlainCodec>> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let writer = BufWriter::new(write_half);

    let temporary_codec = PlainCodec {
        protocol_version: 0,
    };

    match temporary_codec.decode(&mut reader).await? {
        HandshakingPacket::Intention(Intention {
            protocol_version,
            server_address,
            server_port,
            intent,
        }) => Ok(Connection {
            id,
            client_address,
            reader,
            writer,
            codec: PlainCodec {
                protocol_version: *protocol_version,
            },
            server_address,
            server_port,
            intent,
        }),
    }
}

impl<C: Codec> Connection<C> {
    pub async fn read<D: VersionedDeserialize>(&mut self) -> minecrust_protocol::Result<D> {
        self.codec.decode(&mut self.reader).await
    }

    pub async fn write<S: Serialize>(&mut self, packet: S) -> minecrust_protocol::Result<()> {
        let bytes = self.codec.encode(packet)?;
        self.writer.write_all(&bytes).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
