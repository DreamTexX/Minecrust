mod var_int;
mod var_long;
mod intent;

pub use var_int::VarInt;
pub use var_long::VarLong;
pub use intent::Intent;

pub(crate) const SEGMENT_BITS: u8 = 0x7F;
pub(crate) const CONTINUE_BIT: u8 = 0x80;
