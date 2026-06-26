use bgi_core::{is_new_version, read_pathing_task, FarmingPlanConfig, PathingFarmingExecutionPlan};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FarmingPlanError {
    #[error("farming plan I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("farming plan JSON failed at {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("pathing task load failed: {0}")]
    PathingTask(#[from] bgi_core::BgiError),
}

pub type FarmingPlanResult<T> = std::result::Result<T, FarmingPlanError>;

#[derive(Debug, Clone)]
pub struct FarmingPlanExecutionContext {
    pub config: FarmingPlanConfig,
    pub log_directory: PathBuf,
    pub app_version: Option<String>,
}

impl FarmingPlanExecutionContext {
    pub fn from_app_root(app_root: impl AsRef<Path>, config: FarmingPlanConfig) -> Self {
        Self {
            config,
            log_directory: app_root.as_ref().join("log").join("FarmingPlan"),
            app_version: None,
        }
    }

    pub fn with_app_version(mut self, app_version: impl Into<String>) -> Self {
        self.app_version = Some(app_version.into());
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FarmingPlanSkipDecision {
    pub should_skip: bool,
    pub message: String,
}

impl FarmingPlanSkipDecision {
    pub fn keep() -> Self {
        Self {
            should_skip: false,
            message: String::new(),
        }
    }

    pub fn skip(message: impl Into<String>) -> Self {
        Self {
            should_skip: true,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FarmingPlanDailyTotals {
    pub total_elite_mob_count: f64,
    pub total_normal_mob_count: f64,
    pub daily_elite_cap: u64,
    pub daily_mob_cap: u64,
    pub uses_miyoushe_stats: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FarmingRouteRef {
    pub group_name: String,
    pub project_name: String,
    pub folder_name: String,
}

impl FarmingRouteRef {
    pub fn new(
        group_name: impl Into<String>,
        project_name: impl Into<String>,
        folder_name: impl Into<String>,
    ) -> Self {
        Self {
            group_name: group_name.into(),
            project_name: project_name.into(),
            folder_name: folder_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FarmingPlanRecordOutcome {
    pub path: PathBuf,
    pub totals: FarmingPlanDailyTotals,
    pub record: FarmingRecord,
}

pub fn farming_plan_skip_decision_from_pathing_file(
    pathing_root: impl AsRef<Path>,
    folder_name: &str,
    project_name: &str,
    context: Option<&FarmingPlanExecutionContext>,
    now_server: DateTime<FixedOffset>,
) -> FarmingPlanResult<FarmingPlanSkipDecision> {
    let Some(context) = context.filter(|context| context.config.enabled) else {
        return Ok(FarmingPlanSkipDecision::keep());
    };

    let path = pathing_root.as_ref().join(folder_name).join(project_name);
    let task = read_pathing_task(&path)?;
    if let (Some(required), Some(app_version)) = (&task.info.bgi_version, &context.app_version) {
        if !required.trim().is_empty() && is_new_version(app_version, required) {
            return Ok(FarmingPlanSkipDecision::skip(String::new()));
        }
    }

    let farming = task.execution_plan().farming;
    let daily = read_daily_farming_data(&context.log_directory, now_server)?;
    Ok(farming_plan_skip_decision(
        &farming,
        &daily,
        &context.config,
    ))
}

pub fn record_farming_session(
    context: &FarmingPlanExecutionContext,
    route: &FarmingRouteRef,
    farming: &PathingFarmingExecutionPlan,
    now_local: DateTime<FixedOffset>,
    now_server: DateTime<FixedOffset>,
) -> FarmingPlanResult<FarmingPlanRecordOutcome> {
    let path = daily_farming_data_path(&context.log_directory, now_server);
    let mut daily = read_daily_farming_data(&context.log_directory, now_server)?;
    if farming.allow_farming_count {
        daily.total_normal_mob_count += farming.normal_mob_count;
        daily.total_elite_mob_count += farming.elite_mob_count;
    }

    let record = FarmingRecord {
        group_name: route.group_name.clone(),
        project_name: route.project_name.clone(),
        folder_name: route.folder_name.clone(),
        normal_mob_count: if farming.allow_farming_count {
            farming.normal_mob_count
        } else {
            0.0
        },
        elite_mob_count: if farming.allow_farming_count {
            farming.elite_mob_count
        } else {
            0.0
        },
        timestamp: Some(format_farming_datetime(now_local)),
    };
    daily.records.push(record.clone());
    save_daily_farming_data(&path, &daily)?;
    let totals = daily.final_totals(&context.config);

    Ok(FarmingPlanRecordOutcome {
        path,
        totals,
        record,
    })
}

pub fn farming_plan_skip_decision(
    farming: &PathingFarmingExecutionPlan,
    daily: &DailyFarmingData,
    config: &FarmingPlanConfig,
) -> FarmingPlanSkipDecision {
    if !farming.allow_farming_count || farming.primary_target == "disable" {
        return FarmingPlanSkipDecision::keep();
    }

    let totals = daily.final_totals(config);
    let is_elite_over_limit = totals.total_elite_mob_count >= totals.daily_elite_cap as f64;
    let is_normal_over_limit = totals.total_normal_mob_count >= totals.daily_mob_cap as f64;

    let mut messages = Vec::new();
    if is_elite_over_limit {
        messages.push(format!(
            "精英超上限:{}/{}",
            totals.total_elite_mob_count, totals.daily_elite_cap
        ));
    }
    if is_normal_over_limit {
        messages.push(format!(
            "小怪超上限:{}/{}",
            totals.total_normal_mob_count, totals.daily_mob_cap
        ));
    }

    if is_elite_over_limit && is_normal_over_limit {
        return FarmingPlanSkipDecision::skip(messages.join(","));
    }

    if farming.normal_mob_count == 0.0 && farming.elite_mob_count == 0.0 {
        messages.push("精英和小怪计数都为0，请确认配置".to_string());
        return FarmingPlanSkipDecision::skip(messages.join(","));
    }

    if (farming.elite_mob_count == 0.0 && farming.primary_target == "elite")
        || (farming.normal_mob_count == 0.0 && farming.primary_target == "normal")
    {
        messages.push("主目标计数为0，请确认配置".to_string());
        return FarmingPlanSkipDecision::skip(messages.join(","));
    }

    let mut result = false;
    if farming.primary_target == "elite" && is_elite_over_limit {
        result = true;
        if farming.normal_mob_count > 0.0 {
            messages.push("脚本主目标为精英".to_string());
        }
    } else if farming.primary_target == "normal" && is_normal_over_limit {
        result = true;
        if farming.elite_mob_count > 0.0 {
            messages.push("脚本主目标为小怪".to_string());
        }
    }

    if !result {
        result = (is_elite_over_limit && farming.normal_mob_count == 0.0)
            || (is_normal_over_limit && farming.elite_mob_count == 0.0);
    }

    if result {
        FarmingPlanSkipDecision::skip(messages.join(","))
    } else {
        FarmingPlanSkipDecision::keep()
    }
}

pub fn read_daily_farming_data(
    log_directory: impl AsRef<Path>,
    now_server: DateTime<FixedOffset>,
) -> FarmingPlanResult<DailyFarmingData> {
    let log_directory = log_directory.as_ref();
    fs::create_dir_all(log_directory).map_err(|source| FarmingPlanError::Io {
        path: log_directory.to_path_buf(),
        source,
    })?;

    let path = daily_farming_data_path(log_directory, now_server);
    if !path.exists() {
        return Ok(DailyFarmingData::default());
    }

    let content = fs::read_to_string(&path).map_err(|source| FarmingPlanError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

pub fn save_daily_farming_data(
    path: impl AsRef<Path>,
    data: &DailyFarmingData,
) -> FarmingPlanResult<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| FarmingPlanError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|source| FarmingPlanError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, json).map_err(|source| FarmingPlanError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn daily_farming_data_path(
    log_directory: impl AsRef<Path>,
    now_server: DateTime<FixedOffset>,
) -> PathBuf {
    log_directory
        .as_ref()
        .join(format!("{}.json", daily_farming_date_key(now_server)))
}

pub fn daily_farming_date_key(now_server: DateTime<FixedOffset>) -> String {
    let date = if now_server.hour() < 4 {
        now_server.date_naive() - Duration::days(1)
    } else {
        now_server.date_naive()
    };
    date.format("%Y%m%d").to_string()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DailyFarmingData {
    pub total_normal_mob_count: f64,
    pub total_elite_mob_count: f64,
    pub records: Vec<FarmingRecord>,
    pub miyoushe_total_normal_mob_count: f64,
    pub miyoushe_total_elite_mob_count: f64,
    pub last_miyoushe_update_time: Option<String>,
    pub travels_diary_detail_manager_update_time: Option<String>,
}

impl DailyFarmingData {
    pub fn enable_miyoushe_stats(&self) -> bool {
        self.miyoushe_total_elite_mob_count + self.miyoushe_total_normal_mob_count > 0.0
    }

    pub fn final_totals(&self, config: &FarmingPlanConfig) -> FarmingPlanDailyTotals {
        if self.enable_miyoushe_stats() {
            let update_time = self.travels_diary_update_time();
            let records = self.records.iter().filter(|record| {
                record
                    .timestamp()
                    .is_some_and(|timestamp| timestamp > update_time)
            });
            let (sum_elite, sum_normal) = records.fold((0.0, 0.0), |acc, record| {
                (
                    acc.0 + record.elite_mob_count,
                    acc.1 + record.normal_mob_count,
                )
            });
            return FarmingPlanDailyTotals {
                total_elite_mob_count: self.miyoushe_total_elite_mob_count + sum_elite,
                total_normal_mob_count: self.miyoushe_total_normal_mob_count + sum_normal,
                daily_elite_cap: config.miyoushe_data_config.daily_elite_cap,
                daily_mob_cap: config.miyoushe_data_config.daily_mob_cap,
                uses_miyoushe_stats: true,
            };
        }

        FarmingPlanDailyTotals {
            total_elite_mob_count: self.total_elite_mob_count,
            total_normal_mob_count: self.total_normal_mob_count,
            daily_elite_cap: config.daily_elite_cap,
            daily_mob_cap: config.daily_mob_cap,
            uses_miyoushe_stats: false,
        }
    }

    fn travels_diary_update_time(&self) -> NaiveDateTime {
        self.travels_diary_detail_manager_update_time
            .as_deref()
            .and_then(parse_farming_datetime)
            .unwrap_or_else(min_datetime)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FarmingRecord {
    pub group_name: String,
    pub project_name: String,
    pub folder_name: String,
    pub normal_mob_count: f64,
    pub elite_mob_count: f64,
    pub timestamp: Option<String>,
}

impl FarmingRecord {
    fn timestamp(&self) -> Option<NaiveDateTime> {
        self.timestamp.as_deref().and_then(parse_farming_datetime)
    }
}

fn parse_farming_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .ok()
        .or_else(|| {
            DateTime::parse_from_rfc3339(value)
                .ok()
                .map(|datetime| datetime.naive_local())
        })
        .or_else(|| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)
        })
}

fn format_farming_datetime(value: DateTime<FixedOffset>) -> String {
    value.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

fn min_datetime() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(1, 1, 1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .expect("minimum chrono datetime is valid")
}

#[cfg(test)]
#[path = "farming_plan_tests.rs"]
mod tests;
