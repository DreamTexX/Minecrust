use minecrust_protocol_macro::{Deserialize, Serialize};

use crate::datatype::{Intent, VarInt};

/// Handshake | 0x00
#[derive(Debug, Deserialize, Serialize)]
pub struct Intention {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}
