use bytes::BufMut;

use crate::{datatype::VarInt, serialize::Serialize};

pub mod incoming;
pub mod outgoing;

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
