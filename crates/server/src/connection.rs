use std::net::SocketAddr;

use minecrust_protocol::{
    Deserialize, Serialize, datatype::Intent, packets::v773::HandshakingIncoming,
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
pub struct ClientInformation {
    #[allow(unused)]
    pub protocol_version: i32,
    #[allow(unused)]
    pub server_address: String,
    #[allow(unused)]
    pub server_port: u16,
    pub intent: Intent,
}

#[derive(Debug)]
pub struct Connection<C: Codec> {
    #[allow(unused)]
    pub id: usize,
    #[allow(unused)]
    pub client_address: SocketAddr,
    pub reader: BufReader<OwnedReadHalf>,
    pub writer: BufWriter<OwnedWriteHalf>,
    pub codec: C,
}

impl Connection<PlainCodec> {
    /// Returns a new [`Connection`] with [`PlainCodec`].
    pub fn new(id: usize, stream: TcpStream, client_address: SocketAddr) -> Self {
        tracing::info!(?client_address, "client connecting");
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(write_half);

        Self {
            codec: PlainCodec,
            id,
            client_address,
            reader,
            writer,
        }
    }

    /// Performs a handshake with the client.
    ///
    /// Handshake only appear before encryption or compressions, hence its only available with
    /// [`PlainCodec`].
    pub async fn handshake(&mut self) -> minecrust_protocol::Result<ClientInformation> {
        match self.read().await? {
            HandshakingIncoming::Intention(packet) => {
                tracing::debug!(?packet, "client handshake done");
                tracing::info!("client connected");

                Ok(ClientInformation {
                    protocol_version: *packet.protocol_version,
                    server_address: packet.server_address,
                    server_port: packet.server_port,
                    intent: packet.intent,
                })
            }
        }
    }
}

impl<C: Codec> Connection<C> {
    pub async fn read<D: Deserialize>(&mut self) -> minecrust_protocol::Result<D> {
        self.codec.decode(&mut self.reader).await
    }

    pub async fn write<S: Serialize>(&mut self, packet: S) -> minecrust_protocol::Result<()> {
        let bytes = self.codec.encode(packet)?;
        self.writer.write_all(&bytes).await?;
        self.writer.flush().await?;

        Ok(())
    }
}
