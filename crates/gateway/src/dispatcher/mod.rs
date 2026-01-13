use crate::connection::{Action, ConnectionError};
use minecrust_codec::packet::RawPacket;

pub(crate) mod unversioned;
pub(crate) mod v773;

pub(crate) trait Dispatcher {
    fn dispatch(&mut self, raw_packet: RawPacket) -> Result<Vec<Action>, ConnectionError>;
}
