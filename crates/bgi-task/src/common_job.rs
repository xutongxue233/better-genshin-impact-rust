use crate::{Result, TaskError, TaskPortState};
#[path = "common_job_blessing_of_the_welkin_moon.rs"]
mod common_job_blessing_of_the_welkin_moon;
#[path = "common_job_check_rewards.rs"]
mod common_job_check_rewards;
#[path = "common_job_choose_talk_option.rs"]
mod common_job_choose_talk_option;
#[path = "common_job_claim_battle_pass_rewards.rs"]
mod common_job_claim_battle_pass_rewards;
#[path = "common_job_claim_encounter_points_rewards.rs"]
mod common_job_claim_encounter_points_rewards;
#[path = "common_job_claim_mail_rewards.rs"]
mod common_job_claim_mail_rewards;
#[path = "common_job_count_inventory_item.rs"]
mod common_job_count_inventory_item;
#[path = "common_job_go_to_adventurers_guild.rs"]
mod common_job_go_to_adventurers_guild;
#[path = "common_job_go_to_crafting_bench.rs"]
mod common_job_go_to_crafting_bench;
#[path = "common_job_go_to_serenitea_pot.rs"]
mod common_job_go_to_serenitea_pot;
#[path = "common_job_linnea_mining.rs"]
mod common_job_linnea_mining;
#[path = "common_job_lower_head_then_walk_to.rs"]
mod common_job_lower_head_then_walk_to;
#[path = "common_job_one_key_expedition.rs"]
mod common_job_one_key_expedition;
#[path = "common_job_pathing.rs"]
mod common_job_pathing;
#[path = "common_job_relogin.rs"]
mod common_job_relogin;
#[path = "common_job_scan_pick_drops.rs"]
mod common_job_scan_pick_drops;
#[path = "common_job_set_time.rs"]
mod common_job_set_time;
#[path = "common_job_switch_party.rs"]
mod common_job_switch_party;
#[path = "common_job_teleport.rs"]
mod common_job_teleport;
#[path = "common_job_walk_to_f.rs"]
mod common_job_walk_to_f;
#[path = "common_job_wonderland_cycle.rs"]
mod common_job_wonderland_cycle;

use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use common_job_blessing_of_the_welkin_moon::{
    plan_blessing_of_the_welkin_moon, BlessingOfTheWelkinMoonDetectionLocators,
    BlessingOfTheWelkinMoonExecutionConfig, BlessingOfTheWelkinMoonExecutionPlan,
    BlessingOfTheWelkinMoonLoopRule, BlessingOfTheWelkinMoonServerTimeGate,
    BlessingOfTheWelkinMoonStep, BlessingOfTheWelkinMoonStepAction,
    BlessingOfTheWelkinMoonStepCondition, BLESSING_WELKIN_GIRL_MOON,
    BLESSING_WELKIN_MAX_ITERATIONS, BLESSING_WELKIN_PRIMOGEM, BLESSING_WELKIN_RETRY_DELAY_MS,
    BLESSING_WELKIN_STABLE_CLEAR_COUNT, BLESSING_WELKIN_TASK_KEY, BLESSING_WELKIN_WELKIN_MOON,
};
pub use common_job_check_rewards::{
    plan_check_rewards, CheckRewardsExecutionConfig, CheckRewardsExecutionPlan,
    CheckRewardsLocalizedTexts, CheckRewardsLocators, CheckRewardsNotifications,
    CheckRewardsRetryRule, CheckRewardsStep, CheckRewardsStepAction, CheckRewardsStepCondition,
    CheckRewardsStepPhase, CheckRewardsStepResult, CHECK_REWARDS_DEFAULT_CLAIMED_TEXT,
    CHECK_REWARDS_DEFAULT_COMMISSIONS_TEXT, CHECK_REWARDS_DEFAULT_DAILY_REWARD_TEXT,
    CHECK_REWARDS_FAILURE_MESSAGE, CHECK_REWARDS_NOTIFICATION_EVENT, CHECK_REWARDS_SUCCESS_MESSAGE,
    CHECK_REWARDS_TASK_KEY,
};
pub use common_job_choose_talk_option::{
    plan_choose_talk_option, ChooseTalkOptionExecutionConfig, ChooseTalkOptionExecutionPlan,
    ChooseTalkOptionOcrRule, ChooseTalkOptionOrangeRule, ChooseTalkOptionStep,
    ChooseTalkOptionStepAction, ChooseTalkOptionStepCondition, TalkOptionPlanResult,
    CHOOSE_TALK_OPTION_DISABLED_UI, CHOOSE_TALK_OPTION_ICON, CHOOSE_TALK_OPTION_TASK_KEY,
    CHOOSE_TALK_OPTION_VK_SPACE,
};
pub use common_job_claim_battle_pass_rewards::{
    plan_claim_battle_pass_rewards, BattlePassClaimAllRule, BattlePassClaimScope,
    BattlePassManualSelectionDialogRule, BattlePassRewardLocators, BattlePassRewardStep,
    BattlePassRewardStepAction, BattlePassRewardStepCondition, BattlePassRewardStepPhase,
    BattlePassRewardStepResult, ClaimBattlePassRewardsExecutionConfig,
    ClaimBattlePassRewardsExecutionPlan, CLAIM_BATTLE_PASS_BLACK_CANCEL,
    CLAIM_BATTLE_PASS_BLACK_CONFIRM, CLAIM_BATTLE_PASS_PRIMOGEM, CLAIM_BATTLE_PASS_PROMPT_STAR,
    CLAIM_BATTLE_PASS_REWARDS_TASK_KEY, CLAIM_BATTLE_PASS_WHITE_CANCEL,
    CLAIM_BATTLE_PASS_WHITE_CONFIRM,
};
pub use common_job_claim_encounter_points_rewards::{
    plan_claim_encounter_points_rewards, ClaimEncounterPointsRewardsExecutionConfig,
    ClaimEncounterPointsRewardsExecutionPlan, ClaimEncounterPointsRewardsOcrRule,
    ClaimEncounterPointsRewardsStep, ClaimEncounterPointsRewardsStepAction,
    ClaimEncounterPointsRewardsStepCondition, ClaimEncounterPointsRewardsStepResult,
    CLAIM_ENCOUNTER_POINTS_REWARDS_BUTTON, CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES,
    CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY,
};
pub use common_job_claim_mail_rewards::{
    plan_claim_mail_rewards, ClaimMailRewardsExecutionConfig, ClaimMailRewardsExecutionPlan,
    ClaimMailRewardsLocators, ClaimMailRewardsStep, ClaimMailRewardsStepAction,
    ClaimMailRewardsStepCondition, ClaimMailRewardsStepPhase, ClaimMailRewardsStepResult,
    CLAIM_MAIL_REWARDS_COLLECT, CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD, CLAIM_MAIL_REWARDS_TASK_KEY,
};
pub use common_job_count_inventory_item::{
    plan_count_inventory_item, CountInventoryItemExecutionConfig, CountInventoryItemExecutionPlan,
    CountInventoryItemStep, CountInventoryItemStepAction, CountInventoryItemStepCondition,
    CountInventoryItemStepPhase, CountInventoryOpenInventoryRule, CountInventoryResultContract,
    CountInventorySearchMode, GridBgrColor, GridIconClassifierRule, GridIconCropRule,
    GridItemCountOcrRule, GridItemDetectionRule, GridScreenName, GridScrollRule, GridTemplate,
    InventoryTabAssetPair, WeaponOrePrescrollRule, COUNT_INVENTORY_ITEM_TASK_KEY,
    COUNT_INVENTORY_OCR_FAILED, COUNT_INVENTORY_SINGLE_NOT_FOUND, GRID_ICON_INPUT_NAME,
    GRID_ICON_MODEL_NAME, GRID_ICON_MODEL_PATH, GRID_ICON_PROTOTYPE_CSV_PATH,
};
pub use common_job_go_to_adventurers_guild::{
    plan_go_to_adventurers_guild, GoToAdventurersGuildDailyRewardRule,
    GoToAdventurersGuildExecutionConfig, GoToAdventurersGuildExecutionPlan,
    GoToAdventurersGuildExpeditionRule, GoToAdventurersGuildInteractionRule,
    GoToAdventurersGuildLocalizedTexts, GoToAdventurersGuildLocators,
    GoToAdventurersGuildPathingPartyConfig, GoToAdventurersGuildPathingRule,
    GoToAdventurersGuildStep, GoToAdventurersGuildStepAction, GoToAdventurersGuildStepCondition,
    GoToAdventurersGuildStepPhase, GoToAdventurersGuildStepResult,
    GO_TO_ADVENTURERS_GUILD_BLACK_CONFIRM, GO_TO_ADVENTURERS_GUILD_DEFAULT_DAILY_SKIP_TIMES,
    GO_TO_ADVENTURERS_GUILD_DEFAULT_RETRY_TIMES, GO_TO_ADVENTURERS_GUILD_DEFAULT_TALK_RETRY_TIMES,
    GO_TO_ADVENTURERS_GUILD_EXPEDITION_COLLECT, GO_TO_ADVENTURERS_GUILD_EXPEDITION_RE,
    GO_TO_ADVENTURERS_GUILD_TASK_KEY,
};
pub use common_job_go_to_crafting_bench::{
    parse_go_to_crafting_bench_condensed_resin_count_ocr_text,
    parse_go_to_crafting_bench_fragile_resin_count_ocr_text, plan_go_to_crafting_bench,
    GoToCraftingBenchActionPress, GoToCraftingBenchCraftingPageRule, GoToCraftingBenchCropAnchor,
    GoToCraftingBenchExecutionConfig, GoToCraftingBenchExecutionPlan,
    GoToCraftingBenchInteractionRule, GoToCraftingBenchLocalizedTexts, GoToCraftingBenchLocators,
    GoToCraftingBenchPathingPartyConfig, GoToCraftingBenchPathingRule,
    GoToCraftingBenchRelativeCrop, GoToCraftingBenchResinCraftRule,
    GoToCraftingBenchResinRecognitionRule, GoToCraftingBenchStep, GoToCraftingBenchStepAction,
    GoToCraftingBenchStepCondition, GoToCraftingBenchStepPhase, GoToCraftingBenchStepResult,
    GO_TO_CRAFTING_BENCH_BLACK_CONFIRM, GO_TO_CRAFTING_BENCH_CONDENSED_RESIN,
    GO_TO_CRAFTING_BENCH_CONDENSED_RESIN_COUNT, GO_TO_CRAFTING_BENCH_DEFAULT_RETRY_TIMES,
    GO_TO_CRAFTING_BENCH_FRAGILE_RESIN_COUNT, GO_TO_CRAFTING_BENCH_KEY_INCREASE,
    GO_TO_CRAFTING_BENCH_KEY_REDUCE, GO_TO_CRAFTING_BENCH_TALK_UI, GO_TO_CRAFTING_BENCH_TASK_KEY,
    GO_TO_CRAFTING_BENCH_WHITE_CONFIRM,
};
pub use common_job_go_to_serenitea_pot::{
    plan_go_to_serenitea_pot, GoToSereniteaPotActionPress, GoToSereniteaPotBagEntryRule,
    GoToSereniteaPotBuyMaxRule, GoToSereniteaPotConfigRule, GoToSereniteaPotDayOfWeek,
    GoToSereniteaPotEntryMode, GoToSereniteaPotExecutionConfig, GoToSereniteaPotExecutionPlan,
    GoToSereniteaPotFindAYuanRule, GoToSereniteaPotFinishRule, GoToSereniteaPotLocalizedTexts,
    GoToSereniteaPotLocators, GoToSereniteaPotMapEntryRule, GoToSereniteaPotOcrRule,
    GoToSereniteaPotRealmAdjustment, GoToSereniteaPotRecognitionType, GoToSereniteaPotRelativeCrop,
    GoToSereniteaPotRewardRule, GoToSereniteaPotShopDay, GoToSereniteaPotShopItem,
    GoToSereniteaPotShopItemLocator, GoToSereniteaPotShopRule, GoToSereniteaPotStep,
    GoToSereniteaPotStepAction, GoToSereniteaPotStepCondition, GoToSereniteaPotStepPhase,
    GoToSereniteaPotStepResult, GoToSereniteaPotTimedAction, GoToSereniteaPotTimedActionKind,
    GO_TO_SERENITEA_POT_AREA_NAME, GO_TO_SERENITEA_POT_BAG_CLOSE_BUTTON,
    GO_TO_SERENITEA_POT_BAG_TP_TYPE, GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME,
    GO_TO_SERENITEA_POT_FINAL_TP_X, GO_TO_SERENITEA_POT_FINAL_TP_Y, GO_TO_SERENITEA_POT_FINGER,
    GO_TO_SERENITEA_POT_HOME, GO_TO_SERENITEA_POT_ICON, GO_TO_SERENITEA_POT_LOVE,
    GO_TO_SERENITEA_POT_MAP_TP_TYPE, GO_TO_SERENITEA_POT_MONEY,
    GO_TO_SERENITEA_POT_ONE_DRAGON_FOLDER, GO_TO_SERENITEA_POT_PAGE_CLOSE_WHITE,
    GO_TO_SERENITEA_POT_POT_PAGE_CLOSE, GO_TO_SERENITEA_POT_TASK_KEY,
    GO_TO_SERENITEA_POT_TELEPORT_BUTTON, GO_TO_SERENITEA_POT_TELEPORT_HOME,
    GO_TO_SERENITEA_POT_WHITE_CONFIRM,
};
pub use common_job_linnea_mining::{
    execute_linnea_mining_plan, plan_linnea_mining, select_linnea_mining_target,
    LinneaMiningAimingRule, LinneaMiningAlignmentRule, LinneaMiningAvatarRule,
    LinneaMiningCleanupRule, LinneaMiningCluster, LinneaMiningClusterRule, LinneaMiningDecision,
    LinneaMiningDecisionKind, LinneaMiningDetection, LinneaMiningDetectionRule,
    LinneaMiningDetectionSource, LinneaMiningExecutionConfig, LinneaMiningExecutionPlan,
    LinneaMiningExecutionReport, LinneaMiningExecutionStatus, LinneaMiningExecutorState,
    LinneaMiningMineRule, LinneaMiningObservation, LinneaMiningPoint, LinneaMiningRect,
    LinneaMiningRuntime, LinneaMiningRuntimeActionKind, LinneaMiningRuntimeActionReport,
    LinneaMiningRuntimeOutcome, LinneaMiningScanRule, LinneaMiningScreenSize, LinneaMiningStep,
    LinneaMiningStepAction, LinneaMiningStepCondition, LinneaMiningStepPhase, LinneaMiningTarget,
    LINNEA_MINING_DEFAULT_MINE_COUNT, LINNEA_MINING_DEFAULT_SCAN_ROUNDS, LINNEA_MINING_MODEL_NAME,
    LINNEA_MINING_MODEL_PATH, LINNEA_MINING_TASK_KEY,
};
pub use common_job_lower_head_then_walk_to::{
    plan_lower_head_then_walk_to, reduce_lower_head_then_walk_to_tracking_frame,
    LowerHeadThenWalkToActionPress, LowerHeadThenWalkToExecutionConfig,
    LowerHeadThenWalkToExecutionPlan, LowerHeadThenWalkToFKeyRule, LowerHeadThenWalkToLocators,
    LowerHeadThenWalkToMovementRule, LowerHeadThenWalkToStep, LowerHeadThenWalkToStepAction,
    LowerHeadThenWalkToStepCondition, LowerHeadThenWalkToStepPhase, LowerHeadThenWalkToStepResult,
    LowerHeadThenWalkToTrackingDecision, LowerHeadThenWalkToTrackingDecisionKind,
    LowerHeadThenWalkToTrackingObservation, LOWER_HEAD_THEN_WALK_TO_ACTIVATION_TEXT,
    LOWER_HEAD_THEN_WALK_TO_DEFAULT_TARGET, LOWER_HEAD_THEN_WALK_TO_DEFAULT_TIMEOUT_MS,
    LOWER_HEAD_THEN_WALK_TO_LOOP_DELAY_MS, LOWER_HEAD_THEN_WALK_TO_PICK_KEY,
    LOWER_HEAD_THEN_WALK_TO_TASK_KEY,
};
pub use common_job_one_key_expedition::{
    plan_one_key_expedition, plan_one_key_expedition_with_locators,
    OneKeyExpeditionExecutionConfig, OneKeyExpeditionExecutionPlan, OneKeyExpeditionStep,
    OneKeyExpeditionStepAction, OneKeyExpeditionStepCondition, OneKeyExpeditionStepPhase,
    OneKeyExpeditionStepResult, ONE_KEY_EXPEDITION_COLLECT, ONE_KEY_EXPEDITION_RE_DISPATCH,
    ONE_KEY_EXPEDITION_TASK_KEY, ONE_KEY_EXPEDITION_VK_ESCAPE,
};
pub use common_job_pathing::{
    plan_common_job_pathing_action_boundary, preflight_common_job_pathing_rule,
    CommonJobPathingPreflightReport,
};
pub use common_job_relogin::{
    plan_relogin, ReloginDpiAwarePoint, ReloginExecutionConfig, ReloginExecutionPlan,
    ReloginFailurePolicy, ReloginLocators, ReloginRetryAction, ReloginRetryRule, ReloginStep,
    ReloginStepAction, ReloginStepCondition, ReloginStepPhase, ReloginStepResult,
    ReloginThirdPartyRule, RELOGIN_CONFIRM, RELOGIN_DEFAULT_CAPTURE_HEIGHT,
    RELOGIN_DEFAULT_CAPTURE_WIDTH, RELOGIN_ENTER_GAME, RELOGIN_MENU_BAG, RELOGIN_TASK_KEY,
};
pub use common_job_scan_pick_drops::{
    plan_scan_pick_drops, ScanPickCameraResetRule, ScanPickDropsActionPress,
    ScanPickDropsExecutionConfig, ScanPickDropsExecutionPlan, ScanPickDropsStep,
    ScanPickDropsStepAction, ScanPickDropsStepCondition, ScanPickDropsStepPhase,
    ScanPickDropsStepResult, ScanPickMovementRule, ScanPickSearchRule, ScanPickTargetOrderingRule,
    ScanPickYoloRule, ScanPickYoloSource, SCAN_PICK_DROPS_DEFAULT_SCAN_SECONDS,
    SCAN_PICK_DROPS_DROP_LABEL, SCAN_PICK_DROPS_ORE_LABEL, SCAN_PICK_DROPS_TASK_KEY,
    SCAN_PICK_DROPS_WORLD_MODEL_NAME, SCAN_PICK_DROPS_WORLD_MODEL_PATH,
};
pub use common_job_set_time::{
    plan_set_time, SetTimeDialDrag, SetTimeDialPoint, SetTimeExecutionConfig, SetTimeExecutionPlan,
    SET_TIME_CENTER_X_1080P, SET_TIME_CENTER_Y_1080P, SET_TIME_DEFAULT_STEP_DURATION_MS,
    SET_TIME_PAGE_CLOSE_WHITE, SET_TIME_TASK_KEY,
};
pub use common_job_switch_party::{
    plan_switch_party, PartyTextClickYAnchor, SwitchPartyChooseMenuRule, SwitchPartyConfirmRule,
    SwitchPartyCurrentPartyRule, SwitchPartyExecutionConfig, SwitchPartyExecutionPlan,
    SwitchPartyListScanRule, SwitchPartyLocators, SwitchPartyOpenRule, SwitchPartyScreenPoint,
    SwitchPartyStep, SwitchPartyStepAction, SwitchPartyStepCondition, SwitchPartyStepPhase,
    SwitchPartyStepResult, SWITCH_PARTY_CHOOSE_VIEW, SWITCH_PARTY_DEFAULT_LIST_SCAN_PAGES,
    SWITCH_PARTY_DEFAULT_OPEN_ATTEMPTS, SWITCH_PARTY_DEFAULT_OPEN_CHECKS_PER_ATTEMPT,
    SWITCH_PARTY_DELETE, SWITCH_PARTY_TASK_KEY, SWITCH_PARTY_WHITE_CONFIRM,
};
pub use common_job_teleport::{
    apply_teleport_move_map_center_observation, classify_teleport_move_map_post_drag_center,
    decide_teleport_move_map_center_after_drag, default_teleport_move_map_rule, plan_teleport,
    teleport_move_map_expected_move_len, teleport_move_map_false_positive_threshold,
    teleport_move_map_jump_distance, TeleportCountryPositionRule, TeleportExecutionConfig,
    TeleportExecutionPlan, TeleportFailurePolicy, TeleportMapPoint, TeleportMapRule,
    TeleportMoveMapCenterDecision, TeleportMoveMapCenterRejectReason,
    TeleportMoveMapPostDragObservation, TeleportMoveMapRule, TeleportNativeDependency,
    TeleportPlanKind, TeleportPreflightPlan, TeleportQuickTeleportRule, TeleportRetryRule,
    TeleportStep, TeleportStepAction, TeleportStepPhase, TeleportStepResult, TeleportTargetPlan,
    TELEPORT_TASK_KEY,
};
pub use common_job_walk_to_f::{
    plan_walk_to_f, WalkToFActionPress, WalkToFExecutionConfig, WalkToFExecutionPlan,
    WalkToFRetryRule, WalkToFStep, WalkToFStepAction, WalkToFStepCondition, WalkToFStepPhase,
    WalkToFStepResult, WALK_TO_F_DEFAULT_TIMEOUT_MS, WALK_TO_F_MOVE_START_DELAY_MS,
    WALK_TO_F_PICK_KEY, WALK_TO_F_RELEASE_GAP_MS, WALK_TO_F_RETRY_INTERVAL_MS, WALK_TO_F_TASK_KEY,
    WALK_TO_F_VK_F,
};
pub use common_job_wonderland_cycle::{
    plan_wonderland_cycle, WonderlandCycleExecutionConfig, WonderlandCycleExecutionPlan,
    WonderlandCycleLocators, WonderlandCycleRetryAction, WonderlandCycleRetryRule,
    WonderlandCycleStep, WonderlandCycleStepAction, WonderlandCycleStepCondition,
    WonderlandCycleStepPhase, WonderlandCycleStepResult, WONDERLAND_CYCLE_BACK_TEYVAT,
    WONDERLAND_CYCLE_BLACK_CONFIRM, WONDERLAND_CYCLE_CLOSE, WONDERLAND_CYCLE_TASK_KEY,
};

pub const RETURN_MAIN_UI_TASK_KEY: &str = "ReturnMainUi";
pub const RETURN_MAIN_UI_PAIMON_MENU: &str = "Common/Element:paimon_menu.png";
pub const RETURN_MAIN_UI_EXIT_DOOR: &str = "Common/Element:btn_exit_door.png";
pub const RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS: u8 = 8;
pub const RETURN_MAIN_UI_VK_ESCAPE: u16 = 0x1B;
pub const RETURN_MAIN_UI_VK_RETURN: u16 = 0x0D;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CommonJobExecutionPlan {
    ReturnMainUi(ReturnMainUiExecutionPlan),
    SetTime(SetTimeExecutionPlan),
    ChooseTalkOption(ChooseTalkOptionExecutionPlan),
    CheckRewards(CheckRewardsExecutionPlan),
    BlessingOfTheWelkinMoon(BlessingOfTheWelkinMoonExecutionPlan),
    ClaimBattlePassRewards(ClaimBattlePassRewardsExecutionPlan),
    ClaimEncounterPointsRewards(ClaimEncounterPointsRewardsExecutionPlan),
    ClaimMailRewards(ClaimMailRewardsExecutionPlan),
    CountInventoryItem(CountInventoryItemExecutionPlan),
    GoToAdventurersGuild(GoToAdventurersGuildExecutionPlan),
    GoToCraftingBench(GoToCraftingBenchExecutionPlan),
    GoToSereniteaPot(GoToSereniteaPotExecutionPlan),
    LowerHeadThenWalkTo(LowerHeadThenWalkToExecutionPlan),
    LinneaMining(LinneaMiningExecutionPlan),
    OneKeyExpedition(OneKeyExpeditionExecutionPlan),
    Relogin(ReloginExecutionPlan),
    ScanPickDrops(ScanPickDropsExecutionPlan),
    WonderlandCycle(WonderlandCycleExecutionPlan),
    SwitchParty(SwitchPartyExecutionPlan),
    Teleport(TeleportExecutionPlan),
    WalkToF(WalkToFExecutionPlan),
}

impl CommonJobExecutionPlan {
    pub fn task_key(&self) -> &str {
        match self {
            CommonJobExecutionPlan::ReturnMainUi(plan) => &plan.task_key,
            CommonJobExecutionPlan::SetTime(plan) => &plan.task_key,
            CommonJobExecutionPlan::ChooseTalkOption(plan) => &plan.task_key,
            CommonJobExecutionPlan::CheckRewards(plan) => &plan.task_key,
            CommonJobExecutionPlan::BlessingOfTheWelkinMoon(plan) => &plan.task_key,
            CommonJobExecutionPlan::ClaimBattlePassRewards(plan) => &plan.task_key,
            CommonJobExecutionPlan::ClaimEncounterPointsRewards(plan) => &plan.task_key,
            CommonJobExecutionPlan::ClaimMailRewards(plan) => &plan.task_key,
            CommonJobExecutionPlan::CountInventoryItem(plan) => &plan.task_key,
            CommonJobExecutionPlan::GoToAdventurersGuild(plan) => &plan.task_key,
            CommonJobExecutionPlan::GoToCraftingBench(plan) => &plan.task_key,
            CommonJobExecutionPlan::GoToSereniteaPot(plan) => &plan.task_key,
            CommonJobExecutionPlan::LowerHeadThenWalkTo(plan) => &plan.task_key,
            CommonJobExecutionPlan::LinneaMining(plan) => &plan.task_key,
            CommonJobExecutionPlan::OneKeyExpedition(plan) => &plan.task_key,
            CommonJobExecutionPlan::Relogin(plan) => &plan.task_key,
            CommonJobExecutionPlan::ScanPickDrops(plan) => &plan.task_key,
            CommonJobExecutionPlan::WonderlandCycle(plan) => &plan.task_key,
            CommonJobExecutionPlan::SwitchParty(plan) => &plan.task_key,
            CommonJobExecutionPlan::Teleport(plan) => &plan.task_key,
            CommonJobExecutionPlan::WalkToF(plan) => &plan.task_key,
        }
    }

    pub fn executor_ready(&self) -> bool {
        match self {
            CommonJobExecutionPlan::ReturnMainUi(plan) => plan.executor_ready,
            CommonJobExecutionPlan::SetTime(plan) => plan.executor_ready,
            CommonJobExecutionPlan::ChooseTalkOption(plan) => plan.executor_ready,
            CommonJobExecutionPlan::CheckRewards(plan) => plan.executor_ready,
            CommonJobExecutionPlan::BlessingOfTheWelkinMoon(plan) => plan.executor_ready,
            CommonJobExecutionPlan::ClaimBattlePassRewards(plan) => plan.executor_ready,
            CommonJobExecutionPlan::ClaimEncounterPointsRewards(plan) => plan.executor_ready,
            CommonJobExecutionPlan::ClaimMailRewards(plan) => plan.executor_ready,
            CommonJobExecutionPlan::CountInventoryItem(plan) => plan.executor_ready,
            CommonJobExecutionPlan::GoToAdventurersGuild(plan) => plan.executor_ready,
            CommonJobExecutionPlan::GoToCraftingBench(plan) => plan.executor_ready,
            CommonJobExecutionPlan::GoToSereniteaPot(plan) => plan.executor_ready,
            CommonJobExecutionPlan::LowerHeadThenWalkTo(plan) => plan.executor_ready,
            CommonJobExecutionPlan::LinneaMining(plan) => plan.executor_ready,
            CommonJobExecutionPlan::OneKeyExpedition(plan) => plan.executor_ready,
            CommonJobExecutionPlan::Relogin(plan) => plan.executor_ready,
            CommonJobExecutionPlan::ScanPickDrops(plan) => plan.executor_ready,
            CommonJobExecutionPlan::WonderlandCycle(plan) => plan.executor_ready,
            CommonJobExecutionPlan::SwitchParty(plan) => plan.executor_ready,
            CommonJobExecutionPlan::Teleport(plan) => plan.executor_ready,
            CommonJobExecutionPlan::WalkToF(plan) => plan.executor_ready,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobStepPhase {
    Setup,
    RetryLoop,
    TimePanel,
    TimeDial,
    Animation,
    Cleanup,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonJobStepCondition {
    Always,
    WhenMainUiNotDetected,
    WhenExitDoorDetected,
    WhenSkipAnimationRequested,
    WhenSkipAnimationNotResolved,
    AfterTimeAdjustment,
    AfterRetryLimit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CommonJobStepAction {
    CommonJob {
        task_key: String,
        config: Option<Value>,
    },
    Input {
        events: Vec<InputEvent>,
    },
    Page {
        command: BvPageCommand,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommonJobStep {
    pub phase: CommonJobStepPhase,
    pub condition: CommonJobStepCondition,
    pub attempt: Option<u8>,
    pub label: String,
    pub action: CommonJobStepAction,
}

impl CommonJobStep {
    pub(super) fn new(
        phase: CommonJobStepPhase,
        label: impl Into<String>,
        action: CommonJobStepAction,
    ) -> Self {
        Self {
            phase,
            condition: CommonJobStepCondition::Always,
            attempt: None,
            label: label.into(),
            action,
        }
    }

    pub(super) fn conditional(
        phase: CommonJobStepPhase,
        condition: CommonJobStepCondition,
        label: impl Into<String>,
        action: CommonJobStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            attempt: None,
            label: label.into(),
            action,
        }
    }

    pub(super) fn for_attempt(
        attempt: u8,
        condition: CommonJobStepCondition,
        label: impl Into<String>,
        action: CommonJobStepAction,
    ) -> Self {
        Self {
            phase: CommonJobStepPhase::RetryLoop,
            condition,
            attempt: Some(attempt),
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnMainUiExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub max_escape_attempts: u8,
    pub steps: Vec<CommonJobStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReturnMainUiExecutionConfig {
    pub capture_size: Size,
    pub max_escape_attempts: u8,
}

impl Default for ReturnMainUiExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            max_escape_attempts: RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
        }
    }
}

impl ReturnMainUiExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.max_escape_attempts == 0 {
            config.max_escape_attempts = RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS;
        }
        config
    }
}

pub fn plan_common_job(
    task_key: &str,
    config: Option<&Value>,
) -> Result<Option<CommonJobExecutionPlan>> {
    match task_key {
        RETURN_MAIN_UI_TASK_KEY => {
            let config = ReturnMainUiExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ReturnMainUi(
                plan_return_main_ui(config.capture_size, config.max_escape_attempts)?,
            )))
        }
        SET_TIME_TASK_KEY => {
            let config = SetTimeExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::SetTime(plan_set_time(
                config.capture_size,
                config.hour,
                config.minute,
                config.skip,
            )?)))
        }
        CHOOSE_TALK_OPTION_TASK_KEY => {
            let config = ChooseTalkOptionExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ChooseTalkOption(
                plan_choose_talk_option(
                    config.capture_size,
                    config.option,
                    config.skip_times,
                    config.is_orange,
                )?,
            )))
        }
        CHECK_REWARDS_TASK_KEY => {
            let config = CheckRewardsExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::CheckRewards(
                plan_check_rewards(config)?,
            )))
        }
        BLESSING_WELKIN_TASK_KEY => {
            let config = BlessingOfTheWelkinMoonExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::BlessingOfTheWelkinMoon(
                plan_blessing_of_the_welkin_moon(
                    config.capture_size,
                    config.max_iterations,
                    config.stable_clear_count,
                )?,
            )))
        }
        CLAIM_BATTLE_PASS_REWARDS_TASK_KEY => {
            let config = ClaimBattlePassRewardsExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ClaimBattlePassRewards(
                plan_claim_battle_pass_rewards(config.capture_size, config.claim_text_patterns)?,
            )))
        }
        CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY => {
            let config = ClaimEncounterPointsRewardsExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ClaimEncounterPointsRewards(
                plan_claim_encounter_points_rewards(
                    config.capture_size,
                    config.commissions_text,
                    config.max_open_retries,
                )?,
            )))
        }
        CLAIM_MAIL_REWARDS_TASK_KEY => {
            let config = ClaimMailRewardsExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ClaimMailRewards(
                plan_claim_mail_rewards(config.capture_size)?,
            )))
        }
        COUNT_INVENTORY_ITEM_TASK_KEY => {
            let config = CountInventoryItemExecutionConfig::from_value(config)?;
            Ok(Some(CommonJobExecutionPlan::CountInventoryItem(
                plan_count_inventory_item(config)?,
            )))
        }
        GO_TO_ADVENTURERS_GUILD_TASK_KEY => {
            let config = GoToAdventurersGuildExecutionConfig::from_value(config);
            let localized_texts = config.localized_texts();
            Ok(Some(CommonJobExecutionPlan::GoToAdventurersGuild(
                plan_go_to_adventurers_guild(
                    config.capture_size,
                    config.country,
                    config.daily_reward_party_name,
                    config.only_do_once,
                    localized_texts,
                    config.interact_vk,
                )?,
            )))
        }
        GO_TO_CRAFTING_BENCH_TASK_KEY => {
            let config = GoToCraftingBenchExecutionConfig::from_value(config);
            let localized_texts = config.localized_texts();
            Ok(Some(CommonJobExecutionPlan::GoToCraftingBench(
                plan_go_to_crafting_bench(
                    config.capture_size,
                    config.country,
                    localized_texts,
                    config.min_resin_to_keep,
                    config.interact_vk,
                )?,
            )))
        }
        GO_TO_SERENITEA_POT_TASK_KEY => {
            let config = GoToSereniteaPotExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::GoToSereniteaPot(
                plan_go_to_serenitea_pot(config)?,
            )))
        }
        LOWER_HEAD_THEN_WALK_TO_TASK_KEY => {
            let config = LowerHeadThenWalkToExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::LowerHeadThenWalkTo(
                plan_lower_head_then_walk_to(config)?,
            )))
        }
        LINNEA_MINING_TASK_KEY => {
            let config = LinneaMiningExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::LinneaMining(
                plan_linnea_mining(config),
            )))
        }
        ONE_KEY_EXPEDITION_TASK_KEY => {
            let config = OneKeyExpeditionExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::OneKeyExpedition(
                plan_one_key_expedition(config.capture_size)?,
            )))
        }
        SCAN_PICK_DROPS_TASK_KEY => {
            let config = ScanPickDropsExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::ScanPickDrops(
                plan_scan_pick_drops(config)?,
            )))
        }
        RELOGIN_TASK_KEY => {
            let config = ReloginExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::Relogin(plan_relogin(
                config.capture_size,
            )?)))
        }
        WONDERLAND_CYCLE_TASK_KEY => {
            let config = WonderlandCycleExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::WonderlandCycle(
                plan_wonderland_cycle(config.capture_size)?,
            )))
        }
        SWITCH_PARTY_TASK_KEY => {
            let config = SwitchPartyExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::SwitchParty(
                plan_switch_party(config.capture_size, config.party_name)?,
            )))
        }
        TELEPORT_TASK_KEY => {
            let config = TeleportExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::Teleport(plan_teleport(
                config,
            )?)))
        }
        WALK_TO_F_TASK_KEY => {
            let config = WalkToFExecutionConfig::from_value(config);
            Ok(Some(CommonJobExecutionPlan::WalkToF(plan_walk_to_f(
                config,
            )?)))
        }
        _ => Ok(None),
    }
}

pub fn plan_return_main_ui(
    capture_size: Size,
    max_escape_attempts: u8,
) -> Result<ReturnMainUiExecutionPlan> {
    let max_escape_attempts = if max_escape_attempts == 0 {
        RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS
    } else {
        max_escape_attempts
    };
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let mut steps = Vec::new();

    steps.push(CommonJobStep::new(
        CommonJobStepPhase::Setup,
        "log return main UI start",
        CommonJobStepAction::Log {
            message: "start ReturnMainUi common job plan".to_string(),
        },
    ));
    steps.push(CommonJobStep::new(
        CommonJobStepPhase::Setup,
        "check whether already in main UI",
        CommonJobStepAction::Locator {
            locator: image_locator(
                &page,
                RETURN_MAIN_UI_PAIMON_MENU,
                Some(top_left_quarter_rect(capture_size)?),
                0.8,
                BvLocatorOperation::IsExist,
                Some(1_000),
            )?,
        },
    ));

    for attempt in 1..=max_escape_attempts {
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenMainUiNotDetected,
            "press Escape",
            CommonJobStepAction::Input {
                events: InputSequence::new()
                    .key_press(RETURN_MAIN_UI_VK_ESCAPE)
                    .events()
                    .to_vec(),
            },
        ));
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenMainUiNotDetected,
            "wait after Escape",
            CommonJobStepAction::Page {
                command: task_vision_result(page.wait(900))?,
            },
        ));
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenMainUiNotDetected,
            "detect exit-door button",
            CommonJobStepAction::Locator {
                locator: image_locator(
                    &page,
                    RETURN_MAIN_UI_EXIT_DOOR,
                    None,
                    0.8,
                    BvLocatorOperation::IsExist,
                    Some(1_000),
                )?,
            },
        ));
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenExitDoorDetected,
            "click exit-door button",
            CommonJobStepAction::Locator {
                locator: image_locator(
                    &page,
                    RETURN_MAIN_UI_EXIT_DOOR,
                    None,
                    0.8,
                    BvLocatorOperation::Click,
                    Some(1_000),
                )?,
            },
        ));
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenExitDoorDetected,
            "wait after exit-door click",
            CommonJobStepAction::Page {
                command: task_vision_result(page.wait(5_000))?,
            },
        ));
        steps.push(CommonJobStep::for_attempt(
            attempt,
            CommonJobStepCondition::WhenMainUiNotDetected,
            "check main UI after retry",
            CommonJobStepAction::Locator {
                locator: image_locator(
                    &page,
                    RETURN_MAIN_UI_PAIMON_MENU,
                    Some(top_left_quarter_rect(capture_size)?),
                    0.8,
                    BvLocatorOperation::IsExist,
                    Some(1_000),
                )?,
            },
        ));
    }

    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Fallback,
        CommonJobStepCondition::AfterRetryLimit,
        "wait before fallback confirm",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(500))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Fallback,
        CommonJobStepCondition::AfterRetryLimit,
        "press Return fallback",
        CommonJobStepAction::Input {
            events: InputSequence::new()
                .key_press(RETURN_MAIN_UI_VK_RETURN)
                .events()
                .to_vec(),
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Fallback,
        CommonJobStepCondition::AfterRetryLimit,
        "wait after fallback confirm",
        CommonJobStepAction::Page {
            command: task_vision_result(page.wait(500))?,
        },
    ));
    steps.push(CommonJobStep::conditional(
        CommonJobStepPhase::Fallback,
        CommonJobStepCondition::AfterRetryLimit,
        "press Escape fallback",
        CommonJobStepAction::Input {
            events: InputSequence::new()
                .key_press(RETURN_MAIN_UI_VK_ESCAPE)
                .events()
                .to_vec(),
        },
    ));

    Ok(ReturnMainUiExecutionPlan {
        task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        display_name: "Return Main UI".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        max_escape_attempts,
        steps,
        notes: "Legacy ReturnMainUi loop is represented as a capture/input/locator plan with a live template/input executor boundary; revive-prompt exclusion remains pending.".to_string(),
    })
}

pub(super) fn task_vision_result<T>(result: bgi_vision::Result<T>) -> Result<T> {
    result.map_err(|error| TaskError::VisionPlan(error.to_string()))
}

pub(super) fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = task_vision_result(BvImage::new(asset))?;
    let locator = task_vision_result(page.locator_for_image(&image, roi, threshold))?;
    Ok(locator.plan(operation, timeout_ms))
}

fn top_left_quarter_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}
