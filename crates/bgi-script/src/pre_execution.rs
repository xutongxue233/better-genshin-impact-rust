use crate::{PathingPartySkipConfig, ScriptGroup, ScriptGroupProject};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PreExecutionPriorityConfig {
    #[serde(alias = "Enabled")]
    pub enabled: bool,
    #[serde(alias = "GroupNames", alias = "groupNames")]
    pub group_names: String,
    #[serde(alias = "MaxRetryCount", alias = "maxRetryCount")]
    pub max_retry_count: i32,
}

impl Default for PreExecutionPriorityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            group_names: String::new(),
            max_retry_count: 1,
        }
    }
}

impl PreExecutionPriorityConfig {
    pub fn from_pathing_config(value: &Value) -> Self {
        let config = value
            .get("preExecutionPriorityConfig")
            .or_else(|| value.get("PreExecutionPriorityConfig"))
            .unwrap_or(value);
        serde_json::from_value(config.clone()).unwrap_or_default()
    }

    pub fn target_group_names(&self) -> Vec<String> {
        parse_pre_execution_group_names(&self.group_names)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreExecutionPriorityPlan {
    pub active: bool,
    pub target_group_names: Vec<String>,
    pub candidates: Vec<PreExecutionPriorityCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreExecutionPriorityCandidate {
    pub group_name: String,
    pub project_index: usize,
    pub project: ScriptGroupProject,
    pub project_key: String,
    pub previous_count: i32,
    pub max_retry_count: i32,
}

pub fn plan_pre_execution_priority_projects(
    current_group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    execution_counts: &HashMap<String, i32>,
    mut should_skip_project: impl FnMut(&ScriptGroup, &ScriptGroupProject) -> bool,
) -> PreExecutionPriorityPlan {
    let result = try_plan_pre_execution_priority_projects(
        current_group,
        all_groups,
        execution_counts,
        |group, project| Ok::<bool, std::convert::Infallible>(should_skip_project(group, project)),
    );
    match result {
        Ok(plan) => plan,
        Err(never) => match never {},
    }
}

pub fn try_plan_pre_execution_priority_projects<E>(
    current_group: &ScriptGroup,
    all_groups: &[ScriptGroup],
    execution_counts: &HashMap<String, i32>,
    mut should_skip_project: impl FnMut(&ScriptGroup, &ScriptGroupProject) -> Result<bool, E>,
) -> Result<PreExecutionPriorityPlan, E> {
    let pathing_config =
        PathingPartySkipConfig::from_pathing_config(&current_group.config.pathing_config);
    let config =
        PreExecutionPriorityConfig::from_pathing_config(&current_group.config.pathing_config);
    if !pathing_config.enabled || !config.enabled {
        return Ok(PreExecutionPriorityPlan::default());
    }

    let target_group_names = config.target_group_names();
    if target_group_names.is_empty() {
        return Ok(PreExecutionPriorityPlan::default());
    }

    let mut candidates = Vec::new();
    for group in all_groups {
        if !target_group_names
            .iter()
            .any(|target| target.eq_ignore_ascii_case(&group.name))
        {
            continue;
        }

        for (project_index, project) in group.projects.iter().enumerate() {
            if should_skip_project(group, project)? {
                continue;
            }

            let project_key = pre_execution_project_key(&group.name, project);
            let previous_count = execution_counts.get(&project_key).copied().unwrap_or(0);
            if previous_count > config.max_retry_count {
                continue;
            }

            candidates.push(PreExecutionPriorityCandidate {
                group_name: group.name.clone(),
                project_index,
                project: project.clone(),
                project_key,
                previous_count,
                max_retry_count: config.max_retry_count,
            });
        }
    }

    Ok(PreExecutionPriorityPlan {
        active: true,
        target_group_names,
        candidates,
    })
}

pub fn parse_pre_execution_group_names(group_names: &str) -> Vec<String> {
    group_names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn pre_execution_project_key(group_name: &str, project: &ScriptGroupProject) -> String {
    format!("{}|{}|{}", project.name, project.folder_name, group_name)
}

#[cfg(test)]
#[path = "pre_execution_tests.rs"]
mod tests;
