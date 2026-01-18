use bytes::Bytes;
use minecrust_protocol_macro::{Deserialize, Serialize};
use uuid::Uuid;

use crate::datatype::var_int;

#[derive(Debug)]
pub enum LoginPacket {
    Hello(Hello),
    Key(Key),
    CustomQueryAnswer(CustomQueryAnswer),
    LoginAcknowledged(LoginAcknowledged),
    CookieRequest(CookieResponse),
}

/// Login | 0x00
#[derive(Debug, Deserialize, Serialize)]
pub struct Hello {
    pub name: String,
    pub player_uuid: Uuid,
}

/// Login | 0x01
#[derive(Debug, Deserialize, Serialize)]
pub struct Key {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

/// Login | 0x02
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomQueryAnswer {
    #[protocol(with = var_int)]
    pub message_id: i32,
    pub data: Option<Bytes>,
}

/// Login | 0x03
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginAcknowledged;

/// Login | 0x04
#[derive(Debug, Deserialize, Serialize)]
pub struct CookieResponse {
    pub key: String,
    pub data: Option<Vec<u8>>,
}
