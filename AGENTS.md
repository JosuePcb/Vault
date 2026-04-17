# Vault - AGENTS.md

## Project Overview

Tauri desktop application for file encryption and directory containerization with activity tracking and tamper-evident audit logging.

## Tech Stack

- **Language**: Rust (backend/core) + TypeScript (frontend)
- **Framework**: Tauri 2.x (desktop app framework)
- **Frontend**: React + Tailwind CSS + TypeScript
- **Backend**: Tauri Commands (IPC), Tokio (async runtime)

## Architecture

```
┌─────────────────────────────────────────────────────┐
│           Renderer Process (React + TypeScript)     │
│   Pages: Dashboard, Encrypt, Decrypt, Settings      │
└─────────────────────┬───────────────────────────────┘
                      │ Tauri invoke()
┌─────────────────────┴───────────────────────────────┐
│              Main Process (Rust)                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────┐ │
│  │ crypto  │  │ audit   │  │ watcher │  │ email │ │
│  │ .rs     │  │ .rs     │  │ .rs     │  │ .rs   │ │
│  └─────────┘  └─────────┘  └─────────┘  └───────┘ │
│  ┌─────────┐                                        │
│  │  auth   │                                        │
│  │ .rs     │                                        │
│  └─────────┘                                        │
└─────────────────────────────────────────────────────┘
```

## Key Dependencies

| Purpose | Library | Notes |
|---------|---------|-------|
| Encryption | `aes-gcm`, `chacha20poly1305` | AES-256-GCM, ChaCha20-Poly1305 |
| ZIP | `zip` | Create archives for directory encryption |
| File walking | `walkdir` | Recursive directory traversal |
| File watching | `notify` | Monitor file/directory activity |
| USB keys | `rusb` | Physical key integration (not implemented) |
| Email | `reqwest` | HTTP client for Resend API |
| Audit log | `rusqlite` | SQLite with HMAC chain |
| Auth | `argon2` | Password hashing (Argon2id) |
| Base64 | `base64` | Key encoding/decoding |

## File Format

### Single File Encryption
```
Original:     documento.pdf
Encrypted:    documento.pdf.vault      (ciphertext + VAULT_FILE_END magic)
Metadata:     documento.pdf.vault-meta (key in Base64, original extension)
```

Format structure:
```
┌────────────────────────────┬──────────────────┐
│   Encrypted Data           │  Magic Footer    │
│   (ciphertext + nonce)     │  VAULT_FILE_END  │
└────────────────────────────┴──────────────────┘
```

### Directory Encryption
```
Original:     mi_carpeta/
               ├── archivo1.txt
               └── imagen.png
Encrypted:    mi_carpeta.vault         (encrypted ZIP)
Metadata:     mi_carpeta.vault-meta    (key, file list)
```

ZIP structure inside `.vault`:
```
┌─────────────────────────────────────────┐
│  entry: "archivo1.txt" (encrypted)      │
│  entry: "imagen.png" (encrypted)       │
└─────────────────────────────────────────┘
```

## Encryption Flow

### Encrypt File
1. User selects file via dialog
2. Backend generates key if not set
3. File read into memory
4. Data encrypted with AES-256-GCM or ChaCha20-Poly1305
5. Encrypted data written to `{file}.vault`
6. Metadata written to `{file}.vault-meta`
7. Original file deleted
8. Key displayed in UI for user to save

### Decrypt File
1. User selects `.vault` file
2. User provides key in Base64
3. Backend reads metadata from `{file}.vault-meta`
4. Backend decrypts data
5. Original extension restored from metadata
6. Decrypted file written with correct extension
7. `.vault` and `.vault-meta` files deleted

### Encrypt Directory
1. User selects directory via dialog
2. Backend generates key if not set
3. All files enumerated recursively
4. Each file encrypted individually
5. ZIP archive created with encrypted entries
6. ZIP encrypted as single blob
7. Output: `{dir}.vault` + `{dir}.vault-meta`
8. Original directory deleted

### Decrypt Directory
1. User selects `.vault` file
2. User provides key in Base64
3. ZIP decrypted and extracted to temp
4. Each entry decrypted individually
5. Directory structure restored
6. Original extension preserved per file

## Authentication System

- Optional password protection using Argon2id
- Stored hash in `%LOCALAPPDATA%\vault\auth.json`
- Required before using encryption features
- Commands: `check_auth_status`, `setup_password`, `login`

## Audit Log with HMAC Chain

Each record contains:
- `id`: Auto-increment primary key
- `timestamp`: RFC3339 timestamp
- `event_type`: encrypt, decrypt, encrypt_dir, decrypt_dir
- `path`: File/directory path affected
- `description`: Human-readable description
- `prev_hmac`: HMAC of previous record ("GENESIS" for first)
- `hmac`: HMAC(timestamp|event_type|path|description|prev_hmac)

Integrity validation:
1. Start with `expected_prev = "GENESIS"`
2. For each record: if `prev_hmac != expected_prev` → corrupted
3. Update `expected_prev = hmac`
4. Continue until end of chain

Auto-repair: Deletes corrupted records and all subsequent records.

## Config File

Location: `%LOCALAPPDATA%\vault\config.json`

```json
{
  "alert_email": "user@email.com",
  "algorithm": "AES-256",
  "password_hash": "argon2$...",
  "watched_paths": ["C:\\path\\to\\watch"]
}
```

## Implemented Tauri Commands

| Command | Description |
|---------|-------------|
| **Key Management** | |
| `generate_encryption_key` | Generate new key, store in state |
| `set_encryption_key` | Set key manually from Base64 |
| `set_algorithm` | Set algorithm (AES-256/ChaCha20) |
| `get_algorithm` | Get current algorithm |
| **File Encryption** | |
| `encrypt_file_cmd` | Encrypt file to .vault, return key |
| `decrypt_file_cmd` | Decrypt .vault file with key |
| **Directory Encryption** | |
| `encrypt_dir_cmd` | Encrypt directory to .vault container |
| `decrypt_dir_cmd` | Decrypt .vault container to directory |
| **Audit Log** | |
| `get_audit_logs` | Get events with optional filters |
| `validate_audit_integrity` | Validate HMAC chain |
| `repair_audit_integrity` | Auto-repair corrupted chain |
| `get_stats` | Get dashboard statistics |
| **File Watching** | |
| `start_watching` | Start monitoring path |
| `stop_watching` | Stop monitoring path |
| `get_watched_paths` | List watched paths |
| `get_watcher_events` | Get recent file events |
| **Email** | |
| `configure_email` | Set Resend API config |
| `test_email` | Send test email |
| `is_email_configured` | Check if email ready |
| `set_alert_email` | Set alert destination |
| `get_alert_email` | Get alert destination |
| `send_alert` | Send manual alert |
| **Authentication** | |
| `check_auth_status` | Check if password is set |
| `setup_password` | Set initial password |
| `login` | Verify password |

## Project Structure

```
/src-tauri/
  /src/
    main.rs        # Entry point, state initialization
    lib.rs         # All Tauri commands
    crypto.rs      # Encryption/decryption logic
    audit.rs       # HMAC-chained SQLite logging
    watcher.rs     # File system monitoring
    email.rs       # Resend API client
    auth.rs        # Argon2 password hashing
/src/              # React frontend
  App.tsx          # Main component with pages
  types.ts         # TypeScript interfaces
  main.tsx         # React entry point
  index.css        # Tailwind CSS
/public/           # Static assets
```

## How to Run

### Prerequisites
1. **Rust**: Install from https://rustup.rs
2. **Node.js**: Required for npm

### Development Mode
```bash
npm run tauri dev
```
Starts Vite dev server (localhost:1420) and Tauri app.

### Production Build
```bash
npm run tauri build
```
Output: `src-tauri/target/release/vault.exe`

## Environment Variables

- `RESEND_API_KEY`: API key for email notifications

## Known Issues

- File watcher commands exist but not connected to UI
- USB key integration planned but not implemented
