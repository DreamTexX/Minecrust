use bytes::BufMut;

use crate::{Deserialize, Error, Result, datatype::VarInt, serialize::Serialize};

pub mod incoming;
pub mod outgoing;

#[derive(Debug)]
pub enum HandshakingIncoming {
    Intention(incoming::Intention),
}

impl From<incoming::Intention> for HandshakingIncoming {
    fn from(value: incoming::Intention) -> Self {
        Self::Intention(value)
    }
}

impl Deserialize for HandshakingIncoming {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> Result<Self> {
        let packet_id = VarInt::deserialize(reader)?;
        Ok(match *packet_id {
            0x00 => incoming::Intention::deserialize(reader)?.into(),
            id => return Err(Error::UnknownPacket(id)),
        })
    }
}

#[derive(Debug)]
pub enum StatusIncoming {
    StatusRequest(incoming::StatusRequest),
    PingRequest(incoming::PingRequest),
}

impl From<incoming::StatusRequest> for StatusIncoming {
    fn from(value: incoming::StatusRequest) -> Self {
        StatusIncoming::StatusRequest(value)
    }
}

impl From<incoming::PingRequest> for StatusIncoming {
    fn from(value: incoming::PingRequest) -> Self {
        StatusIncoming::PingRequest(value)
    }
}

impl Deserialize for StatusIncoming {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> Result<Self> {
        let packet_id = VarInt::deserialize(reader)?;
        Ok(match *packet_id {
            0x00 => incoming::StatusRequest::deserialize(reader)?.into(),
            0x01 => incoming::PingRequest::deserialize(reader)?.into(),
            id => return Err(Error::UnknownPacket(id)),
        })
    }
}

#[derive(Debug)]
pub enum StatusOutgoing {
    StatusResponse(outgoing::StatusResponse),
    PongResponse(outgoing::PongResponse),
}

impl Serialize for StatusOutgoing {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        tracing::trace!(data=?self, "serializing status outgoing");

        match self {
            StatusOutgoing::StatusResponse(status_response) => {
                VarInt::from(0x00).serialize(buf);
                status_response.serialize(buf);
            }
            StatusOutgoing::PongResponse(pong_response) => {
                VarInt::from(0x01).serialize(buf);
                pong_response.serialize(buf);
            }
        }
    }
}
