use bytes::BytesMut;

use crate::crypto::Cfb8Cipher;

#[derive(Debug)]
pub(crate) struct Decryption {
    cipher: Cfb8Cipher,
}

impl Decryption {
    pub(crate) fn new(cipher: Cfb8Cipher) -> Self {
        Self { cipher }
    }

    pub(crate) fn process(&mut self, input: &mut BytesMut) {
        self.cipher.decrypt(input);
    }
}
