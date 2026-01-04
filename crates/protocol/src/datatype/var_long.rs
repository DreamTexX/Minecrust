use bytes::{Buf, BufMut};

use super::{CONTINUE_BIT, SEGMENT_BITS};
use std::ops::Deref;

use crate::{Deserialize, Error, serialize::Serialize};

#[derive(Debug)]
pub struct VarLong(i64);

impl Deserialize for VarLong {
    fn deserialize<B: Buf>(buf: &mut B) -> Result<Self, Error> {
        let mut value = 0;
        let mut position = 0;

        for i in 0..10 {
            if buf.remaining() < i + 1 {
                return Err(Error::UnexpectedEof);
            }

            let byte = buf.chunk()[i];
            value |= ((byte & SEGMENT_BITS) as i64) << position;

            if (byte & CONTINUE_BIT) == 0 {
                buf.advance(i + 1);
                return Ok(Self(value));
            }

            position += 7;
        }

        Err(Error::Overflow)
    }
}

impl Serialize for VarLong {
    fn serialize<B: BufMut>(&self, buf: &mut B) {
        let mut value = **self;
        loop {
            if (value & !(SEGMENT_BITS as i64)) == 0 {
                buf.put_u8(value as u8);
                break;
            }

            buf.put_u8((value as u8 & SEGMENT_BITS) | CONTINUE_BIT);
            value = (value as u64 >> 7) as i64;
        }
    }
}

impl Deref for VarLong {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut};

    use super::*;

    const TEST_CASES: [(i64, &[u8]); 11] = [
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (
            9223372036854775807,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        ),
        (
            -1,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -2147483648,
            &[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -9223372036854775808,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ),
    ];

    #[test]
    fn test_deserialize() {
        for (expected_num, bytes) in TEST_CASES {
            let mut buf = Bytes::from_static(bytes);

            let var_long = VarLong::deserialize(&mut buf);
            assert!(var_long.is_ok());

            let var_long = var_long.unwrap();
            let int = *var_long;
            assert_eq!(int, expected_num);

            assert_eq!(buf.len(), 0);
        }
    }

    #[test]
    fn test_serialize() {
        for (num, reader) in TEST_CASES {
            let var_long: VarLong = num.into();

            let mut bytes = BytesMut::new();
            var_long.serialize(&mut bytes);
            assert_eq!(&bytes, reader);
        }
    }
}
