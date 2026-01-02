use bytes::BufMut;
use minecrust_macro::Serialize;

/// Status | 0x00 | status_response
#[derive(Debug, Serialize)]
pub struct StatusResponse(pub String);

/// Status | 0x01 | pong_response
#[derive(Debug, Serialize)]
pub struct PongResponse(pub i64);
