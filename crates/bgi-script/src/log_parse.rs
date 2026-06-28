use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LogParseError {
    #[error("seconds cannot be negative")]
    NegativeSeconds,
    #[error("log parse I/O failed at {path}: {message}")]
    Io { path: PathBuf, message: String },
    #[error("log parse JSON failed at {path}: {message}")]
    Json { path: PathBuf, message: String },
}

pub type LogParseResult<T> = std::result::Result<T, LogParseError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogFileEntry {
    pub file_name: PathBuf,
    pub date: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogParseConfig {
    #[serde(rename = "Cookie", alias = "cookie")]
    pub cookie: String,
    #[serde(rename = "CookieDictionary", alias = "cookieDictionary")]
    pub cookie_dictionary: BTreeMap<String, LogGameInfo>,
    #[serde(
        rename = "ScriptGroupLogDictionary",
        alias = "scriptGroupLogDictionary"
    )]
    pub script_group_log_dictionary: BTreeMap<String, ScriptGroupLogParseConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScriptGroupLogParseConfig {
    #[serde(rename = "RangeValue", alias = "rangeValue")]
    pub range_value: String,
    #[serde(rename = "DayRangeValue", alias = "dayRangeValue")]
    pub day_range_value: String,
    #[serde(rename = "HoeingStatsSwitch", alias = "hoeingStatsSwitch")]
    pub hoeing_stats_switch: bool,
    #[serde(rename = "FaultStatsSwitch", alias = "faultStatsSwitch")]
    pub fault_stats_switch: bool,
    #[serde(rename = "GenerateFarmingPlanData", alias = "generateFarmingPlanData")]
    pub generate_farming_plan_data: bool,
    #[serde(rename = "HoeingDelay", alias = "hoeingDelay")]
    pub hoeing_delay: String,
    #[serde(rename = "MergerStatsSwitch", alias = "mergerStatsSwitch")]
    pub merger_stats_switch: bool,
}

impl Default for ScriptGroupLogParseConfig {
    fn default() -> Self {
        Self {
            range_value: "CurrentConfig".to_string(),
            day_range_value: "7".to_string(),
            hoeing_stats_switch: false,
            fault_stats_switch: false,
            generate_farming_plan_data: false,
            hoeing_delay: "0".to_string(),
            merger_stats_switch: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfigGroup {
    pub name: String,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub config_task_list: Vec<LogConfigTask>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfigTask {
    pub is_merger: bool,
    pub name: String,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub picks: BTreeMap<String, i32>,
    pub fault: LogFaultScenario,
}

impl LogConfigTask {
    pub fn add_pick(&mut self, value: impl Into<String>) {
        *self.picks.entry(value.into()).or_insert(0) += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogFaultScenario {
    pub pathing_success_end: bool,
    pub revive_count: i32,
    pub teleport_fail_count: i32,
    pub stuck_count: i32,
    pub retry_count: i32,
    pub battle_timeout_count: i32,
    pub err_count: i32,
}

impl Default for LogFaultScenario {
    fn default() -> Self {
        Self {
            pathing_success_end: true,
            revive_count: 0,
            teleport_fail_count: 0,
            stuck_count: 0,
            retry_count: 0,
            battle_timeout_count: 0,
            err_count: 0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogActionItem {
    #[serde(rename = "action_id")]
    pub action_id: i32,
    pub action: String,
    pub time: String,
    pub num: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogGameInfo {
    #[serde(rename = "game_biz")]
    pub game_biz: String,
    pub region: String,
    #[serde(rename = "game_uid")]
    pub game_uid: String,
    pub nickname: String,
    pub level: i32,
    #[serde(rename = "is_chosen")]
    pub is_chosen: bool,
    #[serde(rename = "region_name")]
    pub region_name: String,
    #[serde(rename = "is_official")]
    pub is_official: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogAnalysisOptions {
    pub merge_stats: bool,
    pub hoeing_delay_seconds: i64,
}

impl LogAnalysisOptions {
    pub fn from_script_group_config(config: &ScriptGroupLogParseConfig) -> Self {
        Self {
            merge_stats: config.merger_stats_switch,
            hoeing_delay_seconds: config.hoeing_delay.parse::<i64>().unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogAnalysisReport {
    pub config_groups: Vec<LogConfigGroup>,
    pub involved_months: Vec<(i32, u32)>,
    pub action_items: Vec<LogActionItem>,
    pub mora_by_custom_day: Vec<MoraStatisticsSummary>,
    pub all_mora: MoraStatisticsSummary,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MoraStatisticsSummary {
    pub name: String,
    pub small_monster_statistics: i32,
    pub small_monster_mora: i32,
    pub small_monster_details: String,
    pub last_small_time: Option<String>,
    pub elite_statistics: i32,
    pub elite_game_statistics: i32,
    pub elite_mora: i32,
    pub elite_details: String,
    pub last_elite_time: Option<String>,
    pub total_mora_killing_monsters: i32,
    pub other_mora: i32,
    pub all_mora: i32,
    pub emergency_bonus: String,
    pub chest_reward: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MoraStatistics {
    pub name: String,
    pub date: Option<NaiveDateTime>,
    pub statistics_start: Option<NaiveDateTime>,
    pub statistics_end: Option<NaiveDateTime>,
    pub action_items: Vec<LogActionItem>,
}

impl MoraStatistics {
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&LogActionItem) -> bool,
    {
        Self {
            action_items: self
                .action_items
                .iter()
                .filter(|item| predicate(item))
                .cloned()
                .collect(),
            ..Self::default()
        }
    }

    pub fn monster_action_items(&self) -> Vec<&LogActionItem> {
        self.action_items
            .iter()
            .filter(|item| item.action_id == 37)
            .collect()
    }

    pub fn elite_monster_action_items(&self) -> Vec<&LogActionItem> {
        self.monster_action_items()
            .into_iter()
            .filter(|item| item.num >= 200)
            .collect()
    }

    pub fn small_monster_action_items(&self) -> Vec<&LogActionItem> {
        self.monster_action_items()
            .into_iter()
            .filter(|item| item.num < 200)
            .collect()
    }

    pub fn emergency_bonus(&self) -> String {
        action_bonus_text(&self.action_items, 28)
    }

    pub fn chest_reward(&self) -> String {
        action_bonus_text(&self.action_items, 39)
    }

    pub fn last_elite_time(&self) -> Option<String> {
        self.elite_monster_action_items()
            .into_iter()
            .map(|item| item.time.as_str())
            .max()
            .map(ToOwned::to_owned)
    }

    pub fn last_small_time(&self) -> Option<String> {
        self.small_monster_action_items()
            .into_iter()
            .map(|item| item.time.as_str())
            .max()
            .map(ToOwned::to_owned)
    }

    pub fn elite_details(&self) -> String {
        grouped_count_text(
            self.elite_monster_action_items()
                .into_iter()
                .map(|item| item.num),
        )
    }

    pub fn elite_statistics(&self) -> i32 {
        self.elite_monster_action_items().len() as i32
    }

    pub fn elite_game_statistics(&self) -> i32 {
        self.elite_monster_action_items()
            .into_iter()
            .map(|item| {
                if item.num >= 3000 {
                    3
                } else if item.num >= 1200 {
                    2
                } else {
                    1
                }
            })
            .sum()
    }

    pub fn elite_mora(&self) -> i32 {
        self.elite_monster_action_items()
            .into_iter()
            .map(|item| item.num)
            .sum()
    }

    pub fn small_monster_statistics(&self) -> i32 {
        self.small_monster_action_items().len() as i32
    }

    pub fn small_monster_mora(&self) -> i32 {
        self.small_monster_action_items()
            .into_iter()
            .map(|item| item.num)
            .sum()
    }

    pub fn small_monster_details(&self) -> String {
        grouped_count_text(
            self.small_monster_action_items()
                .into_iter()
                .map(|item| item.num / 10),
        )
    }

    pub fn total_mora_killing_monsters(&self) -> i32 {
        self.monster_action_items()
            .into_iter()
            .map(|item| item.num)
            .sum()
    }

    pub fn other_mora(&self) -> i32 {
        self.action_items
            .iter()
            .filter(|item| item.action_id != 37)
            .map(|item| item.num)
            .sum()
    }

    pub fn all_mora(&self) -> i32 {
        self.action_items.iter().map(|item| item.num).sum()
    }
}

impl From<&MoraStatistics> for MoraStatisticsSummary {
    fn from(value: &MoraStatistics) -> Self {
        Self {
            name: value.name.clone(),
            small_monster_statistics: value.small_monster_statistics(),
            small_monster_mora: value.small_monster_mora(),
            small_monster_details: value.small_monster_details(),
            last_small_time: value.last_small_time(),
            elite_statistics: value.elite_statistics(),
            elite_game_statistics: value.elite_game_statistics(),
            elite_mora: value.elite_mora(),
            elite_details: value.elite_details(),
            last_elite_time: value.last_elite_time(),
            total_mora_killing_monsters: value.total_mora_killing_monsters(),
            other_mora: value.other_mora(),
            all_mora: value.all_mora(),
            emergency_bonus: value.emergency_bonus(),
            chest_reward: value.chest_reward(),
        }
    }
}

pub fn log_parse_config_path(app_root: impl AsRef<Path>) -> PathBuf {
    app_root
        .as_ref()
        .join("log")
        .join("logparse")
        .join("config.json")
}

pub fn load_log_parse_config(app_root: impl AsRef<Path>) -> LogParseConfig {
    let path = log_parse_config_path(app_root);
    let Ok(content) = fs::read_to_string(&path) else {
        return LogParseConfig::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn write_log_parse_config(
    app_root: impl AsRef<Path>,
    config: &LogParseConfig,
) -> LogParseResult<()> {
    let path = log_parse_config_path(app_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LogParseError::Io {
            path: parent.to_path_buf(),
            message: source.to_string(),
        })?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|source| LogParseError::Json {
        path: path.clone(),
        message: source.to_string(),
    })?;
    fs::write(&path, content).map_err(|source| LogParseError::Io {
        path,
        message: source.to_string(),
    })
}

pub fn discover_log_files(folder_path: impl AsRef<Path>) -> LogParseResult<Vec<LogFileEntry>> {
    let folder_path = folder_path.as_ref();
    if !folder_path.is_dir() {
        return Ok(Vec::new());
    }

    let file_regex = Regex::new(r"^better-genshin-impact(\d{8})(_\d{3})*\.log$")
        .expect("legacy log filename regex is valid");
    let mut result = Vec::new();
    for entry in fs::read_dir(folder_path).map_err(|source| LogParseError::Io {
        path: folder_path.to_path_buf(),
        message: source.to_string(),
    })? {
        let entry = entry.map_err(|source| LogParseError::Io {
            path: folder_path.to_path_buf(),
            message: source.to_string(),
        })?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(captures) = file_regex.captures(&file_name) else {
            continue;
        };
        let Some(raw_date) = captures.get(1) else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(raw_date.as_str(), "%Y%m%d") else {
            continue;
        };
        result.push(LogFileEntry {
            file_name: entry.path(),
            date: date.format("%Y-%m-%d").to_string(),
        });
    }
    result.sort_by(|left, right| left.date.cmp(&right.date));
    Ok(result)
}

pub fn safe_read_log_lines(file_path: impl AsRef<Path>) -> Vec<String> {
    fs::read_to_string(file_path)
        .map(|content| content.lines().map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

pub fn parse_log_file_entries(log_files: &[LogFileEntry]) -> Vec<LogConfigGroup> {
    let mut log_lines = Vec::new();
    for log_file in log_files {
        for line in safe_read_log_lines(&log_file.file_name) {
            log_lines.push((line, log_file.date.clone()));
        }
    }
    parse_log_lines(&log_lines)
}

pub fn parse_log_files(folder_path: impl AsRef<Path>) -> LogParseResult<Vec<LogConfigGroup>> {
    let log_files = discover_log_files(folder_path)?;
    Ok(parse_log_file_entries(&log_files))
}

pub fn analyze_log_lines(
    log_lines: &[(impl AsRef<str>, impl AsRef<str>)],
    action_items: &[LogActionItem],
    options: &LogAnalysisOptions,
) -> LogAnalysisReport {
    analyze_log_groups(parse_log_lines(log_lines), action_items, options)
}

pub fn analyze_log_groups(
    mut config_groups: Vec<LogConfigGroup>,
    action_items: &[LogActionItem],
    options: &LogAnalysisOptions,
) -> LogAnalysisReport {
    config_groups.retain(|group| !group.config_task_list.is_empty());
    if options.merge_stats {
        config_groups = merge_log_config_groups(&config_groups);
        config_groups.reverse();
    }

    let mut action_items = filter_travel_diary_mora_items(action_items);
    if options.hoeing_delay_seconds != 0 {
        for action_item in &mut action_items {
            action_item.time = subtract_seconds(&action_item.time, options.hoeing_delay_seconds);
        }
        action_items.sort_by(|left, right| left.time.cmp(&right.time));
    }

    let mut by_day: BTreeMap<NaiveDateTime, Vec<LogActionItem>> = BTreeMap::new();
    for action_item in &action_items {
        if let Some(day) = log_parse_custom_day_start(&action_item.time) {
            by_day.entry(day).or_default().push(action_item.clone());
        }
    }
    let mora_by_custom_day = by_day
        .into_iter()
        .rev()
        .map(|(day, action_items)| {
            let stats = MoraStatistics {
                name: day.format("%Y-%m-%d").to_string(),
                date: Some(day),
                action_items,
                ..MoraStatistics::default()
            };
            MoraStatisticsSummary::from(&stats)
        })
        .collect();

    let all_stats = MoraStatistics {
        action_items: action_items.clone(),
        ..MoraStatistics::default()
    };
    LogAnalysisReport {
        involved_months: involved_months(&config_groups),
        config_groups,
        action_items,
        mora_by_custom_day,
        all_mora: MoraStatisticsSummary::from(&all_stats),
    }
}

pub fn parse_log_lines(log_lines: &[(impl AsRef<str>, impl AsRef<str>)]) -> Vec<LogConfigGroup> {
    let lines = log_lines
        .iter()
        .map(|(line, date)| (line.as_ref().to_string(), date.as_ref().to_string()))
        .collect::<Vec<_>>();
    let mut groups: Vec<LogConfigGroup> = Vec::new();
    let mut current_group_index: Option<usize> = None;
    let mut current_task_index: Option<usize> = None;

    for (index, (line, date)) in lines.iter().enumerate() {
        if let Some(captures) = parse_bgi_line(r#"配置组 "(.+?)" 加载完成，共(\d+)个脚本"#, line)
        {
            let group = LogConfigGroup {
                name: captures.get(1).cloned().unwrap_or_default(),
                start_date: parse_previous_datetime(&lines, index as isize - 1, date),
                ..LogConfigGroup::default()
            };
            groups.push(group);
            current_group_index = Some(groups.len() - 1);
            current_task_index = None;
        }

        let Some(group_index) = current_group_index else {
            continue;
        };

        let group_name = groups[group_index].name.clone();
        let group_end_pattern = format!(r#"配置组 "{group_name}" 执行结束"#);
        if parse_bgi_line(&group_end_pattern, line).is_some() {
            groups[group_index].end_date =
                parse_previous_datetime(&lines, index as isize - 1, date);
            current_group_index = None;
            current_task_index = None;
            continue;
        }

        if let Some(captures) =
            parse_bgi_line(r#"→ 开始执行(?:地图追踪任务|JS脚本): "(.+?)""#, line)
        {
            let task = LogConfigTask {
                name: captures.get(1).cloned().unwrap_or_default(),
                start_date: parse_previous_datetime(&lines, index as isize - 1, date),
                ..LogConfigTask::default()
            };
            groups[group_index].config_task_list.push(task);
            current_task_index = Some(groups[group_index].config_task_list.len() - 1);
        }

        let Some(task_index) = current_task_index else {
            continue;
        };

        if line.contains("此追踪脚本未正常走完！") {
            groups[group_index].config_task_list[task_index]
                .fault
                .pathing_success_end = false;
        }
        if line.ends_with("前往七天神像复活") {
            groups[group_index].config_task_list[task_index]
                .fault
                .revive_count += 1;
        }
        if let Some(captures) = parse_bgi_line(r#"传送失败，重试 (\d+) 次"#, line) {
            if let Some(value) = captures.get(1).and_then(|value| value.parse::<i32>().ok()) {
                groups[group_index].config_task_list[task_index]
                    .fault
                    .teleport_fail_count = value;
            }
        }
        if line == "战斗超时结束" {
            groups[group_index].config_task_list[task_index]
                .fault
                .battle_timeout_count += 1;
        }
        if line.ends_with("重试一次路线或放弃此路线！") {
            groups[group_index].config_task_list[task_index]
                .fault
                .retry_count += 1;
        }
        if line == "疑似卡死，尝试脱离..." {
            groups[group_index].config_task_list[task_index]
                .fault
                .stuck_count += 1;
        }
        if parse_bgi_line(r#"执行脚本时发生异常: "(.+?)""#, line).is_some() {
            groups[group_index].config_task_list[task_index]
                .fault
                .err_count += 1;
        }

        let task_name = groups[group_index].config_task_list[task_index]
            .name
            .clone();
        if line.starts_with(&format!("→ 脚本执行结束: \"{task_name}\"")) {
            groups[group_index].config_task_list[task_index].end_date =
                parse_previous_datetime(&lines, index as isize - 1, date);
            current_task_index = None;
            continue;
        }

        if let Some(captures) = parse_bgi_line(r#"交互或拾取："(.+?)""#, line) {
            if let Some(value) = captures.get(1) {
                groups[group_index].config_task_list[task_index].add_pick(value.clone());
            }
        }
    }

    for group in &mut groups {
        if group.end_date.is_none() {
            if let Some(last_task) = group.config_task_list.last() {
                group.end_date = last_task.end_date.or(last_task.start_date);
            }
        }
    }

    groups
}

pub fn parse_bgi_line(pattern: &str, value: &str) -> Option<Vec<String>> {
    let regex = Regex::new(pattern).ok()?;
    regex.captures(value).map(|captures| {
        captures
            .iter()
            .map(|capture| {
                capture
                    .map(|capture| capture.as_str().to_string())
                    .unwrap_or_default()
            })
            .collect()
    })
}

pub fn merge_log_config_groups(groups: &[LogConfigGroup]) -> Vec<LogConfigGroup> {
    if groups.is_empty() {
        return Vec::new();
    }

    let mut reversed_groups = groups.to_vec();
    reversed_groups.reverse();

    let mut result = Vec::new();
    let mut current = reversed_groups[0].clone();
    for next in reversed_groups.iter().skip(1) {
        if current.name == next.name
            && are_dates_in_same_custom_day(current.start_date, next.start_date)
        {
            current = LogConfigGroup {
                name: current.name,
                start_date: min_datetime(current.start_date, next.start_date),
                end_date: max_datetime(current.end_date, next.end_date),
                config_task_list: merge_log_config_task_lists(
                    &current.config_task_list,
                    &next.config_task_list,
                ),
            };
        } else {
            result.push(current);
            current = next.clone();
        }
    }
    result.push(current);
    result
}

pub fn merge_log_config_task_lists(
    list1: &[LogConfigTask],
    list2: &[LogConfigTask],
) -> Vec<LogConfigTask> {
    if list1.is_empty() {
        return list2.to_vec();
    }
    if list2.is_empty() {
        return list1.to_vec();
    }

    let mut merged_list = list1.to_vec();
    let cloned_list2 = list2.to_vec();
    let last_task = merged_list.last().expect("list1 is not empty").clone();
    let first_task = cloned_list2.first().expect("list2 is not empty").clone();

    if last_task.name == first_task.name {
        let merged_task = LogConfigTask {
            is_merger: true,
            name: last_task.name,
            start_date: min_datetime(last_task.start_date, first_task.start_date),
            end_date: max_datetime(last_task.end_date, first_task.end_date),
            picks: merge_pick_dictionaries(&last_task.picks, &first_task.picks),
            fault: if datetime_or_min(last_task.start_date)
                >= datetime_or_min(first_task.start_date)
            {
                last_task.fault
            } else {
                first_task.fault
            },
        };
        let last_index = merged_list.len() - 1;
        merged_list[last_index] = merged_task;
        merged_list.extend(cloned_list2.into_iter().skip(1));
    } else {
        merged_list.extend(cloned_list2);
    }

    merged_list
}

pub fn merge_pick_dictionaries(
    first: &BTreeMap<String, i32>,
    second: &BTreeMap<String, i32>,
) -> BTreeMap<String, i32> {
    let mut result = BTreeMap::new();
    for (key, value) in first.iter().chain(second.iter()) {
        *result.entry(key.clone()).or_insert(0) += value;
    }
    result
}

pub fn are_dates_in_same_custom_day(
    left: Option<NaiveDateTime>,
    right: Option<NaiveDateTime>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            (left - Duration::hours(4)).date() == (right - Duration::hours(4)).date()
        }
        _ => false,
    }
}

pub fn log_parse_custom_day_start(value: &str) -> Option<NaiveDateTime> {
    let time = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok()?;
    let midnight = time.date().and_hms_opt(0, 0, 0)?;
    let custom_day_start = midnight + Duration::hours(4);
    if time < custom_day_start {
        Some(custom_day_start - Duration::days(1))
    } else {
        Some(custom_day_start)
    }
}

pub fn involved_months(config_groups: &[LogConfigGroup]) -> Vec<(i32, u32)> {
    let mut months = BTreeSet::new();
    for group in config_groups {
        if let Some(start_date) = group.start_date {
            months.insert((start_date.date().year(), start_date.date().month()));
        }
        if let Some(end_date) = group.end_date {
            months.insert((end_date.date().year(), end_date.date().month()));
        }
    }
    months.into_iter().collect()
}

pub fn filter_travel_diary_mora_items(items: &[LogActionItem]) -> Vec<LogActionItem> {
    let mut result = items
        .iter()
        .filter(|item| matches!(item.action_id, 28 | 37 | 39))
        .cloned()
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.time.cmp(&right.time));
    result
}

pub fn convert_seconds_to_time(total_seconds: f64) -> Result<String, LogParseError> {
    if total_seconds < 0.0 {
        return Err(LogParseError::NegativeSeconds);
    }

    let hours = (total_seconds / 3600.0) as i32;
    let minutes = ((total_seconds % 3600.0) / 60.0) as i32;
    let seconds = total_seconds % 60.0;

    let mut result = String::new();
    if hours > 0 {
        result.push_str(&format!("{hours}小时"));
    }
    if minutes > 0 || hours > 0 {
        result.push_str(&format!("{minutes}分钟"));
    }
    if seconds > 0.0 || (hours == 0 && minutes == 0) {
        if seconds.fract() == 0.0 {
            result.push_str(&format!("{}秒", seconds as i32));
        } else {
            result.push_str(&format!("{seconds:.2}秒"));
        }
    }

    Ok(result)
}

pub fn format_number_with_style(value: i32, threshold: i32) -> String {
    if value == 0 {
        return String::new();
    }
    let color_style = if value >= threshold { "color:red;" } else { "" };
    format!(r#"<span style="font-weight:bold;{color_style}">{value}</span>"#)
}

pub fn number_or_empty_string(value: i32) -> String {
    if value == 0 {
        String::new()
    } else {
        value.to_string()
    }
}

pub fn subtract_seconds(input_time: &str, seconds: i64) -> String {
    match NaiveDateTime::parse_from_str(input_time, "%Y-%m-%d %H:%M:%S") {
        Ok(time) => (time - Duration::seconds(seconds))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        Err(_) => "Invalid input time format. Please use 'yyyy-MM-dd HH:mm:ss'.".to_string(),
    }
}

fn parse_previous_datetime(
    lines: &[(String, String)],
    index: isize,
    log_date: &str,
) -> Option<NaiveDateTime> {
    if index < 0 {
        return None;
    }
    let line = lines.get(index as usize)?.0.as_str();
    let captures = parse_bgi_line(r"\[(\d{2}:\d{2}:\d{2})\.\d+\]", line)?;
    let time = captures.get(1)?;
    NaiveDateTime::parse_from_str(&format!("{log_date} {time}"), "%Y-%m-%d %H:%M:%S").ok()
}

fn min_datetime(
    left: Option<NaiveDateTime>,
    right: Option<NaiveDateTime>,
) -> Option<NaiveDateTime> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn max_datetime(
    left: Option<NaiveDateTime>,
    right: Option<NaiveDateTime>,
) -> Option<NaiveDateTime> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn datetime_or_min(value: Option<NaiveDateTime>) -> NaiveDateTime {
    value.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(1, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    })
}

fn action_bonus_text(items: &[LogActionItem], action_id: i32) -> String {
    let matching = items
        .iter()
        .filter(|item| item.action_id == action_id)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return String::new();
    }
    let sum = matching.iter().map(|item| item.num).sum::<i32>();
    if matching.len() >= 10 {
        sum.to_string()
    } else {
        format!("{sum}({}/10)", matching.len())
    }
}

fn grouped_count_text(values: impl IntoIterator<Item = i32>) -> String {
    let mut groups = BTreeMap::new();
    for value in values {
        *groups.entry(value).or_insert(0) += 1;
    }
    groups
        .into_iter()
        .map(|(value, count)| format!("{value}*{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "log_parse_tests.rs"]
mod tests;
