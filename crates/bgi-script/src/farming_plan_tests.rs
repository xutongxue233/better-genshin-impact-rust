#![allow(clippy::field_reassign_with_default)]

use super::*;
use bgi_core::FarmingPlanConfig;
use chrono::FixedOffset;
use serde_json::json;

#[test]
fn farming_plan_date_key_uses_server_four_am_boundary() {
    assert_eq!(
        daily_farming_date_key(DateTime::parse_from_rfc3339("2026-06-26T03:59:59+08:00").unwrap()),
        "20260625"
    );
    assert_eq!(
        daily_farming_date_key(DateTime::parse_from_rfc3339("2026-06-26T04:00:00+08:00").unwrap()),
        "20260626"
    );
}

#[test]
fn farming_plan_decision_keeps_disabled_or_non_counting_sessions() {
    let config = FarmingPlanConfig {
        enabled: true,
        daily_elite_cap: 1,
        daily_mob_cap: 1,
        ..FarmingPlanConfig::default()
    };
    let daily = DailyFarmingData {
        total_elite_mob_count: 10.0,
        total_normal_mob_count: 10.0,
        ..DailyFarmingData::default()
    };
    let mut farming = farming("elite", 1.0, 1.0);
    farming.allow_farming_count = false;
    assert!(!farming_plan_skip_decision(&farming, &daily, &config).should_skip);

    farming.allow_farming_count = true;
    farming.primary_target = "disable".to_string();
    assert!(!farming_plan_skip_decision(&farming, &daily, &config).should_skip);
}

#[test]
fn farming_plan_decision_matches_limit_and_target_rules() {
    let config = FarmingPlanConfig {
        enabled: true,
        daily_elite_cap: 400,
        daily_mob_cap: 2000,
        ..FarmingPlanConfig::default()
    };
    let daily = DailyFarmingData {
        total_elite_mob_count: 400.0,
        total_normal_mob_count: 100.0,
        ..DailyFarmingData::default()
    };

    let both_zero = farming("", 0.0, 0.0);
    assert_eq!(
        farming_plan_skip_decision(&both_zero, &DailyFarmingData::default(), &config).message,
        "精英和小怪计数都为0，请确认配置"
    );

    let missing_primary_count = farming("elite", 0.0, 1.0);
    assert_eq!(
        farming_plan_skip_decision(&missing_primary_count, &daily, &config).message,
        "精英超上限:400/400,主目标计数为0，请确认配置"
    );

    let elite_target = farming("elite", 3.0, 10.0);
    assert_eq!(
        farming_plan_skip_decision(&elite_target, &daily, &config).message,
        "精英超上限:400/400,脚本主目标为精英"
    );

    let mixed_without_primary_target = farming("", 0.0, 10.0);
    assert!(
        !farming_plan_skip_decision(&mixed_without_primary_target, &daily, &config).should_skip
    );

    let elite_only = farming("", 10.0, 0.0);
    assert_eq!(
        farming_plan_skip_decision(&elite_only, &daily, &config).message,
        "精英超上限:400/400"
    );
}

#[test]
fn farming_plan_decision_uses_miyoushe_totals_and_caps_when_present() {
    let mut config = FarmingPlanConfig {
        enabled: true,
        daily_elite_cap: 400,
        daily_mob_cap: 2000,
        ..FarmingPlanConfig::default()
    };
    config.miyoushe_data_config.daily_elite_cap = 500;
    config.miyoushe_data_config.daily_mob_cap = 900;
    let daily = DailyFarmingData {
        total_elite_mob_count: 0.0,
        total_normal_mob_count: 0.0,
        miyoushe_total_elite_mob_count: 480.0,
        miyoushe_total_normal_mob_count: 800.0,
        travels_diary_detail_manager_update_time: Some("2026-06-26T10:00:00".to_string()),
        records: vec![
            FarmingRecord {
                elite_mob_count: 30.0,
                normal_mob_count: 50.0,
                timestamp: Some("2026-06-26T10:30:00".to_string()),
                ..FarmingRecord::default()
            },
            FarmingRecord {
                elite_mob_count: 200.0,
                normal_mob_count: 200.0,
                timestamp: Some("2026-06-26T09:30:00".to_string()),
                ..FarmingRecord::default()
            },
        ],
        ..DailyFarmingData::default()
    };

    let totals = daily.final_totals(&config);
    assert!(totals.uses_miyoushe_stats);
    assert_eq!(totals.total_elite_mob_count, 510.0);
    assert_eq!(totals.total_normal_mob_count, 850.0);
    assert_eq!(totals.daily_elite_cap, 500);
    assert_eq!(totals.daily_mob_cap, 900);
    assert_eq!(
        farming_plan_skip_decision(&farming("elite", 1.0, 1.0), &daily, &config).message,
        "精英超上限:510/500,脚本主目标为精英"
    );
}

#[test]
fn farming_plan_file_decision_keeps_when_disabled_without_reading_pathing_file() {
    let root = test_root("disabled");
    let context = FarmingPlanExecutionContext::from_app_root(&root, FarmingPlanConfig::default());

    let decision = farming_plan_skip_decision_from_pathing_file(
        root.join("missing"),
        "folder",
        "route.json",
        Some(&context),
        now_server(),
    )
    .unwrap();

    assert!(!decision.should_skip);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn farming_plan_file_decision_reads_pathing_and_ignores_broken_daily_json() {
    let root = test_root("file-decision");
    let pathing_root = root.join("User").join("AutoPathing");
    let route_dir = pathing_root.join("daily");
    std::fs::create_dir_all(&route_dir).unwrap();
    std::fs::write(
        route_dir.join("route.json"),
        serde_json::to_string_pretty(&json!({
            "info": {"name": "route"},
            "farming_info": {
                "allow_farming_count": true,
                "primary_target": "",
                "normal_mob_count": 0,
                "elite_mob_count": 0
            },
            "positions": []
        }))
        .unwrap(),
    )
    .unwrap();
    let log_dir = root.join("log").join("FarmingPlan");
    std::fs::create_dir_all(&log_dir).unwrap();
    std::fs::write(log_dir.join("20260626.json"), "{ broken").unwrap();
    let mut config = FarmingPlanConfig::default();
    config.enabled = true;
    let context = FarmingPlanExecutionContext::from_app_root(&root, config);

    let decision = farming_plan_skip_decision_from_pathing_file(
        &pathing_root,
        "daily",
        "route.json",
        Some(&context),
        now_server(),
    )
    .unwrap();

    assert!(decision.should_skip);
    assert_eq!(decision.message, "精英和小怪计数都为0，请确认配置");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn farming_plan_record_session_updates_daily_totals_and_appends_record() {
    let root = test_root("record-session");
    let mut config = FarmingPlanConfig::default();
    config.enabled = true;
    config.daily_elite_cap = 400;
    config.daily_mob_cap = 2000;
    let context = FarmingPlanExecutionContext::from_app_root(&root, config);
    save_daily_farming_data(
        daily_farming_data_path(&context.log_directory, now_server()),
        &DailyFarmingData {
            total_elite_mob_count: 10.0,
            total_normal_mob_count: 20.0,
            records: vec![FarmingRecord {
                group_name: "old".to_string(),
                project_name: "old-route".to_string(),
                folder_name: "old-folder".to_string(),
                normal_mob_count: 1.0,
                elite_mob_count: 1.0,
                timestamp: Some("2026-06-26T08:00:00+08:00".to_string()),
            }],
            ..DailyFarmingData::default()
        },
    )
    .unwrap();

    let outcome = record_farming_session(
        &context,
        &FarmingRouteRef::new("daily", "route.json", "folder"),
        &farming("elite", 3.0, 5.0),
        DateTime::parse_from_rfc3339("2026-06-26T11:00:00+08:00").unwrap(),
        now_server(),
    )
    .unwrap();

    assert_eq!(outcome.totals.total_elite_mob_count, 13.0);
    assert_eq!(outcome.totals.total_normal_mob_count, 25.0);
    assert_eq!(outcome.record.group_name, "daily");
    assert_eq!(outcome.record.project_name, "route.json");
    assert_eq!(outcome.record.folder_name, "folder");
    assert_eq!(outcome.record.elite_mob_count, 3.0);
    assert_eq!(outcome.record.normal_mob_count, 5.0);
    assert_eq!(
        outcome.record.timestamp.as_deref(),
        Some("2026-06-26T11:00:00+08:00")
    );

    let daily = read_daily_farming_data(&context.log_directory, now_server()).unwrap();
    assert_eq!(daily.total_elite_mob_count, 13.0);
    assert_eq!(daily.total_normal_mob_count, 25.0);
    assert_eq!(daily.records.len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn farming_plan_record_session_keeps_zero_counts_when_counting_disabled() {
    let root = test_root("record-disabled-counting");
    let context = FarmingPlanExecutionContext::from_app_root(&root, FarmingPlanConfig::default());
    let mut farming = farming("disable", 3.0, 5.0);
    farming.allow_farming_count = false;

    let outcome = record_farming_session(
        &context,
        &FarmingRouteRef::new("daily", "route.json", "folder"),
        &farming,
        DateTime::parse_from_rfc3339("2026-06-26T11:00:00+08:00").unwrap(),
        now_server(),
    )
    .unwrap();

    assert_eq!(outcome.totals.total_elite_mob_count, 0.0);
    assert_eq!(outcome.totals.total_normal_mob_count, 0.0);
    assert_eq!(outcome.record.elite_mob_count, 0.0);
    assert_eq!(outcome.record.normal_mob_count, 0.0);
    std::fs::remove_dir_all(root).unwrap();
}

fn farming(
    primary_target: &str,
    elite_mob_count: f64,
    normal_mob_count: f64,
) -> PathingFarmingExecutionPlan {
    PathingFarmingExecutionPlan {
        allow_farming_count: true,
        primary_target: primary_target.to_string(),
        normal_mob_count,
        elite_mob_count,
        expected_fight_count: 0,
    }
}

fn now_server() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-06-26T10:00:00+08:00").unwrap()
}

fn test_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "bgi-script-farming-plan-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}
