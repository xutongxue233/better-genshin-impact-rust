use super::{
    evaluate_task_invocation_plan, TaskInvocationExecutionMode, TaskInvocationExecutionResult,
    TaskInvocationKind, TaskInvocationPlan,
};
use crate::auto_artifact_salvage::{
    plan_auto_artifact_salvage, AutoArtifactSalvageExecutionConfig,
    AutoArtifactSalvageExecutionPlan, AUTO_ARTIFACT_SALVAGE_TASK_KEY,
};
use crate::auto_boss::{
    plan_auto_boss, AutoBossExecutionConfig, AutoBossExecutionPlan, AUTO_BOSS_TASK_KEY,
};
use crate::auto_domain::{
    plan_auto_domain, AutoDomainExecutionConfig, AutoDomainExecutionPlan, AUTO_DOMAIN_TASK_KEY,
};
use crate::auto_eat::{
    plan_auto_eat_food, AutoEatFoodExecutionConfig, AutoEatFoodExecutionPlan,
    AUTO_EAT_FOOD_TASK_KEY,
};
use crate::auto_fight::{plan_auto_fight, AutoFightExecutionConfig, AutoFightExecutionPlan};
use crate::auto_genius_invokation::{
    plan_auto_genius_invokation, AutoGeniusInvokationExecutionConfig,
    AutoGeniusInvokationExecutionPlan, AUTO_GENIUS_INVOKATION_TASK_KEY,
};
use crate::auto_ley_line_outcrop::{
    plan_auto_ley_line_outcrop, AutoLeyLineOutcropExecutionConfig, AutoLeyLineOutcropExecutionPlan,
    AUTO_LEY_LINE_OUTCROP_TASK_KEY,
};
use crate::auto_music_game::{
    plan_auto_music_game, AutoMusicGameExecutionConfig, AutoMusicGameExecutionPlan,
    AUTO_MUSIC_GAME_TASK_KEY,
};
use crate::auto_open_chest::{
    plan_auto_open_chest, AutoOpenChestExecutionConfig, AutoOpenChestExecutionPlan,
    AUTO_OPEN_CHEST_TASK_KEY,
};
use crate::auto_pathing::{
    plan_auto_pathing, AutoPathingExecutionConfig, AutoPathingExecutionPlan,
};
use crate::auto_stygian_onslaught::{
    plan_auto_stygian_onslaught, AutoStygianOnslaughtExecutionConfig,
    AutoStygianOnslaughtExecutionPlan, AUTO_STYGIAN_ONSLAUGHT_TASK_KEY,
};
use crate::auto_track::{
    plan_auto_track, AutoTrackExecutionConfig, AutoTrackExecutionPlan, AUTO_TRACK_TASK_KEY,
};
use crate::auto_track_path::{
    plan_auto_track_path, AutoTrackPathExecutionConfig, AutoTrackPathExecutionPlan,
    AUTO_TRACK_PATH_TASK_KEY,
};
use crate::auto_wood::{
    plan_auto_wood, AutoWoodExecutionConfig, AutoWoodExecutionPlan, AUTO_WOOD_TASK_KEY,
};
use crate::catalog::{find_task_catalog_entry, TaskPortState, TaskRustExecutionSurface};
use crate::get_grid_icons::{
    plan_get_grid_icons, GetGridIconsExecutionConfig, GetGridIconsExecutionPlan,
    GET_GRID_ICONS_TASK_KEY,
};
use crate::macro_hotkeys::{
    plan_quick_enhance_artifact_macro, plan_turn_around_macro, MacroHotkeyExecutionConfig,
    MacroHotkeyExecutionPlan, QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY, TURN_AROUND_MACRO_TASK_KEY,
};
use crate::quick_buy::{
    plan_quick_buy, QuickBuyExecutionConfig, QuickBuyExecutionPlan, QUICK_BUY_TASK_KEY,
};
use crate::quick_serenitea_pot::{
    plan_quick_serenitea_pot, QuickSereniteaPotExecutionConfig, QuickSereniteaPotExecutionPlan,
    QUICK_SERENITEA_POT_TASK_KEY,
};
use crate::redeem_code::{
    plan_use_redeem_codes, RedeemCodeEntry, UseRedeemCodeExecutionConfig,
    UseRedeemCodeExecutionPlan, USE_REDEEM_CODE_TASK_KEY,
};
use crate::shell::{
    execute_shell_task_with_cancel, ShellConfig, ShellExecutionResult, ShellTaskParam,
};
use crate::task_params::AutoFightParam;
use crate::{Result, TaskError};
use bgi_vision::Size;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndependentTaskKind {
    UseRedeemCode,
    AutoGeniusInvokation,
    AutoWood,
    AutoFight,
    AutoDomain,
    AutoTrack,
    AutoTrackPath,
    AutoMusicGame,
    AutoOpenChest,
    AutoEatFood,
    AutoStygianOnslaught,
    AutoPathing,
    AutoBoss,
    AutoArtifactSalvage,
    AutoLeyLineOutcrop,
    GetGridIcons,
    TurnAroundMacro,
    QuickEnhanceArtifactMacro,
    QuickBuy,
    QuickSereniteaPot,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndependentTaskDescriptor {
    pub kind: IndependentTaskKind,
    pub key: &'static str,
    pub display_name: &'static str,
    pub requires_main_ui_wait: bool,
    pub config_section: Option<&'static str>,
    pub hotkey_fields: &'static [&'static str],
    pub asset_roots: &'static [&'static str],
    pub ported: bool,
    pub rust_execution_surface: TaskRustExecutionSurface,
    pub notes: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndependentTaskExecutionRequest {
    pub task_key: String,
    pub command: Option<String>,
    pub config: Option<Value>,
    pub working_directory: PathBuf,
}

impl IndependentTaskExecutionRequest {
    pub fn shell(
        command: impl Into<String>,
        config: ShellConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: "Shell".to_string(),
            command: Some(command.into()),
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn use_redeem_code(
        codes: Vec<RedeemCodeEntry>,
        capture_size: Size,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: USE_REDEEM_CODE_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(UseRedeemCodeExecutionConfig {
                codes,
                capture_size,
            })
            .ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_pathing(route: impl Into<String>, working_directory: impl Into<PathBuf>) -> Self {
        Self {
            task_key: "AutoPathing".to_string(),
            command: None,
            config: serde_json::to_value(AutoPathingExecutionConfig {
                route: route.into(),
            })
            .ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_fight(strategy_name: Option<&str>, working_directory: impl Into<PathBuf>) -> Self {
        let param = AutoFightParam::new(strategy_name);
        Self {
            task_key: "AutoFight".to_string(),
            command: None,
            config: serde_json::to_value(AutoFightExecutionConfig { param }).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_wood(
        wood_round_num: u64,
        wood_daily_max_count: u64,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_WOOD_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({
                "woodRoundNum": wood_round_num,
                "woodDailyMaxCount": wood_daily_max_count
            })),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_domain(
        domain_round_num: i32,
        strategy_name: Option<&str>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_DOMAIN_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({
                "domainRoundNum": domain_round_num,
                "strategyName": strategy_name
            })),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_genius_invokation(
        strategy_name: impl Into<String>,
        strategy: impl Into<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({
                "strategyName": strategy_name.into(),
                "strategy": strategy.into()
            })),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_track_path(
        path_file: impl Into<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_TRACK_PATH_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({
                "pathFile": path_file.into()
            })),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_boss(
        boss_name: impl Into<String>,
        strategy_name: impl Into<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_BOSS_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({
                "bossName": boss_name.into(),
                "strategyName": strategy_name.into()
            })),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_artifact_salvage(
        config: AutoArtifactSalvageExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_ARTIFACT_SALVAGE_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_music_game(
        config: AutoMusicGameExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_MUSIC_GAME_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_stygian_onslaught(
        config: AutoStygianOnslaughtExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_STYGIAN_ONSLAUGHT_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_track(working_directory: impl Into<PathBuf>) -> Self {
        Self {
            task_key: AUTO_TRACK_TASK_KEY.to_string(),
            command: None,
            config: Some(serde_json::json!({})),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_ley_line_outcrop(
        config: AutoLeyLineOutcropExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_LEY_LINE_OUTCROP_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn get_grid_icons(
        config: GetGridIconsExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: GET_GRID_ICONS_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn quick_buy(
        config: QuickBuyExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: QUICK_BUY_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn quick_serenitea_pot(
        config: QuickSereniteaPotExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: QUICK_SERENITEA_POT_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_open_chest(
        config: AutoOpenChestExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_OPEN_CHEST_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn auto_eat_food(
        config: AutoEatFoodExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn turn_around_macro(
        config: MacroHotkeyExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: TURN_AROUND_MACRO_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }

    pub fn quick_enhance_artifact_macro(
        config: MacroHotkeyExecutionConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task_key: QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY.to_string(),
            command: None,
            config: serde_json::to_value(config).ok(),
            working_directory: working_directory.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IndependentTaskExecutionPlan {
    UseRedeemCodePlan(UseRedeemCodeExecutionPlan),
    AutoPathingPlan(AutoPathingExecutionPlan),
    AutoFightPlan(AutoFightExecutionPlan),
    AutoWoodPlan(AutoWoodExecutionPlan),
    AutoDomainPlan(AutoDomainExecutionPlan),
    AutoGeniusInvokationPlan(AutoGeniusInvokationExecutionPlan),
    AutoTrackPathPlan(AutoTrackPathExecutionPlan),
    AutoTrackPlan(AutoTrackExecutionPlan),
    AutoBossPlan(AutoBossExecutionPlan),
    AutoArtifactSalvagePlan(AutoArtifactSalvageExecutionPlan),
    AutoMusicGamePlan(AutoMusicGameExecutionPlan),
    AutoOpenChestPlan(AutoOpenChestExecutionPlan),
    AutoEatFoodPlan(AutoEatFoodExecutionPlan),
    AutoStygianOnslaughtPlan(AutoStygianOnslaughtExecutionPlan),
    AutoLeyLineOutcropPlan(AutoLeyLineOutcropExecutionPlan),
    GetGridIconsPlan(GetGridIconsExecutionPlan),
    TurnAroundMacroPlan(MacroHotkeyExecutionPlan),
    QuickEnhanceArtifactMacroPlan(MacroHotkeyExecutionPlan),
    QuickBuyPlan(QuickBuyExecutionPlan),
    QuickSereniteaPotPlan(QuickSereniteaPotExecutionPlan),
}

impl IndependentTaskExecutionPlan {
    pub fn task_key(&self) -> &str {
        match self {
            IndependentTaskExecutionPlan::UseRedeemCodePlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoPathingPlan(_) => "AutoPathing",
            IndependentTaskExecutionPlan::AutoFightPlan(_) => "AutoFight",
            IndependentTaskExecutionPlan::AutoWoodPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoDomainPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoGeniusInvokationPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoTrackPathPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoTrackPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoBossPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoArtifactSalvagePlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoMusicGamePlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoOpenChestPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoEatFoodPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoStygianOnslaughtPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::AutoLeyLineOutcropPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::GetGridIconsPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::TurnAroundMacroPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::QuickEnhanceArtifactMacroPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::QuickBuyPlan(plan) => &plan.task_key,
            IndependentTaskExecutionPlan::QuickSereniteaPotPlan(plan) => &plan.task_key,
        }
    }

    pub fn executor_ready(&self) -> bool {
        match self {
            IndependentTaskExecutionPlan::UseRedeemCodePlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoPathingPlan(_) => false,
            IndependentTaskExecutionPlan::AutoFightPlan(_) => false,
            IndependentTaskExecutionPlan::AutoWoodPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoDomainPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoGeniusInvokationPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoTrackPathPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoTrackPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoBossPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoArtifactSalvagePlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoMusicGamePlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoOpenChestPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoEatFoodPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoStygianOnslaughtPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::AutoLeyLineOutcropPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::GetGridIconsPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::TurnAroundMacroPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::QuickEnhanceArtifactMacroPlan(plan) => {
                plan.executor_ready
            }
            IndependentTaskExecutionPlan::QuickBuyPlan(plan) => plan.executor_ready,
            IndependentTaskExecutionPlan::QuickSereniteaPotPlan(plan) => plan.executor_ready,
        }
    }
}

impl From<IndependentTaskExecutionPlan> for IndependentTaskExecution {
    fn from(plan: IndependentTaskExecutionPlan) -> Self {
        match plan {
            IndependentTaskExecutionPlan::UseRedeemCodePlan(plan) => Self::UseRedeemCodePlan(plan),
            IndependentTaskExecutionPlan::AutoPathingPlan(plan) => Self::AutoPathingPlan(plan),
            IndependentTaskExecutionPlan::AutoFightPlan(plan) => Self::AutoFightPlan(plan),
            IndependentTaskExecutionPlan::AutoWoodPlan(plan) => Self::AutoWoodPlan(plan),
            IndependentTaskExecutionPlan::AutoDomainPlan(plan) => Self::AutoDomainPlan(plan),
            IndependentTaskExecutionPlan::AutoGeniusInvokationPlan(plan) => {
                Self::AutoGeniusInvokationPlan(plan)
            }
            IndependentTaskExecutionPlan::AutoTrackPathPlan(plan) => Self::AutoTrackPathPlan(plan),
            IndependentTaskExecutionPlan::AutoTrackPlan(plan) => Self::AutoTrackPlan(plan),
            IndependentTaskExecutionPlan::AutoBossPlan(plan) => Self::AutoBossPlan(plan),
            IndependentTaskExecutionPlan::AutoArtifactSalvagePlan(plan) => {
                Self::AutoArtifactSalvagePlan(plan)
            }
            IndependentTaskExecutionPlan::AutoMusicGamePlan(plan) => Self::AutoMusicGamePlan(plan),
            IndependentTaskExecutionPlan::AutoOpenChestPlan(plan) => Self::AutoOpenChestPlan(plan),
            IndependentTaskExecutionPlan::AutoEatFoodPlan(plan) => Self::AutoEatFoodPlan(plan),
            IndependentTaskExecutionPlan::AutoStygianOnslaughtPlan(plan) => {
                Self::AutoStygianOnslaughtPlan(plan)
            }
            IndependentTaskExecutionPlan::AutoLeyLineOutcropPlan(plan) => {
                Self::AutoLeyLineOutcropPlan(plan)
            }
            IndependentTaskExecutionPlan::GetGridIconsPlan(plan) => Self::GetGridIconsPlan(plan),
            IndependentTaskExecutionPlan::TurnAroundMacroPlan(plan) => {
                Self::TurnAroundMacroPlan(plan)
            }
            IndependentTaskExecutionPlan::QuickEnhanceArtifactMacroPlan(plan) => {
                Self::QuickEnhanceArtifactMacroPlan(plan)
            }
            IndependentTaskExecutionPlan::QuickBuyPlan(plan) => Self::QuickBuyPlan(plan),
            IndependentTaskExecutionPlan::QuickSereniteaPotPlan(plan) => {
                Self::QuickSereniteaPotPlan(plan)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IndependentTaskExecution {
    Shell(ShellExecutionResult),
    UseRedeemCodePlan(UseRedeemCodeExecutionPlan),
    AutoPathingPlan(AutoPathingExecutionPlan),
    AutoFightPlan(AutoFightExecutionPlan),
    AutoWoodPlan(AutoWoodExecutionPlan),
    AutoDomainPlan(AutoDomainExecutionPlan),
    AutoGeniusInvokationPlan(AutoGeniusInvokationExecutionPlan),
    AutoTrackPathPlan(AutoTrackPathExecutionPlan),
    AutoTrackPlan(AutoTrackExecutionPlan),
    AutoBossPlan(AutoBossExecutionPlan),
    AutoArtifactSalvagePlan(AutoArtifactSalvageExecutionPlan),
    AutoMusicGamePlan(AutoMusicGameExecutionPlan),
    AutoOpenChestPlan(AutoOpenChestExecutionPlan),
    AutoEatFoodPlan(AutoEatFoodExecutionPlan),
    AutoStygianOnslaughtPlan(AutoStygianOnslaughtExecutionPlan),
    AutoLeyLineOutcropPlan(AutoLeyLineOutcropExecutionPlan),
    GetGridIconsPlan(GetGridIconsExecutionPlan),
    TurnAroundMacroPlan(MacroHotkeyExecutionPlan),
    QuickEnhanceArtifactMacroPlan(MacroHotkeyExecutionPlan),
    QuickBuyPlan(QuickBuyExecutionPlan),
    QuickSereniteaPotPlan(QuickSereniteaPotExecutionPlan),
    NativePending(TaskInvocationExecutionResult),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndependentTaskExecutionResult {
    pub task_key: String,
    pub execution: IndependentTaskExecution,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndependentTaskExecutionPlanResult {
    pub task_key: String,
    pub plan: IndependentTaskExecutionPlan,
}

impl From<IndependentTaskExecutionPlanResult> for IndependentTaskExecutionResult {
    fn from(result: IndependentTaskExecutionPlanResult) -> Self {
        Self {
            task_key: result.task_key,
            execution: result.plan.into(),
        }
    }
}

pub fn plan_independent_task_execution(
    request: &IndependentTaskExecutionRequest,
) -> Result<IndependentTaskExecutionResult> {
    if let Some(plan) = plan_independent_task_execution_plan(request)? {
        return Ok(plan.into());
    }

    let entry = find_task_catalog_entry(&request.task_key)
        .ok_or_else(|| TaskError::UnknownIndependentTask(request.task_key.clone()))?;
    Ok(IndependentTaskExecutionResult {
        task_key: entry.key.to_string(),
        execution: IndependentTaskExecution::NativePending(independent_task_native_pending_result(
            request, entry,
        )),
    })
}

pub fn execute_independent_task_with_cancel(
    request: &IndependentTaskExecutionRequest,
    is_cancelled: impl FnMut() -> bool,
) -> Result<IndependentTaskExecutionResult> {
    let entry = find_task_catalog_entry(&request.task_key)
        .ok_or_else(|| TaskError::UnknownIndependentTask(request.task_key.clone()))?;
    if entry.key == "Shell" {
        return Ok(IndependentTaskExecutionResult {
            task_key: entry.key.to_string(),
            execution: IndependentTaskExecution::Shell(execute_shell_task_with_cancel(
                &ShellTaskParam::build_from_config(
                    request.command.clone().unwrap_or_default(),
                    ShellConfig::from_value(request.config.as_ref()),
                    request.working_directory.clone(),
                ),
                is_cancelled,
            )?),
        });
    }

    if let Some(plan) = plan_independent_task_execution_plan(request)? {
        return Ok(plan.into());
    }

    Ok(IndependentTaskExecutionResult {
        task_key: entry.key.to_string(),
        execution: IndependentTaskExecution::NativePending(independent_task_native_pending_result(
            request, entry,
        )),
    })
}

pub fn plan_independent_task_execution_plan(
    request: &IndependentTaskExecutionRequest,
) -> Result<Option<IndependentTaskExecutionPlanResult>> {
    let entry = find_task_catalog_entry(&request.task_key)
        .ok_or_else(|| TaskError::UnknownIndependentTask(request.task_key.clone()))?;
    if entry.key == "Shell" {
        return Ok(None);
    }
    if entry.key == USE_REDEEM_CODE_TASK_KEY {
        let config = UseRedeemCodeExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::UseRedeemCodePlan(plan_use_redeem_codes(
                config.codes,
                config.capture_size,
            )?),
        }));
    }
    if entry.key == "AutoPathing" {
        let config = AutoPathingExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoPathingPlan(plan_auto_pathing(
                &request.working_directory,
                &config.route,
            )?),
        }));
    }
    if entry.key == "AutoFight" {
        let config = AutoFightExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoFightPlan(plan_auto_fight(
                &request.working_directory,
                config.param,
            )?),
        }));
    }
    if entry.key == AUTO_WOOD_TASK_KEY {
        let config = AutoWoodExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoWoodPlan(plan_auto_wood(config)),
        }));
    }
    if entry.key == AUTO_DOMAIN_TASK_KEY {
        let config = AutoDomainExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoDomainPlan(plan_auto_domain(config)?),
        }));
    }
    if entry.key == AUTO_GENIUS_INVOKATION_TASK_KEY {
        let config = AutoGeniusInvokationExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoGeniusInvokationPlan(
                plan_auto_genius_invokation(&request.working_directory, config)?,
            ),
        }));
    }
    if entry.key == AUTO_TRACK_PATH_TASK_KEY {
        let config = AutoTrackPathExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoTrackPathPlan(plan_auto_track_path(
                &request.working_directory,
                config,
            )?),
        }));
    }
    if entry.key == AUTO_TRACK_TASK_KEY {
        let config = AutoTrackExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoTrackPlan(plan_auto_track(config)),
        }));
    }
    if entry.key == AUTO_MUSIC_GAME_TASK_KEY {
        let config = AutoMusicGameExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoMusicGamePlan(plan_auto_music_game(config)),
        }));
    }
    if entry.key == AUTO_OPEN_CHEST_TASK_KEY {
        let config = AutoOpenChestExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoOpenChestPlan(plan_auto_open_chest(config)?),
        }));
    }
    if entry.key == AUTO_EAT_FOOD_TASK_KEY {
        let config = AutoEatFoodExecutionConfig::from_value(request.config.as_ref())?;
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoEatFoodPlan(plan_auto_eat_food(config)?),
        }));
    }
    if entry.key == AUTO_STYGIAN_ONSLAUGHT_TASK_KEY {
        let config = AutoStygianOnslaughtExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoStygianOnslaughtPlan(
                plan_auto_stygian_onslaught(config)?,
            ),
        }));
    }
    if entry.key == AUTO_BOSS_TASK_KEY {
        let config = AutoBossExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoBossPlan(plan_auto_boss(
                &request.working_directory,
                config,
            )?),
        }));
    }
    if entry.key == AUTO_ARTIFACT_SALVAGE_TASK_KEY {
        let config = AutoArtifactSalvageExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoArtifactSalvagePlan(
                plan_auto_artifact_salvage(config)?,
            ),
        }));
    }
    if entry.key == AUTO_LEY_LINE_OUTCROP_TASK_KEY {
        let config = AutoLeyLineOutcropExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::AutoLeyLineOutcropPlan(plan_auto_ley_line_outcrop(
                &request.working_directory,
                config,
            )?),
        }));
    }
    if entry.key == GET_GRID_ICONS_TASK_KEY {
        let config = GetGridIconsExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::GetGridIconsPlan(plan_get_grid_icons(config)?),
        }));
    }
    if entry.key == TURN_AROUND_MACRO_TASK_KEY {
        let config = MacroHotkeyExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::TurnAroundMacroPlan(plan_turn_around_macro(config)),
        }));
    }
    if entry.key == QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY {
        let config = MacroHotkeyExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::QuickEnhanceArtifactMacroPlan(
                plan_quick_enhance_artifact_macro(config),
            ),
        }));
    }
    if entry.key == QUICK_BUY_TASK_KEY {
        let config = QuickBuyExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::QuickBuyPlan(plan_quick_buy(config)?),
        }));
    }
    if entry.key == QUICK_SERENITEA_POT_TASK_KEY {
        let config = QuickSereniteaPotExecutionConfig::from_value(request.config.as_ref());
        return Ok(Some(IndependentTaskExecutionPlanResult {
            task_key: entry.key.to_string(),
            plan: IndependentTaskExecutionPlan::QuickSereniteaPotPlan(plan_quick_serenitea_pot(
                config,
            )?),
        }));
    }

    Ok(None)
}

fn independent_task_native_pending_result(
    request: &IndependentTaskExecutionRequest,
    entry: crate::TaskCatalogEntry,
) -> TaskInvocationExecutionResult {
    evaluate_task_invocation_plan(
        TaskInvocationPlan {
            kind: TaskInvocationKind::RunIndependentTask,
            task_key: Some(entry.key.to_string()),
            catalog_entry: Some(entry),
            interval_ms: None,
            clears_existing_triggers: false,
            config: request.config.clone(),
            uses_linked_cancellation: false,
        },
        TaskInvocationExecutionMode::ExecuteReady,
    )
}

pub fn independent_tasks() -> Vec<IndependentTaskDescriptor> {
    [
        (IndependentTaskKind::UseRedeemCode, USE_REDEEM_CODE_TASK_KEY),
        (
            IndependentTaskKind::AutoGeniusInvokation,
            "AutoGeniusInvokation",
        ),
        (IndependentTaskKind::AutoWood, "AutoWood"),
        (IndependentTaskKind::AutoFight, "AutoFight"),
        (IndependentTaskKind::AutoDomain, "AutoDomain"),
        (IndependentTaskKind::AutoTrack, "AutoTrack"),
        (IndependentTaskKind::AutoTrackPath, "AutoTrackPath"),
        (IndependentTaskKind::AutoMusicGame, "AutoMusicGame"),
        (IndependentTaskKind::AutoOpenChest, AUTO_OPEN_CHEST_TASK_KEY),
        (IndependentTaskKind::AutoEatFood, AUTO_EAT_FOOD_TASK_KEY),
        (
            IndependentTaskKind::AutoStygianOnslaught,
            "AutoStygianOnslaught",
        ),
        (IndependentTaskKind::AutoPathing, "AutoPathing"),
        (IndependentTaskKind::AutoBoss, "AutoBoss"),
        (
            IndependentTaskKind::AutoArtifactSalvage,
            "AutoArtifactSalvage",
        ),
        (
            IndependentTaskKind::AutoLeyLineOutcrop,
            "AutoLeyLineOutcrop",
        ),
        (IndependentTaskKind::GetGridIcons, "GetGridIcons"),
        (
            IndependentTaskKind::TurnAroundMacro,
            TURN_AROUND_MACRO_TASK_KEY,
        ),
        (
            IndependentTaskKind::QuickEnhanceArtifactMacro,
            QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY,
        ),
        (IndependentTaskKind::QuickBuy, QUICK_BUY_TASK_KEY),
        (
            IndependentTaskKind::QuickSereniteaPot,
            QUICK_SERENITEA_POT_TASK_KEY,
        ),
        (IndependentTaskKind::Shell, "Shell"),
    ]
    .into_iter()
    .filter_map(|(kind, key)| {
        find_task_catalog_entry(key).map(|entry| IndependentTaskDescriptor {
            rust_execution_surface: entry.rust_execution_surface(),
            kind,
            key: entry.key,
            display_name: entry.display_name,
            requires_main_ui_wait: entry.requires_main_ui_wait.unwrap_or(true),
            config_section: entry.config_section,
            hotkey_fields: entry.hotkey_fields,
            asset_roots: entry.asset_roots,
            ported: entry.port_state == TaskPortState::Ported,
            notes: entry.notes,
        })
    })
    .collect()
}
