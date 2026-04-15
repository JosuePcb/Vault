use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChaChaNonce};
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CryptoAlgorithm {
    #[serde(rename = "AES-256")]
    Aes256,
    #[serde(rename = "ChaCha20")]
    ChaCha20,
}

impl Default for CryptoAlgorithm {
    fn default() -> Self {
        CryptoAlgorithm::Aes256
    }
}

pub fn generate_key(algorithm: &CryptoAlgorithm) -> Vec<u8> {
    let key_len = match algorithm {
        CryptoAlgorithm::Aes256 => 32,
        CryptoAlgorithm::ChaCha20 => 32,
    };
    let mut key = vec![0u8; key_len];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

pub fn encrypt_data(
    data: &[u8],
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<Vec<u8>, CryptoError> {
    match algorithm {
        CryptoAlgorithm::Aes256 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey("Key must be 32 bytes for AES-256".to_string()));
            }
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
            
            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, data)
                .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
            
            let mut result = nonce_bytes.to_vec();
            result.extend(ciphertext);
            Ok(result)
        }
        CryptoAlgorithm::ChaCha20 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey("Key must be 32 bytes for ChaCha20".to_string()));
            }
            let cipher = ChaCha20Poly1305::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
            
            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, data)
                .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
            
            let mut result = nonce_bytes.to_vec();
            result.extend(ciphertext);
            Ok(result)
        }
    }
}

pub fn decrypt_data(
    encrypted_data: &[u8],
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<Vec<u8>, CryptoError> {
    if encrypted_data.len() < 12 {
        return Err(CryptoError::DecryptionFailed("Data too short".to_string()));
    }
    
    match algorithm {
        CryptoAlgorithm::Aes256 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey("Key must be 32 bytes for AES-256".to_string()));
            }
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
            
            let nonce = Nonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];
            
            cipher.decrypt(nonce, ciphertext)
                .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
        }
        CryptoAlgorithm::ChaCha20 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey("Key must be 32 bytes for ChaCha20".to_string()));
            }
            let cipher = ChaCha20Poly1305::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
            
            let nonce = ChaChaNonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];
            
            cipher.decrypt(nonce, ciphertext)
                .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
        }
    }
}

pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<(), CryptoError> {
    let data = std::fs::read(input_path)?;
    let encrypted = encrypt_data(&data, key, algorithm)?;
    std::fs::write(output_path, encrypted)?;
    Ok(())
}

pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<(), CryptoError> {
    let encrypted = std::fs::read(input_path)?;
    let decrypted = decrypt_data(&encrypted, key, algorithm)?;
    std::fs::write(output_path, decrypted)?;
    Ok(())
}

pub fn key_to_base64(key: &[u8]) -> String {
    BASE64.encode(key)
}

pub fn key_from_base64(encoded: &str) -> Result<Vec<u8>, CryptoError> {
    BASE64.decode(encoded)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))
}
