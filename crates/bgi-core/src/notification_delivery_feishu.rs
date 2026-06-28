use super::super::{
    NotificationConfig, NotificationDispatchError, NotificationHttpClient, NotificationImage,
    NotificationPayload, NotificationProviderDelivery, NotificationProviderDeliveryStatus,
    NotificationProviderKind, NotificationProviderPlan,
};
use super::helpers::{
    json_request, multipart_request_with_headers, payload_message, required,
    NotificationMultipartFile,
};
use super::{parse_required_json, validate_notification_response};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(crate) const FEISHU_ACCESS_TOKEN_URL: &str =
    "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
pub(crate) const FEISHU_UPLOAD_IMAGE_URL: &str = "https://open.feishu.cn/open-apis/im/v1/images";

pub(super) fn execute_feishu_image_provider<C: NotificationHttpClient>(
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

pub(super) fn feishu_image_dispatch<C: NotificationHttpClient>(
    config: &NotificationConfig,
    payload: &NotificationPayload,
    client: &mut C,
) -> std::result::Result<usize, NotificationDispatchError> {
    let endpoint = required(
        &config.feishu_webhook_url,
        "Feishu webhook endpoint is not set",
    )?;
    let image =
        payload
            .screenshot
            .as_ref()
            .ok_or(NotificationDispatchError::UnsupportedPayload(
                "Feishu image upload",
            ))?;
    let token = feishu_access_token(config, client)?;
    let image_key = feishu_upload_image(image, &token, client)?;
    let message = feishu_post_image_message(payload, &image_key);
    let request = json_request(NotificationProviderKind::Feishu, endpoint, message)?;
    let response = client.send(&request)?;
    validate_notification_response(NotificationProviderKind::Feishu, "Feishu", &response)?;
    Ok(3)
}

pub(super) fn feishu_access_token<C: NotificationHttpClient>(
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

pub(super) fn feishu_upload_image<C: NotificationHttpClient>(
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

pub(super) fn feishu_post_image_message(payload: &NotificationPayload, image_key: &str) -> Value {
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
