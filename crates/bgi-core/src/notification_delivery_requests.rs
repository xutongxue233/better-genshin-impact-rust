use super::*;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use serde_json::{json, Value};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn one_bot_requests(
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

pub(crate) fn work_weixin_requests(
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

pub(crate) fn bark_requests(
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

pub(crate) fn dingding_requests(
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

pub(crate) fn telegram_requests(
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

pub(crate) fn xxtui_requests(
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

pub(crate) fn discord_requests(
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

pub(crate) fn server_chan_requests(
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

pub(crate) fn meow_requests(
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

pub(crate) fn one_bot_request(
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

pub(crate) fn work_weixin_image_request(
    endpoint: &str,
    image: &NotificationImage,
) -> std::result::Result<NotificationHttpRequest, NotificationDispatchError> {
    json_request(
        NotificationProviderKind::WorkWeixin,
        endpoint,
        json!({
            "msgtype": "image",
            "image": {
                "base64": image.base64(),
                "md5": hex_lower(Md5::digest(&image.bytes).as_slice())
            }
        }),
    )
}
