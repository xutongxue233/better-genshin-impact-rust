use super::*;

#[test]
fn default_config_keeps_legacy_defaults_for_first_ported_fields() {
    let config = AppConfig::default();
    assert_eq!(config.trigger_interval, 50);
    assert!(config.auto_fix_win11_bit_blt);
    assert_eq!(config.auto_pick_config.pick_key, "F");
    assert_eq!(config.hot_key_config.bgi_enabled_hotkey, "F11");
    assert_eq!(
        config
            .key_bindings_config
            .action_key(GenshinAction::QuickUseGadget),
        KeyId::Z
    );
    assert_eq!(config.coverage().strongly_typed_sections, 33);
    assert_eq!(config.coverage().compatibility_sections, 0);
}

#[test]
fn overlay_metric_items_follow_legacy_order_defaults_and_cleanup() {
    let descriptors = overlay_metric_descriptors();
    assert_eq!(
        descriptors.iter().map(|item| item.key).collect::<Vec<_>>(),
        vec![
            "GameFps",
            "ProcessingCost",
            "PeakProcessingCost",
            "CaptureCost",
            "TriggerCost",
            "SkippedTicks",
            "GpuUsage",
            "CpuUsage",
            "MemoryUsage",
        ]
    );
    assert_eq!(descriptors[0].display_name, "游戏帧率");
    assert_eq!(descriptors[6].display_name, "显卡占用");
    assert!(OverlayMetricItem::GameFps.enabled_by_default());
    assert!(!OverlayMetricItem::TriggerCost.enabled_by_default());
    assert!(!OverlayMetricItem::GpuUsage.enabled_by_default());

    let mut config = MaskWindowConfig::default();
    config.overlay_metric_items.clear();
    config
        .overlay_metric_items
        .insert("TriggerInterval".to_string(), false);
    config
        .overlay_metric_items
        .insert("UnknownMetric".to_string(), true);
    config.ensure_overlay_metric_items();

    assert!(!config.overlay_metric_items.contains_key("TriggerInterval"));
    assert!(!config.overlay_metric_items.contains_key("UnknownMetric"));
    assert_eq!(
        config.overlay_metric_items.get("PeakProcessingCost"),
        Some(&false)
    );
    assert_eq!(config.overlay_metric_items.get("GameFps"), Some(&true));
    assert_eq!(config.overlay_metric_items.get("GpuUsage"), Some(&false));
}

#[test]
fn overlay_state_migrates_legacy_metrics_layout_without_mutating_source() {
    let mut config = MaskWindowConfig {
        show_overlay_metrics: true,
        metrics_left_ratio: 20.0 / 1920.0,
        metrics_top_ratio: 760.0 / 1080.0,
        metrics_width_ratio: 477.0 / 1920.0,
        metrics_height_ratio: 58.0 / 1080.0,
        ..MaskWindowConfig::default()
    };
    config.overlay_metric_items.clear();
    config
        .overlay_metric_items
        .insert("GameFps".to_string(), true);
    config
        .overlay_metric_items
        .insert("GpuUsage".to_string(), true);

    let state = config.overlay_state();
    assert!(state.show_overlay_metrics);
    assert!(state.migrated_legacy_metrics_layout);
    assert!(same_overlay_layout(
        &state.metrics_layout,
        &DEFAULT_METRICS_LAYOUT
    ));
    assert_eq!(
        state.enabled_metric_keys,
        [
            "GameFps",
            "ProcessingCost",
            "PeakProcessingCost",
            "CaptureCost",
            "SkippedTicks",
            "GpuUsage",
            "MemoryUsage",
        ]
    );
    assert_eq!(state.metrics.len(), OVERLAY_METRIC_ITEMS.len());

    assert!(same_ratio(config.metrics_top_ratio, 760.0 / 1080.0));
}

#[test]
fn deserializes_camel_case_config() {
    let json = r#"{
        "captureMode": "BitBlt",
        "triggerInterval": 75,
        "autoPickConfig": { "enabled": false, "pickKey": "E" }
    }"#;
    let config: AppConfig = json5::from_str(json).unwrap();
    assert_eq!(config.trigger_interval, 75);
    assert!(!config.auto_pick_config.enabled);
    assert_eq!(config.auto_pick_config.pick_key, "E");
}

#[test]
fn deserializes_legacy_numeric_key_bindings() {
    let json = r#"{
        "keyBindingsConfig": {
            "moveForward": 38,
            "normalAttack": 1,
            "quickUseGadget": 90
        }
    }"#;
    let config: AppConfig = json5::from_str(json).unwrap();
    assert_eq!(config.key_bindings_config.move_forward, KeyId::UP);
    assert_eq!(
        config.key_bindings_config.normal_attack,
        KeyId::MOUSE_LEFT_BUTTON
    );
    assert_eq!(
        config
            .key_bindings_config
            .action_key(GenshinAction::QuickUseGadget),
        KeyId::Z
    );
}
