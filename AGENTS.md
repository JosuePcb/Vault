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
| Email | `reqwest` + Resend API | Suspicious activity notifications |
| Audit log | `rusqlite` | SQLite-based structured logging with HMAC chain |
| Secrets storage | `dirs` | Local app data directory |

## Email Configuration

- **Provider**: Resend API
- **API Key**: User provides their own (stored in app state)
- **Usage**: Send alerts when suspicious file activity is detected

## Design Decisions

- **Audit log integrity**: HMAC-chained SQLite (each row contains `prev_hmac` referencing previous row hash) - validation fails if any record is modified/deleted
- **Encryption modes**: User-selectable (AES-256, ChaCha20)
- **Key storage**: Keys saved to `.key` files alongside encrypted files
- **Platform**: Cross-platform (Tauri)

## Implemented Tauri Commands

| Command | Description |
|---------|-------------|
| `generate_encryption_key` | Generate new encryption key |
| `set_encryption_key` | Set encryption key manually |
| `set_algorithm` | Set encryption algorithm (AES-256 or ChaCha20) |
| `get_algorithm` | Get current encryption algorithm |
| `encrypt_file_cmd` | Encrypt file with current key |
| `decrypt_file_cmd` | Decrypt file with provided key |
| `get_stats` | Get dashboard statistics |
| `get_audit_logs` | Get audit log events |
| `validate_audit_integrity` | Validate HMAC chain integrity |
| `start_watching` | Start file watcher (not implemented in UI) |
| `stop_watching` | Stop file watcher |
| `get_watched_paths` | Get watched paths |
| `configure_email` | Configure Resend API settings |
| `test_email` | Send test email |
| `is_email_configured` | Check if email is configured |

## Project Structure

```
/src-tauri/     # Rust backend
  /src/
    main.rs    # Entry point
    lib.rs     # Tauri commands and app state
    crypto.rs  # Encryption module
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

## Configuration

- **Algorithm**: Select in Settings (AES-256 default)
- **Email**: Configure in Settings with Resend API key
- **Key files**: Saved as `.key` files alongside encrypted files

## Known Issues

- File watcher not fully integrated with UI (commands exist but not connected)
- USB key integration not implemented (dependencies added but not coded)