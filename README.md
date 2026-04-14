# Vault
## De que trata el proyecto?

Vault es un cifrador y tracker de archivos y directorios que permite mantener segura la informacion del usuario.
## Requisitos Funcionales

- Cifrar archivos y directorios con claves cifradas o con llaves fisicas (USB).

- Registrar actividad realizada en los directorios y sus archivos.

- Notificar al usuario cuando se detecte actividad sospechosa en los directorios y sus archivos mediante correo electronico.

- Permitir a los usuarios avanzados escoger el estándar de cifrado (por ejemplo: AES-256, ChaCha20, etc.).
## Requisitos no funcionales

- Multiplataforma

- El registro de actividad y auditoría debe estar protegido. Si un atacante entra, lo primero que hará será intentar borrar el log; este archivo debe estar cifrado o tener firmas de integridad.
## Stack Tecnológico
### Lenguaje

-Rust — Lógica del sistema, cifrado y backend.

-JavaScript — Lógica de la interfaz de usuario.
### Framework de escritorio

-Tauri — Alternativa ligera a Electron que utiliza el WebView nativo del sistema y un core en Rust.
### Frontend (Renderer Process)

- React — librería de UI

- Tailwind CSS — estilos
### Backend (Main Process)

-Tauri Commands — Comunicación segura entre el frontend y las funciones de Rust.

-Tokio — Runtime asíncrono para el manejo eficiente de hilos.
### Dependencias clave

| Librería         | Propósito                             | Notas                                                               |
| ---------------- | ------------------------------------- | ------------------------------------------------------------------- |
| `RustCrypto`    | Cifrado de alto nivel    | Usa los crates aes-gcm y chacha20poly1305. Son el estándar de la industria en Rust.                                |
| `notify`       | Watching de archivos y directorios    |Es la librería más madura para monitorear eventos del sistema de archivos de forma eficiente.        |
| `rusb`       | Integración con llaves físicas USB    | Un wrapper de libusb que te permitirá interactuar con las llaves físicas de forma nativa. |
| `lettre`     | Notificaciones por correo | Es una librería de correo electrónico robusta y bien mantenida. |
| `rusqlite` | Log de auditoría estructurado         | Es un wrapper de SQLite para Rust, muy rápido y seguro. |
|`tauri-plugin-stronghold`	|Almacenamiento de secretos	|Protege claves y llaves en memoria cifrada

---

## Diseño del Log de Auditoría
### SQLite + HMAC encadenado (Implementado en Rust)

Cada fila del log contiene una columna prev_hmac que referencia el hash de la fila anterior. Esta cadena de integridad asegura que si un registro es alterado o eliminado, la validación fallará inmediatamente.

- Persistencia: Se utiliza rusqlite para una gestión eficiente de los datos en el disco local.

- Integridad: El cálculo del HMAC se realiza en el proceso de Rust, fuera del alcance del proceso de renderizado, lo que añade una capa extra de seguridad.

- Consultas: Soporta filtros por fecha y tipo de evento directamente desde la interfaz mediante commands de Tauri.