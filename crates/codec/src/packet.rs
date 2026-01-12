use bytes::{Bytes, BytesMut};
use minecrust_protocol::{Deserialize, Error, Serialize, datatype::VarInt};

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub id: VarInt,
    pub data: Bytes,
}

impl TryFrom<BytesMut> for RawPacket {
    type Error = Error;

    fn try_from(mut data: BytesMut) -> Result<Self, Self::Error> {
        let id = VarInt::deserialize(&mut data)?;

        Ok(RawPacket {
            id,
            data: data.freeze(),
        })
    }
}

impl<S: Serialize> From<(i32, S)> for RawPacket {
    fn from((id, data): (i32, S)) -> Self {
        let mut buffer = BytesMut::new();
        data.serialize(&mut buffer);
        Self {
            id: VarInt::from(id),
            data: buffer.freeze(),
        }
    }
}

impl RawPacket {
    pub fn try_into<P: Deserialize>(mut self) -> Result<P, Error> {
        P::deserialize(&mut self.data)
    }
}
