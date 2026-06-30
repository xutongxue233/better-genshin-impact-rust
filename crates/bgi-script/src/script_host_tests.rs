use super::*;
use crate::r#macro::MacroPlaybackContext;
use crate::{
    InputCancellationToken, ScriptHostPolicyError, ScriptHttpPolicy, ScriptNotificationPolicy,
};
use bgi_capture::CaptureFrame;
use bgi_input::{InputEvent, MouseButton, PostMessageEvent};
use bgi_task::{
    CommonJobFrameSource, CommonJobInputDriver, CommonJobRuntimeOutcome, ReloginPlatformDriver,
};
use bgi_vision::{BgrImage, RecognitionType, Rect, Size as VisionSize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

struct StaticFrameSource {
    frame: CaptureFrame,
    area: Option<GameCaptureArea>,
}

impl GameCaptureFrameSource for StaticFrameSource {
    fn capture_frame(&self) -> Result<CaptureFrame> {
        Ok(self.frame.clone())
    }

    fn capture_frame_area(&self, frame: &CaptureFrame) -> GameCaptureArea {
        self.area.unwrap_or(GameCaptureArea {
            x: 0,
            y: 0,
            width: frame.size.width,
            height: frame.size.height,
        })
    }
}

#[test]
fn inline_key_mouse_script_builds_input_plan() {
    let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
    let plan = host
        .run(
            r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
        )
        .unwrap();

    assert_eq!(plan.source, KeyMouseScriptSource::InlineJson);
    assert_eq!(plan.summary.event_count, 2);
    assert_eq!(
        plan.input_events,
        vec![
            InputEvent::Delay { milliseconds: 10 },
            InputEvent::KeyDown {
                vk: 87,
                extended: None
            },
            InputEvent::Delay { milliseconds: 20 },
            InputEvent::KeyUp {
                vk: 87,
                extended: None
            }
        ]
    );
}

#[test]
fn key_mouse_script_plan_only_execution_reports_events_without_dispatch() {
    let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
    let execution = host
        .execute(
            r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
            KeyMouseScriptDispatchMode::PlanOnly,
            None,
        )
        .unwrap();

    assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
    assert!(!execution.dispatched);
    assert_eq!(execution.dispatched_events, 0);
    assert_eq!(execution.plan.summary.event_count, 2);
    assert_eq!(execution.plan.input_events.len(), 4);
}

#[test]
fn key_mouse_script_send_input_honors_pre_cancelled_token() {
    let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
    let cancellation = InputCancellationToken::new();
    cancellation.cancel();

    let execution = host
        .execute_with_cancellation(
            r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
            KeyMouseScriptDispatchMode::SendInput,
            None,
            Some(&cancellation),
        )
        .unwrap();

    assert_eq!(execution.mode, KeyMouseScriptDispatchMode::SendInput);
    assert!(execution.dispatched);
    assert!(execution.cancelled);
    assert_eq!(execution.dispatched_events, 0);
    assert_eq!(execution.processed_events, 0);
    assert_eq!(execution.plan.input_events.len(), 4);
}

#[test]
fn run_file_uses_script_file_policy_root() {
    let root = std::env::temp_dir().join(format!(
        "bgi-keymouse-host-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let script_path = root.join("macro.json");
    fs::write(
        &script_path,
        r#"{
              "macroEvents": [
                { "type": 4, "mouseButton": "Left", "mouseX": 16, "mouseY": 16, "time": 0 }
              ]
            }"#,
    )
    .unwrap();

    let host = KeyMouseScriptHost::new(&root, MacroPlaybackContext::default());
    let plan = host.run_file("macro.json").unwrap();

    assert_eq!(plan.source, KeyMouseScriptSource::File);
    assert_eq!(plan.normalized_path, Some(script_path));
    assert_eq!(
        plan.input_events,
        vec![
            InputEvent::MouseMoveAbsolute {
                x: 546,
                y: 970,
                virtual_desktop: false
            },
            InputEvent::MouseButtonDown {
                button: MouseButton::Left
            }
        ]
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn run_file_rejects_path_traversal() {
    let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
    let error = host.run_file("../macro.json").unwrap_err();

    assert!(matches!(
        error,
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
    ));
}

#[test]
fn global_input_host_maps_keyboard_and_mouse_keys() {
    let host = GlobalInputHost::new(
        GameCaptureArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        1.0,
    )
    .unwrap();

    assert_eq!(
        host.key_press("VK_F").unwrap().events(),
        &[
            InputEvent::KeyDown {
                vk: 0x46,
                extended: None
            },
            InputEvent::KeyUp {
                vk: 0x46,
                extended: None
            }
        ]
    );
    assert_eq!(
        host.key_down("VK_LBUTTON").unwrap().events(),
        &[InputEvent::MouseButtonDown {
            button: MouseButton::Left
        }]
    );
}

#[test]
fn global_input_host_tracks_game_metrics_and_coordinates() {
    let mut host = GlobalInputHost::new(
        GameCaptureArea {
            x: 100,
            y: 50,
            width: 1280,
            height: 720,
        },
        2.0,
    )
    .unwrap();

    host.set_game_metrics(1920, 1080, 1.0).unwrap();

    assert_eq!(
        host.move_mouse_to(960, 540).unwrap().events(),
        &[InputEvent::MouseMoveAbsolute {
            x: 740,
            y: 410,
            virtual_desktop: false
        }]
    );
    assert_eq!(
        host.move_mouse_by(10, -5).events(),
        &[InputEvent::MouseMoveRelative { dx: 20, dy: -10 }]
    );
}

#[test]
fn common_job_frame_source_converts_capture_frame_to_bgr_image() {
    let frame = CaptureFrame::packed_bgr(2, 1, vec![11, 22, 33, 44, 55, 66]).unwrap();
    let host = GlobalInputHost::new_with_frame_source(
        GameCaptureArea {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        },
        1.0,
        Some(Arc::new(StaticFrameSource { frame, area: None })),
    )
    .unwrap();

    let mut source = host.common_job_frame_source().unwrap();
    let image = source.capture_frame().unwrap();

    assert_eq!(image.size, VisionSize::new(2, 1));
    assert_eq!(image.pixels, vec![11, 22, 33, 44, 55, 66]);
}

#[test]
fn common_job_input_driver_maps_capture_click_to_global_input_sequence() {
    let mut host = GlobalInputHost::new(
        GameCaptureArea {
            x: 100,
            y: 200,
            width: 960,
            height: 540,
        },
        1.0,
    )
    .unwrap();
    host.set_game_metrics(1920, 1080, 1.0).unwrap();

    let mut driver = host.common_job_input_driver(GlobalInputDispatchMode::PlanOnly, None);
    driver.click_capture_point(960, 540).unwrap();

    let executions = driver.executions();
    assert_eq!(executions.len(), 1);
    assert!(!executions[0].dispatched);
    assert_eq!(
        executions[0].events,
        vec![
            InputEvent::MouseMoveAbsolute {
                x: 580,
                y: 470,
                virtual_desktop: false
            },
            InputEvent::MouseButtonDown {
                button: MouseButton::Left
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Left
            }
        ]
    );
}

#[test]
fn common_job_input_driver_maps_capture_absolute_moves_to_screen_coordinates() {
    let mut host = GlobalInputHost::new(
        GameCaptureArea {
            x: 100,
            y: 200,
            width: 960,
            height: 540,
        },
        1.0,
    )
    .unwrap();
    host.set_game_metrics(1920, 1080, 1.0).unwrap();

    let mut driver = host.common_job_input_driver(GlobalInputDispatchMode::PlanOnly, None);
    driver
        .dispatch_capture_input(&[
            InputEvent::MouseMoveAbsolute {
                x: 960,
                y: 540,
                virtual_desktop: false,
            },
            InputEvent::MouseButtonDown {
                button: MouseButton::Left,
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            },
        ])
        .unwrap();

    let executions = driver.executions();
    assert_eq!(executions.len(), 1);
    assert_eq!(
        executions[0].events,
        vec![
            InputEvent::MouseMoveAbsolute {
                x: 580,
                y: 470,
                virtual_desktop: false
            },
            InputEvent::MouseButtonDown {
                button: MouseButton::Left
            },
            InputEvent::MouseButtonUp {
                button: MouseButton::Left
            }
        ]
    );
}

#[test]
fn common_job_relogin_platform_driver_focus_requires_window_and_non_bili_probe_noops() {
    let host = GlobalInputHost::new(
        GameCaptureArea {
            x: 100,
            y: 200,
            width: 960,
            height: 540,
        },
        1.0,
    )
    .unwrap();

    let mut driver = host.common_job_input_driver(GlobalInputDispatchMode::PlanOnly, None);
    let focus_error = driver.focus_game_window().unwrap_err();
    assert!(focus_error
        .to_string()
        .contains("requires a game window handle"));

    let mut rule = bgi_task::plan_relogin(VisionSize::new(1920, 1080))
        .unwrap()
        .third_party_rule;
    rule.bilibili_only = false;
    let outcome = driver.execute_third_party_login_probe(&rule).unwrap();
    assert_eq!(outcome, CommonJobRuntimeOutcome::Matched(true));
}

#[test]
fn bilibili_config_text_channel_detection_matches_legacy_suffix_rule() {
    assert!(bilibili_config_text_has_channel_14("channel=14"));
    assert!(bilibili_config_text_has_channel_14(
        "foo=bar\n  channel=hk4e_14\n"
    ));
    assert!(!bilibili_config_text_has_channel_14("channel=1"));
    assert!(!bilibili_config_text_has_channel_14("Channel=14"));
}

#[test]
fn relogin_dpi_aware_coordinate_applies_runtime_dpi_offset() {
    assert_eq!(relogin_dpi_aware_coordinate(960.0, 70.0, 1.0), 1030);
    assert_eq!(relogin_dpi_aware_coordinate(540.0, 75.0, 1.5), 653);
}

#[test]
fn global_input_host_rejects_non_16_by_9_metrics() {
    let mut host = GlobalInputHost::new(
        GameCaptureArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        1.0,
    )
    .unwrap();

    let error = host.set_game_metrics(1024, 768, 1.0).unwrap_err();
    assert!(matches!(
        error,
        ScriptHostRuntimeError::InvalidGameMetrics {
            width: 1024,
            height: 768
        }
    ));
}

#[test]
fn global_input_text_uses_unicode_events() {
    let host = GlobalInputHost::new(
        GameCaptureArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        },
        1.0,
    )
    .unwrap();

    assert_eq!(
        host.input_text("GI").events(),
        &[
            InputEvent::UnicodeChar { ch: 'G' },
            InputEvent::UnicodeChar { ch: 'I' }
        ]
    );
}

#[test]
fn key_mouse_hook_host_dispatches_registered_listeners() {
    let mut host = KeyMouseHookHost::default();
    host.on_key_down(Some("key-code"), true);
    host.on_key_down(Some("key-data"), false);
    host.on_mouse_down(Some("mouse-down"));
    host.on_mouse_move(Some("move-fast"), 50);
    host.on_mouse_move(Some("move-slow"), 200);
    host.on_mouse_wheel(Some("wheel"));

    let key_dispatches = host.dispatch_event(KeyMouseHookEvent::Key {
        event: KeyMouseHookEventKind::KeyDown,
        key_data: "Control, F".to_string(),
        key_code: "F".to_string(),
    });
    assert_eq!(key_dispatches.len(), 2);
    assert_eq!(key_dispatches[0].listener_id, "key-code");
    assert_eq!(key_dispatches[0].args, vec![serde_json::json!("F")]);
    assert_eq!(key_dispatches[1].listener_id, "key-data");
    assert_eq!(
        key_dispatches[1].args,
        vec![serde_json::json!("Control, F")]
    );

    let mouse_down = host.dispatch_event(KeyMouseHookEvent::MouseButton {
        event: KeyMouseHookEventKind::MouseDown,
        button: MouseButton::Right,
        x: 12,
        y: 34,
    });
    assert_eq!(
        mouse_down[0].args,
        vec![
            serde_json::json!("Right"),
            serde_json::json!(12),
            serde_json::json!(34)
        ]
    );

    let first_move = host.dispatch_event(KeyMouseHookEvent::MouseMove {
        x: 10,
        y: 20,
        timestamp_ms: 100,
    });
    assert_eq!(first_move.len(), 2);
    let throttled_global = host.dispatch_event(KeyMouseHookEvent::MouseMove {
        x: 11,
        y: 21,
        timestamp_ms: 105,
    });
    assert!(throttled_global.is_empty());
    let fast_only = host.dispatch_event(KeyMouseHookEvent::MouseMove {
        x: 12,
        y: 22,
        timestamp_ms: 160,
    });
    assert_eq!(fast_only.len(), 1);
    assert_eq!(fast_only[0].listener_id, "move-fast");

    let wheel = host.dispatch_event(KeyMouseHookEvent::MouseWheel {
        delta: -120,
        x: 7,
        y: 8,
    });
    assert_eq!(
        wheel[0].args,
        vec![
            serde_json::json!(-120),
            serde_json::json!(7),
            serde_json::json!(8)
        ]
    );

    host.remove_all_listeners();
    assert!(host.listeners().is_empty());
    host.on_key_up(Some("key-up"), true);
    host.dispose();
    assert!(host.snapshot().disposed);
    assert!(host
        .dispatch_event(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyUp,
            key_data: "F".to_string(),
            key_code: "F".to_string(),
        })
        .is_empty());
}

#[test]
fn realtime_timer_and_solo_task_models_preserve_legacy_defaults() {
    let timer = RealtimeTimerHostPlan::new("AutoPick", None);
    assert_eq!(timer.name, "AutoPick");
    assert_eq!(timer.interval_ms, 50);
    assert_eq!(timer.config, None);

    let auto_pick = AutoPickExternalConfig::from_value(Some(&serde_json::json!({
        "textList": ["Open", "Pick"],
        "forceInteraction": true
    })))
    .unwrap();
    assert_eq!(
        auto_pick.to_legacy_config_value(),
        serde_json::json!({
            "TextList": ["Open", "Pick"],
            "ForceInteraction": true
        })
    );

    let call = ScriptHostCall::new(
        ScriptHostTarget::Dispatcher,
        "AddTrigger",
        vec![serde_json::json!({
            "name": "AutoPick",
            "config": {
                "textList": ["Open"],
                "forceInteraction": true
            }
        })],
    );
    let timer = timer_plan_from_arg(&call, 0, false).unwrap();
    assert_eq!(
        timer.config,
        Some(serde_json::json!({
            "TextList": ["Open"],
            "ForceInteraction": true
        }))
    );

    let solo = SoloTaskHostPlan::new("AutoFight", Some(serde_json::json!({"team": "daily"})));
    assert_eq!(solo.name, "AutoFight");
    assert!(solo.uses_linked_cancellation);
}

#[test]
fn limited_file_host_reads_writes_lists_and_renames() {
    let root = test_root("bgi-limited-file-host");
    let host = LimitedFileHost::new(&root);

    assert!(host.create_directory("nested").unwrap());
    assert!(host
        .write_text_sync("nested/a.txt", "hello", false)
        .unwrap());
    assert!(host
        .write_text_sync("nested/a.txt", " world", true)
        .unwrap());
    assert_eq!(host.read_text_sync("nested/a.txt").unwrap(), "hello world");
    assert!(host.is_file("nested/a.txt").unwrap());
    assert!(host.is_folder("nested").unwrap());
    assert!(host.is_exists("nested/a.txt").unwrap());

    let entries = host.read_path_sync("nested").unwrap();
    assert_eq!(entries, vec!["nested/a.txt"]);

    assert!(host
        .rename_path_sync("nested/a.txt", "nested/b.txt")
        .unwrap());
    assert!(!host.is_exists("nested/a.txt").unwrap());
    assert!(host.is_file("nested/b.txt").unwrap());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn limited_file_host_rejects_disallowed_write_extension() {
    let root = test_root("bgi-limited-file-policy");
    let host = LimitedFileHost::new(&root);

    let error = host.write_text_sync("bad.exe", "x", false).unwrap_err();

    assert!(matches!(
        error,
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::ExtensionNotAllowed(ext))
            if ext == ".exe"
    ));

    fs::remove_dir_all(root).unwrap_or(());
}

#[test]
fn limited_file_host_plans_image_mat_io() {
    let root = test_root("bgi-limited-file-image-policy");
    let host = LimitedFileHost::new(&root);

    let read = host.read_image_mat_plan_sync("assets/icon.png").unwrap();
    assert_eq!(read.normalized_path, root.join("assets/icon.png"));
    assert_eq!(read.color_mode, "color");
    assert_eq!(read.resize, None);

    let resized = host
        .read_image_mat_with_resize_plan_sync("assets/icon.webp", 64.0, 32.0, 4)
        .unwrap();
    assert_eq!(
        resized.resize,
        Some(ImageMatResizePlan {
            width: 64.0,
            height: 32.0,
            interpolation: 4
        })
    );

    let write = host
        .write_image_plan_sync("output/avatar", serde_json::json!({"matHandle": "m1"}))
        .unwrap();
    assert_eq!(write.normalized_path, root.join("output/avatar.png"));
    assert_eq!(write.source, serde_json::json!({"matHandle": "m1"}));

    let error = host
        .read_image_mat_with_resize_sync("assets/icon.png", 0.0, 32.0, 1)
        .unwrap_err();
    assert!(matches!(
        error,
        ScriptHostRuntimeError::InvalidArgument { index: 1, .. }
    ));

    fs::remove_dir_all(root).unwrap_or(());
}

#[test]
fn limited_file_host_executes_image_mat_io() {
    let root = test_root("bgi-limited-file-image-exec");
    let host = LimitedFileHost::new(&root);
    fs::create_dir_all(root.join("assets")).unwrap();
    let source = BgrImage::new(
        VisionSize::new(2, 2),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    )
    .unwrap();
    source.write_png(root.join("assets/icon.png")).unwrap();

    let read = host.read_image_mat_sync("assets/icon.png").unwrap();
    let resized = host
        .read_image_mat_with_resize_sync("assets/icon.png", 1.0, 1.0, 1)
        .unwrap();
    let write = host
        .write_image_sync(
            "output/copy",
            serde_json::json!({
                "width": read.width,
                "height": read.height,
                "pixelFormat": read.pixel_format,
                "pixels": read.pixels,
            }),
        )
        .unwrap();
    let copied = BgrImage::read(root.join("output/copy.png")).unwrap();

    assert_eq!(read.normalized_path, root.join("assets/icon.png"));
    assert_eq!(read.width, 2);
    assert_eq!(read.height, 2);
    assert_eq!(read.pixel_format, "BGR24");
    assert_eq!(read.pixels, source.pixels);
    assert_eq!(resized.width, 1);
    assert_eq!(resized.height, 1);
    assert_eq!(resized.pixels, vec![1, 2, 3]);
    assert_eq!(write.normalized_path, root.join("output/copy.png"));
    assert_eq!(write.width, 2);
    assert_eq!(write.height, 2);
    assert!(write.bytes_written > 0);
    assert_eq!(copied, source);

    fs::remove_dir_all(root).unwrap_or(());
}

#[test]
fn strategy_file_host_is_limited_to_its_root() {
    let root = test_root("bgi-strategy-file-host");
    fs::create_dir_all(root.join("teams")).unwrap();
    fs::write(root.join("teams/default.json"), "{}").unwrap();

    let host = StrategyFileHost::new(&root);

    assert!(host.is_folder("teams").unwrap());
    assert!(host.is_file("teams/default.json").unwrap());
    assert_eq!(
        host.read_path_sync("teams").unwrap(),
        vec!["teams/default.json"]
    );
    assert!(matches!(
        host.is_exists("../outside.json").unwrap_err(),
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
    ));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn log_host_records_script_log_levels() {
    let mut host = ScriptLogHost::default();
    host.debug("a");
    host.info("b");
    host.warn("c");
    host.error("d");

    assert_eq!(host.records().len(), 4);
    assert_eq!(host.records()[2].level, ScriptLogLevel::Warn);
    assert_eq!(host.records()[3].message, "d");
}

#[test]
fn notification_host_applies_policy_and_rate_limit() {
    let policy = ScriptNotificationPolicy::new(true, true);
    let mut host = ScriptNotificationHost::new(policy);

    for i in 0..5 {
        host.send_at(&format!("msg{i}"), i).unwrap();
    }
    let error = host.error_at("too fast", 10).unwrap_err();
    assert!(matches!(
        error,
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::NotificationRateLimited)
    ));

    assert_eq!(host.records().len(), 5);
}

#[test]
fn notification_host_delivers_to_sink_with_legacy_event_codes() {
    let policy = ScriptNotificationPolicy::new(true, true);
    let mut host = ScriptNotificationHost::new(policy);
    let mut sink = RecordingNotificationSink::default();

    let delivery = host.send_to("ready", 100, &mut sink).unwrap();
    assert_eq!(delivery.event_code, "js.custom");
    assert_eq!(delivery.result, "success");
    assert_eq!(delivery.message, "ready");

    let delivery = host.error_to("failed", 101, &mut sink).unwrap();
    assert_eq!(delivery.event_code, "js.error");
    assert_eq!(delivery.result, "fail");
    assert_eq!(sink.deliveries().len(), 2);
    assert_eq!(host.records().len(), 2);
}

#[test]
fn http_host_builds_request_plan_and_normalizes_headers() {
    let host = HttpHost::new(ScriptHttpPolicy::new(
        true,
        ["https://example.com/*".to_string()],
    ));

    let plan = host
        .request(
            "post",
            "https://example.com/api",
            Some("{\"ok\":true}"),
            Some(r#"{"Content-Type":"text/plain","X-Test":"1"}"#),
        )
        .unwrap();

    assert_eq!(plan.method, "POST");
    assert_eq!(plan.body, Some("{\"ok\":true}".to_string()));
    assert_eq!(plan.content_type, "text/plain");
    assert_eq!(plan.headers, vec![("x-test".to_string(), "1".to_string())]);
    assert!(matches!(
        host.request("GET", "https://blocked.example/api", None, None)
            .unwrap_err(),
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::HttpUrlDenied(_))
    ));
    assert!(matches!(
        host.request("GET", "https://example.com/api", None, Some("[]"))
            .unwrap_err(),
        ScriptHostRuntimeError::InvalidHttpHeaders
    ));
}

#[test]
fn http_host_executes_request_through_pluggable_client() {
    let host = HttpHost::new(ScriptHttpPolicy::new(
        true,
        ["https://example.com/*".to_string()],
    ));
    let mut client = RecordingHttpClient::ok_json("{\"ok\":true}");

    let response = host
        .execute_request(
            "post",
            "https://example.com/api",
            Some("{\"ok\":true}"),
            Some(r#"{"Content-Type":"text/plain","X-Test":"1"}"#),
            &mut client,
        )
        .unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, "{\"ok\":true}");
    assert_eq!(
        response.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    assert_eq!(client.requests().len(), 1);
    assert_eq!(client.requests()[0].method, "POST");
    assert_eq!(client.requests()[0].content_type, "text/plain");
    assert_eq!(
        client.requests()[0].headers,
        vec![("x-test".to_string(), "1".to_string())]
    );
}

#[test]
fn html_mask_host_plans_windows_and_message_queues() {
    let root = test_root("bgi-html-mask-host");
    fs::write(root.join("overlay.html"), "<html></html>").unwrap();
    let mut host = HtmlMaskHost::new(&root);

    let command = host.show("overlay.html", Some("mask")).unwrap();
    let HtmlMaskCommand::Show(plan) = command else {
        panic!("expected show command");
    };
    assert_eq!(plan.window_id, "mask");
    assert!(plan.final_url.starts_with("file:///"));
    assert!(plan.normalized_path.unwrap().ends_with("overlay.html"));
    assert!(host.exists("mask"));
    assert_eq!(host.window_ids(), vec!["mask".to_string()]);

    host.show("https://example.com/widget", Some("remote"))
        .unwrap();
    assert_eq!(
        host.snapshot()
            .windows
            .iter()
            .find(|window| window.window_id == "remote")
            .unwrap()
            .final_url,
        "https://example.com/widget"
    );

    let send = host.send("mask", "/status", r#"{"ready":true}"#).unwrap();
    assert!(matches!(
        send,
        HtmlMaskCommand::Send {
            ref window_id,
            ref message,
        } if window_id == "mask" && message.data.as_ref().unwrap()["ready"] == true
    ));
    let send_json = serde_json::to_string(&send).unwrap();
    assert!(send_json.contains("\"window_id\":\"mask\""));
    assert!(!send_json.contains("request_id"));
    assert!(!send_json.contains("requestId"));
    let pending = host.flush_pending_messages("mask").unwrap();
    assert_eq!(pending.len(), 1);
    assert!(pending[0].contains("\"ready\":true"));

    let request = host
        .request("mask", "/status", r#"{"wait":true}"#, 250)
        .unwrap();
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("\"requestId\":\"request-1\""));
    assert!(request_json.contains("\"timeout_ms\":250"));
    assert_eq!(host.flush_pending_messages("mask").unwrap().len(), 1);

    host.send_from_html("mask", "/event", r#"{"ok":true}"#, None)
        .unwrap();
    assert_eq!(
        host.poll("mask").unwrap(),
        Some(r#"{"url":"/event","data":{"ok":true}}"#.to_string())
    );
    assert_eq!(host.poll("mask").unwrap(), None);

    host.send_from_html("mask", "/a", "plain text", Some("response-1"))
        .unwrap();
    let all = host.poll_all("mask").unwrap();
    assert!(all.contains("\"data\":\"plain text\""));
    assert!(all.contains("\"requestId\":\"response-1\""));

    host.toggle_click_through("mask").unwrap();
    assert!(host.get_click_through("mask").unwrap());
    host.close("mask");
    assert!(!host.exists("mask"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn html_mask_host_can_resume_from_desktop_bridge_state() {
    let root = test_root("bgi-html-mask-resume");
    fs::write(root.join("overlay.html"), "<html></html>").unwrap();
    let plan = HtmlMaskWindowPlan {
        window_id: "mask".to_string(),
        final_url: path_to_file_url(&root.join("overlay.html")),
        requested_url: "overlay.html".to_string(),
        normalized_path: Some(root.join("overlay.html")),
        click_through: true,
    };
    let message = HtmlMaskMessage {
        url: "/event".to_string(),
        data: Some(serde_json::json!({ "ok": true })),
        request_id: Some("req-1".to_string()),
    };
    let mut host = HtmlMaskHost::with_initial_state(
        &root,
        HtmlMaskInitialState {
            windows: vec![plan],
            from_html: vec![("mask".to_string(), message)],
        },
    );

    assert!(host.exists("mask"));
    assert!(host.get_click_through("mask").unwrap());
    let polled = host.poll("mask").unwrap().unwrap();
    assert!(polled.contains("\"url\":\"/event\""));
    assert!(polled.contains("\"requestId\":\"req-1\""));
    assert!(host.remaining_from_html_messages().is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn server_time_host_parses_legacy_timespan_offsets() {
    assert_eq!(
        ServerTimeHost::from_offset_string("08:00:00")
            .unwrap()
            .server_time_zone_offset_milliseconds(),
        28_800_000
    );
    assert_eq!(
        ServerTimeHost::from_offset_string("-05:30:00")
            .unwrap()
            .server_time_zone_offset_milliseconds(),
        -19_800_000
    );
    assert!(matches!(
        ServerTimeHost::from_offset_string("08:99:00").unwrap_err(),
        ScriptHostRuntimeError::InvalidServerTimeZoneOffset(_)
    ));
}

#[test]
fn script_host_runtime_routes_global_and_post_message_calls() {
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

    let version = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "GetVersion",
            Vec::new(),
        ))
        .unwrap();
    assert_eq!(
        version,
        ScriptHostCallResult::String(env!("CARGO_PKG_VERSION").to_string())
    );

    let global = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "KeyPress",
            vec![serde_json::json!("VK_F")],
        ))
        .unwrap();
    let ScriptHostCallResult::InputExecution(global) = global else {
        panic!("expected input execution");
    };
    assert_eq!(global.mode, GlobalInputDispatchMode::PlanOnly);
    assert!(!global.dispatched);
    assert_eq!(global.dispatched_events, 0);
    assert_eq!(
        global.events,
        vec![
            InputEvent::KeyDown {
                vk: 0x46,
                extended: None
            },
            InputEvent::KeyUp {
                vk: 0x46,
                extended: None
            }
        ]
    );

    let post_message = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PostMessage,
            "KeyPress",
            vec![serde_json::json!("VK_F")],
        ))
        .unwrap();
    let ScriptHostCallResult::PostMessageEvents(events) = post_message else {
        panic!("expected post message events");
    };
    assert!(matches!(
        events.first(),
        Some(PostMessageEvent::Message {
            message: bgi_input::WM_ACTIVATE,
            ..
        })
    ));
    assert_eq!(events.len(), 4);
}

#[test]
fn script_host_runtime_routes_global_vision_and_host_helper_plans() {
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.capture_area = GameCaptureArea {
        x: 100,
        y: 50,
        width: 1280,
        height: 720,
    };
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let capture = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "CaptureGameRegion",
            Vec::new(),
        ))
        .unwrap();
    assert!(matches!(
        capture,
        ScriptHostCallResult::CaptureGameRegionPlan(CaptureGameRegionPlan {
            area: GameCaptureArea {
                x: 100,
                y: 50,
                width: 1280,
                height: 720
            },
            pixel_format: "BGR24",
            source: "game_capture_region"
        })
    ));

    assert!(!runtime.global_input.has_capture_frame_source());

    let avatars = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "GetAvatars",
            Vec::new(),
        ))
        .unwrap();
    assert!(matches!(
        avatars,
        ScriptHostCallResult::AvatarRecognitionPlan(AvatarRecognitionPlan {
            model_name: "BgiAvatarSide",
            model_relative_path: "Assets/Model/Common/avatar_side_classify_sim.onnx",
            output: "avatar_names",
            ..
        })
    ));

    let array_plan = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::CustomHostFunctions,
            "NewVarOfArr",
            vec![
                serde_json::json!("OpenCvSharp.Point2f"),
                serde_json::json!(2),
            ],
        ))
        .unwrap();
    assert_eq!(
        array_plan,
        ScriptHostCallResult::CustomHostFunctionCommand(
            CustomHostFunctionCommand::NewArrayVariable {
                element_type: "OpenCvSharp.Point2f".to_string(),
                dimensions: 2,
                legacy_jagged_type: "OpenCvSharp.Point2f[][]".to_string()
            }
        )
    );
}

#[test]
fn script_host_runtime_executes_injected_capture_game_region_source() {
    let source_pixels = vec![
        1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, //
        5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, //
        9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12,
    ];
    let frame = CaptureFrame::packed_bgr(4, 3, source_pixels).unwrap();
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.capture_area = GameCaptureArea {
        x: 1,
        y: 1,
        width: 2,
        height: 2,
    };
    config.capture_frame_source = Some(Arc::new(StaticFrameSource {
        frame,
        area: Some(GameCaptureArea {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        }),
    }));
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let result = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "captureGameRegion",
            Vec::new(),
        ))
        .unwrap();
    let ScriptHostCallResult::CaptureGameRegionExecution(capture) = result else {
        panic!("expected capture execution");
    };

    assert_eq!(capture.width, 2);
    assert_eq!(capture.height, 2);
    assert_eq!(capture.pixel_format, "BGR24");
    assert_eq!(capture.source_width, 4);
    assert_eq!(capture.source_height, 3);
    assert_eq!(
        capture.pixels,
        vec![6, 6, 6, 7, 7, 7, 10, 10, 10, 11, 11, 11]
    );
    assert_eq!(capture.plan.area.x, 1);
    assert_eq!(
        capture.image_region.source,
        bgi_vision::ImageRegionSource::DerivedCrop
    );
    assert_eq!(capture.image_region.rect, Rect::new(1, 1, 2, 2).unwrap());
}

#[test]
fn script_host_runtime_uses_initial_game_metrics_for_global_input() {
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.capture_area = GameCaptureArea {
        x: 120,
        y: 108,
        width: 1600,
        height: 900,
    };
    config.initial_game_metrics = Some(GameMetrics {
        width: 1600,
        height: 900,
        dpi: 1.0,
    });
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let metrics = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "getGameMetrics",
            Vec::new(),
        ))
        .unwrap();
    assert_eq!(
        metrics,
        ScriptHostCallResult::GameMetrics(GameMetrics {
            width: 1600,
            height: 900,
            dpi: 1.0
        })
    );

    let click = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "click",
            vec![serde_json::json!(800), serde_json::json!(450)],
        ))
        .unwrap();
    let ScriptHostCallResult::InputExecution(execution) = click else {
        panic!("expected input execution");
    };
    assert!(matches!(
        execution.events.first(),
        Some(InputEvent::MouseMoveAbsolute {
            x: 920,
            y: 558,
            virtual_desktop: false
        })
    ));
}

#[test]
fn injected_capture_frame_source_defaults_to_full_frame_region() {
    let frame = CaptureFrame::packed_bgr(
        2,
        2,
        vec![
            1, 1, 1, 2, 2, 2, //
            3, 3, 3, 4, 4, 4,
        ],
    )
    .unwrap();
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.capture_area = GameCaptureArea {
        x: 200,
        y: 100,
        width: 1280,
        height: 720,
    };
    config.capture_frame_source = Some(Arc::new(StaticFrameSource { frame, area: None }));
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let result = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Global,
            "captureGameRegion",
            Vec::new(),
        ))
        .unwrap();
    let ScriptHostCallResult::CaptureGameRegionExecution(capture) = result else {
        panic!("expected capture execution");
    };

    assert_eq!(capture.width, 2);
    assert_eq!(capture.height, 2);
    assert_eq!(capture.pixels, vec![1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4]);
    assert_eq!(
        capture.plan.area,
        GameCaptureArea {
            x: 0,
            y: 0,
            width: 2,
            height: 2
        }
    );
}

#[test]
fn script_host_runtime_routes_server_time_calls() {
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.server_time_zone_offset_milliseconds = -18_000_000;
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::ServerTime,
                "GetServerTimeZoneOffset",
                Vec::new(),
            ))
            .unwrap(),
        ScriptHostCallResult::Integer(-18_000_000)
    );
}

#[test]
fn script_host_runtime_routes_http_calls() {
    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let result = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Http,
            "Request",
            vec![
                serde_json::json!("GET"),
                serde_json::json!("https://example.com/status"),
                Value::Null,
                serde_json::json!(r#"{"Accept":"application/json"}"#),
            ],
        ))
        .unwrap();

    let ScriptHostCallResult::HttpRequestPlan(plan) = result else {
        panic!("expected HTTP request plan");
    };
    assert_eq!(plan.method, "GET");
    assert_eq!(plan.url, "https://example.com/status");
    assert_eq!(
        plan.headers,
        vec![("accept".to_string(), "application/json".to_string())]
    );

    let mut config = ScriptHostRuntimeConfig::new(".", ".");
    config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
    config.http_dispatch_mode = HttpDispatchMode::Reqwest;
    let runtime = ScriptHostRuntime::new(config).unwrap();
    let mut client = RecordingHttpClient::ok_json(r#"{"status":"ok"}"#);
    let result = runtime
        .call_http_with_client(
            ScriptHostCall::new(
                ScriptHostTarget::Http,
                "Request",
                vec![
                    serde_json::json!("POST"),
                    serde_json::json!("https://example.com/status"),
                    serde_json::json!(r#"{"ping":true}"#),
                    serde_json::json!(r#"{"Content-Type":"application/json"}"#),
                ],
            ),
            &mut client,
        )
        .unwrap();
    let ScriptHostCallResult::HttpExecution(execution) = result else {
        panic!("expected HTTP execution");
    };
    assert_eq!(execution.mode, HttpDispatchMode::Reqwest);
    assert!(execution.dispatched);
    assert_eq!(execution.request.method, "POST");
    assert_eq!(execution.response.as_ref().unwrap().status_code, 200);
    assert_eq!(
        execution.response.as_ref().unwrap().body,
        r#"{"status":"ok"}"#
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn script_host_runtime_routes_html_mask_calls() {
    let root = test_root("bgi-script-host-runtime-html-mask");
    fs::write(root.join("overlay.html"), "<html></html>").unwrap();
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&root, &root)).unwrap();

    let show = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Show",
            vec![serde_json::json!("overlay.html"), serde_json::json!("mask")],
        ))
        .unwrap();
    assert!(matches!(
        show,
        ScriptHostCallResult::HtmlMaskCommand(HtmlMaskCommand::Show(ref plan))
            if plan.window_id == "mask" && plan.final_url.starts_with("file:///")
    ));

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Send",
            vec![
                serde_json::json!("mask"),
                serde_json::json!("/status"),
                serde_json::json!("{\"ready\":true}"),
            ],
        ))
        .unwrap();
    let flushed = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "FlushPendingMessages",
            vec![serde_json::json!("mask")],
        ))
        .unwrap();
    let ScriptHostCallResult::StringList(flushed) = flushed else {
        panic!("expected flushed messages");
    };
    assert_eq!(flushed.len(), 1);

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "SendFromHtml",
            vec![
                serde_json::json!("mask"),
                serde_json::json!("/event"),
                serde_json::json!("{\"ok\":true}"),
            ],
        ))
        .unwrap();
    let polled = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Poll",
            vec![serde_json::json!("mask")],
        ))
        .unwrap();
    assert!(matches!(
        polled,
        ScriptHostCallResult::String(ref message) if message.contains("\"ok\":true")
    ));

    let snapshot = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::HtmlMask,
            "Snapshot",
            Vec::new(),
        ))
        .unwrap();
    assert!(matches!(
        snapshot,
        ScriptHostCallResult::HtmlMaskSnapshot(HtmlMaskSnapshot { ref windows, .. })
            if windows.len() == 1
    ));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn script_host_runtime_routes_key_mouse_hook_calls() {
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

    let registration = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "OnKeyDown",
            vec![serde_json::json!("key"), serde_json::json!(true)],
        ))
        .unwrap();
    assert!(matches!(
        registration,
        ScriptHostCallResult::KeyMouseHookCommand(KeyMouseHookCommand::AddListener(ref listener))
            if listener.id == "key" && listener.event == KeyMouseHookEventKind::KeyDown
    ));
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "OnMouseMove",
            vec![serde_json::json!("move"), serde_json::json!(25)],
        ))
        .unwrap();

    let key_dispatch = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "DispatchEvent",
            vec![serde_json::json!({
                "type": "keyDown",
                "keyData": "Control, F",
                "keyCode": "F"
            })],
        ))
        .unwrap();
    let ScriptHostCallResult::KeyMouseHookDispatches(key_dispatches) = key_dispatch else {
        panic!("expected key hook dispatches");
    };
    assert_eq!(key_dispatches.len(), 1);
    assert_eq!(key_dispatches[0].args, vec![serde_json::json!("F")]);

    let move_dispatch = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "DispatchEvent",
            vec![serde_json::json!({
                "type": "mouseMove",
                "x": 10,
                "y": 20,
                "timestampMs": 100
            })],
        ))
        .unwrap();
    assert!(matches!(
        move_dispatch,
        ScriptHostCallResult::KeyMouseHookDispatches(ref dispatches)
            if dispatches.len() == 1 && dispatches[0].listener_id == "move"
    ));

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "RemoveAllListeners",
            Vec::new(),
        ))
        .unwrap();
    let snapshot = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseHook,
            "Snapshot",
            Vec::new(),
        ))
        .unwrap();
    assert!(matches!(
        snapshot,
        ScriptHostCallResult::KeyMouseHookSnapshot(KeyMouseHookSnapshot { ref listeners, .. })
            if listeners.is_empty()
    ));
}

#[test]
fn dispatcher_host_records_timer_and_task_plans() {
    let mut host = ScriptDispatcherHost::default();

    let timer = RealtimeTimerHostPlan {
        name: "AutoPick".to_string(),
        interval_ms: 50,
        config: Some(serde_json::json!({"enabled": true})),
        clears_existing_triggers: false,
    };
    let add = host.add_timer(timer);
    assert!(matches!(
        host.commands()[0],
        DispatcherCommand::ClearAllTriggers
    ));
    assert!(matches!(add, DispatcherCommand::AddRealtimeTimer(_)));
    assert_eq!(host.commands().len(), 2);

    let task = SoloTaskHostPlan {
        name: "AutoFight".to_string(),
        config: None,
        uses_linked_cancellation: true,
    };
    let run = host.run_solo_task(task);
    assert!(matches!(run, DispatcherCommand::RunSoloTask(_)));
    assert_eq!(host.commands().len(), 3);
}

#[test]
fn script_host_runtime_routes_dispatcher_calls() {
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

    let add = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Dispatcher,
            "AddTrigger",
            vec![serde_json::json!({
                "Name": "AutoSkip",
                "Interval": 100,
                "Config": { "quickTeleportEnabled": true }
            })],
        ))
        .unwrap();
    let ScriptHostCallResult::DispatcherCommand(DispatcherCommand::AddRealtimeTimer(timer)) = add
    else {
        panic!("expected add realtime timer command");
    };
    assert_eq!(timer.name, "AutoSkip");
    assert_eq!(timer.interval_ms, 100);
    assert!(!timer.clears_existing_triggers);

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Dispatcher,
            "RunTask",
            vec![serde_json::json!({
                "name": "AutoFight",
                "config": { "strategy": "default" }
            })],
        ))
        .unwrap();
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Dispatcher,
            "RunAutoBossTask",
            vec![serde_json::json!({"boss": "sample"})],
        ))
        .unwrap();

    assert_eq!(runtime.dispatcher_commands().len(), 3);
    assert!(matches!(
        runtime.dispatcher_commands()[2],
        DispatcherCommand::RunBuiltinTask { ref name, .. } if name == "AutoBoss"
    ));

    let plans = runtime.dispatcher_task_invocation_plans().unwrap();
    assert_eq!(plans[0].task_key.as_deref(), Some("AutoSkip"));
    assert_eq!(
        plans[1].kind,
        bgi_task::TaskInvocationKind::RunIndependentTask
    );
    assert_eq!(plans[2].task_key.as_deref(), Some("AutoBoss"));
}

#[test]
fn genshin_host_maps_action_commands_to_task_invocations() {
    let mut host = GenshinHost::default();
    host.push(GenshinCommand::Uid);
    host.push(GenshinCommand::Teleport {
        x: 100.5,
        y: 200.25,
        map_name: None,
        force: true,
    });
    host.push(GenshinCommand::SwitchParty {
        party_name: "default".to_string(),
    });
    host.push(GenshinCommand::ChooseTalkOption {
        option: "Katheryne".to_string(),
        skip_times: 2,
        is_orange: true,
    });
    host.push(GenshinCommand::SetTime {
        hour: 8,
        minute: 30,
        skip: true,
    });

    let plans = host.task_invocation_plans().unwrap();

    assert_eq!(plans.len(), 4);
    assert!(plans
        .iter()
        .all(|plan| plan.kind == bgi_task::TaskInvocationKind::RunCommonJob));
    assert_eq!(plans[0].task_key.as_deref(), Some("Teleport"));
    assert_eq!(plans[0].config.as_ref().unwrap()["force"], true);
    assert_eq!(plans[1].task_key.as_deref(), Some("SwitchParty"));
    assert_eq!(
        plans[1].config.as_ref().unwrap()["partyName"],
        serde_json::json!("default")
    );
    assert_eq!(plans[2].task_key.as_deref(), Some("ChooseTalkOption"));
    assert_eq!(
        plans[2].config.as_ref().unwrap()["skipTimes"],
        serde_json::json!(2)
    );
    assert_eq!(plans[3].task_key.as_deref(), Some("SetTime"));
}

#[test]
fn script_host_runtime_routes_genshin_calls() {
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "Tp",
            vec![
                serde_json::json!("100.5"),
                serde_json::json!(200.25),
                serde_json::json!(true),
            ],
        ))
        .unwrap();
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "GetPositionFromMapWithMatchingMethod",
            vec![serde_json::json!("featureMatch")],
        ))
        .unwrap();
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "SwitchParty",
            vec![serde_json::json!("daily")],
        ))
        .unwrap();
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "SetTime",
            vec![
                serde_json::json!(8),
                serde_json::json!("30"),
                serde_json::json!(true),
            ],
        ))
        .unwrap();

    assert_eq!(runtime.genshin_commands().len(), 4);
    assert!(matches!(
        runtime.genshin_commands()[0],
        GenshinCommand::Teleport { force: true, .. }
    ));
    assert!(matches!(
        runtime.genshin_commands()[1],
        GenshinCommand::GetPositionFromMap {
            ref matching_method,
            ..
        } if matching_method.as_deref() == Some("featureMatch")
    ));

    let plans = runtime.genshin_task_invocation_plans().unwrap();
    assert_eq!(plans.len(), 3);
    assert_eq!(plans[0].task_key.as_deref(), Some("Teleport"));
    assert_eq!(plans[1].task_key.as_deref(), Some("SwitchParty"));
    assert_eq!(plans[2].task_key.as_deref(), Some("SetTime"));
}

#[test]
fn script_host_runtime_routes_auto_fishing_genshin_calls() {
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "autoFishing",
            vec![],
        ))
        .unwrap();
    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Genshin,
            "AutoFishing",
            vec![serde_json::json!(2)],
        ))
        .unwrap();

    assert_eq!(runtime.genshin_commands().len(), 2);
    assert!(matches!(
        runtime.genshin_commands()[0],
        GenshinCommand::AutoFishing {
            fishing_time_policy: 0
        }
    ));
    assert!(matches!(
        runtime.genshin_commands()[1],
        GenshinCommand::AutoFishing {
            fishing_time_policy: 2
        }
    ));

    let plans = runtime.genshin_task_invocation_plans().unwrap();
    assert_eq!(plans.len(), 2);
    assert!(plans
        .iter()
        .all(|plan| plan.kind == bgi_task::TaskInvocationKind::RunScriptDispatcherTask));
    assert_eq!(plans[0].task_key.as_deref(), Some("AutoFishing"));
    assert_eq!(
        plans[0].config.as_ref().unwrap()["fishingTimePolicy"],
        serde_json::json!(0)
    );
    assert_eq!(plans[1].task_key.as_deref(), Some("AutoFishing"));
    assert_eq!(
        plans[1].config.as_ref().unwrap()["fishingTimePolicy"],
        serde_json::json!(2)
    );
}

#[test]
fn script_host_runtime_routes_file_and_strategy_file_calls() {
    let script_root = test_root("bgi-script-host-runtime-file");
    let strategy_root = test_root("bgi-script-host-runtime-strategy");
    fs::create_dir_all(strategy_root.join("teams")).unwrap();
    fs::write(strategy_root.join("teams/default.json"), "{}").unwrap();

    let mut runtime =
        ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&script_root, &strategy_root)).unwrap();

    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "CreateDirectory",
                vec![serde_json::json!("nested")],
            ))
            .unwrap(),
        ScriptHostCallResult::Bool(true)
    );
    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "WriteTextSync",
                vec![
                    serde_json::json!("nested/a.txt"),
                    serde_json::json!("hello"),
                ],
            ))
            .unwrap(),
        ScriptHostCallResult::Bool(true)
    );
    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "ReadTextSync",
                vec![serde_json::json!("nested/a.txt")],
            ))
            .unwrap(),
        ScriptHostCallResult::String("hello".to_string())
    );
    let source_image = BgrImage::new(
        VisionSize::new(2, 2),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    )
    .unwrap();
    source_image
        .write_png(script_root.join("nested/source.png"))
        .unwrap();
    let image_read = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::File,
            "ReadImageMatWithResizeSync",
            vec![
                serde_json::json!("nested/source.png"),
                serde_json::json!("1"),
                serde_json::json!(1.0),
                serde_json::json!(1),
            ],
        ))
        .unwrap();
    let ScriptHostCallResult::ImageMatReadExecution(read) = image_read else {
        panic!("expected image mat read execution");
    };
    assert_eq!(read.normalized_path, script_root.join("nested/source.png"));
    assert_eq!(read.width, 1);
    assert_eq!(read.height, 1);
    assert_eq!(read.pixel_format, "BGR24");
    assert_eq!(read.pixels, vec![1, 2, 3]);
    let image_write = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::File,
            "WriteImageSync",
            vec![
                serde_json::json!("nested/output"),
                serde_json::json!({
                    "width": source_image.size.width,
                    "height": source_image.size.height,
                    "pixelFormat": "BGR24",
                    "pixels": source_image.pixels,
                }),
            ],
        ))
        .unwrap();
    let ScriptHostCallResult::ImageMatWriteExecution(write) = image_write else {
        panic!("expected image mat write execution");
    };
    assert_eq!(write.normalized_path, script_root.join("nested/output.png"));
    assert_eq!(write.width, 2);
    assert_eq!(write.height, 2);
    assert!(write.bytes_written > 0);
    assert!(script_root.join("nested/output.png").is_file());
    let template_match = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Vision,
            "FindTemplate",
            vec![
                serde_json::json!({
                    "width": 3,
                    "height": 3,
                    "pixelFormat": "BGR24",
                    "pixels": [
                        1, 1, 1,  2, 2, 2,  3, 3, 3,
                        4, 4, 4,  40, 40, 40,  50, 50, 50,
                        6, 6, 6,  70, 70, 70,  80, 80, 80
                    ]
                }),
                serde_json::json!({
                    "width": 2,
                    "height": 2,
                    "pixelFormat": "BGR24",
                    "pixels": [
                        40, 40, 40,  50, 50, 50,
                        70, 70, 70,  80, 80, 80
                    ]
                }),
                serde_json::json!({
                    "threshold": 0.99,
                    "use3Channels": true,
                    "mode": "CCorrNormed",
                    "maxMatchCount": 1,
                    "name": "patch"
                }),
            ],
        ))
        .unwrap();
    let ScriptHostCallResult::VisionRecognitionExecution(template_match) = template_match else {
        panic!("expected vision recognition execution");
    };
    assert_eq!(
        template_match.recognition_type,
        RecognitionType::TemplateMatch
    );
    assert_eq!(template_match.first.rect, Rect::new(1, 1, 2, 2).unwrap());
    assert_eq!(template_match.first.text, "patch");
    assert_eq!(template_match.matched_count, 1);

    let color_match = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Vision,
            "FindColor",
            vec![
                serde_json::json!({
                    "width": 3,
                    "height": 2,
                    "pixelFormat": "BGR24",
                    "pixels": [
                        1, 2, 3,  40, 40, 40,  5, 6, 7,
                        8, 9, 10,  40, 40, 40,  11, 12, 13
                    ]
                }),
                serde_json::json!({
                    "conversion": "BgrToRgb",
                    "lowerColor": [40, 40, 40],
                    "upperColor": [40, 40, 40],
                    "matchCount": 2,
                    "name": "gray-column"
                }),
            ],
        ))
        .unwrap();
    let ScriptHostCallResult::VisionRecognitionExecution(color_match) = color_match else {
        panic!("expected color recognition execution");
    };
    assert_eq!(color_match.recognition_type, RecognitionType::ColorMatch);
    assert_eq!(color_match.first.rect, Rect::new(1, 0, 1, 2).unwrap());
    assert_eq!(color_match.first.text, "gray-column");
    assert_eq!(color_match.first.score, Some(2.0));
    let cropped = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Vision,
            "Crop",
            vec![
                serde_json::json!({
                    "width": 3,
                    "height": 2,
                    "pixelFormat": "BGR24",
                    "pixels": [
                        1, 2, 3,  40, 40, 40,  5, 6, 7,
                        8, 9, 10,  50, 50, 50,  11, 12, 13
                    ]
                }),
                serde_json::json!({"x": 1, "y": 0, "width": 1, "height": 2}),
            ],
        ))
        .unwrap();
    let ScriptHostCallResult::VisionImageMatExecution(cropped) = cropped else {
        panic!("expected vision image mat execution");
    };
    assert_eq!(cropped.width, 1);
    assert_eq!(cropped.height, 2);
    assert_eq!(cropped.pixel_format, "BGR24");
    assert_eq!(cropped.pixels, vec![40, 40, 40, 50, 50, 50]);
    assert!(matches!(
        cropped.image_region.source,
        bgi_vision::ImageRegionSource::DerivedCrop
    ));

    let scaled = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Vision,
            "To1080p",
            vec![serde_json::json!({
                "width": 1921,
                "height": 1081,
                "pixelFormat": "BGR24",
                "pixels": vec![7; 1921 * 1081 * 3]
            })],
        ))
        .unwrap();
    let ScriptHostCallResult::VisionImageMatExecution(scaled) = scaled else {
        panic!("expected vision image mat execution");
    };
    assert_eq!(scaled.width, 1920);
    assert_eq!(scaled.pixel_format, "BGR24");
    assert!(matches!(
        scaled.image_region.source,
        bgi_vision::ImageRegionSource::DerivedScale
    ));
    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::StrategyFile,
                "ReadPathSync",
                vec![serde_json::json!("teams")],
            ))
            .unwrap(),
        ScriptHostCallResult::StringList(vec!["teams/default.json".to_string()])
    );
    assert!(matches!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "IsExists",
                vec![serde_json::json!("../outside.txt")],
            ))
            .unwrap_err(),
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
    ));

    fs::remove_dir_all(script_root).unwrap();
    fs::remove_dir_all(strategy_root).unwrap();
}

#[test]
fn script_host_runtime_routes_pathing_script_calls() {
    let script_root = test_root("bgi-script-host-runtime-pathing-script");
    let strategy_root = test_root("bgi-script-host-runtime-pathing-strategy");
    let user_auto_pathing_root = test_root("bgi-script-host-runtime-user-pathing");
    let route_json = r#"{
          "info": {
            "name": "sample route",
            "type": "collect",
            "map_name": "Teyvat"
          },
          "positions": [
            { "x": 100.0, "y": 200.0, "type": "path", "move_mode": "dash", "action": "pick_around" }
          ]
        }"#;
    fs::write(script_root.join("route.json"), route_json).unwrap();
    fs::write(user_auto_pathing_root.join("user-route.json"), route_json).unwrap();

    let mut config = ScriptHostRuntimeConfig::new(&script_root, &strategy_root);
    config.user_auto_pathing_root = user_auto_pathing_root.clone();
    config.pathing_party_config = Some(serde_json::json!({ "partyName": "daily" }));
    let mut runtime = ScriptHostRuntime::new(config).unwrap();

    let inline = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "Run",
            vec![serde_json::json!(route_json)],
        ))
        .unwrap();
    let ScriptHostCallResult::PathingExecution(inline_execution) = inline else {
        panic!("expected inline pathing execution");
    };
    let inline_plan = &inline_execution.plan;
    assert!(!inline_execution.dispatched);
    assert!(!inline_execution.completed);
    assert_eq!(inline_execution.execution_plan.segment_count, 1);
    assert_eq!(inline_execution.execution_plan.waypoint_count, 1);
    assert_eq!(
        inline_execution.execution_plan.segments[0].seed_previous_position,
        Some(bgi_core::PathingPoint {
            x: 32568.0,
            y: 15984.0
        })
    );
    assert_eq!(
        inline_execution.execution_plan.segments[0].waypoints[0].track_point,
        Some(bgi_core::PathingPoint {
            x: 32568.0,
            y: 15984.0
        })
    );
    assert!(!inline_execution.execution_plan.segments[0].waypoints[0].track_conversion_pending);
    assert_eq!(
        inline_execution.execution_plan.segments[0].seed_previous_position_coordinate_space,
        Some(bgi_core::PathingCoordinateSpace::LegacyTrackMap)
    );
    assert!(!inline_execution
        .execution_plan
        .movement_contract
        .pending_dependencies
        .contains(&bgi_core::PathingMovementDependency::CoordinateConversion));
    assert_eq!(inline_plan.source, PathingScriptSource::InlineJson);
    assert_eq!(inline_plan.summary.waypoint_count, 1);
    assert_eq!(inline_plan.summary.actions, vec!["pick_around".to_string()]);
    assert_eq!(
        inline_plan.party_config,
        Some(serde_json::json!({ "partyName": "daily" }))
    );

    let from_script = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "RunFile",
            vec![serde_json::json!("route.json")],
        ))
        .unwrap();
    let ScriptHostCallResult::PathingExecution(from_script) = from_script else {
        panic!("expected script file pathing execution");
    };
    assert_eq!(from_script.plan.source, PathingScriptSource::ScriptFile);
    assert_eq!(
        from_script.plan.task.file_name.as_deref(),
        Some("route.json")
    );
    assert_eq!(
        from_script.execution_plan.segments[0].waypoints[0].track_point,
        Some(bgi_core::PathingPoint {
            x: 32568.0,
            y: 15984.0
        })
    );
    assert!(!from_script
        .execution_plan
        .movement_contract
        .pending_dependencies
        .contains(&bgi_core::PathingMovementDependency::CoordinateConversion));

    let from_user = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "RunFileFromUser",
            vec![serde_json::json!("user-route.json")],
        ))
        .unwrap();
    let ScriptHostCallResult::PathingExecution(from_user) = from_user else {
        panic!("expected user pathing execution");
    };
    assert_eq!(
        from_user.plan.source,
        PathingScriptSource::UserAutoPathingFile
    );
    assert_eq!(
        from_user.plan.task.file_name.as_deref(),
        Some("user-route.json")
    );
    assert!(!from_user.dispatched);
    assert!(!from_user.completed);
    assert_eq!(from_user.execution_plan.segment_count, 1);
    assert_eq!(from_user.execution_plan.waypoint_count, 1);
    assert_eq!(
        from_user.execution_plan.segments[0].seed_previous_position,
        Some(bgi_core::PathingPoint {
            x: 32568.0,
            y: 15984.0
        })
    );
    assert_eq!(
        from_user.execution_plan.segments[0].seed_previous_position_coordinate_space,
        Some(bgi_core::PathingCoordinateSpace::LegacyTrackMap)
    );
    assert!(!from_user
        .execution_plan
        .movement_contract
        .pending_dependencies
        .contains(&bgi_core::PathingMovementDependency::CoordinateConversion));

    let plan = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "PlanFileFromUser",
            vec![serde_json::json!("user-route.json")],
        ))
        .unwrap();
    let ScriptHostCallResult::PathingPlan(plan) = plan else {
        panic!("expected user pathing plan");
    };
    assert_eq!(plan.source, PathingScriptSource::UserAutoPathingFile);
    assert_eq!(plan.task.file_name.as_deref(), Some("user-route.json"));

    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "IsFile",
                vec![serde_json::json!("user-route.json")],
            ))
            .unwrap(),
        ScriptHostCallResult::Bool(true)
    );
    assert_eq!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "ReadPathSync",
                Vec::new(),
            ))
            .unwrap(),
        ScriptHostCallResult::StringList(vec!["user-route.json".to_string()])
    );
    assert!(matches!(
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "RunFileFromUser",
                vec![serde_json::json!("../outside.json")],
            ))
            .unwrap_err(),
        ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
    ));

    let unsupported_map_route_json = r#"{
          "info": {
            "name": "unsupported route",
            "type": "collect",
            "map_name": "Enkanomiya"
          },
          "positions": [
            { "x": 100.0, "y": 200.0, "type": "path" }
          ]
        }"#;
    let unsupported = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::PathingScript,
            "Run",
            vec![serde_json::json!(unsupported_map_route_json)],
        ))
        .unwrap();
    let ScriptHostCallResult::PathingExecution(unsupported_execution) = unsupported else {
        panic!("expected unsupported-map pathing execution");
    };
    assert_eq!(
        unsupported_execution.execution_plan.segments[0].waypoints[0].track_point,
        None
    );
    assert!(unsupported_execution.execution_plan.segments[0].waypoints[0].track_conversion_pending);
    assert!(unsupported_execution
        .execution_plan
        .movement_contract
        .pending_dependencies
        .contains(&bgi_core::PathingMovementDependency::CoordinateConversion));

    fs::remove_dir_all(script_root).unwrap();
    fs::remove_dir_all(strategy_root).unwrap();
    fs::remove_dir_all(user_auto_pathing_root).unwrap();
}

#[test]
fn script_host_runtime_routes_key_mouse_log_and_notification_calls() {
    let root = test_root("bgi-script-host-runtime-keymouse");
    let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&root, &root)).unwrap();

    let plan = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseScript,
            "Run",
            vec![serde_json::json!(
                r#"{
                      "macroEvents": [
                        { "type": 0, "keyCode": 87, "time": 0 },
                        { "type": 1, "keyCode": 87, "time": 50 }
                      ]
                    }"#
            )],
        ))
        .unwrap();
    let ScriptHostCallResult::KeyMouseExecution(execution) = plan else {
        panic!("expected key/mouse execution");
    };
    assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
    assert!(!execution.dispatched);
    assert_eq!(execution.plan.summary.event_count, 2);
    assert_eq!(execution.plan.input_events.len(), 3);

    let plan = runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::KeyMouseScript,
            "Plan",
            vec![serde_json::json!(
                r#"{
                      "macroEvents": [
                        { "type": 0, "keyCode": 87, "time": 0 },
                        { "type": 1, "keyCode": 87, "time": 50 }
                      ]
                    }"#
            )],
        ))
        .unwrap();
    assert!(matches!(
        plan,
        ScriptHostCallResult::KeyMousePlan(KeyMouseScriptRunPlan { .. })
    ));

    runtime
        .call(ScriptHostCall::new(
            ScriptHostTarget::Log,
            "Info",
            vec![serde_json::json!("ready")],
        ))
        .unwrap();
    let notification = runtime
        .call_at(
            ScriptHostCall::new(
                ScriptHostTarget::Notification,
                "Send",
                vec![serde_json::json!("ready")],
            ),
            10,
        )
        .unwrap();
    let ScriptHostCallResult::NotificationExecution(notification) = notification else {
        panic!("expected notification execution");
    };
    assert_eq!(notification.mode, NotificationDispatchMode::RecordOnly);
    assert!(!notification.dispatched);
    assert_eq!(notification.record.message, "ready");
    assert!(notification.delivery.is_none());

    assert_eq!(runtime.log_records()[0].level, ScriptLogLevel::Info);
    assert_eq!(runtime.notification_records()[0].timestamp_ms, 10);

    let mut config = ScriptHostRuntimeConfig::new(&root, &root);
    config.notification_dispatch_mode = NotificationDispatchMode::Sink;
    let mut runtime = ScriptHostRuntime::new(config).unwrap();
    let mut sink = RecordingNotificationSink::default();
    let notification = runtime
        .call_notification_with_sink(
            ScriptHostCall::new(
                ScriptHostTarget::Notification,
                "Error",
                vec![serde_json::json!("failed")],
            ),
            11,
            &mut sink,
        )
        .unwrap();
    let ScriptHostCallResult::NotificationExecution(notification) = notification else {
        panic!("expected notification execution");
    };
    assert_eq!(notification.mode, NotificationDispatchMode::Sink);
    assert!(notification.dispatched);
    assert_eq!(
        notification.delivery.as_ref().unwrap().event_code,
        "js.error"
    );
    assert_eq!(notification.delivery.as_ref().unwrap().result, "fail");
    assert_eq!(sink.deliveries().len(), 1);

    fs::remove_dir_all(root).unwrap();
}

fn test_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
