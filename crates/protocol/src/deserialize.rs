use bytes::{Buf, BufMut, Bytes, BytesMut};
use uuid::Uuid;

use crate::{Error, datatype::VarInt};

pub trait Deserialize: Sized {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error>;
}

impl Deserialize for bool {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_u8()? == 0x01)
    }
}

impl Deserialize for u8 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_u8()?)
    }
}

impl Deserialize for i8 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_i8()?)
    }
}

impl Deserialize for u16 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_u16()?)
    }
}

impl Deserialize for i16 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_i16()?)
    }
}

impl Deserialize for i32 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_i32()?)
    }
}

impl Deserialize for i64 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_i64()?)
    }
}

impl Deserialize for f32 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_f32()?)
    }
}

impl Deserialize for f64 {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        Ok(buf.try_get_f64()?)
    }
}

impl Deserialize for String {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        tracing::trace!("reading string");
        let len: usize = *VarInt::deserialize(buf)? as usize;
        tracing::trace!(len, "string size");

        if len > 0 {
            let mut bytes = vec![0u8; len];
            buf.try_copy_to_slice(&mut bytes)?;

            Ok(String::from_utf8(bytes)?)
        } else {
            Ok(String::new())
        }
    }
}

impl Deserialize for Uuid {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let mut bytes = [0u8; 16];
        buf.try_copy_to_slice(&mut bytes)?;

        Ok(Uuid::from_bytes(bytes))
    }
}

impl<D: Deserialize> Deserialize for Vec<D> {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let len = *VarInt::deserialize(buf)?;
        let mut array = vec![];

        for _ in 0..len {
            array.push(D::deserialize(buf)?);
        }

        Ok(array)
    }
}

impl<D: Deserialize> Deserialize for Option<D> {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let present = bool::deserialize(buf)?;
        Ok(if present {
            Some(D::deserialize(buf)?)
        } else {
            None
        })
    }
}

impl Deserialize for Bytes {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let mut bytes = BytesMut::with_capacity(buf.remaining());
        bytes.put(buf);
        Ok(bytes.freeze())
    }
}
