use super::*;
use image::codecs::jpeg::JpegEncoder;
use serde_json::{json, Value};
use std::io::Cursor;

#[path = "notification_delivery_feishu.rs"]
mod feishu;
#[path = "notification_delivery_helpers.rs"]
mod helpers;
#[path = "notification_delivery_requests.rs"]
mod requests;

#[cfg(test)]
pub(super) use feishu::{FEISHU_ACCESS_TOKEN_URL, FEISHU_UPLOAD_IMAGE_URL};
use helpers::*;
pub(super) use requests::*;

pub(super) fn execute_email_provider<E: NotificationEmailClient>(
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

pub(super) fn execute_windows_toast_provider<T: NotificationWindowsToastClient>(
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

pub(super) fn windows_toast_request(
    payload: &NotificationPayload,
) -> std::result::Result<NotificationWindowsToastRequest, NotificationDispatchError> {
    Ok(NotificationWindowsToastRequest {
        event: payload.event.clone(),
        message: payload.message.clone(),
        screenshot: payload.screenshot.clone(),
        expiration_hours: 12,
    })
}

pub(super) fn execute_web_socket_provider<W: NotificationWebSocketClient>(
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

pub(super) fn execute_feishu_image_provider<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    provider: &NotificationProviderPlan,
    client: &mut C,
) -> NotificationProviderDelivery {
    feishu::execute_feishu_image_provider(config, payload, provider, client)
}

pub(super) fn validate_notification_response(
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

pub(super) fn parse_optional_json(
    body: &str,
) -> std::result::Result<Option<Value>, NotificationDispatchError> {
    if body.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(body)
        .map(Some)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))
}

pub(super) fn parse_required_json(
    body: &str,
) -> std::result::Result<Value, NotificationDispatchError> {
    parse_optional_json(body)?
        .ok_or_else(|| NotificationDispatchError::Transport("empty JSON response".to_string()))
}

pub(super) fn webhook_requests(
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

pub(super) fn feishu_requests(
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

pub(super) fn web_socket_payload(
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

pub(super) fn email_request(
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

pub(super) fn email_security(server: &str, port: u16) -> NotificationEmailSecurity {
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

pub(super) fn email_screenshot_attachment(
    image: &NotificationImage,
) -> NotificationEmailAttachment {
    NotificationEmailAttachment {
        file_name: "screenshot.jpg".to_string(),
        content_id: Some("screenshot".to_string()),
        content_type: "image/jpeg".to_string(),
        bytes: image_as_jpeg(image).unwrap_or_else(|| image.bytes.clone()),
    }
}

pub(super) fn image_as_jpeg(image: &NotificationImage) -> Option<Vec<u8>> {
    let dynamic = image::load_from_memory(&image.bytes).ok()?;
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    JpegEncoder::new_with_quality(&mut cursor, 90)
        .encode_image(&dynamic)
        .ok()?;
    Some(bytes)
}

pub(super) fn email_body(payload: &NotificationPayload) -> String {
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

pub(super) fn push_email_field(body: &mut String, name: &str, value: &str) {
    body.push_str("<p><strong>");
    body.push_str(&html_escape(name));
    body.push_str(":</strong> ");
    body.push_str(&html_escape(value));
    body.push_str("</p>\n");
}

pub(super) fn html_escape(value: &str) -> String {
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
