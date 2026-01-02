use minecrust_macro::{Deserialize, Registry};

use crate::datatype::{Intent, VarInt};

#[derive(Debug, Registry)]
pub enum HandshakingPacket {
    #[packet(id = 0x00, version = 0)]
    Intention(Intention),
}

/// Handshaking | 0x00 | intention
#[derive(Debug, Deserialize)]
pub struct Intention {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}
