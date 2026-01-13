use minecrust_protocol_macro::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GameProfileProperties {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GameProfile {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<GameProfileProperties>,
}
