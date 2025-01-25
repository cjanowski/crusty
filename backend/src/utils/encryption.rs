use ring::aead::{self, BoundKey, OpeningKey, SealingKey, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use std::error::Error;

struct NonceGen {
    nonce: [u8; 12],
}

impl aead::NonceSequence for NonceGen {
    fn advance(&mut self) -> Result<aead::Nonce, ring::error::Unspecified> {
        Ok(aead::Nonce::assume_unique_for_key(self.nonce))
    }
}

pub struct EncryptionService {
    rng: SystemRandom,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }

    pub fn generate_key(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut key = vec![0; 32]; // 256-bit key
        self.rng.fill(&mut key)?;
        Ok(key)
    }

    pub fn generate_nonce(&self) -> Result<[u8; 12], Box<dyn Error>> {
        let mut nonce = [0u8; 12];
        self.rng.fill(&mut nonce)?;
        Ok(nonce)
    }

    pub fn encrypt(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, [u8; 12]), Box<dyn Error>> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)?;
        let nonce = self.generate_nonce()?;
        let nonce_gen = NonceGen { nonce };
        let mut sealing_key = SealingKey::new(unbound_key, nonce_gen);
        
        let mut in_out = data.to_vec();
        sealing_key.seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)?;
        
        Ok((in_out, nonce))
    }

    pub fn decrypt(&self, encrypted_data: &[u8], key: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, Box<dyn Error>> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)?;
        let nonce_gen = NonceGen { nonce: *nonce };
        let mut opening_key = OpeningKey::new(unbound_key, nonce_gen);
        
        let mut in_out = encrypted_data.to_vec();
        let decrypted = opening_key.open_in_place(aead::Aad::empty(), &mut in_out)?;
        
        Ok(decrypted.to_vec())
    }
} 