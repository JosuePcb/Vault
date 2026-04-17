mod crypto;
mod audit;
mod watcher;
mod email;
mod auth;

use std::sync::Mutex;
use tauri::State;
use serde::{Deserialize, Serialize};
use crypto::{CryptoAlgorithm, generate_key, encrypt_file, decrypt_file, key_to_base64, key_from_base64, encrypt_file_inplace, decrypt_file_inplace, FileMetadata, encrypt_dir_container, decrypt_dir_container, DirMetadata};
use audit::AuditLog;
use watcher::FileWatcher;
use email::{EmailClient, EmailConfig};
use auth::AuthResult;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub alert_email: String,
    pub algorithm: String,
    pub password_hash: Option<String>,
    pub watched_paths: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            alert_email: String::new(),
            algorithm: "AES-256".to_string(),
            password_hash: None,
            watched_paths: Vec::new(),
        }
    }
}

fn get_config_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vault")
        .join("config.json")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub struct AppState {
    pub audit_log: Mutex<Option<AuditLog>>,
    pub file_watcher: Mutex<Option<FileWatcher>>,
    pub email_client: Mutex<EmailClient>,
    pub encryption_key: Mutex<Option<Vec<u8>>>,
    pub algorithm: Mutex<CryptoAlgorithm>,
    pub alert_email: Mutex<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoStats {
    pub files_encrypted: i64,
    pub dirs_watched: i64,
    pub audit_events: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptResult {
    pub success: bool,
    pub output_path: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptResult {
    pub success: bool,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub is_valid: bool,
    pub status: String,
    pub last_valid_id: i64,
    pub failed_at: Option<String>,
    pub details: String,
}

#[tauri::command]
fn generate_encryption_key(state: State<AppState>) -> Result<String, String> {
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?;
    let key = generate_key(&algorithm);
    *state.encryption_key.lock().map_err(|e| e.to_string())? = Some(key.clone());
    Ok(key_to_base64(&key))
}

#[tauri::command]
fn set_encryption_key(state: State<AppState>, key_base64: String) -> Result<(), String> {
    let key = key_from_base64(&key_base64).map_err(|e| e.to_string())?;
    *state.encryption_key.lock().map_err(|e| e.to_string())? = Some(key);
    Ok(())
}

#[tauri::command]
fn set_algorithm(state: State<AppState>, algorithm: String) -> Result<(), String> {
    let algo = match algorithm.as_str() {
        "AES-256" => CryptoAlgorithm::Aes256,
        "ChaCha20" => CryptoAlgorithm::ChaCha20,
        _ => return Err("Invalid algorithm".to_string()),
    };
    *state.algorithm.lock().map_err(|e| e.to_string())? = algo.clone();
    
    let alert_email = state.alert_email.lock().map_err(|e| e.to_string())?.clone();
    let config = AppConfig {
        alert_email,
        algorithm: algorithm,
        password_hash: None,
        watched_paths: Vec::new(),
    };
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
fn get_algorithm(state: State<AppState>) -> Result<String, String> {
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?;
    let algo_str = match *algorithm {
        CryptoAlgorithm::Aes256 => "AES-256",
        CryptoAlgorithm::ChaCha20 => "ChaCha20",
    };
    Ok(algo_str.to_string())
}

#[tauri::command]
fn encrypt_file_cmd(
    state: State<AppState>,
    file_path: String,
) -> Result<FileMetadata, String> {
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    let key = if let Ok(mut mutex) = state.encryption_key.lock() {
        if let Some(ref k) = *mutex {
            k.clone()
        } else {
            let new_key = generate_key(&algorithm);
            let key_clone = new_key.clone();
            *mutex = Some(new_key);
            key_clone
        }
    } else {
        return Err("Failed to access encryption key state".to_string());
    };
    
    let metadata = encrypt_file_inplace(&file_path, &key, &algorithm).map_err(|e| e.to_string())?;
    
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("encrypt", &file_path, "File encrypted in-place");
        }
    }
    
    send_audit_email(&state, &file_path, "encrypt", "Archivo cifrado");
    
    if let Ok(mut key_guard) = state.encryption_key.lock() {
        *key_guard = None;
    }
    
    Ok(metadata)
}

#[tauri::command]
fn decrypt_file_cmd(
    state: State<AppState>,
    file_path: String,
    key_base64: String,
) -> Result<DecryptResult, String> {
    let key = key_from_base64(&key_base64).map_err(|e| e.to_string())?;
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    let original_extension = decrypt_file_inplace(&file_path, &key, &algorithm).map_err(|e| e.to_string())?;
    
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("decrypt", &file_path, "File decrypted in-place");
        }
    }
    
    send_audit_email(&state, &file_path, "decrypt", "Archivo descifrado");
    
    let _ = state.encryption_key.lock().map(|mut k| *k = None);
    
    Ok(DecryptResult {
        success: true,
        output_path: original_extension,
    })
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<CryptoStats, String> {
    let audit_events = if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            log.get_event_count().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    let dirs_watched = if let Ok(watcher) = state.file_watcher.lock() {
        if let Some(ref w) = *watcher {
            w.get_watched_paths().map(|p| p.len() as i64).unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    Ok(CryptoStats {
        files_encrypted: 0,
        dirs_watched,
        audit_events,
    })
}

#[tauri::command]
fn get_audit_logs(
    state: State<AppState>,
    limit: Option<i64>,
    event_type: Option<String>,
) -> Result<Vec<audit::AuditEvent>, String> {
    let audit = state.audit_log.lock().map_err(|e| e.to_string())?;
    let log = audit.as_ref().ok_or("Audit log not initialized")?;
    log.get_events(limit, event_type.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_audit_integrity(state: State<AppState>) -> Result<IntegrityResult, String> {
    let audit = state.audit_log.lock().map_err(|e| e.to_string())?;
    let log = audit.as_ref().ok_or("Audit log not initialized")?;
    
    let (is_valid, last_valid_id, failed_at, details) = log.validate_integrity_detailed()
        .map_err(|e| e.to_string())?;
    
    Ok(IntegrityResult {
        is_valid,
        status: if is_valid { "VALID".to_string() } else { "COMPROMISED".to_string() },
        last_valid_id,
        failed_at,
        details,
    })
}

#[tauri::command]
fn repair_audit_integrity(state: State<AppState>) -> Result<IntegrityResult, String> {
    let audit = state.audit_log.lock().map_err(|e| e.to_string())?;
    let log = audit.as_ref().ok_or("Audit log not initialized")?;
    
    let (is_valid, last_valid_id, failed_at, details) = log.repair_integrity().map_err(|e| e.to_string())?;
    
    Ok(IntegrityResult {
        is_valid,
        status: if is_valid { "REPAIRED".to_string() } else { "ERROR".to_string() },
        last_valid_id,
        failed_at,
        details,
    })
}

#[tauri::command]
fn start_watching(
    state: State<AppState>,
    path: String,
) -> Result<(), String> {
    let watcher = state.file_watcher.lock().map_err(|e| e.to_string())?;
    
    if let Some(ref w) = *watcher {
        w.start_watching(&path).map_err(|e| e.to_string())?;
    }
    
    let mut config = load_config();
    if !config.watched_paths.contains(&path) {
        config.watched_paths.push(path);
        save_config(&config)?;
    }
    
    Ok(())
}

#[tauri::command]
fn stop_watching(
    state: State<AppState>,
    path: String,
) -> Result<(), String> {
    let watcher = state.file_watcher.lock().map_err(|e| e.to_string())?;
    
    if let Some(ref w) = *watcher {
        w.stop_watching(&path).map_err(|e| e.to_string())?;
    }
    
    let mut config = load_config();
    config.watched_paths.retain(|p| p != &path);
    save_config(&config)?;
    
    Ok(())
}

#[tauri::command]
fn get_watched_paths(state: State<AppState>) -> Result<Vec<String>, String> {
    let watcher = state.file_watcher.lock().map_err(|e| e.to_string())?;
    
    if let Some(ref w) = *watcher {
        w.get_watched_paths().map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn get_watcher_events(state: State<AppState>, limit: usize) -> Result<Vec<watcher::FileEvent>, String> {
    let watcher = state.file_watcher.lock().map_err(|e| e.to_string())?;
    
    if let Some(ref w) = *watcher {
        Ok(w.get_recent_events(limit))
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn configure_email(state: State<AppState>, api_key: String, from_email: String, from_name: String) -> Result<(), String> {
    let mut client = state.email_client.lock().map_err(|e| e.to_string())?;
    client.configure(EmailConfig {
        api_key,
        from_email,
        from_name,
    });
    Ok(())
}

#[tauri::command]
fn send_email_alert(
    state: State<AppState>,
    to: Vec<String>,
    path: String,
    event_type: String,
    description: String,
) -> Result<String, String> {
    let client = state.email_client.lock().map_err(|e| e.to_string())?;
    
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(client.send_alert(&to, &path, &event_type, &description))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn test_email(state: State<AppState>, to: Vec<String>) -> Result<String, String> {
    let client = state.email_client.lock().map_err(|e| e.to_string())?;
    
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(client.send(&to, "Vault - Test", "Test email from Vault", None))
        .map_err(|e| e.to_string())
}

fn send_audit_email(state: &State<AppState>, path: &str, event_type: &str, description: &str) {
    if let (Ok(client), Ok(alert_email)) = (state.email_client.lock(), state.alert_email.lock()) {
        if !alert_email.is_empty() && client.is_configured() {
            let runtime = tokio::runtime::Runtime::new();
            if let Ok(runtime) = runtime {
                let _ = runtime.block_on(client.send_alert(
                    &[alert_email.clone()],
                    path,
                    event_type,
                    description
                ));
            }
        }
    }
}

#[tauri::command]
fn is_email_configured(state: State<AppState>) -> Result<bool, String> {
    let client = state.email_client.lock().map_err(|e| e.to_string())?;
    Ok(client.is_configured())
}

#[tauri::command]
fn set_alert_email(state: State<AppState>, email: String) -> Result<(), String> {
    let mut alert_email = state.alert_email.lock().map_err(|e| e.to_string())?;
    *alert_email = email.clone();
    
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    let algo_str = match algorithm {
        CryptoAlgorithm::Aes256 => "AES-256",
        CryptoAlgorithm::ChaCha20 => "ChaCha20",
    };
    
    let config = AppConfig {
        alert_email: email,
        algorithm: algo_str.to_string(),
        password_hash: None,
        watched_paths: Vec::new(),
    };
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
fn get_alert_email(state: State<AppState>) -> Result<String, String> {
    let alert_email = state.alert_email.lock().map_err(|e| e.to_string())?;
    Ok(alert_email.clone())
}

#[tauri::command]
fn send_alert(state: State<AppState>, path: String, event_type: String, description: String) -> Result<String, String> {
    let client = state.email_client.lock().map_err(|e| e.to_string())?;
    let alert_email = state.alert_email.lock().map_err(|e| e.to_string())?;
    
    if alert_email.is_empty() {
        return Err("Email de alertas no configurado".to_string());
    }
    
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(client.send_alert(&[alert_email.clone()], &path, &event_type, &description))
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirEncryptResult {
    pub success: bool,
    pub files_encrypted: Vec<String>,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirDecryptResult {
    pub success: bool,
    pub files_decrypted: Vec<String>,
}

#[tauri::command]
fn encrypt_dir_cmd(
    state: State<AppState>,
    input_dir: String,
) -> Result<DirEncryptResult, String> {
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    let key = if let Ok(mut mutex) = state.encryption_key.lock() {
        if let Some(ref k) = *mutex {
            k.clone()
        } else {
            let new_key = generate_key(&algorithm);
            let key_clone = new_key.clone();
            *mutex = Some(new_key);
            key_clone
        }
    } else {
        return Err("Failed to access encryption key state".to_string());
    };
    
    let input_path = std::path::Path::new(&input_dir);
    
    if !input_path.exists() {
        return Err("Input directory does not exist".to_string());
    }
    
    let (metadata, _encrypted) = encrypt_dir_container(&input_dir, &key, &algorithm).map_err(|e| e.to_string())?;
    
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("encrypt_dir", &input_dir, &format!("{} files encrypted", metadata.files.len()));
        }
    }
    
    send_audit_email(&state, &input_dir, "encrypt_dir", &format!("{} archivos cifrados", metadata.files.len()));
    
    let _ = state.encryption_key.lock().map(|mut k| *k = None);
    
    Ok(DirEncryptResult {
        success: true,
        files_encrypted: metadata.files.iter().map(|f| f.path.clone()).collect(),
        key: metadata.key,
    })
}

#[tauri::command]
fn check_auth_status() -> Result<bool, String> {
    Ok(auth::is_password_set())
}

#[tauri::command]
fn setup_password(password: String) -> Result<AuthResult, String> {
    if password.len() < 4 {
        return Ok(AuthResult {
            success: false,
            message: "Password must be at least 4 characters".to_string(),
        });
    }
    auth::set_password(&password).map_err(|e| e.to_string())
}

#[tauri::command]
fn login(password: String) -> Result<AuthResult, String> {
    auth::check_password(&password).map_err(|e| e.to_string())
}

#[tauri::command]
fn decrypt_dir_cmd(
    state: State<AppState>,
    input_file: String,
    key_base64: String,
) -> Result<DirDecryptResult, String> {
    let key = key_from_base64(&key_base64).map_err(|e| e.to_string())?;
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    let result = decrypt_dir_container(&input_file, &key, &algorithm).map_err(|e| e.to_string())?;
    
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("decrypt_dir", &input_file, "Directory decrypted");
        }
    }
    
    send_audit_email(&state, &input_file, "decrypt_dir", "Directorio descifrado");
    
    Ok(DirDecryptResult {
        success: true,
        files_decrypted: vec![result],
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vault");
    
    std::fs::create_dir_all(&app_data_dir).ok();
    
    let db_path = app_data_dir.join("audit.db");
    let config = load_config();
    
    let algorithm = match config.algorithm.as_str() {
        "ChaCha20" => CryptoAlgorithm::ChaCha20,
        _ => CryptoAlgorithm::Aes256,
    };
    
    dotenv::dotenv().ok();
    
    let hmac_key = generate_key(&CryptoAlgorithm::Aes256);
    
    let audit_log = AuditLog::new(db_path.to_str().unwrap_or("audit.db"), &hmac_key).ok();
    let file_watcher = FileWatcher::new();
    
    for path in &config.watched_paths {
        if std::path::Path::new(path).exists() {
            if let Err(e) = file_watcher.start_watching(path) {
                eprintln!("Failed to restore watch on {}: {}", path, e);
            }
        }
    }
    
    let mut email_client = EmailClient::new();
    let resend_api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
    if !resend_api_key.is_empty() {
        email_client.configure(EmailConfig {
            api_key: resend_api_key,
            from_email: "noreply@resend.dev".to_string(),
            from_name: "Vault".to_string(),
        });
    }
    
    let state = AppState {
        audit_log: Mutex::new(audit_log),
        file_watcher: Mutex::new(Some(file_watcher)),
        email_client: Mutex::new(email_client),
        encryption_key: Mutex::new(None),
        algorithm: Mutex::new(algorithm),
        alert_email: Mutex::new(config.alert_email),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            generate_encryption_key,
            set_encryption_key,
            set_algorithm,
            get_algorithm,
            encrypt_file_cmd,
            decrypt_file_cmd,
            get_stats,
            get_audit_logs,
            validate_audit_integrity,
            repair_audit_integrity,
            start_watching,
            stop_watching,
            get_watched_paths,
            get_watcher_events,
            configure_email,
            send_email_alert,
            test_email,
            is_email_configured,
            set_alert_email,
            get_alert_email,
            send_alert,
            encrypt_dir_cmd,
            decrypt_dir_cmd,
            check_auth_status,
            setup_password,
            login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
