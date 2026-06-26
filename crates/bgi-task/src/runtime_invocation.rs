use super::{DispatcherRuntime, Result};
use crate::auto_cook::{
    plan_auto_cook, AutoCookExecutionConfig, AutoCookExecutionPlan, AutoCookExecutionReport,
    AutoCookExecutionStatus, AUTO_COOK_TASK_KEY,
};
use crate::auto_eat::{
    plan_auto_eat, plan_auto_eat_food, AutoEatExecutionConfig, AutoEatExecutionPlan,
    AutoEatFoodExecutionConfig, AutoEatFoodExecutionPlan, AUTO_EAT_FOOD_TASK_KEY,
    AUTO_EAT_SCRIPT_TASK_NAME, AUTO_EAT_TASK_KEY,
};
use crate::auto_fish::{
    plan_auto_fish, AutoFishExecutionConfig, AutoFishExecutionPlan, AUTO_FISH_TASK_KEY,
};
use crate::auto_fishing_task::{
    plan_auto_fishing_task, AutoFishingTaskExecutionConfig, AutoFishingTaskExecutionPlan,
    AUTO_FISHING_TASK_KEY,
};
use crate::auto_music_game::{
    AutoMusicAlbumExecutionReport, AutoMusicAlbumExecutionStatus, AutoMusicPerformanceReport,
    AutoMusicPerformanceStopReason,
};
use crate::auto_open_chest::AutoOpenChestExecutionReport;
use crate::auto_pick::{
    plan_auto_pick, AutoPickExecutionConfig, AutoPickExecutionPlan, AUTO_PICK_TASK_KEY,
};
use crate::auto_skip::{
    plan_auto_skip, AutoSkipExecutionConfig, AutoSkipExecutionPlan, AUTO_SKIP_TASK_KEY,
};
use crate::catalog::{
    find_task_catalog_entry, TaskCatalogEntry, TaskLaunchPolicy, TaskPortState,
    TaskRustExecutionSurface,
};
use crate::game_loading::{
    plan_game_loading, GameLoadingExecutionConfig, GameLoadingExecutionPlan, GAME_LOADING_TASK_KEY,
};
use crate::macro_hotkeys::{QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY, TURN_AROUND_MACRO_TASK_KEY};
use crate::map_mask::{
    plan_map_mask, MapMaskExecutionConfig, MapMaskExecutionPlan, MAP_MASK_TASK_KEY,
};
use crate::quick_buy::{QuickBuyExecutionReport, QUICK_BUY_TASK_KEY};
use crate::quick_serenitea_pot::{QuickSereniteaPotExecutionReport, QUICK_SERENITEA_POT_TASK_KEY};
use crate::quick_teleport::{
    plan_quick_teleport, QuickTeleportExecutionConfig, QuickTeleportExecutionPlan,
    QUICK_TELEPORT_TASK_KEY,
};
use crate::redeem_code::UseRedeemCodeExecutionReport;
use crate::skill_cd::{
    plan_skill_cd, SkillCdExecutionConfig, SkillCdExecutionPlan, SKILL_CD_TASK_KEY,
};
use crate::TaskError;
use crate::{
    common_job_executor_bridge_plan, plan_common_job, BlessingOfTheWelkinMoonExecutionReport,
    CheckRewardsExecutionReport, ChooseTalkOptionExecutionReport,
    ClaimBattlePassRewardsExecutionReport, ClaimEncounterPointsRewardsExecutionReport,
    ClaimMailRewardsExecutionReport, CommonJobExecutionPlan, CommonJobExecutorBridgePlan,
    CountInventoryItemExecutionReport, GoToAdventurersGuildExecutionReport,
    GoToCraftingBenchExecutionReport, GoToSereniteaPotExecutionReport,
    LowerHeadThenWalkToExecutionReport, OneKeyExpeditionExecutionReport, ReloginExecutionReport,
    ReturnMainUiExecutionReport, ScanPickDropsExecutionReport, SetTimeExecutionReport,
    SwitchPartyExecutionReport, TeleportExecutionReport, WalkToFExecutionReport,
    WonderlandCycleExecutionReport,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatcherTimerInput {
    pub name: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub clears_existing_triggers: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatcherSoloTaskInput {
    pub name: String,
    pub config: Option<Value>,
    pub uses_linked_cancellation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptDispatcherCommandInput {
    ClearAllTriggers,
    AddRealtimeTimer(DispatcherTimerInput),
    RunCurrentTask,
    RunSoloTask(DispatcherSoloTaskInput),
    LinkedCancellationTokenSource,
    LinkedCancellationToken,
    RunBuiltinTask {
        name: String,
        config: Value,
        uses_linked_cancellation: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskInvocationKind {
    ClearRealtimeTriggers,
    AddRealtimeTrigger,
    RunCurrentTask,
    RunIndependentTask,
    RunScriptDispatcherTask,
    RunCommonJob,
    LinkedCancellationTokenSource,
    LinkedCancellationToken,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskInvocationPlan {
    pub kind: TaskInvocationKind,
    pub task_key: Option<String>,
    pub catalog_entry: Option<TaskCatalogEntry>,
    pub interval_ms: Option<u64>,
    pub clears_existing_triggers: bool,
    pub config: Option<Value>,
    pub uses_linked_cancellation: bool,
}

impl TaskInvocationPlan {
    pub fn from_script_dispatcher_command(command: ScriptDispatcherCommandInput) -> Result<Self> {
        match command {
            ScriptDispatcherCommandInput::ClearAllTriggers => Ok(Self {
                kind: TaskInvocationKind::ClearRealtimeTriggers,
                task_key: None,
                catalog_entry: None,
                interval_ms: None,
                clears_existing_triggers: true,
                config: None,
                uses_linked_cancellation: false,
            }),
            ScriptDispatcherCommandInput::AddRealtimeTimer(timer) => {
                let entry = catalog_entry_for_policy(&timer.name, TaskLaunchPolicy::RealtimeTick)?;
                Ok(Self {
                    kind: TaskInvocationKind::AddRealtimeTrigger,
                    task_key: Some(entry.key.to_string()),
                    catalog_entry: Some(entry),
                    interval_ms: Some(timer.interval_ms),
                    clears_existing_triggers: timer.clears_existing_triggers,
                    config: timer.config,
                    uses_linked_cancellation: false,
                })
            }
            ScriptDispatcherCommandInput::RunCurrentTask => Ok(Self {
                kind: TaskInvocationKind::RunCurrentTask,
                task_key: None,
                catalog_entry: None,
                interval_ms: None,
                clears_existing_triggers: false,
                config: None,
                uses_linked_cancellation: true,
            }),
            ScriptDispatcherCommandInput::RunSoloTask(task) => {
                let entry = catalog_entry_for_policy(&task.name, TaskLaunchPolicy::SoloTask)?;
                Ok(Self {
                    kind: TaskInvocationKind::RunIndependentTask,
                    task_key: Some(entry.key.to_string()),
                    catalog_entry: Some(entry),
                    interval_ms: None,
                    clears_existing_triggers: false,
                    config: task.config,
                    uses_linked_cancellation: task.uses_linked_cancellation,
                })
            }
            ScriptDispatcherCommandInput::LinkedCancellationTokenSource => Ok(Self {
                kind: TaskInvocationKind::LinkedCancellationTokenSource,
                task_key: None,
                catalog_entry: None,
                interval_ms: None,
                clears_existing_triggers: false,
                config: None,
                uses_linked_cancellation: true,
            }),
            ScriptDispatcherCommandInput::LinkedCancellationToken => Ok(Self {
                kind: TaskInvocationKind::LinkedCancellationToken,
                task_key: None,
                catalog_entry: None,
                interval_ms: None,
                clears_existing_triggers: false,
                config: None,
                uses_linked_cancellation: true,
            }),
            ScriptDispatcherCommandInput::RunBuiltinTask {
                name,
                config,
                uses_linked_cancellation,
            } => {
                if name.trim().is_empty() {
                    return Err(TaskError::MissingTaskName);
                }
                let catalog_lookup_name = if name.eq_ignore_ascii_case(AUTO_EAT_SCRIPT_TASK_NAME) {
                    AUTO_EAT_FOOD_TASK_KEY
                } else {
                    name.as_str()
                };
                let entry = find_task_catalog_entry(catalog_lookup_name)
                    .ok_or_else(|| TaskError::UnknownIndependentTask(name.clone()))?;
                let supported_hotkey_task = matches!(
                    entry.key,
                    QUICK_BUY_TASK_KEY
                        | QUICK_SERENITEA_POT_TASK_KEY
                        | TURN_AROUND_MACRO_TASK_KEY
                        | QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY
                );
                let supported_launch_policy = matches!(
                    entry.launch_policy,
                    TaskLaunchPolicy::SoloTask
                        | TaskLaunchPolicy::ScriptDispatcher
                        | TaskLaunchPolicy::CommonJob
                ) || (entry.launch_policy
                    == TaskLaunchPolicy::HotkeyCommand
                    && supported_hotkey_task);
                if !supported_launch_policy {
                    return Err(TaskError::InvalidLaunchPolicy {
                        key: entry.key.to_string(),
                        expected: TaskLaunchPolicy::ScriptDispatcher,
                        actual: entry.launch_policy,
                    });
                }
                let kind = match entry.launch_policy {
                    TaskLaunchPolicy::SoloTask | TaskLaunchPolicy::HotkeyCommand => {
                        TaskInvocationKind::RunIndependentTask
                    }
                    TaskLaunchPolicy::ScriptDispatcher => {
                        TaskInvocationKind::RunScriptDispatcherTask
                    }
                    TaskLaunchPolicy::CommonJob => TaskInvocationKind::RunCommonJob,
                    _ => unreachable!("launch policy was validated above"),
                };
                Ok(Self {
                    kind,
                    task_key: Some(entry.key.to_string()),
                    catalog_entry: Some(entry),
                    interval_ms: None,
                    clears_existing_triggers: false,
                    config: Some(config),
                    uses_linked_cancellation,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskInvocationExecutionMode {
    PlanOnly,
    ExecuteReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskInvocationExecutionStatus {
    Planned,
    Ready,
    RustInvocationPlanReady,
    RustExecutionPlanReady,
    NativePending,
    RuntimeOnly,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RealtimeTriggerExecutionPlan {
    AutoEat(AutoEatExecutionPlan),
    AutoFish(AutoFishExecutionPlan),
    AutoPick(AutoPickExecutionPlan),
    AutoSkip(AutoSkipExecutionPlan),
    GameLoading(GameLoadingExecutionPlan),
    MapMask(MapMaskExecutionPlan),
    QuickTeleport(QuickTeleportExecutionPlan),
    SkillCd(SkillCdExecutionPlan),
}

impl RealtimeTriggerExecutionPlan {
    pub fn task_key(&self) -> &str {
        match self {
            RealtimeTriggerExecutionPlan::AutoEat(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::AutoFish(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::AutoPick(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::AutoSkip(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::GameLoading(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::MapMask(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::QuickTeleport(plan) => &plan.task_key,
            RealtimeTriggerExecutionPlan::SkillCd(plan) => &plan.task_key,
        }
    }

    pub fn executor_ready(&self) -> bool {
        match self {
            RealtimeTriggerExecutionPlan::AutoEat(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::AutoFish(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::AutoPick(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::AutoSkip(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::GameLoading(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::MapMask(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::QuickTeleport(plan) => plan.executor_ready,
            RealtimeTriggerExecutionPlan::SkillCd(plan) => plan.executor_ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ScriptDispatcherExecutionPlan {
    AutoCook(AutoCookExecutionPlan),
    AutoEatFood(AutoEatFoodExecutionPlan),
    AutoFishing(AutoFishingTaskExecutionPlan),
}

impl ScriptDispatcherExecutionPlan {
    pub fn task_key(&self) -> &str {
        match self {
            ScriptDispatcherExecutionPlan::AutoCook(plan) => &plan.task_key,
            ScriptDispatcherExecutionPlan::AutoEatFood(plan) => &plan.task_key,
            ScriptDispatcherExecutionPlan::AutoFishing(plan) => &plan.task_key,
        }
    }

    pub fn executor_ready(&self) -> bool {
        match self {
            ScriptDispatcherExecutionPlan::AutoCook(plan) => plan.executor_ready,
            ScriptDispatcherExecutionPlan::AutoEatFood(plan) => plan.executor_ready,
            ScriptDispatcherExecutionPlan::AutoFishing(plan) => plan.executor_ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptDispatcherLiveExecutionReport {
    AutoCook(AutoCookExecutionReport),
}

impl ScriptDispatcherLiveExecutionReport {
    fn task_name(&self) -> &'static str {
        match self {
            ScriptDispatcherLiveExecutionReport::AutoCook(_) => "AutoCook",
        }
    }

    fn summary(&self) -> String {
        match self {
            ScriptDispatcherLiveExecutionReport::AutoCook(report) => format!(
                "status={:?}, frames_processed={}, space_press_count={}, white_confirm_click_count={}",
                report.status,
                report.state.frames_processed,
                report.state.space_press_count,
                report.state.white_confirm_click_count
            ),
        }
    }

    fn completed(&self) -> bool {
        match self {
            ScriptDispatcherLiveExecutionReport::AutoCook(report) => {
                report.status != AutoCookExecutionStatus::IterationLimitReached
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IndependentTaskLiveExecutionReport {
    AutoOpenChest(AutoOpenChestExecutionReport),
    AutoMusicGamePerformance(AutoMusicPerformanceReport),
    AutoMusicGameAlbum(AutoMusicAlbumExecutionReport),
    QuickBuy(QuickBuyExecutionReport),
    QuickSereniteaPot(QuickSereniteaPotExecutionReport),
    UseRedeemCode(UseRedeemCodeExecutionReport),
}

impl IndependentTaskLiveExecutionReport {
    pub fn task_name(&self) -> &'static str {
        match self {
            IndependentTaskLiveExecutionReport::AutoOpenChest(_) => "AutoOpenChest",
            IndependentTaskLiveExecutionReport::AutoMusicGamePerformance(_) => {
                "AutoMusicGame:Performance"
            }
            IndependentTaskLiveExecutionReport::AutoMusicGameAlbum(_) => "AutoMusicGame:Album",
            IndependentTaskLiveExecutionReport::QuickBuy(_) => "QuickBuy",
            IndependentTaskLiveExecutionReport::QuickSereniteaPot(_) => "QuickSereniteaPot",
            IndependentTaskLiveExecutionReport::UseRedeemCode(_) => "UseRedeemCode",
        }
    }

    pub fn completed(&self) -> bool {
        match self {
            IndependentTaskLiveExecutionReport::AutoOpenChest(report) => report.completed,
            IndependentTaskLiveExecutionReport::AutoMusicGamePerformance(report) => {
                report.stop_reason != AutoMusicPerformanceStopReason::CancelledBeforeFrame
            }
            IndependentTaskLiveExecutionReport::AutoMusicGameAlbum(report) => {
                report.status == AutoMusicAlbumExecutionStatus::Completed
            }
            IndependentTaskLiveExecutionReport::QuickBuy(report) => report.completed,
            IndependentTaskLiveExecutionReport::QuickSereniteaPot(report) => report.completed,
            IndependentTaskLiveExecutionReport::UseRedeemCode(report) => report.completed,
        }
    }

    pub fn executed_steps(&self) -> usize {
        match self {
            IndependentTaskLiveExecutionReport::AutoOpenChest(report) => {
                report.dispatched_actions.len()
                    + report.cleanup_actions.len()
                    + report.post_loop_actions.len()
            }
            IndependentTaskLiveExecutionReport::AutoMusicGamePerformance(report) => {
                report.frames_processed
            }
            IndependentTaskLiveExecutionReport::AutoMusicGameAlbum(report) => {
                report.performed_songs as usize
            }
            IndependentTaskLiveExecutionReport::QuickBuy(report) => report.executed_steps.len(),
            IndependentTaskLiveExecutionReport::QuickSereniteaPot(report) => {
                report.executed_steps.len()
            }
            IndependentTaskLiveExecutionReport::UseRedeemCode(report) => {
                report.executed_steps.len()
            }
        }
    }

    pub fn skipped_steps(&self) -> usize {
        match self {
            IndependentTaskLiveExecutionReport::AutoOpenChest(_) => 0,
            IndependentTaskLiveExecutionReport::AutoMusicGamePerformance(_) => 0,
            IndependentTaskLiveExecutionReport::AutoMusicGameAlbum(report) => {
                report.skipped_songs as usize
            }
            IndependentTaskLiveExecutionReport::QuickBuy(report) => report.skipped_steps.len(),
            IndependentTaskLiveExecutionReport::QuickSereniteaPot(report) => {
                report.skipped_steps.len()
            }
            IndependentTaskLiveExecutionReport::UseRedeemCode(report) => report.skipped_steps.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommonJobLiveExecutionReport {
    ReturnMainUi(ReturnMainUiExecutionReport),
    SetTime(SetTimeExecutionReport),
    ChooseTalkOption(ChooseTalkOptionExecutionReport),
    CheckRewards(CheckRewardsExecutionReport),
    BlessingOfTheWelkinMoon(BlessingOfTheWelkinMoonExecutionReport),
    ClaimBattlePassRewards(ClaimBattlePassRewardsExecutionReport),
    ClaimEncounterPointsRewards(ClaimEncounterPointsRewardsExecutionReport),
    ClaimMailRewards(ClaimMailRewardsExecutionReport),
    CountInventoryItem(CountInventoryItemExecutionReport),
    ScanPickDrops(ScanPickDropsExecutionReport),
    LowerHeadThenWalkTo(LowerHeadThenWalkToExecutionReport),
    SwitchParty(SwitchPartyExecutionReport),
    GoToCraftingBench(GoToCraftingBenchExecutionReport),
    Teleport(TeleportExecutionReport),
    GoToAdventurersGuild(GoToAdventurersGuildExecutionReport),
    GoToSereniteaPot(GoToSereniteaPotExecutionReport),
    OneKeyExpedition(OneKeyExpeditionExecutionReport),
    WonderlandCycle(WonderlandCycleExecutionReport),
    Relogin(ReloginExecutionReport),
    WalkToF(WalkToFExecutionReport),
}

impl CommonJobLiveExecutionReport {
    fn task_name(&self) -> &'static str {
        match self {
            CommonJobLiveExecutionReport::ReturnMainUi(_) => "ReturnMainUi",
            CommonJobLiveExecutionReport::SetTime(_) => "SetTime",
            CommonJobLiveExecutionReport::ChooseTalkOption(_) => "ChooseTalkOption",
            CommonJobLiveExecutionReport::CheckRewards(_) => "CheckRewards",
            CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon(_) => "BlessingOfTheWelkinMoon",
            CommonJobLiveExecutionReport::ClaimBattlePassRewards(_) => "ClaimBattlePassRewards",
            CommonJobLiveExecutionReport::ClaimEncounterPointsRewards(_) => {
                "ClaimEncounterPointsRewards"
            }
            CommonJobLiveExecutionReport::ClaimMailRewards(_) => "ClaimMailRewards",
            CommonJobLiveExecutionReport::CountInventoryItem(_) => "CountInventoryItem",
            CommonJobLiveExecutionReport::ScanPickDrops(_) => "ScanPickDrops",
            CommonJobLiveExecutionReport::LowerHeadThenWalkTo(_) => "LowerHeadThenWalkTo",
            CommonJobLiveExecutionReport::SwitchParty(_) => "SwitchParty",
            CommonJobLiveExecutionReport::GoToCraftingBench(_) => "GoToCraftingBench",
            CommonJobLiveExecutionReport::Teleport(_) => "Teleport",
            CommonJobLiveExecutionReport::GoToAdventurersGuild(_) => "GoToAdventurersGuild",
            CommonJobLiveExecutionReport::GoToSereniteaPot(_) => "GoToSereniteaPot",
            CommonJobLiveExecutionReport::OneKeyExpedition(_) => "OneKeyExpedition",
            CommonJobLiveExecutionReport::WonderlandCycle(_) => "WonderlandCycle",
            CommonJobLiveExecutionReport::Relogin(_) => "Relogin",
            CommonJobLiveExecutionReport::WalkToF(_) => "WalkToF",
        }
    }

    fn completed(&self) -> bool {
        match self {
            CommonJobLiveExecutionReport::ReturnMainUi(report) => report.completed,
            CommonJobLiveExecutionReport::SetTime(report) => report.completed,
            CommonJobLiveExecutionReport::ChooseTalkOption(report) => report.completed,
            CommonJobLiveExecutionReport::CheckRewards(report) => report.completed,
            CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon(report) => report.completed,
            CommonJobLiveExecutionReport::ClaimBattlePassRewards(report) => report.completed,
            CommonJobLiveExecutionReport::ClaimEncounterPointsRewards(report) => report.completed,
            CommonJobLiveExecutionReport::ClaimMailRewards(report) => report.completed,
            CommonJobLiveExecutionReport::CountInventoryItem(report) => report.completed,
            CommonJobLiveExecutionReport::ScanPickDrops(report) => report.completed,
            CommonJobLiveExecutionReport::LowerHeadThenWalkTo(report) => report.completed,
            CommonJobLiveExecutionReport::SwitchParty(report) => report.completed,
            CommonJobLiveExecutionReport::GoToCraftingBench(report) => report.completed,
            CommonJobLiveExecutionReport::Teleport(report) => report.completed,
            CommonJobLiveExecutionReport::GoToAdventurersGuild(report) => report.completed,
            CommonJobLiveExecutionReport::GoToSereniteaPot(report) => report.completed,
            CommonJobLiveExecutionReport::OneKeyExpedition(report) => report.completed,
            CommonJobLiveExecutionReport::WonderlandCycle(report) => report.completed,
            CommonJobLiveExecutionReport::Relogin(report) => report.completed,
            CommonJobLiveExecutionReport::WalkToF(report) => report.completed,
        }
    }

    fn executed_steps(&self) -> usize {
        match self {
            CommonJobLiveExecutionReport::ReturnMainUi(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::SetTime(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::ChooseTalkOption(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::CheckRewards(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon(report) => {
                report.executed_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimBattlePassRewards(report) => {
                report.executed_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimEncounterPointsRewards(report) => {
                report.executed_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimMailRewards(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::CountInventoryItem(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::ScanPickDrops(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::LowerHeadThenWalkTo(report) => {
                report.executed_steps.len()
            }
            CommonJobLiveExecutionReport::SwitchParty(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::GoToCraftingBench(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::Teleport(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::GoToAdventurersGuild(report) => {
                report.executed_steps.len()
            }
            CommonJobLiveExecutionReport::GoToSereniteaPot(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::OneKeyExpedition(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::WonderlandCycle(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::Relogin(report) => report.executed_steps.len(),
            CommonJobLiveExecutionReport::WalkToF(report) => report.executed_steps.len(),
        }
    }

    fn skipped_steps(&self) -> usize {
        match self {
            CommonJobLiveExecutionReport::ReturnMainUi(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::SetTime(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::ChooseTalkOption(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::CheckRewards(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon(report) => {
                report.skipped_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimBattlePassRewards(report) => {
                report.skipped_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimEncounterPointsRewards(report) => {
                report.skipped_steps.len()
            }
            CommonJobLiveExecutionReport::ClaimMailRewards(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::CountInventoryItem(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::ScanPickDrops(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::LowerHeadThenWalkTo(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::SwitchParty(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::GoToCraftingBench(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::Teleport(_) => 0,
            CommonJobLiveExecutionReport::GoToAdventurersGuild(report) => {
                report.skipped_steps.len()
            }
            CommonJobLiveExecutionReport::GoToSereniteaPot(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::OneKeyExpedition(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::WonderlandCycle(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::Relogin(report) => report.skipped_steps.len(),
            CommonJobLiveExecutionReport::WalkToF(report) => report.skipped_steps.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskInvocationExecutionResult {
    pub plan: TaskInvocationPlan,
    pub mode: TaskInvocationExecutionMode,
    pub status: TaskInvocationExecutionStatus,
    pub message: String,
    pub common_job_execution_plan: Option<CommonJobExecutionPlan>,
    pub common_job_executor_bridge_plan: Option<CommonJobExecutorBridgePlan>,
    pub common_job_live_execution: Option<CommonJobLiveExecutionReport>,
    pub independent_task_live_execution: Option<IndependentTaskLiveExecutionReport>,
    pub realtime_trigger_execution_plan: Option<RealtimeTriggerExecutionPlan>,
    pub script_dispatcher_execution_plan: Option<ScriptDispatcherExecutionPlan>,
    pub script_dispatcher_live_execution: Option<ScriptDispatcherLiveExecutionReport>,
    pub executed: bool,
}

impl TaskInvocationExecutionResult {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self.status,
            TaskInvocationExecutionStatus::Ready
                | TaskInvocationExecutionStatus::NativePending
                | TaskInvocationExecutionStatus::RustInvocationPlanReady
                | TaskInvocationExecutionStatus::RustExecutionPlanReady
        )
    }
}

pub fn evaluate_task_invocation_plan(
    plan: TaskInvocationPlan,
    mode: TaskInvocationExecutionMode,
) -> TaskInvocationExecutionResult {
    let (mut status, mut message, common_job_execution_plan) = match plan.kind {
        TaskInvocationKind::ClearRealtimeTriggers => (
            TaskInvocationExecutionStatus::Ready,
            "clears registered realtime triggers".to_string(),
            None,
        ),
        TaskInvocationKind::LinkedCancellationTokenSource
        | TaskInvocationKind::LinkedCancellationToken => (
            TaskInvocationExecutionStatus::RuntimeOnly,
            "provides a script cancellation handle; no native task is launched".to_string(),
            None,
        ),
        TaskInvocationKind::RunCurrentTask => (
            TaskInvocationExecutionStatus::NativePending,
            "requires the native current-task runner".to_string(),
            None,
        ),
        TaskInvocationKind::AddRealtimeTrigger
        | TaskInvocationKind::RunIndependentTask
        | TaskInvocationKind::RunScriptDispatcherTask
        | TaskInvocationKind::RunCommonJob => match plan.catalog_entry.as_ref() {
            Some(entry) if entry.port_state == TaskPortState::Ported => (
                TaskInvocationExecutionStatus::Ready,
                format!("{} is marked ported", entry.key),
                None,
            ),
            Some(entry)
                if plan.kind == TaskInvocationKind::RunIndependentTask
                    && entry.rust_execution_surface()
                        == TaskRustExecutionSurface::ExecutionPlanOnly =>
            {
                (
                    TaskInvocationExecutionStatus::RustExecutionPlanReady,
                    format!(
                        "{} has a Rust execution plan; direct native loop execution remains pending",
                        entry.key
                    ),
                    None,
                )
            }
            Some(entry)
                if plan.kind == TaskInvocationKind::RunCommonJob
                    && entry.rust_execution_surface()
                        == TaskRustExecutionSurface::ExecutionPlanOnly =>
            {
                match plan_common_job(entry.key, plan.config.as_ref()) {
                    Ok(Some(common_job_plan)) => (
                        TaskInvocationExecutionStatus::RustExecutionPlanReady,
                        format!(
                            "{} has a Rust common-job execution plan; direct executor remains pending",
                            entry.key
                        ),
                        Some(common_job_plan),
                    ),
                    Ok(None) => (
                        TaskInvocationExecutionStatus::NativePending,
                        format!("{} is {:?}", entry.key, entry.port_state),
                        None,
                    ),
                    Err(error) => (
                        TaskInvocationExecutionStatus::Invalid,
                        error.to_string(),
                        None,
                    ),
                }
            }
            Some(entry)
                if plan.kind == TaskInvocationKind::RunScriptDispatcherTask
                    && entry.rust_execution_surface()
                        == TaskRustExecutionSurface::ExecutionPlanOnly =>
            {
                (
                    TaskInvocationExecutionStatus::RustExecutionPlanReady,
                    format!(
                        "{} has a Rust execution plan; direct native loop execution remains pending",
                        entry.key
                    ),
                    None,
                )
            }
            Some(entry)
                if plan.kind == TaskInvocationKind::RunCommonJob
                    && entry.rust_execution_surface()
                        == TaskRustExecutionSurface::InvocationPlanOnly =>
            {
                (
                    TaskInvocationExecutionStatus::RustInvocationPlanReady,
                    format!(
                        "{} has a Rust task invocation plan; native common-job execution remains pending",
                        entry.key
                    ),
                    None,
                )
            }
            Some(entry) => (
                TaskInvocationExecutionStatus::NativePending,
                format!("{} is {:?}", entry.key, entry.port_state),
                None,
            ),
            None => (
                TaskInvocationExecutionStatus::Invalid,
                "missing task catalog entry".to_string(),
                None,
            ),
        },
    };
    let realtime_trigger_execution_plan = if plan.kind == TaskInvocationKind::AddRealtimeTrigger {
        match plan.catalog_entry.as_ref() {
            Some(entry)
                if entry.rust_execution_surface()
                    == TaskRustExecutionSurface::ExecutionPlanOnly =>
            {
                match plan_realtime_trigger_tick(entry.key, plan.config.as_ref()) {
                    Ok(Some(realtime_plan)) => {
                        status = TaskInvocationExecutionStatus::RustExecutionPlanReady;
                        message = format!(
                            "{} has a Rust realtime-trigger execution plan; direct tick execution remains pending",
                            entry.key
                        );
                        Some(realtime_plan)
                    }
                    Ok(None) => None,
                    Err(error) => {
                        status = TaskInvocationExecutionStatus::Invalid;
                        message = error.to_string();
                        None
                    }
                }
            }
            _ => None,
        }
    } else {
        None
    };

    let script_dispatcher_execution_plan = if plan.kind
        == TaskInvocationKind::RunScriptDispatcherTask
    {
        match plan.catalog_entry.as_ref() {
            Some(entry)
                if entry.rust_execution_surface()
                    == TaskRustExecutionSurface::ExecutionPlanOnly =>
            {
                match plan_script_dispatcher_task(entry.key, plan.config.as_ref()) {
                    Ok(Some(script_plan)) => {
                        status = TaskInvocationExecutionStatus::RustExecutionPlanReady;
                        message = format!(
                            "{} has a Rust script-dispatcher execution plan; direct executor remains pending",
                            entry.key
                        );
                        Some(script_plan)
                    }
                    Ok(None) => None,
                    Err(error) => {
                        status = TaskInvocationExecutionStatus::Invalid;
                        message = error.to_string();
                        None
                    }
                }
            }
            _ => None,
        }
    } else {
        None
    };

    let common_job_executor_bridge_plan = common_job_execution_plan
        .as_ref()
        .and_then(common_job_executor_bridge_plan);
    let executed = false;

    TaskInvocationExecutionResult {
        plan,
        mode,
        status: if mode == TaskInvocationExecutionMode::PlanOnly
            && status == TaskInvocationExecutionStatus::Ready
        {
            TaskInvocationExecutionStatus::Planned
        } else {
            status
        },
        message,
        common_job_execution_plan,
        common_job_executor_bridge_plan,
        common_job_live_execution: None,
        independent_task_live_execution: None,
        realtime_trigger_execution_plan,
        script_dispatcher_execution_plan,
        script_dispatcher_live_execution: None,
        executed,
    }
}

pub fn evaluate_task_invocation_plans(
    plans: impl IntoIterator<Item = TaskInvocationPlan>,
    mode: TaskInvocationExecutionMode,
) -> Vec<TaskInvocationExecutionResult> {
    plans
        .into_iter()
        .map(|plan| evaluate_task_invocation_plan(plan, mode))
        .collect()
}

pub fn execute_task_invocation_plan(
    dispatcher: &mut DispatcherRuntime,
    plan: TaskInvocationPlan,
) -> TaskInvocationExecutionResult {
    let mut result =
        evaluate_task_invocation_plan(plan.clone(), TaskInvocationExecutionMode::ExecuteReady);
    match plan.kind {
        TaskInvocationKind::ClearRealtimeTriggers => {
            let cleared = dispatcher.clear_registered_realtime_triggers();
            result.status = TaskInvocationExecutionStatus::Ready;
            result.executed = true;
            result.message = format!("cleared {cleared} registered realtime trigger(s)");
        }
        TaskInvocationKind::AddRealtimeTrigger => {
            match dispatcher.add_registered_realtime_trigger(&plan) {
                Ok(()) => {
                    result.status = TaskInvocationExecutionStatus::Ready;
                    result.executed = true;
                    result.message = format!(
                        "registered realtime trigger {} at {} ms",
                        plan.task_key.as_deref().unwrap_or("<unknown>"),
                        plan.interval_ms.unwrap_or(dispatcher.interval_ms)
                    );
                }
                Err(error) => {
                    result.status = TaskInvocationExecutionStatus::Invalid;
                    result.executed = false;
                    result.message = error.to_string();
                }
            }
        }
        _ => {}
    }
    result
}

pub fn execute_task_invocation_plan_with_live_executor<F>(
    dispatcher: &mut DispatcherRuntime,
    plan: TaskInvocationPlan,
    live_executor: &mut F,
) -> TaskInvocationExecutionResult
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let mut script_dispatcher_live_executor = no_script_dispatcher_live_executor;
    execute_task_invocation_plan_with_live_executors(
        dispatcher,
        plan,
        live_executor,
        &mut script_dispatcher_live_executor,
    )
}

pub fn execute_task_invocation_plan_with_live_executors<F, G>(
    dispatcher: &mut DispatcherRuntime,
    plan: TaskInvocationPlan,
    common_job_live_executor: &mut F,
    script_dispatcher_live_executor: &mut G,
) -> TaskInvocationExecutionResult
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
    G: FnMut(&ScriptDispatcherExecutionPlan) -> Result<Option<ScriptDispatcherLiveExecutionReport>>,
{
    let mut result = execute_task_invocation_plan(dispatcher, plan);
    execute_common_job_live_if_available(&mut result, common_job_live_executor);
    execute_script_dispatcher_live_if_available(&mut result, script_dispatcher_live_executor);
    result
}

pub fn execute_task_invocation_plans(
    dispatcher: &mut DispatcherRuntime,
    plans: impl IntoIterator<Item = TaskInvocationPlan>,
) -> Vec<TaskInvocationExecutionResult> {
    plans
        .into_iter()
        .map(|plan| execute_task_invocation_plan(dispatcher, plan))
        .collect()
}

pub fn execute_task_invocation_plans_with_live_executor<F>(
    dispatcher: &mut DispatcherRuntime,
    plans: impl IntoIterator<Item = TaskInvocationPlan>,
    mut live_executor: F,
) -> Vec<TaskInvocationExecutionResult>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    plans
        .into_iter()
        .map(|plan| {
            execute_task_invocation_plan_with_live_executor(dispatcher, plan, &mut live_executor)
        })
        .collect()
}

pub fn execute_task_invocation_plans_with_live_executors<F, G>(
    dispatcher: &mut DispatcherRuntime,
    plans: impl IntoIterator<Item = TaskInvocationPlan>,
    mut common_job_live_executor: F,
    mut script_dispatcher_live_executor: G,
) -> Vec<TaskInvocationExecutionResult>
where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
    G: FnMut(&ScriptDispatcherExecutionPlan) -> Result<Option<ScriptDispatcherLiveExecutionReport>>,
{
    plans
        .into_iter()
        .map(|plan| {
            execute_task_invocation_plan_with_live_executors(
                dispatcher,
                plan,
                &mut common_job_live_executor,
                &mut script_dispatcher_live_executor,
            )
        })
        .collect()
}

fn execute_common_job_live_if_available<F>(
    result: &mut TaskInvocationExecutionResult,
    live_executor: &mut F,
) where
    F: FnMut(&CommonJobExecutionPlan) -> Result<Option<CommonJobLiveExecutionReport>>,
{
    let Some(plan) = result.common_job_execution_plan.as_ref() else {
        return;
    };
    if result.status != TaskInvocationExecutionStatus::RustExecutionPlanReady {
        return;
    }
    if !matches!(
        plan,
        CommonJobExecutionPlan::ReturnMainUi(_)
            | CommonJobExecutionPlan::SetTime(_)
            | CommonJobExecutionPlan::ChooseTalkOption(_)
            | CommonJobExecutionPlan::CheckRewards(_)
            | CommonJobExecutionPlan::BlessingOfTheWelkinMoon(_)
            | CommonJobExecutionPlan::ClaimBattlePassRewards(_)
            | CommonJobExecutionPlan::ClaimEncounterPointsRewards(_)
            | CommonJobExecutionPlan::ClaimMailRewards(_)
            | CommonJobExecutionPlan::CountInventoryItem(_)
            | CommonJobExecutionPlan::ScanPickDrops(_)
            | CommonJobExecutionPlan::LowerHeadThenWalkTo(_)
            | CommonJobExecutionPlan::SwitchParty(_)
            | CommonJobExecutionPlan::GoToCraftingBench(_)
            | CommonJobExecutionPlan::Teleport(_)
            | CommonJobExecutionPlan::GoToAdventurersGuild(_)
            | CommonJobExecutionPlan::GoToSereniteaPot(_)
            | CommonJobExecutionPlan::OneKeyExpedition(_)
            | CommonJobExecutionPlan::WonderlandCycle(_)
            | CommonJobExecutionPlan::Relogin(_)
            | CommonJobExecutionPlan::WalkToF(_)
    ) {
        return;
    }
    let plan = plan.clone();

    match live_executor(&plan) {
        Ok(Some(report)) => {
            let task_name = report.task_name();
            let completed = report.completed();
            let executed_steps = report.executed_steps();
            let skipped_steps = report.skipped_steps();
            result.status = TaskInvocationExecutionStatus::Ready;
            result.executed = true;
            result.message = format!(
                "{task_name} live execution completed: completed={completed}, executed_steps={executed_steps}, skipped_steps={skipped_steps}"
            );
            result.common_job_live_execution = Some(report);
        }
        Ok(None) => {
            result.common_job_live_execution = None;
        }
        Err(error) => {
            let task_name = plan.task_key();
            result.status = TaskInvocationExecutionStatus::Invalid;
            result.executed = false;
            result.message = format!("{task_name} live execution failed: {error}");
            result.common_job_live_execution = None;
        }
    }
}

fn execute_script_dispatcher_live_if_available<F>(
    result: &mut TaskInvocationExecutionResult,
    live_executor: &mut F,
) where
    F: FnMut(&ScriptDispatcherExecutionPlan) -> Result<Option<ScriptDispatcherLiveExecutionReport>>,
{
    let Some(plan) = result.script_dispatcher_execution_plan.as_ref() else {
        return;
    };
    if result.status != TaskInvocationExecutionStatus::RustExecutionPlanReady {
        return;
    }
    if !matches!(plan, ScriptDispatcherExecutionPlan::AutoCook(_)) {
        return;
    }
    let plan = plan.clone();

    match live_executor(&plan) {
        Ok(Some(report)) => {
            let task_name = report.task_name();
            let completed = report.completed();
            let summary = report.summary();
            result.status = TaskInvocationExecutionStatus::Ready;
            result.executed = true;
            result.message =
                format!("{task_name} live execution completed: completed={completed}, {summary}");
            result.script_dispatcher_live_execution = Some(report);
        }
        Ok(None) => {
            result.script_dispatcher_live_execution = None;
        }
        Err(error) => {
            let task_name = plan.task_key();
            result.status = TaskInvocationExecutionStatus::Invalid;
            result.executed = false;
            result.message = format!("{task_name} live execution failed: {error}");
            result.script_dispatcher_live_execution = None;
        }
    }
}

fn no_script_dispatcher_live_executor(
    _plan: &ScriptDispatcherExecutionPlan,
) -> Result<Option<ScriptDispatcherLiveExecutionReport>> {
    Ok(None)
}

fn catalog_entry_for_policy(key: &str, expected: TaskLaunchPolicy) -> Result<TaskCatalogEntry> {
    if key.trim().is_empty() {
        return Err(TaskError::MissingTaskName);
    }
    let entry = find_task_catalog_entry(key).ok_or_else(|| match expected {
        TaskLaunchPolicy::RealtimeTick => TaskError::UnknownTrigger(key.to_string()),
        _ => TaskError::UnknownIndependentTask(key.to_string()),
    })?;
    if entry.launch_policy != expected {
        return Err(TaskError::InvalidLaunchPolicy {
            key: entry.key.to_string(),
            expected,
            actual: entry.launch_policy,
        });
    }
    Ok(entry)
}

pub fn plan_realtime_trigger_tick(
    task_key: &str,
    config: Option<&Value>,
) -> Result<Option<RealtimeTriggerExecutionPlan>> {
    match task_key {
        AUTO_EAT_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::AutoEat(plan_auto_eat(
            AutoEatExecutionConfig::from_value(config),
        )))),
        AUTO_FISH_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::AutoFish(
            plan_auto_fish(AutoFishExecutionConfig::from_value(config)),
        ))),
        AUTO_PICK_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::AutoPick(
            plan_auto_pick(AutoPickExecutionConfig::from_value(config)),
        ))),
        AUTO_SKIP_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::AutoSkip(
            plan_auto_skip(AutoSkipExecutionConfig::from_value(config)),
        ))),
        GAME_LOADING_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::GameLoading(
            plan_game_loading(GameLoadingExecutionConfig::from_value(config)),
        ))),
        MAP_MASK_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::MapMask(plan_map_mask(
            MapMaskExecutionConfig::from_value(config),
        )))),
        QUICK_TELEPORT_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::QuickTeleport(
            plan_quick_teleport(QuickTeleportExecutionConfig::from_value(config)),
        ))),
        SKILL_CD_TASK_KEY => Ok(Some(RealtimeTriggerExecutionPlan::SkillCd(plan_skill_cd(
            SkillCdExecutionConfig::from_value(config),
        )))),
        _ => Ok(None),
    }
}

pub fn plan_script_dispatcher_task(
    task_key: &str,
    config: Option<&Value>,
) -> Result<Option<ScriptDispatcherExecutionPlan>> {
    match task_key {
        AUTO_COOK_TASK_KEY => Ok(Some(ScriptDispatcherExecutionPlan::AutoCook(
            plan_auto_cook(AutoCookExecutionConfig::from_value(config)),
        ))),
        AUTO_EAT_FOOD_TASK_KEY => Ok(Some(ScriptDispatcherExecutionPlan::AutoEatFood(
            plan_auto_eat_food(AutoEatFoodExecutionConfig::from_value(config)?)?,
        ))),
        AUTO_FISHING_TASK_KEY => Ok(Some(ScriptDispatcherExecutionPlan::AutoFishing(
            plan_auto_fishing_task(AutoFishingTaskExecutionConfig::from_value(config)),
        ))),
        _ => Ok(None),
    }
}
