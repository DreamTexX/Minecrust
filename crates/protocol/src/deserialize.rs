use std::io::Read;

use crate::{Result, datatype::VarInt};

pub trait Deserialize: Sized {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self>;
}

impl Deserialize for bool {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 1];
        reader.read_exact(&mut bytes)?;

        Ok(bytes[0] == 0x01)
    }
}

impl Deserialize for u8 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 1];
        reader.read_exact(&mut bytes)?;

        Ok(bytes[0])
    }
}

impl Deserialize for i8 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 1];
        reader.read_exact(&mut bytes)?;

        Ok(bytes[0] as i8)
    }
}

impl Deserialize for u16 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 2];
        reader.read_exact(&mut bytes)?;

        Ok(u16::from_be_bytes(bytes))
    }
}

impl Deserialize for i16 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 2];
        reader.read_exact(&mut bytes)?;

        Ok(i16::from_be_bytes(bytes))
    }
}

impl Deserialize for i32 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 4];
        reader.read_exact(&mut bytes)?;

        Ok(i32::from_be_bytes(bytes))
    }
}

impl Deserialize for i64 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 8];
        reader.read_exact(&mut bytes)?;

        Ok(i64::from_be_bytes(bytes))
    }
}

impl Deserialize for f32 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 4];
        reader.read_exact(&mut bytes)?;

        Ok(f32::from_be_bytes(bytes))
    }
}

impl Deserialize for f64 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0; 8];
        reader.read_exact(&mut bytes)?;

        Ok(f64::from_be_bytes(bytes))
    }
}

impl Deserialize for String {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        tracing::trace!("reading string");
        let len = *VarInt::deserialize(reader)?;
        tracing::trace!(len, "string size");

        if len > 0 {
            let mut bytes = vec![0; len as usize];
            reader.read_exact(&mut bytes)?;

            Ok(String::from_utf8(bytes)?)
        } else {
            Ok(String::new())
        }
    }
}
