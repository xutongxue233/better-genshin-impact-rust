use super::*;
use chrono::NaiveDateTime;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn dt(value: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn action(action_id: i32, time: &str, num: i32) -> LogActionItem {
    LogActionItem {
        action_id,
        action: String::new(),
        time: time.to_string(),
        num,
    }
}

fn task(name: &str, start: &str, end: &str) -> LogConfigTask {
    LogConfigTask {
        name: name.to_string(),
        start_date: Some(dt(start)),
        end_date: Some(dt(end)),
        ..LogConfigTask::default()
    }
}

fn test_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("bgi-log-parse-{name}-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn parse_bgi_line_returns_full_match_and_captures() {
    let captures = parse_bgi_line(
        r#"配置组 "(.+?)" 加载完成，共(\d+)个脚本"#,
        r#"配置组 "每日" 加载完成，共25个脚本，开始执行"#,
    )
    .unwrap();

    assert_eq!(
        captures,
        vec![
            r#"配置组 "每日" 加载完成，共25个脚本"#.to_string(),
            "每日".to_string(),
            "25".to_string()
        ]
    );
}

#[test]
fn discover_log_files_matches_legacy_filename_pattern_and_sorts_by_date() {
    let root = test_root("discover");
    fs::write(root.join("better-genshin-impact20260628.log"), "").unwrap();
    fs::write(root.join("better-genshin-impact20260627_001.log"), "").unwrap();
    fs::write(root.join("better-genshin-impact20260626_001_002.log"), "").unwrap();
    fs::write(root.join("better-genshin-impact20260629.txt"), "").unwrap();
    fs::write(root.join("other20260625.log"), "").unwrap();

    let files = discover_log_files(&root).unwrap();

    assert_eq!(
        files
            .iter()
            .map(|file| file.date.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-06-26", "2026-06-27", "2026-06-28"]
    );
    assert_eq!(
        files
            .iter()
            .map(|file| {
                file.file_name
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>(),
        vec![
            "better-genshin-impact20260626_001_002.log",
            "better-genshin-impact20260627_001.log",
            "better-genshin-impact20260628.log"
        ]
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parse_log_file_entries_reads_discovered_files_with_their_legacy_dates() {
    let root = test_root("parse-files");
    fs::write(
        root.join("better-genshin-impact20260628.log"),
        [
            "[10:00:00.000] INFO",
            r#"配置组 "每日" 加载完成，共1个脚本"#,
            "[10:01:00.000] INFO",
            r#"→ 开始执行JS脚本: "foo.js""#,
            "[10:02:00.000] INFO",
            r#"→ 脚本执行结束: "foo.js""#,
        ]
        .join("\n"),
    )
    .unwrap();

    let groups = parse_log_files(&root).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "每日");
    assert_eq!(groups[0].start_date, Some(dt("2026-06-28 10:00:00")));
    assert_eq!(groups[0].end_date, Some(dt("2026-06-28 10:02:00")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn log_parse_config_defaults_paths_and_pascal_case_json_match_legacy_shape() {
    let root = test_root("config");
    assert!(log_parse_config_path(&root).ends_with("log/logparse/config.json"));
    assert_eq!(load_log_parse_config(&root), LogParseConfig::default());

    let config = LogParseConfig {
        cookie: "cookie=value".to_string(),
        script_group_log_dictionary: BTreeMap::from([(
            "Daily".to_string(),
            ScriptGroupLogParseConfig {
                range_value: "CurrentConfig".to_string(),
                day_range_value: "3".to_string(),
                hoeing_stats_switch: true,
                fault_stats_switch: true,
                generate_farming_plan_data: true,
                hoeing_delay: "5".to_string(),
                merger_stats_switch: true,
            },
        )]),
        ..LogParseConfig::default()
    };
    write_log_parse_config(&root, &config).unwrap();
    let content = fs::read_to_string(log_parse_config_path(&root)).unwrap();
    assert!(content.contains("\"Cookie\""));
    assert!(content.contains("\"ScriptGroupLogDictionary\""));
    assert!(content.contains("\"HoeingDelay\""));
    assert_eq!(load_log_parse_config(&root), config);

    fs::write(log_parse_config_path(&root), "{not json").unwrap();
    assert_eq!(load_log_parse_config(&root), LogParseConfig::default());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn log_parse_config_accepts_camel_case_aliases_and_preserves_defaults() {
    let parsed: LogParseConfig = serde_json::from_str(
        r#"{
            "cookie": "abc",
            "scriptGroupLogDictionary": {
                "Daily": {
                    "hoeingStatsSwitch": true,
                    "hoeingDelay": "7"
                }
            }
        }"#,
    )
    .unwrap();

    assert_eq!(parsed.cookie, "abc");
    let daily = parsed.script_group_log_dictionary.get("Daily").unwrap();
    assert!(daily.hoeing_stats_switch);
    assert_eq!(daily.hoeing_delay, "7");
    assert_eq!(daily.range_value, "CurrentConfig");
    assert_eq!(daily.day_range_value, "7");
}

#[test]
fn parse_log_lines_preserves_legacy_group_task_fault_and_pick_state() {
    let lines = vec![
        ("[10:00:00.123] INFO", "2026-06-26"),
        (
            r#"配置组 "每日" 加载完成，共2个脚本，开始执行"#,
            "2026-06-26",
        ),
        ("[10:01:00.000] INFO", "2026-06-26"),
        (r#"→ 开始执行地图追踪任务: "route-a.json""#, "2026-06-26"),
        (r#"交互或拾取："薄荷""#, "2026-06-26"),
        (r#"交互或拾取："薄荷""#, "2026-06-26"),
        (r#"交互或拾取："甜甜花""#, "2026-06-26"),
        ("此追踪脚本未正常走完！", "2026-06-26"),
        ("前往七天神像复活", "2026-06-26"),
        ("传送失败，重试 2 次", "2026-06-26"),
        ("传送失败，重试 4 次", "2026-06-26"),
        ("战斗超时结束", "2026-06-26"),
        ("疑似卡死，尝试脱离...", "2026-06-26"),
        ("执行脚本时发生异常: \"boom\"", "2026-06-26"),
        ("触发重试一次路线或放弃此路线！", "2026-06-26"),
        ("[10:05:00.000] INFO", "2026-06-26"),
        (r#"→ 脚本执行结束: "route-a.json""#, "2026-06-26"),
        ("[10:06:00.000] INFO", "2026-06-26"),
        (r#"配置组 "每日" 执行结束"#, "2026-06-26"),
    ];

    let groups = parse_log_lines(&lines);

    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.name, "每日");
    assert_eq!(group.start_date, Some(dt("2026-06-26 10:00:00")));
    assert_eq!(group.end_date, Some(dt("2026-06-26 10:06:00")));
    assert_eq!(group.config_task_list.len(), 1);

    let task = &group.config_task_list[0];
    assert_eq!(task.name, "route-a.json");
    assert_eq!(task.start_date, Some(dt("2026-06-26 10:01:00")));
    assert_eq!(task.end_date, Some(dt("2026-06-26 10:05:00")));
    assert_eq!(task.picks.get("薄荷"), Some(&2));
    assert_eq!(task.picks.get("甜甜花"), Some(&1));
    assert!(!task.fault.pathing_success_end);
    assert_eq!(task.fault.revive_count, 1);
    assert_eq!(task.fault.teleport_fail_count, 4);
    assert_eq!(task.fault.battle_timeout_count, 1);
    assert_eq!(task.fault.stuck_count, 1);
    assert_eq!(task.fault.retry_count, 1);
    assert_eq!(task.fault.err_count, 1);
}

#[test]
fn parse_log_lines_falls_back_group_end_to_last_task_time() {
    let lines = vec![
        ("[23:00:00.000] INFO", "2026-06-26"),
        (r#"配置组 "夜间" 加载完成，共1个脚本"#, "2026-06-26"),
        ("[23:01:00.000] INFO", "2026-06-26"),
        (r#"→ 开始执行JS脚本: "script.js""#, "2026-06-26"),
        ("[23:02:00.000] INFO", "2026-06-26"),
        (r#"→ 脚本执行结束: "script.js""#, "2026-06-26"),
    ];

    let groups = parse_log_lines(&lines);

    assert_eq!(groups[0].end_date, Some(dt("2026-06-26 23:02:00")));
}

#[test]
fn merge_config_groups_uses_four_am_day_and_merges_boundary_task() {
    let mut older = LogConfigGroup {
        name: "每日".to_string(),
        start_date: Some(dt("2026-06-26 23:00:00")),
        end_date: Some(dt("2026-06-26 23:30:00")),
        config_task_list: vec![task(
            "route.json",
            "2026-06-26 23:00:00",
            "2026-06-26 23:30:00",
        )],
    };
    older.config_task_list[0].add_pick("薄荷");
    older.config_task_list[0].fault.revive_count = 1;

    let mut newer = LogConfigGroup {
        name: "每日".to_string(),
        start_date: Some(dt("2026-06-27 03:00:00")),
        end_date: Some(dt("2026-06-27 03:20:00")),
        config_task_list: vec![task(
            "route.json",
            "2026-06-27 03:00:00",
            "2026-06-27 03:20:00",
        )],
    };
    newer.config_task_list[0].add_pick("薄荷");
    newer.config_task_list[0].add_pick("甜甜花");
    newer.config_task_list[0].fault.err_count = 2;

    let merged = merge_log_config_groups(&[older, newer]);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].start_date, Some(dt("2026-06-26 23:00:00")));
    assert_eq!(merged[0].end_date, Some(dt("2026-06-27 03:20:00")));
    let merged_task = &merged[0].config_task_list[0];
    assert!(merged_task.is_merger);
    assert_eq!(merged_task.start_date, Some(dt("2026-06-26 23:00:00")));
    assert_eq!(merged_task.end_date, Some(dt("2026-06-27 03:20:00")));
    assert_eq!(merged_task.picks.get("薄荷"), Some(&2));
    assert_eq!(merged_task.picks.get("甜甜花"), Some(&1));
    assert_eq!(merged_task.fault.err_count, 2);
    assert_eq!(merged_task.fault.revive_count, 0);
}

#[test]
fn merge_config_groups_does_not_merge_across_four_am_boundary() {
    let older = LogConfigGroup {
        name: "每日".to_string(),
        start_date: Some(dt("2026-06-27 03:59:59")),
        ..LogConfigGroup::default()
    };
    let newer = LogConfigGroup {
        name: "每日".to_string(),
        start_date: Some(dt("2026-06-27 04:00:00")),
        ..LogConfigGroup::default()
    };

    let merged = merge_log_config_groups(&[older, newer]);

    assert_eq!(merged.len(), 2);
}

#[test]
fn merge_pick_dictionaries_sums_matching_keys() {
    let first = BTreeMap::from([("薄荷".to_string(), 2), ("甜甜花".to_string(), 1)]);
    let second = BTreeMap::from([("薄荷".to_string(), 3), ("树莓".to_string(), 4)]);

    let merged = merge_pick_dictionaries(&first, &second);

    assert_eq!(merged.get("薄荷"), Some(&5));
    assert_eq!(merged.get("甜甜花"), Some(&1));
    assert_eq!(merged.get("树莓"), Some(&4));
}

#[test]
fn convert_seconds_to_time_matches_legacy_formatting() {
    assert_eq!(convert_seconds_to_time(0.0).unwrap(), "0秒");
    assert_eq!(convert_seconds_to_time(59.0).unwrap(), "59秒");
    assert_eq!(convert_seconds_to_time(60.0).unwrap(), "1分钟");
    assert_eq!(convert_seconds_to_time(61.0).unwrap(), "1分钟1秒");
    assert_eq!(convert_seconds_to_time(3600.0).unwrap(), "1小时0分钟");
    assert_eq!(convert_seconds_to_time(3661.5).unwrap(), "1小时1分钟1.50秒");
    assert_eq!(
        convert_seconds_to_time(-1.0).unwrap_err(),
        LogParseError::NegativeSeconds
    );
}

#[test]
fn custom_day_format_and_subtract_helpers_match_legacy_behavior() {
    assert_eq!(
        log_parse_custom_day_start("2026-06-27 03:59:59"),
        Some(dt("2026-06-26 04:00:00"))
    );
    assert_eq!(
        log_parse_custom_day_start("2026-06-27 04:00:00"),
        Some(dt("2026-06-27 04:00:00"))
    );
    assert_eq!(
        format_number_with_style(2, 3),
        r#"<span style="font-weight:bold;">2</span>"#
    );
    assert_eq!(
        format_number_with_style(3, 3),
        r#"<span style="font-weight:bold;color:red;">3</span>"#
    );
    assert_eq!(format_number_with_style(0, 3), "");
    assert_eq!(number_or_empty_string(0), "");
    assert_eq!(number_or_empty_string(8), "8");
    assert_eq!(
        subtract_seconds("2026-06-27 04:00:01", 5),
        "2026-06-27 03:59:56"
    );
    assert_eq!(
        subtract_seconds("bad", 5),
        "Invalid input time format. Please use 'yyyy-MM-dd HH:mm:ss'."
    );
}

#[test]
fn mora_statistics_matches_legacy_monster_bonus_and_details_rules() {
    let stats = MoraStatistics {
        name: "2026-06-27".to_string(),
        action_items: vec![
            action(37, "2026-06-27 04:10:00", 80),
            action(37, "2026-06-27 04:20:00", 120),
            action(37, "2026-06-27 04:30:00", 200),
            action(37, "2026-06-27 04:40:00", 1200),
            action(37, "2026-06-27 04:50:00", 3000),
            action(28, "2026-06-27 05:00:00", 1000),
            action(28, "2026-06-27 05:10:00", 1000),
            action(39, "2026-06-27 05:20:00", 500),
        ],
        ..MoraStatistics::default()
    };

    assert_eq!(stats.small_monster_statistics(), 2);
    assert_eq!(stats.small_monster_mora(), 200);
    assert_eq!(stats.small_monster_details(), "8*1, 12*1");
    assert_eq!(
        stats.last_small_time(),
        Some("2026-06-27 04:20:00".to_string())
    );
    assert_eq!(stats.elite_statistics(), 3);
    assert_eq!(stats.elite_game_statistics(), 6);
    assert_eq!(stats.elite_mora(), 4400);
    assert_eq!(stats.elite_details(), "200*1, 1200*1, 3000*1");
    assert_eq!(
        stats.last_elite_time(),
        Some("2026-06-27 04:50:00".to_string())
    );
    assert_eq!(stats.total_mora_killing_monsters(), 4600);
    assert_eq!(stats.other_mora(), 2500);
    assert_eq!(stats.all_mora(), 7100);
    assert_eq!(stats.emergency_bonus(), "2000(2/10)");
    assert_eq!(stats.chest_reward(), "500(1/10)");

    let filtered = stats.filter(|item| item.action_id == 37 && item.num >= 1200);
    assert_eq!(filtered.action_items.len(), 2);
    assert!(filtered.name.is_empty());
}

#[test]
fn involved_months_and_travel_diary_filter_are_sorted_and_deduplicated() {
    let groups = vec![
        LogConfigGroup {
            start_date: Some(dt("2026-06-30 23:00:00")),
            end_date: Some(dt("2026-07-01 00:10:00")),
            ..LogConfigGroup::default()
        },
        LogConfigGroup {
            start_date: Some(dt("2026-06-01 04:00:00")),
            ..LogConfigGroup::default()
        },
    ];
    assert_eq!(involved_months(&groups), vec![(2026, 6), (2026, 7)]);

    let filtered = filter_travel_diary_mora_items(&[
        action(37, "2026-06-27 05:00:00", 200),
        action(1, "2026-06-27 04:00:00", 60),
        action(28, "2026-06-27 03:00:00", 1000),
        action(39, "2026-06-27 04:30:00", 500),
    ]);

    assert_eq!(
        filtered
            .iter()
            .map(|item| item.action_id)
            .collect::<Vec<_>>(),
        vec![28, 39, 37]
    );
}

#[test]
fn analyze_log_lines_returns_structured_report_for_tauri_boundary() {
    let lines = vec![
        ("[03:50:00.000] INFO", "2026-06-27"),
        (r#"配置组 "每日" 加载完成，共1个脚本"#, "2026-06-27"),
        ("[03:51:00.000] INFO", "2026-06-27"),
        (r#"→ 开始执行地图追踪任务: "route.json""#, "2026-06-27"),
        ("[03:55:00.000] INFO", "2026-06-27"),
        (r#"→ 脚本执行结束: "route.json""#, "2026-06-27"),
        ("[03:56:00.000] INFO", "2026-06-27"),
        (r#"配置组 "每日" 执行结束"#, "2026-06-27"),
        ("[04:10:00.000] INFO", "2026-06-27"),
        (r#"配置组 "每日" 加载完成，共1个脚本"#, "2026-06-27"),
        ("[04:11:00.000] INFO", "2026-06-27"),
        (r#"→ 开始执行地图追踪任务: "route.json""#, "2026-06-27"),
        ("[04:15:00.000] INFO", "2026-06-27"),
        (r#"→ 脚本执行结束: "route.json""#, "2026-06-27"),
        ("[04:16:00.000] INFO", "2026-06-27"),
        (r#"配置组 "每日" 执行结束"#, "2026-06-27"),
    ];
    let options = LogAnalysisOptions {
        merge_stats: true,
        hoeing_delay_seconds: 5,
    };

    let report = analyze_log_lines(
        &lines,
        &[
            action(37, "2026-06-27 04:00:03", 200),
            action(37, "2026-06-27 04:00:06", 80),
            action(1, "2026-06-27 04:00:07", 60),
            action(28, "2026-06-27 04:20:00", 1000),
        ],
        &options,
    );

    assert_eq!(report.config_groups.len(), 2);
    assert_eq!(report.involved_months, vec![(2026, 6)]);
    assert_eq!(
        report
            .action_items
            .iter()
            .map(|item| (item.action_id, item.time.as_str(), item.num))
            .collect::<Vec<_>>(),
        vec![
            (37, "2026-06-27 03:59:58", 200),
            (37, "2026-06-27 04:00:01", 80),
            (28, "2026-06-27 04:19:55", 1000)
        ]
    );
    assert_eq!(report.mora_by_custom_day.len(), 2);
    assert_eq!(report.mora_by_custom_day[0].name, "2026-06-27");
    assert_eq!(report.mora_by_custom_day[0].small_monster_statistics, 1);
    assert_eq!(report.mora_by_custom_day[1].name, "2026-06-26");
    assert_eq!(report.mora_by_custom_day[1].elite_game_statistics, 1);
    assert_eq!(report.all_mora.total_mora_killing_monsters, 280);
    assert_eq!(report.all_mora.other_mora, 1000);
    assert_eq!(report.all_mora.emergency_bonus, "1000(1/10)");
}

#[test]
fn analyze_log_groups_accepts_preparsed_and_filtered_groups() {
    let groups = vec![
        LogConfigGroup {
            name: "空组".to_string(),
            start_date: Some(dt("2026-06-27 03:30:00")),
            end_date: Some(dt("2026-06-27 03:40:00")),
            ..LogConfigGroup::default()
        },
        LogConfigGroup {
            name: "每日".to_string(),
            start_date: Some(dt("2026-06-27 04:00:00")),
            end_date: Some(dt("2026-06-27 04:30:00")),
            config_task_list: vec![task(
                "route.json",
                "2026-06-27 04:05:00",
                "2026-06-27 04:25:00",
            )],
        },
    ];

    let report = analyze_log_groups(
        groups,
        &[action(37, "2026-06-27 04:10:00", 80)],
        &LogAnalysisOptions::default(),
    );

    assert_eq!(report.config_groups.len(), 1);
    assert_eq!(report.config_groups[0].name, "每日");
    assert_eq!(report.action_items.len(), 1);
    assert_eq!(report.all_mora.total_mora_killing_monsters, 80);
}
