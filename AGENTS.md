# Vault - AGENTS.md

## Project Overview

Tauri desktop application for file encryption and activity tracking with audit logging.

## Tech Stack

- **Language**: Rust (backend/core) + JavaScript (frontend)
- **Framework**: Tauri (desktop app framework)
- **Frontend**: React + Tailwind CSS
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
| USB keys | `rusb` | Physical key integration |
| Email | `lettre` | Suspicious activity notifications |
| Audit log | `rusqlite` | SQLite-based structured logging |
| Secrets storage | `tauri-plugin-stronghold` | Protected key storage in memory |

## Design Decisions

- **Audit log integrity**: HMAC-chained SQLite (each row contains `prev_hmac` referencing previous row hash) - validation fails if any record is modified/deleted
- **Encryption modes**: User-selectable (AES-256, ChaCha20)
- **Platform**: Cross-platform (Tauri)

## Project Structure (Tauri Standard)

```
/src-tauri/     # Rust backend (Cargo.toml, src/main.rs, tauri.conf.json)
/src/           # React frontend (package.json, src/)
```

## Commands to Implement

Standard Tauri commands:
- `npm run tauri dev` - Development mode
- `npm run tauri build` - Production build