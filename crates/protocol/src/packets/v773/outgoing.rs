use bytes::BufMut;

use crate::serialize::Serialize;

/// Status | 0x00 | status_response
#[derive(Debug)]
pub struct StatusResponse(pub String);

impl Serialize for StatusResponse {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        self.0.serialize(buf)
    }
}

/// Status | 0x01 | pong_response
#[derive(Debug)]
pub struct PongResponse {
    pub timestamp: i64,
}

impl Serialize for PongResponse {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        self.timestamp.serialize(buf)
    }
}
