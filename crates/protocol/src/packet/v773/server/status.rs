use minecrust_protocol_macro::Deserialize;

/// Status | 0x01
#[derive(Debug, Deserialize)]
pub struct PingRequest(pub i64);
