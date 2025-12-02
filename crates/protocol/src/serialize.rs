use std::io::Write;

use crate::{Result, datatype::VarInt};

pub trait Serialize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()>;
}

impl Serialize for bool {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&[if *self { 0x01 } else { 0x00 }])?;

        Ok(())
    }
}

impl Serialize for u8 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&[*self])?;

        Ok(())
    }
}

impl Serialize for i8 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&[*self as u8])?;

        Ok(())
    }
}

impl Serialize for u16 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for i16 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for i32 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for i64 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for f32 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for f64 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.to_be_bytes())?;

        Ok(())
    }
}

impl Serialize for String {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        let bytes = self.as_bytes();
        VarInt::from(bytes.len() as i32).serialize(writer)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}
