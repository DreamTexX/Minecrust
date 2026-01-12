use futures::SinkExt;
use minecrust_codec::{PacketCodec, packet::RawPacket};
use thiserror::Error;
use tokio::{net::TcpStream, task::JoinError};
use tokio_stream::StreamExt;
use tokio_util::{codec::Framed, sync::CancellationToken};

use crate::dispatcher::{self, Dispatcher};

#[derive(Debug, Error)]
pub(crate) enum ConnectionError {
    #[error(transparent)]
    Protocol(#[from] minecrust_protocol::Error),
    #[error(transparent)]
    Codec(#[from] minecrust_codec::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error("{0}")]
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProtocolState {
    Handshake,
    Status,
    Login,
    Configuration,
}

#[derive(Debug, Clone)]
pub(crate) enum Action {
    EnableEncryption([u8; 16]),
    EnableCompression(usize),
    ProtocolState(ProtocolState),
    ProtocolVersion(u32),
    SendPacket(RawPacket),
}

pub(crate) async fn handle_connection(
    shutdown_signal: CancellationToken,
    stream: TcpStream,
) -> Result<(), ConnectionError> {
    tracing::trace!("handle connection started");

    let mut stream = Framed::new(stream, PacketCodec::default());
    let mut protocol_state = ProtocolState::Handshake;
    let mut protocol_version: u32 = 0;
    let mut dispatcher: Box<dyn Dispatcher + Send> =
        Box::new(dispatcher::unversioned::HandshakeDispatcher);

    while let Some(raw_packet) = tokio::select! {
        biased;
        _ = shutdown_signal.cancelled() => return Ok(()),
        next_item = stream.next() => next_item.transpose()?
    } {
        let actions = dispatcher.dispatch(raw_packet)?;

        tracing::trace!(?actions, "running action");
        let mut dispatcher_switch_needed = false;
        for action in actions {
            match action {
                Action::EnableEncryption(shared_secret) => {
                    stream.codec_mut().enable_crypto(&shared_secret);
                }
                Action::EnableCompression(threshold) => {
                    stream.codec_mut().enable_compression(threshold);
                }
                Action::ProtocolState(new_protocol_state) => {
                    protocol_state = new_protocol_state;
                    dispatcher_switch_needed = true;
                }
                Action::ProtocolVersion(new_protocol_version) => {
                    protocol_version = new_protocol_version;
                    dispatcher_switch_needed = true;
                }
                Action::SendPacket(packet) => {
                    stream.send(packet).await?;
                }
            }
        }

        if dispatcher_switch_needed {
            match (protocol_state, protocol_version) {
                (ProtocolState::Status, 773..) => {
                    dispatcher = Box::new(dispatcher::v773::StatusDispatcher);
                }
                (ProtocolState::Login, 773..) => {
                    dispatcher = Box::new(dispatcher::v773::LoginDispatcher::new());
                }
                (_, _) => {
                    tracing::error!(?protocol_state, protocol_version, "no dispatcher found");
                    return Err(ConnectionError::Custom("no dispatcher found"));
                }
            };
        }
    }

    tracing::trace!("connection closed");
    Ok(())
}
