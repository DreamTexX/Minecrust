use minecrust_codec::packet::RawPacket;
use minecrust_protocol::{datatype::Intent, packet::unversioned::server::Intention};

use crate::{
    connection::{Action, ConnectionError, ProtocolState},
    dispatcher::Dispatcher,
};

pub(crate) struct HandshakeDispatcher;

impl Dispatcher for HandshakeDispatcher {
    fn dispatch(&mut self, raw_packet: RawPacket) -> Result<Vec<Action>, ConnectionError> {
        let mut state_changes = vec![];

        let handshake: Intention = raw_packet.try_into()?;
        tracing::trace!(?handshake, "performing handshake");

        state_changes.push(Action::ProtocolVersion(*handshake.protocol_version as u32));
        state_changes.push(match handshake.intent {
            Intent::Login => Action::ProtocolState(ProtocolState::Login),
            Intent::Status => Action::ProtocolState(ProtocolState::Status),
            Intent::Transfer => unimplemented!("transfer intent is not implemented"),
        });

        Ok(state_changes)
    }
}
