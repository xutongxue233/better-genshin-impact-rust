use crate::group::{ScriptGroup, ScriptGroupProject, ScriptProjectType};
use chrono::{
    DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, Offset,
    SecondsFormat, TimeZone, Timelike,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ExecutionRecordStorageError {
    #[error("days must be a positive integer")]
    InvalidDays,
    #[error("boundary hour must be between 0 and 23")]
    InvalidBoundaryHour,
    #[error("execution record time field `{field}` is invalid: {value}")]
    InvalidTime { field: &'static str, value: String },
    #[error("execution record I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("execution record JSON failed at {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

pub type ExecutionRecordResult<T> = std::result::Result<T, ExecutionRecordStorageError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecutionRecord {
    #[serde(rename = "guid")]
    pub id: String,
    #[serde(rename = "group_name")]
    pub group_name: String,
    #[serde(rename = "project_name")]
    pub project_name: String,
    #[serde(rename = "folder_name")]
    pub folder_name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    #[serde(rename = "server_start_time")]
    pub server_start_time: Option<String>,
    #[serde(rename = "start_time")]
    pub start_time: String,
    #[serde(rename = "server_end_time")]
    pub server_end_time: Option<String>,
    #[serde(rename = "end_time")]
    pub end_time: String,
    #[serde(rename = "is_successful")]
    pub is_successful: bool,
}

impl Default for ExecutionRecord {
    fn default() -> Self {
        Self {
            id: new_record_id(),
            group_name: String::new(),
            project_name: String::new(),
            folder_name: String::new(),
            project_type: String::new(),
            server_start_time: None,
            start_time: zero_datetime_string(),
            server_end_time: None,
            end_time: zero_datetime_string(),
            is_successful: false,
        }
    }
}

impl ExecutionRecord {
    pub fn started(
        project: &ExecutionRecordProjectRef,
        clock: &ExecutionRecordClock,
        use_server_start_time: bool,
    ) -> Self {
        let server_start = if use_server_start_time {
            clock.now_server
        } else {
            clock.now_local_with_offset()
        };
        Self {
            id: new_record_id(),
            group_name: project.group_name.clone(),
            project_name: project.project_name.clone(),
            folder_name: project.folder_name.clone(),
            project_type: project.project_type.clone(),
            server_start_time: Some(format_offset_datetime(server_start)),
            start_time: format_offset_datetime(clock.now_local_with_offset()),
            ..Self::default()
        }
    }

    pub fn finish(&mut self, is_successful: bool, clock: &ExecutionRecordClock) {
        self.is_successful = is_successful;
        self.server_end_time = Some(format_offset_datetime(clock.now_server));
        self.end_time = format_offset_datetime(clock.now_local_with_offset());
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DailyExecutionRecord {
    pub name: String,
    #[serde(rename = "execution_records")]
    pub execution_records: Vec<ExecutionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecordStorage {
    storage_directory: PathBuf,
}

impl ExecutionRecordStorage {
    pub fn new(storage_directory: impl Into<PathBuf>) -> Self {
        Self {
            storage_directory: storage_directory.into(),
        }
    }

    pub fn from_app_root(app_root: impl AsRef<Path>) -> Self {
        Self::new(app_root.as_ref().join("log").join("ExecutionRecords"))
    }

    pub fn storage_directory(&self) -> &Path {
        &self.storage_directory
    }

    pub fn save_execution_record(&self, record: &ExecutionRecord) -> ExecutionRecordResult<()> {
        let date_key = record_start_date_key(record)?;
        fs::create_dir_all(&self.storage_directory).map_err(|source| {
            ExecutionRecordStorageError::Io {
                path: self.storage_directory.clone(),
                source,
            }
        })?;

        let path = self.storage_directory.join(format!("{date_key}.json"));
        let mut daily = if path.exists() {
            self.read_daily_file(&path)?
        } else {
            DailyExecutionRecord {
                name: date_key,
                execution_records: Vec::new(),
            }
        };

        if let Some(existing) = daily
            .execution_records
            .iter_mut()
            .find(|existing| existing.id == record.id)
        {
            *existing = record.clone();
        } else {
            daily.execution_records.push(record.clone());
        }

        let json = serde_json::to_string_pretty(&daily).map_err(|source| {
            ExecutionRecordStorageError::Json {
                path: path.clone(),
                source,
            }
        })?;
        fs::write(&path, json).map_err(|source| ExecutionRecordStorageError::Io { path, source })
    }

    pub fn recent_execution_records(
        &self,
        days: i64,
    ) -> ExecutionRecordResult<Vec<DailyExecutionRecord>> {
        self.recent_execution_records_for_today(days, Local::now().date_naive())
    }

    pub fn recent_execution_records_for_today(
        &self,
        days: i64,
        today: NaiveDate,
    ) -> ExecutionRecordResult<Vec<DailyExecutionRecord>> {
        if days <= 0 {
            return Err(ExecutionRecordStorageError::InvalidDays);
        }
        if !self.storage_directory.exists() {
            return Ok(Vec::new());
        }

        let start = today - Duration::days(days - 1);
        let mut records = Vec::new();
        for offset in 0..days {
            let date = start + Duration::days(offset);
            let path = self.storage_directory.join(format_date_key(date));
            let path = path.with_extension("json");
            if path.exists() {
                records.push(self.read_daily_file(&path)?);
            }
        }

        records.reverse();
        for daily in &mut records {
            daily.execution_records.reverse();
        }
        Ok(records)
    }

    pub fn recent_execution_records_by_config(
        &self,
        config: &TaskCompletionSkipRuleConfig,
    ) -> ExecutionRecordResult<Vec<DailyExecutionRecord>> {
        self.recent_execution_records(config.recent_day_count())
    }

    pub fn recent_execution_records_by_config_for_today(
        &self,
        config: &TaskCompletionSkipRuleConfig,
        today: NaiveDate,
    ) -> ExecutionRecordResult<Vec<DailyExecutionRecord>> {
        self.recent_execution_records_for_today(config.recent_day_count(), today)
    }

    fn read_daily_file(&self, path: &Path) -> ExecutionRecordResult<DailyExecutionRecord> {
        let content =
            fs::read_to_string(path).map_err(|source| ExecutionRecordStorageError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        serde_json::from_str(&content).map_err(|source| ExecutionRecordStorageError::Json {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskCompletionSkipRuleConfig {
    #[serde(alias = "Enable")]
    pub enable: bool,
    #[serde(alias = "SkipPolicy", alias = "skipPolicy")]
    pub skip_policy: String,
    #[serde(alias = "BoundaryTime", alias = "boundaryTime")]
    pub boundary_time: i32,
    #[serde(
        alias = "IsBoundaryTimeBasedOnServerTime",
        alias = "isBoundaryTimeBasedOnServerTime"
    )]
    pub is_boundary_time_based_on_server_time: bool,
    #[serde(alias = "LastRunGapSeconds", alias = "lastRunGapSeconds")]
    pub last_run_gap_seconds: i64,
    #[serde(alias = "ReferencePoint", alias = "referencePoint")]
    pub reference_point: String,
}

impl Default for TaskCompletionSkipRuleConfig {
    fn default() -> Self {
        Self {
            enable: false,
            skip_policy: "GroupPhysicalPathSkipPolicy".to_string(),
            boundary_time: 4,
            is_boundary_time_based_on_server_time: false,
            last_run_gap_seconds: -1,
            reference_point: "EndTime".to_string(),
        }
    }
}

impl TaskCompletionSkipRuleConfig {
    pub fn from_pathing_config(value: &Value) -> Option<Self> {
        let config = value
            .get("taskCompletionSkipRuleConfig")
            .or_else(|| value.get("TaskCompletionSkipRuleConfig"))
            .unwrap_or(value);
        serde_json::from_value(config.clone()).ok()
    }

    pub fn boundary_time_enabled(&self) -> bool {
        (0..=23).contains(&self.boundary_time)
    }

    pub fn is_effective(&self) -> bool {
        self.enable && (self.boundary_time_enabled() || self.last_run_gap_seconds >= 0)
    }

    pub fn recent_day_count(&self) -> i64 {
        if self.last_run_gap_seconds >= 0 {
            convert_seconds_to_days_up(self.last_run_gap_seconds)
        } else if self.boundary_time_enabled() {
            2
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecordProjectRef {
    pub group_name: String,
    pub project_name: String,
    pub folder_name: String,
    pub project_type: String,
}

impl ExecutionRecordProjectRef {
    pub fn new(
        group_name: impl Into<String>,
        project_name: impl Into<String>,
        folder_name: impl Into<String>,
        project_type: impl Into<String>,
    ) -> Self {
        Self {
            group_name: group_name.into(),
            project_name: project_name.into(),
            folder_name: folder_name.into(),
            project_type: project_type.into(),
        }
    }

    pub fn from_group_project(group: &ScriptGroup, project: &ScriptGroupProject) -> Self {
        Self::new(
            group.name.clone(),
            project.name.clone(),
            project.folder_name.clone(),
            script_project_type_name(&project.project_type),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecordClock {
    pub now_local: NaiveDateTime,
    pub local_offset: FixedOffset,
    pub now_server: DateTime<FixedOffset>,
}

impl ExecutionRecordClock {
    pub fn now() -> Self {
        let local_now = Local::now();
        let local_offset = local_now.offset().fix();
        Self {
            now_local: local_now.naive_local(),
            local_offset,
            now_server: local_now.with_timezone(&local_offset),
        }
    }

    pub fn fixed(
        now_local: NaiveDateTime,
        local_offset: FixedOffset,
        now_server: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            now_local,
            local_offset,
            now_server,
        }
    }

    pub fn now_local_with_offset(&self) -> DateTime<FixedOffset> {
        self.local_offset
            .from_local_datetime(&self.now_local)
            .single()
            .expect("fixed offset local datetime is always unique")
    }

    pub fn server_offset(&self) -> FixedOffset {
        *self.now_server.offset()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionRecordSkipDecision {
    pub should_skip: bool,
    pub message: String,
}

impl ExecutionRecordSkipDecision {
    pub fn skip(message: impl Into<String>) -> Self {
        Self {
            should_skip: true,
            message: message.into(),
        }
    }

    pub fn keep() -> Self {
        Self {
            should_skip: false,
            message: String::new(),
        }
    }
}

pub fn is_skip_task(
    project: &ExecutionRecordProjectRef,
    config: Option<&TaskCompletionSkipRuleConfig>,
    daily_records: &[DailyExecutionRecord],
    clock: &ExecutionRecordClock,
) -> ExecutionRecordResult<ExecutionRecordSkipDecision> {
    let Some(config) = config else {
        return Ok(ExecutionRecordSkipDecision::keep());
    };
    if !config.is_effective() {
        return Ok(ExecutionRecordSkipDecision::keep());
    }

    let boundary_time_enabled = config.boundary_time_enabled();
    for daily in daily_records {
        for record in &daily.execution_records {
            if !record.is_successful
                || record.project_type != project.project_type
                || record.project_name != project.project_name
            {
                continue;
            }

            let calc_time = record_reference_time(record, config)?;
            if config.last_run_gap_seconds >= 0 {
                let seconds_since_last_run = (clock.now_local - calc_time).num_seconds();
                if seconds_since_last_run > config.last_run_gap_seconds {
                    continue;
                }
            }

            if boundary_time_enabled {
                let target_date = record_boundary_time(record, clock)?;
                if !is_today_by_boundary(
                    config.boundary_time,
                    target_date,
                    config.is_boundary_time_based_on_server_time,
                    clock,
                )? {
                    continue;
                }
            }

            let match_reason = match config.skip_policy.as_str() {
                "GroupPhysicalPathSkipPolicy"
                    if project.group_name == record.group_name
                        && project.folder_name == record.folder_name =>
                {
                    "组和物理路径匹配一致"
                }
                "PhysicalPathSkipPolicy" if project.folder_name == record.folder_name => {
                    "物理路径相同"
                }
                "SameNameSkipPolicy" => "名称相同",
                "GroupPhysicalPathSkipPolicy" | "PhysicalPathSkipPolicy" => continue,
                _ => continue,
            };

            let mut message = format!("检查出满足跳过条件: {match_reason}");
            if config.last_run_gap_seconds >= 0 {
                let next_execution_time =
                    calc_time + Duration::seconds(config.last_run_gap_seconds);
                message.push_str(&format!(
                    ", 需在 {} 之后才能开始执行",
                    format_message_datetime(next_execution_time)
                ));
            } else if boundary_time_enabled {
                message.push_str(&format!(
                    ", 需在下一日 {} 点后才能开始执行",
                    config.boundary_time
                ));
            }
            message.push_str(&format!(", 匹配记录 GUID={}", record.id));
            return Ok(ExecutionRecordSkipDecision::skip(message));
        }
    }

    Ok(ExecutionRecordSkipDecision::keep())
}

pub fn is_today_by_boundary(
    boundary_hour: i32,
    target_date: DateTime<FixedOffset>,
    is_boundary_time_based_on_server_time: bool,
    clock: &ExecutionRecordClock,
) -> ExecutionRecordResult<bool> {
    if !(0..=23).contains(&boundary_hour) {
        return Err(ExecutionRecordStorageError::InvalidBoundaryHour);
    }

    let now = if is_boundary_time_based_on_server_time {
        clock.now_server
    } else {
        clock.now_local_with_offset()
    };
    let now_local = now.naive_local();
    let start_date = if now_local.hour() as i32 >= boundary_hour {
        now_local.date()
    } else {
        now_local.date() - Duration::days(1)
    };
    let today_start = start_date
        .and_hms_opt(boundary_hour as u32, 0, 0)
        .expect("validated boundary hour");
    let today_end = today_start + Duration::days(1);
    let target = target_date.with_timezone(now.offset()).naive_local();
    Ok(target >= today_start && target < today_end)
}

pub fn convert_seconds_to_days_up(seconds: i64) -> i64 {
    if seconds <= 0 {
        0
    } else {
        (seconds + 86_399) / 86_400
    }
}

fn record_start_date_key(record: &ExecutionRecord) -> ExecutionRecordResult<String> {
    let start_time = parse_record_local_datetime(&record.start_time, "start_time")?;
    Ok(format_date_key(start_time.date()))
}

fn record_reference_time(
    record: &ExecutionRecord,
    config: &TaskCompletionSkipRuleConfig,
) -> ExecutionRecordResult<NaiveDateTime> {
    if config.reference_point == "StartTime" {
        parse_record_local_datetime(&record.start_time, "start_time")
    } else {
        parse_record_local_datetime(&record.end_time, "end_time")
    }
}

fn record_boundary_time(
    record: &ExecutionRecord,
    clock: &ExecutionRecordClock,
) -> ExecutionRecordResult<DateTime<FixedOffset>> {
    if let Some(server_start_time) = &record.server_start_time {
        parse_record_offset_datetime(
            server_start_time,
            "server_start_time",
            clock.server_offset(),
        )
    } else {
        let start_time = parse_record_local_datetime(&record.start_time, "start_time")?;
        let local_start = clock
            .local_offset
            .from_local_datetime(&start_time)
            .single()
            .expect("fixed offset local datetime is always unique");
        Ok(local_start.with_timezone(&clock.server_offset()))
    }
}

fn parse_record_local_datetime(
    value: &str,
    field: &'static str,
) -> ExecutionRecordResult<NaiveDateTime> {
    parse_local_datetime(value).ok_or_else(|| ExecutionRecordStorageError::InvalidTime {
        field,
        value: value.to_string(),
    })
}

fn parse_record_offset_datetime(
    value: &str,
    field: &'static str,
    fallback_offset: FixedOffset,
) -> ExecutionRecordResult<DateTime<FixedOffset>> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return Ok(value);
    }
    let naive = parse_record_local_datetime(value, field)?;
    Ok(fallback_offset
        .from_local_datetime(&naive)
        .single()
        .expect("fixed offset local datetime is always unique"))
}

fn parse_local_datetime(value: &str) -> Option<NaiveDateTime> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return Some(value.naive_local());
    }
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S%.f",
    ] {
        if let Ok(value) = NaiveDateTime::parse_from_str(value, format) {
            return Some(value);
        }
    }
    None
}

fn format_date_key(date: NaiveDate) -> String {
    format!("{:04}{:02}{:02}", date.year(), date.month(), date.day())
}

fn format_offset_datetime(value: DateTime<FixedOffset>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn format_message_datetime(value: NaiveDateTime) -> String {
    format!(
        "{}-{}-{} {}:{:02}:{:02}",
        value.year(),
        value.month(),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
}

fn zero_datetime_string() -> String {
    "0001-01-01T00:00:00".to_string()
}

fn new_record_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let process = std::process::id() as u128;
    let mixed = now ^ (process << 64);
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (mixed >> 96) as u32,
        (mixed >> 80) as u16,
        (mixed >> 64) as u16,
        (mixed >> 48) as u16,
        mixed & 0x0000_ffff_ffff_ffff_ffff
    )
}

fn script_project_type_name(project_type: &ScriptProjectType) -> &'static str {
    match project_type {
        ScriptProjectType::Javascript => "Javascript",
        ScriptProjectType::KeyMouse => "KeyMouse",
        ScriptProjectType::Pathing => "Pathing",
        ScriptProjectType::Shell => "Shell",
    }
}

#[cfg(test)]
#[path = "execution_records_tests.rs"]
mod tests;
