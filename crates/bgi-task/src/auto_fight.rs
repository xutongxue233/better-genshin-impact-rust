use crate::task_params::{combat_strategy_path, AutoFightParam};
use crate::{Result, TaskError};
use bgi_core::{GenshinAction, KeyBindingsConfig};
use bgi_input::{InputEvent, KeyActionType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[path = "auto_fight_avatar_catalog.rs"]
mod avatar_catalog;

#[path = "auto_fight_input.rs"]
mod input;
#[path = "auto_fight_input_events.rs"]
mod input_events;

#[path = "auto_fight_loop.rs"]
mod fight_loop;
#[path = "auto_fight_finish_detection.rs"]
mod finish_detection;

#[path = "auto_fight_model.rs"]
mod model;

#[path = "auto_fight_playback.rs"]
mod playback;

#[path = "auto_fight_script.rs"]
mod script;

#[path = "auto_fight_vision.rs"]
mod vision;

pub use avatar_catalog::*;
pub use fight_loop::*;
pub use finish_detection::{
    execute_auto_fight_finish_detection_live_probe, execute_auto_fight_finish_detection_probe,
    finish_detection_events_after_detection, finish_detection_events_before_capture,
    plan_auto_fight_finish_detection,
};
pub use model::*;
pub use playback::*;
pub use script::*;
pub use vision::*;

use self::script::normalize_user_auto_fight_strategy_path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFightExecutionConfig {
    pub param: AutoFightParam,
}

impl AutoFightExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        let Some(value) = value else {
            return config;
        };

        if let Some(strategy_name) = string_config_alias(
            value,
            &[
                "strategyName",
                "strategy_name",
                "strategy",
                "strategyNameOrPath",
            ],
        ) {
            config.param.combat_strategy_path = combat_strategy_path(Some(strategy_name));
        }
        if let Some(strategy_path) =
            string_config_alias(value, &["combatStrategyPath", "combat_strategy_path"])
        {
            config.param.combat_strategy_path = strategy_path.to_string();
        }
        if let Some(team_names) = string_config_alias(value, &["teamNames", "team_names"]) {
            config.param.team_names = team_names.to_string();
        }

        config
    }
}

fn string_config_alias<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoFightExecutionPlan {
    pub param: AutoFightParam,
    pub combat_scripts: CombatScriptBagPlan,
    pub script_execution_plans: Vec<CombatScriptExecutionPlan>,
    pub playback_evaluation: CombatScriptPlaybackBatchEvaluation,
    pub team_selection: CombatScriptTeamSelectionPlan,
    pub team_plan: Option<CombatTeamPlan>,
    pub team_playback: Option<CombatTeamPlaybackExecution>,
    pub fight_loop_plan: CombatFightLoopPlan,
    pub finish_detection_plan: AutoFightFinishDetectionPlan,
    pub action_scheduler_plans: Vec<CombatScriptActionSchedulerPlan>,
    pub dispatched: bool,
    pub completed: bool,
    pub notes: String,
}

pub fn plan_auto_fight(
    working_directory: impl AsRef<Path>,
    param: AutoFightParam,
) -> Result<AutoFightExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let normalized_strategy_path =
        normalize_user_auto_fight_strategy_path(&param.combat_strategy_path)?;
    let source_path = working_directory.join(&normalized_strategy_path);
    if !source_path.exists() {
        return Err(TaskError::CombatStrategy(format!(
            "combat strategy file does not exist: {}",
            source_path.display()
        )));
    }
    let catalog = read_combat_avatar_catalog(working_directory)?;
    let combat_scripts = read_combat_script_bag_with_catalog(
        working_directory,
        &param.combat_strategy_path,
        Some(&catalog),
    )?;
    let script_execution_plans = plan_combat_script_bag_execution(&combat_scripts)?;
    let playback_evaluation = evaluate_combat_script_batch_playback(&script_execution_plans);
    let configured_team_avatar_names =
        standardize_configured_team_avatar_names(&catalog, &param.team_names)?;
    let team_selection =
        plan_combat_script_team_selection(&combat_scripts, &configured_team_avatar_names);
    let team_plan = if team_selection.team_avatar_names.is_empty() {
        None
    } else {
        Some(plan_combat_team(
            &catalog,
            &team_selection.team_avatar_names,
            &team_selection.command_avatar_names,
            &param.action_scheduler_by_cd,
        )?)
    };
    let selected_script_execution = team_selection.script_name.as_ref().and_then(|script_name| {
        script_execution_plans
            .iter()
            .find(|script| &script.name == script_name)
    });
    let team_playback = selected_script_execution
        .zip(team_plan.as_ref())
        .map(|(script, team)| {
            plan_team_context_combat_script_playback(
                script,
                team,
                &team_selection.executable_commands,
            )
        })
        .transpose()?;
    let fight_loop_plan = plan_combat_fight_loop(
        &param,
        &team_selection,
        team_plan.as_ref(),
        selected_script_execution,
    )?;
    let finish_detection_plan = plan_auto_fight_finish_detection(
        &param.finish_detect_config,
        AUTO_FIGHT_DEFAULT_FINISH_DELAY_MS,
        AUTO_FIGHT_DEFAULT_FINISH_DETECT_DELAY_MS,
    )?;
    let action_scheduler_plans =
        plan_combat_script_bag_action_scheduler(&combat_scripts, &param.action_scheduler_by_cd);
    Ok(AutoFightExecutionPlan {
        param,
        combat_scripts,
        script_execution_plans,
        playback_evaluation,
        team_selection,
        team_plan,
        team_playback,
        fight_loop_plan,
        finish_detection_plan,
        action_scheduler_plans,
        dispatched: false,
        completed: false,
        notes: "Combat strategy parsing, configured-team alias normalization, team-context script selection planning, command execution planning, static playback evaluation, known-team input playback, action-scheduler CD planning, fight-loop decision planning, and finish-detection input/pixel planning are migrated; native combat scene recognition and full fight-loop command dispatch remain pending.".to_string(),
    })
}
