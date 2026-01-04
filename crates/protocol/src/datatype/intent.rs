use bytes::Buf;

use crate::{Deserialize, Error, Serialize, datatype::VarInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl From<VarInt> for Intent {
    fn from(value: VarInt) -> Self {
        match *value {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            _ => Self::Status,
        }
    }
}

impl Into<i32> for &Intent {
    fn into(self) -> i32 {
        match self {
            Intent::Status => 1,
            Intent::Login => 2,
            Intent::Transfer => 3,
        }
    }
}

impl Deserialize for Intent {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        VarInt::deserialize(buf).map(|i| i.into())
    }
}

impl Serialize for Intent {
    fn serialize<B: bytes::BufMut>(&self, buf: &mut B) {
        let value: i32 = self.into();
        VarInt::from(value).serialize(buf);
    }
}
