mod game_profile;
mod intent;
mod text_component;
pub mod var_int;
mod var_long;

pub use game_profile::*;
pub use intent::*;
pub use text_component::*;
pub use var_long::*;

pub(crate) const SEGMENT_BITS: u8 = 0x7F;
pub(crate) const CONTINUE_BIT: u8 = 0x80;
