mod var_int;
mod var_long;

pub use var_int::VarInt;

pub(crate) const SEGMENT_BITS: u8 = 0x7F;
pub(crate) const CONTINUE_BIT: u8 = 0x80;
