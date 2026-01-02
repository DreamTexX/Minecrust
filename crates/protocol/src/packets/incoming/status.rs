use minecrust_macro::{Deserialize, Registry};

#[derive(Debug, Registry)]
pub enum StatusPacket {
    #[packet(id = 0x00, version = 773)]
    StatusRequest(StatusRequest),
    #[packet(id = 0x01, version = 773)]
    PingRequest(PingRequest),
}

/// Status | 0x00 | status_request
#[derive(Debug, Deserialize)]
pub struct StatusRequest;

/// Status | 0x01 | ping_request
#[derive(Debug, Deserialize)]
pub struct PingRequest(pub i64);
