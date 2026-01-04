use aes::{Aes128, Block};
use cipher::{BlockEncrypt, KeyInit};

#[derive(Debug)]
pub(crate) struct Cfb8Cipher {
    cipher: Aes128,
    sr: [u8; 16],
}

impl Cfb8Cipher {
    pub fn new(shared_secret: &[u8; 16]) -> Self {
        Self {
            cipher: Aes128::new(shared_secret.into()),
            sr: *shared_secret,
        }
    }

    pub fn encrypt_byte(&mut self, byte: u8) -> u8 {
        let mut block = *Block::from_slice(&self.sr);
        self.cipher.encrypt_block(&mut block);

        let cipher_byte = byte ^ block[0];
        self.sr.copy_within(1.., 0);
        self.sr[15] = cipher_byte;

        cipher_byte
    }

    pub fn decrypt_byte(&mut self, byte: u8) -> u8 {
        let mut block = *Block::from_slice(&self.sr);
        self.cipher.encrypt_block(&mut block);

        let plain_byte = byte ^ block[0];
        self.sr.copy_within(1.., 0);
        self.sr[15] = byte;

        plain_byte
    }

    pub fn encrypt(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.encrypt_byte(*b);
        }
    }

    pub fn decrypt(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.decrypt_byte(*b);
        }
    }
}
