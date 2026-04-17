# Vault - AGENTS.md

## Project Overview

Tauri desktop application for file encryption and activity tracking with audit logging.

## Tech Stack

- **Language**: Rust (backend/core) + JavaScript/TypeScript (frontend)
- **Framework**: Tauri 2.x (desktop app framework)
- **Frontend**: React + Tailwind CSS + TypeScript
- **Backend**: Tauri Commands (IPC), Tokio (async runtime)

## Architecture

- **Main Process (Rust)**: Encryption, file watching, USB key handling, email notifications, audit logging
- **Renderer Process (React)**: UI components, user interactions
- **IPC**: Tauri Commands bridge frontend ↔ backend

## Key Dependencies

| Purpose | Library | Notes |
|---------|---------|-------|
| Encryption | `aes-gcm`, `chacha20poly1305` (RustCrypto) | AES-256, ChaCha20 support |
| File watching | `notify` | Monitor file/directory activity |
| USB keys | `rusb` | Physical key integration (not implemented yet) |
| Email | `reqwest` + Resend API | Alerts on encryption/decryption events |
| Audit log | `rusqlite` | SQLite-based structured logging with HMAC chain |
| Secrets storage | `dirs` | Local app data directory |

## Email Configuration

- **Provider**: Resend API
- **From Email**: `noreply@resend.dev`
- **Usage**: Send alerts when file is encrypted/decrypted (auto-sent)

## Config File

Location: `%LOCALAPPDATA%\vault\config.json`

```json
{
  "alert_email": "user@email.com",
  "algorithm": "AES-256"
}
```

Automatically loaded on app start, persisted when changed.

## Design Decisions

- **In-place encryption**: File is replaced with encrypted data, metadata saved in `.vault-meta` file
- **Extension preservation**: Original file extension saved in metadata, restored on decrypt
- **Key storage**: User must save the key - displayed in UI after encryption (copy button)
- **Audit log integrity**: HMAC-chained SQLite - validation fails if any record is modified/deleted
- **Auto-repair**: Compromised audit log can be automatically repaired
- **Encryption modes**: User-selectable (AES-256, ChaCha20)
- **Platform**: Cross-platform (Tauri)

## Implemented Tauri Commands

| Command | Description |
|---------|-------------|
| `generate_encryption_key` | Generate new encryption key |
| `set_encryption_key` | Set encryption key manually |
| `set_algorithm` | Set encryption algorithm (AES-256 or ChaCha20) |
| `get_algorithm` | Get current encryption algorithm |
| `encrypt_file_cmd` | Encrypt file in-place (same file) |
| `decrypt_file_cmd` | Decrypt file in-place with extension restore |
| `get_stats` | Get dashboard statistics |
| `get_audit_logs` | Get audit log events |
| `validate_audit_integrity` | Validate HMAC chain integrity (detailed result) |
| `repair_audit_integrity` | Auto-repair compromised audit log |
| `start_watching` | Start file watcher (not implemented in UI) |
| `stop_watching` | Stop file watcher |
| `get_watched_paths` | Get watched paths |
| `configure_email` | Configure Resend API settings |
| `test_email` | Send test email |
| `is_email_configured` | Check if email is configured |
| `set_alert_email` | Set alert destination email |
| `get_alert_email` | Get alert destination email |
| `send_alert` | Send manual alert |

| Directory Encryption | Encrypt directory to single .vault container file |
| `decrypt_dir_cmd` | Decrypt .vault container back to original directory |
| `encrypt_dir_container` | Encrypt directory contents to in-memory zip, then encrypt (internal) |
| `decrypt_dir_container` | Decrypt .vault file, extract zip, restore directory (internal) |

## File Encryption Flow

### Encrypt (In-Place)
1. User selects file via dialog
2. Backend reads file content
3. Backend encrypts content in memory
4. Backend writes encrypted data to SAME file (overwrites original)
5. Backend creates `{filename}.vault-meta` file with:
   - `original_path`: Original file path
   - `algorithm`: Encryption algorithm used
   - `original_extension`: Original file extension (e.g., `.png`, `.pdf`)
   - `key`: Encryption key in Base64
6. Key displayed in UI for user to copy/save

### Decrypt (In-Place)
1. User selects `.vault-meta` file via dialog
2. Backend reads metadata from file
3. Backend decrypts encrypted file content
4. Backend restores original extension
5. Metadata file deleted after successful decrypt

## Project Structure

```
/src-tauri/     # Rust backend
  /src/
    main.rs    # Entry point
    lib.rs     # Tauri commands and app state
    crypto.rs  # Encryption module (in-place, metadata)
    audit.rs   # Audit log with HMAC chain
    watcher.rs # File watcher (not fully integrated)
    email.rs   # Email client (Resend API)
/src/           # React frontend
  App.tsx       # Main component with all pages
  types.ts      # TypeScript interfaces
  main.tsx      # React entry point
  index.css     # Tailwind CSS
/public/        # Static assets
```

## How to Run

### Prerequisites

1. **Rust**: Install from https://rustup.rs
2. **Node.js**: Required for npm
3. **Add Rust to PATH**:
   ```powershell
   $env:Path = "C:\Users\<USER>\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin;" + $env:Path
   ```

### Development Mode

```bash
cd I:\coding\projects\Vault
npm run tauri dev
```

This will:
1. Start the Vite dev server (localhost:1420)
2. Build and run the Tauri application
3. Connect the app to the frontend

### Production Build

```bash
npm run tauri build
```

Output: `src-tauri/target/release/vault.exe`

## Known Issues

- File watcher not fully integrated with UI (commands exist but not connected)
- USB key integration not implemented (dependencies added but not coded)