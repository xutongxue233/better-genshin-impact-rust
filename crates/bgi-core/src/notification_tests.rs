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
        dingding_webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=abc".to_string(),
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
