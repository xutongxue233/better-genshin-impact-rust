use super::*;

pub(crate) fn script_host_runtime(json: bool) -> Result<()> {
    let demo_root = std::env::temp_dir().join("bgi-cli-host-runtime-demo");
    let _ = fs::remove_dir_all(&demo_root);
    fs::create_dir_all(demo_root.join("assets"))?;
    fs::create_dir_all(demo_root.join("strategy"))?;
    BgrImage::new(Size::new(2, 2), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])?
        .write_png(demo_root.join("assets").join("avatar.png"))?;

    let mut config = ScriptHostRuntimeConfig::new(&demo_root, demo_root.join("strategy"));
    config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
    let mut runtime = ScriptHostRuntime::new(config)?;
    let mat_payload = serde_json::json!({
        "width": 2,
        "height": 2,
        "pixelFormat": "BGR24",
        "pixels": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    });
    let calls = [
        ScriptHostCall::new(
            ScriptHostTarget::Global,
            "KeyPress",
            vec![serde_json::json!("VK_F")],
        ),
        ScriptHostCall::new(ScriptHostTarget::Global, "GetVersion", Vec::new()),
        ScriptHostCall::new(ScriptHostTarget::Global, "CaptureGameRegion", Vec::new()),
        ScriptHostCall::new(ScriptHostTarget::Global, "GetAvatars", Vec::new()),
        ScriptHostCall::new(
            ScriptHostTarget::File,
            "ReadImageMatWithResizeSync",
            vec![
                serde_json::json!("assets/avatar.png"),
                serde_json::json!(128),
                serde_json::json!(128),
                serde_json::json!(1),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::File,
            "WriteImageSync",
            vec![serde_json::json!("output/avatar"), mat_payload],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::CustomHostFunctions,
            "NewVarOfArr",
            vec![
                serde_json::json!("OpenCvSharp.Point2f"),
                serde_json::json!(2),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::PostMessage,
            "KeyPress",
            vec![serde_json::json!("VK_F")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Log,
            "Info",
            vec![serde_json::json!("host runtime ready")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Notification,
            "Send",
            vec![serde_json::json!("host runtime ready")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Http,
            "Request",
            vec![
                serde_json::json!("POST"),
                serde_json::json!("https://example.com/api"),
                serde_json::json!("{\"ok\":true}"),
                serde_json::json!("{\"Content-Type\":\"application/json\",\"X-Test\":\"1\"}"),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Dispatcher,
            "AddTrigger",
            vec![serde_json::json!({
                "name": "AutoPick",
                "interval": 50,
                "config": { "enabled": true }
            })],
        ),
        ScriptHostCall::new(ScriptHostTarget::Genshin, "Uid", Vec::new()),
        ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "Tp",
            vec![
                serde_json::json!("100.5"),
                serde_json::json!(200.25),
                serde_json::json!(true),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "SwitchParty",
            vec![serde_json::json!("default")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "SetTime",
            vec![
                serde_json::json!(8),
                serde_json::json!("30"),
                serde_json::json!(true),
            ],
        ),
        ScriptHostCall::new(ScriptHostTarget::Genshin, "ReturnMainUi", Vec::new()),
        ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "Run",
            vec![serde_json::json!(
                r#"{
                  "info": {
                    "name": "host runtime route",
                    "type": "collect",
                    "map_name": "Teyvat"
                  },
                  "positions": [
                    { "x": 100.0, "y": 200.0, "type": "path", "move_mode": "dash" }
                  ]
                }"#
            )],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Show",
            vec![
                serde_json::json!("overlay.html"),
                serde_json::json!("demo-mask"),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Send",
            vec![
                serde_json::json!("demo-mask"),
                serde_json::json!("/status"),
                serde_json::json!("{\"ready\":true}"),
            ],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "FlushPendingMessages",
            vec![serde_json::json!("demo-mask")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "ToggleClickThrough",
            vec![serde_json::json!("demo-mask")],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "OnKeyDown",
            vec![serde_json::json!("key-down"), serde_json::json!(true)],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "OnMouseMove",
            vec![serde_json::json!("mouse-move"), serde_json::json!(50)],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "DispatchEvent",
            vec![serde_json::json!({
                "type": "keyDown",
                "keyData": "Control, F",
                "keyCode": "F"
            })],
        ),
        ScriptHostCall::new(
            ScriptHostTarget::ServerTime,
            "GetServerTimeZoneOffset",
            Vec::new(),
        ),
    ];

    let mut results = Vec::new();
    for call in calls {
        let target = call.target;
        let method = call.method.clone();
        let result = runtime.call(call)?;
        results.push(serde_json::json!({
            "target": target,
            "method": method,
            "result": result,
        }));
    }

    let http_host = HttpHost::new(ScriptHttpPolicy::new(
        true,
        ["https://example.com/*".to_string()],
    ));
    let mut http_client = RecordingHttpClient::ok_json("{\"ok\":true}");
    let http_response = http_host.execute_request(
        "GET",
        "https://example.com/status",
        None,
        Some("{\"Accept\":\"application/json\"}"),
        &mut http_client,
    )?;

    let mut notification_host =
        ScriptNotificationHost::new(ScriptNotificationPolicy::new(true, true));
    let mut notification_sink = RecordingNotificationSink::default();
    let notification_delivery =
        notification_host.send_to("host runtime ready", 1, &mut notification_sink)?;

    let payload = serde_json::json!({
        "results": results,
        "http_response": http_response,
        "http_requests": http_client.requests(),
        "metrics": runtime.game_metrics(),
        "log_records": runtime.log_records(),
        "notification_records": runtime.notification_records(),
        "notification_delivery": notification_delivery,
        "notification_deliveries": notification_sink.deliveries(),
        "dispatcher_commands": runtime.dispatcher_commands(),
        "dispatcher_task_invocations": runtime.dispatcher_task_invocation_plans()?,
        "genshin_commands": runtime.genshin_commands(),
        "genshin_task_invocations": runtime.genshin_task_invocation_plans()?,
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "calls: {}",
            payload["results"].as_array().map_or(0, Vec::len)
        );
        println!(
            "logs: {}",
            payload["log_records"].as_array().map_or(0, Vec::len)
        );
        println!(
            "notifications: {}",
            payload["notification_records"]
                .as_array()
                .map_or(0, Vec::len)
        );
        println!(
            "genshin_commands: {}",
            payload["genshin_commands"].as_array().map_or(0, Vec::len)
        );
    }
    Ok(())
}
