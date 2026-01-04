use std::fmt::Debug;

use bytes::Bytes;
use minecrust_protocol::{Deserialize, Error, packet::RawPacket};

#[derive(Debug, Default)]
pub(crate) struct PacketDecoder {}

impl PacketDecoder {
    pub(crate) fn decode(&mut self, mut input: Bytes) -> Result<RawPacket, Error> {
        RawPacket::deserialize(&mut input)
    }
}
