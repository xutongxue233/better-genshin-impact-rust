use super::input_events::combat_virtual_key_plan;
use super::{
    read_combat_avatar_catalog, CombatAvatarCatalog, CombatCommandMethod, CombatCommandPlan,
    CombatScriptBagPlan, CombatScriptMatchPlan, CombatScriptParseFailure, CombatScriptPlan,
    CombatScriptTeamSelectionPlan, CombatScriptTeamSelectionStatus, CombatTeamAvatarPlan,
    CombatTeamPlan, CURRENT_COMBAT_AVATAR_NAME, EXPECTED_COMBAT_TEAM_AVATAR_COUNT,
};
use crate::{Result, TaskError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[path = "auto_fight_script_parser.rs"]
mod parser;

use parser::{
    collect_txt_files, combat_script_logical_lines, parse_combat_script_file,
    parse_combat_script_lines,
};

pub(super) fn parse_f64_arg(value: &str, label: &str) -> Result<f64> {
    parser::parse_f64_arg(value, label)
}

pub(super) fn parse_i32_arg(value: &str, label: &str) -> Result<i32> {
    parser::parse_i32_arg(value, label)
}

pub(super) fn normalize_user_auto_fight_strategy_path(strategy_path: &str) -> Result<PathBuf> {
    parser::normalize_user_auto_fight_strategy_path(strategy_path)
}

pub fn read_combat_script_bag(
    working_directory: impl AsRef<Path>,
    strategy_path: &str,
) -> Result<CombatScriptBagPlan> {
    let working_directory = working_directory.as_ref();
    let normalized_path = normalize_user_auto_fight_strategy_path(strategy_path)?;
    let source_path = working_directory.join(&normalized_path);
    if source_path.exists() {
        let catalog = read_combat_avatar_catalog(working_directory)?;
        return read_combat_script_bag_with_catalog(
            working_directory,
            strategy_path,
            Some(&catalog),
        );
    }
    read_combat_script_bag_with_catalog(working_directory, strategy_path, None)
}

pub fn read_combat_script_bag_with_catalog(
    working_directory: impl AsRef<Path>,
    strategy_path: &str,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptBagPlan> {
    let normalized_path = normalize_user_auto_fight_strategy_path(strategy_path)?;
    let working_directory = working_directory.as_ref();
    let source_path = working_directory.join(&normalized_path);
    if source_path.is_file() {
        let script = parse_combat_script_file(&source_path, catalog)?;
        return Ok(CombatScriptBagPlan {
            source_path: normalized_path,
            scripts: vec![script],
            parse_failures: Vec::new(),
        });
    }
    if source_path.is_dir() {
        let mut files = Vec::new();
        collect_txt_files(&source_path, &mut files)
            .map_err(|error| TaskError::CombatStrategy(error.to_string()))?;
        files.sort();
        if files.is_empty() {
            return Err(TaskError::CombatStrategy(format!(
                "combat strategy file does not exist: {}",
                source_path.display()
            )));
        }
        let mut scripts = Vec::new();
        let mut parse_failures = Vec::new();
        for file in files {
            match parse_combat_script_file(&file, catalog) {
                Ok(script) => scripts.push(script),
                Err(error) => parse_failures.push(CombatScriptParseFailure {
                    path: file,
                    message: error.to_string(),
                }),
            }
        }
        if scripts.is_empty() {
            return Err(TaskError::CombatStrategy(
                "all combat strategy files failed to parse".to_string(),
            ));
        }
        return Ok(CombatScriptBagPlan {
            source_path: normalized_path,
            scripts,
            parse_failures,
        });
    }
    Err(TaskError::CombatStrategy(format!(
        "combat strategy file does not exist: {}",
        source_path.display()
    )))
}

pub fn parse_combat_script_context(context: &str, validate: bool) -> Result<CombatScriptPlan> {
    parse_combat_script_context_with_catalog(context, validate, None)
}

pub fn parse_combat_script_context_with_catalog(
    context: &str,
    validate: bool,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptPlan> {
    let lines = combat_script_logical_lines(context);
    parse_combat_script_lines(lines, validate, catalog).map(|mut script| {
        script.name = String::new();
        script.path = None;
        script
    })
}

pub fn match_combat_script(
    bag: &CombatScriptBagPlan,
    team_avatar_names: &[String],
) -> Result<CombatScriptMatchPlan> {
    if bag.scripts.is_empty() {
        return Err(TaskError::CombatStrategy(
            "combat script bag has no parsed scripts".to_string(),
        ));
    }
    let Some((script, matched_avatar_count, full_match)) =
        select_combat_script_for_team(bag, team_avatar_names)
    else {
        return Err(TaskError::CombatStrategy(
            "no combat script matched the current team".to_string(),
        ));
    };
    Ok(CombatScriptMatchPlan {
        script_name: script.name.clone(),
        script_path: script.path.clone(),
        matched_avatar_count,
        full_match,
        commands: script.commands.clone(),
    })
}

pub fn parse_configured_team_avatar_names(team_names: &str) -> Vec<String> {
    team_names
        .split([',', '，'])
        .filter_map(|name| {
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

pub fn standardize_configured_team_avatar_names(
    catalog: &CombatAvatarCatalog,
    team_names: &str,
) -> Result<Vec<String>> {
    if team_names.trim().is_empty() {
        return Ok(Vec::new());
    }
    let names: Vec<_> = team_names
        .split([',', '，'])
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect();
    if names.len() != EXPECTED_COMBAT_TEAM_AVATAR_COUNT {
        return Err(TaskError::CombatStrategy(format!(
            "强制指定队伍角色数量不正确，必须是4个，当前{}个",
            names.len()
        )));
    }
    names
        .iter()
        .map(|name| catalog.standard_name_for_alias(name))
        .collect()
}

pub fn standardize_configured_team_avatar_names_from_assets(
    working_directory: impl AsRef<Path>,
    team_names: &str,
) -> Result<Vec<String>> {
    if team_names.trim().is_empty() {
        return Ok(Vec::new());
    }
    let catalog = read_combat_avatar_catalog(working_directory)?;
    standardize_configured_team_avatar_names(&catalog, team_names)
}

pub fn plan_combat_team(
    catalog: &CombatAvatarCatalog,
    team_avatar_names: &[String],
    command_avatar_names: &[String],
    action_scheduler_by_cd: &str,
) -> Result<CombatTeamPlan> {
    let avatars: Vec<_> = team_avatar_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let metadata = catalog.avatar_by_name(name).ok_or_else(|| {
                TaskError::CombatStrategy(format!("combat avatar metadata is missing: {name}"))
            })?;
            let manual_skill_cd_seconds =
                parse_action_scheduler_cd_for_avatar(name, action_scheduler_by_cd).unwrap_or(-1.0);
            let action_scheduler_configured =
                parse_action_scheduler_cd_for_avatar(name, action_scheduler_by_cd).is_some();
            Ok(CombatTeamAvatarPlan {
                index: index + 1,
                name: name.clone(),
                id: metadata.id.clone(),
                name_en: metadata.name_en.clone(),
                weapon: metadata.weapon.clone(),
                skill_cd_seconds: metadata.skill_cd,
                skill_hold_cd_seconds: metadata.skill_hold_cd,
                burst_cd_seconds: metadata.burst_cd,
                manual_skill_cd_seconds,
                action_scheduler_configured,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let can_be_skipped_avatar_names: Vec<_> = avatars
        .iter()
        .filter(|avatar| {
            avatar.action_scheduler_configured
                && command_avatar_names
                    .iter()
                    .any(|command_avatar| command_avatar == &avatar.name)
        })
        .map(|avatar| avatar.name.clone())
        .collect();
    let all_command_avatars_can_be_skipped = !command_avatar_names.is_empty()
        && command_avatar_names.iter().all(|avatar| {
            can_be_skipped_avatar_names
                .iter()
                .any(|skipped| skipped == avatar)
        });
    Ok(CombatTeamPlan {
        avatars,
        command_avatar_names: command_avatar_names.to_vec(),
        can_be_skipped_avatar_names,
        all_command_avatars_can_be_skipped,
    })
}

pub fn plan_combat_script_team_selection(
    bag: &CombatScriptBagPlan,
    team_avatar_names: &[String],
) -> CombatScriptTeamSelectionPlan {
    if team_avatar_names.is_empty() {
        return CombatScriptTeamSelectionPlan {
            status: CombatScriptTeamSelectionStatus::NoTeamContext,
            team_avatar_names: Vec::new(),
            script_name: None,
            script_path: None,
            matched_avatar_count: 0,
            full_match: false,
            command_avatar_names: Vec::new(),
            executable_avatar_names: Vec::new(),
            filtered_out_avatar_names: Vec::new(),
            executable_commands: Vec::new(),
            message:
                "team context is not available; native avatar recognition is required for script selection"
                    .to_string(),
        };
    }
    let Some((script, matched_avatar_count, full_match)) =
        select_combat_script_for_team(bag, team_avatar_names)
    else {
        return CombatScriptTeamSelectionPlan {
            status: CombatScriptTeamSelectionStatus::NoMatch,
            team_avatar_names: team_avatar_names.to_vec(),
            script_name: None,
            script_path: None,
            matched_avatar_count: 0,
            full_match: false,
            command_avatar_names: Vec::new(),
            executable_avatar_names: Vec::new(),
            filtered_out_avatar_names: Vec::new(),
            executable_commands: Vec::new(),
            message: "no combat script matched the current team".to_string(),
        };
    };
    let command_avatar_names = combat_script_command_avatar_names(script);
    let executable_avatar_names: Vec<_> = command_avatar_names
        .iter()
        .filter(|name| team_avatar_names.iter().any(|team| team == *name))
        .cloned()
        .collect();
    let filtered_out_avatar_names: Vec<_> = command_avatar_names
        .iter()
        .filter(|name| {
            !executable_avatar_names
                .iter()
                .any(|executable| executable == *name)
        })
        .cloned()
        .collect();
    let executable_commands: Vec<_> = script
        .commands
        .iter()
        .filter(|command| {
            executable_avatar_names
                .iter()
                .any(|name| name == &command.avatar)
        })
        .cloned()
        .collect();
    let status = if full_match {
        CombatScriptTeamSelectionStatus::FullMatch
    } else {
        CombatScriptTeamSelectionStatus::PartialFallback
    };
    CombatScriptTeamSelectionPlan {
        status,
        team_avatar_names: team_avatar_names.to_vec(),
        script_name: Some(script.name.clone()),
        script_path: script.path.clone(),
        matched_avatar_count,
        full_match,
        command_avatar_names,
        executable_avatar_names,
        filtered_out_avatar_names,
        executable_commands,
        message: if full_match {
            format!("matched combat script: {}", script.name)
        } else {
            format!(
                "no full team match; using highest-match combat script: {}",
                script.name
            )
        },
    }
}

fn select_combat_script_for_team<'a>(
    bag: &'a CombatScriptBagPlan,
    team_avatar_names: &[String],
) -> Option<(&'a CombatScriptPlan, usize, bool)> {
    let mut best: Option<(&CombatScriptPlan, usize)> = None;
    for script in &bag.scripts {
        let matched = team_avatar_names
            .iter()
            .filter(|name| script.avatar_names.iter().any(|avatar| avatar == *name))
            .count();
        if matched == team_avatar_names.len() {
            return Some((script, matched, true));
        }
        if best
            .map(|(_, best_count)| matched > best_count)
            .unwrap_or(true)
        {
            best = Some((script, matched));
        }
    }
    best.and_then(|(script, matched)| (matched > 0).then_some((script, matched, false)))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatActionSchedulerEntry {
    pub avatar: String,
    pub manual_skill_cd_seconds: f64,
    pub has_explicit_cd: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatActionSchedulerPlan {
    pub config: String,
    pub entries: Vec<CombatActionSchedulerEntry>,
    pub configured_avatar_names: Vec<String>,
    pub skipped_avatar_names: Vec<String>,
    pub all_command_avatars_can_be_skipped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatScriptActionSchedulerPlan {
    pub script_name: String,
    pub script_path: Option<PathBuf>,
    pub command_avatar_names: Vec<String>,
    pub scheduler: CombatActionSchedulerPlan,
}

pub fn parse_action_scheduler_cd_for_avatar(avatar_name: &str, input: &str) -> Option<f64> {
    if avatar_name.is_empty() || input.is_empty() {
        return None;
    }
    let mut search_end = input.len();
    while let Some(found_index) = input[..search_end].rfind(avatar_name) {
        let start_valid = found_index == 0 || input[..found_index].ends_with(';');
        let value_start = found_index + avatar_name.len();
        let end_valid = value_start == input.len()
            || input[value_start..].starts_with(',')
            || input[value_start..].starts_with(';');
        if start_valid && end_valid {
            if value_start >= input.len() || !input[value_start..].starts_with(',') {
                return Some(-1.0);
            }
            let value_end = input[value_start + 1..]
                .find(';')
                .map(|index| value_start + 1 + index)
                .unwrap_or(input.len());
            return input[value_start + 1..value_end]
                .trim()
                .parse::<f64>()
                .ok()
                .or(Some(-1.0));
        }
        if found_index == 0 {
            break;
        }
        search_end = found_index;
    }
    None
}

pub fn plan_combat_action_scheduler_by_cd(
    config: &str,
    team_avatar_names: &[String],
    command_avatar_names: &[String],
) -> CombatActionSchedulerPlan {
    let entries: Vec<_> = team_avatar_names
        .iter()
        .filter_map(|avatar| {
            parse_action_scheduler_cd_for_avatar(avatar, config).map(|manual_skill_cd_seconds| {
                CombatActionSchedulerEntry {
                    avatar: avatar.clone(),
                    manual_skill_cd_seconds,
                    has_explicit_cd: manual_skill_cd_seconds >= 0.0,
                }
            })
        })
        .collect();
    let configured_avatar_names: Vec<_> =
        entries.iter().map(|entry| entry.avatar.clone()).collect();
    let skipped_avatar_names: Vec<_> = command_avatar_names
        .iter()
        .filter(|avatar| configured_avatar_names.iter().any(|name| name == *avatar))
        .cloned()
        .collect();
    let all_command_avatars_can_be_skipped = !command_avatar_names.is_empty()
        && command_avatar_names
            .iter()
            .all(|avatar| skipped_avatar_names.iter().any(|name| name == avatar));
    CombatActionSchedulerPlan {
        config: config.to_string(),
        entries,
        configured_avatar_names,
        skipped_avatar_names,
        all_command_avatars_can_be_skipped,
    }
}

pub fn plan_combat_script_action_scheduler(
    script: &CombatScriptPlan,
    action_scheduler_by_cd: &str,
) -> CombatScriptActionSchedulerPlan {
    let command_avatar_names = combat_script_command_avatar_names(script);
    let scheduler = plan_combat_action_scheduler_by_cd(
        action_scheduler_by_cd,
        &script.avatar_names,
        &command_avatar_names,
    );
    CombatScriptActionSchedulerPlan {
        script_name: script.name.clone(),
        script_path: script.path.clone(),
        command_avatar_names,
        scheduler,
    }
}

pub fn plan_combat_script_bag_action_scheduler(
    bag: &CombatScriptBagPlan,
    action_scheduler_by_cd: &str,
) -> Vec<CombatScriptActionSchedulerPlan> {
    bag.scripts
        .iter()
        .map(|script| plan_combat_script_action_scheduler(script, action_scheduler_by_cd))
        .collect()
}

fn combat_script_command_avatar_names(script: &CombatScriptPlan) -> Vec<String> {
    let mut names = Vec::new();
    for command in &script.commands {
        if command.avatar == CURRENT_COMBAT_AVATAR_NAME {
            continue;
        }
        if !names.contains(&command.avatar) {
            names.push(command.avatar.clone());
        }
    }
    names
}
