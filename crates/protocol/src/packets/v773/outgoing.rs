use crate::serialize::Serialize;

/// Status | 0x00 | status_response
#[derive(Debug)]
pub struct StatusResponse(pub String);

impl Serialize for StatusResponse {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.0.serialize(writer)
    }
}

/// Status | 0x01 | pong_response
#[derive(Debug)]
pub struct PongResponse {
    pub timestamp: i64,
}

impl Serialize for PongResponse {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.timestamp.serialize(writer)
    }
}
