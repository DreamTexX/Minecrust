use bytes::Bytes;
use minecrust_protocol_macro::Serialize;

use crate::datatype::{GameProfile, TextComponent, var_int};

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
pub struct LoginCompression(#[protocol(with = var_int)] pub i32);

#[derive(Debug, Serialize)]
pub struct CustomQuery {
    #[protocol(with = var_int)]
    pub message_id: i32,
    pub channel: String,
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct CookieRequest(pub String);
