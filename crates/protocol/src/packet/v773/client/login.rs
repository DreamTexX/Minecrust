use bytes::Bytes;
use minecrust_macro::Serialize;

use crate::datatype::{GameProfile, TextComponent, VarInt};

#[derive(Debug, Serialize)]
pub struct LoginDisconnect(pub TextComponent);

#[derive(Debug, Serialize)]
pub struct Hello {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: [u8; 32],
    pub should_authenticate: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginFinished(pub GameProfile);

#[derive(Debug, Serialize)]
pub struct LoginCompression(pub VarInt);

#[derive(Debug, Serialize)]
pub struct CustomQuery {
    pub message_id: VarInt,
    pub channel: String,
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct CookieRequest(pub String);
