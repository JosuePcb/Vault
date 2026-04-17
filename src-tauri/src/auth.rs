use argon2::password_hash::rand_core::OsRng;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Password hashing failed: {0}")]
    HashError(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("No password set")]
    NoPasswordSet,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub message: String,
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::HashError(e.to_string()))?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        argon2::PasswordHash::new(hash).map_err(|e| AuthError::HashError(e.to_string()))?;

    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn is_password_set() -> bool {
    let config = load_auth_config();
    config.password_hash.is_some()
}

pub fn set_password(password: &str) -> Result<AuthResult, AuthError> {
    let hash = hash_password(password)?;

    let mut config = load_auth_config();
    config.password_hash = Some(hash.clone());
    save_auth_config(&config)?;

    Ok(AuthResult {
        success: true,
        message: "Password configured successfully".to_string(),
    })
}

pub fn check_password(password: &str) -> Result<AuthResult, AuthError> {
    let config = load_auth_config();

    match &config.password_hash {
        Some(hash) => {
            if verify_password(password, hash)? {
                Ok(AuthResult {
                    success: true,
                    message: "Authentication successful".to_string(),
                })
            } else {
                Ok(AuthResult {
                    success: false,
                    message: "Invalid password".to_string(),
                })
            }
        }
        None => Err(AuthError::NoPasswordSet),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthConfig {
    password_hash: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            password_hash: None,
        }
    }
}

fn get_auth_config_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vault")
        .join("auth.json")
}

fn load_auth_config() -> AuthConfig {
    let config_path = get_auth_config_path();
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AuthConfig::default()
}

fn save_auth_config(config: &AuthConfig) -> Result<(), AuthError> {
    let config_path = get_auth_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content =
        serde_json::to_string_pretty(config).map_err(|e| AuthError::HashError(e.to_string()))?;
    std::fs::write(config_path, content)?;
    Ok(())
}
