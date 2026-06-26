use super::super::{
    non_empty, NotificationDispatchError, NotificationEventResult, NotificationHttpBodyKind,
    NotificationHttpRequest, NotificationImage, NotificationPayload, NotificationProviderKind,
};
use crate::config::NotificationConfig;
use chrono::{DateTime, Local, TimeZone};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NotificationMultipartFile {
    field_name: String,
    file_name: String,
    mime_type: String,
    bytes: Vec<u8>,
}

impl NotificationMultipartFile {
    pub(super) fn new(field_name: impl Into<String>, image: NotificationImage) -> Self {
        Self {
            field_name: field_name.into(),
            file_name: image.file_name,
            mime_type: image.mime_type,
            bytes: image.bytes,
        }
    }

    pub(super) fn with_file_name(
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

pub(super) fn json_request(
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

pub(super) fn multipart_request(
    provider: NotificationProviderKind,
    url: impl Into<String>,
    fields: Vec<(&str, String)>,
    files: Vec<NotificationMultipartFile>,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    multipart_request_with_headers(provider, url, fields, files, BTreeMap::new())
}

pub(super) fn multipart_request_with_headers(
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

pub(super) fn form_request(
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

pub(super) fn push_multipart_field(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

pub(super) fn push_multipart_file(
    body: &mut Vec<u8>,
    boundary: &str,
    file: &NotificationMultipartFile,
) {
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

pub(super) fn multipart_boundary(provider: NotificationProviderKind) -> String {
    format!("----bettergi-rust-{provider:?}")
}

pub(super) fn required<'a>(
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

pub(super) fn payload_message(payload: &NotificationPayload) -> String {
    payload.message.clone().unwrap_or_default()
}

pub(super) fn payload_has_screenshot(payload: &NotificationPayload) -> bool {
    payload.screenshot.is_some() || payload.has_screenshot
}

pub(super) fn legacy_result(result: NotificationEventResult) -> &'static str {
    match result {
        NotificationEventResult::Success => "Success",
        NotificationEventResult::Fail => "Fail",
        NotificationEventResult::PartialSuccess => "PartialSuccess",
    }
}

pub(super) fn payload_timestamp_string(payload: &NotificationPayload) -> String {
    payload
        .timestamp_ms
        .and_then(|timestamp| Local.timestamp_millis_opt(timestamp as i64).single())
        .map(|time: DateTime<Local>| time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}

pub(super) fn one_bot_send_msg_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim_end_matches('/');
    if trimmed.ends_with("/send_msg") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/send_msg")
    }
}

pub(super) fn split_bark_device_keys(keys: &str) -> Vec<String> {
    keys.split([',', ';', ' ']).filter_map(non_empty).collect()
}

pub(super) fn bark_payload(config: &NotificationConfig, payload: &NotificationPayload) -> Value {
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

pub(super) fn insert_non_empty(map: &mut serde_json::Map<String, Value>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        map.insert(key.to_string(), json!(value));
    }
}

pub(super) fn bark_body(payload: &NotificationPayload) -> String {
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

pub(super) fn telegram_base_url(api_base_url: &str) -> String {
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

pub(super) fn discord_screenshot_file_name(
    config: &NotificationConfig,
    image: &NotificationImage,
) -> String {
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

pub(super) fn ensure_url_scheme(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://{value}")
    }
}

pub(super) fn server_chan_url(
    send_key: &str,
) -> std::result::Result<String, NotificationDispatchError> {
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

pub(super) fn server_chan_description(payload: &NotificationPayload) -> String {
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

pub(super) fn meow_message(payload: &NotificationPayload) -> String {
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

pub(super) fn append_query(base: &str, params: &[(&str, &str)]) -> String {
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

pub(super) fn form_urlencoded(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) fn percent_encode(value: &str) -> String {
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

pub(super) fn percent_encode_path_segment(value: &str) -> String {
    percent_encode(value)
}

pub(super) fn hex_lower(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
