use super::*;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde_json::json;

#[test]
fn skip_during_matches_only_valid_current_local_hour() {
    let clock = test_clock("2026-06-26T10:00:00", 8, "2026-06-26T10:00:00+08:00");
    let decision = pathing_pre_run_skip_decision(
        "route.json",
        &json!({
            "enabled": true,
            "skipDuring": "10"
        }),
        &clock,
    );

    assert!(decision.should_skip);
    assert_eq!(decision.message, "route.json任务已到禁止执行时段，将跳过！");
    assert!(!is_current_hour_equal("abc", 10));
    assert!(!is_current_hour_equal("24", 10));
    assert!(!is_current_hour_equal("9", 10));
    assert!(is_current_hour_equal(" +10 ", 10));
}

#[test]
fn pathing_config_disabled_disables_skip_during_and_cycle_checks() {
    let clock = test_clock("2026-06-26T10:00:00", 8, "2026-06-26T10:00:00+08:00");
    let decision = pathing_pre_run_skip_decision(
        "route.json",
        &json!({
            "enabled": false,
            "skipDuring": "10",
            "taskCycleConfig": {
                "enable": true,
                "boundaryTime": 0,
                "cycle": 3,
                "index": 1
            }
        }),
        &clock,
    );

    assert!(!decision.should_skip);
}

#[test]
fn task_cycle_uses_boundary_adjusted_local_day() {
    let before_boundary = test_clock("2026-06-26T03:00:00", 8, "2026-06-26T03:00:00+08:00");
    let after_boundary = test_clock("2026-06-26T05:00:00", 8, "2026-06-26T05:00:00+08:00");
    let config = PathingPartyTaskCycleConfig {
        enable: true,
        boundary_time: 4,
        cycle: 3,
        index: 1,
        ..PathingPartyTaskCycleConfig::default()
    };

    assert_eq!(config.execution_order(&before_boundary), 2);
    assert_eq!(config.execution_order(&after_boundary), 3);
}

#[test]
fn task_cycle_can_use_server_time_and_invalid_config_keeps_task() {
    let clock = test_clock("2026-06-26T23:00:00", 8, "2026-06-27T01:00:00+08:00");
    let decision = pathing_pre_run_skip_decision(
        "route.json",
        &json!({
            "enabled": true,
            "taskCycleConfig": {
                "enable": true,
                "boundaryTime": 0,
                "isBoundaryTimeBasedOnServerTime": true,
                "cycle": 3,
                "index": 2
            }
        }),
        &clock,
    );

    assert!(decision.should_skip);
    assert_eq!(
        decision.message,
        "route.json任务已经不在执行周期（当前值$1!=配置值$2），将跳过此任务！"
    );

    let invalid = pathing_pre_run_skip_decision(
        "route.json",
        &json!({
            "enabled": true,
            "taskCycleConfig": {
                "enable": true,
                "boundaryTime": 24,
                "cycle": 1,
                "index": 1
            }
        }),
        &clock,
    );
    assert!(!invalid.should_skip);
}

fn test_clock(local: &str, local_offset_hours: i32, server: &str) -> ExecutionRecordClock {
    let local_offset = FixedOffset::east_opt(local_offset_hours * 3_600).unwrap();
    ExecutionRecordClock::fixed(
        NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M:%S").unwrap(),
        local_offset,
        DateTime::parse_from_rfc3339(server).unwrap(),
    )
}
