use bytes::Buf;

use crate::{Deserialize, Error, Serialize};

#[derive(Debug)]
pub struct TextComponent(pub String);

impl Serialize for TextComponent {
    fn serialize<B: bytes::BufMut>(&self, buf: &mut B) {
        self.0.serialize(buf);
    }
}

impl Deserialize for TextComponent {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(Self(String::deserialize(buf)?))
    }
}
