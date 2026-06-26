use crate::config::NotificationConfig;

use super::notification_delivery::*;
use super::notification_model::{
    NotificationDispatchError, NotificationDispatchExecution, NotificationEmailClient,
    NotificationHttpClient, NotificationHttpRequest, NotificationPayload,
    NotificationProviderDelivery, NotificationProviderDeliveryStatus, NotificationProviderKind,
    NotificationProviderPlan, NotificationWebSocketClient, NotificationWindowsToastClient,
    UnsupportedNotificationEmailClient, UnsupportedNotificationWebSocketClient,
    UnsupportedNotificationWindowsToastClient,
};
use super::notification_plans::notification_dispatch_plan;

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
    plan: &super::notification_model::NotificationDispatchPlan,
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
