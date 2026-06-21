use bgi_core::{
    initial_triggers, read_pathing_task, AssetResolver, GameUiCategory, GenshinAction,
    KeyBindingsConfig, KeyId, PathingExecutionPlan, PathingSummary, ScreenSize, TriggerDescriptor,
};
use bgi_input::{
    input_events_for_action, send_events, send_events_with_cancellation, InputCancellationToken,
    InputError, InputEvent, InputSequence, KeyActionType, MouseButton,
};
use bgi_vision::{
    crop_bgr_image, BgrImage, BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand,
    PureRustVisionBackend, RecognitionObject, Rect, RgbPixel, Size, TemplateMatchMode,
    VisionBackend,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("task runtime is not initialized")]
    RuntimeNotInitialized,
    #[error("task {0} is already running")]
    TaskAlreadyRunning(String),
    #[error("unknown trigger: {0}")]
    UnknownTrigger(String),
    #[error("unknown independent task: {0}")]
    UnknownIndependentTask(String),
    #[error(
        "task catalog entry {key} cannot be launched with policy {actual:?}; expected {expected:?}"
    )]
    InvalidLaunchPolicy {
        key: String,
        expected: TaskLaunchPolicy,
        actual: TaskLaunchPolicy,
    },
    #[error("task invocation kind {actual:?} cannot be applied here; expected {expected:?}")]
    InvalidInvocationKind {
        expected: TaskInvocationKind,
        actual: TaskInvocationKind,
    },
    #[error("dispatcher command requires a task name")]
    MissingTaskName,
    #[error("shell task failed to start: {0}")]
    ShellStart(std::io::Error),
    #[error("shell task IO failed: {0}")]
    ShellIo(std::io::Error),
    #[error("shell task timed out after {timeout_seconds} seconds")]
    ShellTimeout { timeout_seconds: i32 },
    #[error("vision plan failed: {0}")]
    VisionPlan(String),
    #[error("pathing route path is empty")]
    EmptyPathingRoute,
    #[error("pathing route path escapes the User/AutoPathing root: {0}")]
    InvalidPathingRoute(String),
    #[error("pathing plan failed: {0}")]
    PathingPlan(String),
    #[error("combat strategy path is empty")]
    EmptyCombatStrategyPath,
    #[error("combat strategy path escapes the User/AutoFight root: {0}")]
    InvalidCombatStrategyPath(String),
    #[error("combat strategy failed: {0}")]
    CombatStrategy(String),
    #[error("combat input dispatch failed: {0}")]
    CombatInputDispatch(String),
}

pub type Result<T> = std::result::Result<T, TaskError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskRuntimeState {
    Stopped,
    Starting,
    Running,
    Suspended,
    Stopping,
    Faulted,
}

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
}

pub fn task_catalog() -> Vec<TaskCatalogEntry> {
    vec![
        TaskCatalogEntry {
            key: "RecognitionTest",
            display_name: "Recognition Test",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.Placeholder.TestTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/Placeholder/PlaceholderTrigger.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &[],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Developer placeholder trigger loaded first by GameTaskManager.",
        },
        TaskCatalogEntry {
            key: "GameLoading",
            display_name: "Game Loading",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.GameLoading.GameLoadingTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/GameLoading/GameLoading.cs",
            config_section: Some("genshinStartConfig"),
            hotkey_fields: &[],
            asset_roots: &["GameTask/GameLoading/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Enabled from GenshinStartConfig.AutoEnterGameEnabled after dispatcher startup.",
        },
        TaskCatalogEntry {
            key: "AutoPick",
            display_name: "Auto Pick",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.AutoPick.AutoPickTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoPick/AutoPickTrigger.cs",
            config_section: Some("autoPickConfig"),
            hotkey_fields: &["autoPickEnabledHotkey"],
            asset_roots: &["GameTask/AutoPick/Assets", "Assets/Config/Pick"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Rust has config, trigger descriptor, asset resolution, and input boundary; OCR/click execution remains native pending.",
        },
        TaskCatalogEntry {
            key: "AutoEat",
            display_name: "Auto Eat",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.AutoEat.AutoEatTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoEat/AutoEatTrigger.cs",
            config_section: Some("autoEatConfig"),
            hotkey_fields: &[],
            asset_roots: &["GameTask/AutoEat/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::ConfigBound,
            notes: "Needs low-HP/resurrection recognition and key binding action execution.",
        },
        TaskCatalogEntry {
            key: "QuickTeleport",
            display_name: "Quick Teleport",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.QuickTeleport.QuickTeleportTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/QuickTeleport/QuickTeleportTrigger.cs",
            config_section: Some("quickTeleportConfig"),
            hotkey_fields: &["quickTeleportEnabledHotkey", "quickTeleportTickHotkey"],
            asset_roots: &["GameTask/QuickTeleport/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Rust models config and BigMap trigger metadata; template matching/OCR click flow is pending.",
        },
        TaskCatalogEntry {
            key: "AutoSkip",
            display_name: "Auto Skip",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.AutoSkip.AutoSkipTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoSkip/AutoSkipTrigger.cs",
            config_section: Some("autoSkipConfig"),
            hotkey_fields: &["autoSkipEnabledHotkey", "autoSkipHangoutEnabledHotkey"],
            asset_roots: &["GameTask/AutoSkip/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Rust models background-capable dialog trigger metadata and config; dialogue OCR/audio wait/click flow is pending.",
        },
        TaskCatalogEntry {
            key: "AutoFish",
            display_name: "Auto Fishing Trigger",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.AutoFishing.AutoFishingTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoFishing/AutoFishingTrigger.cs",
            config_section: Some("autoFishingConfig"),
            hotkey_fields: &["autoFishingEnabledHotkey", "autoFishingGameHotkey"],
            asset_roots: &["GameTask/AutoFishing/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::ConfigBound,
            notes: "Exclusive fishing UI detection, behavior tree, YOLO, OCR, and input loop still need Rust ports.",
        },
        TaskCatalogEntry {
            key: "SkillCd",
            display_name: "Skill Cooldown",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.SkillCd.SkillCdTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/SkillCd/SkillCdTrigger.cs",
            config_section: Some("skillCdConfig"),
            hotkey_fields: &["skillCdEnabledHotkey"],
            asset_roots: &["GameTask/AutoFight/Assets"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::ConfigBound,
            notes: "Requires party/avatar sync, cooldown OCR, key-state tracking, and overlay rendering.",
        },
        TaskCatalogEntry {
            key: "MapMask",
            display_name: "Map Mask",
            kind: TaskKind::RealtimeTrigger,
            legacy_type: "BetterGenshinImpact.GameTask.MapMask.MapMaskTrigger",
            legacy_reference: "BetterGenshinImpact/GameTask/MapMask/MapMaskTrigger.cs",
            config_section: Some("mapMaskConfig"),
            hotkey_fields: &["mapMaskEnabledHotkey"],
            asset_roots: &["GameTask/MapMask/Assets", "User/AutoPathing"],
            launch_policy: TaskLaunchPolicy::RealtimeTick,
            requires_main_ui_wait: None,
            port_state: TaskPortState::ConfigBound,
            notes: "Needs web map coordinate conversion, mask overlay, and route recording integration.",
        },
        TaskCatalogEntry {
            key: "AutoGeniusInvokation",
            display_name: "Auto Genius Invokation",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoGeniusInvokation.AutoGeniusInvokationTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoGeniusInvokation",
            config_section: Some("autoGeniusInvokationConfig"),
            hotkey_fields: &["autoGeniusInvokationHotkey"],
            asset_roots: &["GameTask/AutoGeniusInvokation/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(false),
            port_state: TaskPortState::ConfigBound,
            notes: "TaskRunner skips main-UI wait for the legacy name 自动七圣召唤.",
        },
        TaskCatalogEntry {
            key: "AutoWood",
            display_name: "Auto Wood",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoWood.AutoWoodTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoWood",
            config_section: Some("autoWoodConfig"),
            hotkey_fields: &["autoWoodHotkey"],
            asset_roots: &["GameTask/AutoWood/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs gadget use, cooldown refresh, OCR, input, and route/camera execution.",
        },
        TaskCatalogEntry {
            key: "AutoFight",
            display_name: "Auto Fight",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoFight.AutoFightTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoFight",
            config_section: Some("autoFightConfig"),
            hotkey_fields: &["autoFightHotkey", "oneKeyFightHotkey"],
            asset_roots: &["GameTask/AutoFight/Assets", "User/AutoFight"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Combat strategy paths, script parser, command/round model, script/configured-team alias normalization, team-context script selection, command filtering, team metadata, ActionSchedulerByCd runtime skip planning, command execution planning, static input expansion, known-team input playback, fight-loop decision planning, finish-detection input/pixel probe planning, and independent-task preparation are ported; visual team recognition and full native fight loop execution are pending.",
        },
        TaskCatalogEntry {
            key: "AutoDomain",
            display_name: "Auto Domain",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoDomain.AutoDomainTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoDomain",
            config_section: Some("autoDomainConfig"),
            hotkey_fields: &["autoDomainHotkey"],
            asset_roots: &["GameTask/AutoDomain/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs domain navigation, combat orchestration, resin usage, reward recognition, and artifact salvage integration.",
        },
        TaskCatalogEntry {
            key: "AutoTrack",
            display_name: "Auto Track",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoSkip.AutoTrackTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoSkip/AutoTrackTask.cs",
            config_section: Some("autoSkipConfig"),
            hotkey_fields: &["autoTrackHotkey"],
            asset_roots: &["GameTask/AutoSkip/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::MetadataOnly,
            notes: "Legacy auto-track is coupled to auto-skip assets and map/game-state recognition.",
        },
        TaskCatalogEntry {
            key: "AutoTrackPath",
            display_name: "Auto Track Path",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoTrackPath.AutoTrackPathTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoTrackPath",
            config_section: Some("tpConfig"),
            hotkey_fields: &["autoTrackPathHotkey"],
            asset_roots: &["GameTask/AutoTrackPath/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Map pathing model is ported; map coordinate recognition and teleport executor are pending.",
        },
        TaskCatalogEntry {
            key: "AutoMusicGame",
            display_name: "Auto Music Game",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoMusicGame.AutoMusicGameTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoMusicGame",
            config_section: Some("autoMusicGameConfig"),
            hotkey_fields: &["autoMusicGameHotkey"],
            asset_roots: &["GameTask/AutoMusicGame/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(false),
            port_state: TaskPortState::MetadataOnly,
            notes: "TaskRunner skips main-UI wait for legacy names containing 自动音游.",
        },
        TaskCatalogEntry {
            key: "AutoPathing",
            display_name: "Auto Pathing",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoPathing.PathExecutor",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoPathing",
            config_section: Some("pathingConditionConfig"),
            hotkey_fields: &["executePathHotkey", "pathRecorderHotkey", "addWaypointHotkey"],
            asset_roots: &["User/AutoPathing", "GameTask/AutoTrackPath/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Route JSON model is ported; action handlers, navigation, suspend/resume, and teleport execution are pending.",
        },
        TaskCatalogEntry {
            key: "AutoBoss",
            display_name: "Auto Boss",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoBoss.AutoBossTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoBoss",
            config_section: Some("autoBossConfig"),
            hotkey_fields: &[],
            asset_roots: &["GameTask/AutoBoss/Assets"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs boss pathing, resurrection, resin supplement, fight, and reward recognition flows.",
        },
        TaskCatalogEntry {
            key: "AutoLeyLineOutcrop",
            display_name: "Auto Ley Line Outcrop",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoLeyLineOutcrop.AutoLeyLineOutcropTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoLeyLineOutcrop",
            config_section: Some("autoLeyLineOutcropConfig"),
            hotkey_fields: &[],
            asset_roots: &["GameTask/AutoLeyLineOutcrop"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs region route planning, handbook navigation, fight config, resin, and reward flow parity.",
        },
        TaskCatalogEntry {
            key: "AutoStygianOnslaught",
            display_name: "Auto Stygian Onslaught",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoStygianOnslaught.AutoStygianOnslaughtTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoStygianOnslaught",
            config_section: Some("autoStygianOnslaughtConfig"),
            hotkey_fields: &[],
            asset_roots: &["GameTask/AutoStygianOnslaught"],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(false),
            port_state: TaskPortState::ConfigBound,
            notes: "TaskRunner skips main-UI wait for legacy names containing 幽境危战.",
        },
        TaskCatalogEntry {
            key: "AutoFishing",
            display_name: "Auto Fishing Task",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoFishing.AutoFishingTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoFishing/AutoFishingTask.cs",
            config_section: Some("autoFishingConfig"),
            hotkey_fields: &["autoFishingGameHotkey"],
            asset_roots: &["GameTask/AutoFishing/Assets"],
            launch_policy: TaskLaunchPolicy::ScriptDispatcher,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::MetadataOnly,
            notes: "Callable from script dispatcher and Genshin host; shares fishing vision/input backend with realtime trigger.",
        },
        TaskCatalogEntry {
            key: "AutoCook",
            display_name: "Auto Cook",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoCook.AutoCookTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoCook",
            config_section: Some("autoCookConfig"),
            hotkey_fields: &["autoCookGameHotkey"],
            asset_roots: &["GameTask/AutoCook"],
            launch_policy: TaskLaunchPolicy::ScriptDispatcher,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs cooking progress recognition and input loop.",
        },
        TaskCatalogEntry {
            key: "AutoArtifactSalvage",
            display_name: "Auto Artifact Salvage",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.AutoArtifactSalvage.AutoArtifactSalvageTask",
            legacy_reference: "BetterGenshinImpact/GameTask/AutoArtifactSalvage",
            config_section: Some("autoArtifactSalvageConfig"),
            hotkey_fields: &["enhanceArtifactHotkey"],
            asset_roots: &["GameTask/AutoArtifactSalvage"],
            launch_policy: TaskLaunchPolicy::HotkeyCommand,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Needs grid screen recognition, artifact stat OCR, JS filter evaluation, and salvage input execution.",
        },
        TaskCatalogEntry {
            key: "UseRedeemCode",
            display_name: "Use Redeem Code",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.UseRedeemCode.UseRedemptionCodeTask",
            legacy_reference: "BetterGenshinImpact/GameTask/UseRedeemCode",
            config_section: Some("autoRedeemCodeConfig"),
            hotkey_fields: &[],
            asset_roots: &[
                "GameTask/UseRedeemCode/Assets",
                "GameTask/Common/Element/Assets",
            ],
            launch_policy: TaskLaunchPolicy::SoloTask,
            requires_main_ui_wait: Some(false),
            port_state: TaskPortState::RuntimeScaffolded,
            notes: "Runs from feed/clipboard flow; Rust now models clipboard extraction and the legacy BV/page/input execution plan, while native OCR/click execution remains pending.",
        },
        TaskCatalogEntry {
            key: "GetGridIcons",
            display_name: "Get Grid Icons",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.GetGridIcons.GetGridIconsTask",
            legacy_reference: "BetterGenshinImpact/GameTask/GetGridIcons",
            config_section: Some("getGridIconsConfig"),
            hotkey_fields: &[],
            asset_roots: &[],
            launch_policy: TaskLaunchPolicy::HotkeyCommand,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::ConfigBound,
            notes: "Developer/grid extraction task; depends on inventory grid recognition.",
        },
        TaskCatalogEntry {
            key: "Shell",
            display_name: "Shell Task",
            kind: TaskKind::Independent,
            legacy_type: "BetterGenshinImpact.GameTask.Shell.ShellTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Shell",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &[],
            launch_policy: TaskLaunchPolicy::ScriptGroupStep,
            requires_main_ui_wait: Some(true),
            port_state: TaskPortState::Ported,
            notes: "Rust executes script-group and desktop independent shell commands through the platform shell, including disable, timeout, fire-and-forget, cancellation, and output capture semantics.",
        },
        TaskCatalogEntry {
            key: "ReturnMainUi",
            display_name: "Return Main UI",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.ReturnMainUiTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/ReturnMainUiTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Shared job used by many solo tasks and script hosts.",
        },
        TaskCatalogEntry {
            key: "Teleport",
            display_name: "Teleport",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Map.TpTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Map/TpTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Map", "GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Used by the Genshin script host for teleport and big-map movement helpers.",
        },
        TaskCatalogEntry {
            key: "SwitchParty",
            display_name: "Switch Party",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.SwitchPartyTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/SwitchPartyTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Required by AutoFight, AutoDomain, pathing, and script host helpers.",
        },
        TaskCatalogEntry {
            key: "BlessingOfTheWelkinMoon",
            display_name: "Blessing of the Welkin Moon",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.BlessingOfTheWelkinMoonTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/BlessingOfTheWelkinMoonTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for claiming Welkin Moon popup rewards.",
        },
        TaskCatalogEntry {
            key: "ChooseTalkOption",
            display_name: "Choose Talk Option",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.ChooseTalkOptionTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/ChooseTalkOptionTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for dialogue option selection.",
        },
        TaskCatalogEntry {
            key: "ClaimBattlePassRewards",
            display_name: "Claim Battle Pass Rewards",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.ClaimBattlePassRewardsTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/ClaimBattlePassRewardsTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for claiming battle pass rewards.",
        },
        TaskCatalogEntry {
            key: "ClaimEncounterPointsRewards",
            display_name: "Claim Encounter Points Rewards",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.ClaimEncounterPointsRewardsTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/ClaimEncounterPointsRewardsTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for claiming encounter points rewards.",
        },
        TaskCatalogEntry {
            key: "GoToAdventurersGuild",
            display_name: "Go To Adventurers Guild",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.GoToAdventurersGuildTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/GoToAdventurersGuildTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for navigating to the Adventurers' Guild.",
        },
        TaskCatalogEntry {
            key: "GoToCraftingBench",
            display_name: "Go To Crafting Bench",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.GoToCraftingBenchTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/GoToCraftingBenchTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for navigating to crafting benches.",
        },
        TaskCatalogEntry {
            key: "Relogin",
            display_name: "Relogin",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.ExitAndReloginJob",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/ExitAndReloginJob.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for exiting and logging back in.",
        },
        TaskCatalogEntry {
            key: "WonderlandCycle",
            display_name: "Wonderland Cycle",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.EnterAndExitWonderlandJob",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/EnterAndExitWonderlandJob.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Script host helper for entering and exiting Wonderland.",
        },
        TaskCatalogEntry {
            key: "SetTime",
            display_name: "Set Game Time",
            kind: TaskKind::System,
            legacy_type: "BetterGenshinImpact.GameTask.Common.Job.SetTimeTask",
            legacy_reference: "BetterGenshinImpact/GameTask/Common/Job/SetTimeTask.cs",
            config_section: None,
            hotkey_fields: &[],
            asset_roots: &["GameTask/Common/Element/Assets"],
            launch_policy: TaskLaunchPolicy::CommonJob,
            requires_main_ui_wait: None,
            port_state: TaskPortState::MetadataOnly,
            notes: "Called by fishing, script host, and common automation jobs.",
        },
    ]
}

pub fn find_task_catalog_entry(key: &str) -> Option<TaskCatalogEntry> {
    task_catalog()
        .into_iter()
        .find(|entry| entry.key.eq_ignore_ascii_case(key))
}

pub const REDEEM_CODE_PATTERN: &str = r"(?<![A-Z0-9])(?=[A-Z0-9]*[A-Z])[A-Z0-9]{12}(?![A-Z0-9])";
pub const USE_REDEEM_CODE_TASK_KEY: &str = "UseRedeemCode";
pub const USE_REDEEM_CODE_ESC_RETURN_BUTTON: &str = "UseRedeemCode:esc_return_button.png";
pub const COMMON_BTN_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const COMMON_BTN_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const VK_ESCAPE: u16 = 0x1B;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemCodeEntry {
    pub code: String,
    pub items: Option<String>,
}

impl RedeemCodeEntry {
    pub fn new(code: impl Into<String>, items: Option<String>) -> Option<Self> {
        let code = code.into().trim().to_string();
        if code.is_empty() {
            return None;
        }
        Some(Self { code, items })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeStepPhase {
    Setup,
    PerCode,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseRedeemCodeStepCondition {
    Always,
    WhenSuccessDetected,
    WhenSuccessNotDetected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum UseRedeemCodeStepAction {
    CommonJob { task_key: String },
    Input { events: Vec<InputEvent> },
    Page { command: BvPageCommand },
    Locator { locator: BvLocatorPlan },
    ClipboardSet { text: String },
    ClipboardClear,
    Log { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseRedeemCodeStep {
    pub phase: UseRedeemCodeStepPhase,
    pub condition: UseRedeemCodeStepCondition,
    pub code: Option<String>,
    pub label: String,
    pub action: UseRedeemCodeStepAction,
}

impl UseRedeemCodeStep {
    fn new(
        phase: UseRedeemCodeStepPhase,
        label: impl Into<String>,
        action: UseRedeemCodeStepAction,
    ) -> Self {
        Self {
            phase,
            condition: UseRedeemCodeStepCondition::Always,
            code: None,
            label: label.into(),
            action,
        }
    }

    fn for_code(code: &str, label: impl Into<String>, action: UseRedeemCodeStepAction) -> Self {
        Self {
            phase: UseRedeemCodeStepPhase::PerCode,
            condition: UseRedeemCodeStepCondition::Always,
            code: Some(code.to_string()),
            label: label.into(),
            action,
        }
    }

    fn conditional_for_code(
        code: &str,
        condition: UseRedeemCodeStepCondition,
        label: impl Into<String>,
        action: UseRedeemCodeStepAction,
    ) -> Self {
        Self {
            phase: UseRedeemCodeStepPhase::PerCode,
            condition,
            code: Some(code.to_string()),
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseRedeemCodeExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub codes: Vec<RedeemCodeEntry>,
    pub steps: Vec<UseRedeemCodeStep>,
    pub notes: String,
}

pub fn extract_redeem_codes_from_text(clipboard_text: &str) -> Vec<String> {
    if clipboard_text.is_empty() {
        return Vec::new();
    }

    let bytes = clipboard_text.as_bytes();
    let mut codes = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !is_redeem_code_char(bytes[index]) {
            index += 1;
            continue;
        }

        let start = index;
        while index < bytes.len() && is_redeem_code_char(bytes[index]) {
            index += 1;
        }
        let end = index;
        if end - start == 12
            && bytes[start..end]
                .iter()
                .any(|byte| byte.is_ascii_uppercase())
            && start_boundary_is_clear(bytes, start)
            && end_boundary_is_clear(bytes, end)
        {
            codes.push(clipboard_text[start..end].to_string());
        }
    }
    codes
}

fn is_redeem_code_char(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte.is_ascii_digit()
}

fn start_boundary_is_clear(bytes: &[u8], start: usize) -> bool {
    start == 0 || !is_redeem_code_char(bytes[start - 1])
}

fn end_boundary_is_clear(bytes: &[u8], end: usize) -> bool {
    end >= bytes.len() || !is_redeem_code_char(bytes[end])
}

pub fn redeem_code_entries_from_strings<'a>(
    codes: impl IntoIterator<Item = &'a str>,
) -> Vec<RedeemCodeEntry> {
    codes
        .into_iter()
        .filter_map(|code| RedeemCodeEntry::new(code, None))
        .collect()
}

pub fn plan_use_redeem_code_strings<'a>(
    codes: impl IntoIterator<Item = &'a str>,
) -> Result<UseRedeemCodeExecutionPlan> {
    plan_use_redeem_codes(
        redeem_code_entries_from_strings(codes),
        Size::new(1920, 1080),
    )
}

pub fn plan_use_redeem_codes(
    codes: impl IntoIterator<Item = RedeemCodeEntry>,
    capture_size: Size,
) -> Result<UseRedeemCodeExecutionPlan> {
    let codes: Vec<RedeemCodeEntry> = codes.into_iter().collect();
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let mut steps = Vec::new();

    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "log redeem codes",
        UseRedeemCodeStepAction::Log {
            message: format!(
                "start use redeem code task with {} non-empty code(s)",
                codes.len()
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "return to main UI before opening settings",
        UseRedeemCodeStepAction::CommonJob {
            task_key: "ReturnMainUi".to_string(),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "press Escape to open menu",
        UseRedeemCodeStepAction::Input {
            events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait for Escape return button",
        UseRedeemCodeStepAction::Locator {
            locator: image_locator(
                &page,
                USE_REDEEM_CODE_ESC_RETURN_BUTTON,
                None,
                0.8,
                false,
                BvLocatorOperation::WaitFor,
                None,
            )?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click settings button",
        UseRedeemCodeStepAction::Page {
            command: page.click_1080p(45.0, 825.0),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait after opening settings",
        UseRedeemCodeStepAction::Page {
            command: task_vision_result(page.wait(1_000))?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click account tab",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(
                &page,
                "账户",
                Some(left_ratio_rect(capture_size, 0.2)?),
                BvLocatorOperation::Click,
                None,
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait after account tab",
        UseRedeemCodeStepAction::Page {
            command: task_vision_result(page.wait(300))?,
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "click go redeem",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(
                &page,
                "前往兑换",
                Some(right_ratio_rect(capture_size, 0.3)?),
                BvLocatorOperation::Click,
                None,
            ),
        },
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Setup,
        "wait for redeem dialog",
        UseRedeemCodeStepAction::Locator {
            locator: text_locator(&page, "兑换奖励", None, BvLocatorOperation::WaitFor, None),
        },
    ));

    for entry in &codes {
        let code = &entry.code;
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "log redeem code",
            UseRedeemCodeStepAction::Log {
                message: match entry.items.as_deref() {
                    Some(items) if !items.trim().is_empty() => format!("{code} - {items}"),
                    _ => code.clone(),
                },
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "set clipboard to redeem code",
            UseRedeemCodeStepAction::ClipboardSet { text: code.clone() },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "click paste",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "粘贴",
                    Some(right_ratio_rect(capture_size, 0.5)?),
                    BvLocatorOperation::Click,
                    None,
                ),
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "click redeem confirm",
            UseRedeemCodeStepAction::Locator {
                locator: image_locator(
                    &page,
                    COMMON_BTN_WHITE_CONFIRM,
                    None,
                    0.8,
                    true,
                    BvLocatorOperation::Click,
                    None,
                )?,
            },
        ));
        steps.push(UseRedeemCodeStep::for_code(
            code,
            "wait for success popup",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "兑换成功",
                    None,
                    BvLocatorOperation::WaitFor,
                    Some(1_000),
                ),
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessDetected,
            "click success confirm",
            UseRedeemCodeStepAction::Locator {
                locator: image_locator(
                    &page,
                    COMMON_BTN_BLACK_CONFIRM,
                    None,
                    0.8,
                    true,
                    BvLocatorOperation::Click,
                    None,
                )?,
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessDetected,
            "wait after success",
            UseRedeemCodeStepAction::Page {
                command: task_vision_result(page.wait(5_100))?,
            },
        ));
        steps.push(UseRedeemCodeStep::conditional_for_code(
            code,
            UseRedeemCodeStepCondition::WhenSuccessNotDetected,
            "click clear after failed redeem",
            UseRedeemCodeStepAction::Locator {
                locator: text_locator(
                    &page,
                    "清除",
                    Some(right_ratio_rect(capture_size, 0.5)?),
                    BvLocatorOperation::Click,
                    None,
                ),
            },
        ));
    }

    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Cleanup,
        "clear clipboard",
        UseRedeemCodeStepAction::ClipboardClear,
    ));
    steps.push(UseRedeemCodeStep::new(
        UseRedeemCodeStepPhase::Cleanup,
        "return to main UI after redeem",
        UseRedeemCodeStepAction::CommonJob {
            task_key: "ReturnMainUi".to_string(),
        },
    ));

    Ok(UseRedeemCodeExecutionPlan {
        task_key: USE_REDEEM_CODE_TASK_KEY.to_string(),
        display_name: "Use Redeem Code".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: false,
        capture_size,
        codes,
        steps,
        notes: "Legacy flow is represented as a BV/page/input/clipboard plan; native OCR/click executor parity remains pending.".to_string(),
    })
}

fn task_vision_result<T>(result: bgi_vision::Result<T>) -> Result<T> {
    result.map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    use_3_channels: bool,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = task_vision_result(BvImage::new(asset))?;
    let mut locator = task_vision_result(page.locator_for_image(&image, roi, threshold))?;
    locator.recognition_object.template.use_3_channels = use_3_channels;
    Ok(locator.plan(operation, timeout_ms))
}

fn text_locator(
    page: &BvPage,
    text: &str,
    roi: Option<Rect>,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> BvLocatorPlan {
    page.locator_for_text(text, roi).plan(operation, timeout_ms)
}

fn left_ratio_rect(size: Size, ratio: f64) -> Result<Rect> {
    let width = ratio_width(size, ratio);
    task_vision_result(Rect::new(0, 0, width, size.height as i32))
}

fn right_ratio_rect(size: Size, ratio: f64) -> Result<Rect> {
    let width = ratio_width(size, ratio);
    task_vision_result(Rect::new(
        size.width as i32 - width,
        0,
        width,
        size.height as i32,
    ))
}

fn ratio_width(size: Size, ratio: f64) -> i32 {
    ((size.width as f64) * ratio).round() as i32
}

const AUTO_STRATEGY_NAME: &str = "根据队伍自动选择";
const DEFAULT_RESIN_PRIORITY: &[&str] = &["浓缩树脂", "原粹树脂"];

fn default_resin_priority() -> Vec<String> {
    DEFAULT_RESIN_PRIORITY
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub fn combat_strategy_path(strategy_name: Option<&str>) -> String {
    let strategy_name = strategy_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(AUTO_STRATEGY_NAME);
    if strategy_name == AUTO_STRATEGY_NAME {
        "User/AutoFight/".to_string()
    } else {
        format!("User/AutoFight/{strategy_name}.txt")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombatCommandMethod {
    Skill,
    Burst,
    Attack,
    Charge,
    Wait,
    Ready,
    Check,
    Walk,
    W,
    A,
    S,
    D,
    Dash,
    Jump,
    MouseDown,
    MouseUp,
    Click,
    MoveBy,
    KeyDown,
    KeyUp,
    KeyPress,
    Scroll,
    Round,
}

impl CombatCommandMethod {
    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim() {
            "skill" | "e" => Some(Self::Skill),
            "burst" | "q" => Some(Self::Burst),
            "attack" | "普攻" | "普通攻击" => Some(Self::Attack),
            "charge" | "重击" => Some(Self::Charge),
            "wait" | "after" | "等待" => Some(Self::Wait),
            "ready" | "完成" => Some(Self::Ready),
            "check" | "检测" => Some(Self::Check),
            "walk" | "行走" => Some(Self::Walk),
            "w" => Some(Self::W),
            "a" => Some(Self::A),
            "s" => Some(Self::S),
            "d" => Some(Self::D),
            "dash" | "冲刺" => Some(Self::Dash),
            "jump" | "j" | "跳跃" => Some(Self::Jump),
            "mousedown" => Some(Self::MouseDown),
            "mouseup" => Some(Self::MouseUp),
            "click" => Some(Self::Click),
            "moveby" => Some(Self::MoveBy),
            "keydown" => Some(Self::KeyDown),
            "keyup" => Some(Self::KeyUp),
            "keypress" => Some(Self::KeyPress),
            "scroll" | "verticalscroll" => Some(Self::Scroll),
            "round" => Some(Self::Round),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandPlan {
    pub avatar: String,
    pub method: CombatCommandMethod,
    pub args: Vec<String>,
    pub activating_rounds: Vec<u32>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlan {
    pub name: String,
    pub path: Option<PathBuf>,
    pub avatar_names: Vec<String>,
    pub commands: Vec<CombatCommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptBagPlan {
    pub source_path: PathBuf,
    pub scripts: Vec<CombatScriptPlan>,
    pub parse_failures: Vec<CombatScriptParseFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptParseFailure {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptMatchPlan {
    pub script_name: String,
    pub script_path: Option<PathBuf>,
    pub matched_avatar_count: usize,
    pub full_match: bool,
    pub commands: Vec<CombatCommandPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatScriptTeamSelectionStatus {
    NoTeamContext,
    FullMatch,
    PartialFallback,
    NoMatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptTeamSelectionPlan {
    pub status: CombatScriptTeamSelectionStatus,
    pub team_avatar_names: Vec<String>,
    pub script_name: Option<String>,
    pub script_path: Option<PathBuf>,
    pub matched_avatar_count: usize,
    pub full_match: bool,
    pub command_avatar_names: Vec<String>,
    pub executable_avatar_names: Vec<String>,
    pub filtered_out_avatar_names: Vec<String>,
    pub executable_commands: Vec<CombatCommandPlan>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamAvatarPlan {
    pub index: usize,
    pub name: String,
    pub id: String,
    pub name_en: String,
    pub weapon: String,
    pub skill_cd_seconds: Option<f64>,
    pub skill_hold_cd_seconds: Option<f64>,
    pub burst_cd_seconds: Option<f64>,
    pub manual_skill_cd_seconds: f64,
    pub action_scheduler_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamPlan {
    pub avatars: Vec<CombatTeamAvatarPlan>,
    pub command_avatar_names: Vec<String>,
    pub can_be_skipped_avatar_names: Vec<String>,
    pub all_command_avatars_can_be_skipped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatFightLoopStepPhase {
    Setup,
    LoopStart,
    BeforeCommand,
    Command,
    AfterCommand,
    FinishDetection,
    Cleanup,
    Loot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatFightLoopStepKind {
    InitializeCancellation,
    StartExperienceDetector,
    WaitAllConfiguredSkillCooldowns,
    EnsureGuardianSkill,
    InitialSeekEnemy,
    SkipGuardianCommand,
    SkipCommandBySkillCooldown,
    EnforceTimeout,
    CheckBeforeBurst,
    ExecuteCommand,
    CountFightAvatarSwitch,
    CheckCommandFinish,
    FastCheckAfterAvatar,
    ReleaseAllKeys,
    StopExperienceDetector,
    ApplyBattleThresholdForLoot,
    KazuhaOrJeanPickup,
    SwitchPickupParty,
    ScanPickDrops,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatFightLoopStepPlan {
    pub phase: CombatFightLoopStepPhase,
    pub kind: CombatFightLoopStepKind,
    pub command_index: Option<usize>,
    pub avatar: Option<String>,
    pub enabled: bool,
    pub requires_native_context: Vec<CombatExecutionContextRequirement>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatFightLoopPlan {
    pub timeout_seconds: i32,
    pub command_count: usize,
    pub executable_command_count: usize,
    pub fight_finish_detect_enabled: bool,
    pub rotate_find_enemy_enabled: bool,
    pub check_before_burst_enabled: bool,
    pub guardian_enabled: bool,
    pub guardian_avatar_index: Option<usize>,
    pub guardian_avatar_name: Option<String>,
    pub kazuha_pickup_enabled: bool,
    pub pickup_drops_after_fight_enabled: bool,
    pub exp_based_pickup_enabled: bool,
    pub battle_threshold_for_loot: i32,
    pub steps: Vec<CombatFightLoopStepPlan>,
    pub native_dispatch_ready: bool,
}

pub const AUTO_FIGHT_FINISH_PROGRESS_PIXEL: (u32, u32) = (790, 50);
pub const AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL: (u32, u32) = (768, 50);
pub const AUTO_FIGHT_DEFAULT_FINISH_DELAY_MS: u64 = 1_500;
pub const AUTO_FIGHT_DEFAULT_FINISH_DETECT_DELAY_MS: u64 = 450;
pub const AUTO_FIGHT_AVATAR_INDEX_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1859, 256, 28, 24),
    (1859, 352, 28, 24),
    (1859, 448, 28, 24),
    (1859, 544, 28, 24),
];
pub const AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS: [&str; 4] =
    ["index_1.png", "index_2.png", "index_3.png", "index_4.png"];
pub const AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ROI_1080P: (i32, i32, i32, i32) = (1855, 155, 35, 600);
pub const AUTO_FIGHT_AVATAR_INDEX_DISTANCE_Y_1080P: i32 = 96;
pub const AUTO_FIGHT_AVATAR_SIDE_ICON_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1765, 225, 76, 76),
    (1765, 315, 76, 76),
    (1765, 410, 76, 76),
    (1765, 500, 76, 76),
];
pub const AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1584, 216, 64, 84),
    (1584, 316, 64, 84),
    (1584, 416, 64, 84),
    (1584, 516, 64, 84),
];
pub const AUTO_FIGHT_AVATAR_SIDE_ICON_FROM_INDEX_RECT_1080P: (i32, i32, i32, i32) =
    (-91, -47, 82, 82);
pub const AUTO_FIGHT_COOP_ONE_P_ASSET: &str = "1p.png";
pub const AUTO_FIGHT_COOP_P_ASSET: &str = "p.png";
pub const AUTO_FIGHT_FEATURE: &str = "AutoFight";
pub const AUTO_FIGHT_COOP_SIDE_INDEX_RECTS_1080P: [(&str, &[(i32, i32, i32, i32)]); 6] = [
    ("1p_2", &[(1859, 412, 28, 24), (1859, 508, 28, 24)]),
    ("1p_3", &[(1859, 459, 28, 24), (1859, 555, 28, 24)]),
    ("1p_4", &[(1859, 552, 28, 24)]),
    ("p_2", &[(1859, 412, 28, 24), (1859, 508, 28, 24)]),
    ("p_3", &[(1859, 412, 28, 24)]),
    ("p_4", &[(1859, 507, 28, 24)]),
];
pub const AUTO_FIGHT_COOP_SIDE_ICON_RECTS_1080P: [(&str, &[(i32, i32, i32, i32)]); 6] = [
    ("1p_2", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("1p_3", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("1p_4", &[(1765, 515, 76, 76)]),
    ("p_2", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("p_3", &[(1765, 475, 76, 76)]),
    ("p_4", &[(1765, 515, 76, 76)]),
];
pub const AUTO_FIGHT_CURRENT_AVATAR_FEATURE: &str = "Common/Element";
pub const AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET: &str = "current_avatar_threshold.png";
pub const AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ROI_1080P: (i32, i32, i32, i32) =
    (1680, 155, 210, 600);
pub const AUTO_FIGHT_CURRENT_AVATAR_FLAG_TO_INDEX_RECT_1080P: (i32, i32, i32, i32) =
    (126, -194, 16, 17);
pub const AUTO_FIGHT_ACTIVE_SKILL_COOLDOWN_RECT_1080P: (i32, i32, i32, i32) = (1688, 988, 22, 12);
pub const AUTO_FIGHT_ACTIVE_BURST_COOLDOWN_RECT_1080P: (i32, i32, i32, i32) = (1809, 968, 30, 15);
pub const AUTO_FIGHT_SIDE_BURST_MIN_RADIUS_1080P: i32 = 25;
pub const AUTO_FIGHT_SIDE_BURST_MAX_RADIUS_1080P: i32 = 34;
pub const AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES: usize = 25;
pub const AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionResult {
    pub finished: bool,
    pub progress_pixel: RgbPixel,
    pub white_tile_pixel: RgbPixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFightFinishDetectionStepKind {
    PreDetectDelay,
    SeekEnemy,
    OpenPartySetup,
    WaitForPartySetup,
    CaptureFrame,
    SampleFinishPixels,
    DropFromPartySetup,
    CancelPartySwitchWhenFinished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionStepPlan {
    pub kind: AutoFightFinishDetectionStepKind,
    pub enabled: bool,
    pub input_events: Vec<InputEvent>,
    pub delay_ms: u64,
    pub requires_capture: bool,
    pub requires_vision: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionPlan {
    pub pre_detect_delay_ms: u64,
    pub detect_delay_ms: u64,
    pub rotate_find_enemy_enabled: bool,
    pub progress_pixel: (u32, u32),
    pub white_tile_pixel: (u32, u32),
    pub steps: Vec<AutoFightFinishDetectionStepPlan>,
    pub native_ready_without_capture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFightFinishDetectionExecutionMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionExecution {
    pub mode: AutoFightFinishDetectionExecutionMode,
    pub plan: AutoFightFinishDetectionPlan,
    pub detection: AutoFightFinishDetectionResult,
    pub before_capture_events: Vec<InputEvent>,
    pub after_detection_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionLiveExecution {
    pub mode: AutoFightFinishDetectionExecutionMode,
    pub plan: AutoFightFinishDetectionPlan,
    pub detection: Option<AutoFightFinishDetectionResult>,
    pub before_capture_events: Vec<InputEvent>,
    pub after_detection_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
    pub captured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatActiveAvatarDetectionMethod {
    SingleAvatar,
    WhiteRectMajority,
    EdgeWhiteRatio,
    ImageDifferenceVote,
    ArrowTemplate,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatActiveAvatarDetectionResult {
    pub active_index: Option<usize>,
    pub method: CombatActiveAvatarDetectionMethod,
    pub rects: Vec<Rect>,
    pub white_rect_count: usize,
    pub not_white_rect_index: Option<usize>,
    pub edge_white_ratios: Vec<f64>,
    pub difference_votes: Vec<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillReadinessKind {
    ElementalSkill,
    ElementalBurst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillReadinessStatus {
    Ready,
    CooldownOrUnavailable,
    UnsupportedForInactiveAvatar,
    ActiveAvatarUnresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatSideBurstCircleDetection {
    pub rect: Rect,
    pub detected: bool,
    pub edge_pixel_count: usize,
    pub best_center: Option<(i32, i32)>,
    pub best_radius: Option<i32>,
    pub best_votes: usize,
    pub required_votes: usize,
    pub sampled_points: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatSkillReadinessDetection {
    pub kind: CombatSkillReadinessKind,
    pub requested_index: usize,
    pub active_index: Option<usize>,
    pub status: CombatSkillReadinessStatus,
    pub ready: Option<bool>,
    pub active_detection: CombatActiveAvatarDetectionResult,
    pub cooldown_rect: Option<Rect>,
    pub white_component_count: usize,
    pub legacy_connected_component_labels: usize,
    pub side_burst_rect: Option<Rect>,
    pub side_burst_circle: Option<CombatSideBurstCircleDetection>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarIndexRectsDetection {
    pub rects_by_index: Vec<Option<Rect>>,
    pub resolved_rects: Vec<Rect>,
    pub inferred_from_current_avatar_arrow: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMultiGameStatus {
    pub is_in_multi_game: bool,
    pub is_host: bool,
    pub player_count: usize,
}

impl Default for CombatMultiGameStatus {
    fn default() -> Self {
        Self {
            is_in_multi_game: false,
            is_host: false,
            player_count: 1,
        }
    }
}

impl CombatMultiGameStatus {
    pub fn max_control_avatar_count(self) -> Result<usize> {
        if !self.is_in_multi_game {
            return Ok(4);
        }
        match (self.is_host, self.player_count) {
            (true, 1) => Ok(4),
            (true, 2 | 3) => Ok(2),
            (true, 4) => Ok(1),
            (false, 2) => Ok(2),
            (false, 3 | 4) => Ok(1),
            (true, _) => Err(TaskError::VisionPlan(format!(
                "invalid host co-op player count: {}",
                self.player_count
            ))),
            (false, _) => Err(TaskError::VisionPlan(format!(
                "invalid guest co-op player count: {}",
                self.player_count
            ))),
        }
    }

    pub fn rect_map_key(self) -> Option<String> {
        self.is_in_multi_game.then(|| {
            format!(
                "{}_{}",
                if self.is_host { "1p" } else { "p" },
                self.player_count
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMultiGameDetection {
    pub status: CombatMultiGameStatus,
    pub p_icon_count: usize,
    pub one_p_icon_found: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarSideClassification {
    pub class_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarSideRecognition {
    pub index: usize,
    pub avatar_name: String,
    pub name_en: String,
    pub costume_name: Option<String>,
    pub display_name: String,
    pub confidence: f32,
    pub index_rect: Rect,
    pub side_icon_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamRecognitionExecution {
    pub index_rect_detection: CombatAvatarIndexRectsDetection,
    pub avatars: Vec<CombatAvatarSideRecognition>,
    pub team_avatar_names: Vec<String>,
    pub team_plan: CombatTeamPlan,
}

pub trait CombatAvatarSideClassifier {
    fn classify_avatar_side(
        &mut self,
        index: usize,
        image: &BgrImage,
        side_icon_rect: Rect,
    ) -> Result<CombatAvatarSideClassification>;
}

impl<F> CombatAvatarSideClassifier for F
where
    F: FnMut(usize, &BgrImage, Rect) -> Result<CombatAvatarSideClassification>,
{
    fn classify_avatar_side(
        &mut self,
        index: usize,
        image: &BgrImage,
        side_icon_rect: Rect,
    ) -> Result<CombatAvatarSideClassification> {
        self(index, image, side_icon_rect)
    }
}

pub const COMBAT_AVATAR_CATALOG_PATH: &str = "GameTask/AutoFight/Assets/combat_avatar.json";
pub const LEGACY_COMBAT_AVATAR_CATALOG_PATH: &str =
    "BetterGenshinImpact/GameTask/AutoFight/Assets/combat_avatar.json";
pub const EXPECTED_COMBAT_TEAM_AVATAR_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarMetadata {
    #[serde(default)]
    pub alias: Vec<String>,
    pub id: String,
    pub name: String,
    #[serde(rename = "nameEn")]
    pub name_en: String,
    pub weapon: String,
    #[serde(default, rename = "skillCD")]
    pub skill_cd: Option<f64>,
    #[serde(default, rename = "skillHoldCD")]
    pub skill_hold_cd: Option<f64>,
    #[serde(default, rename = "burstCD")]
    pub burst_cd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CombatAvatarCatalog {
    pub source_path: PathBuf,
    pub avatars: Vec<CombatAvatarMetadata>,
    pub alias_to_name: BTreeMap<String, String>,
}

impl CombatAvatarCatalog {
    pub fn new(source_path: PathBuf, avatars: Vec<CombatAvatarMetadata>) -> Result<Self> {
        let mut alias_to_name = BTreeMap::new();
        for avatar in &avatars {
            for alias in &avatar.alias {
                let alias = alias.trim();
                if alias.is_empty() {
                    continue;
                }
                if let Some(existing) = alias_to_name.insert(alias.to_string(), avatar.name.clone())
                {
                    return Err(TaskError::CombatStrategy(format!(
                        "duplicate combat avatar alias: {alias} maps to both {existing} and {}",
                        avatar.name
                    )));
                }
            }
        }
        Ok(Self {
            source_path,
            avatars,
            alias_to_name,
        })
    }

    pub fn standard_name_for_alias(&self, alias: &str) -> Result<String> {
        let alias = alias.trim();
        self.alias_to_name
            .get(alias)
            .cloned()
            .ok_or_else(|| TaskError::CombatStrategy(format!("角色名称校验失败：{alias}")))
    }

    pub fn avatar_by_name(&self, name: &str) -> Option<&CombatAvatarMetadata> {
        self.avatars.iter().find(|avatar| avatar.name == name)
    }

    pub fn avatar_by_name_en(&self, name_en: &str) -> Option<&CombatAvatarMetadata> {
        self.avatars.iter().find(|avatar| avatar.name_en == name_en)
    }
}

pub fn resolve_combat_avatar_catalog_path(working_directory: impl AsRef<Path>) -> PathBuf {
    let root = working_directory.as_ref();
    [
        root.join(COMBAT_AVATAR_CATALOG_PATH),
        root.join(LEGACY_COMBAT_AVATAR_CATALOG_PATH),
    ]
    .into_iter()
    .find(|path| path.exists())
    .unwrap_or_else(|| root.join(COMBAT_AVATAR_CATALOG_PATH))
}

pub fn read_combat_avatar_catalog(
    working_directory: impl AsRef<Path>,
) -> Result<CombatAvatarCatalog> {
    let path = resolve_combat_avatar_catalog_path(working_directory);
    let json = fs::read_to_string(&path).map_err(|error| {
        TaskError::CombatStrategy(format!(
            "combat avatar catalog cannot be read: {} ({error})",
            path.display()
        ))
    })?;
    let avatars: Vec<CombatAvatarMetadata> = serde_json::from_str(&json).map_err(|error| {
        TaskError::CombatStrategy(format!(
            "combat avatar catalog failed to parse: {} ({error})",
            path.display()
        ))
    })?;
    CombatAvatarCatalog::new(path, avatars)
}

pub const CURRENT_COMBAT_AVATAR_NAME: &str = "当前角色";
pub const COMBAT_ATTACK_INTERVAL_MILLISECONDS: u64 = 200;
pub const COMBAT_DEFAULT_DASH_MILLISECONDS: u64 = 200;
pub const COMBAT_DEFAULT_CHARGE_MILLISECONDS: u64 = 1_000;
pub const COMBAT_READY_INITIAL_DELAY_MILLISECONDS: u64 = 10;
pub const COMBAT_READY_POLL_COUNT: u32 = 20;
pub const COMBAT_READY_POLL_INTERVAL_MILLISECONDS: u64 = 150;
pub const COMBAT_AVATAR_SWITCH_SETTLE_MILLISECONDS: u64 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatAvatarSwitchPolicy {
    CurrentAvatar,
    SwitchOnAvatarChange,
    EnsureSelectedBeforeAction,
    NoSwitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillCooldownPolicy {
    None,
    WaitUntilReady,
    FastSkipIfCoolingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillExecutionVariant {
    Tap,
    GenericHold,
    NahidaCameraSweepHold,
    CandaceLongHold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatChargeExecutionVariant {
    GenericHold,
    NeuvilletteCameraSweep,
    ChascaCameraSweep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMouseButtonPlan {
    pub raw: String,
    pub button: MouseButton,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatVirtualKeyPlan {
    pub raw: String,
    pub vk: u16,
    pub mouse_button: Option<MouseButton>,
    pub mapped_action: Option<GenshinAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CombatCommandActionPlan {
    Skill {
        hold: bool,
        variant: CombatSkillExecutionVariant,
        cooldown_policy: CombatSkillCooldownPolicy,
        options: Vec<String>,
    },
    Burst {
        requires_readiness_check: bool,
    },
    Attack {
        duration_ms: u64,
        click_interval_ms: u64,
        repeat_count: u32,
    },
    Charge {
        duration_ms: u64,
        variant: CombatChargeExecutionVariant,
    },
    Walk {
        direction: String,
        duration_ms: u64,
    },
    Wait {
        duration_ms: u64,
    },
    Ready {
        initial_delay_ms: u64,
        poll_count: u32,
        poll_interval_ms: u64,
    },
    Check {
        handled_by_fight_loop: bool,
    },
    Dash {
        duration_ms: u64,
    },
    Jump,
    MouseDown {
        button: CombatMouseButtonPlan,
    },
    MouseUp {
        button: CombatMouseButtonPlan,
    },
    Click {
        button: CombatMouseButtonPlan,
    },
    MoveBy {
        x: i32,
        y: i32,
    },
    KeyDown {
        key: CombatVirtualKeyPlan,
    },
    KeyUp {
        key: CombatVirtualKeyPlan,
    },
    KeyPress {
        key: CombatVirtualKeyPlan,
    },
    Scroll {
        clicks: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandExecutionPlan {
    pub index: usize,
    pub command: CombatCommandPlan,
    pub switch_policy: CombatAvatarSwitchPolicy,
    pub action: CombatCommandActionPlan,
    pub default_input_events: Vec<InputEvent>,
    pub requires_combat_context: bool,
    pub static_input_ready: bool,
    pub pending_context: Vec<CombatExecutionContextRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptExecutionPlan {
    pub name: String,
    pub path: Option<PathBuf>,
    pub avatar_names: Vec<String>,
    pub commands: Vec<CombatCommandExecutionPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombatExecutionContextRequirement {
    AvatarSelection,
    SkillCooldown,
    BurstReadiness,
    FightLoopFinishDetection,
    ReadyStateDetection,
    CharacterSpecificCameraControl,
    InputEvents,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlaybackEvaluation {
    pub script_name: String,
    pub script_path: Option<PathBuf>,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub default_input_event_count: usize,
    pub first_blocking_command_index: Option<usize>,
    pub first_blocking_requirements: Vec<CombatExecutionContextRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlaybackBatchEvaluation {
    pub scripts: Vec<CombatScriptPlaybackEvaluation>,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub default_input_event_count: usize,
    pub dispatch_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatCommandPlaybackMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandPlaybackExecution {
    pub mode: CombatCommandPlaybackMode,
    pub script_name: String,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub input_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTeamPlaybackCommandPlan {
    pub command_index: usize,
    pub avatar: String,
    pub team_index: Option<usize>,
    pub switch_events: Vec<InputEvent>,
    pub action_events: Vec<InputEvent>,
    pub input_events: Vec<InputEvent>,
    pub resolved_context: Vec<CombatExecutionContextRequirement>,
    pub pending_context: Vec<CombatExecutionContextRequirement>,
    pub executable: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTeamPlaybackExecution {
    pub mode: CombatCommandPlaybackMode,
    pub script_name: String,
    pub total_commands: usize,
    pub candidate_commands: usize,
    pub planned_commands: Vec<CombatTeamPlaybackCommandPlan>,
    pub playable_commands: usize,
    pub blocked_command_index: Option<usize>,
    pub blocked_requirements: Vec<CombatExecutionContextRequirement>,
    pub input_events: Vec<InputEvent>,
    pub dispatch_ready: bool,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}

pub fn plan_combat_script_execution(
    script: &CombatScriptPlan,
) -> Result<CombatScriptExecutionPlan> {
    let mut commands = Vec::with_capacity(script.commands.len());
    let mut previous = None;
    for (index, command) in script.commands.iter().enumerate() {
        let plan = plan_combat_command_execution(index, command, previous)?;
        previous = Some(command);
        commands.push(plan);
    }
    Ok(CombatScriptExecutionPlan {
        name: script.name.clone(),
        path: script.path.clone(),
        avatar_names: script.avatar_names.clone(),
        commands,
    })
}

pub fn plan_combat_command_execution(
    index: usize,
    command: &CombatCommandPlan,
    previous: Option<&CombatCommandPlan>,
) -> Result<CombatCommandExecutionPlan> {
    let switch_policy = combat_avatar_switch_policy(command, previous);
    let action = combat_command_action_plan(command)?;
    let default_input_events = default_input_events_for_combat_action(&action)?;
    let pending_context = combat_command_context_requirements(
        switch_policy,
        &action,
        default_input_events.is_empty(),
    );
    let requires_combat_context = !pending_context.is_empty();
    let static_input_ready = !requires_combat_context && !default_input_events.is_empty();
    Ok(CombatCommandExecutionPlan {
        index,
        command: command.clone(),
        switch_policy,
        action,
        default_input_events,
        requires_combat_context,
        static_input_ready,
        pending_context,
    })
}

pub fn plan_combat_script_bag_execution(
    bag: &CombatScriptBagPlan,
) -> Result<Vec<CombatScriptExecutionPlan>> {
    bag.scripts
        .iter()
        .map(plan_combat_script_execution)
        .collect()
}

pub fn evaluate_combat_script_playback(
    script: &CombatScriptExecutionPlan,
) -> CombatScriptPlaybackEvaluation {
    let static_ready_commands = script
        .commands
        .iter()
        .filter(|command| command.static_input_ready)
        .count();
    let context_bound_commands = script
        .commands
        .iter()
        .filter(|command| command.requires_combat_context)
        .count();
    let default_input_event_count = script
        .commands
        .iter()
        .map(|command| command.default_input_events.len())
        .sum();
    let first_blocking_command = script
        .commands
        .iter()
        .find(|command| command.requires_combat_context);
    CombatScriptPlaybackEvaluation {
        script_name: script.name.clone(),
        script_path: script.path.clone(),
        total_commands: script.commands.len(),
        static_ready_commands,
        context_bound_commands,
        default_input_event_count,
        first_blocking_command_index: first_blocking_command.map(|command| command.index),
        first_blocking_requirements: first_blocking_command
            .map(|command| command.pending_context.clone())
            .unwrap_or_default(),
    }
}

pub fn evaluate_combat_script_batch_playback(
    scripts: &[CombatScriptExecutionPlan],
) -> CombatScriptPlaybackBatchEvaluation {
    let scripts: Vec<_> = scripts
        .iter()
        .map(evaluate_combat_script_playback)
        .collect();
    let total_commands = scripts.iter().map(|script| script.total_commands).sum();
    let static_ready_commands = scripts
        .iter()
        .map(|script| script.static_ready_commands)
        .sum();
    let context_bound_commands = scripts
        .iter()
        .map(|script| script.context_bound_commands)
        .sum();
    let default_input_event_count = scripts
        .iter()
        .map(|script| script.default_input_event_count)
        .sum();
    let dispatch_ready = total_commands > 0 && context_bound_commands == 0;
    CombatScriptPlaybackBatchEvaluation {
        scripts,
        total_commands,
        static_ready_commands,
        context_bound_commands,
        default_input_event_count,
        dispatch_ready,
    }
}

pub fn plan_combat_fight_loop(
    param: &AutoFightParam,
    team_selection: &CombatScriptTeamSelectionPlan,
    team_plan: Option<&CombatTeamPlan>,
    selected_script_execution: Option<&CombatScriptExecutionPlan>,
) -> CombatFightLoopPlan {
    let mut steps = Vec::new();
    let command_count = selected_script_execution
        .map(|script| script.commands.len())
        .unwrap_or(0);
    let executable_command_count = team_selection.executable_commands.len();
    let guardian_avatar_index = parse_guardian_avatar_index(&param.guardian_avatar);
    let guardian_avatar_name = guardian_avatar_index.and_then(|index| {
        team_plan
            .and_then(|team| team.avatars.iter().find(|avatar| avatar.index == index))
            .map(|avatar| avatar.name.clone())
    });
    let guardian_enabled = guardian_avatar_index.is_some();
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Setup,
        CombatFightLoopStepKind::InitializeCancellation,
        None,
        None,
        true,
        Vec::new(),
        "create linked cancellation token and initialize combat avatar runtime context",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Setup,
        CombatFightLoopStepKind::StartExperienceDetector,
        None,
        None,
        param.kazuha_pickup_enabled && param.exp_based_pickup_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "start background elite-experience detector before command loop",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::LoopStart,
        CombatFightLoopStepKind::WaitAllConfiguredSkillCooldowns,
        None,
        None,
        team_plan
            .map(|team| team.all_command_avatars_can_be_skipped)
            .unwrap_or(false),
        vec![CombatExecutionContextRequirement::SkillCooldown],
        "when all command avatars are configured for CD skip, wait for the minimum remaining skill cooldown",
    );
    let commands = selected_script_execution
        .map(|script| script.commands.as_slice())
        .unwrap_or(&[]);
    for command in commands {
        let avatar = Some(command.command.avatar.clone());
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::EnsureGuardianSkill,
            Some(command.index),
            avatar.clone(),
            guardian_enabled,
            vec![
                CombatExecutionContextRequirement::AvatarSelection,
                CombatExecutionContextRequirement::SkillCooldown,
                CombatExecutionContextRequirement::InputEvents,
            ],
            "ensure configured guardian avatar skill before switching to a different combat avatar",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::InitialSeekEnemy,
            Some(command.index),
            avatar.clone(),
            command.index == 0
                && param.finish_detect_config.rotate_find_enemy_enabled
                && param.is_first_check,
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run initial seek-and-fight before the first command when rotate-find-enemy is enabled",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::SkipGuardianCommand,
            Some(command.index),
            avatar.clone(),
            guardian_avatar_name
                .as_ref()
                .map(|guardian| guardian == &command.command.avatar)
                .unwrap_or(false)
                && (param.guardian_combat_skip || param.burst_enabled),
            Vec::new(),
            "skip commands owned by the guardian avatar when guardian combat skip or burst mode is enabled",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::SkipCommandBySkillCooldown,
            Some(command.index),
            avatar.clone(),
            team_plan
                .map(|team| {
                    team.all_command_avatars_can_be_skipped
                        || team
                            .can_be_skipped_avatar_names
                            .iter()
                            .any(|name| name == &command.command.avatar)
                })
                .unwrap_or(false),
            vec![CombatExecutionContextRequirement::SkillCooldown],
            "skip this command when the selected avatar skill cooldown is still active",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::EnforceTimeout,
            Some(command.index),
            avatar.clone(),
            true,
            Vec::new(),
            "stop combat when timeout elapses or seek rotation reaches the legacy cap",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::CheckBeforeBurst,
            Some(command.index),
            avatar.clone(),
            param.finish_detect_config.rotate_find_enemy_enabled
                && param.check_before_burst
                && combat_command_may_trigger_burst(&command.command),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "check fight finish before burst-like commands when configured",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::Command,
            CombatFightLoopStepKind::ExecuteCommand,
            Some(command.index),
            avatar.clone(),
            team_selection
                .executable_commands
                .iter()
                .any(|executable| executable == &command.command),
            command.pending_context.clone(),
            "execute the planned combat command through the native avatar/input/fight context",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::AfterCommand,
            CombatFightLoopStepKind::CountFightAvatarSwitch,
            Some(command.index),
            avatar.clone(),
            true,
            Vec::new(),
            "increment fight-count when this command ends an avatar command group",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::FinishDetection,
            CombatFightLoopStepKind::CheckCommandFinish,
            Some(command.index),
            avatar.clone(),
            param.fight_finish_detect_enabled
                && matches!(
                    command.action,
                    CombatCommandActionPlan::Check {
                        handled_by_fight_loop: true
                    }
                ),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run explicit finish detection for check commands",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::FinishDetection,
            CombatFightLoopStepKind::FastCheckAfterAvatar,
            Some(command.index),
            avatar,
            param.fight_finish_detect_enabled
                && (is_last_command_group(command.index, commands)
                    || param.finish_detect_config.fast_check_enabled),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run end-of-script or fast avatar-bound finish detection",
        );
    }
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Cleanup,
        CombatFightLoopStepKind::ReleaseAllKeys,
        None,
        None,
        true,
        vec![CombatExecutionContextRequirement::InputEvents],
        "release all pressed keys when the fight loop exits",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Cleanup,
        CombatFightLoopStepKind::StopExperienceDetector,
        None,
        None,
        param.kazuha_pickup_enabled && param.exp_based_pickup_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "stop and dispose the background experience detector",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::ApplyBattleThresholdForLoot,
        None,
        None,
        param.battle_threshold_for_loot >= 2,
        Vec::new(),
        "skip loot pickup when fight-count is below the configured threshold",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::KazuhaOrJeanPickup,
        None,
        None,
        param.kazuha_pickup_enabled,
        vec![
            CombatExecutionContextRequirement::AvatarSelection,
            CombatExecutionContextRequirement::SkillCooldown,
            CombatExecutionContextRequirement::InputEvents,
        ],
        "perform Kazuha long-skill or Jean pickup when a picker avatar is available",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::SwitchPickupParty,
        None,
        None,
        param.kazuha_pickup_enabled && !param.kazuha_party_name.trim().is_empty(),
        vec![
            CombatExecutionContextRequirement::AvatarSelection,
            CombatExecutionContextRequirement::FightLoopFinishDetection,
        ],
        "switch to the configured pickup party when the current team lacks a picker",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::ScanPickDrops,
        None,
        None,
        param.pick_drops_after_fight_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "scan and approach loot beams after combat",
    );
    let native_dispatch_ready = !steps.iter().any(|step| {
        step.enabled
            && !step.requires_native_context.is_empty()
            && matches!(
                step.phase,
                CombatFightLoopStepPhase::Command
                    | CombatFightLoopStepPhase::FinishDetection
                    | CombatFightLoopStepPhase::Loot
                    | CombatFightLoopStepPhase::Cleanup
            )
    });
    CombatFightLoopPlan {
        timeout_seconds: param.timeout,
        command_count,
        executable_command_count,
        fight_finish_detect_enabled: param.fight_finish_detect_enabled,
        rotate_find_enemy_enabled: param.finish_detect_config.rotate_find_enemy_enabled,
        check_before_burst_enabled: param.check_before_burst,
        guardian_enabled,
        guardian_avatar_index,
        guardian_avatar_name,
        kazuha_pickup_enabled: param.kazuha_pickup_enabled,
        pickup_drops_after_fight_enabled: param.pick_drops_after_fight_enabled,
        exp_based_pickup_enabled: param.exp_based_pickup_enabled,
        battle_threshold_for_loot: param.battle_threshold_for_loot,
        steps,
        native_dispatch_ready,
    }
}

fn push_fight_loop_step(
    steps: &mut Vec<CombatFightLoopStepPlan>,
    phase: CombatFightLoopStepPhase,
    kind: CombatFightLoopStepKind,
    command_index: Option<usize>,
    avatar: Option<String>,
    enabled: bool,
    requires_native_context: Vec<CombatExecutionContextRequirement>,
    message: &str,
) {
    steps.push(CombatFightLoopStepPlan {
        phase,
        kind,
        command_index,
        avatar,
        enabled,
        requires_native_context,
        message: message.to_string(),
    });
}

fn parse_guardian_avatar_index(value: &str) -> Option<usize> {
    let index = value.trim().parse::<usize>().ok()?;
    (1..=EXPECTED_COMBAT_TEAM_AVATAR_COUNT)
        .contains(&index)
        .then_some(index)
}

fn combat_command_may_trigger_burst(command: &CombatCommandPlan) -> bool {
    command.method == CombatCommandMethod::Burst
        || command.args.iter().any(|arg| arg.eq_ignore_ascii_case("q"))
}

fn is_last_command_group(index: usize, commands: &[CombatCommandExecutionPlan]) -> bool {
    let Some(command) = commands.get(index) else {
        return false;
    };
    commands
        .get(index + 1)
        .map(|next| next.command.avatar != command.command.avatar)
        .unwrap_or(true)
}

pub fn detect_auto_fight_finished_from_image(
    image: &BgrImage,
) -> Result<AutoFightFinishDetectionResult> {
    let progress_pixel = image
        .rgb_pixel_at(
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL.0,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL.1,
        )
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "auto-fight finish progress pixel is outside capture: {:?}",
                AUTO_FIGHT_FINISH_PROGRESS_PIXEL
            ))
        })?;
    let white_tile_pixel = image
        .rgb_pixel_at(
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL.0,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL.1,
        )
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "auto-fight finish white tile pixel is outside capture: {:?}",
                AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL
            ))
        })?;
    Ok(AutoFightFinishDetectionResult {
        finished: is_auto_fight_finish_white(white_tile_pixel)
            && is_auto_fight_finish_yellow(progress_pixel),
        progress_pixel,
        white_tile_pixel,
    })
}

pub fn is_auto_fight_finish_yellow(pixel: RgbPixel) -> bool {
    (200..=255).contains(&pixel.r) && (200..=255).contains(&pixel.g) && (0..=100).contains(&pixel.b)
}

pub fn is_auto_fight_finish_white(pixel: RgbPixel) -> bool {
    (240..=255).contains(&pixel.r)
        && (240..=255).contains(&pixel.g)
        && (240..=255).contains(&pixel.b)
}

pub fn default_combat_avatar_index_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_INDEX_RECTS_1080P)
}

pub fn default_combat_avatar_index_template_roi(size: Size) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (_, y, width, height) = AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ROI_1080P;
    task_vision_result(Rect::new(
        size.width as i32 - (65.0 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn default_combat_avatar_side_icon_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_SIDE_ICON_RECTS_1080P)
}

fn scaled_1080p_rects(size: Size, rects: &[(i32, i32, i32, i32)]) -> Result<Vec<Rect>> {
    rects
        .iter()
        .copied()
        .map(|rect| scaled_1080p_rect(size, rect))
        .collect()
}

fn scaled_1080p_rect(size: Size, rect: (i32, i32, i32, i32)) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (x, y, width, height) = rect;
    task_vision_result(Rect::new(
        size.width as i32 - ((1920 - x) as f64 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn combat_multi_game_detection_from_icon_counts(
    p_icon_count: usize,
    one_p_icon_found: bool,
) -> Result<CombatMultiGameDetection> {
    if p_icon_count > 0 {
        let player_count = p_icon_count + 1;
        if player_count > 4 {
            return Err(TaskError::VisionPlan(
                "当前处于联机状态，但是队伍人数超过4人，无法识别".to_string(),
            ));
        }
        let status = CombatMultiGameStatus {
            is_in_multi_game: true,
            is_host: one_p_icon_found,
            player_count,
        };
        let message = if one_p_icon_found {
            format!("当前处于联机状态，且当前账号是房主，联机人数{player_count}人")
        } else {
            format!("当前处于联机状态，且在别人世界中，联机人数{player_count}人")
        };
        return Ok(CombatMultiGameDetection {
            status,
            p_icon_count,
            one_p_icon_found,
            message,
        });
    }

    if one_p_icon_found {
        return Ok(CombatMultiGameDetection {
            status: CombatMultiGameStatus {
                is_in_multi_game: true,
                is_host: true,
                player_count: 1,
            },
            p_icon_count,
            one_p_icon_found,
            message: "当前处于联机状态，但是没有其他玩家连入".to_string(),
        });
    }

    Ok(CombatMultiGameDetection {
        status: CombatMultiGameStatus::default(),
        p_icon_count,
        one_p_icon_found,
        message: "current scene is not in co-op mode".to_string(),
    })
}

pub fn detect_combat_multi_game_status(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatMultiGameDetection> {
    let p_icon_count = find_auto_fight_template_matches(
        &working_directory,
        image,
        AUTO_FIGHT_COOP_P_ASSET,
        default_auto_fight_coop_p_roi(image.size)?,
        "P",
        4,
    )?
    .len();
    let one_p_icon_found = !find_auto_fight_template_matches(
        working_directory,
        image,
        AUTO_FIGHT_COOP_ONE_P_ASSET,
        default_auto_fight_coop_one_p_roi(image.size)?,
        "1P",
        1,
    )?
    .is_empty();
    combat_multi_game_detection_from_icon_counts(p_icon_count, one_p_icon_found)
}

pub fn default_auto_fight_coop_one_p_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        size.width as i32 / 4,
        size.height as i32 / 7,
    ))
}

pub fn default_auto_fight_coop_p_roi(size: Size) -> Result<Rect> {
    let width = (size.width as f64 / 12.5).round() as i32;
    let height = size.height as i32 / 2 - size.width as i32 / 7;
    task_vision_result(Rect::new(
        size.width as i32 - width,
        size.height as i32 / 5,
        width,
        height,
    ))
}

pub fn combat_avatar_index_rects_for_multi_game_status(
    size: Size,
    status: CombatMultiGameStatus,
) -> Result<Vec<Rect>> {
    if let Some(key) = status.rect_map_key() {
        if let Some((_, rects)) = AUTO_FIGHT_COOP_SIDE_INDEX_RECTS_1080P
            .iter()
            .find(|(entry_key, _)| *entry_key == key)
        {
            return scaled_1080p_rects(size, rects);
        }
    }
    default_combat_avatar_index_rects(size)
}

pub fn combat_avatar_side_icon_rects_for_multi_game_status(
    size: Size,
    status: CombatMultiGameStatus,
) -> Result<Vec<Rect>> {
    if let Some(key) = status.rect_map_key() {
        if let Some((_, rects)) = AUTO_FIGHT_COOP_SIDE_ICON_RECTS_1080P
            .iter()
            .find(|(entry_key, _)| *entry_key == key)
        {
            return scaled_1080p_rects(size, rects);
        }
    }
    default_combat_avatar_side_icon_rects(size)
}

pub fn default_current_avatar_threshold_roi(size: Size) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (_, y, width, height) = AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ROI_1080P;
    task_vision_result(Rect::new(
        size.width as i32 - (240.0 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn active_combat_skill_cooldown_rect(size: Size, is_burst: bool) -> Result<Rect> {
    let rect = if is_burst {
        AUTO_FIGHT_ACTIVE_BURST_COOLDOWN_RECT_1080P
    } else {
        AUTO_FIGHT_ACTIVE_SKILL_COOLDOWN_RECT_1080P
    };
    scaled_1080p_rect(size, rect)
}

pub fn default_combat_avatar_side_burst_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P)
}

pub fn combat_avatar_side_burst_rect_for_index(size: Size, index: usize) -> Result<Rect> {
    if !(1..=AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P.len()).contains(&index) {
        return Err(TaskError::VisionPlan(format!(
            "combat avatar side burst index {index} is outside the supported party range"
        )));
    }
    default_combat_avatar_side_burst_rects(size)?
        .get(index - 1)
        .copied()
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "combat avatar side burst rect {index} is unavailable"
            ))
        })
}

pub fn combat_avatar_side_icon_rects_for_index_rects(
    image_size: Size,
    index_rects: &[Rect],
) -> Result<Vec<Rect>> {
    index_rects
        .iter()
        .copied()
        .map(|rect| combat_avatar_side_icon_rect_from_index_rect(image_size, rect))
        .collect()
}

pub fn combat_avatar_side_icon_rect_from_index_rect(
    image_size: Size,
    index_rect: Rect,
) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let (offset_x, offset_y, width, height) = AUTO_FIGHT_AVATAR_SIDE_ICON_FROM_INDEX_RECT_1080P;
    task_vision_result(Rect::new(
        index_rect.x + (offset_x as f64 * scale).round() as i32,
        index_rect.y + (offset_y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn detect_active_combat_avatar_index_from_default_rects(
    image: &BgrImage,
) -> Result<CombatActiveAvatarDetectionResult> {
    let rects = default_combat_avatar_index_rects(image.size)?;
    detect_active_combat_avatar_index_by_color(image, &rects)
}

pub fn detect_active_combat_avatar_index_from_default_rects_with_arrow(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatActiveAvatarDetectionResult> {
    let rects =
        combat_avatar_index_rect_detection_for_active_avatar_detection(&working_directory, image)?
            .resolved_rects;
    let roi = default_current_avatar_threshold_roi(image.size)?;
    detect_active_combat_avatar_index_by_color_then_arrow(working_directory, image, &rects, roi)
}

pub fn detect_combat_skill_readiness(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    index: usize,
    is_burst: bool,
) -> Result<CombatSkillReadinessDetection> {
    if index == 0 {
        return Err(TaskError::VisionPlan(
            "combat skill readiness index is 1-based".to_string(),
        ));
    }
    let kind = if is_burst {
        CombatSkillReadinessKind::ElementalBurst
    } else {
        CombatSkillReadinessKind::ElementalSkill
    };
    let active_detection =
        detect_active_combat_avatar_index_from_default_rects_with_arrow(working_directory, image)?;
    let Some(active_index) = active_detection.active_index else {
        return Ok(CombatSkillReadinessDetection {
            kind,
            requested_index: index,
            active_index: None,
            status: CombatSkillReadinessStatus::ActiveAvatarUnresolved,
            ready: None,
            active_detection,
            cooldown_rect: None,
            white_component_count: 0,
            legacy_connected_component_labels: 0,
            side_burst_rect: None,
            side_burst_circle: None,
            message: "active avatar index is unresolved; skill readiness cannot be checked"
                .to_string(),
        });
    };
    if active_index != index {
        if is_burst {
            let side_burst_rect = combat_avatar_side_burst_rect_for_index(image.size, index)?;
            let side_burst_circle = detect_side_burst_circle(image, side_burst_rect)?;
            let ready = side_burst_circle.detected;
            return Ok(CombatSkillReadinessDetection {
                kind,
                requested_index: index,
                active_index: Some(active_index),
                status: if ready {
                    CombatSkillReadinessStatus::Ready
                } else {
                    CombatSkillReadinessStatus::CooldownOrUnavailable
                },
                ready: Some(ready),
                active_detection,
                cooldown_rect: None,
                white_component_count: 0,
                legacy_connected_component_labels: 0,
                side_burst_rect: Some(side_burst_rect),
                side_burst_circle: Some(side_burst_circle),
                message: if ready {
                    "inactive avatar burst readiness was resolved by the legacy side-icon circle probe".to_string()
                } else {
                    "inactive avatar burst side-icon circle was not detected".to_string()
                },
            });
        }
        return Ok(CombatSkillReadinessDetection {
            kind,
            requested_index: index,
            active_index: Some(active_index),
            status: CombatSkillReadinessStatus::UnsupportedForInactiveAvatar,
            ready: None,
            active_detection,
            cooldown_rect: None,
            white_component_count: 0,
            legacy_connected_component_labels: 0,
            side_burst_rect: None,
            side_burst_circle: None,
            message: "inactive avatar elemental-skill readiness is not exposed by the legacy UI"
                .to_string(),
        });
    }

    let cooldown_rect = active_combat_skill_cooldown_rect(image.size, is_burst)?;
    let white_component_count = count_exact_white_components(image, cooldown_rect)?;
    let legacy_connected_component_labels = white_component_count + 1;
    let ready = legacy_connected_component_labels <= 2;
    Ok(CombatSkillReadinessDetection {
        kind,
        requested_index: index,
        active_index: Some(active_index),
        status: if ready {
            CombatSkillReadinessStatus::Ready
        } else {
            CombatSkillReadinessStatus::CooldownOrUnavailable
        },
        ready: Some(ready),
        active_detection,
        cooldown_rect: Some(cooldown_rect),
        white_component_count,
        legacy_connected_component_labels,
        side_burst_rect: None,
        side_burst_circle: None,
        message: if ready {
            "active avatar skill has no legacy cooldown digit components".to_string()
        } else {
            "active avatar skill has legacy cooldown digit/lock components".to_string()
        },
    })
}

pub fn combat_avatar_index_rect_detection_for_active_avatar_detection(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatAvatarIndexRectsDetection> {
    match detect_combat_multi_game_status(&working_directory, image) {
        Ok(detection) if detection.status.is_in_multi_game => {
            let rects =
                combat_avatar_index_rects_for_multi_game_status(image.size, detection.status)?;
            Ok(CombatAvatarIndexRectsDetection {
                rects_by_index: rects.iter().copied().map(Some).collect(),
                resolved_rects: rects,
                inferred_from_current_avatar_arrow: false,
                message: detection.message,
            })
        }
        _ => match detect_combat_avatar_index_rects_from_templates(&working_directory, image) {
            Ok(dynamic_rects)
                if dynamic_rects.resolved_rects.len()
                    == AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS.len()
                    || dynamic_rects.inferred_from_current_avatar_arrow =>
            {
                Ok(dynamic_rects)
            }
            _ => {
                let rects = default_combat_avatar_index_rects(image.size)?;
                Ok(CombatAvatarIndexRectsDetection {
                    rects_by_index: rects.iter().copied().map(Some).collect(),
                    resolved_rects: rects,
                    inferred_from_current_avatar_arrow: false,
                    message: "fell back to default avatar index rectangles".to_string(),
                })
            }
        },
    }
}

pub fn detect_combat_avatar_index_rects_from_templates(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatAvatarIndexRectsDetection> {
    let index_roi = default_combat_avatar_index_template_roi(image.size)?;
    let arrow_roi = default_current_avatar_threshold_roi(image.size)?;
    detect_combat_avatar_index_rects_from_templates_in(
        working_directory,
        image,
        index_roi,
        arrow_roi,
    )
}

pub fn detect_combat_avatar_index_rects_from_templates_in(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    index_roi: Rect,
    arrow_roi: Rect,
) -> Result<CombatAvatarIndexRectsDetection> {
    let mut rects_by_index = vec![None; AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS.len()];
    for (index, asset_name) in AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS
        .iter()
        .copied()
        .enumerate()
    {
        rects_by_index[index] = find_combat_avatar_index_template_rect(
            &working_directory,
            image,
            asset_name,
            index_roi,
            &format!("Index{}", index + 1),
        )?;
    }

    let existing_count = rects_by_index.iter().filter(|rect| rect.is_some()).count();
    if existing_count == rects_by_index.len() {
        return Ok(CombatAvatarIndexRectsDetection {
            resolved_rects: rects_by_index.iter().flatten().copied().collect(),
            rects_by_index,
            inferred_from_current_avatar_arrow: false,
            message: "detected all avatar index rectangles by template matching".to_string(),
        });
    }

    let current_avatar =
        find_current_avatar_arrow_template_rect(&working_directory, image, arrow_roi)?;
    if let Some(current_avatar) = current_avatar {
        if let Some((known_index, known_rect)) = first_known_avatar_index_rect(&rects_by_index) {
            let inferred = infer_avatar_index_rects_from_known_and_arrow(
                image.size,
                &rects_by_index,
                known_index,
                known_rect,
                current_avatar,
            )?;
            if avatar_index_rects_are_contiguous(&inferred) {
                let resolved_rects = inferred.iter().flatten().copied().collect();
                return Ok(CombatAvatarIndexRectsDetection {
                    rects_by_index: inferred,
                    resolved_rects,
                    inferred_from_current_avatar_arrow: true,
                    message: "inferred a missing active avatar index rectangle from the current-avatar arrow".to_string(),
                });
            }
        } else {
            let inferred = index_rect_from_current_avatar_arrow(image.size, current_avatar)?;
            return Ok(CombatAvatarIndexRectsDetection {
                rects_by_index: vec![Some(inferred), None, None, None],
                resolved_rects: vec![inferred],
                inferred_from_current_avatar_arrow: true,
                message: "inferred a single avatar index rectangle from the current-avatar arrow"
                    .to_string(),
            });
        }
    }

    Ok(CombatAvatarIndexRectsDetection {
        resolved_rects: rects_by_index.iter().flatten().copied().collect(),
        rects_by_index,
        inferred_from_current_avatar_arrow: false,
        message: format!("detected {existing_count} avatar index rectangles by template matching"),
    })
}

pub fn recognize_combat_team_from_avatar_side_icons(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    action_scheduler_by_cd: &str,
    classifier: &mut impl CombatAvatarSideClassifier,
) -> Result<CombatTeamRecognitionExecution> {
    let catalog = read_combat_avatar_catalog(&working_directory)?;
    let multi_game_detection = detect_combat_multi_game_status(&working_directory, image)
        .unwrap_or_else(|error| CombatMultiGameDetection {
            status: CombatMultiGameStatus::default(),
            p_icon_count: 0,
            one_p_icon_found: false,
            message: format!(
                "co-op status detection failed; using single-player rectangles: {error}"
            ),
        });
    let index_rect_detection = if multi_game_detection.status.is_in_multi_game {
        let rects = combat_avatar_index_rects_for_multi_game_status(
            image.size,
            multi_game_detection.status,
        )?;
        CombatAvatarIndexRectsDetection {
            rects_by_index: rects.iter().copied().map(Some).collect(),
            resolved_rects: rects,
            inferred_from_current_avatar_arrow: false,
            message: multi_game_detection.message.clone(),
        }
    } else {
        match detect_combat_avatar_index_rects_from_templates(&working_directory, image) {
            Ok(detection) if !detection.resolved_rects.is_empty() => detection,
            _ => {
                let rects = default_combat_avatar_index_rects(image.size)?;
                CombatAvatarIndexRectsDetection {
                    rects_by_index: rects.iter().copied().map(Some).collect(),
                    resolved_rects: rects,
                    inferred_from_current_avatar_arrow: false,
                    message: "fell back to default avatar index rectangles".to_string(),
                }
            }
        }
    };
    let side_icon_rects = if multi_game_detection.status.is_in_multi_game {
        combat_avatar_side_icon_rects_for_multi_game_status(
            image.size,
            multi_game_detection.status,
        )?
    } else {
        combat_avatar_side_icon_rects_for_index_rects(
            image.size,
            &index_rect_detection.resolved_rects,
        )?
    };
    if side_icon_rects.len() != index_rect_detection.resolved_rects.len() {
        return Err(TaskError::VisionPlan(format!(
            "avatar index rectangle count ({}) does not match side-icon rectangle count ({})",
            index_rect_detection.resolved_rects.len(),
            side_icon_rects.len()
        )));
    }
    let mut avatars = Vec::with_capacity(side_icon_rects.len());
    for (position, (index_rect, side_icon_rect)) in index_rect_detection
        .resolved_rects
        .iter()
        .copied()
        .zip(side_icon_rects.iter().copied())
        .enumerate()
    {
        let crop = task_vision_result(crop_bgr_image(image, side_icon_rect))?;
        let classification =
            classifier.classify_avatar_side(position + 1, &crop, side_icon_rect)?;
        let recognition = combat_avatar_side_recognition_from_classification(
            &catalog,
            position + 1,
            index_rect,
            side_icon_rect,
            classification,
        )?;
        avatars.push(recognition);
    }
    let team_avatar_names: Vec<_> = avatars
        .iter()
        .map(|avatar| avatar.avatar_name.clone())
        .collect();
    let team_plan = plan_combat_team(
        &catalog,
        &team_avatar_names,
        &team_avatar_names,
        action_scheduler_by_cd,
    )?;
    Ok(CombatTeamRecognitionExecution {
        index_rect_detection,
        avatars,
        team_avatar_names,
        team_plan,
    })
}

pub fn combat_avatar_side_recognition_from_classification(
    catalog: &CombatAvatarCatalog,
    index: usize,
    index_rect: Rect,
    side_icon_rect: Rect,
    classification: CombatAvatarSideClassification,
) -> Result<CombatAvatarSideRecognition> {
    validate_avatar_side_classification(index, &classification)?;
    let (name_en, costume_name) = split_avatar_side_class_name(&classification.class_name);
    let avatar = catalog.avatar_by_name_en(&name_en).ok_or_else(|| {
        TaskError::CombatStrategy(format!(
            "avatar-side classifier returned an unknown class for slot {index}: {}",
            classification.class_name
        ))
    })?;
    let display_name = costume_name
        .as_deref()
        .map(|costume| format!("{}({})", avatar.name, display_avatar_costume_name(costume)))
        .unwrap_or_else(|| avatar.name.clone());
    Ok(CombatAvatarSideRecognition {
        index,
        avatar_name: avatar.name.clone(),
        name_en,
        costume_name,
        display_name,
        confidence: classification.confidence,
        index_rect,
        side_icon_rect,
    })
}

pub fn detect_active_combat_avatar_index_by_color_then_arrow(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    rects: &[Rect],
    arrow_roi: Rect,
) -> Result<CombatActiveAvatarDetectionResult> {
    let color_result = detect_active_combat_avatar_index_by_color(image, rects)?;
    if color_result.active_index.is_some() {
        return Ok(color_result);
    }

    detect_active_combat_avatar_index_by_arrow_template(
        working_directory,
        image,
        rects,
        arrow_roi,
        color_result,
    )
}

pub fn detect_active_combat_avatar_index_by_color(
    image: &BgrImage,
    rects: &[Rect],
) -> Result<CombatActiveAvatarDetectionResult> {
    if rects.is_empty() {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: None,
            method: CombatActiveAvatarDetectionMethod::Unresolved,
            rects: Vec::new(),
            white_rect_count: 0,
            not_white_rect_index: None,
            edge_white_ratios: Vec::new(),
            difference_votes: Vec::new(),
            message: "no avatar index rectangles were provided".to_string(),
        });
    }
    if rects.len() == 1 {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: Some(1),
            method: CombatActiveAvatarDetectionMethod::SingleAvatar,
            rects: rects.to_vec(),
            white_rect_count: 0,
            not_white_rect_index: Some(1),
            edge_white_ratios: Vec::new(),
            difference_votes: Vec::new(),
            message: "single controllable avatar is active by definition".to_string(),
        });
    }

    let mut white_rect_count = 0;
    let mut not_white_rect_index = None;
    let mut gray_regions = Vec::with_capacity(rects.len());
    for (index, rect) in rects.iter().copied().enumerate() {
        let gray = gray_region(image, rect)?;
        if is_avatar_index_white_rect(&gray) {
            white_rect_count += 1;
        } else {
            not_white_rect_index = Some(index + 1);
        }
        gray_regions.push((rect, gray));
    }

    if white_rect_count == rects.len() - 1 {
        let active_index = not_white_rect_index;
        return Ok(CombatActiveAvatarDetectionResult {
            active_index,
            method: CombatActiveAvatarDetectionMethod::WhiteRectMajority,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios: gray_regions
                .iter()
                .map(|(_, gray)| avatar_index_white_edge_ratio(gray))
                .collect(),
            difference_votes: Vec::new(),
            message: format!(
                "detected active avatar by white index blocks: {:?}",
                active_index
            ),
        });
    }

    let edge_white_ratios: Vec<_> = gray_regions
        .iter()
        .map(|(_, gray)| avatar_index_white_edge_ratio(gray))
        .collect();
    let edge_white_count = edge_white_ratios
        .iter()
        .filter(|ratio| **ratio > 0.5)
        .count();
    let edge_not_white_index = edge_white_ratios
        .iter()
        .enumerate()
        .rev()
        .find(|(_, ratio)| **ratio <= 0.5)
        .map(|(index, _)| index + 1);
    if edge_white_count == rects.len() - 1 {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: edge_not_white_index,
            method: CombatActiveAvatarDetectionMethod::EdgeWhiteRatio,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios,
            difference_votes: Vec::new(),
            message: format!(
                "detected active avatar by white edge ratio: {:?}",
                edge_not_white_index
            ),
        });
    }
    if edge_white_count == rects.len() {
        let black_indexes: Vec<_> = gray_regions
            .iter()
            .enumerate()
            .filter(|(_, (_, gray))| count_gray_range(gray, 50, 50) > 0)
            .map(|(index, _)| index + 1)
            .collect();
        let not_black_index = (1..=rects.len()).find(|index| !black_indexes.contains(index));
        if let Some(active_index) = not_black_index {
            return Ok(CombatActiveAvatarDetectionResult {
                active_index: Some(active_index),
                method: CombatActiveAvatarDetectionMethod::EdgeWhiteRatio,
                rects: rects.to_vec(),
                white_rect_count,
                not_white_rect_index,
                edge_white_ratios,
                difference_votes: Vec::new(),
                message: format!(
                    "all index edges are white; active avatar inferred from missing black digit: {active_index}"
                ),
            });
        }
    }

    if gray_regions.len() == 4
        && gray_regions
            .iter()
            .map(|(_, gray)| (gray.width, gray.height))
            .all(|size| size == (gray_regions[0].1.width, gray_regions[0].1.height))
    {
        let (active_index, votes) = most_different_avatar_index(&gray_regions);
        if let Some(active_index) = active_index {
            return Ok(CombatActiveAvatarDetectionResult {
                active_index: Some(active_index),
                method: CombatActiveAvatarDetectionMethod::ImageDifferenceVote,
                rects: rects.to_vec(),
                white_rect_count,
                not_white_rect_index,
                edge_white_ratios,
                difference_votes: votes,
                message: format!("detected active avatar by image-difference vote: {active_index}"),
            });
        }
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: None,
            method: CombatActiveAvatarDetectionMethod::Unresolved,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios,
            difference_votes: votes,
            message: "active avatar index could not be resolved by color or difference voting"
                .to_string(),
        });
    }

    Ok(CombatActiveAvatarDetectionResult {
        active_index: None,
        method: CombatActiveAvatarDetectionMethod::Unresolved,
        rects: rects.to_vec(),
        white_rect_count,
        not_white_rect_index,
        edge_white_ratios,
        difference_votes: Vec::new(),
        message: "active avatar index could not be resolved by color heuristics".to_string(),
    })
}

pub fn detect_active_combat_avatar_index_by_arrow_template(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    rects: &[Rect],
    arrow_roi: Rect,
    mut base_result: CombatActiveAvatarDetectionResult,
) -> Result<CombatActiveAvatarDetectionResult> {
    if rects.len() == 1 {
        base_result.active_index = Some(1);
        base_result.method = CombatActiveAvatarDetectionMethod::SingleAvatar;
        base_result.message = "single controllable avatar is active by definition".to_string();
        return Ok(base_result);
    }
    if rects.is_empty() {
        return Ok(base_result);
    }

    let resolver = AssetResolver::new(working_directory.as_ref());
    let current_avatar =
        find_current_avatar_arrow_template_rect_with_resolver(&resolver, image, arrow_roi)?;

    let Some(current_avatar) = current_avatar else {
        base_result.message =
            "active avatar index could not be resolved by color heuristics or arrow template"
                .to_string();
        return Ok(base_result);
    };

    if let Some((index, _)) = rects
        .iter()
        .copied()
        .enumerate()
        .find(|(_, rect)| rects_intersect_vertically(current_avatar, *rect))
    {
        base_result.active_index = Some(index + 1);
        base_result.method = CombatActiveAvatarDetectionMethod::ArrowTemplate;
        base_result.message = format!(
            "detected active avatar by current-avatar arrow template: {}",
            index + 1
        );
        return Ok(base_result);
    }

    base_result.message = format!(
        "current-avatar arrow template was found at {:?}, but it did not intersect any avatar index rectangle",
        current_avatar
    );
    Ok(base_result)
}

fn find_combat_avatar_index_template_rect(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    asset_name: &str,
    roi: Rect,
    object_name: &str,
) -> Result<Option<Rect>> {
    let resolver = AssetResolver::new(working_directory.as_ref());
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_CURRENT_AVATAR_FEATURE,
            asset_name,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(None);
    }
    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some(object_name.to_string());
    object.template.threshold = 0.95;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.max_match_count = 1;
    let region = task_vision_result(backend.find(&image.pixels, image.size, &object))?;
    Ok(region.is_exist().then_some(region.rect))
}

fn find_auto_fight_template_matches(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    asset_name: &str,
    roi: Rect,
    object_name: &str,
    max_match_count: i32,
) -> Result<Vec<Rect>> {
    let resolver = AssetResolver::new(working_directory.as_ref());
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_FEATURE,
            asset_name,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(Vec::new());
    }
    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some(object_name.to_string());
    object.template.threshold = 0.8;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.max_match_count = max_match_count;
    let matches = task_vision_result(backend.find_multi(&image.pixels, image.size, &object))?;
    Ok(matches.into_iter().map(|region| region.rect).collect())
}

fn validate_avatar_side_classification(
    index: usize,
    classification: &CombatAvatarSideClassification,
) -> Result<()> {
    let minimum_confidence = if classification.class_name.starts_with("Qin")
        || classification.class_name.contains("Costume")
    {
        0.51
    } else {
        0.7
    };
    if classification.confidence < minimum_confidence {
        return Err(TaskError::CombatStrategy(format!(
            "无法识别第{index}位角色，置信度{:.2}，结果：{}",
            classification.confidence, classification.class_name
        )));
    }
    Ok(())
}

fn split_avatar_side_class_name(class_name: &str) -> (String, Option<String>) {
    if let Some(index) = class_name.find("Costume") {
        let name_en = class_name[..index].to_string();
        let costume_name = class_name[index + "Costume".len()..].to_string();
        return (name_en, (!costume_name.is_empty()).then_some(costume_name));
    }
    (class_name.to_string(), None)
}

fn display_avatar_costume_name(costume_name: &str) -> String {
    match costume_name {
        "Flamme" => "殷红终夜",
        "Bamboo" => "雨化竹身",
        "Dai" => "冷花幽露",
        "Yu" => "玄玉瑶芳",
        "Dancer" => "帆影游风",
        "Witch" => "琪花星烛",
        "Wic" => "和谐",
        "Studentin" => "叶隐芳名",
        "Fruhling" => "花时来信",
        "Highness" => "极夜真梦",
        "Feather" => "霓裾翩跹",
        "Floral" => "纱中幽兰",
        "Summertime" => "闪耀协奏",
        "Sea" => "海风之梦",
        _ => costume_name,
    }
    .to_string()
}

fn find_current_avatar_arrow_template_rect(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    roi: Rect,
) -> Result<Option<Rect>> {
    let resolver = AssetResolver::new(working_directory.as_ref());
    find_current_avatar_arrow_template_rect_with_resolver(&resolver, image, roi)
}

fn find_current_avatar_arrow_template_rect_with_resolver(
    resolver: &AssetResolver,
    image: &BgrImage,
    roi: Rect,
) -> Result<Option<Rect>> {
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_CURRENT_AVATAR_FEATURE,
            AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(None);
    }

    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some("CurrentAvatarThreshold".to_string());
    object.template.threshold = 0.8;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.use_binary_match = true;
    object.template.binary_threshold = 200;
    object.template.max_match_count = 1;
    let region = task_vision_result(backend.find(&image.pixels, image.size, &object))?;
    Ok(region.is_exist().then_some(region.rect))
}

fn first_known_avatar_index_rect(rects_by_index: &[Option<Rect>]) -> Option<(usize, Rect)> {
    rects_by_index
        .iter()
        .copied()
        .enumerate()
        .find_map(|(index, rect)| rect.map(|rect| (index + 1, rect)))
}

fn infer_avatar_index_rects_from_known_and_arrow(
    image_size: Size,
    rects_by_index: &[Option<Rect>],
    known_index: usize,
    known_rect: Rect,
    current_avatar: Rect,
) -> Result<Vec<Option<Rect>>> {
    let mut inferred = rects_by_index.to_vec();
    for index in 0..inferred.len() {
        if inferred[index].is_some() {
            continue;
        }
        let rect =
            index_rect_from_known_index_rect(image_size, known_index, known_rect, index + 1)?;
        if rects_intersect_vertically(current_avatar, rect) {
            inferred[index] = Some(rect);
        }
    }
    Ok(inferred)
}

fn index_rect_from_known_index_rect(
    image_size: Size,
    known_index: usize,
    known_rect: Rect,
    target_index: usize,
) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let distance = (AUTO_FIGHT_AVATAR_INDEX_DISTANCE_Y_1080P as f64 * scale).round() as i32;
    task_vision_result(Rect::new(
        known_rect.x,
        known_rect.y + (target_index as i32 - known_index as i32) * distance,
        known_rect.width,
        known_rect.height,
    ))
}

fn index_rect_from_current_avatar_arrow(image_size: Size, current_avatar: Rect) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let (offset_x, offset_y, width, height) = AUTO_FIGHT_CURRENT_AVATAR_FLAG_TO_INDEX_RECT_1080P;
    task_vision_result(Rect::new(
        current_avatar.x + (offset_x as f64 * scale).round() as i32,
        current_avatar.y + (offset_y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

fn avatar_index_rects_are_contiguous(rects_by_index: &[Option<Rect>]) -> bool {
    let mut seen_gap = false;
    for rect in rects_by_index {
        if rect.is_some() {
            if seen_gap {
                return false;
            }
        } else {
            seen_gap = true;
        }
    }
    true
}

fn screen_size_from_image(image: &BgrImage) -> ScreenSize {
    ScreenSize {
        width: image.size.width,
        height: image.size.height,
    }
}

fn rects_intersect_vertically(left: Rect, right: Rect) -> bool {
    left.y < right.bottom() && right.y < left.bottom()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GrayRegion {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

fn gray_region(image: &BgrImage, rect: Rect) -> Result<GrayRegion> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "avatar index rectangle is outside capture: {rect:?}"
        )));
    }
    let width = rect.width as usize;
    let height = rect.height as usize;
    let mut pixels = Vec::with_capacity(width * height);
    for y in rect.y as u32..rect.bottom() as u32 {
        for x in rect.x as u32..rect.right() as u32 {
            let pixel = image.bgr_pixel_at(x, y).ok_or_else(|| {
                TaskError::VisionPlan(format!("avatar index pixel ({x}, {y}) is outside capture"))
            })?;
            pixels.push(gray_from_rgb(pixel.r, pixel.g, pixel.b));
        }
    }
    Ok(GrayRegion {
        width,
        height,
        pixels,
    })
}

fn gray_from_rgb(r: u8, g: u8, b: u8) -> u8 {
    (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8
}

fn is_avatar_index_white_rect(gray: &GrayRegion) -> bool {
    let white_count = count_gray_range(gray, 251, 255);
    let black_text_count = count_gray_range(gray, 50, 54);
    (white_count + black_text_count) as f64 / gray.pixels.len() as f64 > 0.35
}

fn count_gray_range(gray: &GrayRegion, lower: u8, upper: u8) -> usize {
    gray.pixels
        .iter()
        .filter(|pixel| (lower..=upper).contains(pixel))
        .count()
}

fn avatar_index_white_edge_ratio(gray: &GrayRegion) -> f64 {
    if gray.width == 0 || gray.height == 0 {
        return 0.0;
    }
    let mut white_count = 0usize;
    let total_count = 2 * (gray.width + gray.height).saturating_sub(4);
    for x in 0..gray.width {
        if gray_pixel(gray, x, 0) == Some(255) {
            white_count += 1;
        }
        if gray.height > 1 && gray_pixel(gray, x, gray.height - 1) == Some(255) {
            white_count += 1;
        }
    }
    for y in 1..gray.height.saturating_sub(1) {
        if gray_pixel(gray, 0, y) == Some(255) {
            white_count += 1;
        }
        if gray.width > 1 && gray_pixel(gray, gray.width - 1, y) == Some(255) {
            white_count += 1;
        }
    }
    if total_count == 0 {
        0.0
    } else {
        white_count as f64 / total_count as f64
    }
}

fn gray_pixel(gray: &GrayRegion, x: usize, y: usize) -> Option<u8> {
    (x < gray.width && y < gray.height).then(|| gray.pixels[y * gray.width + x])
}

fn most_different_avatar_index(gray_regions: &[(Rect, GrayRegion)]) -> (Option<usize>, Vec<usize>) {
    let mut votes = vec![0usize; gray_regions.len()];
    for i in 0..gray_regions.len() {
        let mut max_diff_index = None;
        let mut max_difference = 0usize;
        for j in 0..gray_regions.len() {
            if i == j {
                continue;
            }
            let difference = gray_difference_count(&gray_regions[i].1, &gray_regions[j].1);
            if difference > max_difference {
                max_difference = difference;
                max_diff_index = Some(j);
            }
        }
        if let Some(index) = max_diff_index {
            votes[index] += 1;
        }
    }
    let active = votes
        .iter()
        .enumerate()
        .find(|(_, votes)| **votes >= 3)
        .map(|(index, _)| index + 1);
    (active, votes)
}

fn gray_difference_count(left: &GrayRegion, right: &GrayRegion) -> usize {
    if left.width != right.width || left.height != right.height {
        return 0;
    }
    left.pixels
        .iter()
        .zip(&right.pixels)
        .filter(|(left, right)| left != right)
        .count()
}

fn count_exact_white_components(image: &BgrImage, rect: Rect) -> Result<usize> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "skill cooldown rectangle is outside capture: {rect:?}"
        )));
    }
    let width = rect.width as usize;
    let height = rect.height as usize;
    let mut mask = vec![false; width * height];
    for local_y in 0..height {
        for local_x in 0..width {
            let x = rect.x as u32 + local_x as u32;
            let y = rect.y as u32 + local_y as u32;
            let pixel = image.rgb_pixel_at(x, y).ok_or_else(|| {
                TaskError::VisionPlan(format!(
                    "skill cooldown pixel ({x}, {y}) is outside capture"
                ))
            })?;
            mask[local_y * width + local_x] = pixel.r == 255 && pixel.g == 255 && pixel.b == 255;
        }
    }
    Ok(count_binary_components_8_connected(&mask, width, height))
}

fn count_binary_components_8_connected(mask: &[bool], width: usize, height: usize) -> usize {
    if width == 0 || height == 0 {
        return 0;
    }
    let mut visited = vec![false; mask.len()];
    let mut components = 0usize;
    let mut stack = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if !mask[index] || visited[index] {
                continue;
            }
            components += 1;
            visited[index] = true;
            stack.push((x, y));
            while let Some((cx, cy)) = stack.pop() {
                let min_x = cx.saturating_sub(1);
                let max_x = (cx + 1).min(width - 1);
                let min_y = cy.saturating_sub(1);
                let max_y = (cy + 1).min(height - 1);
                for ny in min_y..=max_y {
                    for nx in min_x..=max_x {
                        let neighbor = ny * width + nx;
                        if mask[neighbor] && !visited[neighbor] {
                            visited[neighbor] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }
    }
    components
}

pub fn detect_side_burst_circle(
    image: &BgrImage,
    rect: Rect,
) -> Result<CombatSideBurstCircleDetection> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "side burst rectangle is outside capture: {rect:?}"
        )));
    }
    let gray = gray_region(image, rect)?;
    let edge_mask = side_burst_edge_mask(&gray);
    let edge_pixel_count = edge_mask.iter().filter(|edge| **edge).count();
    let scale = image.size.height as f64 / 1080.0;
    let min_radius =
        ((AUTO_FIGHT_SIDE_BURST_MIN_RADIUS_1080P as f64 * scale).round() as i32).max(1);
    let max_radius =
        ((AUTO_FIGHT_SIDE_BURST_MAX_RADIUS_1080P as f64 * scale).round() as i32).max(min_radius);
    let mut best_center = None;
    let mut best_radius = None;
    let mut best_votes = 0usize;
    for radius in min_radius..=max_radius {
        if radius * 2 > rect.width.max(rect.height) + 16 {
            continue;
        }
        for cy in radius..(rect.height - radius).max(radius + 1) {
            for cx in radius..(rect.width - radius).max(radius + 1) {
                let votes = circle_edge_votes(
                    &edge_mask,
                    gray.width,
                    gray.height,
                    cx,
                    cy,
                    radius,
                    AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES,
                );
                if votes > best_votes {
                    best_votes = votes;
                    best_center = Some((rect.x + cx, rect.y + cy));
                    best_radius = Some(radius);
                }
            }
        }
    }
    Ok(CombatSideBurstCircleDetection {
        rect,
        detected: best_votes >= AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES,
        edge_pixel_count,
        best_center,
        best_radius,
        best_votes,
        required_votes: AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES,
        sampled_points: AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES,
    })
}

fn side_burst_edge_mask(gray: &GrayRegion) -> Vec<bool> {
    let mut edges = vec![false; gray.width * gray.height];
    if gray.width < 3 || gray.height < 3 {
        return edges;
    }
    let mean = gray.pixels.iter().map(|pixel| *pixel as u64).sum::<u64>() as f64
        / gray.pixels.len() as f64;
    let bright_threshold = (mean + 35.0).clamp(150.0, 235.0).round() as u8;
    for y in 1..gray.height - 1 {
        for x in 1..gray.width - 1 {
            let center = gray_pixel(gray, x, y).unwrap_or_default();
            let left = gray_pixel(gray, x - 1, y).unwrap_or_default() as i32;
            let right = gray_pixel(gray, x + 1, y).unwrap_or_default() as i32;
            let up = gray_pixel(gray, x, y - 1).unwrap_or_default() as i32;
            let down = gray_pixel(gray, x, y + 1).unwrap_or_default() as i32;
            let gradient = (right - left).abs() + (down - up).abs();
            edges[y * gray.width + x] = gradient >= 45 || center >= bright_threshold;
        }
    }
    edges
}

fn circle_edge_votes(
    edge_mask: &[bool],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
    samples: usize,
) -> usize {
    let mut votes = 0usize;
    let mut last_point = None;
    for sample in 0..samples {
        let angle = sample as f64 * std::f64::consts::TAU / samples as f64;
        let x = center_x + (radius as f64 * angle.cos()).round() as i32;
        let y = center_y + (radius as f64 * angle.sin()).round() as i32;
        let point = (x, y);
        if last_point == Some(point) {
            continue;
        }
        last_point = Some(point);
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            votes += edge_mask[y as usize * width + x as usize] as usize;
        }
    }
    votes
}

pub fn plan_auto_fight_finish_detection(
    config: &FightFinishDetectParam,
    delay_ms: u64,
    detect_delay_ms: u64,
) -> Result<AutoFightFinishDetectionPlan> {
    let bindings = KeyBindingsConfig::default();
    let open_party_events = combat_action_events(
        &bindings,
        GenshinAction::OpenPartySetupScreen,
        KeyActionType::KeyPress,
    )?;
    let drop_events =
        combat_action_events(&bindings, GenshinAction::Drop, KeyActionType::KeyPress)?;
    let mut steps = Vec::new();
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::PreDetectDelay,
        !config.rotate_find_enemy_enabled && delay_ms > 0,
        Vec::new(),
        delay_ms,
        false,
        false,
        "wait before opening party setup when rotate-find-enemy is disabled",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::SeekEnemy,
        config.rotate_find_enemy_enabled,
        Vec::new(),
        delay_ms,
        true,
        true,
        "run seek-and-fight finish probe before party-screen pixel detection",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::OpenPartySetup,
        true,
        open_party_events.clone(),
        0,
        false,
        false,
        "press the configured party setup key before checking fight-finish pixels",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::WaitForPartySetup,
        detect_delay_ms > 0,
        Vec::new(),
        detect_delay_ms,
        false,
        false,
        "wait for the party setup screen before capturing a frame",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::CaptureFrame,
        true,
        Vec::new(),
        0,
        true,
        false,
        "capture the game frame after opening party setup",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::SampleFinishPixels,
        true,
        Vec::new(),
        0,
        false,
        true,
        "sample the legacy white tile and yellow progress pixels",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::DropFromPartySetup,
        true,
        drop_events,
        0,
        false,
        false,
        "press the configured drop key to leave the party setup probe",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished,
        true,
        open_party_events,
        0,
        false,
        false,
        "press party setup again when finish pixels indicate combat has ended",
    );
    let native_ready_without_capture = !steps
        .iter()
        .any(|step| step.enabled && (step.requires_capture || step.requires_vision));
    Ok(AutoFightFinishDetectionPlan {
        pre_detect_delay_ms: delay_ms,
        detect_delay_ms,
        rotate_find_enemy_enabled: config.rotate_find_enemy_enabled,
        progress_pixel: AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
        white_tile_pixel: AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
        steps,
        native_ready_without_capture,
    })
}

fn push_finish_detection_step(
    steps: &mut Vec<AutoFightFinishDetectionStepPlan>,
    kind: AutoFightFinishDetectionStepKind,
    enabled: bool,
    input_events: Vec<InputEvent>,
    delay_ms: u64,
    requires_capture: bool,
    requires_vision: bool,
    message: &str,
) {
    steps.push(AutoFightFinishDetectionStepPlan {
        kind,
        enabled,
        input_events,
        delay_ms,
        requires_capture,
        requires_vision,
        message: message.to_string(),
    });
}

pub fn execute_auto_fight_finish_detection_probe(
    plan: &AutoFightFinishDetectionPlan,
    image: &BgrImage,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<AutoFightFinishDetectionExecution> {
    let before_capture_events = finish_detection_events_before_capture(plan);
    let detection = detect_auto_fight_finished_from_image(image)?;
    let after_detection_events = finish_detection_events_after_detection(plan, detection.finished);
    let mut dispatched_events = 0;
    let mut cancelled = false;
    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let mut events = before_capture_events.clone();
        events.extend(after_detection_events.iter().copied());
        let result = dispatch_auto_fight_input_events(&events, cancellation)?;
        dispatched_events = result.0;
        cancelled = result.1;
    }
    Ok(AutoFightFinishDetectionExecution {
        mode,
        plan: plan.clone(),
        detection,
        before_capture_events,
        after_detection_events,
        dispatched: matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput),
        dispatched_events,
        cancelled,
    })
}

pub fn execute_auto_fight_finish_detection_live_probe(
    plan: &AutoFightFinishDetectionPlan,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Option<&InputCancellationToken>,
    capture: impl FnOnce() -> Result<BgrImage>,
) -> Result<AutoFightFinishDetectionLiveExecution> {
    let before_capture_events = finish_detection_events_before_capture(plan);
    let mut dispatched_events = 0;
    let mut cancelled = false;

    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let result = dispatch_auto_fight_input_events(&before_capture_events, cancellation)?;
        dispatched_events += result.0;
        cancelled = result.1;
        if cancelled {
            return Ok(AutoFightFinishDetectionLiveExecution {
                mode,
                plan: plan.clone(),
                detection: None,
                before_capture_events,
                after_detection_events: Vec::new(),
                dispatched: true,
                dispatched_events,
                cancelled,
                captured: false,
            });
        }
    }

    let image = capture()?;
    let detection = detect_auto_fight_finished_from_image(&image)?;
    let after_detection_events = finish_detection_events_after_detection(plan, detection.finished);

    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let result = dispatch_auto_fight_input_events(&after_detection_events, cancellation)?;
        dispatched_events += result.0;
        cancelled = result.1;
    }

    Ok(AutoFightFinishDetectionLiveExecution {
        mode,
        plan: plan.clone(),
        detection: Some(detection),
        before_capture_events,
        after_detection_events,
        dispatched: matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput),
        dispatched_events,
        cancelled,
        captured: true,
    })
}

fn dispatch_auto_fight_input_events(
    events: &[InputEvent],
    cancellation: Option<&InputCancellationToken>,
) -> Result<(usize, bool)> {
    if events.is_empty() {
        return Ok((0, false));
    }
    let result = if let Some(cancellation) = cancellation {
        match send_events_with_cancellation(events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error),
        }
    } else {
        send_events(events).map(|_| (events.len(), false))
    };
    result.map_err(|error| TaskError::CombatInputDispatch(error.to_string()))
}

pub fn finish_detection_events_before_capture(
    plan: &AutoFightFinishDetectionPlan,
) -> Vec<InputEvent> {
    plan.steps
        .iter()
        .filter(|step| {
            step.enabled
                && matches!(
                    step.kind,
                    AutoFightFinishDetectionStepKind::PreDetectDelay
                        | AutoFightFinishDetectionStepKind::OpenPartySetup
                        | AutoFightFinishDetectionStepKind::WaitForPartySetup
                )
        })
        .flat_map(finish_detection_step_events)
        .collect()
}

pub fn finish_detection_events_after_detection(
    plan: &AutoFightFinishDetectionPlan,
    finished: bool,
) -> Vec<InputEvent> {
    plan.steps
        .iter()
        .filter(|step| {
            step.enabled
                && matches!(
                    step.kind,
                    AutoFightFinishDetectionStepKind::DropFromPartySetup
                        | AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished
                )
                && (step.kind != AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished
                    || finished)
        })
        .flat_map(finish_detection_step_events)
        .collect()
}

fn finish_detection_step_events(step: &AutoFightFinishDetectionStepPlan) -> Vec<InputEvent> {
    let mut events = Vec::new();
    events.extend(step.input_events.iter().copied());
    if step.delay_ms > 0 {
        events.push(InputEvent::Delay {
            milliseconds: step.delay_ms,
        });
    }
    events
}

pub fn execute_static_combat_script_inputs(
    script: &CombatScriptExecutionPlan,
    mode: CombatCommandPlaybackMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<CombatCommandPlaybackExecution> {
    let evaluation = evaluate_combat_script_playback(script);
    if evaluation.context_bound_commands > 0 {
        return Err(TaskError::CombatStrategy(format!(
            "combat script requires native combat context before input dispatch: {:?}",
            evaluation.first_blocking_requirements
        )));
    }
    let input_events = script
        .commands
        .iter()
        .flat_map(|command| command.default_input_events.iter().copied())
        .collect::<Vec<_>>();
    if input_events.is_empty() && matches!(mode, CombatCommandPlaybackMode::SendInput) {
        return Err(TaskError::CombatStrategy(
            "combat script has no static input events to dispatch".to_string(),
        ));
    }
    let mut dispatched_events = 0;
    let mut cancelled = false;
    if matches!(mode, CombatCommandPlaybackMode::SendInput) {
        let result = if let Some(cancellation) = cancellation {
            match send_events_with_cancellation(&input_events, cancellation) {
                Ok(report) => Ok((report.dispatched_events, report.cancelled)),
                Err(InputError::Cancelled {
                    dispatched_events, ..
                }) => Ok((dispatched_events, true)),
                Err(error) => Err(error),
            }
        } else {
            send_events(&input_events).map(|_| (input_events.len(), false))
        };
        let result = result.map_err(|error| TaskError::CombatInputDispatch(error.to_string()))?;
        dispatched_events = result.0;
        cancelled = result.1;
    }
    Ok(CombatCommandPlaybackExecution {
        mode,
        script_name: script.name.clone(),
        total_commands: evaluation.total_commands,
        static_ready_commands: evaluation.static_ready_commands,
        context_bound_commands: evaluation.context_bound_commands,
        input_events,
        dispatched: matches!(mode, CombatCommandPlaybackMode::SendInput),
        dispatched_events,
        cancelled,
    })
}

pub fn plan_team_context_combat_script_playback(
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
) -> Result<CombatTeamPlaybackExecution> {
    let candidate_commands: Vec<_> = script
        .commands
        .iter()
        .filter(|command| {
            executable_commands
                .iter()
                .any(|executable| executable == &command.command)
        })
        .collect();
    let mut planned_commands = Vec::with_capacity(candidate_commands.len());
    let mut input_events = Vec::new();
    let mut blocked_command_index = None;
    let mut blocked_requirements = Vec::new();

    for command in candidate_commands {
        let team_avatar = if command.command.avatar == CURRENT_COMBAT_AVATAR_NAME {
            None
        } else {
            team_plan
                .avatars
                .iter()
                .find(|avatar| avatar.name == command.command.avatar)
        };
        let team_index = team_avatar.map(|avatar| avatar.index);
        let mut switch_events = Vec::new();
        let mut resolved_context = Vec::new();
        let mut pending_context = command.pending_context.clone();
        let mut executable = true;
        let mut message = "command input is ready for known team context".to_string();

        if pending_context.contains(&CombatExecutionContextRequirement::AvatarSelection) {
            if let Some(index) = team_index {
                switch_events = combat_avatar_switch_events(index)?;
                pending_context
                    .retain(|item| item != &CombatExecutionContextRequirement::AvatarSelection);
                resolved_context.push(CombatExecutionContextRequirement::AvatarSelection);
            } else {
                executable = false;
                message = format!(
                    "avatar {} is not available in the configured team",
                    command.command.avatar
                );
            }
        }
        if pending_context.contains(&CombatExecutionContextRequirement::InputEvents)
            && !command.default_input_events.is_empty()
        {
            pending_context.retain(|item| item != &CombatExecutionContextRequirement::InputEvents);
            resolved_context.push(CombatExecutionContextRequirement::InputEvents);
        }
        if !pending_context.is_empty() {
            executable = false;
            message = format!(
                "command still requires native combat context: {:?}",
                pending_context
            );
        }

        let mut command_events = Vec::new();
        if executable {
            command_events.extend(switch_events.iter().copied());
            command_events.extend(command.default_input_events.iter().copied());
            input_events.extend(command_events.iter().copied());
        } else if blocked_command_index.is_none() {
            blocked_command_index = Some(command.index);
            blocked_requirements = pending_context.clone();
        }

        planned_commands.push(CombatTeamPlaybackCommandPlan {
            command_index: command.index,
            avatar: command.command.avatar.clone(),
            team_index,
            switch_events,
            action_events: command.default_input_events.clone(),
            input_events: command_events,
            resolved_context,
            pending_context,
            executable,
            message,
        });
    }

    let playable_commands = planned_commands
        .iter()
        .filter(|command| command.executable)
        .count();
    let dispatch_ready = !planned_commands.is_empty()
        && blocked_command_index.is_none()
        && playable_commands == planned_commands.len()
        && !input_events.is_empty();
    Ok(CombatTeamPlaybackExecution {
        mode: CombatCommandPlaybackMode::PlanOnly,
        script_name: script.name.clone(),
        total_commands: script.commands.len(),
        candidate_commands: planned_commands.len(),
        planned_commands,
        playable_commands,
        blocked_command_index,
        blocked_requirements,
        input_events,
        dispatch_ready,
        dispatched: false,
        dispatched_events: 0,
        cancelled: false,
    })
}

pub fn plan_team_context_combat_script_playback_with_frame(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
) -> Result<CombatTeamPlaybackExecution> {
    let working_directory = working_directory.as_ref();
    let mut execution =
        plan_team_context_combat_script_playback(script, team_plan, executable_commands)?;
    let active_detection =
        detect_active_combat_avatar_index_from_default_rects_with_arrow(working_directory, image)?;
    let active_index = active_detection.active_index;

    for command in &mut execution.planned_commands {
        let target_is_active = command.avatar == CURRENT_COMBAT_AVATAR_NAME
            || command
                .team_index
                .zip(active_index)
                .map(|(target, active)| target == active)
                .unwrap_or(false);

        if target_is_active {
            if !command.switch_events.is_empty() {
                command.switch_events.clear();
                if !command
                    .resolved_context
                    .contains(&CombatExecutionContextRequirement::AvatarSelection)
                {
                    command
                        .resolved_context
                        .push(CombatExecutionContextRequirement::AvatarSelection);
                }
                command.message =
                    "target avatar is already active in the supplied frame; switch input was removed"
                        .to_string();
            }

            if command
                .pending_context
                .contains(&CombatExecutionContextRequirement::SkillCooldown)
            {
                let requested_index = command.team_index.or(active_index).unwrap_or(1);
                let readiness = detect_combat_skill_readiness(
                    working_directory,
                    image,
                    requested_index,
                    false,
                )?;
                if readiness.ready == Some(true) {
                    command
                        .pending_context
                        .retain(|item| item != &CombatExecutionContextRequirement::SkillCooldown);
                    command
                        .resolved_context
                        .push(CombatExecutionContextRequirement::SkillCooldown);
                    command.message =
                        "skill cooldown was resolved from the supplied active-avatar frame"
                            .to_string();
                } else {
                    command.message = format!(
                        "skill cooldown remains pending after frame readiness check: {:?}",
                        readiness.status
                    );
                }
            }
        }

        if command
            .pending_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness)
        {
            let requested_index = command.team_index.or(active_index).unwrap_or(1);
            let readiness =
                detect_combat_skill_readiness(working_directory, image, requested_index, true)?;
            if readiness.ready == Some(true) {
                command
                    .pending_context
                    .retain(|item| item != &CombatExecutionContextRequirement::BurstReadiness);
                command
                    .resolved_context
                    .push(CombatExecutionContextRequirement::BurstReadiness);
                command.message =
                    "burst readiness was resolved from the supplied active-avatar frame"
                        .to_string();
            } else {
                command.message = format!(
                    "burst readiness remains pending after frame readiness check: {:?}",
                    readiness.status
                );
            }
        }

        command.resolved_context.sort();
        command.resolved_context.dedup();
    }
    refresh_team_playback_execution(&mut execution);
    Ok(execution)
}

pub fn execute_team_context_combat_script_inputs(
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
    mode: CombatCommandPlaybackMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<CombatTeamPlaybackExecution> {
    let mut execution =
        plan_team_context_combat_script_playback(script, team_plan, executable_commands)?;
    execution.mode = mode;
    if matches!(mode, CombatCommandPlaybackMode::SendInput) {
        if !execution.dispatch_ready {
            return Err(TaskError::CombatStrategy(format!(
                "team-context combat playback is not dispatch ready; first blocked command: {:?}, requirements: {:?}",
                execution.blocked_command_index, execution.blocked_requirements
            )));
        }
        let result = dispatch_auto_fight_input_events(&execution.input_events, cancellation)?;
        execution.dispatched_events = result.0;
        execution.cancelled = result.1;
    }
    execution.dispatched = matches!(mode, CombatCommandPlaybackMode::SendInput);
    Ok(execution)
}

fn refresh_team_playback_execution(execution: &mut CombatTeamPlaybackExecution) {
    execution.input_events.clear();
    execution.blocked_command_index = None;
    execution.blocked_requirements.clear();
    for command in &mut execution.planned_commands {
        command.executable = command.pending_context.is_empty();
        command.input_events.clear();
        if command.executable {
            command
                .input_events
                .extend(command.switch_events.iter().copied());
            command
                .input_events
                .extend(command.action_events.iter().copied());
            execution
                .input_events
                .extend(command.input_events.iter().copied());
        } else if execution.blocked_command_index.is_none() {
            execution.blocked_command_index = Some(command.command_index);
            execution.blocked_requirements = command.pending_context.clone();
        }
    }
    execution.playable_commands = execution
        .planned_commands
        .iter()
        .filter(|command| command.executable)
        .count();
    execution.dispatch_ready = !execution.planned_commands.is_empty()
        && execution.blocked_command_index.is_none()
        && execution.playable_commands == execution.planned_commands.len()
        && !execution.input_events.is_empty();
}

fn combat_avatar_switch_events(index: usize) -> Result<Vec<InputEvent>> {
    let action = match index {
        1 => GenshinAction::SwitchMember1,
        2 => GenshinAction::SwitchMember2,
        3 => GenshinAction::SwitchMember3,
        4 => GenshinAction::SwitchMember4,
        5 => GenshinAction::SwitchMember5,
        _ => {
            return Err(TaskError::CombatStrategy(format!(
                "combat avatar switch index {index} is outside the supported party range"
            )))
        }
    };
    let bindings = KeyBindingsConfig::default();
    let mut events = combat_action_events(&bindings, GenshinAction::Drop, KeyActionType::KeyPress)?;
    events.extend(combat_action_events(
        &bindings,
        action,
        KeyActionType::KeyPress,
    )?);
    events.push(InputEvent::Delay {
        milliseconds: COMBAT_AVATAR_SWITCH_SETTLE_MILLISECONDS,
    });
    Ok(events)
}

fn combat_command_context_requirements(
    switch_policy: CombatAvatarSwitchPolicy,
    action: &CombatCommandActionPlan,
    default_input_events_empty: bool,
) -> Vec<CombatExecutionContextRequirement> {
    let mut requirements = Vec::new();
    if !matches!(
        switch_policy,
        CombatAvatarSwitchPolicy::CurrentAvatar | CombatAvatarSwitchPolicy::NoSwitch
    ) {
        requirements.push(CombatExecutionContextRequirement::AvatarSelection);
    }
    match action {
        CombatCommandActionPlan::Skill {
            cooldown_policy,
            variant,
            ..
        } => {
            if !matches!(cooldown_policy, CombatSkillCooldownPolicy::None) {
                requirements.push(CombatExecutionContextRequirement::SkillCooldown);
            }
            if matches!(
                variant,
                CombatSkillExecutionVariant::NahidaCameraSweepHold
                    | CombatSkillExecutionVariant::CandaceLongHold
            ) {
                requirements
                    .push(CombatExecutionContextRequirement::CharacterSpecificCameraControl);
            }
        }
        CombatCommandActionPlan::Burst { .. } => {
            requirements.push(CombatExecutionContextRequirement::BurstReadiness);
        }
        CombatCommandActionPlan::Ready { .. } => {
            requirements.push(CombatExecutionContextRequirement::ReadyStateDetection);
        }
        CombatCommandActionPlan::Check {
            handled_by_fight_loop,
        } => {
            if *handled_by_fight_loop {
                requirements.push(CombatExecutionContextRequirement::FightLoopFinishDetection);
            }
        }
        CombatCommandActionPlan::Charge { variant, .. }
            if !matches!(variant, CombatChargeExecutionVariant::GenericHold) =>
        {
            requirements.push(CombatExecutionContextRequirement::CharacterSpecificCameraControl);
        }
        _ => {}
    }
    if default_input_events_empty
        && !matches!(
            action,
            CombatCommandActionPlan::Check { .. } | CombatCommandActionPlan::Ready { .. }
        )
    {
        requirements.push(CombatExecutionContextRequirement::InputEvents);
    }
    requirements.sort();
    requirements.dedup();
    requirements
}

fn combat_avatar_switch_policy(
    command: &CombatCommandPlan,
    previous: Option<&CombatCommandPlan>,
) -> CombatAvatarSwitchPolicy {
    if command.avatar == CURRENT_COMBAT_AVATAR_NAME {
        return CombatAvatarSwitchPolicy::CurrentAvatar;
    }
    if previous
        .map(|previous| previous.avatar != command.avatar)
        .unwrap_or(false)
    {
        return CombatAvatarSwitchPolicy::SwitchOnAvatarChange;
    }
    if combat_command_skips_avatar_switch(command.method) {
        CombatAvatarSwitchPolicy::NoSwitch
    } else {
        CombatAvatarSwitchPolicy::EnsureSelectedBeforeAction
    }
}

fn combat_command_skips_avatar_switch(method: CombatCommandMethod) -> bool {
    matches!(
        method,
        CombatCommandMethod::Wait
            | CombatCommandMethod::Ready
            | CombatCommandMethod::MouseDown
            | CombatCommandMethod::MouseUp
            | CombatCommandMethod::Click
            | CombatCommandMethod::MoveBy
            | CombatCommandMethod::KeyDown
            | CombatCommandMethod::KeyUp
            | CombatCommandMethod::KeyPress
            | CombatCommandMethod::Scroll
    )
}

fn combat_command_action_plan(command: &CombatCommandPlan) -> Result<CombatCommandActionPlan> {
    match command.method {
        CombatCommandMethod::Skill => {
            let options: Vec<String> = command.args.iter().map(|arg| arg.to_lowercase()).collect();
            let hold = options.iter().any(|arg| arg == "hold");
            let cooldown_policy = if options.iter().any(|arg| arg == "fast") {
                CombatSkillCooldownPolicy::FastSkipIfCoolingDown
            } else if options.iter().any(|arg| arg == "wait") {
                CombatSkillCooldownPolicy::WaitUntilReady
            } else {
                CombatSkillCooldownPolicy::None
            };
            let variant = if !hold {
                CombatSkillExecutionVariant::Tap
            } else {
                match command.avatar.as_str() {
                    "纳西妲" => CombatSkillExecutionVariant::NahidaCameraSweepHold,
                    "坎蒂丝" => CombatSkillExecutionVariant::CandaceLongHold,
                    _ => CombatSkillExecutionVariant::GenericHold,
                }
            };
            Ok(CombatCommandActionPlan::Skill {
                hold,
                variant,
                cooldown_policy,
                options,
            })
        }
        CombatCommandMethod::Burst => Ok(CombatCommandActionPlan::Burst {
            requires_readiness_check: true,
        }),
        CombatCommandMethod::Attack => {
            let duration_ms = optional_duration_ms(&command.args, 0, 0)?;
            Ok(CombatCommandActionPlan::Attack {
                duration_ms,
                click_interval_ms: COMBAT_ATTACK_INTERVAL_MILLISECONDS,
                repeat_count: combat_attack_repeat_count(duration_ms),
            })
        }
        CombatCommandMethod::Charge => {
            let duration_ms =
                optional_duration_ms(&command.args, 0, COMBAT_DEFAULT_CHARGE_MILLISECONDS)?;
            let variant = match command.avatar.as_str() {
                "那维莱特" => CombatChargeExecutionVariant::NeuvilletteCameraSweep,
                "恰斯卡" => CombatChargeExecutionVariant::ChascaCameraSweep,
                _ => CombatChargeExecutionVariant::GenericHold,
            };
            Ok(CombatCommandActionPlan::Charge {
                duration_ms,
                variant,
            })
        }
        CombatCommandMethod::Walk => Ok(CombatCommandActionPlan::Walk {
            direction: command.args[0].trim().to_ascii_lowercase(),
            duration_ms: duration_ms_from_seconds(&command.args[1], "walk duration")?,
        }),
        CombatCommandMethod::W
        | CombatCommandMethod::A
        | CombatCommandMethod::S
        | CombatCommandMethod::D => Ok(CombatCommandActionPlan::Walk {
            direction: match command.method {
                CombatCommandMethod::W => "w",
                CombatCommandMethod::A => "a",
                CombatCommandMethod::S => "s",
                CombatCommandMethod::D => "d",
                _ => unreachable!(),
            }
            .to_string(),
            duration_ms: duration_ms_from_seconds(&command.args[0], "walk duration")?,
        }),
        CombatCommandMethod::Wait => {
            let Some(seconds) = command.args.first() else {
                return Err(TaskError::CombatStrategy(
                    "wait command requires one duration argument".to_string(),
                ));
            };
            Ok(CombatCommandActionPlan::Wait {
                duration_ms: duration_ms_from_seconds(seconds, "wait duration")?,
            })
        }
        CombatCommandMethod::Ready => Ok(CombatCommandActionPlan::Ready {
            initial_delay_ms: COMBAT_READY_INITIAL_DELAY_MILLISECONDS,
            poll_count: COMBAT_READY_POLL_COUNT,
            poll_interval_ms: COMBAT_READY_POLL_INTERVAL_MILLISECONDS,
        }),
        CombatCommandMethod::Check => Ok(CombatCommandActionPlan::Check {
            handled_by_fight_loop: true,
        }),
        CombatCommandMethod::Dash => Ok(CombatCommandActionPlan::Dash {
            duration_ms: optional_duration_ms(&command.args, 0, COMBAT_DEFAULT_DASH_MILLISECONDS)?,
        }),
        CombatCommandMethod::Jump => Ok(CombatCommandActionPlan::Jump),
        CombatCommandMethod::MouseDown => Ok(CombatCommandActionPlan::MouseDown {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::MouseUp => Ok(CombatCommandActionPlan::MouseUp {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::Click => Ok(CombatCommandActionPlan::Click {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::MoveBy => Ok(CombatCommandActionPlan::MoveBy {
            x: parse_i32_arg(&command.args[0], "moveby x")?,
            y: parse_i32_arg(&command.args[1], "moveby y")?,
        }),
        CombatCommandMethod::KeyDown => Ok(CombatCommandActionPlan::KeyDown {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::KeyUp => Ok(CombatCommandActionPlan::KeyUp {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::KeyPress => Ok(CombatCommandActionPlan::KeyPress {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::Scroll => Ok(CombatCommandActionPlan::Scroll {
            clicks: parse_i32_arg(&command.args[0], "scroll amount")?,
        }),
        CombatCommandMethod::Round => Ok(CombatCommandActionPlan::Check {
            handled_by_fight_loop: false,
        }),
    }
}

fn default_input_events_for_combat_action(
    action: &CombatCommandActionPlan,
) -> Result<Vec<InputEvent>> {
    let bindings = KeyBindingsConfig::default();
    let mut events = Vec::new();
    match action {
        CombatCommandActionPlan::Skill { variant, .. } => match variant {
            CombatSkillExecutionVariant::Tap => events.extend(combat_action_events(
                &bindings,
                GenshinAction::ElementalSkill,
                KeyActionType::KeyPress,
            )?),
            CombatSkillExecutionVariant::GenericHold => events.extend(combat_action_events(
                &bindings,
                GenshinAction::ElementalSkill,
                KeyActionType::Hold,
            )?),
            CombatSkillExecutionVariant::NahidaCameraSweepHold => {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyDown,
                )?);
                events.push(InputEvent::Delay { milliseconds: 300 });
                for _ in 0..10 {
                    events.push(InputEvent::MouseMoveRelative { dx: 1000, dy: 0 });
                    events.push(InputEvent::Delay { milliseconds: 50 });
                }
                events.push(InputEvent::Delay { milliseconds: 300 });
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyUp,
                )?);
            }
            CombatSkillExecutionVariant::CandaceLongHold => {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyDown,
                )?);
                events.push(InputEvent::Delay {
                    milliseconds: 3_000,
                });
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyUp,
                )?);
            }
        },
        CombatCommandActionPlan::Burst { .. } => events.extend(combat_action_events(
            &bindings,
            GenshinAction::ElementalBurst,
            KeyActionType::KeyPress,
        )?),
        CombatCommandActionPlan::Attack {
            click_interval_ms,
            repeat_count,
            ..
        } => {
            for _ in 0..*repeat_count {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::NormalAttack,
                    KeyActionType::KeyPress,
                )?);
                events.push(InputEvent::Delay {
                    milliseconds: *click_interval_ms,
                });
            }
        }
        CombatCommandActionPlan::Charge {
            duration_ms,
            variant: CombatChargeExecutionVariant::GenericHold,
        } => {
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::NormalAttack,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::NormalAttack,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Charge { .. } => {}
        CombatCommandActionPlan::Walk {
            direction,
            duration_ms,
        } => {
            let action = walk_direction_action(direction)?;
            events.extend(combat_action_events(
                &bindings,
                action,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                action,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Wait { duration_ms } => events.push(InputEvent::Delay {
            milliseconds: *duration_ms,
        }),
        CombatCommandActionPlan::Ready {
            initial_delay_ms, ..
        } => events.push(InputEvent::Delay {
            milliseconds: *initial_delay_ms,
        }),
        CombatCommandActionPlan::Check { .. } => {}
        CombatCommandActionPlan::Dash { duration_ms } => {
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::SprintMouse,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::SprintMouse,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Jump => events.extend(combat_action_events(
            &bindings,
            GenshinAction::Jump,
            KeyActionType::KeyPress,
        )?),
        CombatCommandActionPlan::MouseDown { button } => {
            events.push(InputEvent::MouseButtonDown {
                button: button.button,
            });
        }
        CombatCommandActionPlan::MouseUp { button } => {
            events.push(InputEvent::MouseButtonUp {
                button: button.button,
            });
        }
        CombatCommandActionPlan::Click { button } => {
            events.push(InputEvent::MouseButtonDown {
                button: button.button,
            });
            events.push(InputEvent::MouseButtonUp {
                button: button.button,
            });
        }
        CombatCommandActionPlan::MoveBy { x, y } => {
            events.push(InputEvent::MouseMoveRelative { dx: *x, dy: *y });
        }
        CombatCommandActionPlan::KeyDown { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyDown)?);
        }
        CombatCommandActionPlan::KeyUp { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyUp)?);
        }
        CombatCommandActionPlan::KeyPress { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyPress)?);
        }
        CombatCommandActionPlan::Scroll { clicks } => {
            events.push(InputEvent::MouseWheel {
                amount: *clicks * 120,
                horizontal: false,
            });
        }
    }
    Ok(events)
}

fn combat_action_events(
    bindings: &KeyBindingsConfig,
    action: GenshinAction,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    input_events_for_action(bindings, action, action_type)
        .map_err(|error| TaskError::CombatStrategy(error.to_string()))
}

fn combat_virtual_key_events(
    key: &CombatVirtualKeyPlan,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    if let Some(action) = key.mapped_action {
        return combat_action_events(&KeyBindingsConfig::default(), action, action_type);
    }
    if let Some(button) = key.mouse_button {
        let events = match action_type {
            KeyActionType::KeyDown => vec![InputEvent::MouseButtonDown { button }],
            KeyActionType::KeyUp => vec![InputEvent::MouseButtonUp { button }],
            KeyActionType::KeyPress => vec![
                InputEvent::MouseButtonDown { button },
                InputEvent::MouseButtonUp { button },
            ],
            KeyActionType::Hold => vec![
                InputEvent::MouseButtonDown { button },
                InputEvent::Delay {
                    milliseconds: COMBAT_DEFAULT_CHARGE_MILLISECONDS,
                },
                InputEvent::MouseButtonUp { button },
            ],
        };
        return Ok(events);
    }
    let events = match action_type {
        KeyActionType::KeyDown => vec![InputEvent::KeyDown {
            vk: key.vk,
            extended: None,
        }],
        KeyActionType::KeyUp => vec![InputEvent::KeyUp {
            vk: key.vk,
            extended: None,
        }],
        KeyActionType::KeyPress => vec![
            InputEvent::KeyDown {
                vk: key.vk,
                extended: None,
            },
            InputEvent::KeyUp {
                vk: key.vk,
                extended: None,
            },
        ],
        KeyActionType::Hold => vec![
            InputEvent::KeyDown {
                vk: key.vk,
                extended: None,
            },
            InputEvent::Delay {
                milliseconds: COMBAT_DEFAULT_CHARGE_MILLISECONDS,
            },
            InputEvent::KeyUp {
                vk: key.vk,
                extended: None,
            },
        ],
    };
    Ok(events)
}

fn walk_direction_action(direction: &str) -> Result<GenshinAction> {
    match direction.trim().to_ascii_lowercase().as_str() {
        "w" => Ok(GenshinAction::MoveForward),
        "s" => Ok(GenshinAction::MoveBackward),
        "a" => Ok(GenshinAction::MoveLeft),
        "d" => Ok(GenshinAction::MoveRight),
        other => Err(TaskError::CombatStrategy(format!(
            "unsupported walk direction: {other}"
        ))),
    }
}

fn combat_attack_repeat_count(duration_ms: u64) -> u32 {
    (duration_ms / COMBAT_ATTACK_INTERVAL_MILLISECONDS + 1) as u32
}

fn optional_duration_ms(args: &[String], index: usize, default_ms: u64) -> Result<u64> {
    match args.get(index) {
        Some(value) if !value.trim().is_empty() => duration_ms_from_seconds(value, "duration"),
        _ => Ok(default_ms),
    }
}

fn duration_ms_from_seconds(value: &str, label: &str) -> Result<u64> {
    let seconds = parse_f64_arg(value, label)?;
    if seconds < 0.0 {
        return Err(TaskError::CombatStrategy(format!(
            "{label} must be non-negative"
        )));
    }
    Ok((seconds * 1000.0) as u64)
}

fn combat_mouse_button_plan(value: Option<&str>) -> Result<CombatMouseButtonPlan> {
    let raw = value.unwrap_or("left").trim();
    let button = match raw.to_ascii_lowercase().as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        other => {
            return Err(TaskError::CombatStrategy(format!(
                "unsupported mouse button: {other}"
            )));
        }
    };
    Ok(CombatMouseButtonPlan {
        raw: raw.to_string(),
        button,
    })
}

fn combat_virtual_key_plan(value: &str) -> Result<CombatVirtualKeyPlan> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(TaskError::CombatStrategy(
            "virtual key argument must not be empty".to_string(),
        ));
    }
    let raw_normalized = raw.to_ascii_uppercase().replace(' ', "_");
    let normalized = raw_normalized
        .strip_prefix("VK_")
        .unwrap_or(&raw_normalized)
        .to_string();
    let (vk, mouse_button) = virtual_key_code_and_mouse_button(&normalized).ok_or_else(|| {
        TaskError::CombatStrategy(format!("invalid virtual key argument: {value}"))
    })?;
    Ok(CombatVirtualKeyPlan {
        raw: raw.to_string(),
        vk,
        mouse_button,
        mapped_action: mapped_genshin_action_for_virtual_key(&normalized),
    })
}

fn virtual_key_code_and_mouse_button(key: &str) -> Option<(u16, Option<MouseButton>)> {
    if key.len() == 1 {
        let ch = key.chars().next()?;
        if ch.is_ascii_alphabetic() || ch.is_ascii_digit() {
            return Some((ch as u16, None));
        }
    }
    if let Some(index) = key
        .strip_prefix('F')
        .and_then(|value| value.parse::<u16>().ok())
    {
        if (1..=24).contains(&index) {
            return Some((0x70 + index - 1, None));
        }
    }
    if let Some(index) = key
        .strip_prefix("NUMPAD")
        .and_then(|value| value.parse::<u16>().ok())
    {
        if index <= 9 {
            return Some((0x60 + index, None));
        }
    }
    let result = match key {
        "LBUTTON" => (KeyId::MOUSE_LEFT_BUTTON.vk(), Some(MouseButton::Left)),
        "RBUTTON" => (KeyId::MOUSE_RIGHT_BUTTON.vk(), Some(MouseButton::Right)),
        "MBUTTON" => (KeyId::MOUSE_MIDDLE_BUTTON.vk(), Some(MouseButton::Middle)),
        "XBUTTON1" => (KeyId::MOUSE_SIDE_BUTTON1.vk(), Some(MouseButton::X(1))),
        "XBUTTON2" => (KeyId::MOUSE_SIDE_BUTTON2.vk(), Some(MouseButton::X(2))),
        "SHIFT" | "LSHIFT" | "LEFT_SHIFT" => (KeyId::LEFT_SHIFT.vk(), None),
        "RSHIFT" | "RIGHT_SHIFT" => (KeyId::RIGHT_SHIFT.vk(), None),
        "CONTROL" | "CTRL" | "LCONTROL" | "LCTRL" | "LEFT_CONTROL" | "LEFT_CTRL" => {
            (KeyId::LEFT_CTRL.vk(), None)
        }
        "RCONTROL" | "RCTRL" | "RIGHT_CONTROL" | "RIGHT_CTRL" => (KeyId::RIGHT_CTRL.vk(), None),
        "MENU" | "ALT" | "LMENU" | "LALT" | "LEFT_ALT" => (KeyId::LEFT_ALT.vk(), None),
        "RMENU" | "RALT" | "RIGHT_ALT" => (KeyId::RIGHT_ALT.vk(), None),
        "LWIN" | "LEFT_WIN" => (KeyId::LEFT_WIN.vk(), None),
        "RWIN" | "RIGHT_WIN" => (KeyId::RIGHT_WIN.vk(), None),
        "SPACE" => (KeyId::SPACE.vk(), None),
        "RETURN" | "ENTER" => (KeyId::ENTER.vk(), None),
        "ESCAPE" | "ESC" => (KeyId::ESCAPE.vk(), None),
        "TAB" => (KeyId::TAB.vk(), None),
        "BACK" | "BACKSPACE" => (KeyId::BACKSPACE.vk(), None),
        "INSERT" => (KeyId::INSERT.vk(), None),
        "DELETE" | "DEL" => (KeyId::DELETE.vk(), None),
        "HOME" => (KeyId::HOME.vk(), None),
        "END" => (KeyId::END.vk(), None),
        "PRIOR" | "PAGE_UP" | "PAGEUP" => (KeyId::PAGE_UP.vk(), None),
        "NEXT" | "PAGE_DOWN" | "PAGEDOWN" => (KeyId::PAGE_DOWN.vk(), None),
        "LEFT" => (KeyId::LEFT.vk(), None),
        "UP" => (KeyId::UP.vk(), None),
        "RIGHT" => (KeyId::RIGHT.vk(), None),
        "DOWN" => (KeyId::DOWN.vk(), None),
        "CAPITAL" | "CAPSLOCK" | "CAPS_LOCK" => (KeyId::CAPS_LOCK.vk(), None),
        "SCROLL" | "SCROLLLOCK" | "SCROLL_LOCK" => (KeyId::SCROLL_LOCK.vk(), None),
        "PAUSE" => (KeyId::PAUSE.vk(), None),
        "PRINT" | "SNAPSHOT" | "PRINT_SCREEN" => (KeyId::PRINT_SCREEN.vk(), None),
        "APPS" => (KeyId::APPS.vk(), None),
        "DECIMAL" => (KeyId::DECIMAL.vk(), None),
        "DIVIDE" => (KeyId::DIVIDE.vk(), None),
        "MULTIPLY" => (KeyId::MULTIPLY.vk(), None),
        "SUBTRACT" => (KeyId::SUBTRACT.vk(), None),
        "ADD" => (KeyId::ADD.vk(), None),
        "OEM_PLUS" | "PLUS" | "EQUAL" => (KeyId::EQUAL.vk(), None),
        "OEM_MINUS" | "MINUS" => (KeyId::MINUS.vk(), None),
        "OEM_COMMA" | "COMMA" => (KeyId::COMMA.vk(), None),
        "OEM_PERIOD" | "PERIOD" => (KeyId::PERIOD.vk(), None),
        "OEM_1" | "SEMICOLON" => (KeyId::SEMICOLON.vk(), None),
        "OEM_2" | "SLASH" => (KeyId::SLASH.vk(), None),
        "OEM_3" | "TILDE" => (KeyId::TILDE.vk(), None),
        "OEM_4" | "LEFT_SQUARE_BRACKET" => (KeyId::LEFT_SQUARE_BRACKET.vk(), None),
        "OEM_6" | "RIGHT_SQUARE_BRACKET" => (KeyId::RIGHT_SQUARE_BRACKET.vk(), None),
        "OEM_7" | "APOSTROPHE" => (KeyId::APOSTROPHE.vk(), None),
        "OEM_102" | "BACKSLASH" => (KeyId::BACKSLASH.vk(), None),
        _ => return None,
    };
    Some(result)
}

fn mapped_genshin_action_for_virtual_key(key: &str) -> Option<GenshinAction> {
    match key {
        "W" => Some(GenshinAction::MoveForward),
        "S" => Some(GenshinAction::MoveBackward),
        "A" => Some(GenshinAction::MoveLeft),
        "D" => Some(GenshinAction::MoveRight),
        "LCONTROL" | "LCTRL" | "LEFT_CONTROL" | "LEFT_CTRL" => {
            Some(GenshinAction::SwitchToWalkOrRun)
        }
        "E" => Some(GenshinAction::ElementalSkill),
        "Q" => Some(GenshinAction::ElementalBurst),
        "LSHIFT" | "LEFT_SHIFT" => Some(GenshinAction::SprintKeyboard),
        "R" => Some(GenshinAction::SwitchAimingMode),
        "SPACE" => Some(GenshinAction::Jump),
        "X" => Some(GenshinAction::Drop),
        "F" => Some(GenshinAction::PickUpOrInteract),
        "Z" => Some(GenshinAction::QuickUseGadget),
        "T" => Some(GenshinAction::InteractionInSomeMode),
        "V" => Some(GenshinAction::QuestNavigation),
        "P" => Some(GenshinAction::AbandonChallenge),
        "1" => Some(GenshinAction::SwitchMember1),
        "2" => Some(GenshinAction::SwitchMember2),
        "3" => Some(GenshinAction::SwitchMember3),
        "4" => Some(GenshinAction::SwitchMember4),
        "5" => Some(GenshinAction::SwitchMember5),
        "TAB" => Some(GenshinAction::ShortcutWheel),
        "B" => Some(GenshinAction::OpenInventory),
        "C" => Some(GenshinAction::OpenCharacterScreen),
        "M" => Some(GenshinAction::OpenMap),
        "ESCAPE" | "ESC" => Some(GenshinAction::OpenPaimonMenu),
        "F1" => Some(GenshinAction::OpenAdventurerHandbook),
        "F2" => Some(GenshinAction::OpenCoOpScreen),
        "F3" => Some(GenshinAction::OpenWishScreen),
        "F4" => Some(GenshinAction::OpenBattlePassScreen),
        "F5" => Some(GenshinAction::OpenTheEventsMenu),
        "F6" => Some(GenshinAction::OpenTheSettingsMenu),
        "F7" => Some(GenshinAction::OpenTheFurnishingScreen),
        "F8" => Some(GenshinAction::OpenStellarReunion),
        "J" => Some(GenshinAction::OpenQuestMenu),
        "Y" => Some(GenshinAction::OpenNotificationDetails),
        "RETURN" | "ENTER" => Some(GenshinAction::OpenChatScreen),
        "U" => Some(GenshinAction::OpenSpecialEnvironmentInformation),
        "G" => Some(GenshinAction::CheckTutorialDetails),
        "LMENU" | "LALT" | "LEFT_ALT" => Some(GenshinAction::ShowCursor),
        "L" => Some(GenshinAction::OpenPartySetupScreen),
        "O" => Some(GenshinAction::OpenFriendsScreen),
        "OEM_2" | "SLASH" => Some(GenshinAction::HideUi),
        _ => None,
    }
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

fn parse_combat_script_file(
    path: &Path,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptPlan> {
    let context =
        fs::read_to_string(path).map_err(|error| TaskError::CombatStrategy(error.to_string()))?;
    let mut script = parse_combat_script_context_with_catalog(&context, true, catalog)?;
    script.path = Some(path.to_path_buf());
    script.name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(script)
}

fn combat_script_logical_lines(context: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in context.lines() {
        let line = line
            .trim()
            .replace('（', "(")
            .replace('）', ")")
            .replace('，', ",");
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if line.contains(';') {
            result.extend(
                line.split(';')
                    .filter_map(|part| {
                        let part = part.trim();
                        (!part.is_empty()).then(|| part.to_string())
                    })
                    .collect::<Vec<_>>(),
            );
        } else {
            result.push(line);
        }
    }
    result
}

fn parse_combat_script_lines(
    lines: Vec<String>,
    validate: bool,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<CombatScriptPlan> {
    let mut commands = Vec::new();
    let mut avatar_names = Vec::new();
    for line in lines {
        let mut line_commands = parse_combat_script_line(&line, validate, catalog)?;
        for command in &line_commands {
            if !avatar_names.contains(&command.avatar) {
                avatar_names.push(command.avatar.clone());
            }
        }
        commands.append(&mut line_commands);
    }
    Ok(CombatScriptPlan {
        name: String::new(),
        path: None,
        avatar_names,
        commands,
    })
}

fn parse_combat_script_line(
    line: &str,
    validate: bool,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<Vec<CombatCommandPlan>> {
    let line = line.trim();
    let first_space_index = line.find(' ');
    let (avatar, commands) = match first_space_index {
        Some(index) if index > 0 => {
            let avatar = standardize_combat_script_avatar_name(line[..index].trim(), catalog)?;
            (avatar, line[index + 1..].trim())
        }
        _ if validate => {
            return Err(TaskError::CombatStrategy(
                "combat script line must separate avatar and commands with a space".to_string(),
            ));
        }
        _ => ("当前角色".to_string(), line),
    };
    let mut full_commands = Vec::new();
    for part in commands.split('|').filter(|part| !part.trim().is_empty()) {
        let mut part_commands = parse_combat_script_line_part(part, &avatar)?;
        if part_commands
            .first()
            .map(|command| command.method == CombatCommandMethod::Round)
            .unwrap_or(false)
        {
            let activating_rounds = parse_round_command(&part_commands[0])?;
            part_commands.remove(0);
            for command in &mut part_commands {
                command.activating_rounds = activating_rounds.clone();
            }
        }
        full_commands.extend(part_commands);
    }
    Ok(full_commands)
}

fn standardize_combat_script_avatar_name(
    avatar: &str,
    catalog: Option<&CombatAvatarCatalog>,
) -> Result<String> {
    let avatar = avatar.trim();
    if avatar == CURRENT_COMBAT_AVATAR_NAME {
        return Ok(CURRENT_COMBAT_AVATAR_NAME.to_string());
    }
    match catalog {
        Some(catalog) => catalog.standard_name_for_alias(avatar),
        None => Ok(avatar.to_string()),
    }
}

fn parse_combat_script_line_part(part: &str, avatar: &str) -> Result<Vec<CombatCommandPlan>> {
    let command_array: Vec<&str> = part
        .split(',')
        .filter(|command| !command.is_empty())
        .collect();
    let mut commands = Vec::new();
    let mut index = 0;
    while index < command_array.len() {
        let mut command = command_array[index].trim().to_string();
        if command.contains('(') && !command.contains(')') {
            let mut next = index + 1;
            while next < command_array.len() {
                command.push(',');
                command.push_str(command_array[next]);
                if command.matches('(').count() > 1 {
                    return Err(TaskError::CombatStrategy(format!(
                        "combat command has unpaired parentheses: {command}"
                    )));
                }
                if command.contains(')') {
                    index = next;
                    break;
                }
                next += 1;
            }
            if !(command.contains('(') && command.contains(')')) {
                return Err(TaskError::CombatStrategy(format!(
                    "combat command has incomplete parentheses: {command}"
                )));
            }
        }
        commands.push(parse_combat_command(avatar, &command)?);
        index += 1;
    }
    Ok(commands)
}

fn parse_combat_command(avatar: &str, raw_command: &str) -> Result<CombatCommandPlan> {
    let command = raw_command.trim();
    let (method_code, args) = if let Some(start_index) = command.find('(') {
        if start_index == 0 {
            return Err(TaskError::CombatStrategy(format!(
                "combat command is missing method name: {command}"
            )));
        }
        let Some(end_index) = command.find(')') else {
            return Err(TaskError::CombatStrategy(format!(
                "combat command has incomplete parentheses: {command}"
            )));
        };
        let parameters = &command[start_index + 1..end_index];
        (
            command[..start_index].trim(),
            parameters
                .split(',')
                .map(str::trim)
                .map(ToOwned::to_owned)
                .collect(),
        )
    } else {
        (command.trim(), Vec::new())
    };
    let method = CombatCommandMethod::from_code(method_code).ok_or_else(|| {
        TaskError::CombatStrategy(format!("unknown combat strategy method: {method_code}"))
    })?;
    validate_combat_command_args(method, &args)?;
    Ok(CombatCommandPlan {
        avatar: avatar.trim().to_string(),
        method,
        args,
        activating_rounds: Vec::new(),
        raw: command.to_string(),
    })
}

fn parse_round_command(command: &CombatCommandPlan) -> Result<Vec<u32>> {
    if command.args.is_empty() {
        return Err(TaskError::CombatStrategy(
            "round command requires at least one argument".to_string(),
        ));
    }
    let mut rounds = Vec::new();
    for arg in &command.args {
        if let Some((start, end)) = arg.split_once('-') {
            let start = parse_positive_round(start)?;
            let end = parse_positive_round(end)?;
            if start > end {
                return Err(TaskError::CombatStrategy(
                    "round range start must be less than or equal to end".to_string(),
                ));
            }
            rounds.extend(start..=end);
        } else {
            rounds.push(parse_positive_round(arg)?);
        }
    }
    Ok(rounds)
}

fn parse_positive_round(value: &str) -> Result<u32> {
    let round: u32 = value
        .trim()
        .parse()
        .map_err(|_| TaskError::CombatStrategy(format!("invalid round value: {value}")))?;
    if round == 0 {
        return Err(TaskError::CombatStrategy(
            "round value must be greater than zero".to_string(),
        ));
    }
    Ok(round)
}

fn validate_combat_command_args(method: CombatCommandMethod, args: &[String]) -> Result<()> {
    match method {
        CombatCommandMethod::Walk => {
            if args.len() != 2 {
                return Err(TaskError::CombatStrategy(
                    "walk command requires direction and duration".to_string(),
                ));
            }
            parse_positive_f64(&args[1], "walk duration")?;
        }
        CombatCommandMethod::W
        | CombatCommandMethod::A
        | CombatCommandMethod::S
        | CombatCommandMethod::D => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "w/a/s/d command requires one duration argument".to_string(),
                ));
            }
            parse_f64_arg(&args[0], "walk duration")?;
        }
        CombatCommandMethod::MoveBy => {
            if args.len() != 2 {
                return Err(TaskError::CombatStrategy(
                    "moveby command requires x and y arguments".to_string(),
                ));
            }
            parse_i32_arg(&args[0], "moveby x")?;
            parse_i32_arg(&args[1], "moveby y")?;
        }
        CombatCommandMethod::KeyDown
        | CombatCommandMethod::KeyUp
        | CombatCommandMethod::KeyPress => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "key command requires one key argument".to_string(),
                ));
            }
            validate_virtual_key_name(&args[0])?;
        }
        CombatCommandMethod::Scroll => {
            if args.len() != 1 {
                return Err(TaskError::CombatStrategy(
                    "scroll command requires one integer argument".to_string(),
                ));
            }
            parse_i32_arg(&args[0], "scroll amount")?;
        }
        _ => {}
    }
    Ok(())
}

fn parse_positive_f64(value: &str, label: &str) -> Result<f64> {
    let parsed = parse_f64_arg(value, label)?;
    if parsed <= 0.0 {
        return Err(TaskError::CombatStrategy(format!(
            "{label} must be positive"
        )));
    }
    Ok(parsed)
}

fn parse_f64_arg(value: &str, label: &str) -> Result<f64> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| TaskError::CombatStrategy(format!("{label} must be a number: {value}")))
}

fn parse_i32_arg(value: &str, label: &str) -> Result<i32> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| TaskError::CombatStrategy(format!("{label} must be an integer: {value}")))
}

fn validate_virtual_key_name(value: &str) -> Result<()> {
    combat_virtual_key_plan(value).map(|_| ())
}

fn collect_txt_files(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_txt_files(&path, files)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("txt"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn normalize_user_auto_fight_strategy_path(strategy_path: &str) -> Result<PathBuf> {
    let strategy_path = strategy_path.trim().replace('\\', "/");
    if strategy_path.is_empty() {
        return Err(TaskError::EmptyCombatStrategyPath);
    }
    let path = PathBuf::from(&strategy_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    if !strategy_path
        .split('/')
        .next()
        .map(|first| first.eq_ignore_ascii_case("User"))
        .unwrap_or(false)
    {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    let mut components = path.components();
    let _ = components.next();
    let Some(second) = components.next() else {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    };
    if second.as_os_str() != "AutoFight" {
        return Err(TaskError::InvalidCombatStrategyPath(strategy_path));
    }
    Ok(path)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoSkipConfigParam {
    pub enabled: bool,
    pub quickly_skip_conversations_enabled: bool,
    pub after_choose_option_sleep_delay: i32,
    pub auto_wait_dialogue_option_voice_enabled: bool,
    pub dialogue_option_voice_max_wait_seconds: i32,
    pub before_click_confirm_delay: i32,
    pub auto_get_daily_rewards_enabled: bool,
    pub auto_re_explore_enabled: bool,
    pub click_chat_option: String,
    pub custom_priority_options_enabled: bool,
    pub custom_priority_options: String,
    pub auto_hangout_event_enabled: bool,
    pub auto_hangout_end_choose: String,
    pub auto_hangout_choose_option_sleep_delay: i32,
    pub auto_hangout_press_skip_enabled: bool,
    pub run_background_enabled: bool,
    pub bring_game_to_front_after_background_dialog_enabled: bool,
    pub submit_goods_enabled: bool,
    pub picture_in_picture_enabled: bool,
    pub picture_in_picture_source_type: String,
    pub close_popup_paged_enabled: bool,
    pub skip_built_in_click_options: bool,
}

impl Default for AutoSkipConfigParam {
    fn default() -> Self {
        Self {
            enabled: true,
            quickly_skip_conversations_enabled: true,
            after_choose_option_sleep_delay: 0,
            auto_wait_dialogue_option_voice_enabled: false,
            dialogue_option_voice_max_wait_seconds: 30,
            before_click_confirm_delay: 0,
            auto_get_daily_rewards_enabled: true,
            auto_re_explore_enabled: true,
            click_chat_option: "优先选择第一个选项".to_string(),
            custom_priority_options_enabled: false,
            custom_priority_options: String::new(),
            auto_hangout_event_enabled: false,
            auto_hangout_end_choose: String::new(),
            auto_hangout_choose_option_sleep_delay: 0,
            auto_hangout_press_skip_enabled: true,
            run_background_enabled: false,
            bring_game_to_front_after_background_dialog_enabled: false,
            submit_goods_enabled: true,
            picture_in_picture_enabled: false,
            picture_in_picture_source_type: "CaptureLoop".to_string(),
            close_popup_paged_enabled: true,
            skip_built_in_click_options: false,
        }
    }
}

impl AutoSkipConfigParam {
    pub fn is_click_first_chat_option(&self) -> bool {
        self.click_chat_option == "优先选择第一个选项"
    }

    pub fn is_click_random_chat_option(&self) -> bool {
        self.click_chat_option == "随机选择选项"
    }

    pub fn is_click_none_chat_option(&self) -> bool {
        self.click_chat_option == "不选择选项"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoDomainParam {
    pub domain_round_num: i32,
    pub combat_strategy_path: String,
    pub party_name: String,
    pub domain_name: String,
    pub sunday_selected_value: String,
    pub auto_artifact_salvage: bool,
    pub max_artifact_star: String,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: i32,
    pub original_resin20_use_count: i32,
    pub original_resin40_use_count: i32,
    pub condensed_resin_use_count: i32,
    pub transient_resin_use_count: i32,
    pub fragile_resin_use_count: i32,
    pub reward_recognition_enabled: bool,
}

impl Default for AutoDomainParam {
    fn default() -> Self {
        Self::new(0, None)
    }
}

impl AutoDomainParam {
    pub fn new(domain_round_num: i32, strategy_name: Option<&str>) -> Self {
        Self {
            domain_round_num: if domain_round_num == 0 {
                9999
            } else {
                domain_round_num
            },
            combat_strategy_path: combat_strategy_path(strategy_name),
            party_name: String::new(),
            domain_name: String::new(),
            sunday_selected_value: String::new(),
            auto_artifact_salvage: false,
            max_artifact_star: "4".to_string(),
            specify_resin_use: false,
            resin_priority_list: default_resin_priority(),
            original_resin_use_count: 0,
            original_resin20_use_count: 0,
            original_resin40_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            reward_recognition_enabled: false,
        }
    }

    pub fn set_resin_priority_list(
        &mut self,
        priorities: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.resin_priority_list = priorities.into_iter().map(Into::into).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoBossParam {
    pub boss_name: String,
    pub strategy_name: String,
    pub combat_strategy_path: String,
    pub team_name: String,
    pub specify_run_count: bool,
    pub run_count: i32,
    pub use_transient_resin: bool,
    pub use_fragile_resin: bool,
    pub revive_retry_count: i32,
    pub return_to_statue_after_each_round: bool,
    pub reward_recognition_enabled: bool,
}

impl Default for AutoBossParam {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AutoBossParam {
    pub fn new(strategy_name: Option<&str>) -> Self {
        let strategy_name = strategy_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(AUTO_STRATEGY_NAME)
            .to_string();
        Self {
            combat_strategy_path: combat_strategy_path(Some(&strategy_name)),
            strategy_name,
            boss_name: String::new(),
            team_name: String::new(),
            specify_run_count: false,
            run_count: 1,
            use_transient_resin: false,
            use_fragile_resin: false,
            revive_retry_count: 3,
            return_to_statue_after_each_round: false,
            reward_recognition_enabled: false,
        }
    }

    pub fn set_strategy_name(&mut self, strategy_name: Option<&str>) {
        let strategy_name = strategy_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(AUTO_STRATEGY_NAME)
            .to_string();
        self.combat_strategy_path = combat_strategy_path(Some(&strategy_name));
        self.strategy_name = strategy_name;
    }

    pub fn set_run_count(&mut self, value: i32) {
        self.run_count = value.max(1);
    }

    pub fn set_revive_retry_count(&mut self, value: i32) {
        self.revive_retry_count = value.max(0);
    }

    pub fn set_specify_run_count(&mut self, enabled: bool) {
        self.specify_run_count = enabled;
        if !enabled {
            self.use_transient_resin = false;
            self.use_fragile_resin = false;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightFinishDetectParam {
    pub battle_end_progress_bar_color: String,
    pub battle_end_progress_bar_color_tolerance: String,
    pub fast_check_enabled: bool,
    pub fast_check_params: String,
    pub check_end_delay: String,
    pub before_detect_delay: String,
    pub rotate_find_enemy_enabled: bool,
    pub rotary_factor: i32,
    pub is_first_check: bool,
    pub check_before_burst: bool,
}

impl Default for FightFinishDetectParam {
    fn default() -> Self {
        Self {
            battle_end_progress_bar_color: String::new(),
            battle_end_progress_bar_color_tolerance: String::new(),
            fast_check_enabled: false,
            fast_check_params: String::new(),
            check_end_delay: String::new(),
            before_detect_delay: String::new(),
            rotate_find_enemy_enabled: false,
            rotary_factor: 10,
            is_first_check: true,
            check_before_burst: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightParam {
    pub combat_strategy_path: String,
    pub team_names: String,
    pub finish_detect_config: FightFinishDetectParam,
    pub fight_finish_detect_enabled: bool,
    pub pick_drops_after_fight_enabled: bool,
    pub pick_drops_after_fight_seconds: i32,
    pub battle_threshold_for_loot: i32,
    pub timeout: i32,
    pub kazuha_pickup_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub kazuha_party_name: String,
    pub only_pick_elite_drops_mode: String,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub check_before_burst: bool,
    pub is_first_check: bool,
    pub rotary_factor: i32,
    pub burst_enabled: bool,
    pub qin_double_pick_up: bool,
    pub swimming_enabled: bool,
    pub exp_based_pickup_enabled: bool,
}

impl Default for AutoFightParam {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AutoFightParam {
    pub fn new(strategy_name: Option<&str>) -> Self {
        Self {
            combat_strategy_path: combat_strategy_path(strategy_name),
            team_names: String::new(),
            finish_detect_config: FightFinishDetectParam::default(),
            fight_finish_detect_enabled: false,
            pick_drops_after_fight_enabled: false,
            pick_drops_after_fight_seconds: 15,
            battle_threshold_for_loot: -1,
            timeout: 120,
            kazuha_pickup_enabled: true,
            action_scheduler_by_cd: String::new(),
            kazuha_party_name: String::new(),
            only_pick_elite_drops_mode: String::new(),
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            guardian_avatar_hold: false,
            check_before_burst: false,
            is_first_check: true,
            rotary_factor: 10,
            burst_enabled: false,
            qin_double_pick_up: false,
            swimming_enabled: false,
            exp_based_pickup_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropFightConfigParam {
    pub strategy_name: String,
    pub team_names: String,
    pub fight_finish_detect_enabled: bool,
    pub action_scheduler_by_cd: String,
    pub finish_detect_config: FightFinishDetectParam,
    pub guardian_avatar: String,
    pub guardian_combat_skip: bool,
    pub guardian_avatar_hold: bool,
    pub burst_enabled: bool,
    pub swimming_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pick_up: bool,
    pub timeout: i32,
    pub seek_enemy_enabled: bool,
    pub seek_enemy_interval_seconds: i32,
    pub seek_enemy_rotary_factor: i32,
}

impl Default for AutoLeyLineOutcropFightConfigParam {
    fn default() -> Self {
        Self {
            strategy_name: String::new(),
            team_names: String::new(),
            fight_finish_detect_enabled: true,
            action_scheduler_by_cd: String::new(),
            finish_detect_config: FightFinishDetectParam {
                is_first_check: false,
                ..FightFinishDetectParam::default()
            },
            guardian_avatar: String::new(),
            guardian_combat_skip: false,
            guardian_avatar_hold: false,
            burst_enabled: false,
            swimming_enabled: false,
            kazuha_pickup_enabled: true,
            qin_double_pick_up: false,
            timeout: 120,
            seek_enemy_enabled: false,
            seek_enemy_interval_seconds: 3,
            seek_enemy_rotary_factor: 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoLeyLineOutcropParam {
    pub count: i32,
    pub country: String,
    pub ley_line_outcrop_type: String,
    pub open_mode_count_min: bool,
    pub is_resin_exhaustion_mode: bool,
    pub use_adventurer_handbook: bool,
    pub friendship_team: String,
    pub team: String,
    pub timeout: i32,
    pub fight_config: AutoLeyLineOutcropFightConfigParam,
    pub is_go_to_synthesizer: bool,
    pub use_fragile_resin: bool,
    pub use_transient_resin: bool,
    pub is_notification: bool,
    pub scan_drops_after_reward_enabled: bool,
    pub scan_drops_after_reward_seconds: i32,
}

impl Default for AutoLeyLineOutcropParam {
    fn default() -> Self {
        Self {
            count: 0,
            country: String::new(),
            ley_line_outcrop_type: String::new(),
            open_mode_count_min: false,
            is_resin_exhaustion_mode: false,
            use_adventurer_handbook: false,
            friendship_team: String::new(),
            team: String::new(),
            timeout: 120,
            fight_config: AutoLeyLineOutcropFightConfigParam::default(),
            is_go_to_synthesizer: false,
            use_fragile_resin: false,
            use_transient_resin: false,
            is_notification: false,
            scan_drops_after_reward_enabled: false,
            scan_drops_after_reward_seconds: 0,
        }
    }
}

impl AutoLeyLineOutcropParam {
    pub fn new(
        count: i32,
        country: impl Into<String>,
        ley_line_outcrop_type: impl Into<String>,
    ) -> Self {
        Self {
            count,
            country: country.into(),
            ley_line_outcrop_type: ley_line_outcrop_type.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoStygianOnslaughtParam {
    pub boss_num: i32,
    pub auto_artifact_salvage: bool,
    pub specify_resin_use: bool,
    pub resin_priority_list: Vec<String>,
    pub original_resin_use_count: i32,
    pub condensed_resin_use_count: i32,
    pub transient_resin_use_count: i32,
    pub fragile_resin_use_count: i32,
    pub fight_team_name: String,
    pub combat_script_bag_path: String,
}

impl Default for AutoStygianOnslaughtParam {
    fn default() -> Self {
        Self {
            boss_num: 0,
            auto_artifact_salvage: false,
            specify_resin_use: false,
            resin_priority_list: default_resin_priority(),
            original_resin_use_count: 0,
            condensed_resin_use_count: 0,
            transient_resin_use_count: 0,
            fragile_resin_use_count: 0,
            fight_team_name: String::new(),
            combat_script_bag_path: combat_strategy_path(None),
        }
    }
}

impl AutoStygianOnslaughtParam {
    pub fn new(combat_script_bag_path: Option<&str>) -> Self {
        Self {
            combat_script_bag_path: combat_script_bag_path
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| combat_strategy_path(None)),
            ..Self::default()
        }
    }

    pub fn set_combat_strategy_path(&mut self, strategy_name: Option<&str>) {
        self.combat_script_bag_path = combat_strategy_path(strategy_name);
    }

    pub fn set_resin_priority_list(
        &mut self,
        priorities: impl IntoIterator<Item = impl Into<String>>,
    ) {
        self.resin_priority_list = priorities.into_iter().map(Into::into).collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskParameterModels {
    pub auto_skip: AutoSkipConfigParam,
    pub auto_domain: AutoDomainParam,
    pub auto_boss: AutoBossParam,
    pub auto_fight: AutoFightParam,
    pub auto_ley_line_outcrop: AutoLeyLineOutcropParam,
    pub auto_stygian_onslaught: AutoStygianOnslaughtParam,
}

impl Default for TaskParameterModels {
    fn default() -> Self {
        Self {
            auto_skip: AutoSkipConfigParam::default(),
            auto_domain: AutoDomainParam::default(),
            auto_boss: AutoBossParam::default(),
            auto_fight: AutoFightParam::default(),
            auto_ley_line_outcrop: AutoLeyLineOutcropParam::default(),
            auto_stygian_onslaught: AutoStygianOnslaughtParam::default(),
        }
    }
}

pub fn task_parameter_models() -> TaskParameterModels {
    TaskParameterModels::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatcherCaptureMode {
    NormalTrigger,
    OnlyCacheCapture,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCommandKind {
    Start,
    Stop,
    Pause,
    Resume,
    Cancel,
    ReloadAssets,
    TakeScreenshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCommand {
    pub kind: TaskCommandKind,
    pub target: Option<String>,
}

impl TaskCommand {
    pub fn new(kind: TaskCommandKind) -> Self {
        Self { kind, target: None }
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerRuntimeConfig {
    pub key: String,
    pub enabled: bool,
}

impl TriggerRuntimeConfig {
    pub fn from_descriptor(trigger: &TriggerDescriptor, all_enabled: bool) -> Self {
        Self {
            key: trigger.key.to_string(),
            enabled: all_enabled || trigger.default_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunnableTrigger {
    pub descriptor: TriggerDescriptor,
    pub enabled: bool,
}

impl RunnableTrigger {
    pub fn is_exclusive(&self) -> bool {
        self.enabled && self.descriptor.exclusive
    }

    pub fn can_run_in_background(&self) -> bool {
        self.enabled && self.descriptor.background
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredRealtimeTrigger {
    pub task_key: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub registered_at_frame: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatcherRuntime {
    pub state: TaskRuntimeState,
    pub interval_ms: u64,
    pub frame_index: u64,
    pub capture_mode: DispatcherCaptureMode,
    pub previous_ui: GameUiCategory,
    pub current_ui: GameUiCategory,
    pub ui_grace_period_ms: u64,
    pub game_active: bool,
    pub game_minimized: bool,
    pub picture_in_picture: bool,
    pub registered_realtime_triggers: Vec<RegisteredRealtimeTrigger>,
}

impl Default for DispatcherRuntime {
    fn default() -> Self {
        Self {
            state: TaskRuntimeState::Stopped,
            interval_ms: 50,
            frame_index: 0,
            capture_mode: DispatcherCaptureMode::NormalTrigger,
            previous_ui: GameUiCategory::Unknown,
            current_ui: GameUiCategory::Unknown,
            ui_grace_period_ms: 30_000,
            game_active: true,
            game_minimized: false,
            picture_in_picture: false,
            registered_realtime_triggers: Vec::new(),
        }
    }
}

impl DispatcherRuntime {
    pub fn advance_frame(&mut self, max_frame_index_second: u64) {
        let max_frame_index = (max_frame_index_second * 1000 / self.interval_ms.max(1)).max(1);
        self.frame_index = (self.frame_index + 1) % max_frame_index;
    }

    pub fn update_ui(&mut self, ui: GameUiCategory) -> bool {
        let changed = self.current_ui != ui;
        self.previous_ui = self.current_ui;
        self.current_ui = ui;
        changed
    }

    pub fn clear_registered_realtime_triggers(&mut self) -> usize {
        let count = self.registered_realtime_triggers.len();
        self.registered_realtime_triggers.clear();
        count
    }

    pub fn add_registered_realtime_trigger(&mut self, plan: &TaskInvocationPlan) -> Result<()> {
        if plan.kind != TaskInvocationKind::AddRealtimeTrigger {
            return Err(TaskError::InvalidInvocationKind {
                expected: TaskInvocationKind::AddRealtimeTrigger,
                actual: plan.kind,
            });
        }
        if plan.clears_existing_triggers {
            self.clear_registered_realtime_triggers();
        }
        let task_key = plan.task_key.clone().ok_or(TaskError::MissingTaskName)?;
        self.registered_realtime_triggers
            .retain(|trigger| !trigger.task_key.eq_ignore_ascii_case(&task_key));
        self.registered_realtime_triggers
            .push(RegisteredRealtimeTrigger {
                task_key,
                interval_ms: plan.interval_ms.unwrap_or(self.interval_ms),
                config: plan.config.clone(),
                registered_at_frame: self.frame_index,
            });
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerRuntime {
    pub state: TaskRuntimeState,
    pub current_task: Option<String>,
    pub continuous_run_group: bool,
    pub pre_execution: bool,
    pub suspended: bool,
    pub auto_pick_pause_count: u32,
    pub party_name: Option<String>,
}

impl Default for RunnerRuntime {
    fn default() -> Self {
        Self {
            state: TaskRuntimeState::Stopped,
            current_task: None,
            continuous_run_group: false,
            pre_execution: false,
            suspended: false,
            auto_pick_pause_count: 0,
            party_name: None,
        }
    }
}

impl RunnerRuntime {
    pub fn start_task(&mut self, task: impl Into<String>) -> Result<()> {
        if matches!(
            self.state,
            TaskRuntimeState::Running | TaskRuntimeState::Suspended
        ) {
            return Err(TaskError::TaskAlreadyRunning(
                self.current_task.clone().unwrap_or_default(),
            ));
        }
        self.current_task = Some(task.into());
        self.state = TaskRuntimeState::Running;
        Ok(())
    }

    pub fn stop_task(&mut self) {
        self.current_task = None;
        self.state = TaskRuntimeState::Stopped;
        self.suspended = false;
        if !self.continuous_run_group {
            self.party_name = None;
        }
    }

    pub fn suspend(&mut self) {
        if self.state == TaskRuntimeState::Running {
            self.state = TaskRuntimeState::Suspended;
        }
        self.suspended = true;
    }

    pub fn resume(&mut self) {
        if self.state == TaskRuntimeState::Suspended {
            self.state = TaskRuntimeState::Running;
        }
        self.suspended = false;
    }

    pub fn stop_auto_pick(&mut self) {
        self.auto_pick_pause_count += 1;
    }

    pub fn resume_auto_pick(&mut self) {
        self.auto_pick_pause_count = self.auto_pick_pause_count.saturating_sub(1);
    }
}

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
                let entry = find_task_catalog_entry(&name)
                    .ok_or_else(|| TaskError::UnknownIndependentTask(name.clone()))?;
                if !matches!(
                    entry.launch_policy,
                    TaskLaunchPolicy::SoloTask
                        | TaskLaunchPolicy::ScriptDispatcher
                        | TaskLaunchPolicy::CommonJob
                ) {
                    return Err(TaskError::InvalidLaunchPolicy {
                        key: entry.key.to_string(),
                        expected: TaskLaunchPolicy::ScriptDispatcher,
                        actual: entry.launch_policy,
                    });
                }
                let kind = match entry.launch_policy {
                    TaskLaunchPolicy::SoloTask => TaskInvocationKind::RunIndependentTask,
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
    NativePending,
    RuntimeOnly,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskInvocationExecutionResult {
    pub plan: TaskInvocationPlan,
    pub mode: TaskInvocationExecutionMode,
    pub status: TaskInvocationExecutionStatus,
    pub message: String,
    pub executed: bool,
}

impl TaskInvocationExecutionResult {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self.status,
            TaskInvocationExecutionStatus::Ready | TaskInvocationExecutionStatus::NativePending
        )
    }
}

pub fn evaluate_task_invocation_plan(
    plan: TaskInvocationPlan,
    mode: TaskInvocationExecutionMode,
) -> TaskInvocationExecutionResult {
    let (status, message) = match plan.kind {
        TaskInvocationKind::ClearRealtimeTriggers => (
            TaskInvocationExecutionStatus::Ready,
            "clears registered realtime triggers".to_string(),
        ),
        TaskInvocationKind::LinkedCancellationTokenSource
        | TaskInvocationKind::LinkedCancellationToken => (
            TaskInvocationExecutionStatus::RuntimeOnly,
            "provides a script cancellation handle; no native task is launched".to_string(),
        ),
        TaskInvocationKind::RunCurrentTask => (
            TaskInvocationExecutionStatus::NativePending,
            "requires the native current-task runner".to_string(),
        ),
        TaskInvocationKind::AddRealtimeTrigger
        | TaskInvocationKind::RunIndependentTask
        | TaskInvocationKind::RunScriptDispatcherTask
        | TaskInvocationKind::RunCommonJob => match plan.catalog_entry.as_ref() {
            Some(entry) if entry.port_state == TaskPortState::Ported => (
                TaskInvocationExecutionStatus::Ready,
                format!("{} is marked ported", entry.key),
            ),
            Some(entry) => (
                TaskInvocationExecutionStatus::NativePending,
                format!("{} is {:?}", entry.key, entry.port_state),
            ),
            None => (
                TaskInvocationExecutionStatus::Invalid,
                "missing task catalog entry".to_string(),
            ),
        },
    };

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

pub fn execute_task_invocation_plans(
    dispatcher: &mut DispatcherRuntime,
    plans: impl IntoIterator<Item = TaskInvocationPlan>,
) -> Vec<TaskInvocationExecutionResult> {
    plans
        .into_iter()
        .map(|plan| execute_task_invocation_plan(dispatcher, plan))
        .collect()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskRuntimeSnapshot {
    pub dispatcher: DispatcherRuntime,
    pub runner: RunnerRuntime,
    pub triggers: Vec<RunnableTrigger>,
    pub independent_tasks: Vec<IndependentTaskDescriptor>,
}

impl TaskRuntimeSnapshot {
    pub fn default_with_legacy_tasks() -> Self {
        Self {
            dispatcher: DispatcherRuntime::default(),
            runner: RunnerRuntime::default(),
            triggers: runtime_triggers(false),
            independent_tasks: independent_tasks(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskSelection {
    pub triggers: Vec<RunnableTrigger>,
    pub reason: TaskSelectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TaskSelectionReason {
    NoEnabledTriggers,
    GameMinimized,
    GameInactiveNoBackgroundTrigger,
    ExclusiveTrigger,
    UiGracePeriod,
    CurrentUiMatch,
    BackgroundTriggers,
}

pub fn runtime_triggers(all_enabled: bool) -> Vec<RunnableTrigger> {
    initial_triggers()
        .into_iter()
        .map(|descriptor| RunnableTrigger {
            enabled: all_enabled || descriptor.default_enabled,
            descriptor,
        })
        .collect()
}

pub fn set_trigger_enabled(
    triggers: &mut [RunnableTrigger],
    key: &str,
    enabled: bool,
) -> Result<()> {
    let trigger = triggers
        .iter_mut()
        .find(|trigger| trigger.descriptor.key.eq_ignore_ascii_case(key))
        .ok_or_else(|| TaskError::UnknownTrigger(key.to_string()))?;
    trigger.enabled = enabled;
    Ok(())
}

pub fn select_triggers_for_tick(
    triggers: &[RunnableTrigger],
    runtime: &DispatcherRuntime,
    elapsed_since_ui_change: Duration,
) -> TaskSelection {
    if runtime.game_minimized {
        return TaskSelection {
            triggers: Vec::new(),
            reason: TaskSelectionReason::GameMinimized,
        };
    }

    let enabled: Vec<_> = triggers
        .iter()
        .filter(|trigger| trigger.enabled)
        .cloned()
        .collect();
    if enabled.is_empty() {
        return TaskSelection {
            triggers: Vec::new(),
            reason: TaskSelectionReason::NoEnabledTriggers,
        };
    }

    if let Some(exclusive) = enabled.iter().find(|trigger| trigger.descriptor.exclusive) {
        if runtime.game_active || exclusive.descriptor.background {
            return TaskSelection {
                triggers: vec![exclusive.clone()],
                reason: TaskSelectionReason::ExclusiveTrigger,
            };
        }
    }

    let mut candidates = enabled;
    if !runtime.game_active {
        candidates.retain(|trigger| trigger.descriptor.background);
        if candidates.is_empty() && !runtime.picture_in_picture {
            return TaskSelection {
                triggers: Vec::new(),
                reason: TaskSelectionReason::GameInactiveNoBackgroundTrigger,
            };
        }
    }

    let in_ui_grace_period =
        elapsed_since_ui_change.as_millis() as u64 <= runtime.ui_grace_period_ms;
    if in_ui_grace_period {
        return TaskSelection {
            triggers: candidates,
            reason: TaskSelectionReason::UiGracePeriod,
        };
    }

    candidates.retain(|trigger| {
        trigger.descriptor.supported_game_ui_category == runtime.current_ui
            || trigger.descriptor.supported_game_ui_category == GameUiCategory::Unknown
    });

    let reason = if runtime.game_active {
        TaskSelectionReason::CurrentUiMatch
    } else {
        TaskSelectionReason::BackgroundTriggers
    };

    TaskSelection {
        triggers: candidates,
        reason,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndependentTaskKind {
    AutoGeniusInvokation,
    AutoWood,
    AutoFight,
    AutoDomain,
    AutoTrack,
    AutoTrackPath,
    AutoMusicGame,
    AutoPathing,
    AutoBoss,
    AutoLeyLineOutcrop,
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
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IndependentTaskExecution {
    Shell(ShellExecutionResult),
    UseRedeemCodePlan(UseRedeemCodeExecutionPlan),
    AutoPathingPlan(AutoPathingExecutionPlan),
    AutoFightPlan(AutoFightExecutionPlan),
    NativePending(TaskInvocationExecutionResult),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndependentTaskExecutionResult {
    pub task_key: String,
    pub execution: IndependentTaskExecution,
}

pub fn execute_independent_task_with_cancel(
    request: &IndependentTaskExecutionRequest,
    is_cancelled: impl FnMut() -> bool,
) -> Result<IndependentTaskExecutionResult> {
    let entry = find_task_catalog_entry(&request.task_key)
        .ok_or_else(|| TaskError::UnknownIndependentTask(request.task_key.clone()))?;
    if entry.key == "Shell" {
        let config = ShellConfig::from_value(request.config.as_ref());
        let param = ShellTaskParam::build_from_config(
            request.command.clone().unwrap_or_default(),
            config,
            request.working_directory.clone(),
        );
        return Ok(IndependentTaskExecutionResult {
            task_key: entry.key.to_string(),
            execution: IndependentTaskExecution::Shell(execute_shell_task_with_cancel(
                &param,
                is_cancelled,
            )?),
        });
    }
    if entry.key == USE_REDEEM_CODE_TASK_KEY {
        let config = UseRedeemCodeExecutionConfig::from_value(request.config.as_ref());
        return Ok(IndependentTaskExecutionResult {
            task_key: entry.key.to_string(),
            execution: IndependentTaskExecution::UseRedeemCodePlan(plan_use_redeem_codes(
                config.codes,
                config.capture_size,
            )?),
        });
    }
    if entry.key == "AutoPathing" {
        let config = AutoPathingExecutionConfig::from_value(request.config.as_ref());
        return Ok(IndependentTaskExecutionResult {
            task_key: entry.key.to_string(),
            execution: IndependentTaskExecution::AutoPathingPlan(plan_auto_pathing(
                &request.working_directory,
                &config.route,
            )?),
        });
    }
    if entry.key == "AutoFight" {
        let config = AutoFightExecutionConfig::from_value(request.config.as_ref());
        return Ok(IndependentTaskExecutionResult {
            task_key: entry.key.to_string(),
            execution: IndependentTaskExecution::AutoFightPlan(plan_auto_fight(
                &request.working_directory,
                config.param,
            )?),
        });
    }

    let task_key = entry.key.to_string();
    let plan = TaskInvocationPlan {
        kind: TaskInvocationKind::RunIndependentTask,
        task_key: Some(task_key.clone()),
        catalog_entry: Some(entry),
        interval_ms: None,
        clears_existing_triggers: false,
        config: request.config.clone(),
        uses_linked_cancellation: false,
    };
    Ok(IndependentTaskExecutionResult {
        task_key,
        execution: IndependentTaskExecution::NativePending(evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        )),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoFightExecutionConfig {
    pub param: AutoFightParam,
}

impl Default for AutoFightExecutionConfig {
    fn default() -> Self {
        Self {
            param: AutoFightParam::default(),
        }
    }
}

impl AutoFightExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
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
    );
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoPathingExecutionConfig {
    pub route: String,
}

impl Default for AutoPathingExecutionConfig {
    fn default() -> Self {
        Self {
            route: String::new(),
        }
    }
}

impl AutoPathingExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPathingExecutionPlan {
    pub source: &'static str,
    pub route: String,
    pub normalized_path: PathBuf,
    pub summary: PathingSummary,
    pub execution_plan: PathingExecutionPlan,
    pub dispatched: bool,
    pub completed: bool,
    pub notes: String,
}

pub fn plan_auto_pathing(
    working_directory: impl AsRef<std::path::Path>,
    route: &str,
) -> Result<AutoPathingExecutionPlan> {
    let normalized_path = normalize_user_auto_pathing_route(route)?;
    let path = working_directory
        .as_ref()
        .join("User")
        .join("AutoPathing")
        .join(&normalized_path);
    let task =
        read_pathing_task(&path).map_err(|error| TaskError::PathingPlan(error.to_string()))?;
    let summary = task.summary();
    Ok(AutoPathingExecutionPlan {
        source: "UserAutoPathingFile",
        route: route.to_string(),
        normalized_path,
        summary,
        execution_plan: task.execution_plan(),
        dispatched: false,
        completed: false,
        notes:
            "Route JSON is parsed and converted into the migrated PathExecutor preparation plan; native movement dispatch remains pending."
                .to_string(),
    })
}

fn normalize_user_auto_pathing_route(route: &str) -> Result<PathBuf> {
    let route = route.trim().replace('\\', "/");
    if route.is_empty() {
        return Err(TaskError::EmptyPathingRoute);
    }
    let path = PathBuf::from(&route);
    if path.is_absolute() {
        return Err(TaskError::InvalidPathingRoute(route));
    }
    if path
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(TaskError::InvalidPathingRoute(route));
    }
    Ok(path)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UseRedeemCodeExecutionConfig {
    pub codes: Vec<RedeemCodeEntry>,
    pub capture_size: Size,
}

impl Default for UseRedeemCodeExecutionConfig {
    fn default() -> Self {
        Self {
            codes: Vec::new(),
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl UseRedeemCodeExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn independent_tasks() -> Vec<IndependentTaskDescriptor> {
    [
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
        (IndependentTaskKind::AutoPathing, "AutoPathing"),
        (IndependentTaskKind::AutoBoss, "AutoBoss"),
        (
            IndependentTaskKind::AutoLeyLineOutcrop,
            "AutoLeyLineOutcrop",
        ),
        (IndependentTaskKind::Shell, "Shell"),
    ]
    .into_iter()
    .filter_map(|(kind, key)| {
        find_task_catalog_entry(key).map(|entry| IndependentTaskDescriptor {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShellConfig {
    pub disable: bool,
    pub timeout: i32,
    pub no_window: bool,
    pub output: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            disable: false,
            timeout: 60,
            no_window: true,
            output: true,
        }
    }
}

impl ShellConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellTaskParam {
    pub command: String,
    pub timeout_seconds: i32,
    pub no_window: bool,
    pub output: bool,
    pub disable: bool,
    pub working_directory: PathBuf,
}

impl ShellTaskParam {
    pub fn build_from_config(
        command: impl Into<String>,
        config: ShellConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            command: command.into(),
            timeout_seconds: config.timeout,
            no_window: config.no_window,
            output: config.output,
            disable: config.disable,
            working_directory: working_directory.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellExecutionStatus {
    Disabled,
    EmptyCommand,
    Started,
    Completed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExecutionResult {
    pub command: String,
    pub working_directory: PathBuf,
    pub timeout_seconds: i32,
    pub no_window: bool,
    pub output_enabled: bool,
    pub status: ShellExecutionStatus,
    pub waited_for_exit: bool,
    pub exit_code: Option<i32>,
    pub output_shell: String,
    pub output: String,
}

impl ShellExecutionResult {
    pub fn has_output(&self) -> bool {
        !self.output_shell.is_empty() || !self.output.is_empty()
    }
}

pub fn execute_shell_task(param: &ShellTaskParam) -> Result<ShellExecutionResult> {
    execute_shell_task_with_cancel(param, || false)
}

pub fn execute_shell_task_with_cancel(
    param: &ShellTaskParam,
    mut is_cancelled: impl FnMut() -> bool,
) -> Result<ShellExecutionResult> {
    if param.disable {
        return Ok(shell_result(param, ShellExecutionStatus::Disabled, false));
    }
    if param.command.trim().is_empty() {
        return Ok(shell_result(
            param,
            ShellExecutionStatus::EmptyCommand,
            false,
        ));
    }

    let mut child = shell_command(param)
        .spawn()
        .map_err(TaskError::ShellStart)?;
    {
        let mut stdin = child.stdin.take().ok_or_else(|| {
            TaskError::ShellIo(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "shell stdin was not captured",
            ))
        })?;
        writeln!(stdin, "{}", param.command).map_err(TaskError::ShellIo)?;
        stdin.flush().map_err(TaskError::ShellIo)?;
    }

    if param.timeout_seconds <= 0 {
        return Ok(shell_result(param, ShellExecutionStatus::Started, false));
    }

    let deadline = Duration::from_secs(param.timeout_seconds as u64);
    let start = std::time::Instant::now();
    loop {
        if is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ShellExecutionResult {
                status: ShellExecutionStatus::Cancelled,
                waited_for_exit: true,
                ..shell_result(param, ShellExecutionStatus::Cancelled, true)
            });
        }
        if let Some(status) = child.try_wait().map_err(TaskError::ShellIo)? {
            let output = child.wait_with_output().map_err(TaskError::ShellIo)?;
            let (output_shell, output_text) = if param.output {
                split_legacy_shell_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                (String::new(), String::new())
            };
            return Ok(ShellExecutionResult {
                command: param.command.clone(),
                working_directory: param.working_directory.clone(),
                timeout_seconds: param.timeout_seconds,
                no_window: param.no_window,
                output_enabled: param.output,
                status: ShellExecutionStatus::Completed,
                waited_for_exit: true,
                exit_code: status.code(),
                output_shell,
                output: output_text,
            });
        }
        if start.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ShellExecutionResult {
                status: ShellExecutionStatus::TimedOut,
                waited_for_exit: true,
                ..shell_result(param, ShellExecutionStatus::TimedOut, true)
            });
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn shell_result(
    param: &ShellTaskParam,
    status: ShellExecutionStatus,
    waited_for_exit: bool,
) -> ShellExecutionResult {
    ShellExecutionResult {
        command: param.command.clone(),
        working_directory: param.working_directory.clone(),
        timeout_seconds: param.timeout_seconds,
        no_window: param.no_window,
        output_enabled: param.output,
        status,
        waited_for_exit,
        exit_code: None,
        output_shell: String::new(),
        output: String::new(),
    }
}

#[cfg(windows)]
fn shell_command(param: &ShellTaskParam) -> Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new("cmd.exe");
    command.arg("/k").arg("@echo off");
    if param.no_window {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    configure_shell_command(&mut command, param);
    command
}

#[cfg(not(windows))]
fn shell_command(param: &ShellTaskParam) -> Command {
    let mut command = Command::new("sh");
    configure_shell_command(&mut command, param);
    command
}

fn configure_shell_command(command: &mut Command, param: &ShellTaskParam) {
    command.current_dir(&param.working_directory);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
}

fn split_legacy_shell_output(stdout: &str) -> (String, String) {
    let normalized = stdout.replace("\r\n", "\n");
    let mut lines = normalized.lines();
    let output_shell = lines.next().unwrap_or_default().to_string();
    let output = lines.collect::<Vec<_>>().join("\n");
    (output_shell, output)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskProgress {
    pub name: String,
    pub current_step: Option<String>,
    pub completed: u32,
    pub total: Option<u32>,
    pub message: Option<String>,
}

impl TaskProgress {
    pub fn percentage(&self) -> Option<u8> {
        let total = self.total?;
        if total == 0 {
            return None;
        }
        Some(((self.completed.min(total) * 100) / total) as u8)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRegistry {
    trigger_enabled: BTreeMap<String, bool>,
}

impl TaskRegistry {
    pub fn from_triggers(triggers: &[RunnableTrigger]) -> Self {
        Self {
            trigger_enabled: triggers
                .iter()
                .map(|trigger| (trigger.descriptor.key.to_string(), trigger.enabled))
                .collect(),
        }
    }

    pub fn is_enabled(&self, key: &str) -> Option<bool> {
        self.trigger_enabled
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(key))
            .map(|(_, enabled)| *enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn selects_exclusive_trigger_before_other_enabled_triggers() {
        let mut triggers = runtime_triggers(true);
        let descriptor = triggers
            .iter()
            .find(|trigger| trigger.descriptor.key == "AutoPick")
            .unwrap()
            .descriptor
            .clone();
        triggers.push(RunnableTrigger {
            descriptor: TriggerDescriptor {
                key: "ExclusiveAutoPick",
                exclusive: true,
                ..descriptor
            },
            enabled: true,
        });

        let selection = select_triggers_for_tick(
            &triggers,
            &DispatcherRuntime {
                current_ui: GameUiCategory::Dialog,
                ..DispatcherRuntime::default()
            },
            Duration::from_secs(60),
        );

        assert_eq!(selection.reason, TaskSelectionReason::ExclusiveTrigger);
        assert_eq!(selection.triggers.len(), 1);
        assert_eq!(selection.triggers[0].descriptor.key, "ExclusiveAutoPick");
    }

    #[test]
    fn inactive_game_keeps_only_background_triggers() {
        let triggers = runtime_triggers(true);
        let selection = select_triggers_for_tick(
            &triggers,
            &DispatcherRuntime {
                game_active: false,
                current_ui: GameUiCategory::Dialog,
                ..DispatcherRuntime::default()
            },
            Duration::from_secs(60),
        );

        assert_eq!(selection.reason, TaskSelectionReason::BackgroundTriggers);
        assert!(selection
            .triggers
            .iter()
            .all(|trigger| trigger.descriptor.background));
    }

    #[test]
    fn ui_grace_period_runs_all_candidate_triggers() {
        let triggers = runtime_triggers(false);
        let selection = select_triggers_for_tick(
            &triggers,
            &DispatcherRuntime {
                current_ui: GameUiCategory::BigMap,
                ..DispatcherRuntime::default()
            },
            Duration::from_secs(2),
        );

        assert_eq!(selection.reason, TaskSelectionReason::UiGracePeriod);
        assert!(selection
            .triggers
            .iter()
            .any(|trigger| trigger.descriptor.key == "AutoPick"));
        assert!(selection
            .triggers
            .iter()
            .any(|trigger| trigger.descriptor.key == "AutoSkip"));
    }

    #[test]
    fn runner_preserves_party_name_for_continuous_groups() {
        let mut runner = RunnerRuntime {
            continuous_run_group: true,
            party_name: Some("daily".to_string()),
            ..RunnerRuntime::default()
        };
        runner.start_task("AutoDomain").unwrap();
        runner.stop_task();

        assert_eq!(runner.party_name.as_deref(), Some("daily"));
        assert_eq!(runner.state, TaskRuntimeState::Stopped);
    }

    #[test]
    fn progress_percentage_clamps_completed_count() {
        let progress = TaskProgress {
            completed: 15,
            total: Some(10),
            ..TaskProgress::default()
        };
        assert_eq!(progress.percentage(), Some(100));
    }

    #[test]
    fn combat_script_parser_preserves_legacy_command_syntax() {
        let script = parse_combat_script_context(
            r#"
            # comment
            钟离 s(0.2), e(hold), wait(0.2), w(0.2)
            夜兰 round(1,3-4), keydown(VK_W), wait(0.05), keyup(VK_W) | round(2), q
            "#,
            true,
        )
        .unwrap();

        assert_eq!(
            script.avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string()]
        );
        assert_eq!(script.commands.len(), 8);
        assert_eq!(script.commands[0].method, CombatCommandMethod::S);
        assert_eq!(script.commands[0].args, vec!["0.2".to_string()]);
        assert_eq!(script.commands[1].method, CombatCommandMethod::Skill);
        assert_eq!(script.commands[1].args, vec!["hold".to_string()]);
        assert_eq!(script.commands[4].method, CombatCommandMethod::KeyDown);
        assert_eq!(script.commands[4].activating_rounds, vec![1, 3, 4]);
        assert_eq!(script.commands[7].method, CombatCommandMethod::Burst);
        assert_eq!(script.commands[7].activating_rounds, vec![2]);
    }

    #[test]
    fn combat_script_parser_merges_comma_arguments_inside_parentheses() {
        let script =
            parse_combat_script_context("那维莱特 moveby(1800, -2100), scroll(-1)", true).unwrap();

        assert_eq!(script.commands.len(), 2);
        assert_eq!(script.commands[0].method, CombatCommandMethod::MoveBy);
        assert_eq!(
            script.commands[0].args,
            vec!["1800".to_string(), "-2100".to_string()]
        );
        assert_eq!(script.commands[1].method, CombatCommandMethod::Scroll);
        assert_eq!(script.commands[1].args, vec!["-1".to_string()]);
    }

    #[test]
    fn combat_script_bag_selects_best_matching_team_script() {
        let bag = CombatScriptBagPlan {
            source_path: PathBuf::from("User").join("AutoFight"),
            scripts: vec![
                parse_combat_script_context("钟离 e", true).unwrap(),
                parse_combat_script_context("钟离 e\n夜兰 q", true).unwrap(),
            ],
            parse_failures: Vec::new(),
        };

        let matched = match_combat_script(&bag, &["钟离".to_string(), "夜兰".to_string()]).unwrap();

        assert!(matched.full_match);
        assert_eq!(matched.matched_avatar_count, 2);
        assert_eq!(matched.commands.len(), 2);
    }

    #[test]
    fn combat_team_selection_filters_commands_like_auto_fight_task() {
        let bag = CombatScriptBagPlan {
            source_path: PathBuf::from("User").join("AutoFight"),
            scripts: vec![
                parse_combat_script_context("钟离 e\n夜兰 q\n当前角色 keypress(q)", true).unwrap(),
                parse_combat_script_context("钟离 e\n纳西妲 e\n班尼特 q", true).unwrap(),
            ],
            parse_failures: Vec::new(),
        };

        let selection = plan_combat_script_team_selection(
            &bag,
            &["钟离".to_string(), "纳西妲".to_string(), "行秋".to_string()],
        );

        assert_eq!(
            selection.status,
            CombatScriptTeamSelectionStatus::PartialFallback
        );
        assert_eq!(selection.matched_avatar_count, 2);
        assert_eq!(
            selection.command_avatar_names,
            vec![
                "钟离".to_string(),
                "纳西妲".to_string(),
                "班尼特".to_string()
            ]
        );
        assert_eq!(
            selection.executable_avatar_names,
            vec!["钟离".to_string(), "纳西妲".to_string()]
        );
        assert_eq!(
            selection.filtered_out_avatar_names,
            vec!["班尼特".to_string()]
        );
        assert_eq!(selection.executable_commands.len(), 2);
        assert!(selection
            .executable_commands
            .iter()
            .all(|command| command.avatar != CURRENT_COMBAT_AVATAR_NAME));
    }

    #[test]
    fn combat_team_selection_reports_missing_team_context() {
        let bag = CombatScriptBagPlan {
            source_path: PathBuf::from("User").join("AutoFight"),
            scripts: vec![parse_combat_script_context("钟离 e", true).unwrap()],
            parse_failures: Vec::new(),
        };

        let selection = plan_combat_script_team_selection(&bag, &[]);

        assert_eq!(
            selection.status,
            CombatScriptTeamSelectionStatus::NoTeamContext
        );
        assert!(selection.executable_commands.is_empty());
    }

    #[test]
    fn combat_avatar_catalog_standardizes_configured_team_aliases() {
        let root = unique_test_root("combat-avatar-catalog");
        write_test_combat_avatar_catalog(&root);
        let catalog = read_combat_avatar_catalog(&root).unwrap();

        assert_eq!(catalog.source_path, root.join(COMBAT_AVATAR_CATALOG_PATH));
        assert_eq!(catalog.standard_name_for_alias("班爷").unwrap(), "班尼特");
        assert_eq!(catalog.standard_name_for_alias("秋秋人").unwrap(), "行秋");
        assert_eq!(
            standardize_configured_team_avatar_names(&catalog, "钟离，叶天帝,秋秋人,班爷").unwrap(),
            vec![
                "钟离".to_string(),
                "枫原万叶".to_string(),
                "行秋".to_string(),
                "班尼特".to_string()
            ]
        );

        let wrong_count =
            standardize_configured_team_avatar_names(&catalog, "钟离,夜兰,行秋").unwrap_err();
        assert!(
            matches!(wrong_count, TaskError::CombatStrategy(message) if message.contains("当前3个"))
        );

        let unknown = standardize_configured_team_avatar_names(&catalog, "钟离,夜兰,不存在,班尼特")
            .unwrap_err();
        assert!(
            matches!(unknown, TaskError::CombatStrategy(message) if message.contains("角色名称校验失败：不存在"))
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn combat_script_context_standardizes_avatar_aliases_when_catalog_is_available() {
        let root = unique_test_root("combat-script-avatar-alias");
        write_test_combat_avatar_catalog(&root);
        let catalog = read_combat_avatar_catalog(&root).unwrap();

        let script = parse_combat_script_context_with_catalog(
            "帝君 e\n班爷 q\n当前角色 keypress(q)",
            true,
            Some(&catalog),
        )
        .unwrap();

        assert_eq!(
            script.avatar_names,
            vec![
                "钟离".to_string(),
                "班尼特".to_string(),
                CURRENT_COMBAT_AVATAR_NAME.to_string()
            ]
        );
        assert_eq!(script.commands[0].avatar, "钟离");
        assert_eq!(script.commands[1].avatar, "班尼特");
        assert_eq!(script.commands[2].avatar, CURRENT_COMBAT_AVATAR_NAME);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auto_fight_finish_detection_matches_legacy_pixels() {
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
            RgbPixel {
                r: 230,
                g: 220,
                b: 40,
            },
        );
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
            RgbPixel {
                r: 248,
                g: 250,
                b: 251,
            },
        );

        let result = detect_auto_fight_finished_from_image(&image).unwrap();

        assert!(result.finished);
        assert_eq!(
            result.progress_pixel,
            RgbPixel {
                r: 230,
                g: 220,
                b: 40
            }
        );
        assert_eq!(
            result.white_tile_pixel,
            RgbPixel {
                r: 248,
                g: 250,
                b: 251
            }
        );
        assert!(is_auto_fight_finish_yellow(RgbPixel {
            r: 200,
            g: 255,
            b: 100
        }));
        assert!(is_auto_fight_finish_white(RgbPixel {
            r: 240,
            g: 255,
            b: 255
        }));
    }

    #[test]
    fn auto_fight_finish_detection_rejects_missing_or_non_matching_pixels() {
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
            RgbPixel {
                r: 180,
                g: 220,
                b: 40,
            },
        );
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
            RgbPixel {
                r: 248,
                g: 250,
                b: 251,
            },
        );

        let result = detect_auto_fight_finished_from_image(&image).unwrap();
        assert!(!result.finished);
        assert!(!is_auto_fight_finish_yellow(RgbPixel {
            r: 199,
            g: 255,
            b: 100
        }));
        assert!(!is_auto_fight_finish_white(RgbPixel {
            r: 239,
            g: 255,
            b: 255
        }));

        let small = blank_bgr_image(Size::new(200, 100));
        let error = detect_auto_fight_finished_from_image(&small).unwrap_err();
        assert!(
            matches!(error, TaskError::VisionPlan(message) if message.contains("outside capture"))
        );
    }

    #[test]
    fn auto_fight_finish_detection_plan_preserves_legacy_probe_sequence() {
        let config = FightFinishDetectParam::default();

        let plan = plan_auto_fight_finish_detection(&config, 1500, 450).unwrap();

        assert_eq!(plan.pre_detect_delay_ms, 1500);
        assert_eq!(plan.detect_delay_ms, 450);
        assert_eq!(plan.progress_pixel, AUTO_FIGHT_FINISH_PROGRESS_PIXEL);
        assert_eq!(plan.white_tile_pixel, AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL);
        assert!(!plan.native_ready_without_capture);
        assert_eq!(
            plan.steps
                .iter()
                .filter(|step| step.enabled)
                .map(|step| step.kind)
                .collect::<Vec<_>>(),
            vec![
                AutoFightFinishDetectionStepKind::PreDetectDelay,
                AutoFightFinishDetectionStepKind::OpenPartySetup,
                AutoFightFinishDetectionStepKind::WaitForPartySetup,
                AutoFightFinishDetectionStepKind::CaptureFrame,
                AutoFightFinishDetectionStepKind::SampleFinishPixels,
                AutoFightFinishDetectionStepKind::DropFromPartySetup,
                AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished,
            ]
        );
        let open_step = plan
            .steps
            .iter()
            .find(|step| step.kind == AutoFightFinishDetectionStepKind::OpenPartySetup)
            .unwrap();
        assert_eq!(
            open_step.input_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::L.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::L.vk(),
                    extended: None
                }
            ]
        );
        let drop_step = plan
            .steps
            .iter()
            .find(|step| step.kind == AutoFightFinishDetectionStepKind::DropFromPartySetup)
            .unwrap();
        assert_eq!(
            drop_step.input_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::X.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::X.vk(),
                    extended: None
                }
            ]
        );
        assert!(plan.steps.iter().any(|step| {
            step.kind == AutoFightFinishDetectionStepKind::CaptureFrame && step.requires_capture
        }));
        assert!(plan.steps.iter().any(|step| {
            step.kind == AutoFightFinishDetectionStepKind::SampleFinishPixels
                && step.requires_vision
        }));

        let mut rotate_config = FightFinishDetectParam::default();
        rotate_config.rotate_find_enemy_enabled = true;
        let rotate_plan = plan_auto_fight_finish_detection(&rotate_config, 1500, 450).unwrap();
        assert!(!rotate_plan.steps.iter().any(|step| {
            step.enabled && step.kind == AutoFightFinishDetectionStepKind::PreDetectDelay
        }));
        assert!(rotate_plan
            .steps
            .iter()
            .any(|step| step.enabled && step.kind == AutoFightFinishDetectionStepKind::SeekEnemy));
    }

    #[test]
    fn auto_fight_finish_detection_probe_executes_against_supplied_capture() {
        let config = FightFinishDetectParam::default();
        let plan = plan_auto_fight_finish_detection(&config, 1500, 450).unwrap();
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
            RgbPixel {
                r: 230,
                g: 220,
                b: 40,
            },
        );
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
            RgbPixel {
                r: 248,
                g: 250,
                b: 251,
            },
        );

        let execution = execute_auto_fight_finish_detection_probe(
            &plan,
            &image,
            AutoFightFinishDetectionExecutionMode::PlanOnly,
            None,
        )
        .unwrap();

        assert_eq!(
            execution.mode,
            AutoFightFinishDetectionExecutionMode::PlanOnly
        );
        assert!(execution.detection.finished);
        assert!(!execution.dispatched);
        assert_eq!(
            execution.before_capture_events,
            vec![
                InputEvent::Delay { milliseconds: 1500 },
                InputEvent::KeyDown {
                    vk: KeyId::L.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::L.vk(),
                    extended: None
                },
                InputEvent::Delay { milliseconds: 450 }
            ]
        );
        assert_eq!(
            execution.after_detection_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::X.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::X.vk(),
                    extended: None
                },
                InputEvent::KeyDown {
                    vk: KeyId::L.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::L.vk(),
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn auto_fight_finish_detection_probe_skips_cancel_switch_when_unfinished() {
        let config = FightFinishDetectParam::default();
        let plan = plan_auto_fight_finish_detection(&config, 1500, 450).unwrap();
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
            RgbPixel {
                r: 180,
                g: 220,
                b: 40,
            },
        );
        set_bgr_pixel(
            &mut image,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
            RgbPixel {
                r: 248,
                g: 250,
                b: 251,
            },
        );

        let execution = execute_auto_fight_finish_detection_probe(
            &plan,
            &image,
            AutoFightFinishDetectionExecutionMode::PlanOnly,
            None,
        )
        .unwrap();

        assert!(!execution.detection.finished);
        assert_eq!(
            execution.after_detection_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::X.vk(),
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: KeyId::X.vk(),
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn auto_fight_finish_detection_live_probe_captures_between_event_groups() {
        let config = FightFinishDetectParam::default();
        let plan = plan_auto_fight_finish_detection(&config, 1500, 450).unwrap();

        let execution = execute_auto_fight_finish_detection_live_probe(
            &plan,
            AutoFightFinishDetectionExecutionMode::PlanOnly,
            None,
            || {
                let mut image = blank_bgr_image(Size::new(1920, 1080));
                set_bgr_pixel(
                    &mut image,
                    AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
                    RgbPixel {
                        r: 230,
                        g: 220,
                        b: 40,
                    },
                );
                set_bgr_pixel(
                    &mut image,
                    AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
                    RgbPixel {
                        r: 248,
                        g: 250,
                        b: 251,
                    },
                );
                Ok(image)
            },
        )
        .unwrap();

        assert_eq!(
            execution.mode,
            AutoFightFinishDetectionExecutionMode::PlanOnly
        );
        assert!(execution.captured);
        assert!(!execution.dispatched);
        assert_eq!(execution.dispatched_events, 0);
        assert!(execution.detection.unwrap().finished);
        assert_eq!(execution.before_capture_events.len(), 4);
        assert_eq!(execution.after_detection_events.len(), 4);
    }

    #[test]
    fn auto_fight_finish_detection_live_probe_can_cancel_before_capture() {
        let config = FightFinishDetectParam::default();
        let plan = plan_auto_fight_finish_detection(&config, 1500, 450).unwrap();
        let cancellation = InputCancellationToken::new();
        cancellation.cancel();

        let execution = execute_auto_fight_finish_detection_live_probe(
            &plan,
            AutoFightFinishDetectionExecutionMode::SendInput,
            Some(&cancellation),
            || panic!("capture must not run after pre-capture cancellation"),
        )
        .unwrap();

        assert!(execution.dispatched);
        assert!(execution.cancelled);
        assert!(!execution.captured);
        assert!(execution.detection.is_none());
        assert!(execution.after_detection_events.is_empty());
        assert_eq!(execution.dispatched_events, 0);
    }

    #[test]
    fn active_avatar_detection_uses_white_rect_majority() {
        let mut image = blank_bgr_image(Size::new(24, 4));
        let rects = small_avatar_index_rects();
        fill_rect_gray(&mut image, rects[0], 252);
        fill_rect_gray(&mut image, rects[1], 90);
        fill_rect_gray(&mut image, rects[2], 252);
        fill_rect_gray(&mut image, rects[3], 252);

        let result = detect_active_combat_avatar_index_by_color(&image, &rects).unwrap();

        assert_eq!(result.active_index, Some(2));
        assert_eq!(
            result.method,
            CombatActiveAvatarDetectionMethod::WhiteRectMajority
        );
        assert_eq!(result.white_rect_count, 3);
        assert_eq!(result.not_white_rect_index, Some(2));
    }

    #[test]
    fn active_avatar_detection_uses_edge_white_ratio_when_fill_is_ambiguous() {
        let mut image = blank_bgr_image(Size::new(60, 12));
        let rects = vec![
            Rect::new(0, 0, 12, 12).unwrap(),
            Rect::new(14, 0, 12, 12).unwrap(),
            Rect::new(28, 0, 12, 12).unwrap(),
            Rect::new(42, 0, 12, 12).unwrap(),
        ];
        for (index, rect) in rects.iter().copied().enumerate() {
            fill_rect_gray(&mut image, rect, 100);
            if index != 2 {
                draw_rect_edge_gray(&mut image, rect, 255);
            }
        }

        let result = detect_active_combat_avatar_index_by_color(&image, &rects).unwrap();

        assert_eq!(result.active_index, Some(3));
        assert_eq!(
            result.method,
            CombatActiveAvatarDetectionMethod::EdgeWhiteRatio
        );
        assert_eq!(result.edge_white_ratios.len(), 4);
        assert!(result.edge_white_ratios[0] > 0.5);
        assert!(result.edge_white_ratios[2] <= 0.5);
    }

    #[test]
    fn active_avatar_detection_uses_difference_vote_for_full_team() {
        let mut image = blank_bgr_image(Size::new(24, 4));
        let rects = small_avatar_index_rects();
        for rect in rects.iter().copied() {
            fill_rect_gray(&mut image, rect, 110);
        }
        fill_rect_gray(&mut image, rects[3], 120);

        let result = detect_active_combat_avatar_index_by_color(&image, &rects).unwrap();

        assert_eq!(result.active_index, Some(4));
        assert_eq!(
            result.method,
            CombatActiveAvatarDetectionMethod::ImageDifferenceVote
        );
        assert_eq!(result.difference_votes, vec![1, 0, 0, 3]);
    }

    #[test]
    fn active_avatar_detection_defaults_single_avatar_to_active() {
        let image = blank_bgr_image(Size::new(8, 4));
        let rects = vec![Rect::new(1, 1, 3, 2).unwrap()];

        let result = detect_active_combat_avatar_index_by_color(&image, &rects).unwrap();

        assert_eq!(result.active_index, Some(1));
        assert_eq!(
            result.method,
            CombatActiveAvatarDetectionMethod::SingleAvatar
        );
    }

    #[test]
    fn active_avatar_detection_uses_arrow_template_when_color_is_unresolved() {
        let root = unique_test_root("active-avatar-arrow");
        let asset_path = root
            .join("GameTask")
            .join("Common")
            .join("Element")
            .join("Assets")
            .join("1920x1080")
            .join(AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET);
        fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
        let template = BgrImage::new(
            Size::new(2, 2),
            vec![255, 255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 255],
        )
        .unwrap();
        template.write_png(&asset_path).unwrap();
        let mut image = blank_bgr_image(Size::new(48, 20));
        blit_bgr_image(&mut image, &template, 6, 11);
        let rects = vec![
            Rect::new(32, 1, 6, 3).unwrap(),
            Rect::new(32, 6, 6, 3).unwrap(),
            Rect::new(32, 11, 6, 3).unwrap(),
            Rect::new(32, 16, 6, 3).unwrap(),
        ];
        let result = detect_active_combat_avatar_index_by_color_then_arrow(
            &root,
            &image,
            &rects,
            Rect::new(0, 0, 16, 20).unwrap(),
        )
        .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(result.active_index, Some(3));
        assert_eq!(
            result.method,
            CombatActiveAvatarDetectionMethod::ArrowTemplate
        );
        assert_eq!(result.white_rect_count, 0);
        assert_eq!(result.difference_votes, vec![0, 0, 0, 0]);
    }

    #[test]
    fn combat_avatar_index_rect_detection_finds_template_rects() {
        let root = unique_test_root("avatar-index-templates");
        let index_roi = Rect::new(0, 0, 16, 20).unwrap();
        let arrow_roi = Rect::new(20, 0, 8, 20).unwrap();
        let templates = write_test_index_templates(&root);
        let mut image = blank_bgr_image(Size::new(32, 24));
        for (index, template) in templates.iter().enumerate() {
            blit_bgr_image(&mut image, template, 4, 2 + index as u32 * 5);
        }

        let detection =
            detect_combat_avatar_index_rects_from_templates_in(&root, &image, index_roi, arrow_roi)
                .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(!detection.inferred_from_current_avatar_arrow);
        assert_eq!(detection.resolved_rects.len(), 4);
        assert_eq!(
            detection.rects_by_index[0],
            Some(Rect::new(4, 2, 3, 3).unwrap())
        );
        assert_eq!(
            detection.rects_by_index[3],
            Some(Rect::new(4, 17, 3, 3).unwrap())
        );
    }

    #[test]
    fn combat_avatar_index_rect_detection_inferrs_missing_active_rect_from_arrow() {
        let root = unique_test_root("avatar-index-arrow-infer");
        let index_roi = Rect::new(0, 0, 24, 80).unwrap();
        let arrow_roi = Rect::new(0, 0, 24, 80).unwrap();
        let templates = write_test_index_templates(&root);
        let arrow_template = write_test_current_avatar_arrow_template(&root);
        let mut image = blank_bgr_image(Size::new(48, 216));
        blit_bgr_image(&mut image, &templates[0], 4, 3);
        blit_bgr_image(&mut image, &arrow_template, 8, 22);

        let detection =
            detect_combat_avatar_index_rects_from_templates_in(&root, &image, index_roi, arrow_roi)
                .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(detection.inferred_from_current_avatar_arrow);
        assert_eq!(
            detection.rects_by_index[0],
            Some(Rect::new(4, 3, 3, 3).unwrap())
        );
        assert_eq!(
            detection.rects_by_index[1],
            Some(Rect::new(4, 22, 3, 3).unwrap())
        );
        assert_eq!(
            detection.resolved_rects,
            vec![
                Rect::new(4, 3, 3, 3).unwrap(),
                Rect::new(4, 22, 3, 3).unwrap()
            ]
        );
    }

    #[test]
    fn combat_multi_game_detection_matches_legacy_icon_count_rules() {
        let solo = combat_multi_game_detection_from_icon_counts(0, false).unwrap();
        assert_eq!(solo.status, CombatMultiGameStatus::default());
        assert_eq!(solo.status.max_control_avatar_count().unwrap(), 4);

        let solo_host = combat_multi_game_detection_from_icon_counts(0, true).unwrap();
        assert_eq!(
            solo_host.status,
            CombatMultiGameStatus {
                is_in_multi_game: true,
                is_host: true,
                player_count: 1
            }
        );
        assert_eq!(solo_host.status.max_control_avatar_count().unwrap(), 4);

        let guest = combat_multi_game_detection_from_icon_counts(2, false).unwrap();
        assert_eq!(
            guest.status,
            CombatMultiGameStatus {
                is_in_multi_game: true,
                is_host: false,
                player_count: 3
            }
        );
        assert_eq!(guest.status.max_control_avatar_count().unwrap(), 1);

        let host = combat_multi_game_detection_from_icon_counts(2, true).unwrap();
        assert_eq!(
            host.status,
            CombatMultiGameStatus {
                is_in_multi_game: true,
                is_host: true,
                player_count: 3
            }
        );
        assert_eq!(host.status.max_control_avatar_count().unwrap(), 2);
        assert!(combat_multi_game_detection_from_icon_counts(4, true).is_err());
    }

    #[test]
    fn combat_multi_game_rect_maps_match_legacy_auto_fight_assets() {
        let size = Size::new(1920, 1080);
        let host_three = CombatMultiGameStatus {
            is_in_multi_game: true,
            is_host: true,
            player_count: 3,
        };
        assert_eq!(
            combat_avatar_index_rects_for_multi_game_status(size, host_three).unwrap(),
            vec![
                Rect::new(1859, 459, 28, 24).unwrap(),
                Rect::new(1859, 555, 28, 24).unwrap()
            ]
        );
        assert_eq!(
            combat_avatar_side_icon_rects_for_multi_game_status(size, host_three).unwrap(),
            vec![
                Rect::new(1765, 375, 76, 76).unwrap(),
                Rect::new(1765, 470, 76, 76).unwrap()
            ]
        );

        let guest_four = CombatMultiGameStatus {
            is_in_multi_game: true,
            is_host: false,
            player_count: 4,
        };
        assert_eq!(
            combat_avatar_index_rects_for_multi_game_status(size, guest_four).unwrap(),
            vec![Rect::new(1859, 507, 28, 24).unwrap()]
        );
        assert_eq!(
            combat_avatar_side_icon_rects_for_multi_game_status(size, guest_four).unwrap(),
            vec![Rect::new(1765, 515, 76, 76).unwrap()]
        );
    }

    #[test]
    fn active_avatar_rect_detection_uses_detected_multi_game_rects() {
        let root = unique_test_root("active-avatar-multi-game");
        let p_template = write_test_auto_fight_template(&root, AUTO_FIGHT_COOP_P_ASSET);
        let one_p_template = write_test_auto_fight_template(&root, AUTO_FIGHT_COOP_ONE_P_ASSET);
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        blit_bgr_image(&mut image, &one_p_template, 12, 9);
        blit_bgr_image(&mut image, &p_template, 1810, 230);
        blit_bgr_image(&mut image, &p_template, 1810, 290);

        let detection =
            combat_avatar_index_rect_detection_for_active_avatar_detection(&root, &image).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            detection.resolved_rects,
            vec![
                Rect::new(1859, 459, 28, 24).unwrap(),
                Rect::new(1859, 555, 28, 24).unwrap()
            ]
        );
        assert!(detection.message.contains("房主"));
        assert!(!detection.inferred_from_current_avatar_arrow);
    }

    #[test]
    fn active_skill_readiness_uses_legacy_bottom_cooldown_components() {
        let root = unique_test_root("active-skill-readiness");
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let ready = detect_combat_skill_readiness(&root, &image, 1, false).unwrap();
        assert_eq!(ready.status, CombatSkillReadinessStatus::Ready);
        assert_eq!(ready.ready, Some(true));
        assert_eq!(ready.white_component_count, 0);
        assert_eq!(
            ready.cooldown_rect,
            Some(Rect::new(1688, 988, 22, 12).unwrap())
        );

        let cooldown_rect = active_combat_skill_cooldown_rect(image.size, false).unwrap();
        fill_rect_gray(
            &mut image,
            Rect::new(cooldown_rect.x, cooldown_rect.y, 2, 2).unwrap(),
            255,
        );
        fill_rect_gray(
            &mut image,
            Rect::new(cooldown_rect.x + 8, cooldown_rect.y, 2, 2).unwrap(),
            255,
        );
        let cooldown = detect_combat_skill_readiness(&root, &image, 1, false).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            cooldown.status,
            CombatSkillReadinessStatus::CooldownOrUnavailable
        );
        assert_eq!(cooldown.ready, Some(false));
        assert_eq!(cooldown.white_component_count, 2);
        assert_eq!(cooldown.legacy_connected_component_labels, 3);
    }

    #[test]
    fn burst_readiness_uses_legacy_active_burst_cooldown_rect() {
        let root = unique_test_root("active-burst-readiness");
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let burst = detect_combat_skill_readiness(&root, &image, 1, true).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(burst.kind, CombatSkillReadinessKind::ElementalBurst);
        assert_eq!(burst.status, CombatSkillReadinessStatus::Ready);
        assert_eq!(
            burst.cooldown_rect,
            Some(Rect::new(1809, 968, 30, 15).unwrap())
        );
    }

    #[test]
    fn inactive_skill_readiness_reports_unsupported_without_guessing() {
        let root = unique_test_root("inactive-skill-readiness");
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let detection = detect_combat_skill_readiness(&root, &image, 2, false).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            detection.status,
            CombatSkillReadinessStatus::UnsupportedForInactiveAvatar
        );
        assert_eq!(detection.active_index, Some(1));
        assert_eq!(detection.ready, None);
        assert!(detection.cooldown_rect.is_none());
    }

    #[test]
    fn inactive_burst_readiness_uses_legacy_side_circle_probe() {
        let root = unique_test_root("inactive-burst-readiness");
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let empty = detect_combat_skill_readiness(&root, &image, 2, true).unwrap();
        assert_eq!(
            empty.status,
            CombatSkillReadinessStatus::CooldownOrUnavailable
        );
        assert_eq!(empty.ready, Some(false));
        assert_eq!(
            empty.side_burst_rect,
            Some(Rect::new(1584, 316, 64, 84).unwrap())
        );
        assert!(!empty.side_burst_circle.as_ref().unwrap().detected);

        let q_rect = combat_avatar_side_burst_rect_for_index(image.size, 2).unwrap();
        draw_circle_gray(&mut image, (q_rect.center().x, q_rect.center().y), 29, 255);
        let ready = detect_combat_skill_readiness(&root, &image, 2, true).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(ready.kind, CombatSkillReadinessKind::ElementalBurst);
        assert_eq!(ready.status, CombatSkillReadinessStatus::Ready);
        assert_eq!(ready.ready, Some(true));
        let circle = ready.side_burst_circle.unwrap();
        assert!(circle.detected);
        assert!(circle.best_votes >= AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES);
        assert_eq!(
            circle.best_center,
            Some((q_rect.center().x, q_rect.center().y))
        );
    }

    #[test]
    fn combat_team_recognition_classifies_side_icon_crops_into_team_plan() {
        let root = unique_test_root("team-recognition");
        write_test_combat_avatar_catalog(&root);
        let image = blank_bgr_image(Size::new(1920, 1080));
        let expected_rects = combat_avatar_side_icon_rects_for_index_rects(
            image.size,
            &default_combat_avatar_index_rects(image.size).unwrap(),
        )
        .unwrap();
        let classes = vec![
            ("Zhongli", 0.92),
            ("YelanCostumeYu", 0.52),
            ("Xingqiu", 0.83),
            ("Bennett", 0.78),
        ];
        let mut classifier = RecordingAvatarSideClassifier {
            classes,
            calls: Vec::new(),
        };

        let recognition =
            recognize_combat_team_from_avatar_side_icons(&root, &image, "", &mut classifier)
                .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            recognition.team_avatar_names,
            vec!["钟离", "夜兰", "行秋", "班尼特"]
        );
        assert_eq!(recognition.avatars[1].name_en, "Yelan");
        assert_eq!(recognition.avatars[1].costume_name.as_deref(), Some("Yu"));
        assert_eq!(recognition.avatars[1].display_name, "夜兰(玄玉瑶芳)");
        assert_eq!(recognition.team_plan.avatars.len(), 4);
        assert_eq!(recognition.team_plan.avatars[0].name_en, "Zhongli");
        assert_eq!(classifier.calls, expected_rects);
    }

    #[test]
    fn combat_team_recognition_rejects_low_confidence_side_classification() {
        let root = unique_test_root("team-recognition-low-confidence");
        write_test_combat_avatar_catalog(&root);
        let image = blank_bgr_image(Size::new(1920, 1080));
        let mut classifier = RecordingAvatarSideClassifier {
            classes: vec![("Zhongli", 0.69)],
            calls: Vec::new(),
        };

        let error =
            recognize_combat_team_from_avatar_side_icons(&root, &image, "", &mut classifier)
                .unwrap_err();
        let _ = fs::remove_dir_all(&root);

        assert!(error.to_string().contains("无法识别第1位角色"));
    }

    #[test]
    fn combat_command_execution_plan_maps_legacy_input_actions() {
        let script = parse_combat_script_context(
            "钟离 s(0.2), attack(0.4), wait(0.05), moveby(1800, -2100), keydown(VK_LBUTTON), keyup(VK_LBUTTON), scroll(-1)",
            true,
        )
        .unwrap();

        let plan = plan_combat_script_execution(&script).unwrap();

        assert_eq!(plan.commands.len(), 7);
        assert_eq!(
            plan.commands[0].switch_policy,
            CombatAvatarSwitchPolicy::EnsureSelectedBeforeAction
        );
        assert!(matches!(
            plan.commands[0].action,
            CombatCommandActionPlan::Walk {
                ref direction,
                duration_ms: 200
            } if direction == "s"
        ));
        assert_eq!(
            plan.commands[0].default_input_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::S.vk(),
                    extended: None,
                },
                InputEvent::Delay { milliseconds: 200 },
                InputEvent::KeyUp {
                    vk: KeyId::S.vk(),
                    extended: None,
                }
            ]
        );
        assert!(matches!(
            plan.commands[1].action,
            CombatCommandActionPlan::Attack {
                duration_ms: 400,
                click_interval_ms: COMBAT_ATTACK_INTERVAL_MILLISECONDS,
                repeat_count: 3,
            }
        ));
        assert_eq!(
            plan.commands[2].default_input_events,
            vec![InputEvent::Delay { milliseconds: 50 }]
        );
        assert_eq!(
            plan.commands[3].default_input_events,
            vec![InputEvent::MouseMoveRelative {
                dx: 1800,
                dy: -2100
            }]
        );
        assert_eq!(
            plan.commands[4].default_input_events,
            vec![InputEvent::MouseButtonDown {
                button: MouseButton::Left
            }]
        );
        assert_eq!(
            plan.commands[5].default_input_events,
            vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left
            }]
        );
        assert_eq!(
            plan.commands[6].default_input_events,
            vec![InputEvent::MouseWheel {
                amount: -120,
                horizontal: false,
            }]
        );
        let evaluation = evaluate_combat_script_playback(&plan);
        assert_eq!(evaluation.total_commands, 7);
        assert_eq!(evaluation.context_bound_commands, 2);
        assert_eq!(
            evaluation.first_blocking_requirements,
            vec![CombatExecutionContextRequirement::AvatarSelection]
        );
    }

    #[test]
    fn combat_command_execution_plan_tracks_switch_policy_and_skill_options() {
        let script = parse_combat_script_context(
            "钟离 e(hold,wait), wait(0.1)\n夜兰 q\n当前角色 keypress(q)",
            true,
        )
        .unwrap();

        let plan = plan_combat_script_execution(&script).unwrap();

        assert_eq!(
            plan.commands[0].switch_policy,
            CombatAvatarSwitchPolicy::EnsureSelectedBeforeAction
        );
        assert!(matches!(
            plan.commands[0].action,
            CombatCommandActionPlan::Skill {
                hold: true,
                variant: CombatSkillExecutionVariant::GenericHold,
                cooldown_policy: CombatSkillCooldownPolicy::WaitUntilReady,
                ..
            }
        ));
        assert_eq!(
            plan.commands[1].switch_policy,
            CombatAvatarSwitchPolicy::NoSwitch
        );
        assert_eq!(
            plan.commands[2].switch_policy,
            CombatAvatarSwitchPolicy::SwitchOnAvatarChange
        );
        assert_eq!(
            plan.commands[3].switch_policy,
            CombatAvatarSwitchPolicy::CurrentAvatar
        );
        assert!(matches!(
            plan.commands[3].action,
            CombatCommandActionPlan::KeyPress {
                key: CombatVirtualKeyPlan {
                    mapped_action: Some(GenshinAction::ElementalBurst),
                    ..
                }
            }
        ));
        assert_eq!(
            plan.commands[3].default_input_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::Q.vk(),
                    extended: None,
                },
                InputEvent::KeyUp {
                    vk: KeyId::Q.vk(),
                    extended: None,
                }
            ]
        );
    }

    #[test]
    fn combat_script_playback_plan_handles_static_current_avatar_macro() {
        let script = parse_combat_script_context(
            "当前角色 keydown(VK_LBUTTON), wait(0.05), moveby(200, -100), keyup(VK_LBUTTON)",
            true,
        )
        .unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();

        let execution =
            execute_static_combat_script_inputs(&plan, CombatCommandPlaybackMode::PlanOnly, None)
                .unwrap();

        assert_eq!(execution.mode, CombatCommandPlaybackMode::PlanOnly);
        assert_eq!(execution.total_commands, 4);
        assert_eq!(execution.static_ready_commands, 4);
        assert_eq!(execution.context_bound_commands, 0);
        assert!(!execution.dispatched);
        assert_eq!(
            execution.input_events,
            vec![
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left
                },
                InputEvent::Delay { milliseconds: 50 },
                InputEvent::MouseMoveRelative { dx: 200, dy: -100 },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left
                }
            ]
        );
    }

    #[test]
    fn combat_script_playback_rejects_context_bound_commands_before_dispatch() {
        let script = parse_combat_script_context("钟离 e(wait), wait(0.1)", true).unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();

        let error =
            execute_static_combat_script_inputs(&plan, CombatCommandPlaybackMode::PlanOnly, None)
                .unwrap_err();

        let TaskError::CombatStrategy(message) = error else {
            panic!("expected combat strategy error");
        };
        assert!(message.contains("native combat context"));
        assert!(message.contains("AvatarSelection"));
        assert!(message.contains("SkillCooldown"));
    }

    #[test]
    fn team_context_combat_playback_resolves_avatar_switches() {
        let script =
            parse_combat_script_context("钟离 e, wait(0.05)\n夜兰 click(right)", true).unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();
        let team_plan = CombatTeamPlan {
            avatars: vec![
                test_team_avatar(1, "钟离"),
                test_team_avatar(2, "夜兰"),
                test_team_avatar(3, "纳西妲"),
                test_team_avatar(4, "班尼特"),
            ],
            command_avatar_names: vec!["钟离".to_string(), "夜兰".to_string()],
            can_be_skipped_avatar_names: Vec::new(),
            all_command_avatars_can_be_skipped: false,
        };
        let executable_commands = script.commands.clone();

        let execution = execute_team_context_combat_script_inputs(
            &plan,
            &team_plan,
            &executable_commands,
            CombatCommandPlaybackMode::PlanOnly,
            None,
        )
        .unwrap();

        assert!(execution.dispatch_ready);
        assert_eq!(execution.candidate_commands, 3);
        assert_eq!(execution.playable_commands, 3);
        assert_eq!(execution.blocked_command_index, None);
        assert_eq!(execution.planned_commands[0].team_index, Some(1));
        assert_eq!(
            execution.planned_commands[0].resolved_context,
            vec![CombatExecutionContextRequirement::AvatarSelection]
        );
        assert_eq!(
            execution.planned_commands[0].switch_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::X.vk(),
                    extended: None,
                },
                InputEvent::KeyUp {
                    vk: KeyId::X.vk(),
                    extended: None,
                },
                InputEvent::KeyDown {
                    vk: KeyId::D1.vk(),
                    extended: None,
                },
                InputEvent::KeyUp {
                    vk: KeyId::D1.vk(),
                    extended: None,
                },
                InputEvent::Delay {
                    milliseconds: COMBAT_AVATAR_SWITCH_SETTLE_MILLISECONDS,
                },
            ]
        );
        assert_eq!(execution.planned_commands[1].switch_events, Vec::new());
        assert_eq!(execution.planned_commands[2].team_index, Some(2));
        assert_eq!(execution.planned_commands[2].switch_events.len(), 5);
        assert!(execution.input_events.len() > execution.planned_commands[0].action_events.len());
    }

    #[test]
    fn team_context_combat_playback_keeps_native_blockers() {
        let script = parse_combat_script_context("钟离 e(wait), wait(0.1)\n夜兰 q", true).unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();
        let team_plan = CombatTeamPlan {
            avatars: vec![
                test_team_avatar(1, "钟离"),
                test_team_avatar(2, "夜兰"),
                test_team_avatar(3, "纳西妲"),
                test_team_avatar(4, "班尼特"),
            ],
            command_avatar_names: vec!["钟离".to_string(), "夜兰".to_string()],
            can_be_skipped_avatar_names: Vec::new(),
            all_command_avatars_can_be_skipped: false,
        };
        let executable_commands = script.commands.clone();

        let execution = execute_team_context_combat_script_inputs(
            &plan,
            &team_plan,
            &executable_commands,
            CombatCommandPlaybackMode::PlanOnly,
            None,
        )
        .unwrap();

        assert!(!execution.dispatch_ready);
        assert_eq!(execution.playable_commands, 1);
        assert_eq!(execution.blocked_command_index, Some(0));
        assert_eq!(
            execution.planned_commands[0].resolved_context,
            vec![CombatExecutionContextRequirement::AvatarSelection]
        );
        assert_eq!(
            execution.planned_commands[0].pending_context,
            vec![CombatExecutionContextRequirement::SkillCooldown]
        );
        assert_eq!(
            execution.blocked_requirements,
            vec![CombatExecutionContextRequirement::SkillCooldown]
        );
        assert!(execution.planned_commands[2]
            .pending_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness));
    }

    #[test]
    fn frame_team_playback_resolves_active_skill_and_burst_readiness() {
        let root = unique_test_root("frame-team-playback");
        let script = parse_combat_script_context("钟离 e(wait), q", true).unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();
        let team_plan = CombatTeamPlan {
            avatars: vec![
                test_team_avatar(1, "钟离"),
                test_team_avatar(2, "夜兰"),
                test_team_avatar(3, "纳西妲"),
                test_team_avatar(4, "班尼特"),
            ],
            command_avatar_names: vec!["钟离".to_string()],
            can_be_skipped_avatar_names: Vec::new(),
            all_command_avatars_can_be_skipped: false,
        };
        let executable_commands = script.commands.clone();
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let execution = plan_team_context_combat_script_playback_with_frame(
            &root,
            &image,
            &plan,
            &team_plan,
            &executable_commands,
        )
        .unwrap();

        assert!(execution.dispatch_ready);
        assert_eq!(execution.blocked_command_index, None);
        assert!(execution.planned_commands[0]
            .resolved_context
            .contains(&CombatExecutionContextRequirement::SkillCooldown));
        assert!(execution.planned_commands[1]
            .resolved_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness));
        assert!(execution.planned_commands[0].switch_events.is_empty());

        let cooldown_rect = active_combat_skill_cooldown_rect(image.size, false).unwrap();
        fill_rect_gray(
            &mut image,
            Rect::new(cooldown_rect.x, cooldown_rect.y, 2, 2).unwrap(),
            255,
        );
        fill_rect_gray(
            &mut image,
            Rect::new(cooldown_rect.x + 8, cooldown_rect.y, 2, 2).unwrap(),
            255,
        );
        let blocked = plan_team_context_combat_script_playback_with_frame(
            &root,
            &image,
            &plan,
            &team_plan,
            &executable_commands,
        )
        .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(!blocked.dispatch_ready);
        assert_eq!(blocked.blocked_command_index, Some(0));
        assert_eq!(
            blocked.blocked_requirements,
            vec![CombatExecutionContextRequirement::SkillCooldown]
        );
    }

    #[test]
    fn frame_team_playback_resolves_inactive_burst_readiness_from_side_circle() {
        let root = unique_test_root("frame-team-inactive-burst");
        let script = parse_combat_script_context("夜兰 q", true).unwrap();
        let plan = plan_combat_script_execution(&script).unwrap();
        let team_plan = CombatTeamPlan {
            avatars: vec![
                test_team_avatar(1, "钟离"),
                test_team_avatar(2, "夜兰"),
                test_team_avatar(3, "纳西妲"),
                test_team_avatar(4, "班尼特"),
            ],
            command_avatar_names: vec!["夜兰".to_string()],
            can_be_skipped_avatar_names: Vec::new(),
            all_command_avatars_can_be_skipped: false,
        };
        let executable_commands = script.commands.clone();
        let mut image = blank_bgr_image(Size::new(1920, 1080));
        let index_rects = default_combat_avatar_index_rects(image.size).unwrap();
        fill_rect_gray(&mut image, index_rects[1], 255);
        fill_rect_gray(&mut image, index_rects[2], 255);
        fill_rect_gray(&mut image, index_rects[3], 255);

        let blocked = plan_team_context_combat_script_playback_with_frame(
            &root,
            &image,
            &plan,
            &team_plan,
            &executable_commands,
        )
        .unwrap();
        assert!(!blocked.dispatch_ready);
        assert!(blocked.planned_commands[0]
            .pending_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness));

        let q_rect = combat_avatar_side_burst_rect_for_index(image.size, 2).unwrap();
        draw_circle_gray(&mut image, (q_rect.center().x, q_rect.center().y), 29, 255);
        let ready = plan_team_context_combat_script_playback_with_frame(
            &root,
            &image,
            &plan,
            &team_plan,
            &executable_commands,
        )
        .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(ready.dispatch_ready);
        assert_eq!(ready.blocked_command_index, None);
        assert!(ready.planned_commands[0].switch_events.len() >= 5);
        assert!(ready.planned_commands[0]
            .resolved_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness));
    }

    #[test]
    fn action_scheduler_cd_parser_matches_legacy_boundaries_and_reverse_lookup() {
        let config = "钟离,12;钟离的朋友,3;夜兰;钟离,9;纳西妲,bad";

        assert_eq!(
            parse_action_scheduler_cd_for_avatar("钟离", config),
            Some(9.0)
        );
        assert_eq!(
            parse_action_scheduler_cd_for_avatar("夜兰", config),
            Some(-1.0)
        );
        assert_eq!(
            parse_action_scheduler_cd_for_avatar("纳西妲", config),
            Some(-1.0)
        );
        assert_eq!(parse_action_scheduler_cd_for_avatar("朋友", config), None);
        assert_eq!(parse_action_scheduler_cd_for_avatar("不存在", config), None);
    }

    #[test]
    fn action_scheduler_plan_marks_configured_command_avatars() {
        let script =
            parse_combat_script_context("钟离 e\n夜兰 q\n当前角色 keypress(q)\n纳西妲 e", true)
                .unwrap();

        let plan = plan_combat_script_action_scheduler(&script, "钟离,12;纳西妲");

        assert_eq!(
            plan.command_avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string(), "纳西妲".to_string()]
        );
        assert_eq!(
            plan.scheduler.configured_avatar_names,
            vec!["钟离".to_string(), "纳西妲".to_string()]
        );
        assert_eq!(
            plan.scheduler.skipped_avatar_names,
            vec!["钟离".to_string(), "纳西妲".to_string()]
        );
        assert!(!plan.scheduler.all_command_avatars_can_be_skipped);
        assert_eq!(plan.scheduler.entries[0].manual_skill_cd_seconds, 12.0);
        assert!(plan.scheduler.entries[0].has_explicit_cd);
        assert_eq!(plan.scheduler.entries[1].manual_skill_cd_seconds, -1.0);
        assert!(!plan.scheduler.entries[1].has_explicit_cd);
    }

    #[test]
    fn script_dispatcher_timer_maps_to_realtime_trigger_invocation() {
        let plan = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::AddRealtimeTimer(DispatcherTimerInput {
                name: "AutoPick".to_string(),
                interval_ms: 50,
                config: Some(serde_json::json!({"enabled": true})),
                clears_existing_triggers: true,
            }),
        )
        .unwrap();

        assert_eq!(plan.kind, TaskInvocationKind::AddRealtimeTrigger);
        assert_eq!(plan.task_key.as_deref(), Some("AutoPick"));
        assert_eq!(plan.interval_ms, Some(50));
        assert!(plan.clears_existing_triggers);
        assert_eq!(
            plan.catalog_entry.unwrap().launch_policy,
            TaskLaunchPolicy::RealtimeTick
        );
    }

    #[test]
    fn script_dispatcher_solo_task_maps_to_independent_invocation() {
        let plan = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::RunSoloTask(DispatcherSoloTaskInput {
                name: "AutoFight".to_string(),
                config: Some(serde_json::json!({"strategy": "default"})),
                uses_linked_cancellation: true,
            }),
        )
        .unwrap();

        assert_eq!(plan.kind, TaskInvocationKind::RunIndependentTask);
        assert_eq!(plan.task_key.as_deref(), Some("AutoFight"));
        assert!(plan.uses_linked_cancellation);
    }

    #[test]
    fn script_dispatcher_builtin_task_allows_script_dispatcher_and_solo_policies() {
        let auto_domain = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoDomain".to_string(),
                config: serde_json::json!({"domain": "sample"}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        assert_eq!(auto_domain.kind, TaskInvocationKind::RunIndependentTask);

        let auto_fishing = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoFishing".to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        assert_eq!(
            auto_fishing.kind,
            TaskInvocationKind::RunScriptDispatcherTask
        );

        let return_main_ui = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "ReturnMainUi".to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        assert_eq!(return_main_ui.kind, TaskInvocationKind::RunCommonJob);
        assert_eq!(return_main_ui.task_key.as_deref(), Some("ReturnMainUi"));
    }

    #[test]
    fn task_invocation_evaluation_reports_runtime_and_native_boundaries() {
        let clear = evaluate_task_invocation_plan(
            TaskInvocationPlan::from_script_dispatcher_command(
                ScriptDispatcherCommandInput::ClearAllTriggers,
            )
            .unwrap(),
            TaskInvocationExecutionMode::PlanOnly,
        );
        assert_eq!(clear.status, TaskInvocationExecutionStatus::Planned);
        assert!(!clear.executed);

        let auto_fight = evaluate_task_invocation_plan(
            TaskInvocationPlan::from_script_dispatcher_command(
                ScriptDispatcherCommandInput::RunBuiltinTask {
                    name: "AutoFight".to_string(),
                    config: serde_json::json!({"strategy": "default"}),
                    uses_linked_cancellation: true,
                },
            )
            .unwrap(),
            TaskInvocationExecutionMode::ExecuteReady,
        );
        assert_eq!(
            auto_fight.status,
            TaskInvocationExecutionStatus::NativePending
        );
        assert!(auto_fight.message.contains("AutoFight"));
        assert!(!auto_fight.executed);

        let token = evaluate_task_invocation_plan(
            TaskInvocationPlan::from_script_dispatcher_command(
                ScriptDispatcherCommandInput::LinkedCancellationToken,
            )
            .unwrap(),
            TaskInvocationExecutionMode::ExecuteReady,
        );
        assert_eq!(token.status, TaskInvocationExecutionStatus::RuntimeOnly);
        assert!(!token.is_actionable());
    }

    #[test]
    fn task_invocation_execution_mutates_realtime_dispatcher_state_only_for_supported_plans() {
        let mut dispatcher = DispatcherRuntime {
            frame_index: 42,
            ..DispatcherRuntime::default()
        };
        let first = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::AddRealtimeTimer(DispatcherTimerInput {
                name: "AutoPick".to_string(),
                interval_ms: 250,
                config: Some(serde_json::json!({"enabled": true})),
                clears_existing_triggers: false,
            }),
        )
        .unwrap();

        let first_result = execute_task_invocation_plan(&mut dispatcher, first);

        assert!(first_result.executed);
        assert_eq!(first_result.status, TaskInvocationExecutionStatus::Ready);
        assert_eq!(dispatcher.registered_realtime_triggers.len(), 1);
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].task_key,
            "AutoPick"
        );
        assert_eq!(dispatcher.registered_realtime_triggers[0].interval_ms, 250);
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].registered_at_frame,
            42
        );

        let second = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::AddRealtimeTimer(DispatcherTimerInput {
                name: "AutoSkip".to_string(),
                interval_ms: 100,
                config: None,
                clears_existing_triggers: true,
            }),
        )
        .unwrap();
        let second_result = execute_task_invocation_plan(&mut dispatcher, second);

        assert!(second_result.executed);
        assert_eq!(dispatcher.registered_realtime_triggers.len(), 1);
        assert_eq!(
            dispatcher.registered_realtime_triggers[0].task_key,
            "AutoSkip"
        );

        let auto_fight = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoFight".to_string(),
                config: serde_json::json!({"strategy": "default"}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let auto_fight_result = execute_task_invocation_plan(&mut dispatcher, auto_fight);

        assert!(!auto_fight_result.executed);
        assert_eq!(
            auto_fight_result.status,
            TaskInvocationExecutionStatus::NativePending
        );
        assert_eq!(dispatcher.registered_realtime_triggers.len(), 1);

        let clear = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::ClearAllTriggers,
        )
        .unwrap();
        let clear_result = execute_task_invocation_plan(&mut dispatcher, clear);

        assert!(clear_result.executed);
        assert!(dispatcher.registered_realtime_triggers.is_empty());
        assert!(clear_result.message.contains("cleared 1"));
    }

    #[test]
    fn script_dispatcher_invocation_rejects_wrong_catalog_policy() {
        let error = TaskInvocationPlan::from_script_dispatcher_command(
            ScriptDispatcherCommandInput::AddRealtimeTimer(DispatcherTimerInput {
                name: "AutoFight".to_string(),
                interval_ms: 50,
                config: None,
                clears_existing_triggers: false,
            }),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::InvalidLaunchPolicy {
                key,
                expected: TaskLaunchPolicy::RealtimeTick,
                actual: TaskLaunchPolicy::SoloTask
            } if key == "AutoFight"
        ));
    }

    #[test]
    fn shell_config_matches_legacy_defaults_and_camel_case() {
        let config = ShellConfig::default();
        assert!(!config.disable);
        assert_eq!(config.timeout, 60);
        assert!(config.no_window);
        assert!(config.output);

        let config = ShellConfig::from_value(Some(&serde_json::json!({
            "disable": true,
            "timeout": -1,
            "noWindow": false,
            "output": false
        })));
        assert!(config.disable);
        assert_eq!(config.timeout, -1);
        assert!(!config.no_window);
        assert!(!config.output);
    }

    #[test]
    fn shell_task_executes_and_captures_output() {
        let param = ShellTaskParam::build_from_config(
            shell_echo_command("hello-shell"),
            ShellConfig::default(),
            ".",
        );

        let result = execute_shell_task(&param).unwrap();

        assert_eq!(result.status, ShellExecutionStatus::Completed);
        assert!(result.waited_for_exit);
        assert!(result.has_output());
        assert!(
            format!("{}\n{}", result.output_shell, result.output).contains("hello-shell"),
            "unexpected shell output: {:?}",
            result
        );
    }

    #[test]
    fn shell_task_timeout_zero_starts_without_waiting() {
        let param = ShellTaskParam::build_from_config(
            shell_echo_command("fire-and-forget"),
            ShellConfig {
                timeout: 0,
                ..ShellConfig::default()
            },
            ".",
        );

        let result = execute_shell_task(&param).unwrap();

        assert_eq!(result.status, ShellExecutionStatus::Started);
        assert!(!result.waited_for_exit);
        assert_eq!(result.output_shell, "");
        assert_eq!(result.output, "");
    }

    #[test]
    fn shell_task_disabled_and_empty_commands_do_not_spawn() {
        let disabled = ShellTaskParam::build_from_config(
            "definitely-not-a-command",
            ShellConfig {
                disable: true,
                ..ShellConfig::default()
            },
            ".",
        );
        assert_eq!(
            execute_shell_task(&disabled).unwrap().status,
            ShellExecutionStatus::Disabled
        );

        let empty = ShellTaskParam::build_from_config("", ShellConfig::default(), ".");
        assert_eq!(
            execute_shell_task(&empty).unwrap().status,
            ShellExecutionStatus::EmptyCommand
        );
    }

    #[test]
    fn shell_task_reports_timeout() {
        let param = ShellTaskParam::build_from_config(
            shell_sleep_command(2),
            ShellConfig {
                timeout: 1,
                ..ShellConfig::default()
            },
            ".",
        );

        let result = execute_shell_task(&param).unwrap();

        assert_eq!(result.status, ShellExecutionStatus::TimedOut);
        assert!(result.waited_for_exit);
    }

    #[test]
    fn shell_task_can_be_cancelled() {
        let param = ShellTaskParam::build_from_config(
            shell_sleep_command(2),
            ShellConfig {
                timeout: 30,
                ..ShellConfig::default()
            },
            ".",
        );

        let result = execute_shell_task_with_cancel(&param, || true).unwrap();

        assert_eq!(result.status, ShellExecutionStatus::Cancelled);
        assert!(result.waited_for_exit);
    }

    #[test]
    fn independent_shell_task_executes_through_native_boundary() {
        let request = IndependentTaskExecutionRequest::shell(
            shell_echo_command("independent-shell-ok"),
            ShellConfig::default(),
            ".",
        );

        let result = execute_independent_task_with_cancel(&request, || false).unwrap();

        assert_eq!(result.task_key, "Shell");
        let IndependentTaskExecution::Shell(shell) = result.execution else {
            panic!("expected shell execution result");
        };
        assert_eq!(shell.status, ShellExecutionStatus::Completed);
        assert!(
            format!("{}\n{}", shell.output_shell, shell.output).contains("independent-shell-ok"),
            "unexpected shell output: {:?}",
            shell
        );
    }

    #[test]
    fn independent_use_redeem_code_returns_execution_plan() {
        let codes = redeem_code_entries_from_strings(["ABCD1234EFGH"].into_iter());
        let request =
            IndependentTaskExecutionRequest::use_redeem_code(codes, Size::new(1920, 1080), ".");

        let result = execute_independent_task_with_cancel(&request, || false).unwrap();

        assert_eq!(result.task_key, "UseRedeemCode");
        let IndependentTaskExecution::UseRedeemCodePlan(plan) = result.execution else {
            panic!("expected use-redeem-code execution plan");
        };
        assert_eq!(plan.task_key, "UseRedeemCode");
        assert_eq!(plan.codes.len(), 1);
        assert_eq!(plan.codes[0].code, "ABCD1234EFGH");
        assert!(!plan.executor_ready);
        assert!(plan.steps.len() > 10);
    }

    #[test]
    fn independent_auto_pathing_returns_execution_plan_for_user_route() {
        let root = unique_test_root("auto-pathing");
        let route_dir = root.join("User").join("AutoPathing").join("liyue");
        fs::create_dir_all(&route_dir).unwrap();
        fs::write(
            route_dir.join("route.json"),
            r#"{
                "info": { "name": "mining route", "type": "mining", "map_name": "Teyvat" },
                "config": { "realtime_triggers": { "AutoPick": true } },
                "positions": [
                    { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                    { "x": 3.0, "y": 4.0, "type": "target", "action": "fight" }
                ]
            }"#,
        )
        .unwrap();
        let request = IndependentTaskExecutionRequest::auto_pathing("liyue/route.json", &root);

        let result = execute_independent_task_with_cancel(&request, || false).unwrap();

        assert_eq!(result.task_key, "AutoPathing");
        let IndependentTaskExecution::AutoPathingPlan(plan) = result.execution else {
            panic!("expected auto-pathing execution plan");
        };
        assert_eq!(plan.summary.name, "mining route");
        assert_eq!(
            plan.normalized_path,
            PathBuf::from("liyue").join("route.json")
        );
        assert_eq!(plan.execution_plan.segment_count, 1);
        assert_eq!(plan.execution_plan.waypoint_count, 2);
        assert_eq!(plan.execution_plan.expected_fight_count, 1);
        assert!(plan.execution_plan.autopick_realtime_trigger_enabled);
        assert!(!plan.dispatched);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn independent_auto_pathing_rejects_paths_outside_user_root() {
        let request = IndependentTaskExecutionRequest::auto_pathing("../route.json", ".");

        let error = execute_independent_task_with_cancel(&request, || false).unwrap_err();

        assert!(matches!(error, TaskError::InvalidPathingRoute(route) if route == "../route.json"));
    }

    #[test]
    fn independent_auto_fight_returns_combat_strategy_plan() {
        let root = unique_test_root("auto-fight");
        let strategy_dir = root.join("User").join("AutoFight");
        fs::create_dir_all(&strategy_dir).unwrap();
        fs::write(
            strategy_dir.join("daily.txt"),
            "帝君 e(hold), wait(0.2)\nYelan round(1-2), keypress(q)",
        )
        .unwrap();
        let mut param = AutoFightParam::new(Some("daily"));
        param.action_scheduler_by_cd = "钟离,12;夜兰".to_string();
        param.team_names = "钟离,夜兰,秋秋人,班爷".to_string();
        param.fight_finish_detect_enabled = true;
        param.finish_detect_config.rotate_find_enemy_enabled = true;
        param.finish_detect_config.fast_check_enabled = true;
        param.check_before_burst = true;
        param.pick_drops_after_fight_enabled = true;
        param.battle_threshold_for_loot = 2;
        param.exp_based_pickup_enabled = true;
        write_test_combat_avatar_catalog(&root);
        let request = IndependentTaskExecutionRequest {
            task_key: "AutoFight".to_string(),
            command: None,
            config: serde_json::to_value(AutoFightExecutionConfig { param }).ok(),
            working_directory: root.clone(),
        };

        let result = execute_independent_task_with_cancel(&request, || false).unwrap();

        assert_eq!(result.task_key, "AutoFight");
        let IndependentTaskExecution::AutoFightPlan(plan) = result.execution else {
            panic!("expected auto-fight execution plan");
        };
        assert_eq!(plan.param.combat_strategy_path, "User/AutoFight/daily.txt");
        assert!(!plan.dispatched);
        assert_eq!(plan.combat_scripts.scripts.len(), 1);
        assert_eq!(plan.combat_scripts.scripts[0].name, "daily");
        assert_eq!(plan.combat_scripts.scripts[0].commands.len(), 3);
        assert_eq!(
            plan.combat_scripts.scripts[0].commands[2].activating_rounds,
            vec![1, 2]
        );
        assert_eq!(plan.script_execution_plans.len(), 1);
        assert_eq!(plan.playback_evaluation.total_commands, 3);
        assert_eq!(plan.playback_evaluation.context_bound_commands, 2);
        assert!(!plan.playback_evaluation.dispatch_ready);
        assert_eq!(
            plan.team_selection.status,
            CombatScriptTeamSelectionStatus::PartialFallback
        );
        assert_eq!(
            plan.team_selection.executable_avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string()]
        );
        assert_eq!(plan.team_selection.executable_commands.len(), 3);
        let team_plan = plan.team_plan.as_ref().expect("expected team plan");
        assert_eq!(team_plan.avatars.len(), 4);
        assert_eq!(team_plan.avatars[0].index, 1);
        assert_eq!(team_plan.avatars[0].name, "钟离");
        assert_eq!(team_plan.avatars[0].skill_cd_seconds, Some(4.0));
        assert_eq!(team_plan.avatars[0].skill_hold_cd_seconds, Some(12.0));
        assert_eq!(team_plan.avatars[0].manual_skill_cd_seconds, 12.0);
        assert!(team_plan.avatars[0].action_scheduler_configured);
        assert_eq!(team_plan.avatars[2].name, "行秋");
        assert!(!team_plan.avatars[2].action_scheduler_configured);
        assert_eq!(
            team_plan.command_avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string()]
        );
        assert_eq!(
            team_plan.can_be_skipped_avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string()]
        );
        assert!(team_plan.all_command_avatars_can_be_skipped);
        assert_eq!(plan.fight_loop_plan.command_count, 3);
        assert_eq!(plan.fight_loop_plan.executable_command_count, 3);
        assert!(plan.fight_loop_plan.rotate_find_enemy_enabled);
        assert!(plan.fight_loop_plan.check_before_burst_enabled);
        assert!(plan.fight_loop_plan.kazuha_pickup_enabled);
        assert!(plan.fight_loop_plan.pickup_drops_after_fight_enabled);
        assert!(!plan.fight_loop_plan.native_dispatch_ready);
        assert!(plan.fight_loop_plan.steps.iter().any(|step| {
            step.enabled && step.kind == CombatFightLoopStepKind::WaitAllConfiguredSkillCooldowns
        }));
        assert!(plan.fight_loop_plan.steps.iter().any(|step| {
            step.enabled
                && step.kind == CombatFightLoopStepKind::InitialSeekEnemy
                && step.command_index == Some(0)
        }));
        assert!(plan.fight_loop_plan.steps.iter().any(|step| {
            step.enabled
                && step.kind == CombatFightLoopStepKind::CheckBeforeBurst
                && step.command_index == Some(2)
        }));
        assert!(plan
            .fight_loop_plan
            .steps
            .iter()
            .any(|step| { step.enabled && step.kind == CombatFightLoopStepKind::ScanPickDrops }));
        assert!(plan.fight_loop_plan.steps.iter().any(|step| {
            step.enabled && step.kind == CombatFightLoopStepKind::ApplyBattleThresholdForLoot
        }));
        assert_eq!(
            plan.finish_detection_plan.progress_pixel,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL
        );
        assert_eq!(
            plan.finish_detection_plan.white_tile_pixel,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL
        );
        assert_eq!(
            plan.finish_detection_plan.pre_detect_delay_ms,
            AUTO_FIGHT_DEFAULT_FINISH_DELAY_MS
        );
        assert_eq!(
            plan.finish_detection_plan.detect_delay_ms,
            AUTO_FIGHT_DEFAULT_FINISH_DETECT_DELAY_MS
        );
        assert!(plan
            .finish_detection_plan
            .steps
            .iter()
            .any(|step| step.enabled && step.kind == AutoFightFinishDetectionStepKind::SeekEnemy));
        assert_eq!(plan.action_scheduler_plans.len(), 1);
        assert!(
            plan.action_scheduler_plans[0]
                .scheduler
                .all_command_avatars_can_be_skipped
        );
        assert_eq!(
            plan.action_scheduler_plans[0]
                .scheduler
                .skipped_avatar_names,
            vec!["钟离".to_string(), "夜兰".to_string()]
        );
        assert_eq!(plan.script_execution_plans[0].commands.len(), 3);
        assert!(matches!(
            plan.script_execution_plans[0].commands[0].action,
            CombatCommandActionPlan::Skill {
                hold: true,
                cooldown_policy: CombatSkillCooldownPolicy::None,
                ..
            }
        ));
        assert_eq!(
            plan.script_execution_plans[0].commands[2].default_input_events,
            vec![
                InputEvent::KeyDown {
                    vk: KeyId::Q.vk(),
                    extended: None,
                },
                InputEvent::KeyUp {
                    vk: KeyId::Q.vk(),
                    extended: None,
                }
            ]
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn independent_auto_fight_rejects_paths_outside_user_root() {
        let mut param = AutoFightParam::default();
        param.combat_strategy_path = "../fight.txt".to_string();
        let request = IndependentTaskExecutionRequest {
            task_key: "AutoFight".to_string(),
            command: None,
            config: serde_json::to_value(AutoFightExecutionConfig { param }).ok(),
            working_directory: ".".into(),
        };

        let error = execute_independent_task_with_cancel(&request, || false).unwrap_err();

        assert!(
            matches!(error, TaskError::InvalidCombatStrategyPath(path) if path == "../fight.txt")
        );
    }

    #[test]
    fn independent_native_task_reports_pending_until_ported() {
        let request = IndependentTaskExecutionRequest {
            task_key: "AutoDomain".to_string(),
            command: None,
            config: None,
            working_directory: ".".into(),
        };

        let result = execute_independent_task_with_cancel(&request, || false).unwrap();

        assert_eq!(result.task_key, "AutoDomain");
        let IndependentTaskExecution::NativePending(pending) = result.execution else {
            panic!("expected native-pending execution result");
        };
        assert_eq!(pending.status, TaskInvocationExecutionStatus::NativePending);
        assert!(!pending.executed);
    }

    #[cfg(windows)]
    fn shell_echo_command(message: &str) -> String {
        format!("echo {message} & exit")
    }

    #[cfg(not(windows))]
    fn shell_echo_command(message: &str) -> String {
        format!("echo {message}; exit")
    }

    #[cfg(windows)]
    fn shell_sleep_command(seconds: u64) -> String {
        format!("ping -n {} 127.0.0.1 > nul & exit", seconds + 1)
    }

    #[cfg(not(windows))]
    fn shell_sleep_command(seconds: u64) -> String {
        format!("sleep {seconds}; exit")
    }

    fn test_team_avatar(index: usize, name: &str) -> CombatTeamAvatarPlan {
        CombatTeamAvatarPlan {
            index,
            name: name.to_string(),
            id: format!("avatar-{index}"),
            name_en: name.to_string(),
            weapon: "Sword".to_string(),
            skill_cd_seconds: None,
            skill_hold_cd_seconds: None,
            burst_cd_seconds: None,
            manual_skill_cd_seconds: -1.0,
            action_scheduler_configured: false,
        }
    }

    fn unique_test_root(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bgi-task-{name}-{nonce}"))
    }

    fn write_test_combat_avatar_catalog(root: &Path) {
        let path = root.join(COMBAT_AVATAR_CATALOG_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            r#"[
  {
    "alias": ["钟离", "Zhongli", "帝君"],
    "burstCD": 12,
    "id": "10000030",
    "name": "钟离",
    "nameEn": "Zhongli",
    "skillCD": 4,
    "skillHoldCD": 12,
    "weapon": "13"
  },
  {
    "alias": ["夜兰", "Yelan"],
    "burstCD": 18,
    "id": "10000060",
    "name": "夜兰",
    "nameEn": "Yelan",
    "skillCD": 10,
    "weapon": "12"
  },
  {
    "alias": ["行秋", "秋秋人"],
    "burstCD": 20,
    "id": "10000025",
    "name": "行秋",
    "nameEn": "Xingqiu",
    "skillCD": 21,
    "weapon": "1"
  },
  {
    "alias": ["班尼特", "班爷"],
    "burstCD": 15,
    "id": "10000032",
    "name": "班尼特",
    "nameEn": "Bennett",
    "skillCD": 5,
    "weapon": "1"
  },
  {
    "alias": ["枫原万叶", "叶天帝", "万叶"],
    "burstCD": 15,
    "id": "10000047",
    "name": "枫原万叶",
    "nameEn": "Kazuha",
    "skillCD": 6,
    "skillHoldCD": 9,
    "weapon": "1"
  }
]"#,
        )
        .unwrap();
    }

    fn write_test_index_templates(root: &Path) -> Vec<BgrImage> {
        let patterns = [
            [255, 0, 0, 0, 255, 0, 0, 0, 255],
            [0, 255, 0, 255, 255, 255, 0, 255, 0],
            [255, 255, 255, 0, 255, 0, 0, 255, 0],
            [255, 0, 255, 0, 255, 0, 255, 0, 255],
        ];
        AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS
            .iter()
            .copied()
            .enumerate()
            .map(|(index, asset_name)| {
                let pixels = patterns[index]
                    .into_iter()
                    .flat_map(|gray| [gray, gray, gray])
                    .collect();
                let template = BgrImage::new(Size::new(3, 3), pixels).unwrap();
                let path = common_element_asset_path(root, asset_name);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                template.write_png(&path).unwrap();
                template
            })
            .collect()
    }

    fn write_test_current_avatar_arrow_template(root: &Path) -> BgrImage {
        let template = BgrImage::new(
            Size::new(2, 2),
            vec![255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255],
        )
        .unwrap();
        let path = common_element_asset_path(root, AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        template.write_png(&path).unwrap();
        template
    }

    fn write_test_auto_fight_template(root: &Path, asset_name: &str) -> BgrImage {
        let template = BgrImage::new(
            Size::new(2, 2),
            vec![0, 0, 0, 255, 255, 255, 80, 80, 80, 200, 200, 200],
        )
        .unwrap();
        let path = auto_fight_asset_path(root, asset_name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        template.write_png(&path).unwrap();
        template
    }

    fn common_element_asset_path(root: &Path, asset_name: &str) -> PathBuf {
        root.join("GameTask")
            .join("Common")
            .join("Element")
            .join("Assets")
            .join("1920x1080")
            .join(asset_name)
    }

    fn auto_fight_asset_path(root: &Path, asset_name: &str) -> PathBuf {
        root.join("GameTask")
            .join(AUTO_FIGHT_FEATURE)
            .join("Assets")
            .join("1920x1080")
            .join(asset_name)
    }

    struct RecordingAvatarSideClassifier {
        classes: Vec<(&'static str, f32)>,
        calls: Vec<Rect>,
    }

    impl CombatAvatarSideClassifier for RecordingAvatarSideClassifier {
        fn classify_avatar_side(
            &mut self,
            index: usize,
            image: &BgrImage,
            side_icon_rect: Rect,
        ) -> Result<CombatAvatarSideClassification> {
            assert_eq!(image.size, Size::new(82, 82));
            self.calls.push(side_icon_rect);
            let (class_name, confidence) = self.classes[index - 1];
            Ok(CombatAvatarSideClassification {
                class_name: class_name.to_string(),
                confidence,
            })
        }
    }

    fn blank_bgr_image(size: Size) -> BgrImage {
        BgrImage::new(
            size,
            vec![0; size.width as usize * size.height as usize * 3],
        )
        .unwrap()
    }

    fn small_avatar_index_rects() -> Vec<Rect> {
        vec![
            Rect::new(0, 0, 4, 4).unwrap(),
            Rect::new(5, 0, 4, 4).unwrap(),
            Rect::new(10, 0, 4, 4).unwrap(),
            Rect::new(15, 0, 4, 4).unwrap(),
        ]
    }

    fn set_bgr_pixel(image: &mut BgrImage, position: (u32, u32), pixel: RgbPixel) {
        let index = ((position.1 * image.size.width + position.0) as usize) * 3;
        image.pixels[index] = pixel.b;
        image.pixels[index + 1] = pixel.g;
        image.pixels[index + 2] = pixel.r;
    }

    fn fill_rect_gray(image: &mut BgrImage, rect: Rect, gray: u8) {
        for y in rect.y as u32..rect.bottom() as u32 {
            for x in rect.x as u32..rect.right() as u32 {
                set_bgr_pixel(
                    image,
                    (x, y),
                    RgbPixel {
                        r: gray,
                        g: gray,
                        b: gray,
                    },
                );
            }
        }
    }

    fn draw_circle_gray(image: &mut BgrImage, center: (i32, i32), radius: i32, gray: u8) {
        for sample in 0..192 {
            let angle = sample as f64 * std::f64::consts::TAU / 192.0;
            let x = center.0 + (radius as f64 * angle.cos()).round() as i32;
            let y = center.1 + (radius as f64 * angle.sin()).round() as i32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0
                        && py >= 0
                        && px < image.size.width as i32
                        && py < image.size.height as i32
                    {
                        set_bgr_pixel(
                            image,
                            (px as u32, py as u32),
                            RgbPixel {
                                r: gray,
                                g: gray,
                                b: gray,
                            },
                        );
                    }
                }
            }
        }
    }

    fn blit_bgr_image(target: &mut BgrImage, source: &BgrImage, x: u32, y: u32) {
        for source_y in 0..source.size.height {
            for source_x in 0..source.size.width {
                let target_index =
                    (((y + source_y) * target.size.width + x + source_x) as usize) * 3;
                let source_index = ((source_y * source.size.width + source_x) as usize) * 3;
                target.pixels[target_index..target_index + 3]
                    .copy_from_slice(&source.pixels[source_index..source_index + 3]);
            }
        }
    }

    fn draw_rect_edge_gray(image: &mut BgrImage, rect: Rect, gray: u8) {
        for x in rect.x as u32..rect.right() as u32 {
            set_bgr_pixel(
                image,
                (x, rect.y as u32),
                RgbPixel {
                    r: gray,
                    g: gray,
                    b: gray,
                },
            );
            set_bgr_pixel(
                image,
                (x, (rect.bottom() - 1) as u32),
                RgbPixel {
                    r: gray,
                    g: gray,
                    b: gray,
                },
            );
        }
        for y in rect.y as u32 + 1..(rect.bottom() - 1) as u32 {
            set_bgr_pixel(
                image,
                (rect.x as u32, y),
                RgbPixel {
                    r: gray,
                    g: gray,
                    b: gray,
                },
            );
            set_bgr_pixel(
                image,
                ((rect.right() - 1) as u32, y),
                RgbPixel {
                    r: gray,
                    g: gray,
                    b: gray,
                },
            );
        }
    }

    #[test]
    fn task_parameter_models_preserve_legacy_defaults() {
        let params = task_parameter_models();

        assert!(params.auto_skip.enabled);
        assert!(params.auto_skip.quickly_skip_conversations_enabled);
        assert_eq!(params.auto_skip.dialogue_option_voice_max_wait_seconds, 30);
        assert!(params.auto_skip.is_click_first_chat_option());
        assert_eq!(
            params.auto_skip.picture_in_picture_source_type,
            "CaptureLoop"
        );

        assert_eq!(params.auto_domain.domain_round_num, 9999);
        assert_eq!(
            params.auto_domain.resin_priority_list,
            vec!["浓缩树脂".to_string(), "原粹树脂".to_string()]
        );
        assert_eq!(params.auto_domain.max_artifact_star, "4");

        assert_eq!(params.auto_boss.strategy_name, AUTO_STRATEGY_NAME);
        assert_eq!(params.auto_boss.run_count, 1);
        assert_eq!(params.auto_boss.revive_retry_count, 3);

        assert_eq!(params.auto_fight.timeout, 120);
        assert_eq!(params.auto_fight.pick_drops_after_fight_seconds, 15);
        assert_eq!(params.auto_fight.battle_threshold_for_loot, -1);
        assert!(params.auto_fight.kazuha_pickup_enabled);

        assert!(
            params
                .auto_ley_line_outcrop
                .fight_config
                .fight_finish_detect_enabled
        );
        assert_eq!(
            params
                .auto_ley_line_outcrop
                .fight_config
                .seek_enemy_rotary_factor,
            6
        );

        assert_eq!(
            params.auto_stygian_onslaught.resin_priority_list,
            vec!["浓缩树脂".to_string(), "原粹树脂".to_string()]
        );
    }

    #[test]
    fn task_parameter_models_match_legacy_strategy_path_rules() {
        assert_eq!(
            combat_strategy_path(Some(AUTO_STRATEGY_NAME)),
            "User/AutoFight/"
        );
        assert_eq!(
            combat_strategy_path(Some("daily")),
            "User/AutoFight/daily.txt"
        );

        let domain = AutoDomainParam::new(0, Some("daily"));
        assert_eq!(domain.domain_round_num, 9999);
        assert_eq!(domain.combat_strategy_path, "User/AutoFight/daily.txt");

        let mut boss = AutoBossParam::default();
        boss.set_strategy_name(Some("boss"));
        assert_eq!(boss.combat_strategy_path, "User/AutoFight/boss.txt");
        boss.set_run_count(0);
        assert_eq!(boss.run_count, 1);
        boss.use_fragile_resin = true;
        boss.use_transient_resin = true;
        boss.set_specify_run_count(false);
        assert!(!boss.use_fragile_resin);
        assert!(!boss.use_transient_resin);

        let fight = AutoFightParam::new(Some("abyss"));
        assert_eq!(fight.combat_strategy_path, "User/AutoFight/abyss.txt");

        let ley_line = AutoLeyLineOutcropParam::new(3, "蒙德", "启示之花");
        assert_eq!(ley_line.count, 3);
        assert_eq!(ley_line.country, "蒙德");
        assert_eq!(ley_line.ley_line_outcrop_type, "启示之花");

        let mut stygian = AutoStygianOnslaughtParam::default();
        stygian.set_combat_strategy_path(Some("stygian"));
        assert_eq!(stygian.combat_script_bag_path, "User/AutoFight/stygian.txt");
        stygian.set_resin_priority_list(["须臾树脂", "脆弱树脂"]);
        assert_eq!(
            stygian.resin_priority_list,
            vec!["须臾树脂".to_string(), "脆弱树脂".to_string()]
        );
    }

    #[test]
    fn redeem_code_clipboard_extraction_matches_legacy_regex() {
        let codes = extract_redeem_codes_from_text(
            "prefix GENSHINGIFT1 123456789012 ABCDEFGHIJKL abcdefghijkl XABCDE123456Z",
        );

        assert_eq!(
            codes,
            vec!["GENSHINGIFT1".to_string(), "ABCDEFGHIJKL".to_string()]
        );
    }

    #[test]
    fn redeem_code_plan_preserves_legacy_navigation_and_code_flow() {
        let plan = plan_use_redeem_codes(
            vec![
                RedeemCodeEntry::new(" GENSHINGIFT1 ", Some("primogems".to_string())).unwrap(),
                RedeemCodeEntry::new("ABCDEFGHIJKL", None).unwrap(),
            ],
            Size::new(1920, 1080),
        )
        .unwrap();

        assert_eq!(plan.task_key, "UseRedeemCode");
        assert_eq!(plan.port_state, TaskPortState::RuntimeScaffolded);
        assert!(!plan.executor_ready);
        assert_eq!(plan.codes.len(), 2);
        assert_eq!(plan.steps.len(), 10 + 8 * 2 + 2);
        assert_eq!(
            plan.steps[1].label,
            "return to main UI before opening settings"
        );
        assert_eq!(plan.steps[4].label, "click settings button");
        assert!(matches!(
            plan.steps[2].action,
            UseRedeemCodeStepAction::Input { .. }
        ));
        let UseRedeemCodeStepAction::Input { events } = &plan.steps[2].action else {
            unreachable!()
        };
        assert_eq!(
            events,
            &vec![
                InputEvent::KeyDown {
                    vk: VK_ESCAPE,
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: VK_ESCAPE,
                    extended: None
                }
            ]
        );

        let account_step = &plan.steps[6];
        let UseRedeemCodeStepAction::Locator { locator } = &account_step.action else {
            unreachable!()
        };
        assert_eq!(locator.operation, BvLocatorOperation::Click);
        assert_eq!(
            locator.recognition_object.region_of_interest,
            Some(Rect::new(0, 0, 384, 1080).unwrap())
        );

        let go_redeem_step = &plan.steps[8];
        let UseRedeemCodeStepAction::Locator { locator } = &go_redeem_step.action else {
            unreachable!()
        };
        assert_eq!(
            locator.recognition_object.region_of_interest,
            Some(Rect::new(1344, 0, 576, 1080).unwrap())
        );

        let per_code_steps: Vec<_> = plan
            .steps
            .iter()
            .filter(|step| step.code.as_deref() == Some("GENSHINGIFT1"))
            .collect();
        assert_eq!(per_code_steps.len(), 8);
        assert!(matches!(
            per_code_steps[1].action,
            UseRedeemCodeStepAction::ClipboardSet { .. }
        ));
        assert_eq!(
            per_code_steps[5].condition,
            UseRedeemCodeStepCondition::WhenSuccessDetected
        );
        assert_eq!(
            per_code_steps[7].condition,
            UseRedeemCodeStepCondition::WhenSuccessNotDetected
        );

        let UseRedeemCodeStepAction::Locator { locator } = &per_code_steps[3].action else {
            unreachable!()
        };
        assert_eq!(
            locator.recognition_object.name.as_deref(),
            Some(COMMON_BTN_WHITE_CONFIRM)
        );
        assert!(locator.recognition_object.template.use_3_channels);

        let UseRedeemCodeStepAction::Locator { locator } = &per_code_steps[5].action else {
            unreachable!()
        };
        assert_eq!(
            locator.recognition_object.name.as_deref(),
            Some(COMMON_BTN_BLACK_CONFIRM)
        );
        assert!(locator.recognition_object.template.use_3_channels);
    }

    #[test]
    fn redeem_code_catalog_is_runtime_scaffolded_solo_task() {
        let entry = find_task_catalog_entry("UseRedeemCode").unwrap();

        assert_eq!(entry.launch_policy, TaskLaunchPolicy::SoloTask);
        assert_eq!(entry.port_state, TaskPortState::RuntimeScaffolded);
        assert!(entry.asset_roots.contains(&"GameTask/UseRedeemCode/Assets"));
        assert!(entry
            .asset_roots
            .contains(&"GameTask/Common/Element/Assets"));
    }
}
