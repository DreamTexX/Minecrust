use std::fmt::Debug;

use bytes::{Bytes, BytesMut};
use minecrust_macro::{Deserialize, Serialize};

use crate::{Deserialize, Error, Serialize, datatype::VarInt};

pub mod unversioned;
pub mod v773;

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPacket {
    pub id: VarInt,
    pub data: Bytes,
}

impl<P: Serialize + Deserialize> From<Packet<P>> for RawPacket {
    fn from(value: Packet<P>) -> Self {
        let mut data = BytesMut::new();
        value.data.serialize(&mut data);

        Self {
            id: value.id,
            data: data.freeze(),
        }
    }
}

pub struct Packet<P: Serialize + Deserialize> {
    pub id: VarInt,
    pub data: P,
}

impl<P: Serialize + Deserialize> TryFrom<RawPacket> for Packet<P> {
    type Error = Error;

    fn try_from(mut value: RawPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            data: P::deserialize(&mut value.data)?,
        })
    }
}

pub fn try_into_packet_data<P: Serialize + Deserialize>(raw: RawPacket) -> Result<P, Error> {
    let typed_packet: Packet<P> = raw.try_into()?;
    Ok(typed_packet.data)
}
