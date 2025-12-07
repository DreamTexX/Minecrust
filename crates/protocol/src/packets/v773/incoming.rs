use std::io::Read;

use crate::{Deserialize, Result, datatype::{Intent, VarInt}};

/// Handshaking | 0x00 | intention
#[derive(Debug)]
pub struct Intention {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}

impl Deserialize for Intention {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let protocol_version = VarInt::deserialize(reader)?;
        let server_address = String::deserialize(reader)?;
        let server_port = u16::deserialize(reader)?;
        let intent = Intent::deserialize(reader)?;

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            intent,
        })
    }
}

/// Status | 0x00 | status_request
#[derive(Debug)]
pub struct StatusRequest;

impl Deserialize for StatusRequest {
    fn deserialize<R: Read>(_reader: &mut R) -> Result<Self> {
        Ok(Self)
    }
}

/// Status | 0x01 | ping_request
#[derive(Debug)]
pub struct PingRequest {
    pub timestamp: i64,
}

impl Deserialize for PingRequest {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let timestamp = i64::deserialize(reader)?;

        Ok(Self { timestamp })
    }
}
