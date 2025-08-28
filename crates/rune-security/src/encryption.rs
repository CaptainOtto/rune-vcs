use anyhow::Result;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng, AeadCore};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivation,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivation {
    PBKDF2,
    Argon2,
    Scrypt,
}

pub struct EncryptionManager {
    config: EncryptionConfig,
}

impl EncryptionManager {
    pub fn new(config: EncryptionConfig) -> Self {
        Self { config }
    }

    pub fn encrypt_data(&self, data: &[u8], password: &str) -> Result<Vec<u8>> {
        match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => self.encrypt_aes256gcm(data, password),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // TODO: Implement ChaCha20Poly1305
                Err(anyhow::anyhow!("ChaCha20Poly1305 not yet implemented"))
            }
        }
    }

    pub fn decrypt_data(&self, encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
        match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => self.decrypt_aes256gcm(encrypted_data, password),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // TODO: Implement ChaCha20Poly1305
                Err(anyhow::anyhow!("ChaCha20Poly1305 not yet implemented"))
            }
        }
    }

    fn encrypt_aes256gcm(&self, data: &[u8], password: &str) -> Result<Vec<u8>> {
        let key = self.derive_key(password)?;
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Combine nonce + ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    fn decrypt_aes256gcm(&self, encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }

        let key = self.derive_key(password)?;
        let cipher = Aes256Gcm::new(&key);
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }

    fn derive_key(&self, password: &str) -> Result<Key<Aes256Gcm>> {
        match self.config.key_derivation {
            KeyDerivation::PBKDF2 => {
                // Simple key derivation for now
                let mut context = Context::new(&SHA256);
                context.update(password.as_bytes());
                let digest = context.finish();
                let key_bytes = digest.as_ref();
                Ok(*Key::<Aes256Gcm>::from_slice(key_bytes))
            }
            _ => Err(anyhow::anyhow!("Key derivation method not implemented")),
        }
    }

    pub fn hash_data(&self, data: &[u8]) -> String {
        let mut context = Context::new(&SHA256);
        context.update(data);
        let digest = context.finish();
        general_purpose::STANDARD.encode(digest.as_ref())
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_derivation: KeyDerivation::PBKDF2,
            compression_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config);
        
        let original_data = b"Hello, World!";
        let password = "test_password";
        
        let encrypted = manager.encrypt_data(original_data, password).unwrap();
        let decrypted = manager.decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(original_data, decrypted.as_slice());
    }

    #[test]
    fn test_hash_data() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::new(config);
        
        let data = b"test data";
        let hash1 = manager.hash_data(data);
        let hash2 = manager.hash_data(data);
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
