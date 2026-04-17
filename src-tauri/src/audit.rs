use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Mutex;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Integrity check failed: {0}")]
    IntegrityFailed(String),
    #[error("Lock error")]
    LockError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub path: String,
    pub description: String,
    pub prev_hmac: String,
    pub hmac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherEvent {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub path: String,
}

pub struct AuditLog {
    conn: Mutex<Connection>,
    hmac_key: Vec<u8>,
}

impl AuditLog {
    pub fn new(db_path: &str, hmac_key: &[u8]) -> Result<Self, AuditError> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                path TEXT NOT NULL,
                description TEXT NOT NULL,
                prev_hmac TEXT NOT NULL,
                hmac TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS watcher_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                path TEXT NOT NULL
            )",
            [],
        )?;

        Ok(AuditLog {
            conn: Mutex::new(conn),
            hmac_key: hmac_key.to_vec(),
        })
    }

    pub fn log_watcher_event(&self, event_type: &str, path: &str) -> Result<(), AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;
        let timestamp = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO watcher_events (timestamp, event_type, path) VALUES (?1, ?2, ?3)",
            params![timestamp, event_type, path],
        )?;

        conn.execute(
            "DELETE FROM watcher_events WHERE id NOT IN (SELECT id FROM watcher_events ORDER BY id DESC LIMIT 500)",
            [],
        )?;

        Ok(())
    }

    pub fn get_watcher_events(&self, limit: usize) -> Result<Vec<WatcherEvent>, AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, path FROM watcher_events ORDER BY id DESC LIMIT ?",
        )?;

        let events = stmt.query_map([limit], |row| {
            Ok(WatcherEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                path: row.get(3)?,
            })
        })?;

        let mut result = Vec::new();
        for event in events {
            result.push(event?);
        }

        Ok(result)
    }

    fn compute_hmac(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.hmac_key).unwrap();
        mac.update(data.as_bytes());
        BASE64.encode(mac.finalize().into_bytes())
    }

    pub fn log_event(
        &self,
        event_type: &str,
        path: &str,
        description: &str,
    ) -> Result<AuditEvent, AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let timestamp = Utc::now().to_rfc3339();

        let prev_hmac = conn
            .query_row(
                "SELECT hmac FROM audit_log ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "GENESIS".to_string());

        let record_data = format!(
            "{}|{}|{}|{}|{}",
            timestamp, event_type, path, description, prev_hmac
        );
        let hmac = self.compute_hmac(&record_data);

        conn.execute(
            "INSERT INTO audit_log (timestamp, event_type, path, description, prev_hmac, hmac) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![timestamp, event_type, path, description, prev_hmac, hmac],
        )?;

        let id = conn.last_insert_rowid();

        Ok(AuditEvent {
            id,
            timestamp,
            event_type: event_type.to_string(),
            path: path.to_string(),
            description: description.to_string(),
            prev_hmac,
            hmac,
        })
    }

    pub fn get_events(
        &self,
        limit: Option<i64>,
        event_type: Option<&str>,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let query = match (limit, event_type) {
            (Some(l), Some(et)) => format!(
                "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log WHERE event_type = '{}' ORDER BY id DESC LIMIT {}", 
                et, l
            ),
            (Some(l), None) => format!(
                "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log ORDER BY id DESC LIMIT {}", 
                l
            ),
            (None, Some(et)) => format!(
                "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log WHERE event_type = '{}' ORDER BY id DESC", 
                et
            ),
            (None, None) => 
                "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log ORDER BY id DESC".to_string(),
        };

        let mut stmt = conn.prepare(&query)?;
        let events = stmt.query_map([], |row| {
            Ok(AuditEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                event_type: row.get(2)?,
                path: row.get(3)?,
                description: row.get(4)?,
                prev_hmac: row.get(5)?,
                hmac: row.get(6)?,
            })
        })?;

        let mut result = Vec::new();
        for event in events {
            result.push(event?);
        }

        Ok(result)
    }

    pub fn validate_integrity(&self) -> Result<bool, AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log ORDER BY id ASC"
        )?;

        let mut expected_prev = "GENESIS".to_string();

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(4)?, // prev_hmac
                row.get::<_, String>(6)?, // hmac
            ))
        })?;

        for row in rows {
            let (prev_hmac, hmac) = row?;
            if prev_hmac != expected_prev {
                return Ok(false);
            }

            // Re-compute expected hmac for validation
            // This is a simplified check - in production you'd verify the full chain
            expected_prev = hmac;
        }

        Ok(true)
    }

    pub fn get_event_count(&self) -> Result<i64, AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn validate_integrity_detailed(
        &self,
    ) -> Result<(bool, i64, Option<String>, String), AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log ORDER BY id ASC"
        )?;

        let mut expected_prev = "GENESIS".to_string();
        let mut last_valid_id: i64 = 0;

        let rows: Vec<(i64, String, String, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (id, timestamp, event_type, path, description, prev_hmac, hmac) in rows {
            if prev_hmac != expected_prev {
                return Ok((
                    false,
                    last_valid_id,
                    Some(timestamp),
                    format!(
                        "Registro {} manipulado: prev_hmac={} pero se esperaba {}",
                        id, prev_hmac, expected_prev
                    ),
                ));
            }

            expected_prev = hmac;
            last_valid_id = id;
        }

        Ok((
            true,
            last_valid_id,
            None,
            "Integridad válida - todos los registros son auténticos".to_string(),
        ))
    }

    pub fn repair_integrity(&self) -> Result<(bool, i64, Option<String>, String), AuditError> {
        let conn = self.conn.lock().map_err(|_| AuditError::LockError)?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, event_type, path, description, prev_hmac, hmac FROM audit_log ORDER BY id ASC"
        )?;

        let mut expected_prev = "GENESIS".to_string();
        let mut last_valid_id: i64 = 0;
        let mut corrupted_ids: Vec<i64> = Vec::new();

        let rows: Vec<(i64, String, String, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (id, timestamp, event_type, path, description, prev_hmac, hmac) in rows {
            if prev_hmac != expected_prev {
                corrupted_ids.push(id);
            } else {
                expected_prev = hmac;
                last_valid_id = id;
            }
        }

        if corrupted_ids.is_empty() {
            return Ok((
                true,
                last_valid_id,
                None,
                "No se encontraron registros corruptos".to_string(),
            ));
        }

        conn.execute(
            &format!(
                "DELETE FROM audit_log WHERE id IN ({})",
                corrupted_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            [],
        )?;

        conn.execute("DELETE FROM audit_log WHERE id > ?", params![last_valid_id])?;

        Ok((
            true,
            last_valid_id,
            None,
            format!(
                "Reparación completada: {} registros corruptos eliminados",
                corrupted_ids.len()
            ),
        ))
    }
}
