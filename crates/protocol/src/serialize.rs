use bytes::BufMut;

use crate::datatype::VarInt;

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
        VarInt::from(bytes.len() as i32).serialize(buf);
        buf.put_slice(bytes);
    }
}
