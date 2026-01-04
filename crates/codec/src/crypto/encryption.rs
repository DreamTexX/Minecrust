use bytes::BytesMut;

use crate::crypto::Cfb8Cipher;

#[derive(Debug)]
pub(crate) struct Encryption {
    cipher: Cfb8Cipher,
}

impl Encryption {
    pub(crate) fn new(cipher: Cfb8Cipher) -> Self {
        Self { cipher }
    }

    pub(crate) fn process(&mut self, input: &mut BytesMut) {
        self.cipher.encrypt(input);
    }
}
