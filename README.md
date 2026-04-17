# Vault

Aplicación de escritorio para cifrado de archivos y directorios con tracking de actividad y auditoría segura.

## Descripción

Vault permite cifrar archivos individuales o directorios completos, preservando la extensión original y generando contenedores `.vault` que solo pueden descifrarse con la clave correcta. Incluye un log de auditoría con cadena HMAC para detectar manipulaciones.

## Funcionalidades

- **Cifrado de archivos**: AES-256 o ChaCha20 con encriptación autenticada (AES-GCM, ChaCha20-Poly1305)
- **Cifrado de directorios**: Comprime y cifra directorios completos en un único archivo `.vault`
- **Preservación de extensiones**: La extensión original se guarda en los metadatos y se restaura al descifrar
- **Autenticación**: Sistema de password opcional con Argon2 para proteger el acceso a la app
- **Log de auditoría**: Registro inmutable con cadena HMAC que detecta cualquier modificación
- **Notificaciones por email**: Alertas automáticas via Resend API cuando se cifran/descifran archivos
- **File watching**: Monitoreo de directorios para detectar cambios (en desarrollo)

## Stack Tecnológico

| Componente | Tecnología |
|------------|------------|
| Framework | Tauri 2.x |
| Backend | Rust + Tokio |
| Frontend | React + TypeScript + Tailwind CSS |
| Cifrado | aes-gcm, chacha20poly1305 (RustCrypto) |
| Auditoría | rusqlite + HMAC-SHA256 |
| Email | reqwest + Resend API |

## Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                   Renderer Process (React)              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │Dashboard │ │  Cifrar  │ │ Descifrar│ │ Settings │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │
└───────┼────────────┼────────────┼────────────┼──────────┘
        │            │            │            │
        └────────────┴─────┬──────┴────────────┘
                            │ Tauri Commands (IPC)
┌───────────────────────────┴────────────────────────────────┐
│                     Main Process (Rust)                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│  │ crypto  │  │ audit   │  │ watcher │  │  email  │      │
│  │  .rs    │  │  .rs    │  │  .rs    │  │  .rs    │      │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Formato de Archivos

### Archivo cifrado (`.vault` + `.vault-meta`)

Cuando se cifra un archivo:
1. El archivo original se reemplaza por `{nombre}.vault` (datos cifrados + magic `VAULT_FILE_END`)
2. Se crea `{nombre}.vault-meta` con la clave y metadatos
3. El archivo original se elimina

```
archivo.pdf → archivo.pdf.vault      (datos cifrados + magic)
            → archivo.pdf.vault-meta (clave en Base64 + extensión original)
```

### Directorio cifrado (`.vault` + `.vault-meta`)

Cuando se cifra un directorio:
1. Se crea un ZIP con todos los archivos (cada uno cifrado individualmente)
2. El ZIP se cifra y guarda como `{directorio}.vault`
3. Se crea `{directorio}.vault-meta` con la clave y lista de archivos
4. Los archivos originales se eliminan

```
mi_carpeta/ → mi_carpeta.vault       (ZIP cifrado)
            → mi_carpeta.vault-meta  (clave + metadatos)
```

## Sistema de Auditoría

El log de auditoría usa una cadena HMAC para garantizar integridad:

```
┌────┬─────────────────┬──────────┬─────────────────────────────────┐
│ ID │   Timestamp      │ Tipo     │ HMAC Chain                      │
├────┼─────────────────┼──────────┼─────────────────────────────────┤
│  1 │ 2024-01-01T...   │ encrypt  │ prev=GENESIS, hmac=H(1)         │
│  2 │ 2024-01-01T...   │ decrypt  │ prev=H(1), hmac=H(2)            │
│  3 │ 2024-01-01T...   │ encrypt  │ prev=H(2), hmac=H(3)            │
└────┴─────────────────┴──────────┴─────────────────────────────────┘
```

- Cada registro incluye `prev_hmac` (HMAC del registro anterior)
- Si alguien modifica un registro, la cadena se rompe
- Validación: `validate_audit_integrity` detecta manipulación
- Reparación: `repair_audit_integrity` elimina registros corruptos

## Configuración

Ubicación: `%LOCALAPPDATA%\vault\config.json`

```json
{
  "alert_email": "user@example.com",
  "algorithm": "AES-256"
}
```

Autenticación: `%LOCALAPPDATA%\vault\auth.json`

Base de datos: `%LOCALAPPDATA%\vault\audit.db`

## Ejecución

```bash
# Desarrollo
npm run tauri dev

# Producción
npm run tauri build
```

## Requisitos

- Rust (https://rustup.rs)
- Node.js
- Windows 10/11 (desarrollo actual)
