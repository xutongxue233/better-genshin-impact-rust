use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    RealtimeTrigger,
    Independent,
    ScriptGroup,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskLaunchPolicy {
    RealtimeTick,
    SoloTask,
    ScriptDispatcher,
    ScriptGroupStep,
    HotkeyCommand,
    CommonJob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPortState {
    MetadataOnly,
    ConfigBound,
    RuntimeScaffolded,
    NativePending,
    Ported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskRustExecutionSurface {
    None,
    InvocationPlanOnly,
    ExecutionPlanOnly,
    DirectExecution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskCatalogEntry {
    pub key: &'static str,
    pub display_name: &'static str,
    pub kind: TaskKind,
    pub legacy_type: &'static str,
    pub legacy_reference: &'static str,
    pub config_section: Option<&'static str>,
    pub hotkey_fields: &'static [&'static str],
    pub asset_roots: &'static [&'static str],
    pub launch_policy: TaskLaunchPolicy,
    pub requires_main_ui_wait: Option<bool>,
    pub port_state: TaskPortState,
    pub notes: &'static str,
}

impl TaskCatalogEntry {
    pub fn is_independent(&self) -> bool {
        self.kind == TaskKind::Independent
    }

    pub fn config_bound(&self) -> bool {
        self.config_section.is_some()
    }

    pub fn rust_execution_surface(&self) -> TaskRustExecutionSurface {
        rust_execution_surface_for_task(self.key)
    }

    pub fn has_rust_execution_plan(&self) -> bool {
        !matches!(
            self.rust_execution_surface(),
            TaskRustExecutionSurface::None
        )
    }
}

#[path = "catalog_data.rs"]
mod catalog_data;

pub fn task_catalog() -> Vec<TaskCatalogEntry> {
    catalog_data::task_catalog_entries()
}

pub fn find_task_catalog_entry(key: &str) -> Option<TaskCatalogEntry> {
    task_catalog()
        .into_iter()
        .find(|entry| entry.key.eq_ignore_ascii_case(key))
}

pub fn rust_execution_surface_for_task(key: &str) -> TaskRustExecutionSurface {
    match key {
        "AutoPick"
        | "AutoArtifactSalvage"
        | "AutoBoss"
        | "AutoCook"
        | "AutoDomain"
        | "AutoEat"
        | "AutoEatFood"
        | "AutoFish"
        | "AutoFishing"
        | "AutoGeniusInvokation"
        | "AutoLeyLineOutcrop"
        | "AutoMusicGame"
        | "AutoOpenChest"
        | "AutoSkip"
        | "AutoStygianOnslaught"
        | "AutoTrack"
        | "AutoTrackPath"
        | "AutoWood"
        | "GameLoading"
        | "GetGridIcons"
        | "TurnAroundMacro"
        | "QuickEnhanceArtifactMacro"
        | "QuickBuy"
        | "QuickSereniteaPot"
        | "QuickTeleport"
        | "SkillCd"
        | "MapMask"
        | "AutoFight"
        | "AutoPathing"
        | "UseRedeemCode"
        | "ReturnMainUi"
        | "Teleport"
        | "WalkToF"
        | "LowerHeadThenWalkTo"
        | "LinneaMining"
        | "OneKeyExpedition"
        | "ScanPickDrops"
        | "SetTime"
        | "ChooseTalkOption"
        | "CheckRewards"
        | "BlessingOfTheWelkinMoon"
        | "ClaimBattlePassRewards"
        | "ClaimEncounterPointsRewards"
        | "ClaimMailRewards"
        | "CountInventoryItem"
        | "GoToAdventurersGuild"
        | "GoToCraftingBench"
        | "GoToSereniteaPot"
        | "Relogin"
        | "WonderlandCycle"
        | "SwitchParty" => TaskRustExecutionSurface::ExecutionPlanOnly,
        "Shell" => TaskRustExecutionSurface::DirectExecution,
        _ => TaskRustExecutionSurface::None,
    }
}
