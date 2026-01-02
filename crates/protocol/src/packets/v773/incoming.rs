use std::io::Read;

use minecrust_macro::Deserialize;

use crate::{
    Result,
    datatype::{Intent, VarInt},
};

/// Handshaking | 0x00 | intention
#[derive(Debug, Deserialize)]
pub struct Intention {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}

/// Status | 0x00 | status_request
#[derive(Debug, Deserialize)]
pub struct StatusRequest;

/// Status | 0x01 | ping_request
#[derive(Debug, Deserialize)]
pub struct PingRequest(pub i64);
