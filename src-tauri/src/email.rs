use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Not configured")]
    NotConfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
}

#[derive(Debug, Serialize)]
struct ResendRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    text: Option<String>,
    html: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResendResponse {
    id: Option<String>,
    #[serde(default)]
    error: Option<ResendError>,
}

#[derive(Debug, Deserialize)]
struct ResendError {
    message: Option<String>,
}

pub struct EmailClient {
    config: Option<EmailConfig>,
    client: Client,
}

impl EmailClient {
    pub fn new() -> Self {
        EmailClient {
            config: None,
            client: Client::new(),
        }
    }
    
    pub fn configure(&mut self, config: EmailConfig) {
        self.config = Some(config);
    }
    
    pub fn is_configured(&self) -> bool {
        self.config.is_some()
    }
    
    pub fn get_config(&self) -> Option<&EmailConfig> {
        self.config.as_ref()
    }
    
    pub async fn send(&self, to: &[String], subject: &str, body: &str, html: Option<&str>) -> Result<String, EmailError> {
        let config = self.config.as_ref().ok_or(EmailError::NotConfigured)?;
        
        let from = if config.from_name.is_empty() {
            config.from_email.clone()
        } else {
            format!("{} <{}>", config.from_name, config.from_email)
        };
        
        let request = ResendRequest {
            from,
            to: to.to_vec(),
            subject: subject.to_string(),
            text: Some(body.to_string()),
            html: html.map(|s| s.to_string()),
        };
        
        let response = self.client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        let result: ResendResponse = response.json().await?;
        
        if let Some(id) = result.id {
            Ok(id)
        } else if let Some(error) = result.error {
            Err(EmailError::ApiError(error.message.unwrap_or_default()))
        } else {
            Err(EmailError::ApiError("Unknown error".to_string()))
        }
    }
    
    pub async fn send_alert(&self, to: &[String], path: &str, event_type: &str, description: &str) -> Result<String, EmailError> {
        let subject = format!("[Vault] Alerta: {} en {}", event_type, path);
        let body = format!(
            "Se ha detectado actividad sospechosa en Vault:\n\n\
            Tipo de evento: {}\n\
            Ruta: {}\n\
            Descripción: {}\n\
            Hora: {}",
            event_type,
            path,
            description,
            chrono::Utc::now().to_rfc3339()
        );
        
        self.send(to, &subject, &body, None).await
    }
}

impl Default for EmailClient {
    fn default() -> Self {
        Self::new()
    }
}
