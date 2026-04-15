mod crypto;
mod audit;
mod watcher;
mod email;

use std::sync::Mutex;
use tauri::State;
use serde::{Deserialize, Serialize};
use crypto::{CryptoAlgorithm, generate_key, encrypt_file, decrypt_file, key_to_base64, key_from_base64};
use audit::AuditLog;
use watcher::FileWatcher;
use email::{EmailClient, EmailConfig};

pub struct AppState {
    pub audit_log: Mutex<Option<AuditLog>>,
    pub file_watcher: Mutex<Option<FileWatcher>>,
    pub email_client: Mutex<EmailClient>,
    pub encryption_key: Mutex<Option<Vec<u8>>>,
    pub algorithm: Mutex<CryptoAlgorithm>,
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
    *state.algorithm.lock().map_err(|e| e.to_string())? = algo;
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
    input_path: String,
    output_path: String,
) -> Result<EncryptResult, String> {
    let key = state.encryption_key.lock().map_err(|e| e.to_string())?
        .clone()
        .ok_or("No encryption key set")?;
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    encrypt_file(&input_path, &output_path, &key, &algorithm).map_err(|e| e.to_string())?;
    
    // Log the event
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("encrypt", &input_path, "File encrypted");
        }
    }
    
    Ok(EncryptResult {
        success: true,
        output_path,
        key: key_to_base64(&key),
    })
}

#[tauri::command]
fn decrypt_file_cmd(
    state: State<AppState>,
    input_path: String,
    output_path: String,
    key_base64: String,
) -> Result<DecryptResult, String> {
    let key = key_from_base64(&key_base64).map_err(|e| e.to_string())?;
    let algorithm = state.algorithm.lock().map_err(|e| e.to_string())?.clone();
    
    decrypt_file(&input_path, &output_path, &key, &algorithm).map_err(|e| e.to_string())?;
    
    // Log the event
    if let Ok(audit) = state.audit_log.lock() {
        if let Some(ref log) = *audit {
            let _ = log.log_event("decrypt", &input_path, "File decrypted");
        }
    }
    
    Ok(DecryptResult {
        success: true,
        output_path,
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
fn validate_audit_integrity(state: State<AppState>) -> Result<bool, String> {
    let audit = state.audit_log.lock().map_err(|e| e.to_string())?;
    let log = audit.as_ref().ok_or("Audit log not initialized")?;
    log.validate_integrity().map_err(|e| e.to_string())
}

#[tauri::command]
fn start_watching(
    path: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn stop_watching(path: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn get_watched_paths() -> Result<Vec<String>, String> {
    Ok(vec![])
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

#[tauri::command]
fn is_email_configured(state: State<AppState>) -> Result<bool, String> {
    let client = state.email_client.lock().map_err(|e| e.to_string())?;
    Ok(client.is_configured())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vault");
    
    std::fs::create_dir_all(&app_data_dir).ok();
    
    let db_path = app_data_dir.join("audit.db");
    let hmac_key = generate_key(&CryptoAlgorithm::Aes256);
    
    let audit_log = AuditLog::new(db_path.to_str().unwrap_or("audit.db"), &hmac_key).ok();
    let file_watcher = FileWatcher::new();
    
    let state = AppState {
        audit_log: Mutex::new(audit_log),
        file_watcher: Mutex::new(None),
        email_client: Mutex::new(EmailClient::new()),
        encryption_key: Mutex::new(None),
        algorithm: Mutex::new(CryptoAlgorithm::Aes256),
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
            start_watching,
            stop_watching,
            get_watched_paths,
            configure_email,
            send_email_alert,
            test_email,
            is_email_configured,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
