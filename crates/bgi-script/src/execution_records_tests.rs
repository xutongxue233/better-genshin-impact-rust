use super::*;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn saves_records_by_start_date_and_replaces_first_matching_guid() {
    let root = test_root("save-replace");
    let storage = ExecutionRecordStorage::new(root.join("ExecutionRecords"));
    let mut initial = record("same-guid", "Daily", "route.json", "folder", "Pathing");
    initial.start_time = "2026-06-25T23:59:00+08:00".to_string();
    initial.end_time = "2026-06-26T00:10:00+08:00".to_string();

    storage.save_execution_record(&initial).unwrap();
    let path = storage.storage_directory().join("20260625.json");
    assert!(path.exists());

    let mut finished = initial.clone();
    finished.is_successful = true;
    finished.end_time = "2026-06-26T00:20:00+08:00".to_string();
    storage.save_execution_record(&finished).unwrap();

    let daily = read_daily(&path);
    assert_eq!(daily.name, "20260625");
    assert_eq!(daily.execution_records.len(), 1);
    assert!(daily.execution_records[0].is_successful);
    assert_eq!(daily.execution_records[0].end_time, finished.end_time);

    let mut another = record("other-guid", "Daily", "route.json", "folder", "Pathing");
    another.start_time = initial.start_time.clone();
    storage.save_execution_record(&another).unwrap();
    let daily = read_daily(&path);
    assert_eq!(
        daily
            .execution_records
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>(),
        vec!["same-guid", "other-guid"]
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preserves_existing_daily_name_and_only_replaces_first_duplicate_guid() {
    let root = test_root("duplicate-guid");
    let storage = ExecutionRecordStorage::new(root.join("ExecutionRecords"));
    fs::create_dir_all(storage.storage_directory()).unwrap();
    let path = storage.storage_directory().join("20260626.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&DailyExecutionRecord {
            name: "custom-name".to_string(),
            execution_records: vec![
                record("dup", "A", "old-a", "folder", "Pathing"),
                record("dup", "A", "old-b", "folder", "Pathing"),
            ],
        })
        .unwrap(),
    )
    .unwrap();

    let mut replacement = record("dup", "A", "new", "folder", "Pathing");
    replacement.start_time = "2026-06-26T08:00:00+08:00".to_string();
    storage.save_execution_record(&replacement).unwrap();

    let daily = read_daily(&path);
    assert_eq!(daily.name, "custom-name");
    assert_eq!(daily.execution_records[0].project_name, "new");
    assert_eq!(daily.execution_records[1].project_name, "old-b");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reads_recent_records_by_date_range_with_legacy_reverse_order() {
    let root = test_root("recent-order");
    let storage = ExecutionRecordStorage::new(root.join("ExecutionRecords"));
    fs::create_dir_all(storage.storage_directory()).unwrap();
    write_daily(
        storage.storage_directory(),
        "20260624",
        vec![record("old", "G", "old", "f", "Pathing")],
    );
    write_daily(
        storage.storage_directory(),
        "20260625",
        vec![
            record("yesterday-a", "G", "a", "f", "Pathing"),
            record("yesterday-b", "G", "b", "f", "Pathing"),
        ],
    );
    write_daily(
        storage.storage_directory(),
        "20260626",
        vec![
            record("today-a", "G", "a", "f", "Pathing"),
            record("today-b", "G", "b", "f", "Pathing"),
        ],
    );

    let recent = storage
        .recent_execution_records_for_today(2, date("2026-06-26"))
        .unwrap();
    assert_eq!(
        recent
            .iter()
            .map(|daily| daily.name.as_str())
            .collect::<Vec<_>>(),
        vec!["20260626", "20260625"]
    );
    assert_eq!(
        recent[0]
            .execution_records
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>(),
        vec!["today-b", "today-a"]
    );
    assert!(matches!(
        storage.recent_execution_records_for_today(0, date("2026-06-26")),
        Err(ExecutionRecordStorageError::InvalidDays)
    ));

    let missing = ExecutionRecordStorage::new(root.join("missing"))
        .recent_execution_records_for_today(1, date("2026-06-26"))
        .unwrap();
    assert!(missing.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn config_day_count_matches_legacy_boundary_and_gap_rules() {
    let mut config = TaskCompletionSkipRuleConfig::default();
    config.boundary_time = -1;
    assert_eq!(config.recent_day_count(), 1);

    config.boundary_time = 4;
    assert_eq!(config.recent_day_count(), 2);

    config.last_run_gap_seconds = 0;
    assert_eq!(config.recent_day_count(), 0);
    config.last_run_gap_seconds = 86_400;
    assert_eq!(config.recent_day_count(), 1);
    config.last_run_gap_seconds = 86_401;
    assert_eq!(config.recent_day_count(), 2);

    let parsed = TaskCompletionSkipRuleConfig::from_pathing_config(&json!({
        "taskCompletionSkipRuleConfig": {
            "enable": true,
            "skipPolicy": "SameNameSkipPolicy",
            "boundaryTime": 3,
            "isBoundaryTimeBasedOnServerTime": true,
            "lastRunGapSeconds": 9,
            "referencePoint": "StartTime"
        }
    }))
    .unwrap();
    assert!(parsed.enable);
    assert_eq!(parsed.skip_policy, "SameNameSkipPolicy");
    assert_eq!(parsed.boundary_time, 3);
    assert!(parsed.is_boundary_time_based_on_server_time);
    assert_eq!(parsed.last_run_gap_seconds, 9);
    assert_eq!(parsed.reference_point, "StartTime");
}

#[test]
fn skip_task_matches_same_name_with_gap_and_message() {
    let clock = test_clock("2026-06-26T10:00:00", 8, "2026-06-26T10:00:00+08:00");
    let project = ExecutionRecordProjectRef::new("Daily", "route.json", "folder", "Pathing");
    let mut matching = record(
        "match-guid",
        "OtherGroup",
        "route.json",
        "elsewhere",
        "Pathing",
    );
    matching.is_successful = true;
    matching.end_time = "2026-06-26T09:30:00+08:00".to_string();
    let daily = DailyExecutionRecord {
        name: "20260626".to_string(),
        execution_records: vec![
            unsuccessful_record("bad-success", "Daily", "route.json", "folder", "Pathing"),
            record("bad-name", "Daily", "other.json", "folder", "Pathing"),
            matching,
        ],
    };
    let config = TaskCompletionSkipRuleConfig {
        enable: true,
        skip_policy: "SameNameSkipPolicy".to_string(),
        boundary_time: -1,
        last_run_gap_seconds: 3_600,
        reference_point: "EndTime".to_string(),
        ..TaskCompletionSkipRuleConfig::default()
    };

    let decision = is_skip_task(&project, Some(&config), &[daily], &clock).unwrap();
    assert!(decision.should_skip);
    assert_eq!(
        decision.message,
        "检查出满足跳过条件: 名称相同, 需在 2026-6-26 10:30:00 之后才能开始执行, 匹配记录 GUID=match-guid"
    );
}

#[test]
fn skip_task_matches_group_and_physical_path_policies() {
    let clock = test_clock("2026-06-26T10:00:00", 8, "2026-06-26T10:00:00+08:00");
    let project = ExecutionRecordProjectRef::new("Daily", "route.json", "folder", "Pathing");
    let mut matching = record("path-guid", "Daily", "route.json", "folder", "Pathing");
    matching.is_successful = true;
    matching.server_start_time = Some("2026-06-26T09:00:00+08:00".to_string());
    let daily = DailyExecutionRecord {
        name: "20260626".to_string(),
        execution_records: vec![matching],
    };
    let mut config = TaskCompletionSkipRuleConfig {
        enable: true,
        skip_policy: "GroupPhysicalPathSkipPolicy".to_string(),
        boundary_time: 4,
        last_run_gap_seconds: -1,
        ..TaskCompletionSkipRuleConfig::default()
    };

    let group_decision = is_skip_task(&project, Some(&config), &[daily.clone()], &clock).unwrap();
    assert!(group_decision.should_skip);
    assert_eq!(
        group_decision.message,
        "检查出满足跳过条件: 组和物理路径匹配一致, 需在下一日 4 点后才能开始执行, 匹配记录 GUID=path-guid"
    );

    config.skip_policy = "PhysicalPathSkipPolicy".to_string();
    let physical_decision = is_skip_task(&project, Some(&config), &[daily], &clock).unwrap();
    assert!(physical_decision.should_skip);
    assert!(physical_decision.message.contains("物理路径相同"));
}

#[test]
fn skip_task_respects_boundary_window_and_invalid_config() {
    let before_boundary = test_clock("2026-06-26T03:00:00", 8, "2026-06-26T03:00:00+08:00");
    let project = ExecutionRecordProjectRef::new("Daily", "route.json", "folder", "Pathing");
    let mut matching = record("boundary-guid", "Daily", "route.json", "folder", "Pathing");
    matching.is_successful = true;
    matching.server_start_time = Some("2026-06-25T05:00:00+08:00".to_string());
    let daily = DailyExecutionRecord {
        name: "20260625".to_string(),
        execution_records: vec![matching],
    };
    let config = TaskCompletionSkipRuleConfig {
        enable: true,
        skip_policy: "SameNameSkipPolicy".to_string(),
        boundary_time: 4,
        last_run_gap_seconds: -1,
        ..TaskCompletionSkipRuleConfig::default()
    };

    assert!(
        is_skip_task(&project, Some(&config), &[daily], &before_boundary)
            .unwrap()
            .should_skip
    );

    let ineffective = TaskCompletionSkipRuleConfig {
        enable: true,
        boundary_time: -1,
        last_run_gap_seconds: -1,
        ..TaskCompletionSkipRuleConfig::default()
    };
    assert!(
        !is_skip_task(&project, Some(&ineffective), &[], &before_boundary)
            .unwrap()
            .should_skip
    );

    assert!(is_today_by_boundary(
        4,
        DateTime::parse_from_rfc3339("2026-06-26T04:00:00+08:00").unwrap(),
        false,
        &test_clock("2026-06-26T05:00:00", 8, "2026-06-26T05:00:00+08:00"),
    )
    .unwrap());
    assert!(!is_today_by_boundary(
        4,
        DateTime::parse_from_rfc3339("2026-06-27T04:00:00+08:00").unwrap(),
        false,
        &test_clock("2026-06-26T05:00:00", 8, "2026-06-26T05:00:00+08:00"),
    )
    .unwrap());
}

fn record(
    id: &str,
    group_name: &str,
    project_name: &str,
    folder_name: &str,
    project_type: &str,
) -> ExecutionRecord {
    ExecutionRecord {
        id: id.to_string(),
        group_name: group_name.to_string(),
        project_name: project_name.to_string(),
        folder_name: folder_name.to_string(),
        project_type: project_type.to_string(),
        server_start_time: Some("2026-06-26T08:00:00+08:00".to_string()),
        start_time: "2026-06-26T08:00:00+08:00".to_string(),
        server_end_time: Some("2026-06-26T08:10:00+08:00".to_string()),
        end_time: "2026-06-26T08:10:00+08:00".to_string(),
        is_successful: false,
    }
}

fn unsuccessful_record(
    id: &str,
    group_name: &str,
    project_name: &str,
    folder_name: &str,
    project_type: &str,
) -> ExecutionRecord {
    let mut record = record(id, group_name, project_name, folder_name, project_type);
    record.is_successful = false;
    record
}

fn write_daily(root: &Path, name: &str, execution_records: Vec<ExecutionRecord>) {
    fs::write(
        root.join(format!("{name}.json")),
        serde_json::to_string_pretty(&DailyExecutionRecord {
            name: name.to_string(),
            execution_records,
        })
        .unwrap(),
    )
    .unwrap();
}

fn read_daily(path: &Path) -> DailyExecutionRecord {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn test_clock(local: &str, local_offset_hours: i32, server: &str) -> ExecutionRecordClock {
    let local_offset = FixedOffset::east_opt(local_offset_hours * 3_600).unwrap();
    ExecutionRecordClock::fixed(
        NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M:%S").unwrap(),
        local_offset,
        DateTime::parse_from_rfc3339(server).unwrap(),
    )
}

fn test_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "bgi-script-execution-records-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
