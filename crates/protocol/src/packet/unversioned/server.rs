use minecrust_protocol_macro::{Deserialize, Serialize};

use crate::datatype::{Intent, var_int};

/// Handshake | 0x00
#[derive(Debug, Deserialize, Serialize)]
pub struct Intention {
    #[protocol(with = var_int)]
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}
