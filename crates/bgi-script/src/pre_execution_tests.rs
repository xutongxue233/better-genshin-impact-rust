use super::*;
use crate::{
    ScriptGroup, ScriptGroupConfig, ScriptGroupProject, ScriptProjectStatus, ScriptProjectType,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn pre_execution_config_parses_legacy_shape_and_group_names() {
    let config = PreExecutionPriorityConfig::from_pathing_config(&json!({
        "preExecutionPriorityConfig": {
            "enabled": true,
            "groupNames": " A, ,b , C ",
            "maxRetryCount": 0
        }
    }));

    assert!(config.enabled);
    assert_eq!(config.max_retry_count, 0);
    assert_eq!(config.target_group_names(), vec!["A", "b", "C"]);
    assert_eq!(parse_pre_execution_group_names(""), Vec::<String>::new());
}

#[test]
fn pre_execution_plan_requires_pathing_and_priority_enabled() {
    let all_groups = vec![group("Target", vec![project("One", "one")])];
    let disabled_pathing = group_with_pathing_config(
        "Current",
        json!({
            "enabled": false,
            "preExecutionPriorityConfig": {
                "enabled": true,
                "groupNames": "Target"
            }
        }),
        vec![],
    );
    let disabled_priority = group_with_pathing_config(
        "Current",
        json!({
            "enabled": true,
            "preExecutionPriorityConfig": {
                "enabled": false,
                "groupNames": "Target"
            }
        }),
        vec![],
    );
    let empty_names = group_with_pathing_config(
        "Current",
        json!({
            "enabled": true,
            "preExecutionPriorityConfig": {
                "enabled": true,
                "groupNames": " , "
            }
        }),
        vec![],
    );

    assert!(
        !plan_pre_execution_priority_projects(
            &disabled_pathing,
            &all_groups,
            &HashMap::new(),
            |_, _| false,
        )
        .active
    );
    assert!(
        !plan_pre_execution_priority_projects(
            &disabled_priority,
            &all_groups,
            &HashMap::new(),
            |_, _| false,
        )
        .active
    );
    assert!(
        !plan_pre_execution_priority_projects(
            &empty_names,
            &all_groups,
            &HashMap::new(),
            |_, _| false,
        )
        .active
    );
}

#[test]
fn pre_execution_plan_matches_groups_case_insensitively_in_ui_order() {
    let current = current_group("third, FIRST");
    let all_groups = vec![
        group("First", vec![project("One", "one")]),
        group("Second", vec![project("Two", "two")]),
        group("Third", vec![project("Three", "three")]),
    ];

    let plan =
        plan_pre_execution_priority_projects(&current, &all_groups, &HashMap::new(), |_, _| false);

    assert!(plan.active);
    assert_eq!(plan.target_group_names, vec!["third", "FIRST"]);
    assert_eq!(
        plan.candidates
            .iter()
            .map(|candidate| candidate.group_name.as_str())
            .collect::<Vec<_>>(),
        vec!["First", "Third"]
    );
    assert_eq!(
        plan.candidates
            .iter()
            .map(|candidate| candidate.project.name.as_str())
            .collect::<Vec<_>>(),
        vec!["One", "Three"]
    );
}

#[test]
fn pre_execution_plan_applies_record_skip_callback_and_retry_count_legacy_rule() {
    let current = current_group("Target");
    let disabled_project = ScriptGroupProject {
        status: ScriptProjectStatus::Disabled,
        ..project("Disabled", "disabled")
    };
    let all_groups = vec![group(
        "Target",
        vec![
            project("Allowed", "allowed"),
            project("SkippedByRecord", "skipped"),
            project("AtLimit", "limit"),
            project("PastLimit", "past"),
            disabled_project,
        ],
    )];
    let mut counts = HashMap::new();
    counts.insert("AtLimit|limit|Target".to_string(), 1);
    counts.insert("PastLimit|past|Target".to_string(), 2);

    let plan =
        plan_pre_execution_priority_projects(&current, &all_groups, &counts, |_, project| {
            project.name == "SkippedByRecord"
        });

    assert_eq!(
        plan.candidates
            .iter()
            .map(|candidate| (
                candidate.project.name.as_str(),
                candidate.previous_count,
                candidate.project_key.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Allowed", 0, "Allowed|allowed|Target"),
            ("AtLimit", 1, "AtLimit|limit|Target"),
            ("Disabled", 0, "Disabled|disabled|Target")
        ]
    );
}

#[test]
fn pre_execution_plan_respects_zero_and_negative_retry_count_edges() {
    let current = group_with_pathing_config(
        "Current",
        json!({
            "enabled": true,
            "preExecutionPriorityConfig": {
                "enabled": true,
                "groupNames": "Target",
                "maxRetryCount": 0
            }
        }),
        vec![],
    );
    let all_groups = vec![group("Target", vec![project("Once", "once")])];
    let mut counts = HashMap::new();
    counts.insert("Once|once|Target".to_string(), 0);
    assert_eq!(
        plan_pre_execution_priority_projects(&current, &all_groups, &counts, |_, _| false)
            .candidates
            .len(),
        1
    );
    counts.insert("Once|once|Target".to_string(), 1);
    assert_eq!(
        plan_pre_execution_priority_projects(&current, &all_groups, &counts, |_, _| false)
            .candidates
            .len(),
        0
    );

    let negative = group_with_pathing_config(
        "Current",
        json!({
            "enabled": true,
            "preExecutionPriorityConfig": {
                "enabled": true,
                "groupNames": "Target",
                "maxRetryCount": -1
            }
        }),
        vec![],
    );
    counts.insert("Once|once|Target".to_string(), 0);
    assert_eq!(
        plan_pre_execution_priority_projects(&negative, &all_groups, &counts, |_, _| false)
            .candidates
            .len(),
        0
    );
}

fn current_group(group_names: &str) -> ScriptGroup {
    group_with_pathing_config(
        "Current",
        json!({
            "enabled": true,
            "preExecutionPriorityConfig": {
                "enabled": true,
                "groupNames": group_names,
                "maxRetryCount": 1
            }
        }),
        vec![],
    )
}

fn group(name: &str, projects: Vec<ScriptGroupProject>) -> ScriptGroup {
    group_with_pathing_config(name, json!({}), projects)
}

fn group_with_pathing_config(
    name: &str,
    pathing_config: serde_json::Value,
    projects: Vec<ScriptGroupProject>,
) -> ScriptGroup {
    ScriptGroup {
        name: name.to_string(),
        config: ScriptGroupConfig {
            pathing_config,
            ..ScriptGroupConfig::default()
        },
        projects,
        ..ScriptGroup::default()
    }
}

fn project(name: &str, folder_name: &str) -> ScriptGroupProject {
    ScriptGroupProject {
        name: name.to_string(),
        folder_name: folder_name.to_string(),
        project_type: ScriptProjectType::Pathing,
        ..ScriptGroupProject::default()
    }
}
