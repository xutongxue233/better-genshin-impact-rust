use base64::prelude::{Engine as _, BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

#[path = "notification_model_clients.rs"]
mod notification_model_clients;
pub use notification_model_clients::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NotificationEventDescriptor {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationEventResult {
    Success,
    Fail,
    PartialSuccess,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub event: String,
    pub result: NotificationEventResult,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub timestamp_ms: Option<u64>,
    pub has_screenshot: bool,
    #[serde(default)]
    pub screenshot: Option<NotificationImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub file_name: String,
}

impl NotificationPayload {
    pub fn success(event: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            result: NotificationEventResult::Success,
            message: Some(message.into()),
            data: None,
            timestamp_ms: None,
            has_screenshot: false,
            screenshot: None,
        }
    }

    pub fn fail(event: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            result: NotificationEventResult::Fail,
            message: Some(message.into()),
            data: None,
            timestamp_ms: None,
            has_screenshot: false,
            screenshot: None,
        }
    }

    pub fn with_screenshot(mut self, screenshot: NotificationImage) -> Self {
        self.has_screenshot = true;
        self.screenshot = Some(screenshot);
        self
    }
}

impl NotificationImage {
    pub fn new(
        bytes: impl Into<Vec<u8>>,
        mime_type: impl Into<String>,
        file_name: impl Into<String>,
    ) -> Self {
        Self {
            bytes: bytes.into(),
            mime_type: mime_type.into(),
            file_name: file_name.into(),
        }
    }

    pub fn png(bytes: impl Into<Vec<u8>>) -> Self {
        Self::new(bytes, "image/png", "image.png")
    }

    pub fn jpeg(bytes: impl Into<Vec<u8>>) -> Self {
        Self::new(bytes, "image/jpeg", "screenshot.jpg")
    }

    pub fn base64(&self) -> String {
        BASE64_STANDARD.encode(&self.bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationProviderKind {
    Webhook,
    WindowsUwp,
    Feishu,
    OneBot,
    WorkWeixin,
    WebSocket,
    Bark,
    Email,
    DingDingWebhook,
    Telegram,
    Xxtui,
    DiscordWebhook,
    ServerChan,
    Meow,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NotificationProviderPlan {
    pub kind: NotificationProviderKind,
    pub name: &'static str,
    pub target_summary: Option<String>,
    pub config_summary: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NotificationDispatchPlan {
    pub payload: NotificationPayload,
    pub should_send: bool,
    pub skipped_reason: Option<&'static str>,
    pub include_screenshot: bool,
    pub providers: Vec<NotificationProviderPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationHttpRequest {
    pub provider: NotificationProviderKind,
    pub method: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub body_kind: NotificationHttpBodyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationHttpBodyKind {
    Json,
    FormUrlEncoded,
    MultipartFormData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationHttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationEmailSecurity {
    None,
    StartTls,
    SslOnConnect,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationEmailAttachment {
    pub file_name: String,
    pub content_id: Option<String>,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationEmailRequest {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub security: NotificationEmailSecurity,
    pub from_email: String,
    pub from_name: String,
    pub to_email: String,
    pub subject: String,
    pub html_body: String,
    pub attachments: Vec<NotificationEmailAttachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationWindowsToastRequest {
    pub event: String,
    pub message: Option<String>,
    pub screenshot: Option<NotificationImage>,
    pub expiration_hours: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationProviderDelivery {
    pub provider: NotificationProviderKind,
    pub provider_name: &'static str,
    pub status: NotificationProviderDeliveryStatus,
    pub requests: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationProviderDeliveryStatus {
    Sent,
    Skipped,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationDispatchExecution {
    pub attempted: bool,
    pub skipped_reason: Option<&'static str>,
    pub deliveries: Vec<NotificationProviderDelivery>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NotificationDispatchError {
    #[error("{0}")]
    InvalidConfig(String),
    #[error("{0} notification does not support this payload yet")]
    UnsupportedPayload(&'static str),
    #[error("{0} notification dispatch is not ported yet")]
    UnsupportedProvider(&'static str),
    #[error("{provider} returned HTTP {status}: {body}")]
    HttpStatus {
        provider: &'static str,
        status: u16,
        body: String,
    },
    #[error("{provider} API returned failure: {message}")]
    ApiFailure {
        provider: &'static str,
        message: String,
    },
    #[error("{0}")]
    Transport(String),
}
