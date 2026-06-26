use crate::ExecutionRecordClock;
use chrono::{Duration, NaiveDate, TimeZone, Timelike};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PathingPartySkipConfig {
    #[serde(alias = "Enabled")]
    pub enabled: bool,
    #[serde(alias = "SkipDuring", alias = "skipDuring")]
    pub skip_during: String,
    #[serde(alias = "TaskCycleConfig", alias = "taskCycleConfig")]
    pub task_cycle_config: PathingPartyTaskCycleConfig,
}

impl Default for PathingPartySkipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            skip_during: String::new(),
            task_cycle_config: PathingPartyTaskCycleConfig::default(),
        }
    }
}

impl PathingPartySkipConfig {
    pub fn from_pathing_config(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PathingPartyTaskCycleConfig {
    #[serde(alias = "Enable")]
    pub enable: bool,
    #[serde(alias = "BoundaryTime", alias = "boundaryTime")]
    pub boundary_time: i32,
    #[serde(
        alias = "IsBoundaryTimeBasedOnServerTime",
        alias = "isBoundaryTimeBasedOnServerTime"
    )]
    pub is_boundary_time_based_on_server_time: bool,
    #[serde(alias = "Cycle")]
    pub cycle: i32,
    #[serde(alias = "Index")]
    pub index: i32,
}

impl Default for PathingPartyTaskCycleConfig {
    fn default() -> Self {
        Self {
            enable: false,
            boundary_time: 0,
            is_boundary_time_based_on_server_time: false,
            cycle: 1,
            index: 1,
        }
    }
}

impl PathingPartyTaskCycleConfig {
    pub fn execution_order(&self, clock: &ExecutionRecordClock) -> i32 {
        if self.cycle <= 0 || self.boundary_time < 0 || self.boundary_time > 24 {
            return -1;
        }

        let now = if self.is_boundary_time_based_on_server_time {
            clock.now_server
        } else {
            clock.now_local_with_offset()
        };
        let now_local = now.naive_local();
        let Some(boundary_naive) = now_local
            .date()
            .and_hms_opt(self.boundary_time as u32, 0, 0)
        else {
            return -1;
        };
        let Some(boundary_time_today) = now.offset().from_local_datetime(&boundary_naive).single()
        else {
            return -1;
        };
        let corrected = if now < boundary_time_today {
            now - Duration::days(1)
        } else {
            now
        };
        let Some(base_date) = NaiveDate::from_ymd_opt(1970, 1, 1) else {
            return -1;
        };
        let total_days = (corrected.date_naive() - base_date).num_days() as i32;
        (total_days % self.cycle) + 1
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathingPreRunSkipDecision {
    pub should_skip: bool,
    pub message: String,
}

impl PathingPreRunSkipDecision {
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

pub fn pathing_pre_run_skip_decision(
    project_name: &str,
    pathing_config: &Value,
    clock: &ExecutionRecordClock,
) -> PathingPreRunSkipDecision {
    let config = PathingPartySkipConfig::from_pathing_config(pathing_config);
    if !config.enabled {
        return PathingPreRunSkipDecision::keep();
    }

    if is_current_hour_equal(&config.skip_during, clock.now_local.hour()) {
        return PathingPreRunSkipDecision::skip(format!(
            "{project_name}任务已到禁止执行时段，将跳过！"
        ));
    }

    if config.task_cycle_config.enable {
        let index = config.task_cycle_config.execution_order(clock);
        if index == -1 {
            return PathingPreRunSkipDecision::keep();
        }
        if index != config.task_cycle_config.index {
            return PathingPreRunSkipDecision::skip(format!(
                "{project_name}任务已经不在执行周期（当前值${index}!=配置值${}），将跳过此任务！",
                config.task_cycle_config.index
            ));
        }
    }

    PathingPreRunSkipDecision::keep()
}

pub fn is_current_hour_equal(input: &str, current_hour: u32) -> bool {
    input
        .trim()
        .parse::<i32>()
        .is_ok_and(|hour| (0..=23).contains(&hour) && hour as u32 == current_hour)
}

#[cfg(test)]
#[path = "pathing_skip_tests.rs"]
mod tests;
