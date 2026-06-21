use crate::config::NotificationConfig;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use chrono::{DateTime, Local, TimeZone};
use hmac::{Hmac, Mac};
use image::codecs::jpeg::JpegEncoder;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::io::Cursor;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
const FEISHU_ACCESS_TOKEN_URL: &str =
    "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
const FEISHU_UPLOAD_IMAGE_URL: &str = "https://open.feishu.cn/open-apis/im/v1/images";

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

pub trait NotificationHttpClient {
    fn send(
        &mut self,
        request: &NotificationHttpRequest,
    ) -> std::result::Result<NotificationHttpResponse, NotificationDispatchError>;
}

pub trait NotificationWebSocketClient {
    fn send_text(
        &mut self,
        endpoint: &str,
        text: &str,
    ) -> std::result::Result<(), NotificationDispatchError>;
}

pub trait NotificationEmailClient {
    fn send_email(
        &mut self,
        request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError>;
}

pub trait NotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError>;
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

#[derive(Debug, Default)]
pub struct RecordingNotificationHttpClient {
    pub requests: Vec<NotificationHttpRequest>,
    pub responses: Vec<NotificationHttpResponse>,
}

impl RecordingNotificationHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_response(status: u16, body: impl Into<String>) -> Self {
        Self {
            requests: Vec::new(),
            responses: vec![NotificationHttpResponse {
                status,
                body: body.into(),
            }],
        }
    }

    pub fn with_responses(responses: Vec<NotificationHttpResponse>) -> Self {
        Self {
            requests: Vec::new(),
            responses,
        }
    }
}

impl NotificationHttpClient for RecordingNotificationHttpClient {
    fn send(
        &mut self,
        request: &NotificationHttpRequest,
    ) -> std::result::Result<NotificationHttpResponse, NotificationDispatchError> {
        self.requests.push(request.clone());
        if self.responses.is_empty() {
            return Ok(NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            });
        }
        Ok(self.responses.remove(0))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationWebSocketClient {
    pub messages: Vec<(String, String)>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationWebSocketClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationWebSocketClient for RecordingNotificationWebSocketClient {
    fn send_text(
        &mut self,
        endpoint: &str,
        text: &str,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.messages.push((endpoint.to_string(), text.to_string()));
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationWebSocketClient;

impl NotificationWebSocketClient for UnsupportedNotificationWebSocketClient {
    fn send_text(
        &mut self,
        _endpoint: &str,
        _text: &str,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider("WebSocket"))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationEmailClient {
    pub emails: Vec<NotificationEmailRequest>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationEmailClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationEmailClient for RecordingNotificationEmailClient {
    fn send_email(
        &mut self,
        request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.emails.push(request.clone());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationEmailClient;

impl NotificationEmailClient for UnsupportedNotificationEmailClient {
    fn send_email(
        &mut self,
        _request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider("Email"))
    }
}

#[derive(Debug, Default)]
pub struct RecordingNotificationWindowsToastClient {
    pub toasts: Vec<NotificationWindowsToastRequest>,
    pub fail_with: Option<NotificationDispatchError>,
}

impl RecordingNotificationWindowsToastClient {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationWindowsToastClient for RecordingNotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        if let Some(error) = self.fail_with.clone() {
            return Err(error);
        }
        self.toasts.push(request.clone());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UnsupportedNotificationWindowsToastClient;

impl NotificationWindowsToastClient for UnsupportedNotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        _request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        Err(NotificationDispatchError::UnsupportedProvider(
            "Windows UWP",
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NotificationMultipartFile {
    field_name: String,
    file_name: String,
    mime_type: String,
    bytes: Vec<u8>,
}

impl NotificationMultipartFile {
    fn new(field_name: impl Into<String>, image: NotificationImage) -> Self {
        Self {
            field_name: field_name.into(),
            file_name: image.file_name,
            mime_type: image.mime_type,
            bytes: image.bytes,
        }
    }

    fn with_file_name(
        field_name: impl Into<String>,
        image: NotificationImage,
        file_name: impl Into<String>,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            file_name: file_name.into(),
            mime_type: image.mime_type,
            bytes: image.bytes,
        }
    }
}

pub fn notification_events() -> Vec<NotificationEventDescriptor> {
    NOTIFICATION_EVENTS.to_vec()
}

pub fn parse_notification_event_codes(subscribe_event_str: Option<&str>) -> Vec<String> {
    let Some(subscribe_event_str) = subscribe_event_str else {
        return Vec::new();
    };
    let mut event_codes = Vec::new();
    for event_code in subscribe_event_str.split(',') {
        let trimmed = event_code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if event_codes
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        event_codes.push(trimmed.to_string());
    }
    event_codes
}

pub fn normalize_notification_event_codes<'a>(
    event_codes: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut normalized = Vec::new();
    for event_code in event_codes {
        let trimmed = event_code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized.join(",")
}

pub fn should_send_notification(
    subscribe_event_str: Option<&str>,
    event_code: Option<&str>,
) -> bool {
    let Some(subscribe_event_str) = subscribe_event_str else {
        return true;
    };
    if subscribe_event_str.trim().is_empty() {
        return true;
    }

    let Some(event_code) = event_code else {
        return false;
    };
    if event_code.trim().is_empty() {
        return false;
    }

    parse_notification_event_codes(Some(subscribe_event_str))
        .iter()
        .any(|subscribed| subscribed.eq_ignore_ascii_case(event_code))
}

pub fn notification_provider_plans(config: &NotificationConfig) -> Vec<NotificationProviderPlan> {
    let mut plans = Vec::new();

    if config.webhook_enabled {
        plans.push(provider(
            NotificationProviderKind::Webhook,
            "Webhook",
            non_empty(&config.webhook_endpoint).or_else(|| non_empty(&config.webhook_send_to)),
            json!({
                "endpoint_configured": is_configured(&config.webhook_endpoint),
                "send_to_configured": is_configured(&config.webhook_send_to)
            }),
        ));
    }
    if config.windows_uwp_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::WindowsUwp,
            "Windows UWP",
            None,
            json!({}),
        ));
    }
    if config.feishu_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Feishu,
            "Feishu",
            non_empty(&config.feishu_webhook_url),
            json!({
                "webhook_configured": is_configured(&config.feishu_webhook_url),
                "app_id_configured": is_configured(&config.feishu_app_id),
                "app_secret_configured": is_configured(&config.feishu_app_secret)
            }),
        ));
    }
    if config.one_bot_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::OneBot,
            "OneBot",
            non_empty(&config.one_bot_endpoint),
            json!({
                "endpoint_configured": is_configured(&config.one_bot_endpoint),
                "user_id_configured": is_configured(&config.one_bot_user_id),
                "group_id_configured": is_configured(&config.one_bot_group_id),
                "token_configured": is_configured(&config.one_bot_token)
            }),
        ));
    }
    if config.workweixin_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::WorkWeixin,
            "Work Weixin",
            non_empty(&config.workweixin_webhook_url),
            json!({
                "webhook_configured": is_configured(&config.workweixin_webhook_url)
            }),
        ));
    }
    if config.web_socket_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::WebSocket,
            "WebSocket",
            non_empty(&config.web_socket_endpoint),
            json!({
                "endpoint_configured": is_configured(&config.web_socket_endpoint),
                "naming_policy": "snake_case"
            }),
        ));
    }
    if config.bark_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Bark,
            "Bark",
            non_empty(&config.bark_api_endpoint),
            json!({
                "device_keys_configured": is_configured(&config.bark_device_keys),
                "api_endpoint_configured": is_configured(&config.bark_api_endpoint),
                "group": config.bark_group,
                "sound": config.bark_sound,
                "icon_configured": is_configured(&config.bark_icon),
                "level": config.bark_level,
                "badge": config.bark_badge,
                "volume": config.bark_volume
            }),
        ));
    }
    if config.email_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Email,
            "Email",
            non_empty(&config.to_email),
            json!({
                "smtp_server_configured": is_configured(&config.smtp_server),
                "smtp_port": config.smtp_port,
                "username_configured": is_configured(&config.smtp_username),
                "password_configured": is_configured(&config.smtp_password),
                "from_email_configured": is_configured(&config.from_email),
                "from_name_configured": is_configured(&config.from_name),
                "to_email_configured": is_configured(&config.to_email)
            }),
        ));
    }
    if config.ding_dingwebhook_notification_enabled
        && is_configured(&config.dingding_webhook_url)
        && is_configured(&config.ding_ding_secret)
    {
        plans.push(provider(
            NotificationProviderKind::DingDingWebhook,
            "DingDing",
            non_empty(&config.dingding_webhook_url),
            json!({
                "webhook_configured": true,
                "secret_configured": true
            }),
        ));
    }
    if config.telegram_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Telegram,
            "Telegram",
            non_empty(&config.telegram_chat_id),
            json!({
                "bot_token_configured": is_configured(&config.telegram_bot_token),
                "chat_id_configured": is_configured(&config.telegram_chat_id),
                "api_base_url_configured": is_configured(&config.telegram_api_base_url),
                "proxy_url_configured": is_configured(&config.telegram_proxy_url),
                "proxy_enabled": config.telegram_proxy_enabled
            }),
        ));
    }
    if config.xxtui_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Xxtui,
            "Xxtui",
            non_empty(&config.xxtui_from),
            json!({
                "api_key_configured": is_configured(&config.xxtui_api_key),
                "from": config.xxtui_from,
                "channels": parse_xxtui_channels(&config.xxtui_channels)
            }),
        ));
    }
    if config.discord_webhook_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::DiscordWebhook,
            "Discord Webhook",
            non_empty(&config.discord_webhook_url),
            json!({
                "webhook_configured": is_configured(&config.discord_webhook_url),
                "username_configured": is_configured(&config.discord_webhook_username),
                "avatar_url_configured": is_configured(&config.discord_webhook_avatar_url),
                "image_encoder": config.discord_webhook_image_encoder
            }),
        ));
    }
    if config.server_chan_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::ServerChan,
            "ServerChan",
            None,
            json!({
                "send_key_configured": is_configured(&config.server_chan_send_key)
            }),
        ));
    }
    if config.meow_notification_enabled {
        plans.push(provider(
            NotificationProviderKind::Meow,
            "Meow",
            non_empty(&config.meow_title).or_else(|| non_empty(&config.meow_nickname)),
            json!({
                "nickname_configured": is_configured(&config.meow_nickname),
                "title_configured": is_configured(&config.meow_title)
            }),
        ));
    }

    plans
}

pub fn notification_dispatch_plan(
    config: &NotificationConfig,
    payload: NotificationPayload,
) -> NotificationDispatchPlan {
    let should_send = should_send_notification(
        Some(&config.notification_event_subscribe),
        Some(&payload.event),
    );
    let providers = if should_send {
        notification_provider_plans(config)
    } else {
        Vec::new()
    };
    let skipped_reason = if should_send {
        None
    } else {
        Some("event_not_subscribed")
    };

    NotificationDispatchPlan {
        payload,
        should_send,
        skipped_reason,
        include_screenshot: config.include_screen_shot,
        providers,
    }
}

pub fn notification_dispatch_plan_for_provider(
    config: &NotificationConfig,
    payload: NotificationPayload,
    provider_kind: NotificationProviderKind,
) -> NotificationDispatchPlan {
    let mut plan = notification_dispatch_plan(config, payload);
    if plan.should_send {
        plan.providers
            .retain(|provider| provider.kind == provider_kind);
    }
    plan
}

pub fn execute_notification_dispatch<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: NotificationPayload,
    client: &mut C,
) -> NotificationDispatchExecution {
    let plan = notification_dispatch_plan(config, payload);
    let mut web_socket_client = UnsupportedNotificationWebSocketClient;
    let mut email_client = UnsupportedNotificationEmailClient;
    let mut windows_toast_client = UnsupportedNotificationWindowsToastClient;
    execute_notification_dispatch_plan(
        config,
        &plan,
        client,
        &mut web_socket_client,
        &mut email_client,
        &mut windows_toast_client,
    )
}

pub fn execute_notification_dispatch_with_websocket<C, W>(
    config: &NotificationConfig,
    payload: NotificationPayload,
    http_client: &mut C,
    web_socket_client: &mut W,
) -> NotificationDispatchExecution
where
    C: NotificationHttpClient,
    W: NotificationWebSocketClient,
{
    let plan = notification_dispatch_plan(config, payload);
    let mut email_client = UnsupportedNotificationEmailClient;
    let mut windows_toast_client = UnsupportedNotificationWindowsToastClient;
    execute_notification_dispatch_plan(
        config,
        &plan,
        http_client,
        web_socket_client,
        &mut email_client,
        &mut windows_toast_client,
    )
}

pub fn execute_notification_dispatch_with_transports<C, W, E, T>(
    config: &NotificationConfig,
    payload: NotificationPayload,
    http_client: &mut C,
    web_socket_client: &mut W,
    email_client: &mut E,
    windows_toast_client: &mut T,
) -> NotificationDispatchExecution
where
    C: NotificationHttpClient,
    W: NotificationWebSocketClient,
    E: NotificationEmailClient,
    T: NotificationWindowsToastClient,
{
    let plan = notification_dispatch_plan(config, payload);
    execute_notification_dispatch_plan(
        config,
        &plan,
        http_client,
        web_socket_client,
        email_client,
        windows_toast_client,
    )
}

pub fn execute_notification_dispatch_plan<C, W, E, T>(
    config: &NotificationConfig,
    plan: &NotificationDispatchPlan,
    http_client: &mut C,
    web_socket_client: &mut W,
    email_client: &mut E,
    windows_toast_client: &mut T,
) -> NotificationDispatchExecution
where
    C: NotificationHttpClient,
    W: NotificationWebSocketClient,
    E: NotificationEmailClient,
    T: NotificationWindowsToastClient,
{
    if !plan.should_send {
        return NotificationDispatchExecution {
            attempted: false,
            skipped_reason: plan.skipped_reason,
            deliveries: Vec::new(),
        };
    }

    let deliveries = plan
        .providers
        .iter()
        .map(|provider| {
            execute_notification_provider(
                config,
                &plan.payload,
                provider,
                http_client,
                web_socket_client,
                email_client,
                windows_toast_client,
            )
        })
        .collect::<Vec<_>>();

    NotificationDispatchExecution {
        attempted: true,
        skipped_reason: None,
        deliveries,
    }
}

pub fn notification_http_requests_for_provider(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    match provider.kind {
        NotificationProviderKind::Webhook => webhook_requests(config, payload),
        NotificationProviderKind::Feishu => feishu_requests(config, payload),
        NotificationProviderKind::OneBot => one_bot_requests(config, payload),
        NotificationProviderKind::WorkWeixin => work_weixin_requests(config, payload),
        NotificationProviderKind::Bark => bark_requests(config, payload),
        NotificationProviderKind::DingDingWebhook => dingding_requests(config, payload),
        NotificationProviderKind::Telegram => telegram_requests(config, payload),
        NotificationProviderKind::Xxtui => xxtui_requests(config, payload),
        NotificationProviderKind::DiscordWebhook => discord_requests(config, payload),
        NotificationProviderKind::ServerChan => server_chan_requests(config, payload),
        NotificationProviderKind::Meow => meow_requests(config, payload),
        NotificationProviderKind::WindowsUwp => Ok(Vec::new()),
        NotificationProviderKind::WebSocket => Ok(Vec::new()),
        NotificationProviderKind::Email => Ok(Vec::new()),
    }
}

pub fn parse_xxtui_channels(channels: &str) -> Vec<String> {
    if channels.trim().is_empty() {
        return vec!["WX_MP".to_string()];
    }

    let mut parsed = Vec::new();
    for channel in channels.split(',') {
        let trimmed = channel.trim();
        if matches!(trimmed, "WX_MP" | "DING_TALK" | "EMAIL" | "SMS" | "WEBHOOK") {
            parsed.push(trimmed.to_string());
        }
    }

    if parsed.is_empty() {
        vec!["WX_MP".to_string()]
    } else {
        parsed
    }
}

fn provider(
    kind: NotificationProviderKind,
    name: &'static str,
    target_summary: Option<String>,
    config_summary: Value,
) -> NotificationProviderPlan {
    NotificationProviderPlan {
        kind,
        name,
        target_summary,
        config_summary,
    }
}

fn is_configured(value: &str) -> bool {
    !value.trim().is_empty()
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn execute_notification_provider<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    http_client: &mut C,
    web_socket_client: &mut impl NotificationWebSocketClient,
    email_client: &mut impl NotificationEmailClient,
    windows_toast_client: &mut impl NotificationWindowsToastClient,
) -> NotificationProviderDelivery {
    if provider.kind == NotificationProviderKind::Feishu
        && payload.screenshot.is_some()
        && !config.feishu_app_id.trim().is_empty()
        && !config.feishu_app_secret.trim().is_empty()
    {
        return execute_feishu_image_provider(config, payload, provider, http_client);
    }
    if provider.kind == NotificationProviderKind::WebSocket {
        return execute_web_socket_provider(config, payload, provider, web_socket_client);
    }
    if provider.kind == NotificationProviderKind::Email {
        return execute_email_provider(config, payload, provider, email_client);
    }
    if provider.kind == NotificationProviderKind::WindowsUwp {
        return execute_windows_toast_provider(payload, provider, windows_toast_client);
    }

    let requests = match notification_http_requests_for_provider(config, payload, provider) {
        Ok(requests) if requests.is_empty() => {
            return NotificationProviderDelivery {
                provider: provider.kind,
                provider_name: provider.name,
                status: NotificationProviderDeliveryStatus::Skipped,
                requests: 0,
                message: Some("no request generated".to_string()),
            };
        }
        Ok(requests) => requests,
        Err(error) => {
            let status = match error {
                NotificationDispatchError::UnsupportedPayload(_)
                | NotificationDispatchError::UnsupportedProvider(_) => {
                    NotificationProviderDeliveryStatus::Unsupported
                }
                _ => NotificationProviderDeliveryStatus::Failed,
            };
            return NotificationProviderDelivery {
                provider: provider.kind,
                provider_name: provider.name,
                status,
                requests: 0,
                message: Some(error.to_string()),
            };
        }
    };

    let request_count = requests.len();
    for request in &requests {
        match http_client.send(request).and_then(|response| {
            validate_notification_response(provider.kind, provider.name, &response)
        }) {
            Ok(()) => {}
            Err(error) => {
                return NotificationProviderDelivery {
                    provider: provider.kind,
                    provider_name: provider.name,
                    status: NotificationProviderDeliveryStatus::Failed,
                    requests: request_count,
                    message: Some(error.to_string()),
                };
            }
        }
    }

    NotificationProviderDelivery {
        provider: provider.kind,
        provider_name: provider.name,
        status: NotificationProviderDeliveryStatus::Sent,
        requests: request_count,
        message: None,
    }
}

fn execute_email_provider<E: NotificationEmailClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    client: &mut E,
) -> NotificationProviderDelivery {
    match email_request(config, payload).and_then(|request| client.send_email(&request)) {
        Ok(()) => NotificationProviderDelivery {
            provider: provider.kind,
            provider_name: provider.name,
            status: NotificationProviderDeliveryStatus::Sent,
            requests: 1,
            message: None,
        },
        Err(error) => {
            let status = match error {
                NotificationDispatchError::UnsupportedProvider(_) => {
                    NotificationProviderDeliveryStatus::Unsupported
                }
                _ => NotificationProviderDeliveryStatus::Failed,
            };
            NotificationProviderDelivery {
                provider: provider.kind,
                provider_name: provider.name,
                status,
                requests: 0,
                message: Some(error.to_string()),
            }
        }
    }
}

fn execute_windows_toast_provider<T: NotificationWindowsToastClient>(
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    client: &mut T,
) -> NotificationProviderDelivery {
    match windows_toast_request(payload).and_then(|request| client.show_toast(&request)) {
        Ok(()) => NotificationProviderDelivery {
            provider: provider.kind,
            provider_name: provider.name,
            status: NotificationProviderDeliveryStatus::Sent,
            requests: 1,
            message: None,
        },
        Err(error) => {
            let status = match error {
                NotificationDispatchError::UnsupportedProvider(_) => {
                    NotificationProviderDeliveryStatus::Unsupported
                }
                _ => NotificationProviderDeliveryStatus::Failed,
            };
            NotificationProviderDelivery {
                provider: provider.kind,
                provider_name: provider.name,
                status,
                requests: 0,
                message: Some(error.to_string()),
            }
        }
    }
}

fn windows_toast_request(
    payload: &NotificationPayload,
) -> std::result::Result<NotificationWindowsToastRequest, NotificationDispatchError> {
    Ok(NotificationWindowsToastRequest {
        event: payload.event.clone(),
        message: payload.message.clone(),
        screenshot: payload.screenshot.clone(),
        expiration_hours: 12,
    })
}

fn execute_web_socket_provider<W: NotificationWebSocketClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    client: &mut W,
) -> NotificationProviderDelivery {
    match web_socket_payload(config, payload)
        .and_then(|(endpoint, message)| client.send_text(&endpoint, &message))
    {
        Ok(()) => NotificationProviderDelivery {
            provider: provider.kind,
            provider_name: provider.name,
            status: NotificationProviderDeliveryStatus::Sent,
            requests: 1,
            message: None,
        },
        Err(error) => {
            let status = match error {
                NotificationDispatchError::UnsupportedProvider(_) => {
                    NotificationProviderDeliveryStatus::Unsupported
                }
                _ => NotificationProviderDeliveryStatus::Failed,
            };
            NotificationProviderDelivery {
                provider: provider.kind,
                provider_name: provider.name,
                status,
                requests: 0,
                message: Some(error.to_string()),
            }
        }
    }
}

fn execute_feishu_image_provider<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    client: &mut C,
) -> NotificationProviderDelivery {
    let result = feishu_image_dispatch(config, payload, client);
    match result {
        Ok(requests) => NotificationProviderDelivery {
            provider: provider.kind,
            provider_name: provider.name,
            status: NotificationProviderDeliveryStatus::Sent,
            requests,
            message: None,
        },
        Err(error) => NotificationProviderDelivery {
            provider: provider.kind,
            provider_name: provider.name,
            status: NotificationProviderDeliveryStatus::Failed,
            requests: 0,
            message: Some(error.to_string()),
        },
    }
}

fn validate_notification_response(
    kind: NotificationProviderKind,
    provider_name: &'static str,
    response: &NotificationHttpResponse,
) -> std::result::Result<(), NotificationDispatchError> {
    if !(200..=299).contains(&response.status) {
        return Err(NotificationDispatchError::HttpStatus {
            provider: provider_name,
            status: response.status,
            body: response.body.clone(),
        });
    }

    match kind {
        NotificationProviderKind::OneBot => {
            let value = parse_optional_json(&response.body)?;
            if let Some(status) = value
                .as_ref()
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str)
            {
                if status.eq_ignore_ascii_case("ok") {
                    return Ok(());
                }
                return Err(NotificationDispatchError::ApiFailure {
                    provider: provider_name,
                    message: format!("status={status}"),
                });
            }
            Err(NotificationDispatchError::ApiFailure {
                provider: provider_name,
                message: "missing status".to_string(),
            })
        }
        NotificationProviderKind::Telegram => {
            let Some(value) = parse_optional_json(&response.body)? else {
                return Ok(());
            };
            if value.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                return Ok(());
            }
            let code = value
                .get("error_code")
                .and_then(Value::as_i64)
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let description = value
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("Unknown Telegram API error");
            Err(NotificationDispatchError::ApiFailure {
                provider: provider_name,
                message: format!("{description} (Code: {code})"),
            })
        }
        _ => Ok(()),
    }
}

fn parse_optional_json(
    body: &str,
) -> std::result::Result<Option<Value>, NotificationDispatchError> {
    if body.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(body)
        .map(Some)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))
}

fn parse_required_json(body: &str) -> std::result::Result<Value, NotificationDispatchError> {
    parse_optional_json(body)?
        .ok_or_else(|| NotificationDispatchError::Transport("empty JSON response".to_string()))
}

fn webhook_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let endpoint = required(&config.webhook_endpoint, "Webhook address is empty")?;
    let screenshot = payload.screenshot.as_ref().map(NotificationImage::base64);
    let body = json!({
        "send_to": config.webhook_send_to,
        "event": payload.event,
        "result": legacy_result(payload.result),
        "timestamp": payload_timestamp_string(payload),
        "screenshot": screenshot,
        "message": payload.message,
        "data": payload.data
    });
    Ok(vec![json_request(
        NotificationProviderKind::Webhook,
        endpoint,
        body,
    )?])
}

fn feishu_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let endpoint = required(
        &config.feishu_webhook_url,
        "Feishu webhook endpoint is not set",
    )?;
    Ok(vec![json_request(
        NotificationProviderKind::Feishu,
        endpoint,
        json!({
            "msg_type": "text",
            "content": {
                "text": payload_message(payload)
            }
        }),
    )?])
}

fn feishu_image_dispatch<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    client: &mut C,
) -> std::result::Result<usize, NotificationDispatchError> {
    let endpoint = required(
        &config.feishu_webhook_url,
        "Feishu webhook endpoint is not set",
    )?;
    let image = payload
        .screenshot
        .as_ref()
        .ok_or_else(|| NotificationDispatchError::UnsupportedPayload("Feishu image upload"))?;
    let token = feishu_access_token(config, client)?;
    let image_key = feishu_upload_image(image, &token, client)?;
    let message = feishu_post_image_message(payload, &image_key);
    let request = json_request(NotificationProviderKind::Feishu, endpoint, message)?;
    let response = client.send(&request)?;
    validate_notification_response(NotificationProviderKind::Feishu, "Feishu", &response)?;
    Ok(3)
}

fn feishu_access_token<C: NotificationHttpClient>(
    config: &NotificationConfig,
    client: &mut C,
) -> std::result::Result<String, NotificationDispatchError> {
    let app_id = required(&config.feishu_app_id, "Feishu AppId is not set")?;
    let app_secret = required(&config.feishu_app_secret, "Feishu AppSecret is not set")?;
    let request = json_request(
        NotificationProviderKind::Feishu,
        FEISHU_ACCESS_TOKEN_URL,
        json!({
            "app_id": app_id,
            "app_secret": app_secret
        }),
    )?;
    let response = client.send(&request)?;
    validate_notification_response(NotificationProviderKind::Feishu, "Feishu", &response)?;
    let value = parse_required_json(&response.body)?;
    value
        .get("tenant_access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| NotificationDispatchError::ApiFailure {
            provider: "Feishu",
            message: "tenant_access_token not found".to_string(),
        })
}

fn feishu_upload_image<C: NotificationHttpClient>(
    image: &NotificationImage,
    access_token: &str,
    client: &mut C,
) -> std::result::Result<String, NotificationDispatchError> {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", access_token.trim()),
    );
    let request = multipart_request_with_headers(
        NotificationProviderKind::Feishu,
        FEISHU_UPLOAD_IMAGE_URL,
        vec![("image_type", "message".to_string())],
        vec![NotificationMultipartFile::with_file_name(
            "image",
            image.clone(),
            "image.png",
        )],
        headers,
    )?;
    let response = client.send(&request)?;
    validate_notification_response(NotificationProviderKind::Feishu, "Feishu", &response)?;
    let value = parse_required_json(&response.body)?;
    value
        .get("data")
        .and_then(|value| value.get("image_key"))
        .and_then(Value::as_str)
        .filter(|key| !key.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| NotificationDispatchError::ApiFailure {
            provider: "Feishu",
            message: "image_key not found".to_string(),
        })
}

fn feishu_post_image_message(payload: &NotificationPayload, image_key: &str) -> Value {
    json!({
        "msg_type": "post",
        "content": {
            "post": {
                "zh_cn": {
                    "content": [[
                        {
                            "tag": "text",
                            "text": payload_message(payload)
                        },
                        {
                            "tag": "img",
                            "image_key": image_key
                        }
                    ]]
                }
            }
        }
    })
}

fn web_socket_payload(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<(String, String), NotificationDispatchError> {
    let endpoint = required(&config.web_socket_endpoint, "WebSocket endpoint is not set")?;
    let screenshot = payload.screenshot.as_ref().map(NotificationImage::base64);
    let body = json!({
        "event": payload.event,
        "result": legacy_result(payload.result),
        "timestamp": payload_timestamp_string(payload),
        "screenshot": screenshot,
        "message": payload.message,
        "data": payload.data
    });
    let message = serde_json::to_string(&body)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    Ok((endpoint.to_string(), message))
}

fn email_request(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<NotificationEmailRequest, NotificationDispatchError> {
    let smtp_server = required(&config.smtp_server, "SMTP server is not set")?;
    if config.smtp_port == 0 {
        return Err(NotificationDispatchError::InvalidConfig(
            "SMTP port is not set".to_string(),
        ));
    }
    let from_email = required(&config.from_email, "Sender email address is empty")?;
    let to_email = required(&config.to_email, "Recipient email address is empty")?;
    let smtp_username = non_empty(&config.smtp_username);
    let smtp_password = smtp_username
        .as_ref()
        .map(|_| config.smtp_password.trim().to_string());
    let attachments = payload
        .screenshot
        .as_ref()
        .map(email_screenshot_attachment)
        .into_iter()
        .collect();

    Ok(NotificationEmailRequest {
        smtp_server: smtp_server.to_string(),
        smtp_port: config.smtp_port,
        smtp_username,
        smtp_password,
        security: email_security(smtp_server, config.smtp_port),
        from_email: from_email.to_string(),
        from_name: config.from_name.trim().to_string(),
        to_email: to_email.to_string(),
        subject: "通知 - BaseNotificationData".to_string(),
        html_body: email_body(payload),
        attachments,
    })
}

fn email_security(server: &str, port: u16) -> NotificationEmailSecurity {
    let server = server.to_ascii_lowercase();
    if port == 587
        && (server.contains("163.com") || server.contains("126.com") || server.contains("yeah.net"))
    {
        return NotificationEmailSecurity::SslOnConnect;
    }

    match port {
        465 => NotificationEmailSecurity::SslOnConnect,
        587 => NotificationEmailSecurity::StartTls,
        25 => NotificationEmailSecurity::None,
        _ => NotificationEmailSecurity::Auto,
    }
}

fn email_screenshot_attachment(image: &NotificationImage) -> NotificationEmailAttachment {
    NotificationEmailAttachment {
        file_name: "screenshot.jpg".to_string(),
        content_id: Some("screenshot".to_string()),
        content_type: "image/jpeg".to_string(),
        bytes: image_as_jpeg(image).unwrap_or_else(|| image.bytes.clone()),
    }
}

fn image_as_jpeg(image: &NotificationImage) -> Option<Vec<u8>> {
    let dynamic = image::load_from_memory(&image.bytes).ok()?;
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    JpegEncoder::new_with_quality(&mut cursor, 90)
        .encode_image(&dynamic)
        .ok()?;
    Some(bytes)
}

fn email_body(payload: &NotificationPayload) -> String {
    let mut body = String::new();
    body.push_str("<html><body style='font-family: Arial, sans-serif;'>\n");
    body.push_str("<h2 style='color: #333;'>通知详情</h2>\n");
    push_email_field(&mut body, "Event", &payload.event);
    push_email_field(&mut body, "Result", legacy_result(payload.result));
    let timestamp = payload_timestamp_string(payload);
    if !timestamp.is_empty() {
        push_email_field(&mut body, "Timestamp", &timestamp);
    }
    if let Some(message) = &payload.message {
        push_email_field(&mut body, "Message", message);
    }
    if let Some(data) = &payload.data {
        push_email_field(&mut body, "Data", &data.to_string());
    }
    if payload.screenshot.is_some() {
        body.push_str("<p><em>截图已作为附件添加到邮件中。</em></p>\n");
    }
    body.push_str("</body></html>");
    body
}

fn push_email_field(body: &mut String, name: &str, value: &str) {
    body.push_str("<p><strong>");
    body.push_str(&html_escape(name));
    body.push_str(":</strong> ");
    body.push_str(&html_escape(value));
    body.push_str("</p>\n");
}

fn html_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn one_bot_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let endpoint = required(&config.one_bot_endpoint, "OneBot endpoint is not set")?;
    if config.one_bot_user_id.trim().is_empty() && config.one_bot_group_id.trim().is_empty() {
        return Err(NotificationDispatchError::InvalidConfig(
            "OneBot requires either a user ID or group ID".to_string(),
        ));
    }
    let url = one_bot_send_msg_url(endpoint);
    let mut requests = Vec::new();
    if !config.one_bot_user_id.trim().is_empty() {
        requests.push(one_bot_request(
            &url,
            "private",
            "user_id",
            config.one_bot_user_id.trim(),
            config.one_bot_token.trim(),
            payload,
        )?);
    }
    if !config.one_bot_group_id.trim().is_empty() {
        requests.push(one_bot_request(
            &url,
            "group",
            "group_id",
            config.one_bot_group_id.trim(),
            config.one_bot_token.trim(),
            payload,
        )?);
    }
    Ok(requests)
}

fn work_weixin_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let endpoint = required(
        &config.workweixin_webhook_url,
        "WorkWeixin webhook endpoint is not set",
    )?;
    let mut requests = Vec::new();
    if let Some(image) = payload.screenshot.as_ref() {
        requests.push(work_weixin_image_request(endpoint, image)?);
    }
    requests.push(json_request(
        NotificationProviderKind::WorkWeixin,
        endpoint,
        json!({
            "msgtype": "text",
            "text": {
                "content": format!("{}\n\n{}", payload_timestamp_string(payload), payload_message(payload))
            }
        }),
    )?);
    Ok(requests)
}

fn bark_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let api_host = required(&config.bark_api_endpoint, "Bark API endpoint is not set")?;
    let device_keys = split_bark_device_keys(&config.bark_device_keys);
    if device_keys.is_empty() {
        return Err(NotificationDispatchError::InvalidConfig(
            "Bark requires at least one device key".to_string(),
        ));
    }
    let base_url = ensure_url_scheme(api_host)
        .trim_end_matches('/')
        .to_string();
    let payload = bark_payload(config, payload);
    device_keys
        .into_iter()
        .map(|key| {
            json_request(
                NotificationProviderKind::Bark,
                format!("{base_url}/{key}"),
                payload.clone(),
            )
        })
        .collect()
}

fn dingding_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let webhook = required(
        &config.dingding_webhook_url,
        "DingDing webhook URL is not set",
    )?;
    let secret = required(&config.ding_ding_secret, "DingDing secret is not set")?;
    let timestamp = payload.timestamp_ms.unwrap_or(0).to_string();
    let string_to_sign = format!("{timestamp}\n{secret}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    mac.update(string_to_sign.as_bytes());
    let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
    let url = append_query(
        webhook,
        &[
            ("timestamp", timestamp.as_str()),
            ("sign", &percent_encode(&signature)),
        ],
    );

    Ok(vec![json_request(
        NotificationProviderKind::DingDingWebhook,
        url,
        json!({
            "msgtype": "text",
            "text": { "content": payload_message(payload) },
            "at": { "atUserIds": [], "isAtAll": false }
        }),
    )?])
}

fn telegram_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let token = required(&config.telegram_bot_token, "Telegram bot token is not set")?;
    let chat_id = required(&config.telegram_chat_id, "Telegram chat ID is not set")?;
    let message = payload_message(payload);
    if message.is_empty() {
        return Err(NotificationDispatchError::InvalidConfig(
            "No message content to send".to_string(),
        ));
    }
    if let Some(image) = payload.screenshot.as_ref() {
        let url = format!(
            "{}{}/sendPhoto",
            telegram_base_url(&config.telegram_api_base_url),
            token
        );
        let mut fields = vec![("chat_id", chat_id.to_string())];
        if message.chars().count() < 1024 {
            fields.push(("caption", message));
        }
        return Ok(vec![multipart_request(
            NotificationProviderKind::Telegram,
            url,
            fields,
            vec![NotificationMultipartFile::new("photo", image.clone())],
        )?]);
    }
    let url = format!(
        "{}{}/sendMessage",
        telegram_base_url(&config.telegram_api_base_url),
        token
    );
    Ok(vec![json_request(
        NotificationProviderKind::Telegram,
        url,
        json!({
            "chat_id": chat_id,
            "text": message,
            "disable_web_page_preview": true
        }),
    )?])
}

fn xxtui_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let api_key = required(&config.xxtui_api_key, "Xxtui API key is not set")?;
    let mut message = format!("[{}] {}", payload.event, payload_message(payload));
    if message.chars().count() > 2000 {
        message = message.chars().take(1997).collect::<String>() + "...";
    }
    let from = if config.xxtui_from.chars().count() > 20 {
        config.xxtui_from.chars().take(20).collect::<String>()
    } else {
        config.xxtui_from.clone()
    };
    let channels = parse_xxtui_channels(&config.xxtui_channels).join(",");
    Ok(vec![form_request(
        NotificationProviderKind::Xxtui,
        format!("https://www.xxtui.com/xxtui/{api_key}"),
        &[("content", message), ("from", from), ("channel", channels)],
    )])
}

fn discord_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let webhook = required(
        &config.discord_webhook_url,
        "Discord webhook URL is not set",
    )?;
    let components = vec![
        json!({
            "type": 10,
            "content": payload_message(payload)
        }),
        json!({
            "type": 10,
            "content": format!("-# {} | {}\n-# {}", payload.event, legacy_result(payload.result), payload_timestamp_string(payload))
        }),
    ];
    let mut body = serde_json::Map::new();
    body.insert("flags".to_string(), json!(1 << 15));
    if !config.discord_webhook_username.trim().is_empty() {
        body.insert(
            "username".to_string(),
            json!(config.discord_webhook_username.trim()),
        );
    }
    if !config.discord_webhook_avatar_url.trim().is_empty() {
        body.insert(
            "avatar_url".to_string(),
            json!(config.discord_webhook_avatar_url.trim()),
        );
    }
    let url = append_query(webhook, &[("with_components", "true")]);
    if let Some(image) = payload.screenshot.as_ref() {
        let file_name = discord_screenshot_file_name(config, image);
        body.insert(
            "attachments".to_string(),
            json!([{ "id": 0, "filename": file_name }]),
        );
        body.insert(
            "components".to_string(),
            json!([{
                "type": 17,
                "components": [{
                    "type": 9,
                    "components": components,
                    "accessory": {
                        "type": 11,
                        "media": { "url": format!("attachment://{file_name}") },
                        "description": "Screenshot"
                    }
                }]
            }]),
        );
        return Ok(vec![multipart_request(
            NotificationProviderKind::DiscordWebhook,
            url,
            vec![("payload_json", Value::Object(body).to_string())],
            vec![NotificationMultipartFile::with_file_name(
                "files[0]",
                image.clone(),
                file_name,
            )],
        )?]);
    }

    body.insert(
        "components".to_string(),
        json!([{ "type": 17, "components": components }]),
    );
    Ok(vec![json_request(
        NotificationProviderKind::DiscordWebhook,
        url,
        Value::Object(body),
    )?])
}

fn server_chan_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let send_key = required(&config.server_chan_send_key, "ServerChan SendKey is empty")?;
    let url = server_chan_url(send_key)?;
    Ok(vec![form_request(
        NotificationProviderKind::ServerChan,
        url,
        &[
            ("title", "BetterGI·更好的原神".to_string()),
            ("desp", server_chan_description(payload)),
        ],
    )])
}

fn meow_requests(
    config: &NotificationConfig,
    payload: &NotificationPayload,
) -> std::result::Result<Vec<NotificationHttpRequest>, NotificationDispatchError> {
    let nickname = required(&config.meow_nickname, "MeoW nickname is empty")?;
    let mut url = format!(
        "https://api.chuckfang.com/{}",
        percent_encode_path_segment(nickname)
    );
    if !config.meow_title.trim().is_empty() {
        url.push('/');
        url.push_str(&percent_encode_path_segment(config.meow_title.trim()));
    }
    Ok(vec![json_request(
        NotificationProviderKind::Meow,
        url,
        json!({
            "title": "BetterGI·更好的原神",
            "msg": meow_message(payload)
        }),
    )?])
}

fn one_bot_request(
    url: &str,
    message_type: &str,
    id_field: &str,
    id_value: &str,
    token: &str,
    payload: &NotificationPayload,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    let mut body = serde_json::Map::new();
    let mut message = vec![json!({ "type": "text", "data": { "text": payload_message(payload) } })];
    if let Some(image) = payload.screenshot.as_ref() {
        message.push(json!({
            "type": "image",
            "data": { "file": format!("base64://{}", image.base64()) }
        }));
    }
    body.insert("message".to_string(), Value::Array(message));
    body.insert("message_type".to_string(), json!(message_type));
    body.insert(id_field.to_string(), json!(id_value));
    let mut request = json_request(NotificationProviderKind::OneBot, url, Value::Object(body))?;
    if !token.is_empty() {
        request
            .headers
            .insert("Authorization".to_string(), format!("Bearer {token}"));
    }
    Ok(request)
}

fn work_weixin_image_request(
    endpoint: &str,
    image: &NotificationImage,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    Ok(json_request(
        NotificationProviderKind::WorkWeixin,
        endpoint,
        json!({
            "msgtype": "image",
            "image": {
                "base64": image.base64(),
                "md5": hex_lower(Md5::digest(&image.bytes).as_slice())
            }
        }),
    )?)
}

fn json_request(
    provider: NotificationProviderKind,
    url: impl Into<String>,
    body: Value,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    let mut headers = BTreeMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let body = serde_json::to_vec(&body)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    Ok(NotificationHttpRequest {
        provider,
        method: "POST".to_string(),
        url: url.into(),
        headers,
        body,
        body_kind: NotificationHttpBodyKind::Json,
    })
}

fn multipart_request(
    provider: NotificationProviderKind,
    url: impl Into<String>,
    fields: Vec<(&str, String)>,
    files: Vec<NotificationMultipartFile>,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    multipart_request_with_headers(provider, url, fields, files, BTreeMap::new())
}

fn multipart_request_with_headers(
    provider: NotificationProviderKind,
    url: impl Into<String>,
    fields: Vec<(&str, String)>,
    files: Vec<NotificationMultipartFile>,
    mut headers: BTreeMap<String, String>,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    let boundary = multipart_boundary(provider);
    let mut body = Vec::new();
    for (name, value) in fields {
        push_multipart_field(&mut body, &boundary, name, &value);
    }
    for file in files {
        push_multipart_file(&mut body, &boundary, &file);
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    headers.insert(
        "Content-Type".to_string(),
        format!("multipart/form-data; boundary={boundary}"),
    );
    Ok(NotificationHttpRequest {
        provider,
        method: "POST".to_string(),
        url: url.into(),
        headers,
        body,
        body_kind: NotificationHttpBodyKind::MultipartFormData,
    })
}

fn form_request(
    provider: NotificationProviderKind,
    url: impl Into<String>,
    fields: &[(&str, String)],
) -> NotificationHttpRequest {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Content-Type".to_string(),
        "application/x-www-form-urlencoded".to_string(),
    );
    NotificationHttpRequest {
        provider,
        method: "POST".to_string(),
        url: url.into(),
        headers,
        body: form_urlencoded(fields).into_bytes(),
        body_kind: NotificationHttpBodyKind::FormUrlEncoded,
    }
}

fn push_multipart_field(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn push_multipart_file(body: &mut Vec<u8>, boundary: &str, file: &NotificationMultipartFile) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
            file.field_name, file.file_name
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", file.mime_type).as_bytes());
    body.extend_from_slice(&file.bytes);
    body.extend_from_slice(b"\r\n");
}

fn multipart_boundary(provider: NotificationProviderKind) -> String {
    format!("----bettergi-rust-{provider:?}")
}

fn required<'a>(
    value: &'a str,
    message: &'static str,
) -> std::result::Result<&'a str, NotificationDispatchError> {
    let value = value.trim();
    if value.is_empty() {
        Err(NotificationDispatchError::InvalidConfig(
            message.to_string(),
        ))
    } else {
        Ok(value)
    }
}

fn payload_message(payload: &NotificationPayload) -> String {
    payload.message.clone().unwrap_or_default()
}

fn payload_has_screenshot(payload: &NotificationPayload) -> bool {
    payload.screenshot.is_some() || payload.has_screenshot
}

fn legacy_result(result: NotificationEventResult) -> &'static str {
    match result {
        NotificationEventResult::Success => "Success",
        NotificationEventResult::Fail => "Fail",
        NotificationEventResult::PartialSuccess => "PartialSuccess",
    }
}

fn payload_timestamp_string(payload: &NotificationPayload) -> String {
    payload
        .timestamp_ms
        .and_then(|timestamp| Local.timestamp_millis_opt(timestamp as i64).single())
        .map(|time: DateTime<Local>| time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}

fn one_bot_send_msg_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim_end_matches('/');
    if trimmed.ends_with("/send_msg") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/send_msg")
    }
}

fn split_bark_device_keys(keys: &str) -> Vec<String> {
    keys.split([',', ';', ' ']).filter_map(non_empty).collect()
}

fn bark_payload(config: &NotificationConfig, payload: &NotificationPayload) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("title".to_string(), json!("通知 - BaseNotificationData"));
    body.insert("body".to_string(), json!(bark_body(payload)));

    insert_non_empty(&mut body, "level", &config.bark_level);
    insert_non_empty(&mut body, "sound", &config.bark_sound);
    if config.bark_volume > 0 {
        body.insert("volume".to_string(), json!(config.bark_volume));
    }
    if config.bark_badge > 0 {
        body.insert("badge".to_string(), json!(config.bark_badge));
    }
    insert_non_empty(&mut body, "call", &config.bark_call);
    insert_non_empty(&mut body, "autoCopy", &config.bark_auto_copy);
    insert_non_empty(&mut body, "copy", &config.bark_copy);
    insert_non_empty(&mut body, "icon", &config.bark_icon);
    insert_non_empty(&mut body, "group", &config.bark_group);
    insert_non_empty(&mut body, "ciphertext", &config.bark_ciphertext);
    insert_non_empty(&mut body, "isArchive", &config.bark_is_archive);
    insert_non_empty(&mut body, "url", &config.bark_url);
    insert_non_empty(&mut body, "action", &config.bark_action);
    Value::Object(body)
}

fn insert_non_empty(map: &mut serde_json::Map<String, Value>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        map.insert(key.to_string(), json!(value));
    }
}

fn bark_body(payload: &NotificationPayload) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Event: {}", payload.event));
    lines.push(format!("Result: {}", legacy_result(payload.result)));
    if let Some(timestamp) = payload.timestamp_ms {
        lines.push(format!("Timestamp: {}", timestamp));
    }
    if payload_has_screenshot(payload) {
        lines.push("Screenshot: <captured>".to_string());
    }
    if let Some(message) = &payload.message {
        lines.push(format!("Message: {message}"));
    }
    if let Some(data) = &payload.data {
        lines.push(format!("Data: {data}"));
    }
    lines.join("\n")
}

fn telegram_base_url(api_base_url: &str) -> String {
    if api_base_url.trim().is_empty() {
        return "https://api.telegram.org/bot".to_string();
    }
    let mut url = ensure_url_scheme(api_base_url.trim());
    if !url.ends_with('/') {
        url.push('/');
    }
    if !url.ends_with("/bot") {
        url.push_str("bot");
    }
    url
}

fn discord_screenshot_file_name(config: &NotificationConfig, image: &NotificationImage) -> String {
    let extension = match config
        .discord_webhook_image_encoder
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "png",
        "webp" => "webp",
        "web" => "webp",
        "jpeg" | "jpg" => "jpeg",
        _ => image
            .file_name
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .unwrap_or("jpeg"),
    };
    format!("screenshot.{extension}")
}

fn ensure_url_scheme(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://{value}")
    }
}

fn server_chan_url(send_key: &str) -> std::result::Result<String, NotificationDispatchError> {
    if let Some(rest) = send_key.strip_prefix("sctp") {
        let Some((number, _)) = rest.split_once('t') else {
            return Err(NotificationDispatchError::InvalidConfig(
                "Invalid key format for sctp".to_string(),
            ));
        };
        if number.is_empty() || !number.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(NotificationDispatchError::InvalidConfig(
                "Invalid key format for sctp".to_string(),
            ));
        }
        Ok(format!(
            "https://{number}.push.ft07.com/send/{send_key}.send"
        ))
    } else {
        Ok(format!("https://sctapi.ftqq.com/{send_key}.send"))
    }
}

fn server_chan_description(payload: &NotificationPayload) -> String {
    let mut lines = Vec::new();
    lines.push(format!("**时间**: {}", payload_timestamp_string(payload)));
    if let Some(message) = &payload.message {
        if !message.is_empty() {
            lines.push(String::new());
            lines.push(format!("**消息**: {message}"));
        }
    }
    lines.join("\n")
}

fn meow_message(payload: &NotificationPayload) -> String {
    let mut lines = Vec::new();
    lines.push(format!("时间: {}", payload_timestamp_string(payload)));
    if let Some(message) = &payload.message {
        if !message.is_empty() {
            lines.push(String::new());
            lines.push(format!("事件消息: {message}"));
        }
    }
    lines.join("\n")
}

fn append_query(base: &str, params: &[(&str, &str)]) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    let mut url = base.to_string();
    if !params.is_empty() {
        url.push(separator);
        url.push_str(
            &params
                .iter()
                .map(|(key, value)| format!("{}={}", percent_encode(key), value))
                .collect::<Vec<_>>()
                .join("&"),
        );
    }
    url
}

fn form_urlencoded(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['%', '2', '0'],
            _ => {
                let encoded = format!("%{byte:02X}");
                encoded.chars().collect()
            }
        })
        .collect()
}

fn percent_encode_path_segment(value: &str) -> String {
    percent_encode(value)
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

const NOTIFICATION_EVENTS: &[NotificationEventDescriptor] = &[
    NotificationEventDescriptor {
        code: "notify.test",
        message: "测试通知",
    },
    NotificationEventDescriptor {
        code: "domain.reward",
        message: "自动秘境奖励",
    },
    NotificationEventDescriptor {
        code: "domain.start",
        message: "自动秘境启动",
    },
    NotificationEventDescriptor {
        code: "domain.end",
        message: "自动秘境结束",
    },
    NotificationEventDescriptor {
        code: "domain.retry",
        message: "自动秘境重试",
    },
    NotificationEventDescriptor {
        code: "task.cancel",
        message: "任务启动",
    },
    NotificationEventDescriptor {
        code: "task.error",
        message: "任务错误",
    },
    NotificationEventDescriptor {
        code: "group.start",
        message: "配置组启动",
    },
    NotificationEventDescriptor {
        code: "group.end",
        message: "配置组结束",
    },
    NotificationEventDescriptor {
        code: "dragon.start",
        message: "一条龙启动",
    },
    NotificationEventDescriptor {
        code: "dragon.end",
        message: "一条龙结束",
    },
    NotificationEventDescriptor {
        code: "tcg.start",
        message: "七圣召唤启动",
    },
    NotificationEventDescriptor {
        code: "tcg.end",
        message: "七圣召唤结束",
    },
    NotificationEventDescriptor {
        code: "album.start",
        message: "自动音游专辑启动",
    },
    NotificationEventDescriptor {
        code: "album.end",
        message: "自动音游专辑结束",
    },
    NotificationEventDescriptor {
        code: "album.error",
        message: "自动音游专辑错误",
    },
    NotificationEventDescriptor {
        code: "daily.reward",
        message: "检查每日奖励领取状态",
    },
    NotificationEventDescriptor {
        code: "js.custom",
        message: "JS自定义事件",
    },
    NotificationEventDescriptor {
        code: "js.error",
        message: "JS运行时错误",
    },
    NotificationEventDescriptor {
        code: "autoeat.start",
        message: "自动吃药启动",
    },
    NotificationEventDescriptor {
        code: "autoeat.end",
        message: "自动吃药结束",
    },
    NotificationEventDescriptor {
        code: "autoeat.info",
        message: "自动吃药信息",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_events_preserve_legacy_order_and_codes() {
        let events = notification_events();
        assert_eq!(events.first().unwrap().code, "notify.test");
        assert_eq!(events[17].code, "js.custom");
        assert_eq!(events[18].code, "js.error");
        assert_eq!(events.last().unwrap().code, "autoeat.info");
        assert_eq!(events.len(), 22);
    }

    #[test]
    fn notification_subscription_helpers_match_legacy_rules() {
        assert_eq!(
            parse_notification_event_codes(Some(" js.custom,JS.CUSTOM, task.error ,, ")),
            vec!["js.custom".to_string(), "task.error".to_string()]
        );
        assert_eq!(
            normalize_notification_event_codes([" js.custom ", "JS.CUSTOM", "task.error"]),
            "js.custom,task.error"
        );
        assert!(should_send_notification(None, Some("any")));
        assert!(should_send_notification(Some(""), Some("any")));
        assert!(should_send_notification(
            Some("js.custom,task.error"),
            Some("JS.CUSTOM")
        ));
        assert!(!should_send_notification(
            Some("js.custom"),
            Some("task.error")
        ));
        assert!(!should_send_notification(Some("js.custom"), None));
    }

    #[test]
    fn notification_provider_plans_follow_legacy_initialization_conditions() {
        let mut config = NotificationConfig::default();
        assert!(notification_provider_plans(&config).is_empty());

        config.webhook_enabled = true;
        config.webhook_endpoint = "https://example.com/webhook".to_string();
        config.windows_uwp_notification_enabled = true;
        config.feishu_notification_enabled = true;
        config.one_bot_notification_enabled = true;
        config.workweixin_notification_enabled = true;
        config.web_socket_notification_enabled = true;
        config.bark_notification_enabled = true;
        config.email_notification_enabled = true;
        config.ding_dingwebhook_notification_enabled = true;
        config.telegram_notification_enabled = true;
        config.xxtui_notification_enabled = true;
        config.xxtui_channels = "WX_MP,invalid,EMAIL".to_string();
        config.discord_webhook_notification_enabled = true;
        config.server_chan_notification_enabled = true;
        config.meow_notification_enabled = true;

        let without_dingding = notification_provider_plans(&config);
        assert_eq!(without_dingding.len(), 13);
        assert!(!without_dingding
            .iter()
            .any(|plan| plan.kind == NotificationProviderKind::DingDingWebhook));
        let xxtui = without_dingding
            .iter()
            .find(|plan| plan.kind == NotificationProviderKind::Xxtui)
            .unwrap();
        assert_eq!(xxtui.config_summary["channels"], json!(["WX_MP", "EMAIL"]));

        config.dingding_webhook_url = "https://example.com/ding".to_string();
        config.ding_ding_secret = "secret".to_string();
        let with_dingding = notification_provider_plans(&config);
        assert_eq!(with_dingding.len(), 14);
        assert!(with_dingding
            .iter()
            .any(|plan| plan.kind == NotificationProviderKind::DingDingWebhook));
    }

    #[test]
    fn notification_dispatch_plan_filters_by_subscription_and_screenshot_setting() {
        let mut config = NotificationConfig {
            webhook_enabled: true,
            webhook_endpoint: "https://example.com/webhook".to_string(),
            notification_event_subscribe: "task.error".to_string(),
            include_screen_shot: true,
            ..NotificationConfig::default()
        };

        let skipped = notification_dispatch_plan(
            &config,
            NotificationPayload::success("js.custom", "ignored"),
        );
        assert!(!skipped.should_send);
        assert_eq!(skipped.skipped_reason, Some("event_not_subscribed"));
        assert!(skipped.providers.is_empty());

        let sent =
            notification_dispatch_plan(&config, NotificationPayload::fail("task.error", "failed"));
        assert!(sent.should_send);
        assert!(sent.include_screenshot);
        assert_eq!(sent.providers.len(), 1);

        config.include_screen_shot = false;
        let sent_without_screenshot =
            notification_dispatch_plan(&config, NotificationPayload::fail("task.error", "failed"));
        assert!(!sent_without_screenshot.include_screenshot);
    }

    #[test]
    fn notification_dispatch_plan_can_target_one_provider_for_legacy_tests() {
        let config = NotificationConfig {
            webhook_enabled: true,
            webhook_endpoint: "https://example.com/webhook".to_string(),
            web_socket_notification_enabled: true,
            web_socket_endpoint: "ws://127.0.0.1:12345/notify".to_string(),
            ..NotificationConfig::default()
        };

        let targeted = notification_dispatch_plan_for_provider(
            &config,
            NotificationPayload::success("notify.test", "hello"),
            NotificationProviderKind::WebSocket,
        );
        assert!(targeted.should_send);
        assert_eq!(targeted.providers.len(), 1);
        assert_eq!(
            targeted.providers[0].kind,
            NotificationProviderKind::WebSocket
        );

        let disabled = notification_dispatch_plan_for_provider(
            &config,
            NotificationPayload::success("notify.test", "hello"),
            NotificationProviderKind::Email,
        );
        assert!(disabled.should_send);
        assert!(disabled.providers.is_empty());
    }

    #[test]
    fn notification_http_dispatch_executes_webhook_like_legacy_payload() {
        let config = NotificationConfig {
            webhook_enabled: true,
            webhook_endpoint: "https://example.com/webhook".to_string(),
            webhook_send_to: "ops".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::success("js.custom", "done");
        payload.timestamp_ms = Some(1_700_000_000_000);
        payload.data = Some(json!({"count": 2}));
        let mut client = RecordingNotificationHttpClient::new();

        let execution = execute_notification_dispatch(&config, payload, &mut client);

        assert!(execution.attempted);
        assert_eq!(execution.deliveries.len(), 1);
        assert_eq!(
            execution.deliveries[0].status,
            NotificationProviderDeliveryStatus::Sent
        );
        assert_eq!(client.requests.len(), 1);
        let request = &client.requests[0];
        assert_eq!(request.provider, NotificationProviderKind::Webhook);
        assert_eq!(request.url, "https://example.com/webhook");
        assert_eq!(
            request.headers.get("Content-Type").map(String::as_str),
            Some("application/json")
        );
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(body["send_to"], json!("ops"));
        assert_eq!(body["event"], json!("js.custom"));
        assert_eq!(body["result"], json!("Success"));
        assert_eq!(body["message"], json!("done"));
        assert_eq!(body["data"], json!({"count": 2}));
        assert!(body["timestamp"].as_str().unwrap().contains("2023"));
        assert!(body["screenshot"].is_null());
    }

    #[test]
    fn notification_websocket_dispatch_sends_snake_case_json_text() {
        let config = NotificationConfig {
            web_socket_notification_enabled: true,
            web_socket_endpoint: "ws://127.0.0.1:12345/notify".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::fail("task.error", "boom")
            .with_screenshot(NotificationImage::png(vec![9, 8, 7]));
        payload.timestamp_ms = Some(1_700_000_000_000);
        payload.data = Some(json!({"step_count": 3}));
        let mut http_client = RecordingNotificationHttpClient::new();
        let mut web_socket_client = RecordingNotificationWebSocketClient::new();

        let execution = execute_notification_dispatch_with_websocket(
            &config,
            payload,
            &mut http_client,
            &mut web_socket_client,
        );

        assert!(execution.attempted);
        assert_eq!(execution.deliveries.len(), 1);
        assert_eq!(
            execution.deliveries[0].status,
            NotificationProviderDeliveryStatus::Sent
        );
        assert!(http_client.requests.is_empty());
        assert_eq!(web_socket_client.messages.len(), 1);
        assert_eq!(
            web_socket_client.messages[0].0,
            "ws://127.0.0.1:12345/notify"
        );
        let body: Value = serde_json::from_str(&web_socket_client.messages[0].1).unwrap();
        assert_eq!(body["event"], json!("task.error"));
        assert_eq!(body["result"], json!("Fail"));
        assert_eq!(body["message"], json!("boom"));
        assert_eq!(body["data"], json!({"step_count": 3}));
        assert_eq!(body["screenshot"], json!("CQgH"));
        assert!(body["timestamp"].as_str().unwrap().contains("2023"));
    }

    #[test]
    fn notification_email_dispatch_builds_legacy_html_and_attachment() {
        let config = NotificationConfig {
            email_notification_enabled: true,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "bot".to_string(),
            smtp_password: "secret".to_string(),
            from_email: "from@example.com".to_string(),
            from_name: "BetterGI".to_string(),
            to_email: "to@example.com".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::fail("task.error", "boom <failed>").with_screenshot(
            NotificationImage::new(vec![9, 8, 7], "application/octet-stream", "raw.bin"),
        );
        payload.timestamp_ms = Some(1_700_000_000_000);
        payload.data = Some(json!({"step": 3}));
        let mut http_client = RecordingNotificationHttpClient::new();
        let mut web_socket_client = RecordingNotificationWebSocketClient::new();
        let mut email_client = RecordingNotificationEmailClient::new();
        let mut windows_toast_client = RecordingNotificationWindowsToastClient::new();

        let execution = execute_notification_dispatch_with_transports(
            &config,
            payload,
            &mut http_client,
            &mut web_socket_client,
            &mut email_client,
            &mut windows_toast_client,
        );

        assert!(execution.attempted);
        assert_eq!(execution.deliveries.len(), 1);
        assert_eq!(
            execution.deliveries[0].status,
            NotificationProviderDeliveryStatus::Sent
        );
        assert!(http_client.requests.is_empty());
        assert!(web_socket_client.messages.is_empty());
        assert!(windows_toast_client.toasts.is_empty());
        assert_eq!(email_client.emails.len(), 1);
        let email = &email_client.emails[0];
        assert_eq!(email.smtp_server, "smtp.example.com");
        assert_eq!(email.smtp_port, 587);
        assert_eq!(email.smtp_username.as_deref(), Some("bot"));
        assert_eq!(email.smtp_password.as_deref(), Some("secret"));
        assert_eq!(email.security, NotificationEmailSecurity::StartTls);
        assert_eq!(email.from_email, "from@example.com");
        assert_eq!(email.from_name, "BetterGI");
        assert_eq!(email.to_email, "to@example.com");
        assert_eq!(email.subject, "通知 - BaseNotificationData");
        assert!(email.html_body.contains("通知详情"));
        assert!(email
            .html_body
            .contains("<strong>Event:</strong> task.error"));
        assert!(email.html_body.contains("<strong>Result:</strong> Fail"));
        assert!(email.html_body.contains("boom &lt;failed&gt;"));
        assert!(email.html_body.contains("{&quot;step&quot;:3}"));
        assert!(email.html_body.contains("截图已作为附件添加到邮件中。"));
        assert_eq!(email.attachments.len(), 1);
        assert_eq!(email.attachments[0].file_name, "screenshot.jpg");
        assert_eq!(
            email.attachments[0].content_id.as_deref(),
            Some("screenshot")
        );
        assert_eq!(email.attachments[0].content_type, "image/jpeg");
        assert_eq!(email.attachments[0].bytes, vec![9, 8, 7]);
    }

    #[test]
    fn notification_windows_toast_dispatch_preserves_legacy_shape() {
        let config = NotificationConfig {
            windows_uwp_notification_enabled: true,
            ..NotificationConfig::default()
        };
        let payload = NotificationPayload::success("notify.test", "toast message")
            .with_screenshot(NotificationImage::png(vec![1, 2, 3]));
        let mut http_client = RecordingNotificationHttpClient::new();
        let mut web_socket_client = RecordingNotificationWebSocketClient::new();
        let mut email_client = RecordingNotificationEmailClient::new();
        let mut windows_toast_client = RecordingNotificationWindowsToastClient::new();

        let execution = execute_notification_dispatch_with_transports(
            &config,
            payload,
            &mut http_client,
            &mut web_socket_client,
            &mut email_client,
            &mut windows_toast_client,
        );

        assert!(execution.attempted);
        assert_eq!(execution.deliveries.len(), 1);
        assert_eq!(
            execution.deliveries[0].status,
            NotificationProviderDeliveryStatus::Sent
        );
        assert!(http_client.requests.is_empty());
        assert!(web_socket_client.messages.is_empty());
        assert!(email_client.emails.is_empty());
        assert_eq!(windows_toast_client.toasts.len(), 1);
        let toast = &windows_toast_client.toasts[0];
        assert_eq!(toast.event, "notify.test");
        assert_eq!(toast.message.as_deref(), Some("toast message"));
        assert_eq!(toast.expiration_hours, 12);
        assert_eq!(
            toast
                .screenshot
                .as_ref()
                .map(|image| image.bytes.as_slice()),
            Some([1, 2, 3].as_slice())
        );
    }

    #[test]
    fn notification_email_security_preserves_legacy_port_rules() {
        assert_eq!(
            email_security("smtp.163.com", 587),
            NotificationEmailSecurity::SslOnConnect
        );
        assert_eq!(
            email_security("smtp.example.com", 465),
            NotificationEmailSecurity::SslOnConnect
        );
        assert_eq!(
            email_security("smtp.example.com", 587),
            NotificationEmailSecurity::StartTls
        );
        assert_eq!(
            email_security("smtp.example.com", 25),
            NotificationEmailSecurity::None
        );
        assert_eq!(
            email_security("smtp.example.com", 2525),
            NotificationEmailSecurity::Auto
        );
    }

    #[test]
    fn notification_one_bot_generates_private_and_group_requests_with_token() {
        let config = NotificationConfig {
            one_bot_notification_enabled: true,
            one_bot_endpoint: "http://127.0.0.1:5700".to_string(),
            one_bot_user_id: "10001".to_string(),
            one_bot_group_id: "20002".to_string(),
            one_bot_token: "secret-token".to_string(),
            ..NotificationConfig::default()
        };
        let provider = provider(NotificationProviderKind::OneBot, "OneBot", None, json!({}));

        let requests = notification_http_requests_for_provider(
            &config,
            &NotificationPayload::fail("task.error", "boom"),
            &provider,
        )
        .unwrap();

        assert_eq!(requests.len(), 2);
        assert!(requests
            .iter()
            .all(|request| request.url == "http://127.0.0.1:5700/send_msg"));
        assert!(requests.iter().all(|request| request
            .headers
            .get("Authorization")
            .map(String::as_str)
            == Some("Bearer secret-token")));
        let private: Value = serde_json::from_slice(&requests[0].body).unwrap();
        let group: Value = serde_json::from_slice(&requests[1].body).unwrap();
        assert_eq!(private["message_type"], json!("private"));
        assert_eq!(private["user_id"], json!("10001"));
        assert_eq!(private["message"][0]["data"]["text"], json!("boom"));
        assert_eq!(group["message_type"], json!("group"));
        assert_eq!(group["group_id"], json!("20002"));
    }

    #[test]
    fn notification_dispatch_builds_signed_dingding_and_server_chan_requests() {
        let config = NotificationConfig {
            ding_dingwebhook_notification_enabled: true,
            dingding_webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=abc"
                .to_string(),
            ding_ding_secret: "ding-secret".to_string(),
            server_chan_notification_enabled: true,
            server_chan_send_key: "sctp123tkey".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::success("notify.test", "hello");
        payload.timestamp_ms = Some(1_234);
        let mut client = RecordingNotificationHttpClient::new();

        let execution = execute_notification_dispatch(&config, payload, &mut client);

        assert_eq!(execution.deliveries.len(), 2);
        assert!(execution
            .deliveries
            .iter()
            .all(|delivery| delivery.status == NotificationProviderDeliveryStatus::Sent));
        let ding = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::DingDingWebhook)
            .unwrap();
        assert!(ding.url.contains("&timestamp=1234&sign="));
        let ding_body: Value = serde_json::from_slice(&ding.body).unwrap();
        assert_eq!(ding_body["msgtype"], json!("text"));
        assert_eq!(ding_body["text"]["content"], json!("hello"));

        let server_chan = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::ServerChan)
            .unwrap();
        assert_eq!(
            server_chan.url,
            "https://123.push.ft07.com/send/sctp123tkey.send"
        );
        let body = String::from_utf8(server_chan.body.clone()).unwrap();
        assert!(body.contains("title=BetterGI"));
        assert!(body.contains("%E6%9B%B4%E5%A5%BD%E7%9A%84%E5%8E%9F%E7%A5%9E"));
        assert!(body.contains("hello"));
    }

    #[test]
    fn notification_feishu_image_upload_uses_token_upload_and_post_message() {
        let config = NotificationConfig {
            feishu_notification_enabled: true,
            feishu_webhook_url: "https://open.feishu.cn/webhook".to_string(),
            feishu_app_id: "app".to_string(),
            feishu_app_secret: "secret".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::success("notify.test", "with image");
        payload = payload.with_screenshot(NotificationImage::png(vec![1, 2, 3]));
        let mut client = RecordingNotificationHttpClient::with_responses(vec![
            NotificationHttpResponse {
                status: 200,
                body: r#"{"tenant_access_token":"tenant-token"}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: r#"{"data":{"image_key":"img-key"}}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            },
        ]);

        let execution = execute_notification_dispatch(&config, payload, &mut client);

        assert_eq!(client.requests.len(), 3);
        assert_eq!(execution.deliveries.len(), 1);
        assert_eq!(
            execution.deliveries[0].status,
            NotificationProviderDeliveryStatus::Sent
        );
        assert_eq!(execution.deliveries[0].requests, 3);
        let token_body: Value = serde_json::from_slice(&client.requests[0].body).unwrap();
        assert_eq!(client.requests[0].url, FEISHU_ACCESS_TOKEN_URL);
        assert_eq!(token_body["app_id"], json!("app"));
        assert_eq!(token_body["app_secret"], json!("secret"));
        assert_eq!(client.requests[1].url, FEISHU_UPLOAD_IMAGE_URL);
        assert_eq!(
            client.requests[1].headers["Authorization"],
            "Bearer tenant-token"
        );
        let upload_body = String::from_utf8(client.requests[1].body.clone()).unwrap();
        assert!(upload_body.contains("name=\"image_type\""));
        assert!(upload_body.contains("message"));
        assert!(upload_body.contains("name=\"image\"; filename=\"image.png\""));
        let webhook_body: Value = serde_json::from_slice(&client.requests[2].body).unwrap();
        assert_eq!(webhook_body["msg_type"], json!("post"));
        assert_eq!(
            webhook_body["content"]["post"]["zh_cn"]["content"][0][0]["text"],
            json!("with image")
        );
        assert_eq!(
            webhook_body["content"]["post"]["zh_cn"]["content"][0][1]["image_key"],
            json!("img-key")
        );
    }

    #[test]
    fn notification_screenshot_requests_cover_json_and_multipart_providers() {
        let config = NotificationConfig {
            webhook_enabled: true,
            webhook_endpoint: "https://example.com/webhook".to_string(),
            one_bot_notification_enabled: true,
            one_bot_endpoint: "http://127.0.0.1:5700".to_string(),
            one_bot_user_id: "10001".to_string(),
            workweixin_notification_enabled: true,
            workweixin_webhook_url: "https://qyapi.example/webhook".to_string(),
            feishu_notification_enabled: true,
            feishu_webhook_url: "https://open.feishu.cn/webhook".to_string(),
            feishu_app_id: "app".to_string(),
            feishu_app_secret: "secret".to_string(),
            telegram_notification_enabled: true,
            telegram_bot_token: "token".to_string(),
            telegram_chat_id: "chat".to_string(),
            discord_webhook_notification_enabled: true,
            discord_webhook_url: "https://discord.example/webhook".to_string(),
            discord_webhook_image_encoder: "Png".to_string(),
            ..NotificationConfig::default()
        };
        let payload = NotificationPayload::success("notify.test", "with image")
            .with_screenshot(NotificationImage::png(vec![1, 2, 3, 4]));
        let mut client = RecordingNotificationHttpClient::with_responses(vec![
            NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: r#"{"tenant_access_token":"tenant-token"}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: r#"{"data":{"image_key":"img-key"}}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: r#"{"status":"ok"}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: "{}".to_string(),
            },
            NotificationHttpResponse {
                status: 200,
                body: r#"{"ok":true}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 204,
                body: String::new(),
            },
        ]);

        let execution = execute_notification_dispatch(&config, payload, &mut client);

        assert!(execution
            .deliveries
            .iter()
            .all(|delivery| delivery.status == NotificationProviderDeliveryStatus::Sent));
        assert_eq!(client.requests.len(), 9);

        let webhook = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::Webhook)
            .unwrap();
        let webhook_body: Value = serde_json::from_slice(&webhook.body).unwrap();
        assert_eq!(webhook_body["screenshot"], json!("AQIDBA=="));

        let one_bot = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::OneBot)
            .unwrap();
        let one_bot_body: Value = serde_json::from_slice(&one_bot.body).unwrap();
        assert_eq!(
            one_bot_body["message"][1]["data"]["file"],
            json!("base64://AQIDBA==")
        );

        let work_image = client
            .requests
            .iter()
            .find(|request| {
                request.provider == NotificationProviderKind::WorkWeixin
                    && serde_json::from_slice::<Value>(&request.body)
                        .ok()
                        .and_then(|body| body.get("msgtype").cloned())
                        == Some(json!("image"))
            })
            .unwrap();
        let work_body: Value = serde_json::from_slice(&work_image.body).unwrap();
        assert_eq!(work_body["image"]["base64"], json!("AQIDBA=="));
        assert_eq!(
            work_body["image"]["md5"],
            json!("08d6c05a21512a79a1dfeb9d2a8f262f")
        );

        let feishu_upload = client
            .requests
            .iter()
            .find(|request| request.url == FEISHU_UPLOAD_IMAGE_URL)
            .unwrap();
        assert_eq!(
            feishu_upload.headers["Authorization"],
            "Bearer tenant-token"
        );

        let telegram = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::Telegram)
            .unwrap();
        assert_eq!(
            telegram.body_kind,
            NotificationHttpBodyKind::MultipartFormData
        );
        let telegram_body = String::from_utf8_lossy(&telegram.body);
        assert!(telegram_body.contains("name=\"chat_id\""));
        assert!(telegram_body.contains("name=\"caption\""));
        assert!(telegram_body.contains("name=\"photo\"; filename=\"image.png\""));
        assert!(telegram_body
            .as_bytes()
            .windows(4)
            .any(|window| window == [1, 2, 3, 4]));

        let discord = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::DiscordWebhook)
            .unwrap();
        assert_eq!(
            discord.body_kind,
            NotificationHttpBodyKind::MultipartFormData
        );
        let discord_body = String::from_utf8_lossy(&discord.body);
        assert!(discord_body.contains("name=\"payload_json\""));
        assert!(discord_body.contains("attachment://screenshot.png"));
        assert!(discord_body.contains("name=\"files[0]\"; filename=\"screenshot.png\""));
    }

    #[test]
    fn notification_telegram_and_discord_requests_match_legacy_text_shapes() {
        let config = NotificationConfig {
            telegram_notification_enabled: true,
            telegram_bot_token: "token".to_string(),
            telegram_chat_id: "chat".to_string(),
            telegram_api_base_url: "telegram.example/api".to_string(),
            discord_webhook_notification_enabled: true,
            discord_webhook_url: "https://discord.example/webhook".to_string(),
            discord_webhook_username: "BetterGI".to_string(),
            discord_webhook_avatar_url: "https://example.com/avatar.png".to_string(),
            ..NotificationConfig::default()
        };
        let mut payload = NotificationPayload::fail("js.error", "stack");
        payload.timestamp_ms = Some(1_700_000_000_000);
        let mut client = RecordingNotificationHttpClient::with_responses(vec![
            NotificationHttpResponse {
                status: 200,
                body: r#"{"ok":true}"#.to_string(),
            },
            NotificationHttpResponse {
                status: 204,
                body: String::new(),
            },
        ]);

        let execution = execute_notification_dispatch(&config, payload, &mut client);

        assert!(execution
            .deliveries
            .iter()
            .all(|delivery| delivery.status == NotificationProviderDeliveryStatus::Sent));
        let telegram = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::Telegram)
            .unwrap();
        assert_eq!(
            telegram.url,
            "https://telegram.example/api/bottoken/sendMessage"
        );
        let telegram_body: Value = serde_json::from_slice(&telegram.body).unwrap();
        assert_eq!(telegram_body["chat_id"], json!("chat"));
        assert_eq!(telegram_body["text"], json!("stack"));
        assert_eq!(telegram_body["disable_web_page_preview"], json!(true));

        let discord = client
            .requests
            .iter()
            .find(|request| request.provider == NotificationProviderKind::DiscordWebhook)
            .unwrap();
        assert_eq!(
            discord.url,
            "https://discord.example/webhook?with_components=true"
        );
        let discord_body: Value = serde_json::from_slice(&discord.body).unwrap();
        assert_eq!(discord_body["flags"], json!(1 << 15));
        assert_eq!(discord_body["username"], json!("BetterGI"));
        assert_eq!(
            discord_body["avatar_url"],
            json!("https://example.com/avatar.png")
        );
        assert_eq!(
            discord_body["components"][0]["components"][0]["content"],
            json!("stack")
        );
    }
}
