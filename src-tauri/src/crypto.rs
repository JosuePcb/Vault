use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChaChaNonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::Write;
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
                return Err(CryptoError::InvalidKey(
                    "Key must be 32 bytes for AES-256".to_string(),
                ));
            }
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data)
                .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

            let mut result = nonce_bytes.to_vec();
            result.extend(ciphertext);
            Ok(result)
        }
        CryptoAlgorithm::ChaCha20 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey(
                    "Key must be 32 bytes for ChaCha20".to_string(),
                ));
            }
            let cipher = ChaCha20Poly1305::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let nonce = ChaChaNonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data)
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
                return Err(CryptoError::InvalidKey(
                    "Key must be 32 bytes for AES-256".to_string(),
                ));
            }
            let cipher = Aes256Gcm::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

            let nonce = Nonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];

            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
        }
        CryptoAlgorithm::ChaCha20 => {
            if key.len() != 32 {
                return Err(CryptoError::InvalidKey(
                    "Key must be 32 bytes for ChaCha20".to_string(),
                ));
            }
            let cipher = ChaCha20Poly1305::new_from_slice(key)
                .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

            let nonce = ChaChaNonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];

            cipher
                .decrypt(nonce, ciphertext)
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
    BASE64
        .decode(encoded)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub original_path: String,
    pub algorithm: String,
    pub original_extension: String,
    pub key: String,
}

pub fn encrypt_file_inplace(
    file_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<FileMetadata, CryptoError> {
    let path = std::path::Path::new(file_path);
    let original_extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let original_path = file_path.to_string();

    eprintln!("[ENCRYPT] original_extension='{}'", original_extension);

    let data = std::fs::read(file_path)?;
    eprintln!("[ENCRYPT] archivo original {} bytes", data.len());

    let encrypted = encrypt_data(&data, key, algorithm)?;
    eprintln!(
        "[ENCRYPT] datos cifrados {} bytes (incluye 12 bytes nonce)",
        encrypted.len()
    );

    let key_b64 = key_to_base64(key);
    let algo_str = match algorithm {
        CryptoAlgorithm::Aes256 => "AES-256",
        CryptoAlgorithm::ChaCha20 => "ChaCha20",
    };

    let metadata = FileMetadata {
        original_path: original_path.clone(),
        algorithm: algo_str.to_string(),
        original_extension: original_extension.clone(),
        key: key_b64.clone(),
    };

    #[derive(Serialize)]
    struct MetadataNoKey {
        original_path: String,
        algorithm: String,
        original_extension: String,
    }

    let metadata_no_key = MetadataNoKey {
        original_path,
        algorithm: algo_str.to_string(),
        original_extension,
    };

    let meta_json = serde_json::to_string(&metadata_no_key).map_err(|e| {
        CryptoError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    eprintln!(
        "[ENCRYPT] metadata JSON {} bytes: {}",
        meta_json.len(),
        meta_json
    );

    let mut file_data = encrypted;
    let magic = b"VAULT";
    file_data.extend_from_slice(magic);
    file_data.extend_from_slice(meta_json.as_bytes());

    eprintln!(
        "[ENCRYPT] archivo final {} bytes (ciphertext + magic + metadata)",
        file_data.len()
    );

    std::fs::write(file_path, file_data)?;

    Ok(metadata)
}

pub fn decrypt_file_inplace(
    file_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<String, CryptoError> {
    let file_data = std::fs::read(file_path)?;
    let file_size = file_data.len();
    log::info!("Archivo leido: {} bytes", file_size);

    let magic = b"VAULT";
    let magic_len = magic.len();

    fn find_magic_from_end(data: &[u8], magic: &[u8]) -> Option<usize> {
        if data.len() < magic.len() {
            return None;
        }
        const MAX_METADATA_SIZE: usize = 512;
        let search_start = if data.len() > MAX_METADATA_SIZE + magic.len() {
            data.len() - MAX_METADATA_SIZE - magic.len()
        } else {
            0
        };
        for i in (search_start..data.len() - magic.len() + 1).rev() {
            if &data[i..i + magic.len()] == magic {
                return Some(i);
            }
        }
        None
    }

    eprintln!("[DECRYPT] Archivo leido: {} bytes", file_data.len());

    let magic_pos = find_magic_from_end(&file_data, magic);
    eprintln!(
        "[DECRYPT] Magic VAULT encontrado en posicion: {:?}",
        magic_pos
    );

    if magic_pos.is_none() {
        let meta_path = format!("{}.vault-meta", file_path);
        if std::path::Path::new(&meta_path).exists() {
            log::info!("No se encontro magic VAULT, buscando archivo .vault-meta");
            let meta_json = std::fs::read_to_string(&meta_path)?;
            let metadata: FileMetadata = serde_json::from_str(&meta_json)
                .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

            let encrypted = &file_data;
            let decrypted = decrypt_data(encrypted, key, algorithm)?;

            let base_path = std::path::Path::new(file_path);
            let current_ext = base_path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            let target_ext = metadata.original_extension.trim_start_matches('.');

            let output_path = if current_ext == target_ext {
                file_path.to_string()
            } else if current_ext.is_empty() {
                format!("{}.{}", file_path, target_ext)
            } else {
                format!("{}.{}", file_path, target_ext)
            };

            std::fs::write(&output_path, decrypted)?;
            if output_path != file_path {
                let _ = std::fs::remove_file(file_path);
            }
            let _ = std::fs::remove_file(meta_path);
            return Ok(output_path);
        }
        return Err(CryptoError::DecryptionFailed(
            "No se encontro magic VAULT ni archivo .vault-meta".to_string(),
        ));
    }

    let pos = magic_pos.unwrap();
    eprintln!("[DECRYPT] Posición del magic: {}", pos);
    eprintln!("[DECRYPT] Total bytes: {}", file_data.len());
    eprintln!(
        "[DECRYPT] Bytes despues del magic: {}",
        file_data.len() - pos - magic_len
    );

    let metadata: FileMetadata = {
        let after_magic = &file_data[pos + magic_len..];
        eprintln!(
            "[DECRYPT] Bytes despues magic (raw): {:?}",
            &after_magic[..after_magic.len().min(50)]
        );
        let meta_json = std::str::from_utf8(after_magic)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
        eprintln!("[DECRYPT] Metadata JSON: {}", meta_json);
        serde_json::from_str(meta_json).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?
    };

    let encrypted = &file_data[..pos];
    eprintln!(
        "[DECRYPT] Datos cifrados (sin footer): {} bytes",
        encrypted.len()
    );

    if encrypted.len() < 12 {
        return Err(CryptoError::DecryptionFailed(format!(
            "Datos cifrados muy pequenos: {} bytes",
            encrypted.len()
        )));
    }

    eprintln!(
        "[DECRYPT] Descifrando {} bytes con algoritmo {:?}",
        encrypted.len(),
        algorithm
    );

    let decrypted = decrypt_data(encrypted, key, algorithm)?;
    eprintln!("[DECRYPT] Descifrado exitoso: {} bytes", decrypted.len());

    let base_path = std::path::Path::new(file_path);
    let current_ext = base_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let target_ext = metadata.original_extension.trim_start_matches('.');

    log::info!(
        "Extension actual: '{}', Extension objetivo: '{}'",
        current_ext,
        target_ext
    );

    let output_path = if current_ext == target_ext {
        file_path.to_string()
    } else if current_ext.is_empty() {
        format!("{}.{}", file_path, target_ext)
    } else {
        format!("{}.{}", file_path, target_ext)
    };

    log::info!("Escribiendo archivo descifrado en: {}", output_path);
    std::fs::write(&output_path, decrypted)?;

    if output_path != file_path {
        let _ = std::fs::remove_file(file_path);
    }

    let meta_path = format!("{}.vault-meta", file_path);
    let _ = std::fs::remove_file(meta_path);

    Ok(output_path)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirMetadata {
    pub original_path: String,
    pub algorithm: String,
    pub files: Vec<FileEntry>,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub original_name: String,
    pub extension: String,
}

pub fn encrypt_dir_container(
    dir_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<(DirMetadata, Vec<u8>), CryptoError> {
    let source_dir = std::path::Path::new(dir_path);
    let dir_name = source_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "directory".to_string());

    let mut files: Vec<FileEntry> = Vec::new();
    let mut zip_data: Vec<u8> = Vec::new();

    {
        let cursor = std::io::Cursor::new(&mut zip_data);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for entry in walkdir::WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(source_dir).map_err(|e| {
                CryptoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            let file_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let extension = file_path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();

            let file_data = std::fs::read(file_path)?;
            let encrypted = encrypt_data(&file_data, key, algorithm)?;

            zip.start_file(relative_path.to_string_lossy(), options)
                .map_err(|e| {
                    CryptoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                })?;
            zip.write_all(&encrypted).map_err(|e| {
                CryptoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            files.push(FileEntry {
                path: relative_path.to_string_lossy().to_string(),
                original_name: file_name,
                extension,
            });
        }

        zip.finish().map_err(|e| {
            CryptoError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
    }

    let key_b64 = key_to_base64(key);
    let algo_str = match algorithm {
        CryptoAlgorithm::Aes256 => "AES-256",
        CryptoAlgorithm::ChaCha20 => "ChaCha20",
    };

    let metadata = DirMetadata {
        original_path: dir_path.to_string(),
        algorithm: algo_str.to_string(),
        files,
        key: key_b64,
    };

    let encrypted_zip = encrypt_data(&zip_data, key, algorithm)?;

    let output_path = format!("{}.vault", dir_path);
    std::fs::write(&output_path, &encrypted_zip)?;

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let _ = std::fs::remove_file(entry.path());
    }
    let _ = std::fs::remove_dir(dir_path);

    Ok((metadata, encrypted_zip))
}

pub fn decrypt_dir_container(
    container_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<String, CryptoError> {
    let meta_path = format!("{}.vault-meta", container_path);

    let encrypted_zip = std::fs::read(container_path)?;
    let zip_data = decrypt_data(&encrypted_zip, key, algorithm)?;

    let temp_dir = std::env::temp_dir().join("vault_extract");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).map_err(|e| CryptoError::IoError(e))?;

    {
        let cursor = std::io::Cursor::new(&zip_data);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            CryptoError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                CryptoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            let out_path = temp_dir.join(file.name());
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| CryptoError::IoError(e))?;
            }

            if !file.is_dir() {
                let mut encrypted_content = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut encrypted_content).map_err(|e| {
                    CryptoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                })?;

                let decrypted = decrypt_data(&encrypted_content, key, algorithm)?;
                std::fs::write(&out_path, decrypted).map_err(|e| CryptoError::IoError(e))?;
            }
        }
    }

    let output_path = std::path::PathBuf::from(container_path.trim_end_matches(".vault"));
    std::fs::create_dir_all(&output_path).map_err(|e| CryptoError::IoError(e))?;

    for entry in walkdir::WalkDir::new(&temp_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let dest = entry.path().strip_prefix(&temp_dir).map_err(|e| {
            CryptoError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        let dest_path = output_path.join(dest);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path).map_err(|e| CryptoError::IoError(e))?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| CryptoError::IoError(e))?;
            }
            std::fs::copy(entry.path(), &dest_path).map_err(|e| CryptoError::IoError(e))?;
        }
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::remove_file(container_path);
    let _ = std::fs::remove_file(meta_path);

    Ok(output_path.to_string_lossy().to_string())
}

const VAULT_FILE_MAGIC: &[u8; 14] = b"VAULT_FILE_END";

pub fn encrypt_file_vault(
    file_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<FileMetadata, CryptoError> {
    let path = std::path::Path::new(file_path);
    let original_extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let original_path = file_path.to_string();

    let data = std::fs::read(file_path)?;
    let encrypted = encrypt_data(&data, key, algorithm)?;

    let mut vault_data = encrypted;
    vault_data.extend_from_slice(VAULT_FILE_MAGIC);

    let vault_path = format!("{}.vault", file_path);
    std::fs::write(&vault_path, &vault_data)?;

    let key_b64 = key_to_base64(key);
    let algo_str = match algorithm {
        CryptoAlgorithm::Aes256 => "AES-256",
        CryptoAlgorithm::ChaCha20 => "ChaCha20",
    };

    let metadata = FileMetadata {
        original_path: original_path.clone(),
        algorithm: algo_str.to_string(),
        original_extension: original_extension.clone(),
        key: key_b64.clone(),
    };

    let meta_path = format!("{}.vault-meta", file_path);
    let meta_json = serde_json::to_string(&metadata).map_err(|e| {
        CryptoError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    std::fs::write(&meta_path, meta_json)?;

    let _ = std::fs::remove_file(file_path);

    Ok(FileMetadata {
        original_path,
        algorithm: algo_str.to_string(),
        original_extension,
        key: key_b64.clone(),
    })
}

pub fn decrypt_file_vault(
    file_path: &str,
    key: &[u8],
    algorithm: &CryptoAlgorithm,
) -> Result<String, CryptoError> {
    let meta_path = format!("{}-meta", file_path);
    if !std::path::Path::new(&meta_path).exists() {
        return Err(CryptoError::DecryptionFailed(
            "Archivo .vault-meta no encontrado".to_string(),
        ));
    }

    let meta_json = std::fs::read_to_string(&meta_path)?;
    let metadata: FileMetadata = serde_json::from_str(&meta_json)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    let vault_data = std::fs::read(file_path)?;

    let magic_len = VAULT_FILE_MAGIC.len();
    if vault_data.len() < magic_len {
        return Err(CryptoError::DecryptionFailed(
            "Archivo vault corrupto".to_string(),
        ));
    }

    let magic_from_file = &vault_data[vault_data.len() - magic_len..];
    if magic_from_file != VAULT_FILE_MAGIC {
        return Err(CryptoError::DecryptionFailed(
            "Magic inválido - archivo corrupto".to_string(),
        ));
    }

    let encrypted = &vault_data[..vault_data.len() - magic_len];
    let decrypted = decrypt_data(encrypted, key, algorithm)?;

    let output_path = if file_path.ends_with(".vault") {
        format!(
            "{}{}",
            file_path.trim_end_matches(".vault"),
            metadata.original_extension
        )
    } else {
        return Err(CryptoError::DecryptionFailed(
            "Archivo no tiene extensión .vault".to_string(),
        ));
    };

    std::fs::write(&output_path, decrypted)?;

    let _ = std::fs::remove_file(file_path);
    let _ = std::fs::remove_file(&meta_path);

    Ok(output_path)
}
