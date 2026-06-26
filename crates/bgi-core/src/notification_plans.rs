use crate::config::NotificationConfig;
use serde_json::{json, Value};

use super::notification_events::should_send_notification;
use super::notification_model::{
    NotificationDispatchPlan, NotificationPayload, NotificationProviderKind,
    NotificationProviderPlan,
};

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

pub(super) fn provider(
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

pub(super) fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn is_configured(value: &str) -> bool {
    !value.trim().is_empty()
}
