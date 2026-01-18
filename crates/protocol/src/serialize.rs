use bytes::{BufMut, Bytes};
use uuid::Uuid;

use crate::datatype::var_int;

pub trait Serialize {
    fn serialize<B: BufMut>(&self, buf: &mut B);
}

impl Serialize for bool {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(if *self { 0x01 } else { 0x00 });
    }
}

impl Serialize for u8 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self);
    }
}

impl Serialize for i8 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_i8(*self);
    }
}

impl Serialize for u16 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_u16(*self);
    }
}

impl Serialize for i16 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_i16(*self);
    }
}

impl Serialize for i32 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32(*self);
    }
}

impl Serialize for i64 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64(*self);
    }
}

impl Serialize for f32 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_f32(*self);
    }
}

impl Serialize for f64 {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64(*self);
    }
}

impl Serialize for String {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        let bytes = self.as_bytes();
        var_int::serialize(&(bytes.len() as i32), buf);
        buf.put_slice(bytes);
    }
}

impl Serialize for Uuid {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(self.as_bytes());
    }
}

impl<S: Serialize> Serialize for Vec<S> {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        let len = self.len();
        var_int::serialize(&(len as i32), buf);

        for item in self {
            item.serialize(buf);
        }
    }
}

impl<S: Serialize> Serialize for Option<S> {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        if let Some(data) = self {
            true.serialize(buf);
            data.serialize(buf);
        } else {
            false.serialize(buf);
        }
    }
}

impl Serialize for Bytes {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        buf.put(&self[..]);
    }
}

impl<S: Serialize, const N: usize> Serialize for [S; N] {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        let len = self.len();
        var_int::serialize(&(len as i32), buf);

        for item in self {
            item.serialize(buf);
        }
    }
}
