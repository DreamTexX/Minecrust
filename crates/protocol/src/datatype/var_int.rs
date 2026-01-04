use bytes::{Buf, BufMut};

use super::{CONTINUE_BIT, SEGMENT_BITS};
use std::ops::Deref;

use crate::{Deserialize, Error, serialize::Serialize};

#[derive(Debug)]
pub struct VarInt(i32);

impl Deserialize for VarInt {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let mut value = 0;
        let mut position = 0;

        for i in 0..5 {
            if buf.remaining() < i + 1 {
                return Err(Error::UnexpectedEof);
            }

            let byte = buf.chunk()[i];
            value |= ((byte & SEGMENT_BITS) as i32) << position;

            if (byte & CONTINUE_BIT) == 0 {
                buf.advance(i + 1);
                return Ok(Self(value));
            }

            position += 7;
        }

        Err(Error::Overflow)
    }
}

impl Serialize for VarInt {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        let mut value = **self;
        loop {
            if (value & !(SEGMENT_BITS as i32)) == 0 {
                buf.put_u8(value as u8);
                break;
            }

            buf.put_u8((value as u8 & SEGMENT_BITS) | CONTINUE_BIT);
            value = (value as u32 >> 7) as i32;
        }
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut};

    use super::*;

    const TEST_CASES: [(i32, &[u8]); 11] = [
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];

    #[test]
    fn test_deserialize() {
        for (expected_num, bytes) in TEST_CASES {
            let mut buf = Bytes::from_static(bytes);

            let var_int = VarInt::deserialize(&mut buf);
            assert!(var_int.is_ok());

            let var_int = var_int.unwrap();
            let int = *var_int;
            assert_eq!(int, expected_num);

            assert_eq!(buf.len(), 0);
        }
    }

    #[test]
    fn test_serialize() {
        for (num, reader) in TEST_CASES {
            let var_int: VarInt = num.into();

            let mut buf = BytesMut::new();
            var_int.serialize(&mut buf);
            assert_eq!(&buf, reader);
        }
    }
}
