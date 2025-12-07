use crate::{Deserialize, datatype::VarInt};

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

impl Deserialize for Intent {
    fn deserialize<R: std::io::Read>(reader: &mut R) -> crate::Result<Self> {
        VarInt::deserialize(reader).map(|i| i.into())
    }
}
