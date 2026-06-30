#![allow(clippy::large_enum_variant, clippy::too_many_arguments)]

mod auto_artifact_salvage;
mod auto_boss;
mod auto_cook;
mod auto_domain;
mod auto_eat;
mod auto_fight;
mod auto_fish;
mod auto_fishing_task;
mod auto_genius_invokation;
mod auto_ley_line_outcrop;
mod auto_music_game;
mod auto_open_chest;
mod auto_pathing;
mod auto_pick;
mod auto_skip;
mod auto_stygian_onslaught;
mod auto_track;
mod auto_track_path;
mod auto_wood;
mod catalog;
mod common_job;
mod common_job_executor;
mod game_loading;
mod get_grid_icons;
mod macro_hotkeys;
mod map_mask;
mod map_recognition;
mod quick_buy;
mod quick_serenitea_pot;
mod quick_teleport;
mod redeem_code;
mod runtime;
mod shell;
mod skill_cd;
mod task_params;
mod task_state;

use std::path::{Path, PathBuf};

const BGI_TASK_ASSET_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const BGI_TASK_ASSET_ROOT_ENV: &str = "BGI_TASK_ASSET_ROOT";

pub fn task_asset_root() -> PathBuf {
    if let Some(root) = task_asset_root_from_env() {
        return root;
    }

    if let Some(root) = task_asset_root_from_current_exe() {
        return root;
    }

    PathBuf::from(BGI_TASK_ASSET_ROOT)
}

fn task_asset_root_from_env() -> Option<PathBuf> {
    std::env::var_os(BGI_TASK_ASSET_ROOT_ENV).map(PathBuf::from)
}

fn task_asset_root_from_current_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    task_asset_root_from_app_dir(exe_dir)
}

fn task_asset_root_from_app_dir(app_dir: &Path) -> Option<PathBuf> {
    for dir in app_dir.ancestors() {
        for candidate in [dir.join("assets"), dir.join("Assets"), dir.to_path_buf()] {
            if candidate.join("GameTask").is_dir() {
                return Some(candidate);
            }
        }
    }
    None
}

pub use auto_artifact_salvage::*;
pub use auto_boss::*;
pub use auto_cook::*;
pub use auto_domain::*;
pub use auto_eat::*;
pub use auto_fight::*;
pub use auto_fish::*;
pub use auto_fishing_task::*;
pub use auto_genius_invokation::*;
pub use auto_ley_line_outcrop::*;
pub use auto_music_game::*;
pub use auto_open_chest::*;
pub use auto_pathing::{
    evaluate_auto_pathing_resolution_preflight,
    execute_auto_pathing_action_boundary_with_live_executor,
    execute_auto_pathing_movement_contract_with_runtime,
    execute_auto_pathing_movement_contract_with_runtime_and_cancellation,
    execute_auto_pathing_with_runtime, execute_auto_pathing_with_runtime_and_cancellation,
    plan_auto_pathing, plan_auto_pathing_with_track_converter,
    AutoPathingActionBoundaryCompletionScope, AutoPathingActionBoundaryReport,
    AutoPathingExecutionConfig, AutoPathingExecutionPlan, AutoPathingExecutionReport,
    AutoPathingFailedPreflight, AutoPathingFailedWaypoint, AutoPathingMovementBoundaryReport,
    AutoPathingMovementCompletionStatus, AutoPathingMovementFailedPhase,
    AutoPathingMovementPhaseExecutionContext, AutoPathingMovementRuntime,
    AutoPathingPhaseExecution, AutoPathingPhaseExecutionContext, AutoPathingPhaseExecutionReport,
    AutoPathingPhaseExecutionStatus, AutoPathingPreflightExecutionContext,
    AutoPathingPreflightExecutionReport, AutoPathingPreflightPhase,
    AutoPathingResolutionPreflightReport, AutoPathingResolutionPreflightStatus, AutoPathingRuntime,
    PathingActionBoundaryReport, PathingBoundaryStatus, PathingMovementPhaseBoundaryReport,
    PathingMovementSegmentBoundaryReport, PathingMovementWaypointBoundaryReport,
    PathingNavigationSeedReport, PathingPhaseBoundaryReport, PathingWaypointBoundaryReport,
    UnsupportedAutoPathingMovementRuntime, UnsupportedAutoPathingRuntime,
};
pub use auto_pick::{
    auto_pick_text_only_crop_width, auto_pick_tick_requires_runtime_lists,
    compute_auto_pick_text_rect, decide_auto_pick_pre_ocr, decide_auto_pick_text,
    decide_auto_pick_tick, execute_auto_pick_tick_plan, is_valid_auto_pick_text_bounds,
    parse_auto_pick_text_list, parse_auto_pick_text_set, plan_auto_pick,
    process_auto_pick_ocr_text, should_auto_pick_skip_in_progress, should_auto_pick_skip_text,
    AutoPickColorProbe, AutoPickConfigRule, AutoPickDecisionRule, AutoPickDoNotPickRule,
    AutoPickExecutedAction, AutoPickExecutionConfig, AutoPickExecutionPlan, AutoPickExternalConfig,
    AutoPickInProgressRule, AutoPickListFiles, AutoPickOcrCleanupRule, AutoPickOcrEngine,
    AutoPickPreOcrDecision, AutoPickPreOcrSkipReason, AutoPickRegionAnchor,
    AutoPickRelativeHeightSource, AutoPickRelativeRegion, AutoPickRelativeTemplateLocator,
    AutoPickRgbColor, AutoPickRuntime, AutoPickRuntimeLists, AutoPickScrollRule,
    AutoPickTemplateLocator, AutoPickTemplateRule, AutoPickTextDecision,
    AutoPickTextExtractionRule, AutoPickTextRegionRule, AutoPickTextSkipReason, AutoPickTickAction,
    AutoPickTickDecisionAction, AutoPickTickDecisionReport, AutoPickTickExecutionReport,
    AutoPickTickObservation, AutoPickTickPhase, AutoPickTickPickReason, AutoPickTickSkipReason,
    AutoPickTickStep, AUTO_PICK_CHAT_ICON_ASSET, AUTO_PICK_DEFAULT_BLACK_LIST_JSON,
    AUTO_PICK_DEFAULT_CAPTURE_HEIGHT, AUTO_PICK_DEFAULT_CAPTURE_WIDTH, AUTO_PICK_L_KEY_ASSET,
    AUTO_PICK_PICK_KEY_ASSET, AUTO_PICK_SETTINGS_ICON_ASSET, AUTO_PICK_TASK_KEY,
    AUTO_PICK_USER_BLACK_LIST_TXT, AUTO_PICK_USER_FUZZY_BLACK_LIST_TXT,
    AUTO_PICK_USER_WHITE_LIST_TXT,
};
pub use auto_skip::*;
pub use auto_stygian_onslaught::*;
pub use auto_track::*;
pub use auto_track_path::*;
pub use auto_wood::*;
pub use catalog::{
    find_task_catalog_entry, task_catalog, TaskCatalogEntry, TaskKind, TaskLaunchPolicy,
    TaskPortState, TaskRustExecutionSurface,
};
pub use common_job::{
    apply_teleport_move_map_center_observation, classify_teleport_move_map_post_drag_center,
    decide_teleport_move_map_center_after_drag, default_teleport_move_map_rule,
    execute_linnea_mining_plan, plan_blessing_of_the_welkin_moon, plan_check_rewards,
    plan_choose_talk_option, plan_claim_battle_pass_rewards, plan_claim_encounter_points_rewards,
    plan_claim_mail_rewards, plan_common_job, plan_count_inventory_item,
    plan_go_to_adventurers_guild, plan_go_to_crafting_bench, plan_go_to_serenitea_pot,
    plan_linnea_mining, plan_lower_head_then_walk_to, plan_one_key_expedition,
    plan_one_key_expedition_with_locators, plan_relogin, plan_return_main_ui, plan_scan_pick_drops,
    plan_set_time, plan_switch_party, plan_walk_to_f, plan_wonderland_cycle,
    preflight_common_job_pathing_rule, reduce_lower_head_then_walk_to_tracking_frame,
    select_linnea_mining_target, teleport_move_map_expected_move_len,
    teleport_move_map_false_positive_threshold, teleport_move_map_jump_distance,
    BattlePassClaimAllRule, BattlePassClaimScope, BattlePassManualSelectionDialogRule,
    BattlePassRewardLocators, BattlePassRewardStep, BattlePassRewardStepAction,
    BattlePassRewardStepCondition, BattlePassRewardStepPhase, BattlePassRewardStepResult,
    BlessingOfTheWelkinMoonDetectionLocators, BlessingOfTheWelkinMoonExecutionConfig,
    BlessingOfTheWelkinMoonExecutionPlan, BlessingOfTheWelkinMoonLoopRule,
    BlessingOfTheWelkinMoonServerTimeGate, BlessingOfTheWelkinMoonStep,
    BlessingOfTheWelkinMoonStepAction, BlessingOfTheWelkinMoonStepCondition,
    CheckRewardsExecutionConfig, CheckRewardsExecutionPlan, CheckRewardsLocalizedTexts,
    CheckRewardsLocators, CheckRewardsNotifications, CheckRewardsRetryRule, CheckRewardsStep,
    CheckRewardsStepAction, CheckRewardsStepCondition, CheckRewardsStepPhase,
    CheckRewardsStepResult, ChooseTalkOptionExecutionConfig, ChooseTalkOptionExecutionPlan,
    ChooseTalkOptionOcrRule, ChooseTalkOptionOrangeRule, ChooseTalkOptionStep,
    ChooseTalkOptionStepAction, ChooseTalkOptionStepCondition,
    ClaimBattlePassRewardsExecutionConfig, ClaimBattlePassRewardsExecutionPlan,
    ClaimEncounterPointsRewardsExecutionConfig, ClaimEncounterPointsRewardsExecutionPlan,
    ClaimEncounterPointsRewardsOcrRule, ClaimEncounterPointsRewardsStep,
    ClaimEncounterPointsRewardsStepAction, ClaimEncounterPointsRewardsStepCondition,
    ClaimEncounterPointsRewardsStepResult, ClaimMailRewardsExecutionConfig,
    ClaimMailRewardsExecutionPlan, ClaimMailRewardsLocators, ClaimMailRewardsStep,
    ClaimMailRewardsStepAction, ClaimMailRewardsStepCondition, ClaimMailRewardsStepPhase,
    ClaimMailRewardsStepResult, CommonJobExecutionPlan, CommonJobPathingPreflightReport,
    CommonJobStep, CommonJobStepAction, CommonJobStepCondition, CommonJobStepPhase,
    CountInventoryItemExecutionConfig, CountInventoryItemExecutionPlan, CountInventoryItemStep,
    CountInventoryItemStepAction, CountInventoryItemStepCondition, CountInventoryItemStepPhase,
    CountInventoryOpenInventoryRule, CountInventoryResultContract, CountInventorySearchMode,
    GoToAdventurersGuildDailyRewardRule, GoToAdventurersGuildExecutionConfig,
    GoToAdventurersGuildExecutionPlan, GoToAdventurersGuildExpeditionRule,
    GoToAdventurersGuildInteractionRule, GoToAdventurersGuildLocalizedTexts,
    GoToAdventurersGuildLocators, GoToAdventurersGuildPathingPartyConfig,
    GoToAdventurersGuildPathingRule, GoToAdventurersGuildStep, GoToAdventurersGuildStepAction,
    GoToAdventurersGuildStepCondition, GoToAdventurersGuildStepPhase,
    GoToAdventurersGuildStepResult, GoToCraftingBenchActionPress,
    GoToCraftingBenchCraftingPageRule, GoToCraftingBenchCropAnchor,
    GoToCraftingBenchExecutionConfig, GoToCraftingBenchExecutionPlan,
    GoToCraftingBenchInteractionRule, GoToCraftingBenchLocalizedTexts, GoToCraftingBenchLocators,
    GoToCraftingBenchPathingPartyConfig, GoToCraftingBenchPathingRule,
    GoToCraftingBenchRelativeCrop, GoToCraftingBenchResinCraftRule,
    GoToCraftingBenchResinRecognitionRule, GoToCraftingBenchStep, GoToCraftingBenchStepAction,
    GoToCraftingBenchStepCondition, GoToCraftingBenchStepPhase, GoToCraftingBenchStepResult,
    GoToSereniteaPotActionPress, GoToSereniteaPotBagEntryRule, GoToSereniteaPotBuyMaxRule,
    GoToSereniteaPotConfigRule, GoToSereniteaPotDayOfWeek, GoToSereniteaPotEntryMode,
    GoToSereniteaPotExecutionConfig, GoToSereniteaPotExecutionPlan, GoToSereniteaPotFindAYuanRule,
    GoToSereniteaPotFinishRule, GoToSereniteaPotLocalizedTexts, GoToSereniteaPotLocators,
    GoToSereniteaPotMapEntryRule, GoToSereniteaPotOcrRule, GoToSereniteaPotRealmAdjustment,
    GoToSereniteaPotRecognitionType, GoToSereniteaPotRelativeCrop, GoToSereniteaPotRewardRule,
    GoToSereniteaPotShopDay, GoToSereniteaPotShopItem, GoToSereniteaPotShopItemLocator,
    GoToSereniteaPotShopRule, GoToSereniteaPotStep, GoToSereniteaPotStepAction,
    GoToSereniteaPotStepCondition, GoToSereniteaPotStepPhase, GoToSereniteaPotStepResult,
    GoToSereniteaPotTimedAction, GoToSereniteaPotTimedActionKind, GridBgrColor,
    GridIconClassifierRule, GridIconCropRule, GridItemCountOcrRule, GridItemDetectionRule,
    GridScreenName, GridScrollRule, GridTemplate, InventoryTabAssetPair, LinneaMiningAimingRule,
    LinneaMiningAlignmentRule, LinneaMiningAvatarRule, LinneaMiningCleanupRule,
    LinneaMiningCluster, LinneaMiningClusterRule, LinneaMiningDecision, LinneaMiningDecisionKind,
    LinneaMiningDetection, LinneaMiningDetectionRule, LinneaMiningDetectionSource,
    LinneaMiningExecutionConfig, LinneaMiningExecutionPlan, LinneaMiningExecutionReport,
    LinneaMiningExecutionStatus, LinneaMiningExecutorState, LinneaMiningMineRule,
    LinneaMiningObservation, LinneaMiningPoint, LinneaMiningRect, LinneaMiningRuntime,
    LinneaMiningRuntimeActionKind, LinneaMiningRuntimeActionReport, LinneaMiningRuntimeOutcome,
    LinneaMiningScanRule, LinneaMiningScreenSize, LinneaMiningStep, LinneaMiningStepAction,
    LinneaMiningStepCondition, LinneaMiningStepPhase, LinneaMiningTarget,
    LowerHeadThenWalkToActionPress, LowerHeadThenWalkToExecutionConfig,
    LowerHeadThenWalkToExecutionPlan, LowerHeadThenWalkToFKeyRule, LowerHeadThenWalkToLocators,
    LowerHeadThenWalkToMovementRule, LowerHeadThenWalkToStep, LowerHeadThenWalkToStepAction,
    LowerHeadThenWalkToStepCondition, LowerHeadThenWalkToStepPhase, LowerHeadThenWalkToStepResult,
    LowerHeadThenWalkToTrackingDecision, LowerHeadThenWalkToTrackingDecisionKind,
    LowerHeadThenWalkToTrackingObservation, OneKeyExpeditionExecutionConfig,
    OneKeyExpeditionExecutionPlan, OneKeyExpeditionStep, OneKeyExpeditionStepAction,
    OneKeyExpeditionStepCondition, OneKeyExpeditionStepPhase, OneKeyExpeditionStepResult,
    PartyTextClickYAnchor, ReloginDpiAwarePoint, ReloginExecutionConfig, ReloginExecutionPlan,
    ReloginFailurePolicy, ReloginLocators, ReloginRetryAction, ReloginRetryRule, ReloginStep,
    ReloginStepAction, ReloginStepCondition, ReloginStepPhase, ReloginStepResult,
    ReloginThirdPartyRule, ReturnMainUiExecutionConfig, ReturnMainUiExecutionPlan,
    ScanPickCameraResetRule, ScanPickDropsActionPress, ScanPickDropsExecutionConfig,
    ScanPickDropsExecutionPlan, ScanPickDropsStep, ScanPickDropsStepAction,
    ScanPickDropsStepCondition, ScanPickDropsStepPhase, ScanPickDropsStepResult,
    ScanPickMovementRule, ScanPickSearchRule, ScanPickTargetOrderingRule, ScanPickYoloRule,
    ScanPickYoloSource, SetTimeDialDrag, SetTimeDialPoint, SetTimeExecutionConfig,
    SetTimeExecutionPlan, SwitchPartyChooseMenuRule, SwitchPartyConfirmRule,
    SwitchPartyCurrentPartyRule, SwitchPartyExecutionConfig, SwitchPartyExecutionPlan,
    SwitchPartyListScanRule, SwitchPartyLocators, SwitchPartyOpenRule, SwitchPartyScreenPoint,
    SwitchPartyStep, SwitchPartyStepAction, SwitchPartyStepCondition, SwitchPartyStepPhase,
    SwitchPartyStepResult, TalkOptionPlanResult, TeleportCountryPositionRule,
    TeleportExecutionConfig, TeleportExecutionPlan, TeleportFailurePolicy, TeleportMapPoint,
    TeleportMapRule, TeleportMoveMapCenterDecision, TeleportMoveMapCenterRejectReason,
    TeleportMoveMapPostDragObservation, TeleportMoveMapRule, TeleportNativeDependency,
    TeleportPlanKind, TeleportPreflightPlan, TeleportQuickTeleportRule, TeleportRetryRule,
    TeleportStep, TeleportStepAction, TeleportStepPhase, TeleportStepResult, TeleportTargetPlan,
    WalkToFActionPress, WalkToFExecutionConfig, WalkToFExecutionPlan, WalkToFRetryRule,
    WalkToFStep, WalkToFStepAction, WalkToFStepCondition, WalkToFStepPhase, WalkToFStepResult,
    WeaponOrePrescrollRule, WonderlandCycleExecutionConfig, WonderlandCycleExecutionPlan,
    WonderlandCycleLocators, WonderlandCycleRetryAction, WonderlandCycleRetryRule,
    WonderlandCycleStep, WonderlandCycleStepAction, WonderlandCycleStepCondition,
    WonderlandCycleStepPhase, WonderlandCycleStepResult, BLESSING_WELKIN_GIRL_MOON,
    BLESSING_WELKIN_MAX_ITERATIONS, BLESSING_WELKIN_PRIMOGEM, BLESSING_WELKIN_RETRY_DELAY_MS,
    BLESSING_WELKIN_STABLE_CLEAR_COUNT, BLESSING_WELKIN_TASK_KEY, BLESSING_WELKIN_WELKIN_MOON,
    CHECK_REWARDS_DEFAULT_CLAIMED_TEXT, CHECK_REWARDS_DEFAULT_COMMISSIONS_TEXT,
    CHECK_REWARDS_DEFAULT_DAILY_REWARD_TEXT, CHECK_REWARDS_FAILURE_MESSAGE,
    CHECK_REWARDS_NOTIFICATION_EVENT, CHECK_REWARDS_SUCCESS_MESSAGE, CHECK_REWARDS_TASK_KEY,
    CHOOSE_TALK_OPTION_DISABLED_UI, CHOOSE_TALK_OPTION_ICON, CHOOSE_TALK_OPTION_TASK_KEY,
    CHOOSE_TALK_OPTION_VK_SPACE, CLAIM_BATTLE_PASS_BLACK_CANCEL, CLAIM_BATTLE_PASS_BLACK_CONFIRM,
    CLAIM_BATTLE_PASS_PRIMOGEM, CLAIM_BATTLE_PASS_PROMPT_STAR, CLAIM_BATTLE_PASS_REWARDS_TASK_KEY,
    CLAIM_BATTLE_PASS_WHITE_CANCEL, CLAIM_BATTLE_PASS_WHITE_CONFIRM,
    CLAIM_ENCOUNTER_POINTS_REWARDS_BUTTON, CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES,
    CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY, CLAIM_MAIL_REWARDS_COLLECT,
    CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD, CLAIM_MAIL_REWARDS_TASK_KEY, COUNT_INVENTORY_ITEM_TASK_KEY,
    COUNT_INVENTORY_OCR_FAILED, COUNT_INVENTORY_SINGLE_NOT_FOUND,
    GO_TO_ADVENTURERS_GUILD_BLACK_CONFIRM, GO_TO_ADVENTURERS_GUILD_DEFAULT_DAILY_SKIP_TIMES,
    GO_TO_ADVENTURERS_GUILD_DEFAULT_RETRY_TIMES, GO_TO_ADVENTURERS_GUILD_DEFAULT_TALK_RETRY_TIMES,
    GO_TO_ADVENTURERS_GUILD_EXPEDITION_COLLECT, GO_TO_ADVENTURERS_GUILD_EXPEDITION_RE,
    GO_TO_ADVENTURERS_GUILD_TASK_KEY, GO_TO_CRAFTING_BENCH_BLACK_CONFIRM,
    GO_TO_CRAFTING_BENCH_CONDENSED_RESIN, GO_TO_CRAFTING_BENCH_CONDENSED_RESIN_COUNT,
    GO_TO_CRAFTING_BENCH_DEFAULT_RETRY_TIMES, GO_TO_CRAFTING_BENCH_FRAGILE_RESIN_COUNT,
    GO_TO_CRAFTING_BENCH_KEY_INCREASE, GO_TO_CRAFTING_BENCH_KEY_REDUCE,
    GO_TO_CRAFTING_BENCH_TALK_UI, GO_TO_CRAFTING_BENCH_TASK_KEY,
    GO_TO_CRAFTING_BENCH_WHITE_CONFIRM, GO_TO_SERENITEA_POT_AREA_NAME,
    GO_TO_SERENITEA_POT_BAG_CLOSE_BUTTON, GO_TO_SERENITEA_POT_BAG_TP_TYPE,
    GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME, GO_TO_SERENITEA_POT_FINAL_TP_X,
    GO_TO_SERENITEA_POT_FINAL_TP_Y, GO_TO_SERENITEA_POT_FINGER, GO_TO_SERENITEA_POT_HOME,
    GO_TO_SERENITEA_POT_ICON, GO_TO_SERENITEA_POT_LOVE, GO_TO_SERENITEA_POT_MAP_TP_TYPE,
    GO_TO_SERENITEA_POT_MONEY, GO_TO_SERENITEA_POT_ONE_DRAGON_FOLDER,
    GO_TO_SERENITEA_POT_PAGE_CLOSE_WHITE, GO_TO_SERENITEA_POT_POT_PAGE_CLOSE,
    GO_TO_SERENITEA_POT_TASK_KEY, GO_TO_SERENITEA_POT_TELEPORT_BUTTON,
    GO_TO_SERENITEA_POT_TELEPORT_HOME, GO_TO_SERENITEA_POT_WHITE_CONFIRM, GRID_ICON_INPUT_NAME,
    GRID_ICON_MODEL_NAME, GRID_ICON_MODEL_PATH, GRID_ICON_PROTOTYPE_CSV_PATH,
    LINNEA_MINING_DEFAULT_MINE_COUNT, LINNEA_MINING_DEFAULT_SCAN_ROUNDS, LINNEA_MINING_MODEL_NAME,
    LINNEA_MINING_MODEL_PATH, LINNEA_MINING_TASK_KEY, LOWER_HEAD_THEN_WALK_TO_ACTIVATION_TEXT,
    LOWER_HEAD_THEN_WALK_TO_DEFAULT_TARGET, LOWER_HEAD_THEN_WALK_TO_DEFAULT_TIMEOUT_MS,
    LOWER_HEAD_THEN_WALK_TO_LOOP_DELAY_MS, LOWER_HEAD_THEN_WALK_TO_PICK_KEY,
    LOWER_HEAD_THEN_WALK_TO_TASK_KEY, ONE_KEY_EXPEDITION_COLLECT, ONE_KEY_EXPEDITION_RE_DISPATCH,
    ONE_KEY_EXPEDITION_TASK_KEY, ONE_KEY_EXPEDITION_VK_ESCAPE, RELOGIN_CONFIRM,
    RELOGIN_DEFAULT_CAPTURE_HEIGHT, RELOGIN_DEFAULT_CAPTURE_WIDTH, RELOGIN_ENTER_GAME,
    RELOGIN_MENU_BAG, RELOGIN_TASK_KEY, RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
    RETURN_MAIN_UI_EXIT_DOOR, RETURN_MAIN_UI_PAIMON_MENU, RETURN_MAIN_UI_TASK_KEY,
    RETURN_MAIN_UI_VK_ESCAPE, RETURN_MAIN_UI_VK_RETURN, SCAN_PICK_DROPS_DEFAULT_SCAN_SECONDS,
    SCAN_PICK_DROPS_DROP_LABEL, SCAN_PICK_DROPS_ORE_LABEL, SCAN_PICK_DROPS_TASK_KEY,
    SCAN_PICK_DROPS_WORLD_MODEL_NAME, SCAN_PICK_DROPS_WORLD_MODEL_PATH, SET_TIME_CENTER_X_1080P,
    SET_TIME_CENTER_Y_1080P, SET_TIME_DEFAULT_STEP_DURATION_MS, SET_TIME_PAGE_CLOSE_WHITE,
    SET_TIME_TASK_KEY, SWITCH_PARTY_CHOOSE_VIEW, SWITCH_PARTY_DEFAULT_LIST_SCAN_PAGES,
    SWITCH_PARTY_DEFAULT_OPEN_ATTEMPTS, SWITCH_PARTY_DEFAULT_OPEN_CHECKS_PER_ATTEMPT,
    SWITCH_PARTY_DELETE, SWITCH_PARTY_TASK_KEY, SWITCH_PARTY_WHITE_CONFIRM, TELEPORT_TASK_KEY,
    WALK_TO_F_DEFAULT_TIMEOUT_MS, WALK_TO_F_MOVE_START_DELAY_MS, WALK_TO_F_PICK_KEY,
    WALK_TO_F_RELEASE_GAP_MS, WALK_TO_F_RETRY_INTERVAL_MS, WALK_TO_F_TASK_KEY, WALK_TO_F_VK_F,
    WONDERLAND_CYCLE_BACK_TEYVAT, WONDERLAND_CYCLE_BLACK_CONFIRM, WONDERLAND_CYCLE_CLOSE,
    WONDERLAND_CYCLE_TASK_KEY,
};
pub use common_job_executor::*;
pub use game_loading::*;
pub use get_grid_icons::*;
pub use macro_hotkeys::*;
pub use map_mask::*;
pub use map_recognition::*;
pub use quick_buy::*;
pub use quick_serenitea_pot::*;
pub use quick_teleport::*;
pub use redeem_code::{
    execute_use_redeem_code_plan, extract_redeem_codes_from_text, plan_use_redeem_code_strings,
    plan_use_redeem_codes, redeem_code_entries_from_strings, RedeemCodeEntry,
    UseRedeemCodeExecutionConfig, UseRedeemCodeExecutionPlan, UseRedeemCodeExecutionReport,
    UseRedeemCodeExecutorState, UseRedeemCodeRuntime, UseRedeemCodeRuntimeActionKind,
    UseRedeemCodeRuntimeStepReport, UseRedeemCodeSkipReason, UseRedeemCodeSkippedStep,
    UseRedeemCodeStep, UseRedeemCodeStepAction, UseRedeemCodeStepCondition, UseRedeemCodeStepPhase,
    UseRedeemCodeSuccessDetection, COMMON_BTN_BLACK_CONFIRM, COMMON_BTN_WHITE_CONFIRM,
    REDEEM_CODE_PATTERN, USE_REDEEM_CODE_ESC_RETURN_BUTTON, USE_REDEEM_CODE_TASK_KEY, VK_ESCAPE,
};
pub use runtime::*;
pub use shell::{
    execute_shell_task, execute_shell_task_with_cancel, ShellConfig, ShellExecutionResult,
    ShellExecutionStatus, ShellTaskParam,
};
pub use skill_cd::*;
pub use task_params::{
    combat_strategy_path, task_parameter_models, AutoBossParam, AutoDomainParam, AutoFightParam,
    AutoLeyLineOutcropFightConfigParam, AutoLeyLineOutcropParam, AutoSkipConfigParam,
    AutoStygianOnslaughtParam, FightFinishDetectParam, TaskParameterModels, AUTO_STRATEGY_NAME,
};
pub use task_state::{TaskProgress, TaskRegistry};

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
    #[error("invalid task config for {key}: {message}")]
    InvalidTaskConfig { key: String, message: String },
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
    #[error("common job execution failed: {0}")]
    CommonJobExecution(String),
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

#[cfg(test)]
mod tests;
