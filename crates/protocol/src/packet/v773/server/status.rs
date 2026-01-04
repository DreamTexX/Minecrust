use minecrust_macro::Deserialize;

/// Status | 0x00
#[derive(Debug, Deserialize)]
pub struct StatusRequest;

/// Status | 0x01
#[derive(Debug, Deserialize)]
pub struct PingRequest(pub i64);
