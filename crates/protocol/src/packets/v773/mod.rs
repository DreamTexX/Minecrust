use crate::{Deserialize, Error, Result, datatype::VarInt, serialize::Serialize};

pub mod incoming;
pub mod outgoing;

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

pub enum StatusOutgoing {
    StatusResponse(outgoing::StatusResponse),
    PongResponse(outgoing::PongResponse),
}

impl Serialize for StatusOutgoing {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        let mut packet = Vec::new();
        match self {
            StatusOutgoing::StatusResponse(status_response) => {
                VarInt::from(0x00).serialize(&mut packet)?;
                status_response.serialize(&mut packet)?;
            }
            StatusOutgoing::PongResponse(pong_response) => {
                VarInt::from(0x01).serialize(&mut packet)?;
                pong_response.serialize(&mut packet)?;
            }
        }

        VarInt::from(packet.len() as i32).serialize(writer)?;
        writer.write_all(&packet)?;

        Ok(())
    }
}
