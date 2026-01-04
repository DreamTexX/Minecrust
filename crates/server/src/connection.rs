use bytes::BytesMut;
use minecrust_codec::pipeline::InboundPipeline;
use minecrust_protocol::packet::{self, RawPacket, unversioned};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader, BufWriter},
    net::{TcpStream, tcp::OwnedReadHalf},
    sync::mpsc::{Receiver, Sender, channel, error::SendError},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Error)]
pub(crate) enum ConnectionError {
    #[error(transparent)]
    Protocol(#[from] minecrust_protocol::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Channel(#[from] SendError<RawPacket>),
}

async fn read_next_buf(
    cancellation_token: &CancellationToken,
    reader: &mut BufReader<OwnedReadHalf>,
) -> Result<Option<BytesMut>, ConnectionError> {
    let mut buf = BytesMut::with_capacity(4064);
    tokio::select! {
        biased;
        _ = cancellation_token.cancelled() => Ok(None),
        read_result = reader.read_buf(&mut buf) => {
            let num_read_bytes = read_result?;
            Ok(if num_read_bytes > 0 {
                Some(buf)
            } else {
                None
            })
        },
    }
}

async fn run_inbound_task(
    cancellation_token: &CancellationToken,
    mut reader: BufReader<OwnedReadHalf>,
    inbound_sender: Sender<RawPacket>,
) -> Result<(), ConnectionError> {
    let mut pipeline = InboundPipeline::default();

    // Config Channel with tokio select to react instantly and reconfigure pipeline

    while let Some(buf) = read_next_buf(cancellation_token, &mut reader).await? {
        let packets = pipeline.process(buf)?;
        for packet in packets {
            inbound_sender.send(packet).await?;
        }
    }
    Ok(())
}

async fn run_dispatch_task(
    cancellation_token: &CancellationToken,
    mut inbound_receiver: Receiver<RawPacket>,
) -> Result<(), ConnectionError> {
    let handshake: unversioned::server::Intention = packet::try_into_packet_data(
        inbound_receiver
            .recv()
            .await
            .ok_or(minecrust_protocol::Error::OutOfOrder)?,
    )?;
    tracing::trace!(?handshake, "performing handshake");

    while let Some(raw_packet) = tokio::select! {
        biased;
        _ = cancellation_token.cancelled() => None,
        value = inbound_receiver.recv() => value
    } {
        tracing::debug!(?raw_packet, "packet received");
    }

    Ok(())
}

pub(crate) async fn handle_connection(
    cancellation_token: CancellationToken,
    stream: TcpStream,
) -> Result<(), ConnectionError> {
    let (read_half, write_half) = stream.into_split();
    let reader = BufReader::new(read_half);
    let _writer = BufWriter::new(write_half);

    let (inbound_sender, inbound_receiver) = channel(1);

    tokio::select! {
        biased;
        _ = cancellation_token.cancelled() => {
            tracing::trace!("loop exit reason is cancellation token");
            Ok(())
        },
        result = run_inbound_task(&cancellation_token, reader, inbound_sender) => {
            tracing::trace!(?result, "loop exit reason is a finished inbound task");
            result
        },
        result = run_dispatch_task(&cancellation_token, inbound_receiver) => {
            tracing::trace!(?result, "loop exit reason is a finished dispatch task");
            result
        }
    }
}

/*
use std::{
    fmt::Debug,
    marker::PhantomData,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use aes::Aes128;
use cipher::{
    BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher, StreamCipherCore,
    StreamCipherCoreWrapper, consts::U1, crypto_common::Output, generic_array::GenericArray,
};
use minecrust_protocol::{
    Serialize, VersionedDeserialize,
    datatype::Intent,
    packets::inbound::handshake::{HandshakingPacket, Intention},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadBuf},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use crate::{
    codec::{Codec, PlainCodec},
    crypto::cfb8::Cfb8Cipher,
    handler::{Handler, LoginHandler, StatusHandler},
};

trait Executor {
    type Value;

    fn execute(
        self,
    ) -> impl Future<Output = minecrust_protocol::Result<Self::Value>> + Send;
}

trait ConnectionState {}

struct HandshakeState;
impl ConnectionState for HandshakeState {}

struct LoginState;
impl ConnectionState for LoginState {}

pub struct Connection<C, R, W, S>
where
    C: Codec + Send,
    R: AsyncRead + Send + Unpin,
    W: AsyncWrite + Send + Unpin,
    S: ConnectionState,
{
    #[allow(unused)]
    pub id: usize,
    #[allow(unused)]
    pub client_address: SocketAddr,
    pub reader: BufReader<R>,
    pub writer: BufWriter<W>,
    pub codec: C,
    #[allow(unused)]
    pub server_address: String,
    #[allow(unused)]
    pub server_port: u16,
    _state: PhantomData<S>,
}

impl<C> Executor for Connection<C, OwnedReadHalf, OwnedWriteHalf, HandshakeState>
where
    C: Codec + Send,
{
    type Value = Connection<C, OwnedReadHalf, OwnedWriteHalf, LoginState>;

    async fn execute(self) -> minecrust_protocol::Result<Self::Value> {
        match self.codec.decode(&mut self.reader).await? {
            HandshakingPacket::Intention(Intention {
                protocol_version,
                server_address,
                server_port,
                intent,
            }) => Ok(
                Self { id: self.id, client_address: self.client_address, reader: self.reader, writer: self.writer, codec: self.codec, server_address, server_port, _state: () }
            ),
        }
    }
}

impl<C> Executor for Connection<C, OwnedReadHalf, OwnedWriteHalf, LoginState>
where
    C: Codec + Send,
{
    type Value = ();

    async fn execute(self) -> minecrust_protocol::Result<Self::Value> {
        Ok(())
    }
}

pub async fn handshake(
    id: usize,
    stream: TcpStream,
    client_address: SocketAddr,
) -> minecrust_protocol::Result<Connection<PlainCodec>> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(Box::new(read_half) as Box<dyn AsyncRead + Unpin + Send>);
    let writer = BufWriter::new(Box::new(write_half) as Box<dyn AsyncWrite + Unpin + Send>);

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

pub fn start_encryption<C: Codec>(
    mut connection: Connection<C>,
    shared_secret: &[u8],
) -> Connection<C> {
    let mut shared_secret_padded = [0u8; 16];
    shared_secret_padded.copy_from_slice(shared_secret);

    let reader = EncryptedReader::new(connection.reader.into_inner(), &shared_secret_padded);
    let writer = EncryptedWriter::new(connection.writer.into_inner(), &shared_secret_padded);
    connection.reader = BufReader::new(Box::new(reader) as Box<dyn AsyncRead + Unpin + Send>);
    connection.writer = BufWriter::new(Box::new(writer) as Box<dyn AsyncWrite + Unpin + Send>);

    tracing::trace!("switched to encrypted connection");
    connection
}

impl<C: Codec> Connection<C> {
    pub async fn read<D: VersionedDeserialize>(&mut self) -> minecrust_protocol::Result<D> {
        self.codec.decode(&mut self.reader).await
    }

    pub async fn write<S: Serialize + Debug>(
        &mut self,
        packet: S,
    ) -> minecrust_protocol::Result<()> {
        tracing::trace!(?packet, "writing packet");
        let bytes = self.codec.encode(packet)?;
        self.writer.write_all(&bytes).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct EncryptedReader<R: AsyncRead> {
    inner: R,
    cipher: Cfb8Cipher,
}

impl<R: AsyncRead> EncryptedReader<R> {
    pub fn new(inner: R, shared_secret: &[u8; 16]) -> Self {
        let cipher = Cfb8Cipher::new(shared_secret);

        Self { inner, cipher }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for EncryptedReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let self_mut = self.get_mut();
        let before = buf.filled().len();
        let poll = Pin::new(&mut self_mut.inner).poll_read(cx, buf);

        match poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Ready(Ok(())) => {
                let after = buf.filled().len();
                let buf_ref = &mut buf.filled_mut()[before..after];
                self_mut.cipher.decrypt(buf_ref);

                Poll::Ready(Ok(()))
            }
        }
    }
}

pub struct EncryptedWriter<W: AsyncWrite> {
    inner: W,
    cipher: Cfb8Cipher,
}

impl<W: AsyncWrite> EncryptedWriter<W> {
    pub fn new(inner: W, shared_secret: &[u8; 16]) -> Self {
        let cipher = Cfb8Cipher::new(shared_secret);
        Self { inner, cipher }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for EncryptedWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let self_mut = self.get_mut();
        let mut encrypted_buf = buf.to_vec();
        let encrypted_buf = encrypted_buf.as_mut_slice();

        self_mut.cipher.encrypt(encrypted_buf);

        Pin::new(&mut self_mut.inner).poll_write(cx, &encrypted_buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
 */
