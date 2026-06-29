use crate::{
    BattlePassClaimAllRule, BattlePassClaimScope, BattlePassManualSelectionDialogRule,
    BattlePassRewardStep, BattlePassRewardStepAction, BattlePassRewardStepCondition,
    BattlePassRewardStepPhase, BattlePassRewardStepResult,
    BlessingOfTheWelkinMoonDetectionLocators, BlessingOfTheWelkinMoonExecutionPlan,
    BlessingOfTheWelkinMoonLoopRule, BlessingOfTheWelkinMoonServerTimeGate,
    BlessingOfTheWelkinMoonStep, BlessingOfTheWelkinMoonStepAction,
    BlessingOfTheWelkinMoonStepCondition, CheckRewardsExecutionPlan, CheckRewardsRetryRule,
    CheckRewardsStep, CheckRewardsStepAction, CheckRewardsStepCondition, CheckRewardsStepPhase,
    CheckRewardsStepResult, ChooseTalkOptionExecutionPlan, ChooseTalkOptionOcrRule,
    ChooseTalkOptionOrangeRule, ChooseTalkOptionStep, ChooseTalkOptionStepAction,
    ChooseTalkOptionStepCondition, ClaimBattlePassRewardsExecutionPlan,
    ClaimEncounterPointsRewardsExecutionPlan, ClaimEncounterPointsRewardsOcrRule,
    ClaimEncounterPointsRewardsStep, ClaimEncounterPointsRewardsStepAction,
    ClaimEncounterPointsRewardsStepCondition, ClaimEncounterPointsRewardsStepResult,
    ClaimMailRewardsExecutionPlan, ClaimMailRewardsStep, ClaimMailRewardsStepAction,
    ClaimMailRewardsStepCondition, ClaimMailRewardsStepPhase, ClaimMailRewardsStepResult,
    CommonJobExecutionPlan, CommonJobStep, CommonJobStepAction, CommonJobStepCondition,
    CommonJobStepPhase, CountInventoryItemExecutionPlan, CountInventoryItemStep,
    CountInventoryItemStepAction, CountInventoryItemStepCondition, CountInventoryItemStepPhase,
    CountInventoryOpenInventoryRule, CountInventoryResultContract, CountInventorySearchMode,
    GoToAdventurersGuildExecutionPlan, GoToAdventurersGuildInteractionRule,
    GoToAdventurersGuildPathingRule, GoToAdventurersGuildStep, GoToAdventurersGuildStepAction,
    GoToAdventurersGuildStepCondition, GoToAdventurersGuildStepPhase,
    GoToAdventurersGuildStepResult, GoToCraftingBenchActionPress, GoToCraftingBenchExecutionPlan,
    GoToCraftingBenchInteractionRule, GoToCraftingBenchPathingRule,
    GoToCraftingBenchResinCraftRule, GoToCraftingBenchResinRecognitionRule, GoToCraftingBenchStep,
    GoToCraftingBenchStepAction, GoToCraftingBenchStepCondition, GoToCraftingBenchStepPhase,
    GoToCraftingBenchStepResult, GoToSereniteaPotActionPress, GoToSereniteaPotBagEntryRule,
    GoToSereniteaPotEntryMode, GoToSereniteaPotExecutionPlan, GoToSereniteaPotFindAYuanRule,
    GoToSereniteaPotFinishRule, GoToSereniteaPotMapEntryRule, GoToSereniteaPotRewardRule,
    GoToSereniteaPotShopRule, GoToSereniteaPotStep, GoToSereniteaPotStepAction,
    GoToSereniteaPotStepCondition, GoToSereniteaPotStepPhase, GoToSereniteaPotStepResult,
    GridIconClassifierRule, GridIconCropRule, GridItemCountOcrRule, GridItemDetectionRule,
    GridScrollRule, GridTemplate, LowerHeadThenWalkToActionPress, LowerHeadThenWalkToExecutionPlan,
    LowerHeadThenWalkToFKeyRule, LowerHeadThenWalkToMovementRule, LowerHeadThenWalkToStep,
    LowerHeadThenWalkToStepAction, LowerHeadThenWalkToStepCondition, LowerHeadThenWalkToStepPhase,
    LowerHeadThenWalkToStepResult, OneKeyExpeditionExecutionPlan, OneKeyExpeditionStep,
    OneKeyExpeditionStepAction, OneKeyExpeditionStepCondition, OneKeyExpeditionStepPhase,
    OneKeyExpeditionStepResult, ReloginExecutionPlan, ReloginFailurePolicy, ReloginRetryAction,
    ReloginRetryRule, ReloginStep, ReloginStepAction, ReloginStepCondition, ReloginStepPhase,
    ReloginStepResult, ReloginThirdPartyRule, Result, ReturnMainUiExecutionPlan,
    ScanPickCameraResetRule, ScanPickDropsActionPress, ScanPickDropsExecutionPlan,
    ScanPickDropsStep, ScanPickDropsStepAction, ScanPickDropsStepCondition, ScanPickDropsStepPhase,
    ScanPickDropsStepResult, ScanPickMovementRule, ScanPickSearchRule, ScanPickTargetOrderingRule,
    ScanPickYoloRule, SetTimeExecutionPlan, SwitchPartyChooseMenuRule, SwitchPartyConfirmRule,
    SwitchPartyCurrentPartyRule, SwitchPartyExecutionPlan, SwitchPartyListScanRule,
    SwitchPartyStep, SwitchPartyStepAction, SwitchPartyStepCondition, SwitchPartyStepPhase,
    SwitchPartyStepResult, TalkOptionPlanResult, TaskError, TeleportExecutionPlan,
    TeleportPlanKind, TeleportStep, TeleportStepAction, TeleportStepPhase, TeleportStepResult,
    TeleportTargetPlan, WalkToFActionPress, WalkToFExecutionPlan, WalkToFStep, WalkToFStepAction,
    WalkToFStepCondition, WalkToFStepPhase, WalkToFStepResult, WeaponOrePrescrollRule,
    WonderlandCycleExecutionPlan, WonderlandCycleRetryAction, WonderlandCycleRetryRule,
    WonderlandCycleStep, WonderlandCycleStepAction, WonderlandCycleStepCondition,
    WonderlandCycleStepPhase, WonderlandCycleStepResult, CHOOSE_TALK_OPTION_TASK_KEY,
    CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY, CLAIM_MAIL_REWARDS_COLLECT,
    CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD, RELOGIN_CONFIRM, RELOGIN_ENTER_GAME, RELOGIN_MENU_BAG,
    RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS, RETURN_MAIN_UI_EXIT_DOOR, RETURN_MAIN_UI_PAIMON_MENU,
    RETURN_MAIN_UI_TASK_KEY, SET_TIME_PAGE_CLOSE_WHITE, SWITCH_PARTY_TASK_KEY,
    WONDERLAND_CYCLE_BACK_TEYVAT, WONDERLAND_CYCLE_BLACK_CONFIRM, WONDERLAND_CYCLE_CLOSE,
};
use bgi_core::{GenshinAction, KeyBindingsConfig, NotificationPayload};
use bgi_input::{
    input_events_for_action, release_all_keys_sequence, InputCancellationToken, InputEvent,
    KeyActionType,
};
use bgi_vision::{
    BgrImage, BvLocatorOperation, BvLocatorPlan, BvPageCommand, OcrResultRegion,
    PureRustVisionBackend, Rect, Region, Size, VisionBackend,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Input,
    Page,
    Locator,
    Ocr,
    RecognizeOptions,
    MatchText,
    CheckOrange,
    ClickMatchedText,
    ClickMatchedOption,
    Pathing,
    InteractionRetry,
    SelectLastTalkOption,
    DetectResin,
    RecognizeResinCounts,
    ComputeCraftsNeeded,
    CraftCondensedResin,
    OneKeyExpedition,
    MapEntry,
    BagEntry,
    FindAYuan,
    Reward,
    ShopPurchase,
    Finish,
    ReleaseAllKeys,
    ScanPartyList,
    ConfirmParty,
    ClearCombatScenes,
    OpenInventory,
    ConfirmExpiredItemPrompt,
    OpenInventoryTab,
    LoadGridIconClassifier,
    PreScrollWeaponOre,
    EnumerateGridItems,
    CropGridIcon,
    InferGridIcon,
    OcrGridItemCount,
    CameraReset,
    YoloDetect,
    SearchSweep,
    SelectTarget,
    ApproachTarget,
    TeleportAction,
    ReturnResult,
    FocusGameWindow,
    ThirdPartyLoginProbe,
    Notify,
    TrackingLoop,
    ClearVisionDrawings,
    AvatarSwitch,
    MiningDetection,
    MiningAlignment,
    MineAttack,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobExecutorBridgeKind {
    ReturnMainUi,
    SetTime,
    ChooseTalkOption,
    CheckRewards,
    BlessingOfTheWelkinMoon,
    ClaimBattlePassRewards,
    ClaimEncounterPointsRewards,
    ClaimMailRewards,
    CountInventoryItem,
    ScanPickDrops,
    Relogin,
    WonderlandCycle,
    WalkToF,
    LowerHeadThenWalkTo,
    SwitchParty,
    GoToCraftingBench,
    Teleport,
    GoToAdventurersGuild,
    GoToSereniteaPot,
    LinneaMining,
    OneKeyExpedition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonJobExecutorBridgePlan {
    pub task_key: String,
    pub bridge_kind: CommonJobExecutorBridgeKind,
    pub state_machine_ready: bool,
    pub live_io_ready: bool,
    pub supported_actions: Vec<CommonJobRuntimeActionKind>,
    pub notes: String,
}

pub const RETURN_MAIN_UI_LIVE_CAPTURE_SIZE: Size = Size {
    width: 1920,
    height: 1080,
};

pub fn common_job_executor_bridge_plan(
    plan: &CommonJobExecutionPlan,
) -> Option<CommonJobExecutorBridgePlan> {
    match plan {
        CommonJobExecutionPlan::ReturnMainUi(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::ReturnMainUi,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
            ],
            notes: "ReturnMainUi has a Rust condition state machine plus a template/input runtime boundary; callers provide capture and input adapters for live execution.".to_string(),
        }),
        CommonJobExecutionPlan::SetTime(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::SetTime,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
            ],
            notes: "SetTime has a Rust clock-dial state machine plus nested ReturnMainUi execution; callers provide capture, input, and clock adapters for live execution.".to_string(),
        }),
        CommonJobExecutionPlan::ChooseTalkOption(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::ChooseTalkOption,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::RecognizeOptions,
                CommonJobRuntimeActionKind::MatchText,
                CommonJobRuntimeActionKind::CheckOrange,
                CommonJobRuntimeActionKind::ClickMatchedOption,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "ChooseTalkOption has a Rust OCR-option state machine and an injectable options runtime; invocation live execution can now hand it to a caller-provided OCR, color, click, page, and input adapter.".to_string(),
        }),
        CommonJobExecutionPlan::CheckRewards(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::CheckRewards,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Ocr,
                CommonJobRuntimeActionKind::MatchText,
                CommonJobRuntimeActionKind::ClickMatchedText,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Notify,
                CommonJobRuntimeActionKind::Page,
            ],
            notes: "CheckRewards has a Rust handbook OCR/open-retry, claimed-status retry, notification state machine with injectable OCR/click/notify hooks; invocation live execution can now hand it to a caller-provided OCR, click, notify, page, and locator adapter.".to_string(),
        }),
        CommonJobExecutionPlan::BlessingOfTheWelkinMoon(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::BlessingOfTheWelkinMoon,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
            ],
            notes: "BlessingOfTheWelkinMoon has a Rust server-time gate plus template locator, double-click input, and stable-clear loop execution.".to_string(),
        }),
        CommonJobExecutionPlan::ClaimBattlePassRewards(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::ClaimBattlePassRewards,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Ocr,
                CommonJobRuntimeActionKind::MatchText,
                CommonJobRuntimeActionKind::ClickMatchedText,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "ClaimBattlePassRewards has a Rust two-stage claim-all state machine with injectable OCR, text-click, page, locator, and manual-selection dialog hooks; invocation live execution can now hand it to a caller-provided adapter.".to_string(),
        }),
        CommonJobExecutionPlan::ClaimEncounterPointsRewards(plan) => {
            Some(CommonJobExecutorBridgePlan {
                task_key: plan.task_key.clone(),
                bridge_kind: CommonJobExecutorBridgeKind::ClaimEncounterPointsRewards,
                state_machine_ready: true,
                live_io_ready: true,
                supported_actions: vec![
                    CommonJobRuntimeActionKind::Log,
                    CommonJobRuntimeActionKind::CommonJob,
                    CommonJobRuntimeActionKind::GenshinAction,
                    CommonJobRuntimeActionKind::Page,
                    CommonJobRuntimeActionKind::Ocr,
                    CommonJobRuntimeActionKind::MatchText,
                    CommonJobRuntimeActionKind::Locator,
                    CommonJobRuntimeActionKind::ClickMatchedText,
                    CommonJobRuntimeActionKind::ReturnResult,
                ],
                notes: "ClaimEncounterPointsRewards has a Rust OCR retry/branch state machine with injectable text recognition, matched-text click, page, and locator hooks; invocation live execution can now hand it to a caller-provided adapter.".to_string(),
            })
        }
        CommonJobExecutionPlan::ClaimMailRewards(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::ClaimMailRewards,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "ClaimMailRewards has a Rust condition state machine plus nested ReturnMainUi, template locator, and input dispatch execution.".to_string(),
        }),
        CommonJobExecutionPlan::CountInventoryItem(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::CountInventoryItem,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::OpenInventory,
                CommonJobRuntimeActionKind::ConfirmExpiredItemPrompt,
                CommonJobRuntimeActionKind::OpenInventoryTab,
                CommonJobRuntimeActionKind::LoadGridIconClassifier,
                CommonJobRuntimeActionKind::PreScrollWeaponOre,
                CommonJobRuntimeActionKind::EnumerateGridItems,
                CommonJobRuntimeActionKind::CropGridIcon,
                CommonJobRuntimeActionKind::InferGridIcon,
                CommonJobRuntimeActionKind::OcrGridItemCount,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "CountInventoryItem has a Rust state machine with injectable inventory open/tab, grid enumeration, ONNX-classifier, OCR-count, result-contract, and cleanup hooks; invocation live execution can now hand it to a caller-provided vision, classifier, OCR, and input adapter.".to_string(),
        }),
        CommonJobExecutionPlan::ScanPickDrops(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::ScanPickDrops,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CameraReset,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::YoloDetect,
                CommonJobRuntimeActionKind::SearchSweep,
                CommonJobRuntimeActionKind::SelectTarget,
                CommonJobRuntimeActionKind::ApproachTarget,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::ReleaseAllKeys,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "ScanPickDrops has a Rust state machine with injectable YOLO target detection plus Rust search-sweep, target-ordering, movement, camera-reset, and cleanup orchestration; invocation live execution can now hand it to a caller-provided capture/ONNX/input/overlay adapter.".to_string(),
        }),
        CommonJobExecutionPlan::WonderlandCycle(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::WonderlandCycle,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "WonderlandCycle has a Rust retry state machine plus template locator, page click/wait, and input dispatch execution.".to_string(),
        }),
        CommonJobExecutionPlan::Relogin(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::Relogin,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::FocusGameWindow,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::ThirdPartyLoginProbe,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "Relogin has a Rust retry/failure-policy state machine plus a template/input live runtime with game-window focus and Bilibili third-party login platform hooks.".to_string(),
        }),
        CommonJobExecutionPlan::LowerHeadThenWalkTo(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::LowerHeadThenWalkTo,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::TrackingLoop,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "LowerHeadThenWalkTo has a Rust state machine with an injectable camera/movement/F-key tracking loop; invocation live execution can now hand it to a caller-provided mouse, capture, OCR, and overlay adapter.".to_string(),
        }),
        CommonJobExecutionPlan::LinneaMining(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::LinneaMining,
            state_machine_ready: true,
            live_io_ready: false,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::AvatarSwitch,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::MiningDetection,
                CommonJobRuntimeActionKind::MiningAlignment,
                CommonJobRuntimeActionKind::MineAttack,
                CommonJobRuntimeActionKind::YoloDetect,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "LinneaMining has a Rust common-job plan for avatar selection, BgiMine detection, clustering, aiming, mining, compensation, and cleanup; live avatar/capture/ONNX/mouse/overlay adapters remain pending.".to_string(),
        }),
        CommonJobExecutionPlan::OneKeyExpedition(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::OneKeyExpedition,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::OneKeyExpedition,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "OneKeyExpedition has a Rust collect/re-dispatch/exit/cleanup state machine with injectable template, page, input, focus, and overlay hooks.".to_string(),
        }),
        CommonJobExecutionPlan::SwitchParty(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::SwitchParty,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Ocr,
                CommonJobRuntimeActionKind::MatchText,
                CommonJobRuntimeActionKind::ScanPartyList,
                CommonJobRuntimeActionKind::ConfirmParty,
                CommonJobRuntimeActionKind::ClearCombatScenes,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "SwitchParty has a Rust state machine with injectable current-party OCR, list scan, confirm, and combat-scene cleanup hooks; invocation live execution can now hand it to a caller-provided OCR/click adapter.".to_string(),
        }),
        CommonJobExecutionPlan::GoToCraftingBench(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::GoToCraftingBench,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::Pathing,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::InteractionRetry,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::SelectLastTalkOption,
                CommonJobRuntimeActionKind::DetectResin,
                CommonJobRuntimeActionKind::RecognizeResinCounts,
                CommonJobRuntimeActionKind::ComputeCraftsNeeded,
                CommonJobRuntimeActionKind::CraftCondensedResin,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "GoToCraftingBench has a Rust state machine with injectable pathing, interaction retry, crafting-page, resin OCR, and craft-confirm hooks; invocation live execution can now hand it to a caller-provided PathExecutor/OCR/click adapter.".to_string(),
        }),
        CommonJobExecutionPlan::Teleport(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::Teleport,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::TeleportAction,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "Teleport has a Rust state machine with injectable big-map, map-matching, click, teleport-completion, and navigation-seed hooks; invocation live execution can now hand it to a caller-provided map/click adapter and the execution report records the effective navigation seed for later pathing consumption.".to_string(),
        }),
        CommonJobExecutionPlan::GoToAdventurersGuild(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::GoToAdventurersGuild,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::Pathing,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::InteractionRetry,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::SelectLastTalkOption,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::OneKeyExpedition,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "GoToAdventurersGuild has a Rust state machine with injectable party switching, encounter reward, pathing, Catherine interaction, talk-option, and one-key expedition hooks; invocation live execution can now hand it to caller-provided PathExecutor/OCR/click adapters.".to_string(),
        }),
        CommonJobExecutionPlan::GoToSereniteaPot(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::GoToSereniteaPot,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::CommonJob,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::MapEntry,
                CommonJobRuntimeActionKind::BagEntry,
                CommonJobRuntimeActionKind::FindAYuan,
                CommonJobRuntimeActionKind::Reward,
                CommonJobRuntimeActionKind::ShopPurchase,
                CommonJobRuntimeActionKind::Finish,
                CommonJobRuntimeActionKind::ReleaseAllKeys,
                CommonJobRuntimeActionKind::ClearVisionDrawings,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "GoToSereniteaPot has a Rust state machine with injectable map/bag entry, A Yuan search, reward, shop, finish, and cleanup hooks; invocation live execution can now hand it to caller-provided map/OCR/click/shop/path adapters.".to_string(),
        }),
        CommonJobExecutionPlan::WalkToF(plan) => Some(CommonJobExecutorBridgePlan {
            task_key: plan.task_key.clone(),
            bridge_kind: CommonJobExecutorBridgeKind::WalkToF,
            state_machine_ready: true,
            live_io_ready: true,
            supported_actions: vec![
                CommonJobRuntimeActionKind::Log,
                CommonJobRuntimeActionKind::GenshinAction,
                CommonJobRuntimeActionKind::Input,
                CommonJobRuntimeActionKind::Page,
                CommonJobRuntimeActionKind::Locator,
                CommonJobRuntimeActionKind::ReturnResult,
            ],
            notes: "WalkToF has a Rust condition state machine plus a template/input runtime boundary; callers provide capture, input, key-binding, and clock adapters for live execution.".to_string(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobRuntimeOutcome {
    None,
    Matched(bool),
}

impl CommonJobRuntimeOutcome {
    fn as_match(self, step: &CommonJobStep) -> Result<bool> {
        match self {
            CommonJobRuntimeOutcome::Matched(value) => Ok(value),
            CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
                "step {:?}/{:?}/{} did not return a match result",
                step.phase, step.condition, step.label
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobSkipReason {
    MainUiAlreadyDetected,
    MainUiDetectedAfterRetry,
    ExitDoorNotDetected,
    RetryLimitNotReached,
    SkipAnimationNotRequested,
    SkipAnimationRequested,
    SkipAnimationAlreadyResolved,
    ConditionNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonJobRuntimeStepReport {
    pub phase: CommonJobStepPhase,
    pub condition: CommonJobStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub action_kind: CommonJobRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl CommonJobRuntimeStepReport {
    fn executed(step: &CommonJobStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            action_kind: action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonJobSkippedStep {
    pub phase: CommonJobStepPhase,
    pub condition: CommonJobStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub reason: CommonJobSkipReason,
}

impl CommonJobSkippedStep {
    fn new(step: &CommonJobStep, reason: CommonJobSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReturnMainUiExecutorState {
    pub main_ui_detected: bool,
    pub exit_door_detected: bool,
    pub last_escape_attempt: Option<u8>,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReturnMainUiExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ReturnMainUiExecutorState,
    pub executed_steps: Vec<CommonJobRuntimeStepReport>,
    pub skipped_steps: Vec<CommonJobSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimeExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub skip_animation_return_main_ui_completed: Option<bool>,
    pub final_return_main_ui_completed: Option<bool>,
    pub page_close_detected: bool,
    pub skip_animation_resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimeExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: SetTimeExecutorState,
    pub executed_steps: Vec<CommonJobRuntimeStepReport>,
    pub skipped_steps: Vec<CommonJobSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRewardsTextCandidate {
    pub text: String,
    pub rect: Rect,
}

pub fn check_rewards_text_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
) -> Result<Vec<CheckRewardsTextCandidate>> {
    let mut candidates = Vec::new();
    for region in regions {
        let text = region.text.trim();
        if text.is_empty() || region.rect.width <= 0 || region.rect.height <= 0 {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(CheckRewardsTextCandidate {
            text: text.to_string(),
            rect,
        });
    }
    Ok(candidates)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub open_attempts: u8,
    pub handbook_open_dispatched: bool,
    pub recognized_texts: Vec<CheckRewardsTextCandidate>,
    pub matched_commissions_text: Option<CheckRewardsTextCandidate>,
    pub commissions_text_clicked: bool,
    pub daily_reward_title_detected: Option<bool>,
    pub claimed_text_detected: Option<bool>,
    pub notifications_sent: Vec<NotificationPayload>,
    pub final_return_main_ui_completed: Option<bool>,
    pub result: Option<CheckRewardsStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckRewardsRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Ocr,
    MatchText,
    Locator,
    Notify,
    ReturnResult,
    Page,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckRewardsSkipReason {
    ResultAlreadySet,
    CommissionsTextMissing,
    DailyRewardTitleMissing,
    DailyRewardTitleProbeMissing,
    ClaimedTextDetected,
    ClaimedTextMissing,
    ClaimedTextProbeMissing,
    StatusAlreadyChecked,
    StatusCheckMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRewardsRuntimeStepReport {
    pub phase: CheckRewardsStepPhase,
    pub condition: CheckRewardsStepCondition,
    pub label: String,
    pub action_kind: CheckRewardsRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl CheckRewardsRuntimeStepReport {
    fn executed(step: &CheckRewardsStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: check_rewards_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRewardsSkippedStep {
    pub phase: CheckRewardsStepPhase,
    pub condition: CheckRewardsStepCondition,
    pub label: String,
    pub reason: CheckRewardsSkipReason,
}

impl CheckRewardsSkippedStep {
    fn new(step: &CheckRewardsStep, reason: CheckRewardsSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: CheckRewardsExecutorState,
    pub executed_steps: Vec<CheckRewardsRuntimeStepReport>,
    pub skipped_steps: Vec<CheckRewardsSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePassRewardTextCandidate {
    pub text: String,
    pub rect: Rect,
}

pub fn battle_pass_reward_text_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
) -> Result<Vec<BattlePassRewardTextCandidate>> {
    let mut candidates = Vec::new();
    for region in regions {
        let text = region.text.trim();
        if text.is_empty() || region.rect.width <= 0 || region.rect.height <= 0 {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(BattlePassRewardTextCandidate {
            text: text.to_string(),
            rect,
        });
    }
    Ok(candidates)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePassClaimStageState {
    pub recognized_texts: Vec<BattlePassRewardTextCandidate>,
    pub matched_claim_all_text: Option<BattlePassRewardTextCandidate>,
    pub claim_clicked: bool,
    pub manual_selection_dialog_detected: Option<bool>,
    pub primogem_detected: Option<bool>,
    pub primogem_dismissed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBattlePassRewardsExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub battle_pass_open_dispatched: bool,
    pub points_claim: BattlePassClaimStageState,
    pub upgrade_primogem_detected: Option<bool>,
    pub upgrade_primogem_dismissed: bool,
    pub rewards_claim: BattlePassClaimStageState,
    pub final_return_main_ui_completed: Option<bool>,
    pub result: Option<BattlePassRewardStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimBattlePassRewardsRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Page,
    Ocr,
    MatchText,
    ClickMatchedText,
    DetectManualSelectionDialog,
    DismissPrimogemIfVisible,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimBattlePassRewardsSkipReason {
    ResultAlreadySet,
    ClaimAllTextMissing,
    ClaimNotClicked,
    ManualSelectionDialogDetected,
    ScopeMissing,
    ClaimStagesNotReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBattlePassRewardsRuntimeStepReport {
    pub phase: BattlePassRewardStepPhase,
    pub condition: BattlePassRewardStepCondition,
    pub scope: Option<BattlePassClaimScope>,
    pub label: String,
    pub action_kind: ClaimBattlePassRewardsRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ClaimBattlePassRewardsRuntimeStepReport {
    fn executed(step: &BattlePassRewardStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            scope: step.scope,
            label: step.label.clone(),
            action_kind: claim_battle_pass_rewards_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBattlePassRewardsSkippedStep {
    pub phase: BattlePassRewardStepPhase,
    pub condition: BattlePassRewardStepCondition,
    pub scope: Option<BattlePassClaimScope>,
    pub label: String,
    pub reason: ClaimBattlePassRewardsSkipReason,
}

impl ClaimBattlePassRewardsSkippedStep {
    fn new(step: &BattlePassRewardStep, reason: ClaimBattlePassRewardsSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            scope: step.scope,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimBattlePassRewardsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ClaimBattlePassRewardsExecutorState,
    pub executed_steps: Vec<ClaimBattlePassRewardsRuntimeStepReport>,
    pub skipped_steps: Vec<ClaimBattlePassRewardsSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsTextCandidate {
    pub text: String,
    pub rect: Rect,
}

pub fn claim_encounter_points_text_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
) -> Result<Vec<ClaimEncounterPointsRewardsTextCandidate>> {
    let mut candidates = Vec::new();
    for region in regions {
        let text = region.text.trim();
        if text.is_empty() || region.rect.width <= 0 || region.rect.height <= 0 {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(ClaimEncounterPointsRewardsTextCandidate {
            text: text.to_string(),
            rect,
        });
    }
    Ok(candidates)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub handbook_open_dispatched: bool,
    pub ocr_attempts: u8,
    pub recognized_texts: Vec<ClaimEncounterPointsRewardsTextCandidate>,
    pub matched_commissions_text: Option<ClaimEncounterPointsRewardsTextCandidate>,
    pub early_claim_button_detected: Option<bool>,
    pub matched_text_clicked: bool,
    pub claim_button_detected: Option<bool>,
    pub final_return_main_ui_completed: Option<bool>,
    pub open_retry_limit_reached: bool,
    pub result: Option<ClaimEncounterPointsRewardsStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimEncounterPointsRewardsRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Page,
    Ocr,
    MatchText,
    Locator,
    ClickMatchedText,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimEncounterPointsRewardsSkipReason {
    ResultAlreadySet,
    CommissionsTextMissing,
    EarlyClaimButtonDetected,
    EarlyClaimButtonMissing,
    EarlyClaimButtonProbeMissing,
    ClaimButtonMissing,
    ClaimButtonProbeMissing,
    OpenRetryLimitNotReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsRuntimeStepReport {
    pub condition: ClaimEncounterPointsRewardsStepCondition,
    pub label: String,
    pub action_kind: ClaimEncounterPointsRewardsRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ClaimEncounterPointsRewardsRuntimeStepReport {
    fn executed(step: &ClaimEncounterPointsRewardsStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            condition: step.condition,
            label: step.label.clone(),
            action_kind: claim_encounter_points_rewards_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsSkippedStep {
    pub condition: ClaimEncounterPointsRewardsStepCondition,
    pub label: String,
    pub reason: ClaimEncounterPointsRewardsSkipReason,
}

impl ClaimEncounterPointsRewardsSkippedStep {
    fn new(
        step: &ClaimEncounterPointsRewardsStep,
        reason: ClaimEncounterPointsRewardsSkipReason,
    ) -> Self {
        Self {
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ClaimEncounterPointsRewardsExecutorState,
    pub executed_steps: Vec<ClaimEncounterPointsRewardsRuntimeStepReport>,
    pub skipped_steps: Vec<ClaimEncounterPointsRewardsSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMailRewardsExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub paimon_menu_opened: bool,
    pub mail_reward_detected: Option<bool>,
    pub mail_reward_clicked: Option<bool>,
    pub collect_all_detected: Option<bool>,
    pub collect_all_clicked: Option<bool>,
    pub escape_after_claim_dispatched: bool,
    pub final_return_main_ui_completed: Option<bool>,
    pub result: Option<ClaimMailRewardsStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimMailRewardsSkipReason {
    ResultAlreadySet,
    MailRewardDetected,
    MailRewardMissing,
    MailRewardProbeMissing,
    CollectAllDetected,
    CollectAllMissing,
    CollectAllProbeMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMailRewardsRuntimeStepReport {
    pub phase: ClaimMailRewardsStepPhase,
    pub condition: ClaimMailRewardsStepCondition,
    pub label: String,
    pub action_kind: CommonJobRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ClaimMailRewardsRuntimeStepReport {
    fn executed(step: &ClaimMailRewardsStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: claim_mail_rewards_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMailRewardsSkippedStep {
    pub phase: ClaimMailRewardsStepPhase,
    pub condition: ClaimMailRewardsStepCondition,
    pub label: String,
    pub reason: ClaimMailRewardsSkipReason,
}

impl ClaimMailRewardsSkippedStep {
    fn new(step: &ClaimMailRewardsStep, reason: ClaimMailRewardsSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMailRewardsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ClaimMailRewardsExecutorState,
    pub executed_steps: Vec<ClaimMailRewardsRuntimeStepReport>,
    pub skipped_steps: Vec<ClaimMailRewardsSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonExecutorState {
    pub server_time_checked: bool,
    pub server_time_inside_claim_window: bool,
    pub claim_ui_detected: Option<bool>,
    pub girl_moon_detected: Option<bool>,
    pub welkin_moon_detected: Option<bool>,
    pub primogem_detected: Option<bool>,
    pub claim_click_dispatched: bool,
    pub clear_iterations: u8,
    pub stable_clear_count: u8,
    pub cleared: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlessingOfTheWelkinMoonRuntimeActionKind {
    ServerTimeGate,
    DetectClaimUi,
    Input,
    Page,
    LoopUntilClear,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlessingOfTheWelkinMoonSkipReason {
    ServerTimeNotChecked,
    OutsideClaimWindow,
    ClaimUiProbeMissing,
    ClaimUiMissing,
    StableClearAlreadyReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonRuntimeStepReport {
    pub condition: BlessingOfTheWelkinMoonStepCondition,
    pub label: String,
    pub action_kind: BlessingOfTheWelkinMoonRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl BlessingOfTheWelkinMoonRuntimeStepReport {
    fn executed(step: &BlessingOfTheWelkinMoonStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            condition: step.condition,
            label: step.label.clone(),
            action_kind: blessing_of_the_welkin_moon_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonSkippedStep {
    pub condition: BlessingOfTheWelkinMoonStepCondition,
    pub label: String,
    pub reason: BlessingOfTheWelkinMoonSkipReason,
}

impl BlessingOfTheWelkinMoonSkippedStep {
    fn new(step: &BlessingOfTheWelkinMoonStep, reason: BlessingOfTheWelkinMoonSkipReason) -> Self {
        Self {
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlessingOfTheWelkinMoonExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: BlessingOfTheWelkinMoonExecutorState,
    pub executed_steps: Vec<BlessingOfTheWelkinMoonRuntimeStepReport>,
    pub skipped_steps: Vec<BlessingOfTheWelkinMoonSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloginExecutorState {
    pub focus_requested: bool,
    pub menu_open_probe_completed: bool,
    pub menu_opened: bool,
    pub exit_confirm_probe_completed: bool,
    pub exit_confirm_appeared: bool,
    pub exit_confirm_disappeared: bool,
    pub third_party_login_checked: bool,
    pub third_party_login_completed: bool,
    pub login_screen_probe_completed: bool,
    pub login_screen_visible: bool,
    pub enter_game_disappeared: bool,
    pub main_ui_detected: bool,
    pub result: Option<ReloginStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReloginRuntimeActionKind {
    FocusGameWindow,
    RetryUntilAppear,
    RetryUntilDisappear,
    ThirdPartyLoginProbe,
    Page,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReloginSkipReason {
    MenuOpenProbeMissing,
    ExitConfirmProbeMissing,
    LoginScreenMissing,
    EnterGameNotConfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloginRuntimeStepReport {
    pub phase: ReloginStepPhase,
    pub condition: ReloginStepCondition,
    pub label: String,
    pub action_kind: ReloginRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ReloginRuntimeStepReport {
    fn executed(step: &ReloginStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: relogin_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloginSkippedStep {
    pub phase: ReloginStepPhase,
    pub condition: ReloginStepCondition,
    pub label: String,
    pub reason: ReloginSkipReason,
}

impl ReloginSkippedStep {
    fn new(step: &ReloginStep, reason: ReloginSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloginExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ReloginExecutorState,
    pub executed_steps: Vec<ReloginRuntimeStepReport>,
    pub skipped_steps: Vec<ReloginSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderlandCycleExecutorState {
    pub wonderland_menu_detected: bool,
    pub enter_confirm_dialog_detected: bool,
    pub enter_confirm_dialog_disappeared: bool,
    pub in_wonderland_main_ui: bool,
    pub entered_wonderland_reported: bool,
    pub back_teyvat_menu_detected: bool,
    pub return_confirm_dialog_detected: bool,
    pub return_confirm_dialog_disappeared: bool,
    pub returned_to_teyvat_main_ui: bool,
    pub result: Option<WonderlandCycleStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderlandCycleRuntimeActionKind {
    RetryUntilAppear,
    RetryUntilDisappear,
    Page,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderlandCycleSkipReason {
    WonderlandMenuMissing,
    ConfirmDialogMissing,
    WonderlandMainUiMissing,
    BackTeyvatMenuMissing,
    ReturnedMainUiMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderlandCycleRuntimeStepReport {
    pub phase: WonderlandCycleStepPhase,
    pub condition: WonderlandCycleStepCondition,
    pub label: String,
    pub action_kind: WonderlandCycleRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl WonderlandCycleRuntimeStepReport {
    fn executed(step: &WonderlandCycleStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: wonderland_cycle_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderlandCycleSkippedStep {
    pub phase: WonderlandCycleStepPhase,
    pub condition: WonderlandCycleStepCondition,
    pub label: String,
    pub reason: WonderlandCycleSkipReason,
}

impl WonderlandCycleSkippedStep {
    fn new(step: &WonderlandCycleStep, reason: WonderlandCycleSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderlandCycleExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: WonderlandCycleExecutorState,
    pub executed_steps: Vec<WonderlandCycleRuntimeStepReport>,
    pub skipped_steps: Vec<WonderlandCycleSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkToFExecutorState {
    pub pick_detected: bool,
    pub move_forward_held: bool,
    pub sprint_held: bool,
    pub result: Option<WalkToFStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalkToFRuntimeActionKind {
    GenshinAction,
    Input,
    Page,
    Locator,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalkToFSkipReason {
    RunToFDisabled,
    PickDetected,
    PickMissing,
    NeedPressDisabled,
    NeedPressEnabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkToFRuntimeStepReport {
    pub phase: WalkToFStepPhase,
    pub condition: WalkToFStepCondition,
    pub label: String,
    pub action_kind: WalkToFRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl WalkToFRuntimeStepReport {
    fn executed(step: &WalkToFStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: walk_to_f_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkToFSkippedStep {
    pub phase: WalkToFStepPhase,
    pub condition: WalkToFStepCondition,
    pub label: String,
    pub reason: WalkToFSkipReason,
}

impl WalkToFSkippedStep {
    fn new(step: &WalkToFStep, reason: WalkToFSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkToFExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: WalkToFExecutorState,
    pub executed_steps: Vec<WalkToFRuntimeStepReport>,
    pub skipped_steps: Vec<WalkToFSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToExecutorState {
    pub initial_target_detected: Option<bool>,
    pub tracking_loop_completed: bool,
    pub activation_text_detected: bool,
    pub timed_out: bool,
    pub move_forward_held: bool,
    pub vision_drawings_cleared: bool,
    pub result: Option<LowerHeadThenWalkToStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToRuntimeActionKind {
    Locator,
    TrackingLoop,
    GenshinAction,
    ClearVisionDrawings,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerHeadThenWalkToSkipReason {
    InitialTargetMissing,
    InitialTargetDetected,
    InitialTargetProbeMissing,
    ActivationTextMissing,
    TimeoutMissing,
    ResultAlreadySet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToRuntimeStepReport {
    pub phase: LowerHeadThenWalkToStepPhase,
    pub condition: LowerHeadThenWalkToStepCondition,
    pub label: String,
    pub action_kind: LowerHeadThenWalkToRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl LowerHeadThenWalkToRuntimeStepReport {
    fn executed(step: &LowerHeadThenWalkToStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: lower_head_then_walk_to_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToSkippedStep {
    pub phase: LowerHeadThenWalkToStepPhase,
    pub condition: LowerHeadThenWalkToStepCondition,
    pub label: String,
    pub reason: LowerHeadThenWalkToSkipReason,
}

impl LowerHeadThenWalkToSkippedStep {
    fn new(step: &LowerHeadThenWalkToStep, reason: LowerHeadThenWalkToSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerHeadThenWalkToExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: LowerHeadThenWalkToExecutorState,
    pub executed_steps: Vec<LowerHeadThenWalkToRuntimeStepReport>,
    pub skipped_steps: Vec<LowerHeadThenWalkToSkippedStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyTextCandidate {
    pub text: String,
    pub rect: Rect,
}

pub fn switch_party_text_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
) -> Result<Vec<SwitchPartyTextCandidate>> {
    let mut candidates = Vec::new();
    for region in regions {
        let text = region.text.trim();
        if text.is_empty() || region.rect.width <= 0 || region.rect.height <= 0 {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(SwitchPartyTextCandidate {
            text: text.to_string(),
            rect,
        });
    }
    Ok(candidates)
}

pub fn switch_party_text_matches_pattern(
    text: &str,
    pattern: &str,
    match_as_regex: bool,
) -> Result<bool> {
    if match_as_regex {
        let regex = Regex::new(pattern).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "invalid SwitchParty regex pattern {pattern:?}: {error}"
            ))
        })?;
        Ok(regex.is_match(text))
    } else {
        Ok(text.contains(pattern))
    }
}

pub fn switch_party_find_matching_text_candidate(
    candidates: &[SwitchPartyTextCandidate],
    pattern: &str,
    match_as_regex: bool,
) -> Result<Option<SwitchPartyTextCandidate>> {
    for candidate in candidates {
        if switch_party_text_matches_pattern(&candidate.text, pattern, match_as_regex)? {
            return Ok(Some(candidate.clone()));
        }
    }
    Ok(None)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyListScanOutcome {
    pub scanned_pages: u8,
    pub matched_party: Option<SwitchPartyTextCandidate>,
    pub reached_end: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub party_view_open_dispatched: bool,
    pub party_view_opened: Option<bool>,
    pub current_party_raw_text: Option<String>,
    pub current_party_normalized_text: Option<String>,
    pub current_party_matched: Option<bool>,
    pub choose_menu_opened: Option<bool>,
    pub top_reset_dispatched: bool,
    pub party_list_texts: Vec<SwitchPartyTextCandidate>,
    pub list_scan_completed: bool,
    pub scanned_pages: u8,
    pub matched_party: Option<SwitchPartyTextCandidate>,
    pub party_not_found: bool,
    pub party_confirmed: Option<bool>,
    pub combat_scenes_cleared: bool,
    pub final_return_main_ui_completed: Option<bool>,
    pub result: Option<SwitchPartyStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchPartyRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Input,
    Page,
    Locator,
    Ocr,
    NormalizeCurrentPartyName,
    MatchCurrentParty,
    OpenPartyChooseMenu,
    ScanPartyList,
    ConfirmParty,
    ClearCombatScenes,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchPartySkipReason {
    ResultAlreadySet,
    MainUiDetected,
    PartyViewAlreadyOpened,
    PartyViewMissing,
    CurrentPartyMatched,
    CurrentPartyNotMatched,
    CurrentPartyMatchMissing,
    PartyMatchedInList,
    PartyMissingInList,
    PartyListScanMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyRuntimeStepReport {
    pub phase: SwitchPartyStepPhase,
    pub condition: SwitchPartyStepCondition,
    pub label: String,
    pub action_kind: SwitchPartyRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl SwitchPartyRuntimeStepReport {
    fn executed(step: &SwitchPartyStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: switch_party_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartySkippedStep {
    pub phase: SwitchPartyStepPhase,
    pub condition: SwitchPartyStepCondition,
    pub label: String,
    pub reason: SwitchPartySkipReason,
}

impl SwitchPartySkippedStep {
    fn new(step: &SwitchPartyStep, reason: SwitchPartySkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: SwitchPartyExecutorState,
    pub executed_steps: Vec<SwitchPartyRuntimeStepReport>,
    pub skipped_steps: Vec<SwitchPartySkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountInventoryOpenInventoryOutcome {
    pub expired_item_prompt_detected: bool,
    pub inventory_tab_checked: bool,
    pub still_on_main_ui: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryGridItemFrame {
    pub page_index: u32,
    pub item_index: u32,
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryGridIconMatch {
    pub frame: CountInventoryGridItemFrame,
    pub item_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountInventoryItemCount {
    pub item_name: String,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CountInventoryItemExecutionResult {
    Single {
        count: i32,
    },
    Multiple {
        counts: Vec<CountInventoryItemCount>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryItemExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub open_inventory_outcome: Option<CountInventoryOpenInventoryOutcome>,
    pub expired_item_prompt_confirmed: Option<bool>,
    pub inventory_tab_opened: Option<bool>,
    pub retry_open_inventory_outcome: Option<CountInventoryOpenInventoryOutcome>,
    pub classifier_loaded: bool,
    pub weapon_ore_prescrolled: bool,
    pub grid_items: Vec<CountInventoryGridItemFrame>,
    pub grid_icons_cropped: bool,
    pub inferred_icons: Vec<CountInventoryGridIconMatch>,
    pub target_matches: Vec<CountInventoryGridIconMatch>,
    pub item_counts: Vec<CountInventoryItemCount>,
    pub scan_complete: bool,
    pub all_requested_items_found: bool,
    pub result: Option<CountInventoryItemExecutionResult>,
    pub vision_drawings_cleared: bool,
    pub final_return_main_ui_completed: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountInventoryItemRuntimeActionKind {
    CommonJob,
    GenshinAction,
    OpenInventory,
    ConfirmExpiredItemPrompt,
    OpenInventoryTab,
    LoadGridIconClassifier,
    PreScrollWeaponOre,
    EnumerateGridItems,
    CropGridIcon,
    InferGridIcon,
    OcrGridItemCount,
    ReturnResult,
    ClearVisionDrawings,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountInventoryItemSkipReason {
    ResultAlreadySet,
    ExpiredItemPromptMissing,
    InventoryTabAlreadyChecked,
    InventoryTabStateUnknown,
    NotStillOnMainUi,
    OpenInventoryStateUnknown,
    WeaponOreNotRequested,
    ClassifierTargetMissing,
    AllRequestedItemsNotFound,
    ScanIncomplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryItemRuntimeStepReport {
    pub phase: CountInventoryItemStepPhase,
    pub condition: CountInventoryItemStepCondition,
    pub label: String,
    pub action_kind: CountInventoryItemRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl CountInventoryItemRuntimeStepReport {
    fn executed(step: &CountInventoryItemStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: count_inventory_item_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountInventoryItemSkippedStep {
    pub phase: CountInventoryItemStepPhase,
    pub condition: CountInventoryItemStepCondition,
    pub label: String,
    pub reason: CountInventoryItemSkipReason,
}

impl CountInventoryItemSkippedStep {
    fn new(step: &CountInventoryItemStep, reason: CountInventoryItemSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryItemExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: CountInventoryItemExecutorState,
    pub executed_steps: Vec<CountInventoryItemRuntimeStepReport>,
    pub skipped_steps: Vec<CountInventoryItemSkippedStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPickDropsMovementCommand {
    pub action: GenshinAction,
    pub press: ScanPickDropsActionPress,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScanPickDropsExecutorState {
    pub camera_reset_completed: bool,
    pub initial_drop_dispatched: bool,
    pub detection_attempted: bool,
    pub detected_targets: Vec<Rect>,
    pub search_sweep_completed: bool,
    pub search_iterations_run: u8,
    pub selected_target: Option<Rect>,
    pub movement_commands: Vec<ScanPickDropsMovementCommand>,
    pub approach_completed: bool,
    pub release_all_keys_completed: bool,
    pub cleanup_drop_dispatched: bool,
    pub vision_drawings_cleared: bool,
    pub result: Option<ScanPickDropsStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsRuntimeActionKind {
    CameraReset,
    YoloDetect,
    SearchSweep,
    SelectTarget,
    ApproachTarget,
    GenshinAction,
    Page,
    ReleaseAllKeys,
    ClearVisionDrawings,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanPickDropsSkipReason {
    ResultAlreadySet,
    TimeoutReached,
    DetectionMissing,
    ItemsDetected,
    NoItemsDetected,
    TargetMissing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickDropsRuntimeStepReport {
    pub phase: ScanPickDropsStepPhase,
    pub condition: ScanPickDropsStepCondition,
    pub label: String,
    pub action_kind: ScanPickDropsRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ScanPickDropsRuntimeStepReport {
    fn executed(step: &ScanPickDropsStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: scan_pick_drops_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPickDropsSkippedStep {
    pub phase: ScanPickDropsStepPhase,
    pub condition: ScanPickDropsStepCondition,
    pub label: String,
    pub reason: ScanPickDropsSkipReason,
}

impl ScanPickDropsSkippedStep {
    fn new(step: &ScanPickDropsStep, reason: ScanPickDropsSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanPickDropsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ScanPickDropsExecutorState,
    pub executed_steps: Vec<ScanPickDropsRuntimeStepReport>,
    pub skipped_steps: Vec<ScanPickDropsSkippedStep>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchResinCounts {
    pub fragile_resin_count: i32,
    pub condensed_resin_count: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchExecutorState {
    pub pathing_completed: Option<bool>,
    pub talk_ui_detected: Option<bool>,
    pub interaction_retry_succeeded: Option<bool>,
    pub move_backward_held: bool,
    pub crafting_page_opened: Option<bool>,
    pub condensed_resin_visible: Option<bool>,
    pub resin_counts: Option<GoToCraftingBenchResinCounts>,
    pub resin_count_recognition_failed: bool,
    pub crafts_needed: Option<u8>,
    pub crafted: bool,
    pub final_return_main_ui_completed: Option<bool>,
    pub result: Option<GoToCraftingBenchStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchRuntimeActionKind {
    Pathing,
    InteractionRetry,
    GenshinAction,
    SelectLastTalkOptionUntilEnd,
    DetectResin,
    RecognizeResinCounts,
    ComputeCraftsNeeded,
    CraftCondensedResin,
    CommonJob,
    Page,
    Locator,
    Input,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchSkipReason {
    ResultAlreadySet,
    TalkUiDetected,
    TalkUiMissing,
    TalkUiProbeMissing,
    CraftingPageMissing,
    CraftingPageProbeMissing,
    CondensedResinMissing,
    CondensedResinProbeMissing,
    MinResinToKeepDisabled,
    MinResinToKeepEnabled,
    ResinCountRecognitionSucceeded,
    ResinCountRecognitionFailed,
    ResinCountsMissing,
    CraftsNotNeeded,
    CraftedMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchRuntimeStepReport {
    pub phase: GoToCraftingBenchStepPhase,
    pub condition: GoToCraftingBenchStepCondition,
    pub label: String,
    pub action_kind: GoToCraftingBenchRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl GoToCraftingBenchRuntimeStepReport {
    fn executed(step: &GoToCraftingBenchStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: go_to_crafting_bench_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchSkippedStep {
    pub phase: GoToCraftingBenchStepPhase,
    pub condition: GoToCraftingBenchStepCondition,
    pub label: String,
    pub reason: GoToCraftingBenchSkipReason,
}

impl GoToCraftingBenchSkippedStep {
    fn new(step: &GoToCraftingBenchStep, reason: GoToCraftingBenchSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: GoToCraftingBenchExecutorState,
    pub executed_steps: Vec<GoToCraftingBenchRuntimeStepReport>,
    pub skipped_steps: Vec<GoToCraftingBenchSkippedStep>,
    pub nested_return_main_ui_reports: Vec<ReturnMainUiExecutionReport>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneKeyExpeditionExecutorState {
    pub window_activated: bool,
    pub collect_attempts: u8,
    pub collect_detected: Option<bool>,
    pub collect_clicked: bool,
    pub re_dispatch_attempts: u8,
    pub re_dispatch_detected: Option<bool>,
    pub re_dispatch_clicked: bool,
    pub exit_dispatched: bool,
    pub vision_drawings_cleared: bool,
    pub result: Option<OneKeyExpeditionStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneKeyExpeditionRuntimeActionKind {
    ActivateWindow,
    Locator,
    Page,
    Input,
    Log,
    ClearVisionDrawings,
    ReturnResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OneKeyExpeditionSkipReason {
    CollectAlreadyDetected,
    CollectMissing,
    CollectMissingButCanRetry,
    ReDispatchAlreadyDetected,
    ReDispatchMissing,
    ReDispatchMissingButCanRetry,
    ResultAlreadySet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneKeyExpeditionRuntimeStepReport {
    pub phase: OneKeyExpeditionStepPhase,
    pub condition: OneKeyExpeditionStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub action_kind: OneKeyExpeditionRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl OneKeyExpeditionRuntimeStepReport {
    fn executed(step: &crate::OneKeyExpeditionStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            action_kind: one_key_expedition_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneKeyExpeditionSkippedStep {
    pub phase: OneKeyExpeditionStepPhase,
    pub condition: OneKeyExpeditionStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub reason: OneKeyExpeditionSkipReason,
}

impl OneKeyExpeditionSkippedStep {
    fn new(step: &crate::OneKeyExpeditionStep, reason: OneKeyExpeditionSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneKeyExpeditionExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: OneKeyExpeditionExecutorState,
    pub executed_steps: Vec<OneKeyExpeditionRuntimeStepReport>,
    pub skipped_steps: Vec<OneKeyExpeditionSkippedStep>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildExecutorState {
    pub party_switch_completed: Option<bool>,
    pub encounter_points_claimed: Option<bool>,
    pub pathing_completed: Option<bool>,
    pub interaction_retry_succeeded: Option<bool>,
    pub daily_reward_option_result: Option<TalkOptionPlanResult>,
    pub daily_reward_dialogue_finished: Option<bool>,
    pub paimon_menu_detected_after_daily: Option<bool>,
    pub return_main_ui_after_daily_completed: Option<bool>,
    pub catherine_reopened_after_daily: Option<bool>,
    pub expedition_option_result: Option<TalkOptionPlanResult>,
    pub expedition_completed: Option<bool>,
    pub talk_ui_still_open: Option<bool>,
    pub cleanup_dialogue_closed: Option<bool>,
    pub result: Option<GoToAdventurersGuildStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToAdventurersGuildRuntimeActionKind {
    CommonJob,
    Pathing,
    InteractionRetry,
    SelectLastTalkOptionUntilEnd,
    OneKeyExpedition,
    Page,
    Locator,
    Input,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToAdventurersGuildSkipReason {
    ResultAlreadySet,
    DailyRewardPartyMissing,
    OnlyDoOnceEnabled,
    DailyRewardOptionResultMissing,
    DailyRewardOptionMissing,
    DailyRewardOptionNotOrange,
    DailyRewardDialogueMissing,
    ExpeditionOptionResultMissing,
    ExpeditionOptionMissing,
    ExpeditionOptionNotOrange,
    TalkUiClosed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildRuntimeStepReport {
    pub phase: GoToAdventurersGuildStepPhase,
    pub condition: GoToAdventurersGuildStepCondition,
    pub label: String,
    pub action_kind: GoToAdventurersGuildRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl GoToAdventurersGuildRuntimeStepReport {
    fn executed(step: &GoToAdventurersGuildStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: go_to_adventurers_guild_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildSkippedStep {
    pub phase: GoToAdventurersGuildStepPhase,
    pub condition: GoToAdventurersGuildStepCondition,
    pub label: String,
    pub reason: GoToAdventurersGuildSkipReason,
}

impl GoToAdventurersGuildSkippedStep {
    fn new(step: &GoToAdventurersGuildStep, reason: GoToAdventurersGuildSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: GoToAdventurersGuildExecutorState,
    pub executed_steps: Vec<GoToAdventurersGuildRuntimeStepReport>,
    pub skipped_steps: Vec<GoToAdventurersGuildSkippedStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToAdventurersGuildNestedOutcome {
    Completed(bool),
    TalkOption(TalkOptionPlanResult),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotExecutorState {
    pub entry_succeeded: Option<bool>,
    pub entry_realm_name: Option<String>,
    pub ayuan_found: Option<bool>,
    pub rewards_claimed: Option<bool>,
    pub shop_purchase_completed: Option<bool>,
    pub entry_failure_finish_completed: Option<bool>,
    pub ayuan_missing_finish_completed: Option<bool>,
    pub final_finish_completed: Option<bool>,
    pub keys_released: bool,
    pub vision_drawings_cleared: bool,
    pub result: Option<GoToSereniteaPotStepResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotEntryOutcome {
    pub entered: bool,
    pub realm_name: Option<String>,
}

impl GoToSereniteaPotEntryOutcome {
    pub fn entered(realm_name: impl Into<String>) -> Self {
        Self {
            entered: true,
            realm_name: Some(realm_name.into()),
        }
    }

    pub fn failed() -> Self {
        Self {
            entered: false,
            realm_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotRuntimeActionKind {
    CommonJob,
    GenshinAction,
    Locator,
    Page,
    MapEntry,
    BagEntry,
    FindAYuan,
    Reward,
    ShopPurchase,
    Finish,
    ReleaseAllKeys,
    ClearVisionDrawings,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotSkipReason {
    ResultAlreadySet,
    MapTeleportNotConfigured,
    BagGadgetNotConfigured,
    EntrySucceeded,
    EntryFailed,
    EntryResultMissing,
    AYuanFound,
    AYuanMissing,
    AYuanResultMissing,
    ShopNotConfigured,
    ShopPurchased,
    ShopMissingOrNotDue,
    RewardAvailabilityUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotRuntimeStepReport {
    pub phase: GoToSereniteaPotStepPhase,
    pub condition: GoToSereniteaPotStepCondition,
    pub label: String,
    pub action_kind: GoToSereniteaPotRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl GoToSereniteaPotRuntimeStepReport {
    fn executed(step: &GoToSereniteaPotStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: go_to_serenitea_pot_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotSkippedStep {
    pub phase: GoToSereniteaPotStepPhase,
    pub condition: GoToSereniteaPotStepCondition,
    pub label: String,
    pub reason: GoToSereniteaPotSkipReason,
}

impl GoToSereniteaPotSkippedStep {
    fn new(step: &GoToSereniteaPotStep, reason: GoToSereniteaPotSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: GoToSereniteaPotExecutorState,
    pub executed_steps: Vec<GoToSereniteaPotRuntimeStepReport>,
    pub skipped_steps: Vec<GoToSereniteaPotSkippedStep>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TeleportExecutorState {
    pub big_map_open_requested: bool,
    pub big_map_verified: bool,
    pub coordinate_target_resolved: bool,
    pub nearest_teleport_point_resolved: bool,
    pub country_or_map_switched: bool,
    pub underground_map_normalized: bool,
    pub zoom_level_read: bool,
    pub zoom_level_adjusted: bool,
    pub big_map_center_recognized: bool,
    pub big_map_rect_recognized: bool,
    pub big_map_dragged: bool,
    pub target_point_verified: bool,
    pub screen_point_converted: bool,
    pub map_teleport_point_clicked: bool,
    pub teleport_panel_clicked: bool,
    pub point_not_activated_handled: bool,
    pub move_map_completed: bool,
    pub statue_selected: bool,
    pub teleport_completion_waited: bool,
    pub navigation_previous_position_seeded: bool,
    pub navigation_previous_position_seed: Option<TeleportTargetPlan>,
    pub result: Option<TeleportStepResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportRuntimeActionKind {
    OpenBigMapUi,
    VerifyBigMapUi,
    ResolveCoordinateTarget,
    ResolveNearestTeleportPoint,
    SwitchCountryOrMap,
    NormalizeUndergroundMap,
    ReadBigMapZoomLevel,
    AdjustMapZoomLevel,
    RecognizeBigMapCenter,
    RecognizeBigMapRect,
    DragBigMapToTarget,
    VerifyTargetPointInBigMapWindow,
    ConvertMapCoordinateToScreenPoint,
    ClickMapTeleportPoint,
    ClickTeleportPanelOrCandidate,
    MoveMapTo,
    SelectStatueOfTheSeven,
    HandlePointNotActivated,
    WaitForTeleportCompletion,
    SeedNavigationPreviousPositionAfterTeleport,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportRuntimeStepReport {
    pub phase: TeleportStepPhase,
    pub label: String,
    pub action_kind: TeleportRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl TeleportRuntimeStepReport {
    fn executed(step: &TeleportStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            phase: step.phase,
            label: step.label.clone(),
            action_kind: teleport_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: TeleportExecutorState,
    pub executed_steps: Vec<TeleportRuntimeStepReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionCandidate {
    pub text: String,
    pub rect: Rect,
    pub orange_pixel_rate: Option<f64>,
}

pub fn choose_talk_option_ocr_rect_from_lowest_icon(
    lowest_icon_rect: Rect,
    capture_size: Size,
    rule: &ChooseTalkOptionOcrRule,
) -> Result<Rect> {
    let rect = Rect::new(
        lowest_icon_rect.x + lowest_icon_rect.width + rule.ocr_x_padding,
        rule.ocr_y,
        rule.ocr_width,
        lowest_icon_rect.y + lowest_icon_rect.height + rule.ocr_bottom_padding
            - rule.option_icon_roi.y,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let rect = rect
        .clamp_to(capture_size)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(
            "choose-talk-option OCR rect is empty after clamping to the captured image".to_string(),
        ));
    }
    Ok(rect)
}

pub fn choose_talk_option_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
    rule: &ChooseTalkOptionOcrRule,
) -> Result<Vec<ChooseTalkOptionCandidate>> {
    let mut regions = regions.to_vec();
    if rule.sort_ocr_results_by_y_ascending {
        regions.sort_by_key(|region| region.rect.y);
    }

    let mut candidates = Vec::new();
    let short_alphanumeric = Regex::new(r"^[a-zA-Z0-9]+$")
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
    for index in 0..regions.len() {
        let region = &regions[index];
        let text = region.text.trim();
        if text.is_empty() || region.rect.width <= 0 || region.rect.height <= 0 {
            continue;
        }
        if rule.ignore_short_alphanumeric_text
            && text.len() < rule.short_alphanumeric_max_len
            && short_alphanumeric.is_match(text)
        {
            continue;
        }
        if index + 1 < regions.len()
            && regions[index + 1].rect.y - region.rect.y > rule.ignored_large_y_gap
        {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(ChooseTalkOptionCandidate {
            text: text.to_string(),
            rect,
            orange_pixel_rate: None,
        });
    }
    Ok(candidates)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionExecutorState {
    pub talk_ui_detected: bool,
    pub last_attempt: Option<u32>,
    pub option_icons_detected: bool,
    pub first_ocr_stabilized: bool,
    pub recognized_options: Vec<ChooseTalkOptionCandidate>,
    pub matched_option: Option<ChooseTalkOptionCandidate>,
    pub orange_accepted: Option<bool>,
    pub clicked: bool,
    pub result: Option<TalkOptionPlanResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChooseTalkOptionRuntimeActionKind {
    Input,
    Page,
    Locator,
    RecognizeOptions,
    MatchText,
    CheckOrange,
    ClickMatchedOption,
    ReturnResult,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChooseTalkOptionSkipReason {
    ResultAlreadySet,
    TalkUiMissing,
    OptionIconMissing,
    OptionIconPresent,
    FirstOcrAlreadyStabilized,
    FirstOcrNotReady,
    OptionTextMissing,
    OptionTextMatched,
    OrangeNotRequired,
    OrangeAccepted,
    OrangeRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChooseTalkOptionRuntimeStepReport {
    pub condition: ChooseTalkOptionStepCondition,
    pub attempt: Option<u32>,
    pub label: String,
    pub action_kind: ChooseTalkOptionRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

impl ChooseTalkOptionRuntimeStepReport {
    fn executed(step: &ChooseTalkOptionStep, outcome: CommonJobRuntimeOutcome) -> Self {
        Self {
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            action_kind: choose_talk_option_action_kind(&step.action),
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChooseTalkOptionSkippedStep {
    pub condition: ChooseTalkOptionStepCondition,
    pub attempt: Option<u32>,
    pub label: String,
    pub reason: ChooseTalkOptionSkipReason,
}

impl ChooseTalkOptionSkippedStep {
    fn new(step: &ChooseTalkOptionStep, reason: ChooseTalkOptionSkipReason) -> Self {
        Self {
            condition: step.condition,
            attempt: step.attempt,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: ChooseTalkOptionExecutorState,
    pub executed_steps: Vec<ChooseTalkOptionRuntimeStepReport>,
    pub skipped_steps: Vec<ChooseTalkOptionSkippedStep>,
}

pub trait CommonJobRuntime {
    fn log(&mut self, message: &str) -> Result<CommonJobRuntimeOutcome>;
    fn dispatch_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome>;
    fn dispatch_capture_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome>;
    fn execute_page_command(&mut self, command: &BvPageCommand) -> Result<CommonJobRuntimeOutcome>;
    fn execute_locator(&mut self, locator: &BvLocatorPlan) -> Result<CommonJobRuntimeOutcome>;
}

pub trait CommonJobFrameSource {
    fn capture_frame(&mut self) -> Result<BgrImage>;
}

pub trait CountInventoryItemRuntime: CommonJobRuntime {
    fn execute_count_inventory_common_job(
        &mut self,
        task_key: &str,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn open_count_inventory(
        &mut self,
        rule: &CountInventoryOpenInventoryRule,
    ) -> Result<CountInventoryOpenInventoryOutcome>;

    fn confirm_count_inventory_expired_item_prompt(
        &mut self,
        confirm_asset: &str,
        crop_bottom_ratio: f64,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn open_count_inventory_tab(
        &mut self,
        rule: &CountInventoryOpenInventoryRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn load_count_inventory_grid_icon_classifier(
        &mut self,
        rule: &GridIconClassifierRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn pre_scroll_count_inventory_weapon_ore(
        &mut self,
        rule: &WeaponOrePrescrollRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn enumerate_count_inventory_grid_items(
        &mut self,
        template: &GridTemplate,
        detection_rule: &GridItemDetectionRule,
        scroll_rule: &GridScrollRule,
    ) -> Result<Vec<CountInventoryGridItemFrame>>;

    fn crop_count_inventory_grid_icons(
        &mut self,
        items: &[CountInventoryGridItemFrame],
        rule: &GridIconCropRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn infer_count_inventory_grid_icons(
        &mut self,
        items: &[CountInventoryGridItemFrame],
        rule: &GridIconClassifierRule,
    ) -> Result<Vec<CountInventoryGridIconMatch>>;

    fn ocr_count_inventory_item_counts(
        &mut self,
        matches: &[CountInventoryGridIconMatch],
        rule: &GridItemCountOcrRule,
    ) -> Result<Vec<CountInventoryItemCount>>;

    fn clear_count_inventory_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait ScanPickDropsRuntime: CommonJobRuntime {
    fn detect_scan_pick_targets(&mut self, rule: &ScanPickYoloRule) -> Result<Vec<Rect>>;

    fn clear_scan_pick_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait OneKeyExpeditionRuntime: CommonJobRuntime {
    fn activate_one_key_expedition_window(&mut self) -> Result<CommonJobRuntimeOutcome>;

    fn clear_one_key_expedition_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait CommonJobInputDriver {
    fn dispatch_input(&mut self, events: &[InputEvent]) -> Result<()>;
    fn dispatch_capture_input(&mut self, events: &[InputEvent]) -> Result<()> {
        self.dispatch_input(events)
    }
    fn click_capture_point(&mut self, x: i32, y: i32) -> Result<()>;
}

pub trait CommonJobClock {
    fn wait(&mut self, milliseconds: u32) -> Result<()>;
}

pub trait ChooseTalkOptionRuntime: CommonJobRuntime {
    fn recognize_talk_options(
        &mut self,
        rule: &ChooseTalkOptionOcrRule,
    ) -> Result<Vec<ChooseTalkOptionCandidate>>;

    fn is_orange_talk_option(
        &mut self,
        candidate: &ChooseTalkOptionCandidate,
        rule: &ChooseTalkOptionOrangeRule,
    ) -> Result<bool>;

    fn click_talk_option(&mut self, candidate: &ChooseTalkOptionCandidate) -> Result<()>;
}

pub trait ClaimEncounterPointsRewardsRuntime: CommonJobRuntime {
    fn recognize_encounter_points_text(
        &mut self,
        command: &BvPageCommand,
        rule: &ClaimEncounterPointsRewardsOcrRule,
    ) -> Result<Vec<ClaimEncounterPointsRewardsTextCandidate>>;

    fn click_encounter_points_text(
        &mut self,
        candidate: &ClaimEncounterPointsRewardsTextCandidate,
    ) -> Result<()>;
}

pub trait ClaimBattlePassRewardsRuntime: CommonJobRuntime {
    fn recognize_battle_pass_reward_text(
        &mut self,
        command: &BvPageCommand,
        rule: &BattlePassClaimAllRule,
        scope: BattlePassClaimScope,
    ) -> Result<Vec<BattlePassRewardTextCandidate>>;

    fn click_battle_pass_reward_text(
        &mut self,
        candidate: &BattlePassRewardTextCandidate,
        scope: BattlePassClaimScope,
    ) -> Result<()>;
}

pub trait CheckRewardsRuntime: CommonJobRuntime {
    fn recognize_check_rewards_text(
        &mut self,
        command: &BvPageCommand,
    ) -> Result<Vec<CheckRewardsTextCandidate>>;

    fn click_check_rewards_text(&mut self, candidate: &CheckRewardsTextCandidate) -> Result<()>;

    fn notify_check_rewards(
        &mut self,
        payload: &NotificationPayload,
    ) -> Result<CommonJobRuntimeOutcome>;
}

pub trait LowerHeadThenWalkToRuntime: CommonJobRuntime {
    fn execute_lower_head_tracking_loop(
        &mut self,
        target_locator: &BvLocatorPlan,
        movement_rule: &LowerHeadThenWalkToMovementRule,
        f_key_rule: &LowerHeadThenWalkToFKeyRule,
    ) -> Result<LowerHeadThenWalkToStepResult>;

    fn clear_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait SwitchPartyRuntime: CommonJobRuntime {
    fn recognize_switch_party_text(
        &mut self,
        command: &BvPageCommand,
    ) -> Result<Vec<SwitchPartyTextCandidate>>;

    fn open_switch_party_choose_menu(
        &mut self,
        rule: &SwitchPartyChooseMenuRule,
        choose_locator: &BvLocatorPlan,
        delete_locator: &BvLocatorPlan,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn scan_switch_party_list(
        &mut self,
        rule: &SwitchPartyListScanRule,
        party_name: &str,
        current_page_texts: &[SwitchPartyTextCandidate],
    ) -> Result<SwitchPartyListScanOutcome>;

    fn confirm_switch_party(
        &mut self,
        rule: &SwitchPartyConfirmRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn clear_switch_party_combat_scenes(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait GoToCraftingBenchRuntime: CommonJobRuntime {
    fn execute_crafting_bench_pathing(
        &mut self,
        rule: &GoToCraftingBenchPathingRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn retry_crafting_bench_interaction(
        &mut self,
        rule: &GoToCraftingBenchInteractionRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn select_last_crafting_bench_talk_option_until_end(
        &mut self,
        until_locator: &BvLocatorPlan,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn recognize_crafting_bench_resin_counts(
        &mut self,
        rule: &GoToCraftingBenchResinRecognitionRule,
    ) -> Result<Option<GoToCraftingBenchResinCounts>>;

    fn craft_condensed_resin(
        &mut self,
        rule: &GoToCraftingBenchResinCraftRule,
        crafts_needed: u8,
    ) -> Result<CommonJobRuntimeOutcome>;
}

pub trait GoToAdventurersGuildRuntime: CommonJobRuntime {
    fn execute_adventurers_guild_common_job(
        &mut self,
        task_key: &str,
        config: Option<&Value>,
    ) -> Result<GoToAdventurersGuildNestedOutcome>;

    fn execute_adventurers_guild_pathing(
        &mut self,
        rule: &GoToAdventurersGuildPathingRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn retry_adventurers_guild_interaction(
        &mut self,
        rule: &GoToAdventurersGuildInteractionRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn select_last_adventurers_guild_talk_option_until_end(
        &mut self,
        max_times: Option<u8>,
        until_paimon_menu: bool,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn run_one_key_adventurers_guild_expedition(
        &mut self,
        plan: &OneKeyExpeditionExecutionPlan,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn is_adventurers_guild_talk_ui_open(&mut self) -> Result<bool>;
}

pub trait GoToSereniteaPotRuntime: CommonJobRuntime {
    fn enter_serenitea_pot_by_map(
        &mut self,
        rule: &GoToSereniteaPotMapEntryRule,
    ) -> Result<GoToSereniteaPotEntryOutcome>;

    fn enter_serenitea_pot_by_bag(
        &mut self,
        rule: &GoToSereniteaPotBagEntryRule,
    ) -> Result<GoToSereniteaPotEntryOutcome>;

    fn find_and_approach_serenitea_pot_ayuan(
        &mut self,
        rule: &GoToSereniteaPotFindAYuanRule,
        realm_name: Option<&str>,
    ) -> Result<bool>;

    fn claim_serenitea_pot_rewards(
        &mut self,
        rule: &GoToSereniteaPotRewardRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn purchase_serenitea_pot_shop(
        &mut self,
        rule: &GoToSereniteaPotShopRule,
        configured_objects: &[String],
    ) -> Result<CommonJobRuntimeOutcome>;

    fn finish_serenitea_pot(
        &mut self,
        rule: &GoToSereniteaPotFinishRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn release_serenitea_pot_keys(&mut self) -> Result<CommonJobRuntimeOutcome>;

    fn clear_serenitea_pot_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;
}

pub trait TeleportRuntime: CommonJobRuntime {
    fn execute_teleport_action(
        &mut self,
        action: &TeleportStepAction,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn teleport_navigation_seed_target(&self) -> Option<TeleportTargetPlan> {
        None
    }
}

pub trait ReloginRuntime: CommonJobRuntime {
    fn focus_game_window(&mut self) -> Result<CommonJobRuntimeOutcome>;
    fn execute_third_party_login_probe(
        &mut self,
        rule: &ReloginThirdPartyRule,
    ) -> Result<CommonJobRuntimeOutcome>;
}

pub trait ReloginPlatformDriver {
    fn focus_game_window(&mut self) -> Result<()>;
    fn execute_third_party_login_probe(
        &mut self,
        rule: &ReloginThirdPartyRule,
    ) -> Result<CommonJobRuntimeOutcome>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StdCommonJobClock;

impl CommonJobClock for StdCommonJobClock {
    fn wait(&mut self, milliseconds: u32) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds as u64));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CancellableCommonJobClock {
    cancellation: Arc<InputCancellationToken>,
    poll_interval_milliseconds: u32,
}

impl CancellableCommonJobClock {
    pub fn new(cancellation: Arc<InputCancellationToken>) -> Self {
        Self {
            cancellation,
            poll_interval_milliseconds: 25,
        }
    }

    pub fn with_poll_interval(mut self, milliseconds: u32) -> Self {
        self.poll_interval_milliseconds = milliseconds.max(1);
        self
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "common job execution cancelled".to_string(),
            ));
        }
        Ok(())
    }
}

impl CommonJobClock for CancellableCommonJobClock {
    fn wait(&mut self, milliseconds: u32) -> Result<()> {
        self.check_cancelled()?;
        let mut remaining = milliseconds;
        while remaining > 0 {
            let chunk = remaining.min(self.poll_interval_milliseconds);
            std::thread::sleep(std::time::Duration::from_millis(chunk as u64));
            remaining -= chunk;
            self.check_cancelled()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TemplateCommonJobRuntime<B, F, I, C> {
    vision_backend: B,
    frame_source: F,
    input_driver: I,
    clock: C,
    logs: Vec<String>,
}

pub type PureTemplateCommonJobRuntime<F, I, C> =
    TemplateCommonJobRuntime<PureRustVisionBackend, F, I, C>;

impl<B, F, I, C> TemplateCommonJobRuntime<B, F, I, C> {
    pub fn new(vision_backend: B, frame_source: F, input_driver: I, clock: C) -> Self {
        Self {
            vision_backend,
            frame_source,
            input_driver,
            clock,
            logs: Vec::new(),
        }
    }

    pub fn vision_backend(&self) -> &B {
        &self.vision_backend
    }

    pub fn frame_source(&self) -> &F {
        &self.frame_source
    }

    pub fn input_driver(&self) -> &I {
        &self.input_driver
    }

    pub fn frame_source_mut(&mut self) -> &mut F {
        &mut self.frame_source
    }

    pub fn input_driver_mut(&mut self) -> &mut I {
        &mut self.input_driver
    }

    pub fn clock(&self) -> &C {
        &self.clock
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn into_parts(self) -> (B, F, I, C, Vec<String>) {
        (
            self.vision_backend,
            self.frame_source,
            self.input_driver,
            self.clock,
            self.logs,
        )
    }
}

impl<F, I, C> PureTemplateCommonJobRuntime<F, I, C> {
    pub fn with_pure_vision(frame_source: F, input_driver: I, clock: C) -> Self {
        Self::new(
            PureRustVisionBackend::new(),
            frame_source,
            input_driver,
            clock,
        )
    }

    pub fn with_task_assets(frame_source: F, input_driver: I, clock: C) -> Self {
        Self::new(
            PureRustVisionBackend::new().with_template_root(crate::task_asset_root()),
            frame_source,
            input_driver,
            clock,
        )
    }
}

pub fn execute_return_main_ui_live<F, I, C>(
    capture_size: Size,
    max_escape_attempts: u8,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<ReturnMainUiExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    if capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "ReturnMainUi live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            capture_size.width,
            capture_size.height
        )));
    }
    let plan = crate::plan_return_main_ui(capture_size, max_escape_attempts)?;
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_return_main_ui_plan(&plan, &mut runtime)
}

pub fn execute_set_time_live<F, I, C>(
    plan: &SetTimeExecutionPlan,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<SetTimeExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "SetTime live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_set_time_plan(plan, &mut runtime)
}

impl<B, F, I, C> CommonJobRuntime for TemplateCommonJobRuntime<B, F, I, C>
where
    B: VisionBackend,
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> Result<CommonJobRuntimeOutcome> {
        self.logs.push(message.to_string());
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn dispatch_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome> {
        self.input_driver.dispatch_input(events)?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn dispatch_capture_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome> {
        self.input_driver.dispatch_capture_input(events)?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn execute_page_command(&mut self, command: &BvPageCommand) -> Result<CommonJobRuntimeOutcome> {
        match command {
            BvPageCommand::Screenshot { .. } => {
                self.frame_source.capture_frame()?;
                Ok(CommonJobRuntimeOutcome::None)
            }
            BvPageCommand::Wait { milliseconds } => {
                self.clock.wait(*milliseconds)?;
                Ok(CommonJobRuntimeOutcome::None)
            }
            BvPageCommand::Click1080p {
                screen_x, screen_y, ..
            } => {
                self.input_driver
                    .click_capture_point(screen_x.round() as i32, screen_y.round() as i32)?;
                Ok(CommonJobRuntimeOutcome::None)
            }
            BvPageCommand::Ocr { .. } => Err(TaskError::CommonJobExecution(
                "OCR page command is not supported by the template common-job runtime".to_string(),
            )),
        }
    }

    fn execute_locator(&mut self, locator: &BvLocatorPlan) -> Result<CommonJobRuntimeOutcome> {
        match locator.operation {
            BvLocatorOperation::FindAll
            | BvLocatorOperation::IsExist
            | BvLocatorOperation::WaitFor => Ok(CommonJobRuntimeOutcome::Matched(
                self.wait_for_locator(locator)?.is_some(),
            )),
            BvLocatorOperation::WaitForDisappear => Ok(CommonJobRuntimeOutcome::Matched(
                self.wait_for_disappear(locator)?,
            )),
            BvLocatorOperation::Click | BvLocatorOperation::DoubleClick => {
                let Some(region) = self.wait_for_locator(locator)? else {
                    return Ok(CommonJobRuntimeOutcome::Matched(false));
                };
                let center = region.rect.center();
                self.input_driver.click_capture_point(center.x, center.y)?;
                if locator.operation == BvLocatorOperation::DoubleClick {
                    self.input_driver.click_capture_point(center.x, center.y)?;
                }
                Ok(CommonJobRuntimeOutcome::Matched(true))
            }
            BvLocatorOperation::ClickUntilDisappears => {
                let mut disappeared = false;
                for _ in 0..locator.retry_count.max(1) {
                    let Some(region) = self.locate_once(locator)? else {
                        disappeared = true;
                        break;
                    };
                    let center = region.rect.center();
                    self.input_driver.click_capture_point(center.x, center.y)?;
                    self.clock.wait(locator.retry_interval_ms)?;
                }
                Ok(CommonJobRuntimeOutcome::Matched(disappeared))
            }
        }
    }
}

impl<B, F, I, C> ReloginRuntime for TemplateCommonJobRuntime<B, F, I, C>
where
    B: VisionBackend,
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    fn focus_game_window(&mut self) -> Result<CommonJobRuntimeOutcome> {
        ReloginPlatformDriver::focus_game_window(&mut self.input_driver)?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn execute_third_party_login_probe(
        &mut self,
        rule: &ReloginThirdPartyRule,
    ) -> Result<CommonJobRuntimeOutcome> {
        ReloginPlatformDriver::execute_third_party_login_probe(&mut self.input_driver, rule)
    }
}

impl<B, F, I, C> OneKeyExpeditionRuntime for TemplateCommonJobRuntime<B, F, I, C>
where
    B: VisionBackend,
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    fn activate_one_key_expedition_window(&mut self) -> Result<CommonJobRuntimeOutcome> {
        ReloginPlatformDriver::focus_game_window(&mut self.input_driver)?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn clear_one_key_expedition_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::None)
    }
}

impl<B, F, I, C> TemplateCommonJobRuntime<B, F, I, C>
where
    B: VisionBackend,
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn wait_for_locator(&mut self, locator: &BvLocatorPlan) -> Result<Option<Region>> {
        for index in 0..locator.retry_count.max(1) {
            let region = self.locate_once(locator)?;
            if region.is_some() {
                return Ok(region);
            }
            if index + 1 < locator.retry_count.max(1) {
                self.clock.wait(locator.retry_interval_ms)?;
            }
        }
        Ok(None)
    }

    fn wait_for_disappear(&mut self, locator: &BvLocatorPlan) -> Result<bool> {
        for index in 0..locator.retry_count.max(1) {
            if self.locate_once(locator)?.is_none() {
                return Ok(true);
            }
            if index + 1 < locator.retry_count.max(1) {
                self.clock.wait(locator.retry_interval_ms)?;
            }
        }
        Ok(false)
    }

    fn locate_once(&mut self, locator: &BvLocatorPlan) -> Result<Option<Region>> {
        let frame = self.frame_source.capture_frame()?;
        let region = self
            .vision_backend
            .find(&frame.pixels, frame.size, &locator.recognition_object)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        Ok(region.is_exist().then_some(region))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordingCommonJobRuntime {
    pub locator_outcomes: VecDeque<CommonJobRuntimeOutcome>,
    pub third_party_login_outcomes: VecDeque<CommonJobRuntimeOutcome>,
    pub lower_head_tracking_results: VecDeque<LowerHeadThenWalkToStepResult>,
    pub input_batches: Vec<Vec<InputEvent>>,
    pub page_commands: Vec<BvPageCommand>,
    pub locator_calls: Vec<BvLocatorPlan>,
    pub lower_head_tracking_calls: Vec<(
        BvLocatorPlan,
        LowerHeadThenWalkToMovementRule,
        LowerHeadThenWalkToFKeyRule,
    )>,
    pub clear_vision_drawings_calls: usize,
    pub focus_calls: usize,
    pub third_party_login_calls: usize,
    pub logs: Vec<String>,
}

impl RecordingCommonJobRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_locator_outcomes(
        outcomes: impl IntoIterator<Item = CommonJobRuntimeOutcome>,
    ) -> Self {
        Self {
            locator_outcomes: outcomes.into_iter().collect(),
            ..Self::default()
        }
    }

    pub fn with_locator_matches(matches: impl IntoIterator<Item = bool>) -> Self {
        Self::with_locator_outcomes(matches.into_iter().map(CommonJobRuntimeOutcome::Matched))
    }

    pub fn with_locator_and_third_party_outcomes(
        locator_outcomes: impl IntoIterator<Item = CommonJobRuntimeOutcome>,
        third_party_login_outcomes: impl IntoIterator<Item = CommonJobRuntimeOutcome>,
    ) -> Self {
        Self {
            locator_outcomes: locator_outcomes.into_iter().collect(),
            third_party_login_outcomes: third_party_login_outcomes.into_iter().collect(),
            ..Self::default()
        }
    }

    pub fn with_lower_head_tracking_results(
        locator_outcomes: impl IntoIterator<Item = CommonJobRuntimeOutcome>,
        tracking_results: impl IntoIterator<Item = LowerHeadThenWalkToStepResult>,
    ) -> Self {
        Self {
            locator_outcomes: locator_outcomes.into_iter().collect(),
            lower_head_tracking_results: tracking_results.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl CommonJobRuntime for RecordingCommonJobRuntime {
    fn log(&mut self, message: &str) -> Result<CommonJobRuntimeOutcome> {
        self.logs.push(message.to_string());
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn dispatch_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome> {
        self.input_batches.push(events.to_vec());
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn dispatch_capture_input(&mut self, events: &[InputEvent]) -> Result<CommonJobRuntimeOutcome> {
        self.input_batches.push(events.to_vec());
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn execute_page_command(&mut self, command: &BvPageCommand) -> Result<CommonJobRuntimeOutcome> {
        self.page_commands.push(command.clone());
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn execute_locator(&mut self, locator: &BvLocatorPlan) -> Result<CommonJobRuntimeOutcome> {
        self.locator_calls.push(locator.clone());
        if matches!(
            locator.operation,
            BvLocatorOperation::IsExist | BvLocatorOperation::WaitFor
        ) {
            Ok(self
                .locator_outcomes
                .pop_front()
                .unwrap_or(CommonJobRuntimeOutcome::Matched(false)))
        } else if let Some(outcome) = self.locator_outcomes.pop_front() {
            Ok(outcome)
        } else {
            Ok(CommonJobRuntimeOutcome::None)
        }
    }
}

impl ReloginRuntime for RecordingCommonJobRuntime {
    fn focus_game_window(&mut self) -> Result<CommonJobRuntimeOutcome> {
        self.focus_calls += 1;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn execute_third_party_login_probe(
        &mut self,
        _rule: &ReloginThirdPartyRule,
    ) -> Result<CommonJobRuntimeOutcome> {
        self.third_party_login_calls += 1;
        Ok(self
            .third_party_login_outcomes
            .pop_front()
            .unwrap_or(CommonJobRuntimeOutcome::Matched(true)))
    }
}

impl LowerHeadThenWalkToRuntime for RecordingCommonJobRuntime {
    fn execute_lower_head_tracking_loop(
        &mut self,
        target_locator: &BvLocatorPlan,
        movement_rule: &LowerHeadThenWalkToMovementRule,
        f_key_rule: &LowerHeadThenWalkToFKeyRule,
    ) -> Result<LowerHeadThenWalkToStepResult> {
        self.lower_head_tracking_calls.push((
            target_locator.clone(),
            *movement_rule,
            f_key_rule.clone(),
        ));
        Ok(self
            .lower_head_tracking_results
            .pop_front()
            .unwrap_or(LowerHeadThenWalkToStepResult::Timeout))
    }

    fn clear_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome> {
        self.clear_vision_drawings_calls += 1;
        Ok(CommonJobRuntimeOutcome::None)
    }
}

impl OneKeyExpeditionRuntime for RecordingCommonJobRuntime {
    fn activate_one_key_expedition_window(&mut self) -> Result<CommonJobRuntimeOutcome> {
        self.focus_calls += 1;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn clear_one_key_expedition_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome> {
        self.clear_vision_drawings_calls += 1;
        Ok(CommonJobRuntimeOutcome::None)
    }
}

pub fn execute_return_main_ui_plan<R>(
    plan: &ReturnMainUiExecutionPlan,
    runtime: &mut R,
) -> Result<ReturnMainUiExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = ReturnMainUiExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_return_main_ui_step(step, &state) {
            Ok(()) => {
                let outcome = execute_common_job_step(step, runtime)?;
                apply_return_main_ui_outcome(step, outcome, &mut state)?;
                executed_steps.push(CommonJobRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(CommonJobSkippedStep::new(step, reason)),
        }
    }

    Ok(ReturnMainUiExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.main_ui_detected,
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_set_time_plan<R>(
    plan: &SetTimeExecutionPlan,
    runtime: &mut R,
) -> Result<SetTimeExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = SetTimeExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_set_time_step(step, plan, &state) {
            Ok(()) => {
                let (outcome, nested_report) = execute_set_time_step(step, plan, runtime)?;
                apply_set_time_outcome(step, outcome, &mut state)?;
                executed_steps.push(CommonJobRuntimeStepReport::executed(step, outcome));
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
            }
            Err(reason) => skipped_steps.push(CommonJobSkippedStep::new(step, reason)),
        }
    }

    let completed = if plan.skip_time_adjustment_animation && state.skip_animation_resolved {
        true
    } else {
        state.final_return_main_ui_completed.unwrap_or(false)
    };

    Ok(SetTimeExecutionReport {
        task_key: plan.task_key.clone(),
        completed,
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_choose_talk_option_plan<R>(
    plan: &ChooseTalkOptionExecutionPlan,
    runtime: &mut R,
) -> Result<ChooseTalkOptionExecutionReport>
where
    R: ChooseTalkOptionRuntime,
{
    let mut state = ChooseTalkOptionExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_choose_talk_option_step(step, plan, &state) {
            Ok(()) => {
                let outcome = execute_choose_talk_option_step(step, plan, runtime, &mut state)?;
                apply_choose_talk_option_outcome(step, outcome, &mut state)?;
                executed_steps.push(ChooseTalkOptionRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(ChooseTalkOptionSkippedStep::new(step, reason)),
        }
    }

    Ok(ChooseTalkOptionExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_check_rewards_plan<R>(
    plan: &CheckRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<CheckRewardsExecutionReport>
where
    R: CheckRewardsRuntime,
{
    let mut state = CheckRewardsExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_check_rewards_step(step, &state) {
            Ok(()) => {
                let (outcome, nested_report) =
                    execute_check_rewards_step(step, plan, key_bindings, runtime, &mut state)?;
                apply_check_rewards_outcome(step, outcome, &mut state)?;
                executed_steps.push(CheckRewardsRuntimeStepReport::executed(step, outcome));
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
            }
            Err(reason) => skipped_steps.push(CheckRewardsSkippedStep::new(step, reason)),
        }
    }

    Ok(CheckRewardsExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_claim_battle_pass_rewards_plan<R>(
    plan: &ClaimBattlePassRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<ClaimBattlePassRewardsExecutionReport>
where
    R: ClaimBattlePassRewardsRuntime,
{
    let mut state = ClaimBattlePassRewardsExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_claim_battle_pass_rewards_step(step, &state) {
            Ok(()) => {
                let (outcome, nested_report) = execute_claim_battle_pass_rewards_step(
                    step,
                    plan,
                    key_bindings,
                    runtime,
                    &mut state,
                )?;
                apply_claim_battle_pass_rewards_outcome(step, outcome, &mut state)?;
                executed_steps.push(ClaimBattlePassRewardsRuntimeStepReport::executed(
                    step, outcome,
                ));
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
            }
            Err(reason) => skipped_steps.push(ClaimBattlePassRewardsSkippedStep::new(step, reason)),
        }
    }

    Ok(ClaimBattlePassRewardsExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(BattlePassRewardStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_claim_encounter_points_rewards_plan<R>(
    plan: &ClaimEncounterPointsRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<ClaimEncounterPointsRewardsExecutionReport>
where
    R: ClaimEncounterPointsRewardsRuntime,
{
    let mut state = ClaimEncounterPointsRewardsExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_claim_encounter_points_rewards_step(step, &state) {
            Ok(()) => {
                let (outcome, nested_report) = execute_claim_encounter_points_rewards_step(
                    step,
                    plan,
                    key_bindings,
                    runtime,
                    &mut state,
                )?;
                apply_claim_encounter_points_rewards_outcome(step, outcome, &mut state)?;
                executed_steps.push(ClaimEncounterPointsRewardsRuntimeStepReport::executed(
                    step, outcome,
                ));
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
            }
            Err(reason) => {
                skipped_steps.push(ClaimEncounterPointsRewardsSkippedStep::new(step, reason))
            }
        }
    }

    Ok(ClaimEncounterPointsRewardsExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_blessing_of_the_welkin_moon_live<F, I, C>(
    plan: &BlessingOfTheWelkinMoonExecutionPlan,
    server_time_zone_offset_minutes: i32,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<BlessingOfTheWelkinMoonExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    let server_minutes = current_server_minutes_since_midnight(server_time_zone_offset_minutes)?;
    execute_blessing_of_the_welkin_moon_live_at_server_minutes(
        plan,
        server_minutes,
        frame_source,
        input_driver,
        clock,
    )
}

pub fn execute_blessing_of_the_welkin_moon_live_at_server_minutes<F, I, C>(
    plan: &BlessingOfTheWelkinMoonExecutionPlan,
    server_minutes_since_midnight: u16,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<BlessingOfTheWelkinMoonExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "BlessingOfTheWelkinMoon live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_blessing_of_the_welkin_moon_plan_at_server_minutes(
        plan,
        &mut runtime,
        server_minutes_since_midnight,
    )
}

pub fn execute_blessing_of_the_welkin_moon_plan<R>(
    plan: &BlessingOfTheWelkinMoonExecutionPlan,
    runtime: &mut R,
) -> Result<BlessingOfTheWelkinMoonExecutionReport>
where
    R: CommonJobRuntime,
{
    let server_minutes = current_server_minutes_since_midnight(8 * 60)?;
    execute_blessing_of_the_welkin_moon_plan_at_server_minutes(plan, runtime, server_minutes)
}

pub fn execute_blessing_of_the_welkin_moon_plan_at_server_minutes<R>(
    plan: &BlessingOfTheWelkinMoonExecutionPlan,
    runtime: &mut R,
    server_minutes_since_midnight: u16,
) -> Result<BlessingOfTheWelkinMoonExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = BlessingOfTheWelkinMoonExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_blessing_of_the_welkin_moon_step(step, &state) {
            Ok(()) => {
                let outcome = execute_blessing_of_the_welkin_moon_step(
                    step,
                    runtime,
                    server_minutes_since_midnight,
                    &mut state,
                )?;
                executed_steps.push(BlessingOfTheWelkinMoonRuntimeStepReport::executed(
                    step, outcome,
                ));
            }
            Err(reason) => {
                skipped_steps.push(BlessingOfTheWelkinMoonSkippedStep::new(step, reason))
            }
        }
    }

    Ok(BlessingOfTheWelkinMoonExecutionReport {
        task_key: plan.task_key.clone(),
        completed: blessing_of_the_welkin_moon_completed(&state),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_claim_mail_rewards_live<F, I, C>(
    plan: &ClaimMailRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<ClaimMailRewardsExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "ClaimMailRewards live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_claim_mail_rewards_plan(plan, key_bindings, &mut runtime)
}

pub fn execute_relogin_live<F, I, C>(
    plan: &ReloginExecutionPlan,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<ReloginExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "Relogin live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_relogin_plan(plan, &mut runtime)
}

pub fn execute_relogin_plan<R>(
    plan: &ReloginExecutionPlan,
    runtime: &mut R,
) -> Result<ReloginExecutionReport>
where
    R: ReloginRuntime,
{
    let mut state = ReloginExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_relogin_step(step, &state) {
            Ok(()) => {
                let outcome = execute_relogin_step(step, runtime)?;
                apply_relogin_outcome(step, outcome, &mut state)?;
                executed_steps.push(ReloginRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(ReloginSkippedStep::new(step, reason)),
        }
    }

    Ok(ReloginExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(ReloginStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_claim_mail_rewards_plan<R>(
    plan: &ClaimMailRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<ClaimMailRewardsExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = ClaimMailRewardsExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_claim_mail_rewards_step(step, &state) {
            Ok(()) => {
                let (outcome, nested_report) =
                    execute_claim_mail_rewards_step(step, plan, key_bindings, runtime)?;
                apply_claim_mail_rewards_outcome(step, outcome, &mut state)?;
                executed_steps.push(ClaimMailRewardsRuntimeStepReport::executed(step, outcome));
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
            }
            Err(reason) => skipped_steps.push(ClaimMailRewardsSkippedStep::new(step, reason)),
        }
    }

    Ok(ClaimMailRewardsExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some() && state.final_return_main_ui_completed.unwrap_or(false),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_wonderland_cycle_live<F, I, C>(
    plan: &WonderlandCycleExecutionPlan,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<WonderlandCycleExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "WonderlandCycle live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_wonderland_cycle_plan(plan, &mut runtime)
}

pub fn execute_wonderland_cycle_plan<R>(
    plan: &WonderlandCycleExecutionPlan,
    runtime: &mut R,
) -> Result<WonderlandCycleExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = WonderlandCycleExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_wonderland_cycle_step(step, &state) {
            Ok(()) => {
                let outcome = execute_wonderland_cycle_step(step, runtime)?;
                apply_wonderland_cycle_outcome(step, outcome, &mut state)?;
                executed_steps.push(WonderlandCycleRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(WonderlandCycleSkippedStep::new(step, reason)),
        }
    }

    Ok(WonderlandCycleExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(WonderlandCycleStepResult::ReturnedToTeyvat),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_lower_head_then_walk_to_plan<R>(
    plan: &LowerHeadThenWalkToExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<LowerHeadThenWalkToExecutionReport>
where
    R: LowerHeadThenWalkToRuntime,
{
    let mut state = LowerHeadThenWalkToExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_lower_head_then_walk_to_step(step, &state) {
            Ok(()) => {
                let outcome =
                    execute_lower_head_then_walk_to_step(step, key_bindings, runtime, &mut state)?;
                apply_lower_head_then_walk_to_outcome(step, outcome, &mut state)?;
                executed_steps.push(LowerHeadThenWalkToRuntimeStepReport::executed(
                    step, outcome,
                ));
            }
            Err(reason) => skipped_steps.push(LowerHeadThenWalkToSkippedStep::new(step, reason)),
        }
    }

    Ok(LowerHeadThenWalkToExecutionReport {
        task_key: plan.task_key.clone(),
        completed: matches!(
            state.result,
            Some(
                LowerHeadThenWalkToStepResult::Activated
                    | LowerHeadThenWalkToStepResult::InitialTargetMissing
                    | LowerHeadThenWalkToStepResult::Timeout
            )
        ),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_switch_party_plan<R>(
    plan: &SwitchPartyExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<SwitchPartyExecutionReport>
where
    R: SwitchPartyRuntime,
{
    let mut state = SwitchPartyExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_switch_party_step(step, &state) {
            Ok(()) => {
                let (outcome, nested_report) =
                    execute_switch_party_step(step, plan, key_bindings, runtime, &mut state)?;
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
                apply_switch_party_outcome(step, outcome, &mut state)?;
                executed_steps.push(SwitchPartyRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(SwitchPartySkippedStep::new(step, reason)),
        }
    }

    Ok(SwitchPartyExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_count_inventory_item_plan<R>(
    plan: &CountInventoryItemExecutionPlan,
    runtime: &mut R,
) -> Result<CountInventoryItemExecutionReport>
where
    R: CountInventoryItemRuntime,
{
    let mut state = CountInventoryItemExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_count_inventory_item_step(step, plan, &state) {
            Ok(()) => {
                let outcome = execute_count_inventory_item_step(step, plan, runtime, &mut state)?;
                apply_count_inventory_item_outcome(step, plan, outcome, &mut state)?;
                executed_steps.push(CountInventoryItemRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(CountInventoryItemSkippedStep::new(step, reason)),
        }
    }

    Ok(CountInventoryItemExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_scan_pick_drops_plan<R>(
    plan: &ScanPickDropsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<ScanPickDropsExecutionReport>
where
    R: ScanPickDropsRuntime,
{
    let mut state = ScanPickDropsExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_scan_pick_drops_step(step, plan, &state) {
            Ok(()) => {
                let outcome =
                    execute_scan_pick_drops_step(step, key_bindings, runtime, &mut state)?;
                apply_scan_pick_drops_outcome(step, outcome, &mut state)?;
                executed_steps.push(ScanPickDropsRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(ScanPickDropsSkippedStep::new(step, reason)),
        }
    }

    Ok(ScanPickDropsExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(ScanPickDropsStepResult::ScanComplete),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_one_key_expedition_plan<R>(
    plan: &OneKeyExpeditionExecutionPlan,
    runtime: &mut R,
) -> Result<OneKeyExpeditionExecutionReport>
where
    R: OneKeyExpeditionRuntime,
{
    let mut state = OneKeyExpeditionExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_one_key_expedition_step(step, plan, &state) {
            Ok(()) => {
                let outcome = execute_one_key_expedition_step(step, runtime)?;
                apply_one_key_expedition_outcome(step, outcome, &mut state)?;
                executed_steps.push(OneKeyExpeditionRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(OneKeyExpeditionSkippedStep::new(step, reason)),
        }
    }

    Ok(OneKeyExpeditionExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(OneKeyExpeditionStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_one_key_expedition_live<F, I, C>(
    plan: &OneKeyExpeditionExecutionPlan,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<OneKeyExpeditionExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    if plan.capture_size != RETURN_MAIN_UI_LIVE_CAPTURE_SIZE {
        return Err(TaskError::CommonJobExecution(format!(
            "OneKeyExpedition live template runtime currently supports {}x{} capture frames only; got {}x{}",
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.width,
            RETURN_MAIN_UI_LIVE_CAPTURE_SIZE.height,
            plan.capture_size.width,
            plan.capture_size.height
        )));
    }
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_one_key_expedition_plan(plan, &mut runtime)
}

pub fn execute_go_to_crafting_bench_plan<R>(
    plan: &GoToCraftingBenchExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<GoToCraftingBenchExecutionReport>
where
    R: GoToCraftingBenchRuntime,
{
    let mut state = GoToCraftingBenchExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();
    let mut nested_return_main_ui_reports = Vec::new();

    for step in &plan.steps {
        match should_execute_go_to_crafting_bench_step(step, plan, &state) {
            Ok(()) => {
                let (outcome, nested_report) = execute_go_to_crafting_bench_step(
                    step,
                    plan,
                    key_bindings,
                    runtime,
                    &mut state,
                )?;
                if let Some(report) = nested_report {
                    nested_return_main_ui_reports.push(report);
                }
                apply_go_to_crafting_bench_outcome(step, outcome, &mut state)?;
                executed_steps.push(GoToCraftingBenchRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(GoToCraftingBenchSkippedStep::new(step, reason)),
        }
    }

    Ok(GoToCraftingBenchExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(GoToCraftingBenchStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
        nested_return_main_ui_reports,
    })
}

pub fn execute_go_to_adventurers_guild_plan<R>(
    plan: &GoToAdventurersGuildExecutionPlan,
    runtime: &mut R,
) -> Result<GoToAdventurersGuildExecutionReport>
where
    R: GoToAdventurersGuildRuntime,
{
    let mut state = GoToAdventurersGuildExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_go_to_adventurers_guild_step(step, plan, &state) {
            Ok(()) => {
                let outcome = execute_go_to_adventurers_guild_step(step, runtime, &mut state)?;
                apply_go_to_adventurers_guild_outcome(step, outcome, &mut state)?;
                executed_steps.push(GoToAdventurersGuildRuntimeStepReport::executed(
                    step, outcome,
                ));
            }
            Err(reason) => skipped_steps.push(GoToAdventurersGuildSkippedStep::new(step, reason)),
        }
    }

    Ok(GoToAdventurersGuildExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(GoToAdventurersGuildStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_go_to_serenitea_pot_plan<R>(
    plan: &GoToSereniteaPotExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<GoToSereniteaPotExecutionReport>
where
    R: GoToSereniteaPotRuntime,
{
    let mut state = GoToSereniteaPotExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_go_to_serenitea_pot_step(step, plan, &state) {
            Ok(()) => {
                let outcome =
                    execute_go_to_serenitea_pot_step(step, key_bindings, runtime, &mut state)?;
                apply_go_to_serenitea_pot_outcome(step, outcome, &mut state)?;
                executed_steps.push(GoToSereniteaPotRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(GoToSereniteaPotSkippedStep::new(step, reason)),
        }
    }

    Ok(GoToSereniteaPotExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result == Some(GoToSereniteaPotStepResult::Completed),
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn execute_teleport_plan<R>(
    plan: &TeleportExecutionPlan,
    runtime: &mut R,
) -> Result<TeleportExecutionReport>
where
    R: TeleportRuntime,
{
    let mut state = TeleportExecutorState::default();
    let mut executed_steps = Vec::new();
    let max_attempts = if plan.kind == TeleportPlanKind::CoordinateTeleport {
        plan.retry_rule.max_attempts.max(1)
    } else {
        1
    };

    for attempt_index in 0..max_attempts {
        let mut retry_after_point_not_activated = false;
        for step in &plan.steps {
            let outcome = execute_teleport_step(step, runtime)?;
            let point_not_activated_click = plan.kind == TeleportPlanKind::CoordinateTeleport
                && matches!(
                    step.action,
                    TeleportStepAction::ClickTeleportPanelOrCandidate { .. }
                )
                && matches!(outcome, CommonJobRuntimeOutcome::Matched(false));
            let move_map_incomplete = plan.kind == TeleportPlanKind::MoveMapTo
                && matches!(step.action, TeleportStepAction::MoveMapTo { .. })
                && matches!(outcome, CommonJobRuntimeOutcome::Matched(false));

            apply_teleport_outcome(step, outcome, &mut state)?;
            if move_map_incomplete {
                return Err(TaskError::CommonJobExecution(
                    "Teleport MoveMapTo did not converge to the target window".to_string(),
                ));
            }
            if matches!(
                step.action,
                TeleportStepAction::SeedNavigationPreviousPositionAfterTeleport { .. }
            ) && state.navigation_previous_position_seeded
            {
                if let Some(target) = runtime.teleport_navigation_seed_target() {
                    state.navigation_previous_position_seed = Some(target);
                }
            }
            executed_steps.push(TeleportRuntimeStepReport::executed(step, outcome));

            if point_not_activated_click {
                retry_after_point_not_activated = true;
                continue;
            }
            if retry_after_point_not_activated
                && matches!(
                    step.action,
                    TeleportStepAction::HandlePointNotActivated { .. }
                )
            {
                if !state.point_not_activated_handled {
                    return Err(TaskError::CommonJobExecution(
                        "Teleport point-not-activated handler did not complete".to_string(),
                    ));
                }
                if attempt_index + 1 >= max_attempts {
                    return Err(TaskError::CommonJobExecution("传送失败".to_string()));
                }
                break;
            }
        }

        if retry_after_point_not_activated {
            continue;
        }

        return Ok(TeleportExecutionReport {
            task_key: plan.task_key.clone(),
            completed: teleport_plan_completed(plan.kind, &state),
            state,
            executed_steps,
        });
    }

    Err(TaskError::CommonJobExecution("传送失败".to_string()))
}

fn teleport_plan_completed(kind: TeleportPlanKind, state: &TeleportExecutorState) -> bool {
    if state.result != Some(TeleportStepResult::Planned) {
        return false;
    }
    match kind {
        TeleportPlanKind::MoveMapTo => state.move_map_completed,
        TeleportPlanKind::CoordinateTeleport | TeleportPlanKind::StatueOfTheSeven => true,
    }
}

pub fn execute_walk_to_f_live<F, I, C>(
    plan: &WalkToFExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    frame_source: F,
    input_driver: I,
    clock: C,
) -> Result<WalkToFExecutionReport>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    let mut runtime =
        PureTemplateCommonJobRuntime::with_task_assets(frame_source, input_driver, clock);
    execute_walk_to_f_plan(plan, key_bindings, &mut runtime)
}

pub fn execute_walk_to_f_plan<R>(
    plan: &WalkToFExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<WalkToFExecutionReport>
where
    R: CommonJobRuntime,
{
    let mut state = WalkToFExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_walk_to_f_step(step, plan, &state) {
            Ok(()) => {
                let outcome = execute_walk_to_f_step(step, key_bindings, runtime)?;
                apply_walk_to_f_outcome(step, outcome, &mut state)?;
                executed_steps.push(WalkToFRuntimeStepReport::executed(step, outcome));
            }
            Err(reason) => skipped_steps.push(WalkToFSkippedStep::new(step, reason)),
        }
    }

    Ok(WalkToFExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state.result.is_some(),
        state,
        executed_steps,
        skipped_steps,
    })
}

fn should_execute_blessing_of_the_welkin_moon_step(
    step: &BlessingOfTheWelkinMoonStep,
    state: &BlessingOfTheWelkinMoonExecutorState,
) -> std::result::Result<(), BlessingOfTheWelkinMoonSkipReason> {
    match step.condition {
        BlessingOfTheWelkinMoonStepCondition::Always => Ok(()),
        BlessingOfTheWelkinMoonStepCondition::WhenServerTimeInsideClaimWindow => {
            if !state.server_time_checked {
                Err(BlessingOfTheWelkinMoonSkipReason::ServerTimeNotChecked)
            } else if state.server_time_inside_claim_window {
                Ok(())
            } else {
                Err(BlessingOfTheWelkinMoonSkipReason::OutsideClaimWindow)
            }
        }
        BlessingOfTheWelkinMoonStepCondition::WhenBlessingOrPrimogemDetected
        | BlessingOfTheWelkinMoonStepCondition::UntilStableClear => {
            if !state.server_time_checked {
                return Err(BlessingOfTheWelkinMoonSkipReason::ServerTimeNotChecked);
            }
            if !state.server_time_inside_claim_window {
                return Err(BlessingOfTheWelkinMoonSkipReason::OutsideClaimWindow);
            }
            if step.condition == BlessingOfTheWelkinMoonStepCondition::UntilStableClear
                && state.cleared
            {
                return Err(BlessingOfTheWelkinMoonSkipReason::StableClearAlreadyReached);
            }
            match state.claim_ui_detected {
                Some(true) => Ok(()),
                Some(false) => Err(BlessingOfTheWelkinMoonSkipReason::ClaimUiMissing),
                None => Err(BlessingOfTheWelkinMoonSkipReason::ClaimUiProbeMissing),
            }
        }
    }
}

fn execute_blessing_of_the_welkin_moon_step<R>(
    step: &BlessingOfTheWelkinMoonStep,
    runtime: &mut R,
    server_minutes_since_midnight: u16,
    state: &mut BlessingOfTheWelkinMoonExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match &step.action {
        BlessingOfTheWelkinMoonStepAction::ServerTimeGate { gate } => {
            let inside = blessing_of_the_welkin_moon_inside_claim_window(
                gate,
                server_minutes_since_midnight,
            );
            state.server_time_checked = true;
            state.server_time_inside_claim_window = inside;
            Ok(CommonJobRuntimeOutcome::Matched(inside))
        }
        BlessingOfTheWelkinMoonStepAction::DetectClaimUi { locators } => {
            let detection = detect_blessing_of_the_welkin_moon_ui(locators, runtime)?;
            apply_blessing_of_the_welkin_moon_detection(detection, state);
            Ok(CommonJobRuntimeOutcome::Matched(detection.detected()))
        }
        BlessingOfTheWelkinMoonStepAction::Input { events } => {
            state.claim_click_dispatched = true;
            runtime.dispatch_input(events)
        }
        BlessingOfTheWelkinMoonStepAction::Page { command } => {
            runtime.execute_page_command(command)
        }
        BlessingOfTheWelkinMoonStepAction::LoopUntilClear {
            rule,
            locators,
            claim_click_events,
        } => execute_blessing_of_the_welkin_moon_clear_loop(
            rule,
            locators,
            claim_click_events,
            runtime,
            state,
        ),
        BlessingOfTheWelkinMoonStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_blessing_of_the_welkin_moon_clear_loop<R>(
    rule: &BlessingOfTheWelkinMoonLoopRule,
    locators: &BlessingOfTheWelkinMoonDetectionLocators,
    claim_click_events: &[InputEvent],
    runtime: &mut R,
    state: &mut BlessingOfTheWelkinMoonExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let max_iterations = rule.max_iterations.max(1);
    let stable_clear_target = rule.stable_clear_count.max(1);
    for iteration in 0..max_iterations {
        if state.stable_clear_count == 0 {
            runtime.dispatch_input(claim_click_events)?;
        }
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.retry_delay_ms,
        })?;

        let detection = detect_blessing_of_the_welkin_moon_ui(locators, runtime)?;
        apply_blessing_of_the_welkin_moon_detection(detection, state);
        state.clear_iterations = iteration + 1;
        if detection.detected() {
            state.stable_clear_count = 0;
        } else {
            state.stable_clear_count = state.stable_clear_count.saturating_add(1);
            if state.stable_clear_count >= stable_clear_target {
                state.cleared = true;
                break;
            }
        }
    }

    Ok(CommonJobRuntimeOutcome::Matched(state.cleared))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BlessingOfTheWelkinMoonDetection {
    girl_moon: Option<bool>,
    welkin_moon: Option<bool>,
    primogem: Option<bool>,
}

impl BlessingOfTheWelkinMoonDetection {
    fn detected(self) -> bool {
        self.girl_moon.unwrap_or(false)
            || self.welkin_moon.unwrap_or(false)
            || self.primogem.unwrap_or(false)
    }
}

fn detect_blessing_of_the_welkin_moon_ui<R>(
    locators: &BlessingOfTheWelkinMoonDetectionLocators,
    runtime: &mut R,
) -> Result<BlessingOfTheWelkinMoonDetection>
where
    R: CommonJobRuntime,
{
    let girl_moon = blessing_of_the_welkin_moon_locator_match(
        runtime.execute_locator(&locators.girl_moon)?,
        "girl moon",
    )?;
    if girl_moon {
        return Ok(BlessingOfTheWelkinMoonDetection {
            girl_moon: Some(true),
            ..BlessingOfTheWelkinMoonDetection::default()
        });
    }

    let welkin_moon = blessing_of_the_welkin_moon_locator_match(
        runtime.execute_locator(&locators.welkin_moon)?,
        "welkin moon",
    )?;
    if welkin_moon {
        return Ok(BlessingOfTheWelkinMoonDetection {
            girl_moon: Some(false),
            welkin_moon: Some(true),
            ..BlessingOfTheWelkinMoonDetection::default()
        });
    }

    let primogem = blessing_of_the_welkin_moon_locator_match(
        runtime.execute_locator(&locators.primogem)?,
        "primogem",
    )?;
    Ok(BlessingOfTheWelkinMoonDetection {
        girl_moon: Some(false),
        welkin_moon: Some(false),
        primogem: Some(primogem),
    })
}

fn apply_blessing_of_the_welkin_moon_detection(
    detection: BlessingOfTheWelkinMoonDetection,
    state: &mut BlessingOfTheWelkinMoonExecutorState,
) {
    state.girl_moon_detected = detection.girl_moon;
    state.welkin_moon_detected = detection.welkin_moon;
    state.primogem_detected = detection.primogem;
    state.claim_ui_detected = Some(detection.detected());
}

fn blessing_of_the_welkin_moon_locator_match(
    outcome: CommonJobRuntimeOutcome,
    label: &str,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "BlessingOfTheWelkinMoon {label} locator did not return a match result"
        ))),
    }
}

fn blessing_of_the_welkin_moon_inside_claim_window(
    gate: &BlessingOfTheWelkinMoonServerTimeGate,
    server_minutes_since_midnight: u16,
) -> bool {
    let adjusted_minutes =
        (server_minutes_since_midnight as i32 + gate.offset_minutes).rem_euclid(24 * 60);
    let reset_start = gate.reset_hour as i32 * 60;
    let reset_end = reset_start + gate.grace_minutes as i32;
    adjusted_minutes >= reset_start && adjusted_minutes < reset_end
}

fn blessing_of_the_welkin_moon_completed(state: &BlessingOfTheWelkinMoonExecutorState) -> bool {
    (state.server_time_checked && !state.server_time_inside_claim_window)
        || state.claim_ui_detected == Some(false)
        || state.cleared
}

fn current_server_minutes_since_midnight(server_time_zone_offset_minutes: i32) -> Result<u16> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            TaskError::CommonJobExecution(format!("failed to read system time: {error}"))
        })?;
    let server_seconds = elapsed.as_secs() as i64 + server_time_zone_offset_minutes as i64 * 60;
    Ok((server_seconds.rem_euclid(24 * 60 * 60) / 60) as u16)
}

fn should_execute_relogin_step(
    step: &ReloginStep,
    state: &ReloginExecutorState,
) -> std::result::Result<(), ReloginSkipReason> {
    match step.condition {
        ReloginStepCondition::Always => Ok(()),
        ReloginStepCondition::WhenMenuOpened => {
            if state.menu_open_probe_completed {
                Ok(())
            } else {
                Err(ReloginSkipReason::MenuOpenProbeMissing)
            }
        }
        ReloginStepCondition::WhenExitConfirmAppeared => {
            if state.exit_confirm_probe_completed {
                Ok(())
            } else {
                Err(ReloginSkipReason::ExitConfirmProbeMissing)
            }
        }
        ReloginStepCondition::WhenLoginScreenVisible => {
            if state.login_screen_visible {
                Ok(())
            } else {
                Err(ReloginSkipReason::LoginScreenMissing)
            }
        }
        ReloginStepCondition::WhenEnteredGame => {
            if state.enter_game_disappeared {
                Ok(())
            } else {
                Err(ReloginSkipReason::EnterGameNotConfirmed)
            }
        }
    }
}

fn execute_relogin_step<R>(step: &ReloginStep, runtime: &mut R) -> Result<CommonJobRuntimeOutcome>
where
    R: ReloginRuntime,
{
    match &step.action {
        ReloginStepAction::FocusGameWindow => runtime.focus_game_window(),
        ReloginStepAction::RetryUntilAppear {
            locator,
            rule,
            retry_action,
        } => {
            let outcome = execute_relogin_retry_until_appear(locator, rule, retry_action, runtime)?;
            enforce_relogin_failure_policy(outcome, rule, step)?;
            Ok(outcome)
        }
        ReloginStepAction::RetryUntilDisappear {
            locator,
            rule,
            retry_action,
        } => {
            let outcome =
                execute_relogin_retry_until_disappear(locator, rule, retry_action, runtime)?;
            enforce_relogin_failure_policy(outcome, rule, step)?;
            Ok(outcome)
        }
        ReloginStepAction::ThirdPartyLoginProbe { rule } => {
            runtime.execute_third_party_login_probe(rule)
        }
        ReloginStepAction::Page { command } => runtime.execute_page_command(command),
        ReloginStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        ReloginStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_relogin_retry_until_appear<R>(
    locator: &BvLocatorPlan,
    rule: &ReloginRetryRule,
    retry_action: &ReloginRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let attempts = rule.max_attempts.max(1);
    for attempt in 0..attempts {
        if relogin_outcome_as_match(runtime.execute_locator(locator)?)? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if attempt + 1 >= attempts {
            break;
        }
        execute_relogin_retry_action(retry_action, runtime)?;
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.interval_ms,
        })?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_relogin_retry_until_disappear<R>(
    locator: &BvLocatorPlan,
    rule: &ReloginRetryRule,
    retry_action: &ReloginRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let attempts = rule.max_attempts.max(1);
    for attempt in 0..attempts {
        if relogin_outcome_as_match(runtime.execute_locator(locator)?)? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if attempt + 1 >= attempts {
            break;
        }
        execute_relogin_retry_action(retry_action, runtime)?;
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.interval_ms,
        })?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_relogin_retry_action<R>(
    action: &ReloginRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match action {
        ReloginRetryAction::None => Ok(CommonJobRuntimeOutcome::None),
        ReloginRetryAction::Input { events } => runtime.dispatch_input(events),
        ReloginRetryAction::Page { command } => runtime.execute_page_command(command),
        ReloginRetryAction::Locator { locator } => runtime.execute_locator(locator),
    }
}

fn enforce_relogin_failure_policy(
    outcome: CommonJobRuntimeOutcome,
    rule: &ReloginRetryRule,
    step: &ReloginStep,
) -> Result<()> {
    if relogin_step_outcome_as_match(outcome, step)? {
        return Ok(());
    }
    match &rule.failure_policy {
        ReloginFailurePolicy::BestEffort | ReloginFailurePolicy::WarningOnly { .. } => Ok(()),
        ReloginFailurePolicy::HardError { message } => {
            Err(TaskError::CommonJobExecution(message.clone()))
        }
    }
}

fn apply_relogin_outcome(
    step: &ReloginStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ReloginExecutorState,
) -> Result<()> {
    match &step.action {
        ReloginStepAction::FocusGameWindow => {
            state.focus_requested = true;
        }
        ReloginStepAction::RetryUntilAppear { locator, .. } => {
            let matched = relogin_step_outcome_as_match(outcome, step)?;
            match locator.recognition_object.name.as_deref() {
                Some(RELOGIN_MENU_BAG) => {
                    state.menu_open_probe_completed = true;
                    state.menu_opened = matched;
                }
                Some(RELOGIN_CONFIRM) => {
                    state.exit_confirm_probe_completed = true;
                    state.exit_confirm_appeared = matched;
                }
                Some(RELOGIN_ENTER_GAME) => {
                    state.login_screen_probe_completed = true;
                    state.login_screen_visible = matched;
                }
                Some(RETURN_MAIN_UI_PAIMON_MENU) => {
                    state.main_ui_detected = matched;
                }
                _ => {}
            }
        }
        ReloginStepAction::RetryUntilDisappear { locator, .. } => {
            let matched = relogin_step_outcome_as_match(outcome, step)?;
            match locator.recognition_object.name.as_deref() {
                Some(RELOGIN_CONFIRM) => state.exit_confirm_disappeared = matched,
                Some(RELOGIN_ENTER_GAME) => state.enter_game_disappeared = matched,
                _ => {}
            }
        }
        ReloginStepAction::ThirdPartyLoginProbe { .. } => {
            state.third_party_login_checked = true;
            state.third_party_login_completed = relogin_step_outcome_as_match(outcome, step)?;
        }
        ReloginStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn relogin_outcome_as_match(outcome: CommonJobRuntimeOutcome) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(
            "relogin locator did not return a match result".to_string(),
        )),
    }
}

fn relogin_step_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &ReloginStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "relogin step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_claim_mail_rewards_step(
    step: &ClaimMailRewardsStep,
    state: &ClaimMailRewardsExecutorState,
) -> std::result::Result<(), ClaimMailRewardsSkipReason> {
    if state.result.is_some() {
        return Err(ClaimMailRewardsSkipReason::ResultAlreadySet);
    }

    match step.condition {
        ClaimMailRewardsStepCondition::Always => Ok(()),
        ClaimMailRewardsStepCondition::WhenMailRewardDetected => match state.mail_reward_detected {
            Some(true) => Ok(()),
            Some(false) => Err(ClaimMailRewardsSkipReason::MailRewardMissing),
            None => Err(ClaimMailRewardsSkipReason::MailRewardProbeMissing),
        },
        ClaimMailRewardsStepCondition::WhenMailRewardMissing => match state.mail_reward_detected {
            Some(false) => Ok(()),
            Some(true) => Err(ClaimMailRewardsSkipReason::MailRewardDetected),
            None => Err(ClaimMailRewardsSkipReason::MailRewardProbeMissing),
        },
        ClaimMailRewardsStepCondition::WhenCollectAllDetected => match state.collect_all_detected {
            Some(true) => Ok(()),
            Some(false) => Err(ClaimMailRewardsSkipReason::CollectAllMissing),
            None => Err(ClaimMailRewardsSkipReason::CollectAllProbeMissing),
        },
        ClaimMailRewardsStepCondition::WhenCollectAllMissing => match state.collect_all_detected {
            Some(false) => Ok(()),
            Some(true) => Err(ClaimMailRewardsSkipReason::CollectAllDetected),
            None => Err(ClaimMailRewardsSkipReason::CollectAllProbeMissing),
        },
        ClaimMailRewardsStepCondition::AfterClaimAttempt => {
            if state.mail_reward_detected.is_some() {
                Ok(())
            } else {
                Err(ClaimMailRewardsSkipReason::MailRewardProbeMissing)
            }
        }
    }
}

fn execute_claim_mail_rewards_step<R>(
    step: &ClaimMailRewardsStep,
    plan: &ClaimMailRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: CommonJobRuntime,
{
    match &step.action {
        ClaimMailRewardsStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        ClaimMailRewardsStepAction::CommonJob { task_key } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        ClaimMailRewardsStepAction::GenshinAction { action } => {
            let events = input_events_for_action(key_bindings, *action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            Ok((runtime.dispatch_input(&events)?, None))
        }
        ClaimMailRewardsStepAction::Input { events } => Ok((runtime.dispatch_input(events)?, None)),
        ClaimMailRewardsStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        ClaimMailRewardsStepAction::Locator { locator } => {
            Ok((runtime.execute_locator(locator)?, None))
        }
        ClaimMailRewardsStepAction::ReturnResult { .. } => {
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        ClaimMailRewardsStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn apply_claim_mail_rewards_outcome(
    step: &ClaimMailRewardsStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ClaimMailRewardsExecutorState,
) -> Result<()> {
    match &step.action {
        ClaimMailRewardsStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let completed = claim_mail_rewards_outcome_as_match(outcome, step)?;
            match step.phase {
                ClaimMailRewardsStepPhase::Setup => {
                    state.initial_return_main_ui_completed = Some(completed);
                }
                ClaimMailRewardsStepPhase::Cleanup => {
                    state.final_return_main_ui_completed = Some(completed);
                }
                _ => {}
            }
        }
        ClaimMailRewardsStepAction::GenshinAction {
            action: GenshinAction::OpenPaimonMenu,
        } => {
            state.paimon_menu_opened = true;
        }
        ClaimMailRewardsStepAction::Input { .. }
            if step.phase == ClaimMailRewardsStepPhase::MailClaim
                && step.condition == ClaimMailRewardsStepCondition::WhenCollectAllDetected =>
        {
            state.escape_after_claim_dispatched = true;
        }
        ClaimMailRewardsStepAction::Locator { locator } => {
            let matched = claim_mail_rewards_outcome_as_match(outcome, step)?;
            let name = locator.recognition_object.name.as_deref();
            if locator.operation == BvLocatorOperation::IsExist
                && name == Some(CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD)
            {
                state.mail_reward_detected = Some(matched);
            } else if locator.operation == BvLocatorOperation::Click
                && name == Some(CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD)
            {
                state.mail_reward_clicked = Some(matched);
                if !matched {
                    return Err(TaskError::CommonJobExecution(
                        "ClaimMailRewards failed to click the mail reward icon".to_string(),
                    ));
                }
            } else if locator.operation == BvLocatorOperation::IsExist
                && name == Some(CLAIM_MAIL_REWARDS_COLLECT)
            {
                state.collect_all_detected = Some(matched);
            } else if locator.operation == BvLocatorOperation::Click
                && name == Some(CLAIM_MAIL_REWARDS_COLLECT)
            {
                state.collect_all_clicked = Some(matched);
                if !matched {
                    return Err(TaskError::CommonJobExecution(
                        "ClaimMailRewards failed to click the collect-all button".to_string(),
                    ));
                }
            }
        }
        ClaimMailRewardsStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn claim_mail_rewards_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &ClaimMailRewardsStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "claim-mail-rewards step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_wonderland_cycle_step(
    step: &WonderlandCycleStep,
    state: &WonderlandCycleExecutorState,
) -> std::result::Result<(), WonderlandCycleSkipReason> {
    match step.condition {
        WonderlandCycleStepCondition::Always => Ok(()),
        WonderlandCycleStepCondition::WhenWonderlandMenuDetected => {
            if state.wonderland_menu_detected {
                Ok(())
            } else {
                Err(WonderlandCycleSkipReason::WonderlandMenuMissing)
            }
        }
        WonderlandCycleStepCondition::WhenConfirmDialogDetected => {
            let detected = match step.phase {
                WonderlandCycleStepPhase::EnterWonderland => state.enter_confirm_dialog_detected,
                WonderlandCycleStepPhase::ReturnTeyvat => state.return_confirm_dialog_detected,
                WonderlandCycleStepPhase::Cleanup => false,
            };
            if detected {
                Ok(())
            } else {
                Err(WonderlandCycleSkipReason::ConfirmDialogMissing)
            }
        }
        WonderlandCycleStepCondition::WhenInWonderlandMainUi => {
            if state.in_wonderland_main_ui || state.enter_confirm_dialog_disappeared {
                Ok(())
            } else {
                Err(WonderlandCycleSkipReason::WonderlandMainUiMissing)
            }
        }
        WonderlandCycleStepCondition::WhenBackTeyvatMenuDetected => {
            if state.back_teyvat_menu_detected {
                Ok(())
            } else {
                Err(WonderlandCycleSkipReason::BackTeyvatMenuMissing)
            }
        }
        WonderlandCycleStepCondition::WhenReturnedToTeyvat => {
            if state.returned_to_teyvat_main_ui || state.return_confirm_dialog_disappeared {
                Ok(())
            } else {
                Err(WonderlandCycleSkipReason::ReturnedMainUiMissing)
            }
        }
    }
}

fn execute_wonderland_cycle_step<R>(
    step: &WonderlandCycleStep,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match &step.action {
        WonderlandCycleStepAction::RetryUntilAppear {
            locator,
            rule,
            retry_action,
        } => execute_wonderland_cycle_retry_until_appear(locator, rule, retry_action, runtime),
        WonderlandCycleStepAction::RetryUntilDisappear {
            locator,
            rule,
            retry_action,
        } => execute_wonderland_cycle_retry_until_disappear(locator, rule, retry_action, runtime),
        WonderlandCycleStepAction::Page { command } => runtime.execute_page_command(command),
        WonderlandCycleStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        WonderlandCycleStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_wonderland_cycle_retry_until_appear<R>(
    locator: &BvLocatorPlan,
    rule: &WonderlandCycleRetryRule,
    retry_action: &WonderlandCycleRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let attempts = rule.max_attempts.max(1);
    for attempt in 0..attempts {
        if wonderland_cycle_outcome_as_match(runtime.execute_locator(locator)?)? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if attempt + 1 >= attempts {
            break;
        }
        execute_wonderland_cycle_retry_action(retry_action, runtime)?;
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.interval_ms,
        })?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_wonderland_cycle_retry_until_disappear<R>(
    locator: &BvLocatorPlan,
    rule: &WonderlandCycleRetryRule,
    retry_action: &WonderlandCycleRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let attempts = rule.max_attempts.max(1);
    for attempt in 0..attempts {
        if wonderland_cycle_outcome_as_match(runtime.execute_locator(locator)?)? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if attempt + 1 >= attempts {
            break;
        }
        execute_wonderland_cycle_retry_action(retry_action, runtime)?;
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.interval_ms,
        })?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_wonderland_cycle_retry_action<R>(
    action: &WonderlandCycleRetryAction,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match action {
        WonderlandCycleRetryAction::None => Ok(CommonJobRuntimeOutcome::None),
        WonderlandCycleRetryAction::Input { events } => runtime.dispatch_input(events),
        WonderlandCycleRetryAction::Page { command } => runtime.execute_page_command(command),
        WonderlandCycleRetryAction::Locator { locator } => runtime.execute_locator(locator),
    }
}

fn apply_wonderland_cycle_outcome(
    step: &WonderlandCycleStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut WonderlandCycleExecutorState,
) -> Result<()> {
    match &step.action {
        WonderlandCycleStepAction::RetryUntilAppear { locator, .. } => {
            let matched = wonderland_cycle_step_outcome_as_match(outcome, step)?;
            let name = locator.recognition_object.name.as_deref();
            match (step.phase, name) {
                (WonderlandCycleStepPhase::EnterWonderland, Some(WONDERLAND_CYCLE_CLOSE)) => {
                    state.wonderland_menu_detected = matched;
                }
                (
                    WonderlandCycleStepPhase::EnterWonderland,
                    Some(WONDERLAND_CYCLE_BLACK_CONFIRM),
                ) => {
                    state.enter_confirm_dialog_detected = matched;
                }
                (WonderlandCycleStepPhase::EnterWonderland, Some(RETURN_MAIN_UI_PAIMON_MENU)) => {
                    state.in_wonderland_main_ui = matched;
                }
                (WonderlandCycleStepPhase::ReturnTeyvat, Some(WONDERLAND_CYCLE_BACK_TEYVAT)) => {
                    state.back_teyvat_menu_detected = matched;
                }
                (WonderlandCycleStepPhase::ReturnTeyvat, Some(WONDERLAND_CYCLE_BLACK_CONFIRM)) => {
                    state.return_confirm_dialog_detected = matched;
                }
                (WonderlandCycleStepPhase::ReturnTeyvat, Some(RETURN_MAIN_UI_PAIMON_MENU)) => {
                    state.returned_to_teyvat_main_ui = matched;
                }
                _ => {}
            }
        }
        WonderlandCycleStepAction::RetryUntilDisappear { locator, .. } => {
            let matched = wonderland_cycle_step_outcome_as_match(outcome, step)?;
            if locator.recognition_object.name.as_deref() == Some(WONDERLAND_CYCLE_BLACK_CONFIRM) {
                match step.phase {
                    WonderlandCycleStepPhase::EnterWonderland => {
                        state.enter_confirm_dialog_disappeared = matched;
                    }
                    WonderlandCycleStepPhase::ReturnTeyvat => {
                        state.return_confirm_dialog_disappeared = matched;
                    }
                    WonderlandCycleStepPhase::Cleanup => {}
                }
            }
        }
        WonderlandCycleStepAction::ReturnResult { result } => {
            if *result == WonderlandCycleStepResult::EnteredWonderland {
                state.entered_wonderland_reported = true;
            }
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn wonderland_cycle_outcome_as_match(outcome: CommonJobRuntimeOutcome) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(
            "WonderlandCycle retry locator did not return a match result".to_string(),
        )),
    }
}

fn wonderland_cycle_step_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &WonderlandCycleStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "wonderland-cycle step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_choose_talk_option_step(
    step: &ChooseTalkOptionStep,
    plan: &ChooseTalkOptionExecutionPlan,
    state: &ChooseTalkOptionExecutorState,
) -> std::result::Result<(), ChooseTalkOptionSkipReason> {
    if state.result.is_some() {
        return Err(ChooseTalkOptionSkipReason::ResultAlreadySet);
    }
    if !state.talk_ui_detected && step.attempt.is_some() {
        return Err(ChooseTalkOptionSkipReason::TalkUiMissing);
    }

    match &step.action {
        ChooseTalkOptionStepAction::RecognizeOptions { .. }
        | ChooseTalkOptionStepAction::MatchText { .. }
        | ChooseTalkOptionStepAction::CheckOrange { .. }
        | ChooseTalkOptionStepAction::ClickMatchedOption
            if step.attempt.is_some() && !state.option_icons_detected =>
        {
            return Err(ChooseTalkOptionSkipReason::OptionIconMissing);
        }
        _ => {}
    }

    match step.condition {
        ChooseTalkOptionStepCondition::Always => Ok(()),
        ChooseTalkOptionStepCondition::FirstOcrPass => {
            if state.option_icons_detected && !state.first_ocr_stabilized {
                Ok(())
            } else if state.first_ocr_stabilized {
                Err(ChooseTalkOptionSkipReason::FirstOcrAlreadyStabilized)
            } else {
                Err(ChooseTalkOptionSkipReason::FirstOcrNotReady)
            }
        }
        ChooseTalkOptionStepCondition::WhenOptionIconMissing => {
            if state.option_icons_detected {
                Err(ChooseTalkOptionSkipReason::OptionIconPresent)
            } else {
                Ok(())
            }
        }
        ChooseTalkOptionStepCondition::WhenOptionTextMatched => {
            if state.matched_option.is_none() {
                Err(ChooseTalkOptionSkipReason::OptionTextMissing)
            } else if plan.is_orange && state.orange_accepted != Some(true) {
                Err(ChooseTalkOptionSkipReason::OrangeRejected)
            } else {
                Ok(())
            }
        }
        ChooseTalkOptionStepCondition::WhenOrangeRequired => {
            if !plan.is_orange {
                Err(ChooseTalkOptionSkipReason::OrangeNotRequired)
            } else if state.matched_option.is_none() {
                Err(ChooseTalkOptionSkipReason::OptionTextMissing)
            } else {
                Ok(())
            }
        }
        ChooseTalkOptionStepCondition::WhenOrangeRejected => {
            if !plan.is_orange {
                Err(ChooseTalkOptionSkipReason::OrangeNotRequired)
            } else if state.orange_accepted == Some(false) {
                Ok(())
            } else if state.orange_accepted == Some(true) {
                Err(ChooseTalkOptionSkipReason::OrangeAccepted)
            } else {
                Err(ChooseTalkOptionSkipReason::OptionTextMissing)
            }
        }
    }
}

fn execute_choose_talk_option_step<R>(
    step: &ChooseTalkOptionStep,
    plan: &ChooseTalkOptionExecutionPlan,
    runtime: &mut R,
    state: &mut ChooseTalkOptionExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: ChooseTalkOptionRuntime,
{
    if let Some(attempt) = step.attempt {
        if state.last_attempt != Some(attempt) {
            state.last_attempt = Some(attempt);
            state.option_icons_detected = false;
            state.recognized_options.clear();
            state.matched_option = None;
            state.orange_accepted = None;
            state.clicked = false;
        }
    }

    match &step.action {
        ChooseTalkOptionStepAction::Page { command } => runtime.execute_page_command(command),
        ChooseTalkOptionStepAction::Input { events } => runtime.dispatch_input(events),
        ChooseTalkOptionStepAction::Locator { locator } => runtime.execute_locator(locator),
        ChooseTalkOptionStepAction::RecognizeOptions { rule } => {
            state.recognized_options = runtime.recognize_talk_options(rule)?;
            Ok(CommonJobRuntimeOutcome::Matched(
                !state.recognized_options.is_empty(),
            ))
        }
        ChooseTalkOptionStepAction::MatchText { option } => {
            state.matched_option = state
                .recognized_options
                .iter()
                .find(|candidate| candidate.text.contains(option))
                .cloned();
            Ok(CommonJobRuntimeOutcome::Matched(
                state.matched_option.is_some(),
            ))
        }
        ChooseTalkOptionStepAction::CheckOrange { rule } => {
            let Some(candidate) = state.matched_option.clone() else {
                return Ok(CommonJobRuntimeOutcome::Matched(false));
            };
            let is_orange = runtime.is_orange_talk_option(&candidate, rule)?;
            state.orange_accepted = Some(is_orange);
            Ok(CommonJobRuntimeOutcome::Matched(is_orange))
        }
        ChooseTalkOptionStepAction::ClickMatchedOption => {
            let Some(candidate) = state.matched_option.clone() else {
                return Err(TaskError::CommonJobExecution(format!(
                    "{} has no matched option to click",
                    plan.task_key
                )));
            };
            runtime.click_talk_option(&candidate)?;
            state.clicked = true;
            Ok(CommonJobRuntimeOutcome::None)
        }
        ChooseTalkOptionStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        ChooseTalkOptionStepAction::Log { message } => runtime.log(message),
    }
}

fn apply_choose_talk_option_outcome(
    step: &ChooseTalkOptionStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ChooseTalkOptionExecutorState,
) -> Result<()> {
    match &step.action {
        ChooseTalkOptionStepAction::Locator { .. } if step.attempt.is_none() => {
            let detected = choose_talk_option_outcome_as_match(outcome, step)?;
            state.talk_ui_detected = detected;
            if !detected {
                state.result = Some(TalkOptionPlanResult::NotFound);
            }
        }
        ChooseTalkOptionStepAction::Locator { .. } if step.attempt.is_some() => {
            state.option_icons_detected = choose_talk_option_outcome_as_match(outcome, step)?;
        }
        ChooseTalkOptionStepAction::Page { .. }
            if step.condition == ChooseTalkOptionStepCondition::FirstOcrPass =>
        {
            state.first_ocr_stabilized = true;
        }
        ChooseTalkOptionStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn choose_talk_option_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &ChooseTalkOptionStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "choose-talk-option step {:?}/{} did not return a match result",
            step.condition, step.label
        ))),
    }
}

fn should_execute_check_rewards_step(
    step: &CheckRewardsStep,
    state: &CheckRewardsExecutorState,
) -> std::result::Result<(), CheckRewardsSkipReason> {
    if state.result.is_some() && step.condition != CheckRewardsStepCondition::AfterStatusCheck {
        return Err(CheckRewardsSkipReason::ResultAlreadySet);
    }

    match step.condition {
        CheckRewardsStepCondition::Always | CheckRewardsStepCondition::EachOpenRetry => Ok(()),
        CheckRewardsStepCondition::WhenCommissionsTextMatched => {
            if state.matched_commissions_text.is_some() {
                Ok(())
            } else {
                Err(CheckRewardsSkipReason::CommissionsTextMissing)
            }
        }
        CheckRewardsStepCondition::WhenDailyRewardTitleDetected => {
            match state.daily_reward_title_detected {
                Some(true) => Ok(()),
                Some(false) => Err(CheckRewardsSkipReason::DailyRewardTitleMissing),
                None => Err(CheckRewardsSkipReason::DailyRewardTitleProbeMissing),
            }
        }
        CheckRewardsStepCondition::WhenClaimedTextDetected => match state.claimed_text_detected {
            Some(true) => Ok(()),
            Some(false) => Err(CheckRewardsSkipReason::ClaimedTextMissing),
            None => Err(CheckRewardsSkipReason::ClaimedTextProbeMissing),
        },
        CheckRewardsStepCondition::WhenClaimedTextMissing => match state.claimed_text_detected {
            Some(false) => Ok(()),
            Some(true) => Err(CheckRewardsSkipReason::ClaimedTextDetected),
            None => Err(CheckRewardsSkipReason::ClaimedTextProbeMissing),
        },
        CheckRewardsStepCondition::AfterStatusCheck => {
            if state.claimed_text_detected.is_some() {
                Ok(())
            } else {
                Err(CheckRewardsSkipReason::StatusCheckMissing)
            }
        }
    }
}

fn execute_check_rewards_step<R>(
    step: &CheckRewardsStep,
    plan: &CheckRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut CheckRewardsExecutorState,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: CheckRewardsRuntime,
{
    match &step.action {
        CheckRewardsStepAction::CommonJob { task_key } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        CheckRewardsStepAction::CommonJob { task_key } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        CheckRewardsStepAction::GenshinAction { action } => {
            let events = input_events_for_action(key_bindings, *action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            state.open_attempts = state.open_attempts.saturating_add(1);
            Ok((runtime.dispatch_input(&events)?, None))
        }
        CheckRewardsStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        CheckRewardsStepAction::Ocr { command } => Ok((
            execute_check_rewards_open_retry_ocr(plan, key_bindings, command, runtime, state)?,
            None,
        )),
        CheckRewardsStepAction::MatchCommissions {
            text,
            click_first_match,
        } => {
            let matched = state
                .recognized_texts
                .iter()
                .find(|candidate| candidate.text.trim() == text.trim())
                .cloned();
            if *click_first_match {
                if let Some(candidate) = matched.as_ref() {
                    runtime.click_check_rewards_text(candidate)?;
                    state.commissions_text_clicked = true;
                }
            }
            state.matched_commissions_text = matched;
            Ok((
                CommonJobRuntimeOutcome::Matched(state.matched_commissions_text.is_some()),
                None,
            ))
        }
        CheckRewardsStepAction::Locator { locator }
            if step.condition == CheckRewardsStepCondition::WhenDailyRewardTitleDetected =>
        {
            Ok((
                execute_check_rewards_claim_status_retry(locator, &plan.claim_check_rule, runtime)?,
                None,
            ))
        }
        CheckRewardsStepAction::Locator { locator } => {
            Ok((runtime.execute_locator(locator)?, None))
        }
        CheckRewardsStepAction::Notify { payload } => {
            Ok((runtime.notify_check_rewards(payload)?, None))
        }
        CheckRewardsStepAction::ReturnResult { .. } => Ok((CommonJobRuntimeOutcome::None, None)),
        CheckRewardsStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn execute_check_rewards_open_retry_ocr<R>(
    plan: &CheckRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    command: &BvPageCommand,
    runtime: &mut R,
    state: &mut CheckRewardsExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CheckRewardsRuntime,
{
    let attempts = plan.open_handbook_rule.max_retries.max(1);
    let open_events = input_events_for_action(
        key_bindings,
        GenshinAction::OpenAdventurerHandbook,
        KeyActionType::KeyPress,
    )
    .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;

    loop {
        state.recognized_texts = runtime.recognize_check_rewards_text(command)?;
        let matched = state
            .recognized_texts
            .iter()
            .any(|candidate| candidate.text.trim() == plan.localized_texts.commissions_text.trim());
        if matched {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }

        if state.open_attempts >= attempts {
            break;
        }
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: plan.open_handbook_rule.interval_ms,
        })?;
        state.open_attempts = state.open_attempts.saturating_add(1);
        runtime.dispatch_input(&open_events)?;
    }

    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_check_rewards_claim_status_retry<R>(
    locator: &BvLocatorPlan,
    rule: &CheckRewardsRetryRule,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let attempts = rule.max_retries.max(1);
    for attempt in 0..attempts {
        if check_rewards_outcome_as_match(runtime.execute_locator(locator)?, "claimed text")? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if attempt + 1 >= attempts {
            break;
        }
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.interval_ms,
        })?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn apply_check_rewards_outcome(
    step: &CheckRewardsStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut CheckRewardsExecutorState,
) -> Result<()> {
    match &step.action {
        CheckRewardsStepAction::CommonJob { task_key } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let completed = check_rewards_step_outcome_as_match(outcome, step)?;
            match step.phase {
                CheckRewardsStepPhase::Setup => {
                    state.initial_return_main_ui_completed = Some(completed);
                }
                CheckRewardsStepPhase::Cleanup => {
                    state.final_return_main_ui_completed = Some(completed);
                }
                _ => {}
            }
        }
        CheckRewardsStepAction::GenshinAction {
            action: GenshinAction::OpenAdventurerHandbook,
        } => {
            state.handbook_open_dispatched = true;
        }
        CheckRewardsStepAction::Locator { .. }
            if step.condition == CheckRewardsStepCondition::WhenCommissionsTextMatched =>
        {
            state.daily_reward_title_detected =
                Some(check_rewards_step_outcome_as_match(outcome, step)?);
        }
        CheckRewardsStepAction::Locator { .. }
            if step.condition == CheckRewardsStepCondition::WhenDailyRewardTitleDetected =>
        {
            state.claimed_text_detected = Some(check_rewards_step_outcome_as_match(outcome, step)?);
        }
        CheckRewardsStepAction::Notify { payload } => {
            state.notifications_sent.push(payload.clone());
        }
        CheckRewardsStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn check_rewards_outcome_as_match(outcome: CommonJobRuntimeOutcome, label: &str) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "CheckRewards {label} did not return a match result"
        ))),
    }
}

fn check_rewards_step_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &CheckRewardsStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "check-rewards step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_claim_battle_pass_rewards_step(
    step: &BattlePassRewardStep,
    state: &ClaimBattlePassRewardsExecutorState,
) -> std::result::Result<(), ClaimBattlePassRewardsSkipReason> {
    if state.result.is_some() {
        return Err(ClaimBattlePassRewardsSkipReason::ResultAlreadySet);
    }

    match step.condition {
        BattlePassRewardStepCondition::Always => Ok(()),
        BattlePassRewardStepCondition::WhenClaimAllTextMatched => {
            let stage = claim_battle_pass_rewards_stage_for_condition(step, state)?;
            if stage.matched_claim_all_text.is_some() {
                Ok(())
            } else {
                Err(ClaimBattlePassRewardsSkipReason::ClaimAllTextMissing)
            }
        }
        BattlePassRewardStepCondition::AfterClaimClick => {
            let stage = claim_battle_pass_rewards_stage_for_condition(step, state)?;
            if stage.claim_clicked {
                Ok(())
            } else {
                Err(ClaimBattlePassRewardsSkipReason::ClaimNotClicked)
            }
        }
        BattlePassRewardStepCondition::WhenPrimogemDetected => match step.scope {
            Some(BattlePassClaimScope::UpgradeAnimation) => Ok(()),
            Some(BattlePassClaimScope::Points) | Some(BattlePassClaimScope::Rewards) => {
                let stage = claim_battle_pass_rewards_stage_for_condition(step, state)?;
                if !stage.claim_clicked {
                    Err(ClaimBattlePassRewardsSkipReason::ClaimNotClicked)
                } else if stage.manual_selection_dialog_detected == Some(true) {
                    Err(ClaimBattlePassRewardsSkipReason::ManualSelectionDialogDetected)
                } else {
                    Ok(())
                }
            }
            None => Err(ClaimBattlePassRewardsSkipReason::ScopeMissing),
        },
        BattlePassRewardStepCondition::WhenManualSelectionDialogDetected => {
            let stage = claim_battle_pass_rewards_stage_for_condition(step, state)?;
            if stage.manual_selection_dialog_detected == Some(true) {
                Ok(())
            } else {
                Err(ClaimBattlePassRewardsSkipReason::ManualSelectionDialogDetected)
            }
        }
        BattlePassRewardStepCondition::AfterClaimStages => {
            if claim_battle_pass_rewards_claim_stages_reached(state) {
                Ok(())
            } else {
                Err(ClaimBattlePassRewardsSkipReason::ClaimStagesNotReached)
            }
        }
    }
}

fn execute_claim_battle_pass_rewards_step<R>(
    step: &BattlePassRewardStep,
    plan: &ClaimBattlePassRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut ClaimBattlePassRewardsExecutorState,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: ClaimBattlePassRewardsRuntime,
{
    match &step.action {
        BattlePassRewardStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        BattlePassRewardStepAction::CommonJob { task_key } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        BattlePassRewardStepAction::GenshinAction { action } => {
            let events = input_events_for_action(key_bindings, *action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            Ok((runtime.dispatch_input(&events)?, None))
        }
        BattlePassRewardStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        BattlePassRewardStepAction::Ocr { command } => {
            let scope = step
                .scope
                .ok_or(ClaimBattlePassRewardsSkipReason::ScopeMissing)
                .map_err(|reason| {
                    TaskError::CommonJobExecution(format!(
                        "ClaimBattlePassRewards OCR scope missing: {reason:?}"
                    ))
                })?;
            let texts =
                runtime.recognize_battle_pass_reward_text(command, &plan.claim_all_rule, scope)?;
            claim_battle_pass_rewards_stage_mut(step, state)?.recognized_texts = texts;
            Ok((CommonJobRuntimeOutcome::Matched(true), None))
        }
        BattlePassRewardStepAction::MatchClaimAll { rule } => {
            let matched = claim_battle_pass_rewards_match_claim_all_text(
                &claim_battle_pass_rewards_stage(step, state)?.recognized_texts,
                rule,
            )?;
            claim_battle_pass_rewards_stage_mut(step, state)?.matched_claim_all_text =
                matched.clone();
            Ok((CommonJobRuntimeOutcome::Matched(matched.is_some()), None))
        }
        BattlePassRewardStepAction::ClickMatchedText => {
            let scope = step
                .scope
                .ok_or(ClaimBattlePassRewardsSkipReason::ScopeMissing)
                .map_err(|reason| {
                    TaskError::CommonJobExecution(format!(
                        "ClaimBattlePassRewards click scope missing: {reason:?}"
                    ))
                })?;
            let Some(candidate) = claim_battle_pass_rewards_stage(step, state)?
                .matched_claim_all_text
                .clone()
            else {
                return Err(TaskError::CommonJobExecution(format!(
                    "{} has no matched claim-all text to click",
                    plan.task_key
                )));
            };
            runtime.click_battle_pass_reward_text(&candidate, scope)?;
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        BattlePassRewardStepAction::DetectManualSelectionDialog { rule } => Ok((
            execute_battle_pass_manual_selection_dialog_detection(rule, runtime)?,
            None,
        )),
        BattlePassRewardStepAction::DismissPrimogemIfVisible { locator, events } => Ok((
            execute_battle_pass_primogem_dismiss(locator, events, runtime)?,
            None,
        )),
        BattlePassRewardStepAction::ReturnResult { .. } => {
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        BattlePassRewardStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn execute_battle_pass_manual_selection_dialog_detection<R>(
    rule: &BattlePassManualSelectionDialogRule,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    if rule.prompt_star_or_cancel_and_confirm
        && claim_battle_pass_rewards_outcome_as_bool(
            runtime.execute_locator(&rule.prompt_star_locator)?,
            "manual selection prompt-star locator",
        )?
    {
        return Ok(CommonJobRuntimeOutcome::Matched(true));
    }

    let mut has_cancel = false;
    for locator in &rule.cancel_locators {
        if claim_battle_pass_rewards_outcome_as_bool(
            runtime.execute_locator(locator)?,
            "manual selection cancel locator",
        )? {
            has_cancel = true;
            break;
        }
    }
    if !has_cancel {
        return Ok(CommonJobRuntimeOutcome::Matched(false));
    }

    for locator in &rule.confirm_locators {
        if claim_battle_pass_rewards_outcome_as_bool(
            runtime.execute_locator(locator)?,
            "manual selection confirm locator",
        )? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
    }

    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn execute_battle_pass_primogem_dismiss<R>(
    locator: &BvLocatorPlan,
    events: &[InputEvent],
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let visible = claim_battle_pass_rewards_outcome_as_bool(
        runtime.execute_locator(locator)?,
        "battle-pass primogem locator",
    )?;
    if visible {
        runtime.dispatch_input(events)?;
    }
    Ok(CommonJobRuntimeOutcome::Matched(visible))
}

fn apply_claim_battle_pass_rewards_outcome(
    step: &BattlePassRewardStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ClaimBattlePassRewardsExecutorState,
) -> Result<()> {
    match &step.action {
        BattlePassRewardStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let completed = claim_battle_pass_rewards_step_outcome_as_match(outcome, step)?;
            match step.phase {
                BattlePassRewardStepPhase::Setup => {
                    state.initial_return_main_ui_completed = Some(completed);
                }
                BattlePassRewardStepPhase::Cleanup => {
                    state.final_return_main_ui_completed = Some(completed);
                }
                _ => {}
            }
        }
        BattlePassRewardStepAction::GenshinAction {
            action: GenshinAction::OpenBattlePassScreen,
        } => {
            state.battle_pass_open_dispatched = true;
        }
        BattlePassRewardStepAction::ClickMatchedText => {
            claim_battle_pass_rewards_stage_mut(step, state)?.claim_clicked = true;
        }
        BattlePassRewardStepAction::DetectManualSelectionDialog { .. } => {
            let detected = claim_battle_pass_rewards_step_outcome_as_match(outcome, step)?;
            claim_battle_pass_rewards_stage_mut(step, state)?.manual_selection_dialog_detected =
                Some(detected);
        }
        BattlePassRewardStepAction::DismissPrimogemIfVisible { .. } => {
            let visible = claim_battle_pass_rewards_step_outcome_as_match(outcome, step)?;
            match step.scope {
                Some(BattlePassClaimScope::UpgradeAnimation) => {
                    state.upgrade_primogem_detected = Some(visible);
                    state.upgrade_primogem_dismissed = visible;
                }
                Some(BattlePassClaimScope::Points) | Some(BattlePassClaimScope::Rewards) => {
                    let stage = claim_battle_pass_rewards_stage_mut(step, state)?;
                    stage.primogem_detected = Some(visible);
                    stage.primogem_dismissed = visible;
                }
                None => {}
            }
        }
        BattlePassRewardStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn claim_battle_pass_rewards_stage_for_condition<'a>(
    step: &BattlePassRewardStep,
    state: &'a ClaimBattlePassRewardsExecutorState,
) -> std::result::Result<&'a BattlePassClaimStageState, ClaimBattlePassRewardsSkipReason> {
    match step.scope {
        Some(BattlePassClaimScope::Points) => Ok(&state.points_claim),
        Some(BattlePassClaimScope::Rewards) => Ok(&state.rewards_claim),
        _ => Err(ClaimBattlePassRewardsSkipReason::ScopeMissing),
    }
}

fn claim_battle_pass_rewards_stage<'a>(
    step: &BattlePassRewardStep,
    state: &'a ClaimBattlePassRewardsExecutorState,
) -> Result<&'a BattlePassClaimStageState> {
    match step.scope {
        Some(BattlePassClaimScope::Points) => Ok(&state.points_claim),
        Some(BattlePassClaimScope::Rewards) => Ok(&state.rewards_claim),
        _ => Err(TaskError::CommonJobExecution(format!(
            "ClaimBattlePassRewards step {} has no claim stage scope",
            step.label
        ))),
    }
}

fn claim_battle_pass_rewards_stage_mut<'a>(
    step: &BattlePassRewardStep,
    state: &'a mut ClaimBattlePassRewardsExecutorState,
) -> Result<&'a mut BattlePassClaimStageState> {
    match step.scope {
        Some(BattlePassClaimScope::Points) => Ok(&mut state.points_claim),
        Some(BattlePassClaimScope::Rewards) => Ok(&mut state.rewards_claim),
        _ => Err(TaskError::CommonJobExecution(format!(
            "ClaimBattlePassRewards step {} has no claim stage scope",
            step.label
        ))),
    }
}

fn claim_battle_pass_rewards_claim_stages_reached(
    state: &ClaimBattlePassRewardsExecutorState,
) -> bool {
    state.points_claim.matched_claim_all_text.is_some()
        || state.points_claim.recognized_texts.is_empty()
        || state.rewards_claim.matched_claim_all_text.is_some()
        || state.rewards_claim.recognized_texts.is_empty()
}

fn claim_battle_pass_rewards_match_claim_all_text(
    candidates: &[BattlePassRewardTextCandidate],
    rule: &BattlePassClaimAllRule,
) -> Result<Option<BattlePassRewardTextCandidate>> {
    for candidate in candidates {
        for pattern in &rule.claim_text_patterns {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                continue;
            }
            let matched = if rule.match_as_regex {
                Regex::new(pattern)
                    .map_err(|error| {
                        TaskError::CommonJobExecution(format!(
                            "invalid ClaimBattlePassRewards claim-all regex pattern {pattern:?}: {error}"
                        ))
                    })?
                    .is_match(&candidate.text)
            } else {
                candidate.text.contains(pattern)
            };
            if matched {
                return Ok(Some(candidate.clone()));
            }
        }
    }
    Ok(None)
}

fn claim_battle_pass_rewards_outcome_as_bool(
    outcome: CommonJobRuntimeOutcome,
    label: &str,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "ClaimBattlePassRewards {label} did not return a match result"
        ))),
    }
}

fn claim_battle_pass_rewards_step_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &BattlePassRewardStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "claim-battle-pass-rewards step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_claim_encounter_points_rewards_step(
    step: &ClaimEncounterPointsRewardsStep,
    state: &ClaimEncounterPointsRewardsExecutorState,
) -> std::result::Result<(), ClaimEncounterPointsRewardsSkipReason> {
    if state.result.is_some() {
        return Err(ClaimEncounterPointsRewardsSkipReason::ResultAlreadySet);
    }

    match step.condition {
        ClaimEncounterPointsRewardsStepCondition::Always => Ok(()),
        ClaimEncounterPointsRewardsStepCondition::WhenCommissionsTextMatched => {
            if state.matched_commissions_text.is_some() {
                Ok(())
            } else {
                Err(ClaimEncounterPointsRewardsSkipReason::CommissionsTextMissing)
            }
        }
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonDetected => {
            match state.early_claim_button_detected {
                Some(true) => Ok(()),
                Some(false) => Err(ClaimEncounterPointsRewardsSkipReason::EarlyClaimButtonMissing),
                None => Err(ClaimEncounterPointsRewardsSkipReason::EarlyClaimButtonProbeMissing),
            }
        }
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonMissing => {
            if state.matched_commissions_text.is_none() {
                Err(ClaimEncounterPointsRewardsSkipReason::CommissionsTextMissing)
            } else {
                match state.early_claim_button_detected {
                    Some(false) => Ok(()),
                    Some(true) => {
                        Err(ClaimEncounterPointsRewardsSkipReason::EarlyClaimButtonDetected)
                    }
                    None => {
                        Err(ClaimEncounterPointsRewardsSkipReason::EarlyClaimButtonProbeMissing)
                    }
                }
            }
        }
        ClaimEncounterPointsRewardsStepCondition::WhenClaimButtonDetected => {
            match state.claim_button_detected {
                Some(true) => Ok(()),
                Some(false) => Err(ClaimEncounterPointsRewardsSkipReason::ClaimButtonMissing),
                None => Err(ClaimEncounterPointsRewardsSkipReason::ClaimButtonProbeMissing),
            }
        }
        ClaimEncounterPointsRewardsStepCondition::AfterOpenRetryLimit => {
            if state.open_retry_limit_reached {
                Ok(())
            } else {
                Err(ClaimEncounterPointsRewardsSkipReason::OpenRetryLimitNotReached)
            }
        }
    }
}

fn execute_claim_encounter_points_rewards_step<R>(
    step: &ClaimEncounterPointsRewardsStep,
    plan: &ClaimEncounterPointsRewardsExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut ClaimEncounterPointsRewardsExecutorState,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: ClaimEncounterPointsRewardsRuntime,
{
    match &step.action {
        ClaimEncounterPointsRewardsStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        ClaimEncounterPointsRewardsStepAction::CommonJob { task_key } => {
            Err(TaskError::CommonJobExecution(format!(
                "nested common job execution is not supported yet: {task_key}"
            )))
        }
        ClaimEncounterPointsRewardsStepAction::GenshinAction { action } => {
            let events = input_events_for_action(key_bindings, *action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            Ok((runtime.dispatch_input(&events)?, None))
        }
        ClaimEncounterPointsRewardsStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        ClaimEncounterPointsRewardsStepAction::Ocr { command } => Ok((
            execute_claim_encounter_points_rewards_ocr(plan, command, runtime, state)?,
            None,
        )),
        ClaimEncounterPointsRewardsStepAction::MatchCommissions { rule } => {
            let matched = claim_encounter_points_rewards_match_commissions_text(
                &state.recognized_texts,
                rule,
            );
            state.matched_commissions_text = matched;
            state.open_retry_limit_reached = state.matched_commissions_text.is_none()
                && state.ocr_attempts >= rule.max_open_retries.max(1);
            Ok((
                CommonJobRuntimeOutcome::Matched(state.matched_commissions_text.is_some()),
                None,
            ))
        }
        ClaimEncounterPointsRewardsStepAction::Locator { locator } => {
            Ok((runtime.execute_locator(locator)?, None))
        }
        ClaimEncounterPointsRewardsStepAction::ClickMatchedText => {
            let Some(candidate) = state.matched_commissions_text.clone() else {
                return Err(TaskError::CommonJobExecution(format!(
                    "{} has no matched commissions text to click",
                    plan.task_key
                )));
            };
            runtime.click_encounter_points_text(&candidate)?;
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        ClaimEncounterPointsRewardsStepAction::ReturnResult { .. } => {
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        ClaimEncounterPointsRewardsStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn execute_claim_encounter_points_rewards_ocr<R>(
    plan: &ClaimEncounterPointsRewardsExecutionPlan,
    command: &BvPageCommand,
    runtime: &mut R,
    state: &mut ClaimEncounterPointsRewardsExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: ClaimEncounterPointsRewardsRuntime,
{
    let attempts = plan.max_open_retries.max(1);
    let mut matched = false;
    for _ in 0..attempts {
        state.ocr_attempts = state.ocr_attempts.saturating_add(1);
        state.recognized_texts =
            runtime.recognize_encounter_points_text(command, &plan.ocr_rule)?;
        matched = claim_encounter_points_rewards_match_commissions_text(
            &state.recognized_texts,
            &plan.ocr_rule,
        )
        .is_some();
        if matched {
            break;
        }
    }
    Ok(CommonJobRuntimeOutcome::Matched(matched))
}

fn apply_claim_encounter_points_rewards_outcome(
    step: &ClaimEncounterPointsRewardsStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ClaimEncounterPointsRewardsExecutorState,
) -> Result<()> {
    match &step.action {
        ClaimEncounterPointsRewardsStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let completed = claim_encounter_points_rewards_outcome_as_match(outcome, step)?;
            if step.condition == ClaimEncounterPointsRewardsStepCondition::Always {
                state.initial_return_main_ui_completed = Some(completed);
            } else {
                state.final_return_main_ui_completed = Some(completed);
            }
        }
        ClaimEncounterPointsRewardsStepAction::GenshinAction {
            action: GenshinAction::OpenAdventurerHandbook,
        } => {
            state.handbook_open_dispatched = true;
        }
        ClaimEncounterPointsRewardsStepAction::Locator { .. } => {
            let matched = claim_encounter_points_rewards_outcome_as_match(outcome, step)?;
            match step.condition {
                ClaimEncounterPointsRewardsStepCondition::WhenCommissionsTextMatched => {
                    state.early_claim_button_detected = Some(matched);
                }
                ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonMissing => {
                    state.claim_button_detected = Some(matched);
                }
                _ => {}
            }
        }
        ClaimEncounterPointsRewardsStepAction::ClickMatchedText => {
            state.matched_text_clicked = true;
        }
        ClaimEncounterPointsRewardsStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn claim_encounter_points_rewards_match_commissions_text(
    candidates: &[ClaimEncounterPointsRewardsTextCandidate],
    rule: &ClaimEncounterPointsRewardsOcrRule,
) -> Option<ClaimEncounterPointsRewardsTextCandidate> {
    let expected = rule.commissions_text.trim();
    candidates
        .iter()
        .find(|candidate| {
            if rule.match_exact_text {
                candidate.text.trim() == expected
            } else {
                candidate.text.contains(expected)
            }
        })
        .cloned()
}

fn claim_encounter_points_rewards_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &ClaimEncounterPointsRewardsStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "claim-encounter-points-rewards step {:?}/{} did not return a match result",
            step.condition, step.label
        ))),
    }
}

fn should_execute_walk_to_f_step(
    step: &WalkToFStep,
    plan: &WalkToFExecutionPlan,
    state: &WalkToFExecutorState,
) -> std::result::Result<(), WalkToFSkipReason> {
    match step.condition {
        WalkToFStepCondition::Always | WalkToFStepCondition::Finally => Ok(()),
        WalkToFStepCondition::WhenRunToF => {
            if plan.run_to_f {
                Ok(())
            } else {
                Err(WalkToFSkipReason::RunToFDisabled)
            }
        }
        WalkToFStepCondition::WhenPickDetected => {
            if !state.pick_detected {
                Err(WalkToFSkipReason::PickMissing)
            } else if plan.need_press {
                Err(WalkToFSkipReason::NeedPressEnabled)
            } else {
                Ok(())
            }
        }
        WalkToFStepCondition::WhenNeedPressAndPickDetected => {
            if !state.pick_detected {
                Err(WalkToFSkipReason::PickMissing)
            } else if !plan.need_press {
                Err(WalkToFSkipReason::NeedPressDisabled)
            } else {
                Ok(())
            }
        }
        WalkToFStepCondition::WhenPickMissing => {
            if state.pick_detected {
                Err(WalkToFSkipReason::PickDetected)
            } else {
                Ok(())
            }
        }
    }
}

fn should_execute_lower_head_then_walk_to_step(
    step: &LowerHeadThenWalkToStep,
    state: &LowerHeadThenWalkToExecutorState,
) -> std::result::Result<(), LowerHeadThenWalkToSkipReason> {
    if state.result.is_some() && step.condition != LowerHeadThenWalkToStepCondition::Finally {
        return Err(LowerHeadThenWalkToSkipReason::ResultAlreadySet);
    }

    match step.condition {
        LowerHeadThenWalkToStepCondition::Always | LowerHeadThenWalkToStepCondition::Finally => {
            Ok(())
        }
        LowerHeadThenWalkToStepCondition::WhenInitialTargetFound => {
            match state.initial_target_detected {
                Some(true) => Ok(()),
                Some(false) => Err(LowerHeadThenWalkToSkipReason::InitialTargetMissing),
                None => Err(LowerHeadThenWalkToSkipReason::InitialTargetProbeMissing),
            }
        }
        LowerHeadThenWalkToStepCondition::WhenInitialTargetMissing => {
            match state.initial_target_detected {
                Some(false) => Ok(()),
                Some(true) => Err(LowerHeadThenWalkToSkipReason::InitialTargetDetected),
                None => Err(LowerHeadThenWalkToSkipReason::InitialTargetProbeMissing),
            }
        }
        LowerHeadThenWalkToStepCondition::WhenActivationTextDetected => {
            if state.activation_text_detected {
                Ok(())
            } else {
                Err(LowerHeadThenWalkToSkipReason::ActivationTextMissing)
            }
        }
        LowerHeadThenWalkToStepCondition::WhenTimeout => {
            if state.timed_out {
                Ok(())
            } else {
                Err(LowerHeadThenWalkToSkipReason::TimeoutMissing)
            }
        }
    }
}

fn should_execute_switch_party_step(
    step: &SwitchPartyStep,
    state: &SwitchPartyExecutorState,
) -> std::result::Result<(), SwitchPartySkipReason> {
    if state.result.is_some() {
        return Err(SwitchPartySkipReason::ResultAlreadySet);
    }

    match step.condition {
        SwitchPartyStepCondition::Always => Ok(()),
        SwitchPartyStepCondition::WhenMainUiMissing => Ok(()),
        SwitchPartyStepCondition::WhenPartyViewMissing => {
            if state.party_view_opened == Some(true) {
                Err(SwitchPartySkipReason::PartyViewAlreadyOpened)
            } else {
                Ok(())
            }
        }
        SwitchPartyStepCondition::WhenCurrentPartyMatched => match state.current_party_matched {
            Some(true) => Ok(()),
            Some(false) => Err(SwitchPartySkipReason::CurrentPartyNotMatched),
            None => Err(SwitchPartySkipReason::CurrentPartyMatchMissing),
        },
        SwitchPartyStepCondition::WhenCurrentPartyNotMatched => match state.current_party_matched {
            Some(false) => Ok(()),
            Some(true) => Err(SwitchPartySkipReason::CurrentPartyMatched),
            None => Err(SwitchPartySkipReason::CurrentPartyMatchMissing),
        },
        SwitchPartyStepCondition::WhenPartyMatchedInList => {
            if state.matched_party.is_some() {
                Ok(())
            } else if state.party_not_found {
                Err(SwitchPartySkipReason::PartyMissingInList)
            } else {
                Err(SwitchPartySkipReason::PartyListScanMissing)
            }
        }
        SwitchPartyStepCondition::WhenPartyNotFound => {
            if state.party_not_found {
                Ok(())
            } else if state.matched_party.is_some() {
                Err(SwitchPartySkipReason::PartyMatchedInList)
            } else {
                Err(SwitchPartySkipReason::PartyListScanMissing)
            }
        }
    }
}

fn should_execute_go_to_crafting_bench_step(
    step: &GoToCraftingBenchStep,
    plan: &GoToCraftingBenchExecutionPlan,
    state: &GoToCraftingBenchExecutorState,
) -> std::result::Result<(), GoToCraftingBenchSkipReason> {
    if state.result.is_some() {
        return Err(GoToCraftingBenchSkipReason::ResultAlreadySet);
    }

    match step.condition {
        GoToCraftingBenchStepCondition::Always => Ok(()),
        GoToCraftingBenchStepCondition::WhenTalkUiMissing => match state.talk_ui_detected {
            Some(false) => Ok(()),
            Some(true) => Err(GoToCraftingBenchSkipReason::TalkUiDetected),
            None => Err(GoToCraftingBenchSkipReason::TalkUiProbeMissing),
        },
        GoToCraftingBenchStepCondition::WhenTalkUiStillMissing => {
            if state.talk_ui_detected == Some(true) {
                Err(GoToCraftingBenchSkipReason::TalkUiDetected)
            } else {
                Ok(())
            }
        }
        GoToCraftingBenchStepCondition::WhenCraftingPageOpen => match state.crafting_page_opened {
            Some(true) => Ok(()),
            Some(false) => Err(GoToCraftingBenchSkipReason::CraftingPageMissing),
            None => Err(GoToCraftingBenchSkipReason::CraftingPageProbeMissing),
        },
        GoToCraftingBenchStepCondition::WhenCondensedResinVisible => {
            match state.condensed_resin_visible {
                Some(true) => Ok(()),
                Some(false) => Err(GoToCraftingBenchSkipReason::CondensedResinMissing),
                None => Err(GoToCraftingBenchSkipReason::CondensedResinProbeMissing),
            }
        }
        GoToCraftingBenchStepCondition::WhenMinResinToKeepEnabled => {
            if state.resin_count_recognition_failed {
                Err(GoToCraftingBenchSkipReason::ResinCountRecognitionFailed)
            } else if state.condensed_resin_visible != Some(true) {
                Err(GoToCraftingBenchSkipReason::CondensedResinMissing)
            } else if plan.min_resin_to_keep > 0 {
                Ok(())
            } else {
                Err(GoToCraftingBenchSkipReason::MinResinToKeepDisabled)
            }
        }
        GoToCraftingBenchStepCondition::WhenResinCountRecognitionFailed => {
            if state.resin_count_recognition_failed {
                Ok(())
            } else {
                Err(GoToCraftingBenchSkipReason::ResinCountRecognitionSucceeded)
            }
        }
        GoToCraftingBenchStepCondition::WhenCraftsNeeded => match state.crafts_needed {
            Some(count) if count > 0 => Ok(()),
            Some(_) => Err(GoToCraftingBenchSkipReason::CraftsNotNeeded),
            None => Err(GoToCraftingBenchSkipReason::ResinCountsMissing),
        },
        GoToCraftingBenchStepCondition::WhenMinResinToKeepDisabled => {
            if state.condensed_resin_visible != Some(true) {
                Err(GoToCraftingBenchSkipReason::CondensedResinMissing)
            } else if plan.min_resin_to_keep <= 0 {
                Ok(())
            } else {
                Err(GoToCraftingBenchSkipReason::MinResinToKeepEnabled)
            }
        }
        GoToCraftingBenchStepCondition::WhenCrafted => {
            if state.crafted {
                Ok(())
            } else {
                Err(GoToCraftingBenchSkipReason::CraftedMissing)
            }
        }
    }
}

fn should_execute_count_inventory_item_step(
    step: &CountInventoryItemStep,
    plan: &CountInventoryItemExecutionPlan,
    state: &CountInventoryItemExecutorState,
) -> std::result::Result<(), CountInventoryItemSkipReason> {
    if state.result.is_some() && step.phase != CountInventoryItemStepPhase::Cleanup {
        return Err(CountInventoryItemSkipReason::ResultAlreadySet);
    }

    match step.condition {
        CountInventoryItemStepCondition::Always => Ok(()),
        CountInventoryItemStepCondition::WhenExpiredItemPromptDetected => {
            match state.open_inventory_outcome {
                Some(outcome) if outcome.expired_item_prompt_detected => Ok(()),
                Some(_) => Err(CountInventoryItemSkipReason::ExpiredItemPromptMissing),
                None => Err(CountInventoryItemSkipReason::OpenInventoryStateUnknown),
            }
        }
        CountInventoryItemStepCondition::WhenInventoryTabUnchecked => {
            match state.open_inventory_outcome {
                Some(outcome) if !outcome.inventory_tab_checked => Ok(()),
                Some(_) => Err(CountInventoryItemSkipReason::InventoryTabAlreadyChecked),
                None => Err(CountInventoryItemSkipReason::InventoryTabStateUnknown),
            }
        }
        CountInventoryItemStepCondition::WhenStillOnMainUi => match state.open_inventory_outcome {
            Some(outcome) if outcome.still_on_main_ui => Ok(()),
            Some(_) => Err(CountInventoryItemSkipReason::NotStillOnMainUi),
            None => Err(CountInventoryItemSkipReason::OpenInventoryStateUnknown),
        },
        CountInventoryItemStepCondition::WhenWeaponOreRequested => {
            if plan.weapon_ore_prescroll_rule.enabled {
                Ok(())
            } else {
                Err(CountInventoryItemSkipReason::WeaponOreNotRequested)
            }
        }
        CountInventoryItemStepCondition::WhenClassifierMatchesTarget => {
            if state.target_matches.is_empty() {
                Err(CountInventoryItemSkipReason::ClassifierTargetMissing)
            } else {
                Ok(())
            }
        }
        CountInventoryItemStepCondition::WhenAllRequestedItemsFound => {
            if state.all_requested_items_found {
                Ok(())
            } else {
                Err(CountInventoryItemSkipReason::AllRequestedItemsNotFound)
            }
        }
        CountInventoryItemStepCondition::WhenScanComplete => {
            if state.scan_complete {
                Ok(())
            } else {
                Err(CountInventoryItemSkipReason::ScanIncomplete)
            }
        }
    }
}

fn should_execute_scan_pick_drops_step(
    step: &ScanPickDropsStep,
    plan: &ScanPickDropsExecutionPlan,
    state: &ScanPickDropsExecutorState,
) -> std::result::Result<(), ScanPickDropsSkipReason> {
    if state.result.is_some() && step.condition != ScanPickDropsStepCondition::Finally {
        return Err(ScanPickDropsSkipReason::ResultAlreadySet);
    }

    match step.condition {
        ScanPickDropsStepCondition::Always | ScanPickDropsStepCondition::Finally => Ok(()),
        ScanPickDropsStepCondition::WhileBeforeTimeout => {
            if plan.scan_seconds > 0 {
                Ok(())
            } else {
                Err(ScanPickDropsSkipReason::TimeoutReached)
            }
        }
        ScanPickDropsStepCondition::WhenItemsDetected => {
            if state.detected_targets.is_empty() {
                Err(ScanPickDropsSkipReason::NoItemsDetected)
            } else {
                Ok(())
            }
        }
        ScanPickDropsStepCondition::WhenNoItemsDetected => {
            if !state.detection_attempted {
                Err(ScanPickDropsSkipReason::DetectionMissing)
            } else if state.detected_targets.is_empty() {
                Ok(())
            } else {
                Err(ScanPickDropsSkipReason::ItemsDetected)
            }
        }
    }
}

fn should_execute_one_key_expedition_step(
    step: &OneKeyExpeditionStep,
    plan: &OneKeyExpeditionExecutionPlan,
    state: &OneKeyExpeditionExecutorState,
) -> std::result::Result<(), OneKeyExpeditionSkipReason> {
    if step.condition == OneKeyExpeditionStepCondition::Finally {
        return Ok(());
    }
    if state.result.is_some() {
        return Err(OneKeyExpeditionSkipReason::ResultAlreadySet);
    }

    match step.condition {
        OneKeyExpeditionStepCondition::Always => Ok(()),
        OneKeyExpeditionStepCondition::ForCollectAttempt => {
            if state.collect_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectAlreadyDetected)
            } else {
                Ok(())
            }
        }
        OneKeyExpeditionStepCondition::WhenCollectMissingAndCanRetry => {
            if state.collect_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectAlreadyDetected)
            } else if state.collect_detected == Some(false)
                && state.collect_attempts == step.attempt.unwrap_or_default()
                && state.collect_attempts < plan.collect_attempts
            {
                Ok(())
            } else {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            }
        }
        OneKeyExpeditionStepCondition::WhenCollectMissingAfterRetries => {
            if state.collect_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectAlreadyDetected)
            } else if state.collect_detected == Some(false)
                && state.collect_attempts >= plan.collect_attempts
            {
                Ok(())
            } else if state.collect_detected == Some(false) {
                Err(OneKeyExpeditionSkipReason::CollectMissingButCanRetry)
            } else {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            }
        }
        OneKeyExpeditionStepCondition::WhenCollectDetected => {
            if state.collect_detected == Some(true) {
                Ok(())
            } else {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            }
        }
        OneKeyExpeditionStepCondition::ForReDispatchAttempt => {
            if state.collect_detected != Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            } else if state.re_dispatch_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::ReDispatchAlreadyDetected)
            } else {
                Ok(())
            }
        }
        OneKeyExpeditionStepCondition::WhenReDispatchMissingAndCanRetry => {
            if state.collect_detected != Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            } else if state.re_dispatch_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::ReDispatchAlreadyDetected)
            } else if state.re_dispatch_detected == Some(false)
                && state.re_dispatch_attempts == step.attempt.unwrap_or_default()
                && state.re_dispatch_attempts < plan.re_dispatch_retry_attempts
            {
                Ok(())
            } else {
                Err(OneKeyExpeditionSkipReason::ReDispatchMissing)
            }
        }
        OneKeyExpeditionStepCondition::WhenReDispatchMissingAfterRetries => {
            if state.collect_detected != Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            } else if state.re_dispatch_detected == Some(true) {
                Err(OneKeyExpeditionSkipReason::ReDispatchAlreadyDetected)
            } else if state.re_dispatch_detected == Some(false)
                && state.re_dispatch_attempts >= plan.re_dispatch_retry_attempts
            {
                Ok(())
            } else if state.re_dispatch_detected == Some(false) {
                Err(OneKeyExpeditionSkipReason::ReDispatchMissingButCanRetry)
            } else {
                Err(OneKeyExpeditionSkipReason::ReDispatchMissing)
            }
        }
        OneKeyExpeditionStepCondition::WhenReDispatchDetected => {
            if state.collect_detected != Some(true) {
                Err(OneKeyExpeditionSkipReason::CollectMissing)
            } else if state.re_dispatch_detected == Some(true) {
                Ok(())
            } else {
                Err(OneKeyExpeditionSkipReason::ReDispatchMissing)
            }
        }
        OneKeyExpeditionStepCondition::Finally => Ok(()),
    }
}

fn should_execute_go_to_adventurers_guild_step(
    step: &GoToAdventurersGuildStep,
    plan: &GoToAdventurersGuildExecutionPlan,
    state: &GoToAdventurersGuildExecutorState,
) -> std::result::Result<(), GoToAdventurersGuildSkipReason> {
    if state.result.is_some() {
        return Err(GoToAdventurersGuildSkipReason::ResultAlreadySet);
    }

    match step.condition {
        GoToAdventurersGuildStepCondition::Always => Ok(()),
        GoToAdventurersGuildStepCondition::WhenDailyRewardPartyConfigured => {
            if plan
                .daily_reward_party_name
                .as_deref()
                .is_some_and(|party_name| !party_name.trim().is_empty())
            {
                Ok(())
            } else {
                Err(GoToAdventurersGuildSkipReason::DailyRewardPartyMissing)
            }
        }
        GoToAdventurersGuildStepCondition::WhenOnlyDoOnceFalse => {
            if plan.only_do_once {
                Err(GoToAdventurersGuildSkipReason::OnlyDoOnceEnabled)
            } else {
                Ok(())
            }
        }
        GoToAdventurersGuildStepCondition::WhenDailyRewardOptionFound => {
            match state.daily_reward_option_result {
                Some(TalkOptionPlanResult::FoundAndClick) => Ok(()),
                Some(TalkOptionPlanResult::FoundButNotOrange) => {
                    Err(GoToAdventurersGuildSkipReason::DailyRewardOptionNotOrange)
                }
                Some(TalkOptionPlanResult::NotFound) => {
                    Err(GoToAdventurersGuildSkipReason::DailyRewardOptionMissing)
                }
                None => Err(GoToAdventurersGuildSkipReason::DailyRewardOptionResultMissing),
            }
        }
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished => {
            if state.daily_reward_dialogue_finished == Some(true) {
                Ok(())
            } else {
                Err(GoToAdventurersGuildSkipReason::DailyRewardDialogueMissing)
            }
        }
        GoToAdventurersGuildStepCondition::WhenExpeditionOptionFound => {
            match state.expedition_option_result {
                Some(TalkOptionPlanResult::FoundAndClick) => Ok(()),
                Some(TalkOptionPlanResult::FoundButNotOrange) => {
                    Err(GoToAdventurersGuildSkipReason::ExpeditionOptionNotOrange)
                }
                Some(TalkOptionPlanResult::NotFound) => {
                    Err(GoToAdventurersGuildSkipReason::ExpeditionOptionMissing)
                }
                None => Err(GoToAdventurersGuildSkipReason::ExpeditionOptionResultMissing),
            }
        }
        GoToAdventurersGuildStepCondition::WhenTalkUiStillOpen => {
            if state.talk_ui_still_open == Some(false) {
                Err(GoToAdventurersGuildSkipReason::TalkUiClosed)
            } else {
                Ok(())
            }
        }
    }
}

fn should_execute_go_to_serenitea_pot_step(
    step: &GoToSereniteaPotStep,
    plan: &GoToSereniteaPotExecutionPlan,
    state: &GoToSereniteaPotExecutorState,
) -> std::result::Result<(), GoToSereniteaPotSkipReason> {
    if state.result.is_some() {
        return Err(GoToSereniteaPotSkipReason::ResultAlreadySet);
    }

    match step.condition {
        GoToSereniteaPotStepCondition::Always | GoToSereniteaPotStepCondition::Finally => Ok(()),
        GoToSereniteaPotStepCondition::WhenMapTeleportConfigured => {
            if plan.entry_mode == GoToSereniteaPotEntryMode::MapTeleport {
                Ok(())
            } else {
                Err(GoToSereniteaPotSkipReason::MapTeleportNotConfigured)
            }
        }
        GoToSereniteaPotStepCondition::WhenBagGadgetConfigured => {
            if plan.entry_mode == GoToSereniteaPotEntryMode::BagGadget {
                Ok(())
            } else {
                Err(GoToSereniteaPotSkipReason::BagGadgetNotConfigured)
            }
        }
        GoToSereniteaPotStepCondition::WhenEntryFailed => match state.entry_succeeded {
            Some(false) => Ok(()),
            Some(true) => Err(GoToSereniteaPotSkipReason::EntrySucceeded),
            None => Err(GoToSereniteaPotSkipReason::EntryResultMissing),
        },
        GoToSereniteaPotStepCondition::WhenEntrySucceeded => match state.entry_succeeded {
            Some(true) => Ok(()),
            Some(false) => Err(GoToSereniteaPotSkipReason::EntryFailed),
            None => Err(GoToSereniteaPotSkipReason::EntryResultMissing),
        },
        GoToSereniteaPotStepCondition::WhenAYuanFound => match state.ayuan_found {
            Some(true) => Ok(()),
            Some(false) => Err(GoToSereniteaPotSkipReason::AYuanMissing),
            None => Err(GoToSereniteaPotSkipReason::AYuanResultMissing),
        },
        GoToSereniteaPotStepCondition::WhenAYuanMissing => match state.ayuan_found {
            Some(false) => Ok(()),
            Some(true) => Err(GoToSereniteaPotSkipReason::AYuanFound),
            None => Err(GoToSereniteaPotSkipReason::AYuanResultMissing),
        },
        GoToSereniteaPotStepCondition::WhenTrustRewardAvailable => {
            Err(GoToSereniteaPotSkipReason::RewardAvailabilityUnknown)
        }
        GoToSereniteaPotStepCondition::WhenShopConfiguredAndDue => {
            if state.ayuan_found != Some(true) {
                Err(GoToSereniteaPotSkipReason::AYuanMissing)
            } else if plan.secret_treasure_objects.is_empty() {
                Err(GoToSereniteaPotSkipReason::ShopNotConfigured)
            } else {
                Ok(())
            }
        }
        GoToSereniteaPotStepCondition::WhenShopMissingOrNotDue => {
            if state.ayuan_found != Some(true) {
                Err(GoToSereniteaPotSkipReason::AYuanMissing)
            } else if plan.secret_treasure_objects.is_empty()
                || state.shop_purchase_completed == Some(false)
            {
                Ok(())
            } else if state.shop_purchase_completed == Some(true) {
                Err(GoToSereniteaPotSkipReason::ShopPurchased)
            } else {
                Err(GoToSereniteaPotSkipReason::ShopMissingOrNotDue)
            }
        }
    }
}

fn should_execute_set_time_step(
    step: &CommonJobStep,
    plan: &SetTimeExecutionPlan,
    state: &SetTimeExecutorState,
) -> std::result::Result<(), CommonJobSkipReason> {
    match step.condition {
        CommonJobStepCondition::Always => Ok(()),
        CommonJobStepCondition::WhenSkipAnimationRequested => {
            if plan.skip_time_adjustment_animation {
                Ok(())
            } else {
                Err(CommonJobSkipReason::SkipAnimationNotRequested)
            }
        }
        CommonJobStepCondition::WhenSkipAnimationNotResolved => {
            if !plan.skip_time_adjustment_animation {
                Err(CommonJobSkipReason::SkipAnimationNotRequested)
            } else if state.skip_animation_resolved {
                Err(CommonJobSkipReason::SkipAnimationAlreadyResolved)
            } else {
                Ok(())
            }
        }
        CommonJobStepCondition::AfterTimeAdjustment => {
            if plan.skip_time_adjustment_animation {
                Err(CommonJobSkipReason::SkipAnimationRequested)
            } else {
                Ok(())
            }
        }
        _ => Err(CommonJobSkipReason::ConditionNotSupported),
    }
}

fn execute_set_time_step<R>(
    step: &CommonJobStep,
    plan: &SetTimeExecutionPlan,
    runtime: &mut R,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: CommonJobRuntime,
{
    match &step.action {
        CommonJobStepAction::CommonJob { task_key, .. } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        CommonJobStepAction::CommonJob { task_key, .. } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        CommonJobStepAction::Input { events } => {
            Ok((runtime.dispatch_capture_input(events)?, None))
        }
        _ => Ok((execute_common_job_step(step, runtime)?, None)),
    }
}

fn apply_set_time_outcome(
    step: &CommonJobStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut SetTimeExecutorState,
) -> Result<()> {
    match &step.action {
        CommonJobStepAction::CommonJob { task_key, .. } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let completed = outcome.as_match(step)?;
            match step.phase {
                CommonJobStepPhase::Setup => {
                    state.initial_return_main_ui_completed = Some(completed);
                }
                CommonJobStepPhase::Animation => {
                    state.skip_animation_return_main_ui_completed = Some(completed);
                    state.skip_animation_resolved = completed;
                }
                CommonJobStepPhase::Cleanup => {
                    state.final_return_main_ui_completed = Some(completed);
                }
                _ => {}
            }
        }
        CommonJobStepAction::Locator { locator }
            if step.phase == CommonJobStepPhase::Cleanup
                && locator.recognition_object.name.as_deref()
                    == Some(SET_TIME_PAGE_CLOSE_WHITE) =>
        {
            state.page_close_detected = outcome.as_match(step)?;
        }
        _ => {}
    }
    Ok(())
}

fn execute_walk_to_f_step<R>(
    step: &WalkToFStep,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match &step.action {
        WalkToFStepAction::GenshinAction { action, press } => {
            let action_type = match press {
                WalkToFActionPress::KeyDown => KeyActionType::KeyDown,
                WalkToFActionPress::KeyUp => KeyActionType::KeyUp,
            };
            let events = input_events_for_action(key_bindings, *action, action_type)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            runtime.dispatch_input(&events)
        }
        WalkToFStepAction::Input { events } => runtime.dispatch_input(events),
        WalkToFStepAction::Page { command } => runtime.execute_page_command(command),
        WalkToFStepAction::Locator { locator } => runtime.execute_locator(locator),
        WalkToFStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        WalkToFStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_lower_head_then_walk_to_step<R>(
    step: &LowerHeadThenWalkToStep,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut LowerHeadThenWalkToExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: LowerHeadThenWalkToRuntime,
{
    match &step.action {
        LowerHeadThenWalkToStepAction::Locator { locator } => runtime.execute_locator(locator),
        LowerHeadThenWalkToStepAction::TrackingLoop {
            target_locator,
            movement_rule,
            f_key_rule,
        } => {
            let result = runtime.execute_lower_head_tracking_loop(
                target_locator,
                movement_rule,
                f_key_rule,
            )?;
            state.tracking_loop_completed = true;
            match result {
                LowerHeadThenWalkToStepResult::Activated => {
                    state.activation_text_detected = true;
                    Ok(CommonJobRuntimeOutcome::Matched(true))
                }
                LowerHeadThenWalkToStepResult::InitialTargetMissing => {
                    state.initial_target_detected = Some(false);
                    Ok(CommonJobRuntimeOutcome::Matched(false))
                }
                LowerHeadThenWalkToStepResult::Timeout => {
                    state.timed_out = true;
                    Ok(CommonJobRuntimeOutcome::Matched(false))
                }
            }
        }
        LowerHeadThenWalkToStepAction::GenshinAction { action, press } => {
            let action_type = match press {
                LowerHeadThenWalkToActionPress::KeyDown => KeyActionType::KeyDown,
                LowerHeadThenWalkToActionPress::KeyUp => KeyActionType::KeyUp,
            };
            let events = input_events_for_action(key_bindings, *action, action_type)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            runtime.dispatch_input(&events)
        }
        LowerHeadThenWalkToStepAction::ClearVisionDrawings => runtime.clear_vision_drawings(),
        LowerHeadThenWalkToStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        LowerHeadThenWalkToStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_switch_party_step<R>(
    step: &SwitchPartyStep,
    plan: &SwitchPartyExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut SwitchPartyExecutorState,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: SwitchPartyRuntime,
{
    match &step.action {
        SwitchPartyStepAction::CommonJob { task_key } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        SwitchPartyStepAction::CommonJob { task_key } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        SwitchPartyStepAction::GenshinAction { action } => {
            let events = input_events_for_action(key_bindings, *action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            Ok((runtime.dispatch_input(&events)?, None))
        }
        SwitchPartyStepAction::Input { events } => {
            Ok((runtime.dispatch_capture_input(events)?, None))
        }
        SwitchPartyStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        SwitchPartyStepAction::Locator { locator } => Ok((runtime.execute_locator(locator)?, None)),
        SwitchPartyStepAction::Ocr { command } => {
            let candidates = runtime.recognize_switch_party_text(command)?;
            let matched = !candidates.is_empty();
            match step.phase {
                SwitchPartyStepPhase::CurrentPartyCheck => {
                    state.current_party_raw_text =
                        candidates.first().map(|candidate| candidate.text.clone());
                }
                SwitchPartyStepPhase::PartyList => {
                    state.party_list_texts = candidates;
                }
                _ => {}
            }
            Ok((CommonJobRuntimeOutcome::Matched(matched), None))
        }
        SwitchPartyStepAction::NormalizeCurrentPartyName { rule } => {
            let raw = state.current_party_raw_text.clone().unwrap_or_default();
            let normalized = normalize_switch_party_current_name(&raw, rule);
            let matched = !normalized.is_empty();
            state.current_party_normalized_text = Some(normalized);
            Ok((CommonJobRuntimeOutcome::Matched(matched), None))
        }
        SwitchPartyStepAction::MatchCurrentParty { party_name } => {
            let text = state.current_party_normalized_text.as_deref().unwrap_or("");
            let matched = switch_party_text_matches(
                text,
                party_name,
                plan.current_party_rule.match_as_regex,
            )?;
            state.current_party_matched = Some(matched);
            Ok((CommonJobRuntimeOutcome::Matched(matched), None))
        }
        SwitchPartyStepAction::OpenPartyChooseMenu {
            rule,
            choose_locator,
            delete_locator,
        } => Ok((
            runtime.open_switch_party_choose_menu(rule, choose_locator, delete_locator)?,
            None,
        )),
        SwitchPartyStepAction::ScanPartyList { rule, party_name } => {
            let outcome =
                runtime.scan_switch_party_list(rule, party_name, &state.party_list_texts)?;
            state.list_scan_completed = true;
            state.scanned_pages = outcome.scanned_pages;
            state.matched_party = outcome.matched_party;
            state.party_not_found = state.matched_party.is_none() && outcome.reached_end;
            Ok((
                CommonJobRuntimeOutcome::Matched(state.matched_party.is_some()),
                None,
            ))
        }
        SwitchPartyStepAction::ConfirmParty { rule } => {
            Ok((runtime.confirm_switch_party(rule)?, None))
        }
        SwitchPartyStepAction::ClearCombatScenes => {
            Ok((runtime.clear_switch_party_combat_scenes()?, None))
        }
        SwitchPartyStepAction::ReturnResult { .. } => Ok((CommonJobRuntimeOutcome::None, None)),
        SwitchPartyStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn execute_count_inventory_item_step<R>(
    step: &CountInventoryItemStep,
    plan: &CountInventoryItemExecutionPlan,
    runtime: &mut R,
    state: &mut CountInventoryItemExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CountInventoryItemRuntime,
{
    match &step.action {
        CountInventoryItemStepAction::CommonJob { task_key } => {
            runtime.execute_count_inventory_common_job(task_key)
        }
        CountInventoryItemStepAction::GenshinAction { action }
            if *action == GenshinAction::OpenInventory =>
        {
            let outcome = runtime.open_count_inventory(&plan.open_inventory_rule)?;
            if step.condition == CountInventoryItemStepCondition::WhenStillOnMainUi {
                state.retry_open_inventory_outcome = Some(outcome);
            } else {
                state.open_inventory_outcome = Some(outcome);
            }
            Ok(CommonJobRuntimeOutcome::Matched(!outcome.still_on_main_ui))
        }
        CountInventoryItemStepAction::GenshinAction { action } => {
            Err(TaskError::CommonJobExecution(format!(
                "CountInventoryItem unsupported GenshinAction in executor: {action:?}"
            )))
        }
        CountInventoryItemStepAction::OpenInventoryTab { rule } => {
            runtime.open_count_inventory_tab(rule)
        }
        CountInventoryItemStepAction::ConfirmExpiredItemPrompt {
            confirm_asset,
            crop_bottom_ratio,
        } => runtime.confirm_count_inventory_expired_item_prompt(confirm_asset, *crop_bottom_ratio),
        CountInventoryItemStepAction::LoadGridIconClassifier { rule } => {
            runtime.load_count_inventory_grid_icon_classifier(rule)
        }
        CountInventoryItemStepAction::PreScrollWeaponOre { rule } => {
            runtime.pre_scroll_count_inventory_weapon_ore(rule)
        }
        CountInventoryItemStepAction::EnumerateGridItems {
            template,
            detection_rule,
            scroll_rule,
        } => {
            state.grid_items = runtime.enumerate_count_inventory_grid_items(
                template,
                detection_rule,
                scroll_rule,
            )?;
            state.scan_complete = true;
            Ok(CommonJobRuntimeOutcome::Matched(
                !state.grid_items.is_empty(),
            ))
        }
        CountInventoryItemStepAction::CropGridIcon { rule } => {
            runtime.crop_count_inventory_grid_icons(&state.grid_items, rule)
        }
        CountInventoryItemStepAction::InferGridIcon { rule } => {
            state.inferred_icons =
                runtime.infer_count_inventory_grid_icons(&state.grid_items, rule)?;
            state.target_matches =
                filter_count_inventory_target_matches(&state.inferred_icons, &plan.search_mode);
            Ok(CommonJobRuntimeOutcome::Matched(
                !state.target_matches.is_empty(),
            ))
        }
        CountInventoryItemStepAction::OcrGridItemCount { rule } => {
            let counts = runtime.ocr_count_inventory_item_counts(&state.target_matches, rule)?;
            merge_count_inventory_counts(&mut state.item_counts, counts);
            state.all_requested_items_found =
                count_inventory_all_requested_items_found(&plan.search_mode, &state.item_counts);
            Ok(CommonJobRuntimeOutcome::Matched(
                !state.item_counts.is_empty(),
            ))
        }
        CountInventoryItemStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        CountInventoryItemStepAction::ClearVisionDrawings => {
            runtime.clear_count_inventory_vision_drawings()
        }
        CountInventoryItemStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_scan_pick_drops_step<R>(
    step: &ScanPickDropsStep,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut ScanPickDropsExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: ScanPickDropsRuntime,
{
    match &step.action {
        ScanPickDropsStepAction::Log { message } => runtime.log(message),
        ScanPickDropsStepAction::CameraReset { rule } => {
            execute_scan_pick_camera_reset(rule, runtime)
        }
        ScanPickDropsStepAction::YoloDetect { rule } => {
            state.detection_attempted = true;
            state.detected_targets = runtime.detect_scan_pick_targets(rule)?;
            Ok(CommonJobRuntimeOutcome::Matched(
                !state.detected_targets.is_empty(),
            ))
        }
        ScanPickDropsStepAction::SearchSweep { rule, yolo_rule } => {
            execute_scan_pick_search_sweep(rule, yolo_rule, key_bindings, runtime, state)
        }
        ScanPickDropsStepAction::SelectTarget { rule } => {
            let target = select_scan_pick_target(&state.detected_targets, rule);
            state.selected_target = target;
            Ok(CommonJobRuntimeOutcome::Matched(target.is_some()))
        }
        ScanPickDropsStepAction::ApproachTarget { rule } => {
            let target = state.selected_target.ok_or_else(|| {
                TaskError::CommonJobExecution(
                    "ScanPickDrops has no selected target to approach".to_string(),
                )
            })?;
            let commands = scan_pick_movement_commands(&target, rule);
            for command in &commands {
                dispatch_scan_pick_action(key_bindings, runtime, command.action, command.press)?;
            }
            state.movement_commands = commands;
            Ok(CommonJobRuntimeOutcome::Matched(true))
        }
        ScanPickDropsStepAction::GenshinAction { action, press } => {
            dispatch_scan_pick_action(key_bindings, runtime, *action, *press)
        }
        ScanPickDropsStepAction::Page { command } => runtime.execute_page_command(command),
        ScanPickDropsStepAction::ReleaseAllKeys => {
            runtime.dispatch_input(release_all_keys_sequence().events())
        }
        ScanPickDropsStepAction::ClearVisionDrawings => runtime.clear_scan_pick_vision_drawings(),
        ScanPickDropsStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
    }
}

fn execute_one_key_expedition_step<R>(
    step: &OneKeyExpeditionStep,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: OneKeyExpeditionRuntime,
{
    match &step.action {
        OneKeyExpeditionStepAction::ActivateWindow => runtime.activate_one_key_expedition_window(),
        OneKeyExpeditionStepAction::Locator { locator } => runtime.execute_locator(locator),
        OneKeyExpeditionStepAction::Page { command } => runtime.execute_page_command(command),
        OneKeyExpeditionStepAction::Input { events } => runtime.dispatch_input(events),
        OneKeyExpeditionStepAction::Log { message } => runtime.log(message),
        OneKeyExpeditionStepAction::ClearVisionDrawings => {
            runtime.clear_one_key_expedition_vision_drawings()
        }
        OneKeyExpeditionStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
    }
}

fn execute_go_to_crafting_bench_step<R>(
    step: &GoToCraftingBenchStep,
    plan: &GoToCraftingBenchExecutionPlan,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut GoToCraftingBenchExecutorState,
) -> Result<(CommonJobRuntimeOutcome, Option<ReturnMainUiExecutionReport>)>
where
    R: GoToCraftingBenchRuntime,
{
    match &step.action {
        GoToCraftingBenchStepAction::Pathing { rule } => {
            Ok((runtime.execute_crafting_bench_pathing(rule)?, None))
        }
        GoToCraftingBenchStepAction::InteractionRetry { rule } => {
            Ok((runtime.retry_crafting_bench_interaction(rule)?, None))
        }
        GoToCraftingBenchStepAction::GenshinAction { action, press } => {
            let action_type = match press {
                GoToCraftingBenchActionPress::KeyDown => KeyActionType::KeyDown,
                GoToCraftingBenchActionPress::KeyUp => KeyActionType::KeyUp,
            };
            let events = input_events_for_action(key_bindings, *action, action_type)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            Ok((runtime.dispatch_input(&events)?, None))
        }
        GoToCraftingBenchStepAction::SelectLastTalkOptionUntilEnd { until_locator } => Ok((
            runtime.select_last_crafting_bench_talk_option_until_end(until_locator)?,
            None,
        )),
        GoToCraftingBenchStepAction::DetectResin { locator } => {
            Ok((runtime.execute_locator(locator)?, None))
        }
        GoToCraftingBenchStepAction::RecognizeResinCounts { rule } => {
            let counts = runtime.recognize_crafting_bench_resin_counts(rule)?;
            state.resin_counts = counts;
            state.resin_count_recognition_failed = state.resin_counts.is_none();
            Ok((
                CommonJobRuntimeOutcome::Matched(!state.resin_count_recognition_failed),
                None,
            ))
        }
        GoToCraftingBenchStepAction::ComputeCraftsNeeded { rule } => {
            let counts = state.resin_counts.ok_or_else(|| {
                TaskError::CommonJobExecution(
                    "GoToCraftingBench has no resin counts to compute craft count".to_string(),
                )
            })?;
            let max_crafts_possible =
                (rule.max_condensed_resin_count as i32 - counts.condensed_resin_count).max(0);
            let resin_available = counts.fragile_resin_count - rule.min_resin_to_keep;
            let crafts_needed = (resin_available.max(0) / rule.resin_consumed_per_craft as i32)
                .min(max_crafts_possible)
                .max(0) as u8;
            state.crafts_needed = Some(crafts_needed);
            Ok((CommonJobRuntimeOutcome::Matched(crafts_needed > 0), None))
        }
        GoToCraftingBenchStepAction::CraftCondensedResin { rule } => {
            let crafts_needed = if rule.min_resin_to_keep <= 0 {
                1
            } else {
                state.crafts_needed.unwrap_or(0)
            };
            Ok((runtime.craft_condensed_resin(rule, crafts_needed)?, None))
        }
        GoToCraftingBenchStepAction::CommonJob { task_key, .. }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let nested_plan = crate::plan_return_main_ui(
                plan.capture_size,
                RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
            )?;
            let report = execute_return_main_ui_plan(&nested_plan, runtime)?;
            Ok((
                CommonJobRuntimeOutcome::Matched(report.completed),
                Some(report),
            ))
        }
        GoToCraftingBenchStepAction::CommonJob { task_key, .. } => {
            Err(TaskError::CommonJobExecution(format!(
                "nested common job execution is not supported yet: {task_key}"
            )))
        }
        GoToCraftingBenchStepAction::Page { command } => {
            Ok((runtime.execute_page_command(command)?, None))
        }
        GoToCraftingBenchStepAction::Locator { locator } => {
            Ok((runtime.execute_locator(locator)?, None))
        }
        GoToCraftingBenchStepAction::Input { events } => {
            Ok((runtime.dispatch_input(events)?, None))
        }
        GoToCraftingBenchStepAction::ReturnResult { .. } => {
            Ok((CommonJobRuntimeOutcome::None, None))
        }
        GoToCraftingBenchStepAction::Log { message } => Ok((runtime.log(message)?, None)),
    }
}

fn execute_go_to_adventurers_guild_step<R>(
    step: &GoToAdventurersGuildStep,
    runtime: &mut R,
    state: &mut GoToAdventurersGuildExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: GoToAdventurersGuildRuntime,
{
    match &step.action {
        GoToAdventurersGuildStepAction::CommonJob { task_key, config } => {
            let outcome =
                runtime.execute_adventurers_guild_common_job(task_key, config.as_ref())?;
            match task_key.as_str() {
                SWITCH_PARTY_TASK_KEY
                | CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY
                | RETURN_MAIN_UI_TASK_KEY => match outcome {
                    GoToAdventurersGuildNestedOutcome::Completed(completed) => {
                        Ok(CommonJobRuntimeOutcome::Matched(completed))
                    }
                    GoToAdventurersGuildNestedOutcome::TalkOption(result) => {
                        Err(TaskError::CommonJobExecution(format!(
                            "GoToAdventurersGuild nested {task_key} returned talk option result {result:?}"
                        )))
                    }
                },
                CHOOSE_TALK_OPTION_TASK_KEY => match outcome {
                    GoToAdventurersGuildNestedOutcome::TalkOption(result) => {
                        match step.phase {
                            GoToAdventurersGuildStepPhase::DailyReward => {
                                state.daily_reward_option_result = Some(result);
                            }
                            GoToAdventurersGuildStepPhase::Expedition => {
                                state.expedition_option_result = Some(result);
                            }
                            _ => {}
                        }
                        Ok(CommonJobRuntimeOutcome::Matched(
                            result == TalkOptionPlanResult::FoundAndClick,
                        ))
                    }
                    GoToAdventurersGuildNestedOutcome::Completed(completed) => {
                        Err(TaskError::CommonJobExecution(format!(
                            "GoToAdventurersGuild nested ChooseTalkOption returned completion {completed}"
                        )))
                    }
                },
                _ => Err(TaskError::CommonJobExecution(format!(
                    "nested common job execution is not supported yet: {task_key}"
                ))),
            }
        }
        GoToAdventurersGuildStepAction::Pathing { rule } => {
            runtime.execute_adventurers_guild_pathing(rule)
        }
        GoToAdventurersGuildStepAction::InteractionRetry { rule } => {
            runtime.retry_adventurers_guild_interaction(rule)
        }
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd {
            max_times,
            until_paimon_menu,
        } if step.condition == GoToAdventurersGuildStepCondition::WhenTalkUiStillOpen => {
            let talk_ui_open = runtime.is_adventurers_guild_talk_ui_open()?;
            state.talk_ui_still_open = Some(talk_ui_open);
            if talk_ui_open {
                runtime.select_last_adventurers_guild_talk_option_until_end(
                    *max_times,
                    *until_paimon_menu,
                )
            } else {
                Ok(CommonJobRuntimeOutcome::Matched(false))
            }
        }
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd {
            max_times,
            until_paimon_menu,
        } => runtime
            .select_last_adventurers_guild_talk_option_until_end(*max_times, *until_paimon_menu),
        GoToAdventurersGuildStepAction::OneKeyExpedition { rule } => {
            runtime.run_one_key_adventurers_guild_expedition(&rule.one_key_plan)
        }
        GoToAdventurersGuildStepAction::Page { command } => runtime.execute_page_command(command),
        GoToAdventurersGuildStepAction::Locator { locator } => runtime.execute_locator(locator),
        GoToAdventurersGuildStepAction::Input { events } => runtime.dispatch_input(events),
        GoToAdventurersGuildStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        GoToAdventurersGuildStepAction::Log { message } => runtime.log(message),
    }
}

fn execute_go_to_serenitea_pot_step<R>(
    step: &GoToSereniteaPotStep,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut GoToSereniteaPotExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: GoToSereniteaPotRuntime,
{
    match &step.action {
        GoToSereniteaPotStepAction::Log { message } => runtime.log(message),
        GoToSereniteaPotStepAction::CommonJob { task_key, .. } => {
            Err(TaskError::CommonJobExecution(format!(
                "nested common job execution is not supported yet: {task_key}"
            )))
        }
        GoToSereniteaPotStepAction::GenshinAction { action, press } => {
            let action_type = match press {
                GoToSereniteaPotActionPress::KeyDown => KeyActionType::KeyDown,
                GoToSereniteaPotActionPress::KeyUp => KeyActionType::KeyUp,
                GoToSereniteaPotActionPress::KeyPress => KeyActionType::KeyPress,
            };
            let events = input_events_for_action(key_bindings, *action, action_type)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            runtime.dispatch_input(&events)
        }
        GoToSereniteaPotStepAction::Locator { locator } => runtime.execute_locator(locator),
        GoToSereniteaPotStepAction::Page { command } => runtime.execute_page_command(command),
        GoToSereniteaPotStepAction::MapEntry { rule } => {
            let outcome = runtime.enter_serenitea_pot_by_map(rule)?;
            state.entry_realm_name = outcome.realm_name;
            Ok(CommonJobRuntimeOutcome::Matched(outcome.entered))
        }
        GoToSereniteaPotStepAction::BagEntry { rule } => {
            let outcome = runtime.enter_serenitea_pot_by_bag(rule)?;
            state.entry_realm_name = outcome.realm_name;
            Ok(CommonJobRuntimeOutcome::Matched(outcome.entered))
        }
        GoToSereniteaPotStepAction::FindAYuan { rule } => {
            let found = runtime
                .find_and_approach_serenitea_pot_ayuan(rule, state.entry_realm_name.as_deref())?;
            Ok(CommonJobRuntimeOutcome::Matched(found))
        }
        GoToSereniteaPotStepAction::Reward { rule } => runtime.claim_serenitea_pot_rewards(rule),
        GoToSereniteaPotStepAction::ShopPurchase { rule } => {
            runtime.purchase_serenitea_pot_shop(rule, &rule.configured_objects)
        }
        GoToSereniteaPotStepAction::Finish { rule } => runtime.finish_serenitea_pot(rule),
        GoToSereniteaPotStepAction::ReleaseAllKeys => runtime.release_serenitea_pot_keys(),
        GoToSereniteaPotStepAction::ClearVisionDrawings => {
            runtime.clear_serenitea_pot_vision_drawings()
        }
        GoToSereniteaPotStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
    }
}

fn execute_teleport_step<R>(step: &TeleportStep, runtime: &mut R) -> Result<CommonJobRuntimeOutcome>
where
    R: TeleportRuntime,
{
    match &step.action {
        TeleportStepAction::Log { message } => runtime.log(message),
        TeleportStepAction::ReturnResult { .. } => Ok(CommonJobRuntimeOutcome::None),
        action => runtime.execute_teleport_action(action),
    }
}

fn apply_walk_to_f_outcome(
    step: &WalkToFStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut WalkToFExecutorState,
) -> Result<()> {
    match &step.action {
        WalkToFStepAction::GenshinAction { action, press } => {
            let held = matches!(press, WalkToFActionPress::KeyDown);
            match action {
                GenshinAction::MoveForward => state.move_forward_held = held,
                GenshinAction::SprintKeyboard => state.sprint_held = held,
                _ => {}
            }
        }
        WalkToFStepAction::Locator { .. } if step.phase == WalkToFStepPhase::Search => {
            state.pick_detected = walk_to_f_outcome_as_match(outcome, step)?;
        }
        WalkToFStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_lower_head_then_walk_to_outcome(
    step: &LowerHeadThenWalkToStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut LowerHeadThenWalkToExecutorState,
) -> Result<()> {
    match &step.action {
        LowerHeadThenWalkToStepAction::Locator { .. } => {
            state.initial_target_detected =
                Some(lower_head_then_walk_to_outcome_as_match(outcome, step)?);
        }
        LowerHeadThenWalkToStepAction::GenshinAction { action, press } => {
            if *action == GenshinAction::MoveForward {
                state.move_forward_held = matches!(press, LowerHeadThenWalkToActionPress::KeyDown);
            }
        }
        LowerHeadThenWalkToStepAction::ClearVisionDrawings => {
            state.vision_drawings_cleared = true;
        }
        LowerHeadThenWalkToStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_teleport_outcome(
    step: &TeleportStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut TeleportExecutorState,
) -> Result<()> {
    match &step.action {
        TeleportStepAction::OpenBigMapUi => {
            state.big_map_open_requested = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::VerifyBigMapUi => {
            state.big_map_verified = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ResolveCoordinateTarget { .. } => {
            state.coordinate_target_resolved = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ResolveNearestTeleportPoint { .. } => {
            state.nearest_teleport_point_resolved = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::SwitchCountryOrMap { .. } => {
            state.country_or_map_switched = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::NormalizeUndergroundMap => {
            state.underground_map_normalized = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ReadBigMapZoomLevel => {
            state.zoom_level_read = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::AdjustMapZoomLevel => {
            state.zoom_level_adjusted = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::RecognizeBigMapCenter => {
            state.big_map_center_recognized = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::RecognizeBigMapRect => {
            state.big_map_rect_recognized = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::DragBigMapToTarget { .. } => {
            state.big_map_dragged = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::VerifyTargetPointInBigMapWindow { .. } => {
            state.target_point_verified = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ConvertMapCoordinateToScreenPoint { .. } => {
            state.screen_point_converted = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ClickMapTeleportPoint => {
            state.map_teleport_point_clicked = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::ClickTeleportPanelOrCandidate { .. } => {
            state.teleport_panel_clicked = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::HandlePointNotActivated { .. } => {
            state.point_not_activated_handled = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::MoveMapTo { .. } => {
            state.move_map_completed = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::SelectStatueOfTheSeven => {
            state.statue_selected = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::WaitForTeleportCompletion { .. } => {
            state.teleport_completion_waited = teleport_outcome_as_match(outcome, step)?;
        }
        TeleportStepAction::SeedNavigationPreviousPositionAfterTeleport { target } => {
            state.navigation_previous_position_seeded = teleport_outcome_as_match(outcome, step)?;
            state.navigation_previous_position_seed = state
                .navigation_previous_position_seeded
                .then(|| target.clone())
                .flatten();
        }
        TeleportStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        TeleportStepAction::Log { .. } => {}
    }
    Ok(())
}

fn apply_switch_party_outcome(
    step: &SwitchPartyStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut SwitchPartyExecutorState,
) -> Result<()> {
    match &step.action {
        SwitchPartyStepAction::CommonJob { task_key } if task_key == RETURN_MAIN_UI_TASK_KEY => {
            let completed = switch_party_outcome_as_match(outcome, step)?;
            match step.phase {
                SwitchPartyStepPhase::Setup => {
                    state.initial_return_main_ui_completed = Some(completed);
                }
                SwitchPartyStepPhase::Cleanup | SwitchPartyStepPhase::CurrentPartyCheck => {
                    state.final_return_main_ui_completed = Some(completed);
                }
                _ => {}
            }
        }
        SwitchPartyStepAction::GenshinAction {
            action: GenshinAction::OpenPartySetupScreen,
        } => {
            state.party_view_open_dispatched = true;
        }
        SwitchPartyStepAction::Input { .. } if step.phase == SwitchPartyStepPhase::PartyList => {
            state.top_reset_dispatched = true;
        }
        SwitchPartyStepAction::Locator { .. }
            if step.phase == SwitchPartyStepPhase::OpenPartyView =>
        {
            let opened = switch_party_outcome_as_match(outcome, step)?;
            state.party_view_opened = Some(opened);
            if !opened {
                return Err(TaskError::CommonJobExecution(
                    "SwitchParty failed to open party setup screen".to_string(),
                ));
            }
        }
        SwitchPartyStepAction::OpenPartyChooseMenu { .. } => {
            let opened = switch_party_outcome_as_match(outcome, step)?;
            state.choose_menu_opened = Some(opened);
            if !opened {
                return Err(TaskError::CommonJobExecution(
                    "SwitchParty failed to open party choose menu".to_string(),
                ));
            }
        }
        SwitchPartyStepAction::ConfirmParty { .. } => {
            let confirmed = switch_party_outcome_as_match(outcome, step)?;
            state.party_confirmed = Some(confirmed);
            if !confirmed {
                return Err(TaskError::CommonJobExecution(
                    "SwitchParty confirm timed out".to_string(),
                ));
            }
        }
        SwitchPartyStepAction::ClearCombatScenes => {
            state.combat_scenes_cleared = true;
        }
        SwitchPartyStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_count_inventory_item_outcome(
    step: &CountInventoryItemStep,
    plan: &CountInventoryItemExecutionPlan,
    outcome: CommonJobRuntimeOutcome,
    state: &mut CountInventoryItemExecutorState,
) -> Result<()> {
    match &step.action {
        CountInventoryItemStepAction::CommonJob { task_key }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            let completed = count_inventory_item_outcome_as_match(outcome, step)?;
            if step.phase == CountInventoryItemStepPhase::Setup {
                state.initial_return_main_ui_completed = Some(completed);
            } else if step.phase == CountInventoryItemStepPhase::Cleanup {
                state.final_return_main_ui_completed = Some(completed);
            }
        }
        CountInventoryItemStepAction::ConfirmExpiredItemPrompt { .. } => {
            state.expired_item_prompt_confirmed =
                Some(count_inventory_item_outcome_as_match(outcome, step)?);
        }
        CountInventoryItemStepAction::OpenInventoryTab { .. } => {
            state.inventory_tab_opened =
                Some(count_inventory_item_outcome_as_match(outcome, step)?);
        }
        CountInventoryItemStepAction::LoadGridIconClassifier { .. } => {
            state.classifier_loaded = count_inventory_item_outcome_as_match(outcome, step)?;
        }
        CountInventoryItemStepAction::PreScrollWeaponOre { .. } => {
            state.weapon_ore_prescrolled = count_inventory_item_outcome_as_match(outcome, step)?;
        }
        CountInventoryItemStepAction::CropGridIcon { .. } => {
            state.grid_icons_cropped = count_inventory_item_outcome_as_match(outcome, step)?;
        }
        CountInventoryItemStepAction::ReturnResult { contract } => {
            state.result = Some(build_count_inventory_item_result(
                &plan.search_mode,
                &state.item_counts,
                contract,
            ));
        }
        CountInventoryItemStepAction::ClearVisionDrawings => {
            state.vision_drawings_cleared = count_inventory_item_outcome_as_match(outcome, step)?;
        }
        _ => {}
    }
    Ok(())
}

fn apply_scan_pick_drops_outcome(
    step: &ScanPickDropsStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ScanPickDropsExecutorState,
) -> Result<()> {
    match &step.action {
        ScanPickDropsStepAction::CameraReset { .. } => {
            state.camera_reset_completed = scan_pick_drops_outcome_as_match(outcome, step)?;
        }
        ScanPickDropsStepAction::GenshinAction { action, .. }
            if *action == GenshinAction::Drop && step.phase == ScanPickDropsStepPhase::Setup =>
        {
            state.initial_drop_dispatched = true;
        }
        ScanPickDropsStepAction::GenshinAction { action, .. }
            if *action == GenshinAction::Drop && step.phase == ScanPickDropsStepPhase::Cleanup =>
        {
            state.cleanup_drop_dispatched = true;
        }
        ScanPickDropsStepAction::SearchSweep { .. } => {
            state.search_sweep_completed = true;
        }
        ScanPickDropsStepAction::ApproachTarget { .. } => {
            state.approach_completed = scan_pick_drops_outcome_as_match(outcome, step)?;
        }
        ScanPickDropsStepAction::ReleaseAllKeys => {
            state.release_all_keys_completed = true;
        }
        ScanPickDropsStepAction::ClearVisionDrawings => {
            state.vision_drawings_cleared = scan_pick_drops_outcome_as_match(outcome, step)?;
        }
        ScanPickDropsStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_one_key_expedition_outcome(
    step: &OneKeyExpeditionStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut OneKeyExpeditionExecutorState,
) -> Result<()> {
    match &step.action {
        OneKeyExpeditionStepAction::ActivateWindow => {
            state.window_activated = true;
        }
        OneKeyExpeditionStepAction::Locator { .. }
            if step.phase == OneKeyExpeditionStepPhase::Collect =>
        {
            state.collect_attempts = step.attempt.unwrap_or(state.collect_attempts);
            let matched = one_key_expedition_outcome_as_match(outcome, step)?;
            state.collect_detected = Some(matched);
            state.collect_clicked = matched;
        }
        OneKeyExpeditionStepAction::Locator { .. }
            if step.phase == OneKeyExpeditionStepPhase::ReDispatch =>
        {
            state.re_dispatch_attempts = step.attempt.unwrap_or(state.re_dispatch_attempts);
            let matched = one_key_expedition_outcome_as_match(outcome, step)?;
            state.re_dispatch_detected = Some(matched);
            state.re_dispatch_clicked = matched;
        }
        OneKeyExpeditionStepAction::Input { .. }
            if step.phase == OneKeyExpeditionStepPhase::Exit =>
        {
            state.exit_dispatched = true;
        }
        OneKeyExpeditionStepAction::ClearVisionDrawings => {
            state.vision_drawings_cleared = match outcome {
                CommonJobRuntimeOutcome::Matched(value) => value,
                CommonJobRuntimeOutcome::None => true,
            };
        }
        OneKeyExpeditionStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_go_to_crafting_bench_outcome(
    step: &GoToCraftingBenchStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut GoToCraftingBenchExecutorState,
) -> Result<()> {
    match &step.action {
        GoToCraftingBenchStepAction::Pathing { rule } => {
            let completed = go_to_crafting_bench_outcome_as_match(outcome, step)?;
            state.pathing_completed = Some(completed);
            if !completed && rule.fail_when_task_missing {
                return Err(TaskError::CommonJobExecution(format!(
                    "GoToCraftingBench pathing failed for {}",
                    rule.pathing_json
                )));
            }
        }
        GoToCraftingBenchStepAction::InteractionRetry { rule } => {
            let succeeded = go_to_crafting_bench_outcome_as_match(outcome, step)?;
            state.interaction_retry_succeeded = Some(succeeded);
            state.talk_ui_detected = Some(succeeded);
            if !succeeded
                && step.condition == GoToCraftingBenchStepCondition::WhenTalkUiStillMissing
            {
                return Err(TaskError::CommonJobExecution(rule.fail_message.clone()));
            }
        }
        GoToCraftingBenchStepAction::GenshinAction { action, press } => {
            if *action == GenshinAction::MoveBackward {
                state.move_backward_held = matches!(press, GoToCraftingBenchActionPress::KeyDown);
            }
        }
        GoToCraftingBenchStepAction::SelectLastTalkOptionUntilEnd { .. } => {
            let opened = go_to_crafting_bench_outcome_as_match(outcome, step)?;
            state.crafting_page_opened = Some(opened);
            if !opened {
                return Err(TaskError::CommonJobExecution(
                    "GoToCraftingBench failed to open crafting page".to_string(),
                ));
            }
        }
        GoToCraftingBenchStepAction::DetectResin { .. } => {
            state.condensed_resin_visible =
                Some(go_to_crafting_bench_outcome_as_match(outcome, step)?);
        }
        GoToCraftingBenchStepAction::CraftCondensedResin { .. } => {
            state.crafted = go_to_crafting_bench_outcome_as_match(outcome, step)?;
        }
        GoToCraftingBenchStepAction::CommonJob { task_key, .. }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            state.final_return_main_ui_completed =
                Some(go_to_crafting_bench_outcome_as_match(outcome, step)?);
        }
        GoToCraftingBenchStepAction::ReturnResult { result } => {
            if state.resin_count_recognition_failed {
                return Err(TaskError::CommonJobExecution(
                    "GoToCraftingBench resin-count OCR failed".to_string(),
                ));
            }
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_go_to_adventurers_guild_outcome(
    step: &GoToAdventurersGuildStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut GoToAdventurersGuildExecutorState,
) -> Result<()> {
    match &step.action {
        GoToAdventurersGuildStepAction::CommonJob { task_key, .. }
            if task_key == SWITCH_PARTY_TASK_KEY =>
        {
            state.party_switch_completed =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::CommonJob { task_key, .. }
            if task_key == CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY =>
        {
            state.encounter_points_claimed =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::CommonJob { task_key, .. }
            if task_key == RETURN_MAIN_UI_TASK_KEY =>
        {
            state.return_main_ui_after_daily_completed =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::Pathing { rule } => {
            let completed = go_to_adventurers_guild_outcome_as_match(outcome, step)?;
            state.pathing_completed = Some(completed);
            if !completed && rule.fail_when_task_missing {
                return Err(TaskError::CommonJobExecution(format!(
                    "GoToAdventurersGuild pathing failed for {}",
                    rule.pathing_json
                )));
            }
        }
        GoToAdventurersGuildStepAction::InteractionRetry { rule } => {
            let succeeded = go_to_adventurers_guild_outcome_as_match(outcome, step)?;
            state.interaction_retry_succeeded = Some(succeeded);
            if step.phase == GoToAdventurersGuildStepPhase::DailyReward
                && step.condition
                    == GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished
            {
                state.catherine_reopened_after_daily = Some(succeeded);
            }
            if !succeeded && rule.fail_when_talk_ui_missing_after_retries {
                return Err(TaskError::CommonJobExecution(format!(
                    "GoToAdventurersGuild failed to open talk UI for {} after {} retries",
                    rule.interact_text, rule.retry_talk_times
                )));
            }
        }
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd { .. }
            if step.phase == GoToAdventurersGuildStepPhase::DailyReward =>
        {
            state.daily_reward_dialogue_finished =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd { .. }
            if step.phase == GoToAdventurersGuildStepPhase::Cleanup =>
        {
            let closed = go_to_adventurers_guild_outcome_as_match(outcome, step)?;
            state.cleanup_dialogue_closed = Some(closed);
            if state.talk_ui_still_open == Some(true) {
                state.talk_ui_still_open = Some(!closed);
            }
        }
        GoToAdventurersGuildStepAction::OneKeyExpedition { .. } => {
            state.expedition_completed =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::Locator { .. }
            if step.phase == GoToAdventurersGuildStepPhase::DailyReward
                && step.condition
                    == GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished =>
        {
            state.paimon_menu_detected_after_daily =
                Some(go_to_adventurers_guild_outcome_as_match(outcome, step)?);
        }
        GoToAdventurersGuildStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn apply_go_to_serenitea_pot_outcome(
    step: &GoToSereniteaPotStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut GoToSereniteaPotExecutorState,
) -> Result<()> {
    match &step.action {
        GoToSereniteaPotStepAction::MapEntry { .. }
        | GoToSereniteaPotStepAction::BagEntry { .. } => {
            state.entry_succeeded = Some(go_to_serenitea_pot_outcome_as_match(outcome, step)?);
        }
        GoToSereniteaPotStepAction::FindAYuan { .. } => {
            state.ayuan_found = Some(go_to_serenitea_pot_outcome_as_match(outcome, step)?);
        }
        GoToSereniteaPotStepAction::Reward { .. } => {
            state.rewards_claimed = Some(go_to_serenitea_pot_outcome_as_match(outcome, step)?);
        }
        GoToSereniteaPotStepAction::ShopPurchase { .. } => {
            state.shop_purchase_completed =
                Some(go_to_serenitea_pot_outcome_as_match(outcome, step)?);
        }
        GoToSereniteaPotStepAction::Finish { .. } => {
            let completed = go_to_serenitea_pot_outcome_as_match(outcome, step)?;
            match step.condition {
                GoToSereniteaPotStepCondition::WhenEntryFailed => {
                    state.entry_failure_finish_completed = Some(completed);
                }
                GoToSereniteaPotStepCondition::WhenAYuanMissing => {
                    state.ayuan_missing_finish_completed = Some(completed);
                }
                GoToSereniteaPotStepCondition::Always => {
                    state.final_finish_completed = Some(completed);
                }
                _ => {}
            }
        }
        GoToSereniteaPotStepAction::ReleaseAllKeys => {
            state.keys_released = go_to_serenitea_pot_outcome_as_match(outcome, step)?;
        }
        GoToSereniteaPotStepAction::ClearVisionDrawings => {
            state.vision_drawings_cleared = go_to_serenitea_pot_outcome_as_match(outcome, step)?;
        }
        GoToSereniteaPotStepAction::ReturnResult { result } => {
            state.result = Some(*result);
        }
        _ => {}
    }
    Ok(())
}

fn walk_to_f_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &WalkToFStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "walk-to-f step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn teleport_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &TeleportStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "teleport step {:?}/{} did not return a match result",
            step.phase, step.label
        ))),
    }
}

fn go_to_crafting_bench_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &GoToCraftingBenchStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "go-to-crafting-bench step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn count_inventory_item_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &CountInventoryItemStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "count-inventory-item step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn scan_pick_drops_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &ScanPickDropsStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "scan-pick-drops step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn execute_scan_pick_camera_reset<R>(
    rule: &ScanPickCameraResetRule,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    runtime.dispatch_input(&rule.middle_click_events)?;
    runtime.execute_page_command(&BvPageCommand::Wait {
        milliseconds: rule.wait_after_middle_click_ms,
    })?;
    runtime.dispatch_input(&[InputEvent::MouseMoveRelative {
        dx: rule.look_down_mouse_dx,
        dy: rule.look_down_mouse_dy,
    }])?;
    runtime.execute_page_command(&BvPageCommand::Wait {
        milliseconds: rule.wait_after_look_down_ms,
    })?;
    Ok(CommonJobRuntimeOutcome::Matched(true))
}

fn execute_scan_pick_search_sweep<R>(
    rule: &ScanPickSearchRule,
    yolo_rule: &ScanPickYoloRule,
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    state: &mut ScanPickDropsExecutorState,
) -> Result<CommonJobRuntimeOutcome>
where
    R: ScanPickDropsRuntime,
{
    runtime.dispatch_input(release_all_keys_sequence().events())?;
    for index in 0..rule.iterations {
        state.search_iterations_run = index.saturating_add(1);
        runtime.dispatch_input(&[InputEvent::MouseMoveRelative {
            dx: rule.mouse_move_dx,
            dy: rule.mouse_move_dy,
        }])?;
        if index > rule.walk_forward_after_index {
            dispatch_scan_pick_action(
                key_bindings,
                runtime,
                rule.walk_action,
                ScanPickDropsActionPress::KeyDown,
            )?;
            runtime.execute_page_command(&BvPageCommand::Wait {
                milliseconds: rule.walk_forward_ms,
            })?;
            dispatch_scan_pick_action(
                key_bindings,
                runtime,
                rule.walk_action,
                ScanPickDropsActionPress::KeyUp,
            )?;
        }
        dispatch_scan_pick_action(
            key_bindings,
            runtime,
            rule.drop_action,
            ScanPickDropsActionPress::KeyPress,
        )?;
        runtime.execute_page_command(&BvPageCommand::Wait {
            milliseconds: rule.wait_after_drop_ms,
        })?;
        state.detected_targets = runtime.detect_scan_pick_targets(yolo_rule)?;
        if !state.detected_targets.is_empty() {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
    }
    Ok(CommonJobRuntimeOutcome::Matched(false))
}

fn select_scan_pick_target(targets: &[Rect], rule: &ScanPickTargetOrderingRule) -> Option<Rect> {
    targets.iter().copied().min_by(|left, right| {
        scan_pick_target_score(left, rule)
            .partial_cmp(&scan_pick_target_score(right, rule))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn scan_pick_target_score(target: &Rect, rule: &ScanPickTargetOrderingRule) -> f64 {
    let x = target.x as f64;
    let bottom = (target.y + target.height) as f64;
    (x - rule.center_x_1080p).powi(2)
        + rule.vertical_weight * (bottom - rule.reference_bottom_y_1080p).powi(2)
}

fn scan_pick_movement_commands(
    target: &Rect,
    rule: &ScanPickMovementRule,
) -> Vec<ScanPickDropsMovementCommand> {
    let mut commands = Vec::new();
    let x = target.x as f64;
    let bottom = (target.y + target.height) as f64;

    if bottom > rule.horizontal_bottom_min_1080p {
        if x < rule.move_left_when_x_below_1080p {
            commands.push(ScanPickDropsMovementCommand {
                action: rule.right_action,
                press: ScanPickDropsActionPress::KeyUp,
            });
            commands.push(ScanPickDropsMovementCommand {
                action: rule.left_action,
                press: ScanPickDropsActionPress::KeyDown,
            });
        } else if x > rule.move_right_when_x_above_1080p {
            commands.push(ScanPickDropsMovementCommand {
                action: rule.left_action,
                press: ScanPickDropsActionPress::KeyUp,
            });
            commands.push(ScanPickDropsMovementCommand {
                action: rule.right_action,
                press: ScanPickDropsActionPress::KeyDown,
            });
        } else {
            commands.push(ScanPickDropsMovementCommand {
                action: rule.left_action,
                press: ScanPickDropsActionPress::KeyUp,
            });
            commands.push(ScanPickDropsMovementCommand {
                action: rule.right_action,
                press: ScanPickDropsActionPress::KeyUp,
            });
        }
    }

    if bottom < rule.move_forward_when_bottom_below_1080p {
        commands.push(ScanPickDropsMovementCommand {
            action: rule.backward_action,
            press: ScanPickDropsActionPress::KeyUp,
        });
        commands.push(ScanPickDropsMovementCommand {
            action: rule.forward_action,
            press: ScanPickDropsActionPress::KeyDown,
        });
    } else if bottom > rule.move_backward_when_bottom_above_1080p {
        commands.push(ScanPickDropsMovementCommand {
            action: rule.forward_action,
            press: ScanPickDropsActionPress::KeyUp,
        });
        commands.push(ScanPickDropsMovementCommand {
            action: rule.backward_action,
            press: ScanPickDropsActionPress::KeyDown,
        });
    } else {
        commands.push(ScanPickDropsMovementCommand {
            action: rule.forward_action,
            press: ScanPickDropsActionPress::KeyUp,
        });
        commands.push(ScanPickDropsMovementCommand {
            action: rule.backward_action,
            press: ScanPickDropsActionPress::KeyUp,
        });
    }

    commands
}

fn dispatch_scan_pick_action<R>(
    key_bindings: &KeyBindingsConfig,
    runtime: &mut R,
    action: GenshinAction,
    press: ScanPickDropsActionPress,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    let action_type = match press {
        ScanPickDropsActionPress::KeyDown => KeyActionType::KeyDown,
        ScanPickDropsActionPress::KeyUp => KeyActionType::KeyUp,
        ScanPickDropsActionPress::KeyPress => KeyActionType::KeyPress,
    };
    let events = input_events_for_action(key_bindings, action, action_type)
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
    runtime.dispatch_input(&events)
}

fn filter_count_inventory_target_matches(
    inferred_icons: &[CountInventoryGridIconMatch],
    search_mode: &CountInventorySearchMode,
) -> Vec<CountInventoryGridIconMatch> {
    let requested = search_mode.item_names();
    inferred_icons
        .iter()
        .filter(|item| requested.iter().any(|name| name == &item.item_name))
        .cloned()
        .collect()
}

fn merge_count_inventory_counts(
    existing: &mut Vec<CountInventoryItemCount>,
    new_counts: Vec<CountInventoryItemCount>,
) {
    for count in new_counts {
        if existing
            .iter()
            .any(|existing_count| existing_count.item_name == count.item_name)
        {
            continue;
        }
        existing.push(count);
    }
}

fn count_inventory_all_requested_items_found(
    search_mode: &CountInventorySearchMode,
    counts: &[CountInventoryItemCount],
) -> bool {
    search_mode
        .item_names()
        .iter()
        .all(|requested| counts.iter().any(|count| &count.item_name == requested))
}

fn build_count_inventory_item_result(
    search_mode: &CountInventorySearchMode,
    counts: &[CountInventoryItemCount],
    contract: &CountInventoryResultContract,
) -> CountInventoryItemExecutionResult {
    match search_mode {
        CountInventorySearchMode::Single { item_name } => {
            let count = counts
                .iter()
                .find(|count| &count.item_name == item_name)
                .map(|count| count.count)
                .unwrap_or(contract.single_not_found_value);
            CountInventoryItemExecutionResult::Single { count }
        }
        CountInventorySearchMode::Multiple { item_names } => {
            let mut result_counts = Vec::new();
            for item_name in item_names {
                if result_counts
                    .iter()
                    .any(|count: &CountInventoryItemCount| &count.item_name == item_name)
                {
                    continue;
                }
                if let Some(count) = counts.iter().find(|count| &count.item_name == item_name) {
                    result_counts.push(count.clone());
                }
            }
            CountInventoryItemExecutionResult::Multiple {
                counts: result_counts,
            }
        }
    }
}

fn go_to_adventurers_guild_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &GoToAdventurersGuildStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "go-to-adventurers-guild step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn one_key_expedition_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &OneKeyExpeditionStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "one-key-expedition step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn go_to_serenitea_pot_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &GoToSereniteaPotStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "go-to-serenitea-pot step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn switch_party_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &SwitchPartyStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "switch-party step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn normalize_switch_party_current_name(raw: &str, rule: &SwitchPartyCurrentPartyRule) -> String {
    let mut text = raw.to_string();
    if rule.strip_double_quotes {
        text = text.replace('"', "");
    }
    if rule.remove_crlf {
        text = text.replace("\r\n", "").replace('\r', "");
    }
    if rule.truncate_at_first_lf {
        if let Some((prefix, _)) = text.split_once('\n') {
            text = prefix.to_string();
        }
    }
    if rule.trim {
        text = text.trim().to_string();
    }
    text
}

fn switch_party_text_matches(text: &str, pattern: &str, match_as_regex: bool) -> Result<bool> {
    switch_party_text_matches_pattern(text, pattern, match_as_regex)
}

fn lower_head_then_walk_to_outcome_as_match(
    outcome: CommonJobRuntimeOutcome,
    step: &LowerHeadThenWalkToStep,
) -> Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "lower-head-then-walk-to step {:?}/{:?}/{} did not return a match result",
            step.phase, step.condition, step.label
        ))),
    }
}

fn should_execute_return_main_ui_step(
    step: &CommonJobStep,
    state: &ReturnMainUiExecutorState,
) -> std::result::Result<(), CommonJobSkipReason> {
    match step.condition {
        CommonJobStepCondition::Always => Ok(()),
        CommonJobStepCondition::WhenMainUiNotDetected => {
            if state.main_ui_detected {
                Err(if step.phase == CommonJobStepPhase::RetryLoop {
                    CommonJobSkipReason::MainUiDetectedAfterRetry
                } else {
                    CommonJobSkipReason::MainUiAlreadyDetected
                })
            } else {
                Ok(())
            }
        }
        CommonJobStepCondition::WhenExitDoorDetected => {
            if state.exit_door_detected {
                Ok(())
            } else {
                Err(CommonJobSkipReason::ExitDoorNotDetected)
            }
        }
        CommonJobStepCondition::AfterRetryLimit => {
            if state.main_ui_detected {
                Err(CommonJobSkipReason::RetryLimitNotReached)
            } else {
                Ok(())
            }
        }
        _ => Err(CommonJobSkipReason::ConditionNotSupported),
    }
}

fn execute_common_job_step<R>(
    step: &CommonJobStep,
    runtime: &mut R,
) -> Result<CommonJobRuntimeOutcome>
where
    R: CommonJobRuntime,
{
    match &step.action {
        CommonJobStepAction::CommonJob { task_key, .. } => Err(TaskError::CommonJobExecution(
            format!("nested common job execution is not supported yet: {task_key}"),
        )),
        CommonJobStepAction::Input { events } => runtime.dispatch_input(events),
        CommonJobStepAction::Page { command } => runtime.execute_page_command(command),
        CommonJobStepAction::Locator { locator } => runtime.execute_locator(locator),
        CommonJobStepAction::Log { message } => runtime.log(message),
    }
}

fn apply_return_main_ui_outcome(
    step: &CommonJobStep,
    outcome: CommonJobRuntimeOutcome,
    state: &mut ReturnMainUiExecutorState,
) -> Result<()> {
    if step.phase == CommonJobStepPhase::RetryLoop
        && step.condition == CommonJobStepCondition::WhenMainUiNotDetected
        && matches!(step.action, CommonJobStepAction::Input { .. })
    {
        state.last_escape_attempt = step.attempt;
    }
    if step.condition == CommonJobStepCondition::AfterRetryLimit {
        state.fallback_used = true;
    }

    let CommonJobStepAction::Locator { locator } = &step.action else {
        return Ok(());
    };
    if locator.operation != BvLocatorOperation::IsExist {
        return Ok(());
    }

    match locator.recognition_object.name.as_deref() {
        Some(RETURN_MAIN_UI_PAIMON_MENU) => state.main_ui_detected = outcome.as_match(step)?,
        Some(RETURN_MAIN_UI_EXIT_DOOR) => state.exit_door_detected = outcome.as_match(step)?,
        _ => {}
    }

    Ok(())
}

fn action_kind(action: &CommonJobStepAction) -> CommonJobRuntimeActionKind {
    match action {
        CommonJobStepAction::CommonJob { .. } => CommonJobRuntimeActionKind::CommonJob,
        CommonJobStepAction::Input { .. } => CommonJobRuntimeActionKind::Input,
        CommonJobStepAction::Page { .. } => CommonJobRuntimeActionKind::Page,
        CommonJobStepAction::Locator { .. } => CommonJobRuntimeActionKind::Locator,
        CommonJobStepAction::Log { .. } => CommonJobRuntimeActionKind::Log,
    }
}

fn walk_to_f_action_kind(action: &WalkToFStepAction) -> WalkToFRuntimeActionKind {
    match action {
        WalkToFStepAction::GenshinAction { .. } => WalkToFRuntimeActionKind::GenshinAction,
        WalkToFStepAction::Input { .. } => WalkToFRuntimeActionKind::Input,
        WalkToFStepAction::Page { .. } => WalkToFRuntimeActionKind::Page,
        WalkToFStepAction::Locator { .. } => WalkToFRuntimeActionKind::Locator,
        WalkToFStepAction::ReturnResult { .. } => WalkToFRuntimeActionKind::ReturnResult,
        WalkToFStepAction::Log { .. } => WalkToFRuntimeActionKind::Log,
    }
}

fn lower_head_then_walk_to_action_kind(
    action: &LowerHeadThenWalkToStepAction,
) -> LowerHeadThenWalkToRuntimeActionKind {
    match action {
        LowerHeadThenWalkToStepAction::Locator { .. } => {
            LowerHeadThenWalkToRuntimeActionKind::Locator
        }
        LowerHeadThenWalkToStepAction::TrackingLoop { .. } => {
            LowerHeadThenWalkToRuntimeActionKind::TrackingLoop
        }
        LowerHeadThenWalkToStepAction::GenshinAction { .. } => {
            LowerHeadThenWalkToRuntimeActionKind::GenshinAction
        }
        LowerHeadThenWalkToStepAction::ClearVisionDrawings => {
            LowerHeadThenWalkToRuntimeActionKind::ClearVisionDrawings
        }
        LowerHeadThenWalkToStepAction::ReturnResult { .. } => {
            LowerHeadThenWalkToRuntimeActionKind::ReturnResult
        }
        LowerHeadThenWalkToStepAction::Log { .. } => LowerHeadThenWalkToRuntimeActionKind::Log,
    }
}

fn switch_party_action_kind(action: &SwitchPartyStepAction) -> SwitchPartyRuntimeActionKind {
    match action {
        SwitchPartyStepAction::CommonJob { .. } => SwitchPartyRuntimeActionKind::CommonJob,
        SwitchPartyStepAction::GenshinAction { .. } => SwitchPartyRuntimeActionKind::GenshinAction,
        SwitchPartyStepAction::Input { .. } => SwitchPartyRuntimeActionKind::Input,
        SwitchPartyStepAction::Page { .. } => SwitchPartyRuntimeActionKind::Page,
        SwitchPartyStepAction::Locator { .. } => SwitchPartyRuntimeActionKind::Locator,
        SwitchPartyStepAction::Ocr { .. } => SwitchPartyRuntimeActionKind::Ocr,
        SwitchPartyStepAction::NormalizeCurrentPartyName { .. } => {
            SwitchPartyRuntimeActionKind::NormalizeCurrentPartyName
        }
        SwitchPartyStepAction::MatchCurrentParty { .. } => {
            SwitchPartyRuntimeActionKind::MatchCurrentParty
        }
        SwitchPartyStepAction::OpenPartyChooseMenu { .. } => {
            SwitchPartyRuntimeActionKind::OpenPartyChooseMenu
        }
        SwitchPartyStepAction::ScanPartyList { .. } => SwitchPartyRuntimeActionKind::ScanPartyList,
        SwitchPartyStepAction::ConfirmParty { .. } => SwitchPartyRuntimeActionKind::ConfirmParty,
        SwitchPartyStepAction::ClearCombatScenes => SwitchPartyRuntimeActionKind::ClearCombatScenes,
        SwitchPartyStepAction::ReturnResult { .. } => SwitchPartyRuntimeActionKind::ReturnResult,
        SwitchPartyStepAction::Log { .. } => SwitchPartyRuntimeActionKind::Log,
    }
}

fn count_inventory_item_action_kind(
    action: &CountInventoryItemStepAction,
) -> CountInventoryItemRuntimeActionKind {
    match action {
        CountInventoryItemStepAction::CommonJob { .. } => {
            CountInventoryItemRuntimeActionKind::CommonJob
        }
        CountInventoryItemStepAction::GenshinAction { action }
            if *action == GenshinAction::OpenInventory =>
        {
            CountInventoryItemRuntimeActionKind::OpenInventory
        }
        CountInventoryItemStepAction::GenshinAction { .. } => {
            CountInventoryItemRuntimeActionKind::GenshinAction
        }
        CountInventoryItemStepAction::OpenInventoryTab { .. } => {
            CountInventoryItemRuntimeActionKind::OpenInventoryTab
        }
        CountInventoryItemStepAction::ConfirmExpiredItemPrompt { .. } => {
            CountInventoryItemRuntimeActionKind::ConfirmExpiredItemPrompt
        }
        CountInventoryItemStepAction::LoadGridIconClassifier { .. } => {
            CountInventoryItemRuntimeActionKind::LoadGridIconClassifier
        }
        CountInventoryItemStepAction::PreScrollWeaponOre { .. } => {
            CountInventoryItemRuntimeActionKind::PreScrollWeaponOre
        }
        CountInventoryItemStepAction::EnumerateGridItems { .. } => {
            CountInventoryItemRuntimeActionKind::EnumerateGridItems
        }
        CountInventoryItemStepAction::CropGridIcon { .. } => {
            CountInventoryItemRuntimeActionKind::CropGridIcon
        }
        CountInventoryItemStepAction::InferGridIcon { .. } => {
            CountInventoryItemRuntimeActionKind::InferGridIcon
        }
        CountInventoryItemStepAction::OcrGridItemCount { .. } => {
            CountInventoryItemRuntimeActionKind::OcrGridItemCount
        }
        CountInventoryItemStepAction::ReturnResult { .. } => {
            CountInventoryItemRuntimeActionKind::ReturnResult
        }
        CountInventoryItemStepAction::ClearVisionDrawings => {
            CountInventoryItemRuntimeActionKind::ClearVisionDrawings
        }
        CountInventoryItemStepAction::Log { .. } => CountInventoryItemRuntimeActionKind::Log,
    }
}

fn scan_pick_drops_action_kind(action: &ScanPickDropsStepAction) -> ScanPickDropsRuntimeActionKind {
    match action {
        ScanPickDropsStepAction::CameraReset { .. } => ScanPickDropsRuntimeActionKind::CameraReset,
        ScanPickDropsStepAction::YoloDetect { .. } => ScanPickDropsRuntimeActionKind::YoloDetect,
        ScanPickDropsStepAction::SearchSweep { .. } => ScanPickDropsRuntimeActionKind::SearchSweep,
        ScanPickDropsStepAction::SelectTarget { .. } => {
            ScanPickDropsRuntimeActionKind::SelectTarget
        }
        ScanPickDropsStepAction::ApproachTarget { .. } => {
            ScanPickDropsRuntimeActionKind::ApproachTarget
        }
        ScanPickDropsStepAction::GenshinAction { .. } => {
            ScanPickDropsRuntimeActionKind::GenshinAction
        }
        ScanPickDropsStepAction::Page { .. } => ScanPickDropsRuntimeActionKind::Page,
        ScanPickDropsStepAction::ReleaseAllKeys => ScanPickDropsRuntimeActionKind::ReleaseAllKeys,
        ScanPickDropsStepAction::ClearVisionDrawings => {
            ScanPickDropsRuntimeActionKind::ClearVisionDrawings
        }
        ScanPickDropsStepAction::ReturnResult { .. } => {
            ScanPickDropsRuntimeActionKind::ReturnResult
        }
        ScanPickDropsStepAction::Log { .. } => ScanPickDropsRuntimeActionKind::Log,
    }
}

fn go_to_crafting_bench_action_kind(
    action: &GoToCraftingBenchStepAction,
) -> GoToCraftingBenchRuntimeActionKind {
    match action {
        GoToCraftingBenchStepAction::Pathing { .. } => GoToCraftingBenchRuntimeActionKind::Pathing,
        GoToCraftingBenchStepAction::InteractionRetry { .. } => {
            GoToCraftingBenchRuntimeActionKind::InteractionRetry
        }
        GoToCraftingBenchStepAction::GenshinAction { .. } => {
            GoToCraftingBenchRuntimeActionKind::GenshinAction
        }
        GoToCraftingBenchStepAction::SelectLastTalkOptionUntilEnd { .. } => {
            GoToCraftingBenchRuntimeActionKind::SelectLastTalkOptionUntilEnd
        }
        GoToCraftingBenchStepAction::DetectResin { .. } => {
            GoToCraftingBenchRuntimeActionKind::DetectResin
        }
        GoToCraftingBenchStepAction::RecognizeResinCounts { .. } => {
            GoToCraftingBenchRuntimeActionKind::RecognizeResinCounts
        }
        GoToCraftingBenchStepAction::ComputeCraftsNeeded { .. } => {
            GoToCraftingBenchRuntimeActionKind::ComputeCraftsNeeded
        }
        GoToCraftingBenchStepAction::CraftCondensedResin { .. } => {
            GoToCraftingBenchRuntimeActionKind::CraftCondensedResin
        }
        GoToCraftingBenchStepAction::CommonJob { .. } => {
            GoToCraftingBenchRuntimeActionKind::CommonJob
        }
        GoToCraftingBenchStepAction::Page { .. } => GoToCraftingBenchRuntimeActionKind::Page,
        GoToCraftingBenchStepAction::Locator { .. } => GoToCraftingBenchRuntimeActionKind::Locator,
        GoToCraftingBenchStepAction::Input { .. } => GoToCraftingBenchRuntimeActionKind::Input,
        GoToCraftingBenchStepAction::ReturnResult { .. } => {
            GoToCraftingBenchRuntimeActionKind::ReturnResult
        }
        GoToCraftingBenchStepAction::Log { .. } => GoToCraftingBenchRuntimeActionKind::Log,
    }
}

fn one_key_expedition_action_kind(
    action: &OneKeyExpeditionStepAction,
) -> OneKeyExpeditionRuntimeActionKind {
    match action {
        OneKeyExpeditionStepAction::ActivateWindow => {
            OneKeyExpeditionRuntimeActionKind::ActivateWindow
        }
        OneKeyExpeditionStepAction::Locator { .. } => OneKeyExpeditionRuntimeActionKind::Locator,
        OneKeyExpeditionStepAction::Page { .. } => OneKeyExpeditionRuntimeActionKind::Page,
        OneKeyExpeditionStepAction::Input { .. } => OneKeyExpeditionRuntimeActionKind::Input,
        OneKeyExpeditionStepAction::Log { .. } => OneKeyExpeditionRuntimeActionKind::Log,
        OneKeyExpeditionStepAction::ClearVisionDrawings => {
            OneKeyExpeditionRuntimeActionKind::ClearVisionDrawings
        }
        OneKeyExpeditionStepAction::ReturnResult { .. } => {
            OneKeyExpeditionRuntimeActionKind::ReturnResult
        }
    }
}

fn go_to_adventurers_guild_action_kind(
    action: &GoToAdventurersGuildStepAction,
) -> GoToAdventurersGuildRuntimeActionKind {
    match action {
        GoToAdventurersGuildStepAction::CommonJob { .. } => {
            GoToAdventurersGuildRuntimeActionKind::CommonJob
        }
        GoToAdventurersGuildStepAction::Pathing { .. } => {
            GoToAdventurersGuildRuntimeActionKind::Pathing
        }
        GoToAdventurersGuildStepAction::InteractionRetry { .. } => {
            GoToAdventurersGuildRuntimeActionKind::InteractionRetry
        }
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd { .. } => {
            GoToAdventurersGuildRuntimeActionKind::SelectLastTalkOptionUntilEnd
        }
        GoToAdventurersGuildStepAction::OneKeyExpedition { .. } => {
            GoToAdventurersGuildRuntimeActionKind::OneKeyExpedition
        }
        GoToAdventurersGuildStepAction::Page { .. } => GoToAdventurersGuildRuntimeActionKind::Page,
        GoToAdventurersGuildStepAction::Locator { .. } => {
            GoToAdventurersGuildRuntimeActionKind::Locator
        }
        GoToAdventurersGuildStepAction::Input { .. } => {
            GoToAdventurersGuildRuntimeActionKind::Input
        }
        GoToAdventurersGuildStepAction::ReturnResult { .. } => {
            GoToAdventurersGuildRuntimeActionKind::ReturnResult
        }
        GoToAdventurersGuildStepAction::Log { .. } => GoToAdventurersGuildRuntimeActionKind::Log,
    }
}

fn go_to_serenitea_pot_action_kind(
    action: &GoToSereniteaPotStepAction,
) -> GoToSereniteaPotRuntimeActionKind {
    match action {
        GoToSereniteaPotStepAction::CommonJob { .. } => {
            GoToSereniteaPotRuntimeActionKind::CommonJob
        }
        GoToSereniteaPotStepAction::GenshinAction { .. } => {
            GoToSereniteaPotRuntimeActionKind::GenshinAction
        }
        GoToSereniteaPotStepAction::Locator { .. } => GoToSereniteaPotRuntimeActionKind::Locator,
        GoToSereniteaPotStepAction::Page { .. } => GoToSereniteaPotRuntimeActionKind::Page,
        GoToSereniteaPotStepAction::MapEntry { .. } => GoToSereniteaPotRuntimeActionKind::MapEntry,
        GoToSereniteaPotStepAction::BagEntry { .. } => GoToSereniteaPotRuntimeActionKind::BagEntry,
        GoToSereniteaPotStepAction::FindAYuan { .. } => {
            GoToSereniteaPotRuntimeActionKind::FindAYuan
        }
        GoToSereniteaPotStepAction::Reward { .. } => GoToSereniteaPotRuntimeActionKind::Reward,
        GoToSereniteaPotStepAction::ShopPurchase { .. } => {
            GoToSereniteaPotRuntimeActionKind::ShopPurchase
        }
        GoToSereniteaPotStepAction::Finish { .. } => GoToSereniteaPotRuntimeActionKind::Finish,
        GoToSereniteaPotStepAction::ReleaseAllKeys => {
            GoToSereniteaPotRuntimeActionKind::ReleaseAllKeys
        }
        GoToSereniteaPotStepAction::ClearVisionDrawings => {
            GoToSereniteaPotRuntimeActionKind::ClearVisionDrawings
        }
        GoToSereniteaPotStepAction::ReturnResult { .. } => {
            GoToSereniteaPotRuntimeActionKind::ReturnResult
        }
        GoToSereniteaPotStepAction::Log { .. } => GoToSereniteaPotRuntimeActionKind::Log,
    }
}

fn teleport_action_kind(action: &TeleportStepAction) -> TeleportRuntimeActionKind {
    match action {
        TeleportStepAction::OpenBigMapUi => TeleportRuntimeActionKind::OpenBigMapUi,
        TeleportStepAction::VerifyBigMapUi => TeleportRuntimeActionKind::VerifyBigMapUi,
        TeleportStepAction::ResolveCoordinateTarget { .. } => {
            TeleportRuntimeActionKind::ResolveCoordinateTarget
        }
        TeleportStepAction::ResolveNearestTeleportPoint { .. } => {
            TeleportRuntimeActionKind::ResolveNearestTeleportPoint
        }
        TeleportStepAction::SwitchCountryOrMap { .. } => {
            TeleportRuntimeActionKind::SwitchCountryOrMap
        }
        TeleportStepAction::NormalizeUndergroundMap => {
            TeleportRuntimeActionKind::NormalizeUndergroundMap
        }
        TeleportStepAction::ReadBigMapZoomLevel => TeleportRuntimeActionKind::ReadBigMapZoomLevel,
        TeleportStepAction::AdjustMapZoomLevel => TeleportRuntimeActionKind::AdjustMapZoomLevel,
        TeleportStepAction::RecognizeBigMapCenter => {
            TeleportRuntimeActionKind::RecognizeBigMapCenter
        }
        TeleportStepAction::RecognizeBigMapRect => TeleportRuntimeActionKind::RecognizeBigMapRect,
        TeleportStepAction::DragBigMapToTarget { .. } => {
            TeleportRuntimeActionKind::DragBigMapToTarget
        }
        TeleportStepAction::VerifyTargetPointInBigMapWindow { .. } => {
            TeleportRuntimeActionKind::VerifyTargetPointInBigMapWindow
        }
        TeleportStepAction::ConvertMapCoordinateToScreenPoint { .. } => {
            TeleportRuntimeActionKind::ConvertMapCoordinateToScreenPoint
        }
        TeleportStepAction::ClickMapTeleportPoint => {
            TeleportRuntimeActionKind::ClickMapTeleportPoint
        }
        TeleportStepAction::ClickTeleportPanelOrCandidate { .. } => {
            TeleportRuntimeActionKind::ClickTeleportPanelOrCandidate
        }
        TeleportStepAction::MoveMapTo { .. } => TeleportRuntimeActionKind::MoveMapTo,
        TeleportStepAction::SelectStatueOfTheSeven => {
            TeleportRuntimeActionKind::SelectStatueOfTheSeven
        }
        TeleportStepAction::HandlePointNotActivated { .. } => {
            TeleportRuntimeActionKind::HandlePointNotActivated
        }
        TeleportStepAction::WaitForTeleportCompletion { .. } => {
            TeleportRuntimeActionKind::WaitForTeleportCompletion
        }
        TeleportStepAction::SeedNavigationPreviousPositionAfterTeleport { .. } => {
            TeleportRuntimeActionKind::SeedNavigationPreviousPositionAfterTeleport
        }
        TeleportStepAction::ReturnResult { .. } => TeleportRuntimeActionKind::ReturnResult,
        TeleportStepAction::Log { .. } => TeleportRuntimeActionKind::Log,
    }
}

fn check_rewards_action_kind(action: &CheckRewardsStepAction) -> CheckRewardsRuntimeActionKind {
    match action {
        CheckRewardsStepAction::CommonJob { .. } => CheckRewardsRuntimeActionKind::CommonJob,
        CheckRewardsStepAction::GenshinAction { .. } => {
            CheckRewardsRuntimeActionKind::GenshinAction
        }
        CheckRewardsStepAction::Page { .. } => CheckRewardsRuntimeActionKind::Page,
        CheckRewardsStepAction::Ocr { .. } => CheckRewardsRuntimeActionKind::Ocr,
        CheckRewardsStepAction::MatchCommissions { .. } => CheckRewardsRuntimeActionKind::MatchText,
        CheckRewardsStepAction::Locator { .. } => CheckRewardsRuntimeActionKind::Locator,
        CheckRewardsStepAction::Notify { .. } => CheckRewardsRuntimeActionKind::Notify,
        CheckRewardsStepAction::ReturnResult { .. } => CheckRewardsRuntimeActionKind::ReturnResult,
        CheckRewardsStepAction::Log { .. } => CheckRewardsRuntimeActionKind::Log,
    }
}

fn claim_battle_pass_rewards_action_kind(
    action: &BattlePassRewardStepAction,
) -> ClaimBattlePassRewardsRuntimeActionKind {
    match action {
        BattlePassRewardStepAction::CommonJob { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::CommonJob
        }
        BattlePassRewardStepAction::GenshinAction { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::GenshinAction
        }
        BattlePassRewardStepAction::Page { .. } => ClaimBattlePassRewardsRuntimeActionKind::Page,
        BattlePassRewardStepAction::Ocr { .. } => ClaimBattlePassRewardsRuntimeActionKind::Ocr,
        BattlePassRewardStepAction::MatchClaimAll { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::MatchText
        }
        BattlePassRewardStepAction::ClickMatchedText => {
            ClaimBattlePassRewardsRuntimeActionKind::ClickMatchedText
        }
        BattlePassRewardStepAction::DetectManualSelectionDialog { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::DetectManualSelectionDialog
        }
        BattlePassRewardStepAction::DismissPrimogemIfVisible { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::DismissPrimogemIfVisible
        }
        BattlePassRewardStepAction::ReturnResult { .. } => {
            ClaimBattlePassRewardsRuntimeActionKind::ReturnResult
        }
        BattlePassRewardStepAction::Log { .. } => ClaimBattlePassRewardsRuntimeActionKind::Log,
    }
}

fn claim_encounter_points_rewards_action_kind(
    action: &ClaimEncounterPointsRewardsStepAction,
) -> ClaimEncounterPointsRewardsRuntimeActionKind {
    match action {
        ClaimEncounterPointsRewardsStepAction::CommonJob { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::CommonJob
        }
        ClaimEncounterPointsRewardsStepAction::GenshinAction { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::GenshinAction
        }
        ClaimEncounterPointsRewardsStepAction::Page { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::Page
        }
        ClaimEncounterPointsRewardsStepAction::Ocr { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::Ocr
        }
        ClaimEncounterPointsRewardsStepAction::MatchCommissions { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::MatchText
        }
        ClaimEncounterPointsRewardsStepAction::Locator { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::Locator
        }
        ClaimEncounterPointsRewardsStepAction::ClickMatchedText => {
            ClaimEncounterPointsRewardsRuntimeActionKind::ClickMatchedText
        }
        ClaimEncounterPointsRewardsStepAction::ReturnResult { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::ReturnResult
        }
        ClaimEncounterPointsRewardsStepAction::Log { .. } => {
            ClaimEncounterPointsRewardsRuntimeActionKind::Log
        }
    }
}

fn claim_mail_rewards_action_kind(
    action: &ClaimMailRewardsStepAction,
) -> CommonJobRuntimeActionKind {
    match action {
        ClaimMailRewardsStepAction::CommonJob { .. } => CommonJobRuntimeActionKind::CommonJob,
        ClaimMailRewardsStepAction::GenshinAction { .. } => {
            CommonJobRuntimeActionKind::GenshinAction
        }
        ClaimMailRewardsStepAction::Input { .. } => CommonJobRuntimeActionKind::Input,
        ClaimMailRewardsStepAction::Page { .. } => CommonJobRuntimeActionKind::Page,
        ClaimMailRewardsStepAction::Locator { .. } => CommonJobRuntimeActionKind::Locator,
        ClaimMailRewardsStepAction::ReturnResult { .. } => CommonJobRuntimeActionKind::ReturnResult,
        ClaimMailRewardsStepAction::Log { .. } => CommonJobRuntimeActionKind::Log,
    }
}

fn blessing_of_the_welkin_moon_action_kind(
    action: &BlessingOfTheWelkinMoonStepAction,
) -> BlessingOfTheWelkinMoonRuntimeActionKind {
    match action {
        BlessingOfTheWelkinMoonStepAction::ServerTimeGate { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::ServerTimeGate
        }
        BlessingOfTheWelkinMoonStepAction::DetectClaimUi { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::DetectClaimUi
        }
        BlessingOfTheWelkinMoonStepAction::Input { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::Input
        }
        BlessingOfTheWelkinMoonStepAction::Page { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::Page
        }
        BlessingOfTheWelkinMoonStepAction::LoopUntilClear { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::LoopUntilClear
        }
        BlessingOfTheWelkinMoonStepAction::Log { .. } => {
            BlessingOfTheWelkinMoonRuntimeActionKind::Log
        }
    }
}

fn relogin_action_kind(action: &ReloginStepAction) -> ReloginRuntimeActionKind {
    match action {
        ReloginStepAction::FocusGameWindow => ReloginRuntimeActionKind::FocusGameWindow,
        ReloginStepAction::RetryUntilAppear { .. } => ReloginRuntimeActionKind::RetryUntilAppear,
        ReloginStepAction::RetryUntilDisappear { .. } => {
            ReloginRuntimeActionKind::RetryUntilDisappear
        }
        ReloginStepAction::ThirdPartyLoginProbe { .. } => {
            ReloginRuntimeActionKind::ThirdPartyLoginProbe
        }
        ReloginStepAction::Page { .. } => ReloginRuntimeActionKind::Page,
        ReloginStepAction::ReturnResult { .. } => ReloginRuntimeActionKind::ReturnResult,
        ReloginStepAction::Log { .. } => ReloginRuntimeActionKind::Log,
    }
}

fn wonderland_cycle_action_kind(
    action: &WonderlandCycleStepAction,
) -> WonderlandCycleRuntimeActionKind {
    match action {
        WonderlandCycleStepAction::RetryUntilAppear { .. } => {
            WonderlandCycleRuntimeActionKind::RetryUntilAppear
        }
        WonderlandCycleStepAction::RetryUntilDisappear { .. } => {
            WonderlandCycleRuntimeActionKind::RetryUntilDisappear
        }
        WonderlandCycleStepAction::Page { .. } => WonderlandCycleRuntimeActionKind::Page,
        WonderlandCycleStepAction::ReturnResult { .. } => {
            WonderlandCycleRuntimeActionKind::ReturnResult
        }
        WonderlandCycleStepAction::Log { .. } => WonderlandCycleRuntimeActionKind::Log,
    }
}

fn choose_talk_option_action_kind(
    action: &ChooseTalkOptionStepAction,
) -> ChooseTalkOptionRuntimeActionKind {
    match action {
        ChooseTalkOptionStepAction::Page { .. } => ChooseTalkOptionRuntimeActionKind::Page,
        ChooseTalkOptionStepAction::Input { .. } => ChooseTalkOptionRuntimeActionKind::Input,
        ChooseTalkOptionStepAction::Locator { .. } => ChooseTalkOptionRuntimeActionKind::Locator,
        ChooseTalkOptionStepAction::RecognizeOptions { .. } => {
            ChooseTalkOptionRuntimeActionKind::RecognizeOptions
        }
        ChooseTalkOptionStepAction::MatchText { .. } => {
            ChooseTalkOptionRuntimeActionKind::MatchText
        }
        ChooseTalkOptionStepAction::CheckOrange { .. } => {
            ChooseTalkOptionRuntimeActionKind::CheckOrange
        }
        ChooseTalkOptionStepAction::ClickMatchedOption => {
            ChooseTalkOptionRuntimeActionKind::ClickMatchedOption
        }
        ChooseTalkOptionStepAction::ReturnResult { .. } => {
            ChooseTalkOptionRuntimeActionKind::ReturnResult
        }
        ChooseTalkOptionStepAction::Log { .. } => ChooseTalkOptionRuntimeActionKind::Log,
    }
}
