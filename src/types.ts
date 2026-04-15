export interface CryptoStats {
  files_encrypted: number;
  dirs_watched: number;
  audit_events: number;
}

export interface AuditEvent {
  id: number;
  timestamp: string;
  event_type: string;
  path: string;
  description: string;
  prev_hmac: string;
  hmac: string;
}

export interface EncryptResult {
  success: boolean;
  output_path: string;
  key: string;
}

export interface DecryptResult {
  success: boolean;
  output_path: string;
}

export interface EmailConfig {
  api_key: string;
  from_email: string;
  from_name: string;
}

export interface DirEncryptResult {
  success: boolean;
  files_encrypted: string[];
  key: string;
}

export interface DirDecryptResult {
  success: boolean;
  files_decrypted: string[];
}