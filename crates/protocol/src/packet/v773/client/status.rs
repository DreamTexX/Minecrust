use minecrust_macro::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusResponse(pub String); // TODO: Json Status Response

#[derive(Debug, Serialize)]
pub struct PongResponse(pub i64);
