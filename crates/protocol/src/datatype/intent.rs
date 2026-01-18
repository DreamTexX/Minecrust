use bytes::Buf;

use crate::{Deserialize, Error, Serialize, datatype::var_int};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl From<i32> for Intent {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            _ => Self::Status,
        }
    }
}

impl From<&Intent> for i32 {
    fn from(value: &Intent) -> i32 {
        match *value {
            Intent::Status => 1,
            Intent::Login => 2,
            Intent::Transfer => 3,
        }
    }
}

impl Deserialize for Intent {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        var_int::deserialize(buf).map(|i| i.into())
    }
}

impl Serialize for Intent {
    fn serialize<B: bytes::BufMut>(&self, buf: &mut B) {
        let value: i32 = self.into();
        var_int::serialize(&value, buf);
    }
}
