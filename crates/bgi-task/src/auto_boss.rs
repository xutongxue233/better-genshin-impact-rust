use bgi_core::AutoBossConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::task_params::{combat_strategy_path, AutoBossParam, AUTO_STRATEGY_NAME};
use crate::{Result, TaskError};

pub const AUTO_BOSS_TASK_KEY: &str = "AutoBoss";
pub const AUTO_BOSS_DISPLAY_NAME: &str = "自动首领讨伐";
pub const AUTO_BOSS_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_BOSS_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_BOSS_ORIGINAL_RESIN_COST: i32 = 40;
pub const AUTO_BOSS_ORIGINAL_RESIN_RECOVERY_INTERVAL_MINUTES: u64 = 8;
pub const AUTO_BOSS_MAX_QUICK_USE_QUANTITY: i32 = 20;
pub const AUTO_BOSS_PATHING_ASSET_DIR: &str = "GameTask/AutoBoss/Assets/Pathing";
pub const AUTO_BOSS_ORIGINAL_RESIN_TOP_ICON_ASSET: &str = "AutoBoss:original_resin_top_icon.png";
pub const AUTO_BOSS_REWARD_BOX_ASSET: &str = "AutoBoss:box.png";
pub const AUTO_BOSS_OPEN_RESIN_SUPPLEMENT_PANE_BUTTON_ASSET: &str =
    "AutoBoss:open_resin_supplement_pane_button.png";
pub const AUTO_BOSS_TRANSIENT_RESIN_ASSET: &str = "AutoBoss:transient_resin_in_supplement_pane.png";
pub const AUTO_BOSS_FRAGILE_RESIN_ASSET: &str = "AutoBoss:fragile_resin_in_supplement_pane.png";
pub const AUTO_BOSS_INCREASE_RESIN_QUANTITY_BUTTON_ASSET: &str =
    "AutoBoss:increase_resin_usage_quantity_button.png";

pub const AUTO_BOSS_TALK_TO_START_BOSSES: &[&str] =
    &["歌裴莉娅的葬送", "科培琉司的劫罚", "纯水精灵", "重拳出击鸭"];

pub const AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES: &[&str] =
    &["蕴光月守宫", "超重型陆巡舰·机动战垒", "蕴光月幻蝶"];

const AUTO_BOSS_COUNTRY_TO_BOSSES: &[(&str, &[&str])] = &[
    ("蒙德", &["急冻树", "无相之雷", "守望者·堕天"]),
    (
        "璃月",
        &[
            "爆炎树",
            "纯水精灵",
            "古岩龙蜥",
            "无相之岩",
            "遗迹巨蛇",
            "隐山猊兽",
        ],
    ),
    (
        "稻妻",
        &[
            "无相之火",
            "恒常机关阵列",
            "雷音权现",
            "魔偶剑鬼",
            "无相之水",
        ],
    ),
    (
        "须弥",
        &[
            "掣电树",
            "半永恒统辖矩阵",
            "翠翎恐蕈",
            "风蚀沙虫",
            "无相之草",
            "深罪浸礼者",
            "兆载永劫龙兽",
        ],
    ),
    (
        "枫丹",
        &[
            "歌裴莉娅的葬送",
            "科培琉司的劫罚",
            "实验性场力发生装置",
            "魔像督军",
            "千年珍珠骏麟",
            "水形幻人",
            "铁甲熔火帝皇",
        ],
    ),
    (
        "纳塔",
        &[
            "金焰绒翼龙暴君",
            "灵觉隐修的迷者",
            "秘源机兵·构型械",
            "秘源机兵·统御械",
            "熔岩辉龙像",
            "深邃摹结株",
            "贪食匿叶龙山王",
        ],
    ),
    (
        "挪德卡莱",
        &[
            "蕴光月守宫",
            "深黯魇语之主",
            "超重型陆巡舰·机动战垒",
            "霜夜巡天灵主",
            "蕴光月幻蝶",
            "重拳出击鸭",
        ],
    ),
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoBossParam,
    pub boss_data: AutoBossDataPlan,
    pub validation_rule: AutoBossValidationRule,
    pub startup_rule: AutoBossStartupRule,
    pub loop_rule: AutoBossLoopRule,
    pub pathing_rule: AutoBossPathingRule,
    pub resin_rule: AutoBossResinRule,
    pub supplemental_resin_rule: AutoBossSupplementalResinRule,
    pub combat_rule: AutoBossCombatRule,
    pub reward_navigation_rule: AutoBossRewardNavigationRule,
    pub reward_rule: AutoBossRewardRule,
    pub reposition_rule: AutoBossRepositionRule,
    pub locators: AutoBossLocators,
    pub steps: Vec<AutoBossTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoBossParam,
    pub auto_boss_config: AutoBossConfig,
}

impl Default for AutoBossExecutionConfig {
    fn default() -> Self {
        let auto_boss_config = AutoBossConfig::default();
        let mut param = AutoBossParam::default();
        apply_auto_boss_config(&mut param, &auto_boss_config);
        Self {
            capture_size: Size::new(
                AUTO_BOSS_DEFAULT_CAPTURE_WIDTH,
                AUTO_BOSS_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            param,
            auto_boss_config,
        }
    }
}

impl AutoBossExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        if let Some(asset_scale) = f64_member(value, ["assetScale", "AssetScale", "asset_scale"]) {
            config.asset_scale = asset_scale.max(0.0);
        }

        let auto_boss_value = value
            .get("autoBossConfig")
            .or_else(|| value.get("AutoBossConfig"))
            .or_else(|| value.get("auto_boss_config"))
            .unwrap_or(value);
        config.auto_boss_config =
            serde_json::from_value(auto_boss_value.clone()).unwrap_or_default();

        let strategy_name = string_member_from(
            value
                .get("param")
                .or_else(|| value.get("Param"))
                .or_else(|| value.get("autoBossParam"))
                .or_else(|| value.get("AutoBossParam"))
                .or_else(|| value.get("auto_boss_param")),
            value,
            &["strategyName", "StrategyName", "strategy_name"],
        )
        .or_else(|| {
            Some(config.auto_boss_config.strategy_name.clone()).filter(|value| !value.is_empty())
        });
        let mut param = AutoBossParam::new(strategy_name.as_deref());
        apply_auto_boss_config(&mut param, &config.auto_boss_config);
        overlay_auto_boss_param_members(&mut param, value);
        if let Some(param_value) = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("autoBossParam"))
            .or_else(|| value.get("AutoBossParam"))
            .or_else(|| value.get("auto_boss_param"))
        {
            overlay_auto_boss_param_members(&mut param, param_value);
        }
        if param.combat_strategy_path.trim().is_empty() {
            param.combat_strategy_path = combat_strategy_path(Some(&param.strategy_name));
        }
        config.param = param;
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossDataPlan {
    pub country_to_bosses: Vec<AutoBossCountryBosses>,
    pub supported_boss_count: usize,
    pub selected_boss_supported: bool,
    pub selected_boss_country: Option<String>,
    pub selected_boss_talk_to_start: bool,
    pub selected_boss_no_pathing_support: bool,
    pub talk_to_start_bosses: Vec<String>,
    pub no_pathing_support_bosses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossCountryBosses {
    pub country: String,
    pub bosses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossValidationRule {
    pub requires_boss_name: bool,
    pub requires_supported_boss: bool,
    pub requires_non_negative_revive_retry_count: bool,
    pub requires_positive_run_count_when_specified: bool,
    pub supplemental_resin_only_allowed_with_specified_run_count: bool,
    pub requires_existing_combat_strategy_file_or_directory: bool,
    pub requires_existing_route_files: Vec<String>,
    pub requires_16_to_9_resolution: bool,
    pub warns_below_1920x1080: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossStartupRule {
    pub parses_combat_script_bag_before_start: bool,
    pub logs_screen_resolution: bool,
    pub sends_start_notification: bool,
    pub sends_end_notification: bool,
    pub outer_retry_exception_respects_revive_retry_count: bool,
    pub retry_delay_ms: u64,
    pub releases_all_keys_on_finish: bool,
    pub releases_left_mouse_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossLoopRule {
    pub prepares_main_ui_before_loop: bool,
    pub switches_party_when_team_name_configured: bool,
    pub specified_run_count_stops_after_rewards: bool,
    pub unspecified_run_count_runs_until_resin_exhausted: bool,
    pub reward_count_increments_only_after_successful_reward: bool,
    pub return_to_statue_after_each_round_option: bool,
    pub statue_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossPathingRule {
    pub pathing_asset_directory: String,
    pub required_route_files: Vec<String>,
    pub first_navigation_files: Vec<String>,
    pub no_pathing_support_uses_force_teleport_and_key_mouse: bool,
    pub normal_boss_uses_go_to_route: bool,
    pub pathing_party_skip_party_switch: bool,
    pub pathing_party_auto_fight_enabled: bool,
    pub runs_return_main_ui_before_pathing_file: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossResinRule {
    pub original_resin_cost: i32,
    pub resin_recovery_interval_minutes: u64,
    pub precheck_opens_big_map: bool,
    pub precheck_returns_main_ui_finally: bool,
    pub resin_icon_search_rect: Rect,
    pub resin_count_ocr_rect_offset_from_icon_right: Rect,
    pub recovery_detail_rect_offset_from_icon: Rect,
    pub precheck_failure_falls_back_to_reward_prompt: bool,
    pub insufficient_resin_stops_when_not_specified_run_count: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossSupplementalResinRule {
    pub enabled_resin_options: Vec<AutoBossSupplementalResinOption>,
    pub target_quantity_formula: String,
    pub max_quick_use_quantity: i32,
    pub title_rect: Rect,
    pub open_button_roi: Rect,
    pub icon_roi: Rect,
    pub selected_name_rect: Rect,
    pub quick_use_title_rect: Rect,
    pub quick_use_available_count_rect: Rect,
    pub quick_use_quantity_rect: Rect,
    pub increase_button_roi: Rect,
    pub quick_use_retry_base_attempts: i32,
    pub quick_use_retry_multiplier: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossSupplementalResinOption {
    pub name: String,
    pub asset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossCombatRule {
    pub initializes_team_with_retry: bool,
    pub team_initialization_retry_attempts: u64,
    pub team_initialization_retry_interval_ms: u64,
    pub switches_to_first_script_avatar_before_fight: bool,
    pub switch_avatar_sleep_ms: u64,
    pub auto_fight_finish_detection_enabled: bool,
    pub pick_drops_after_fight_enabled: bool,
    pub kazuha_pickup_enabled: bool,
    pub qin_double_pickup_enabled: bool,
    pub exp_based_pickup_enabled: bool,
    pub battle_threshold_for_loot: i32,
    pub only_pick_elite_drops_mode: String,
    pub normal_end_exception_is_logged: bool,
    pub calls_combat_scenes_after_task: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossRewardNavigationRule {
    pub navigation_timeout_seconds: u64,
    pub reward_prompt_ocr_rect: Rect,
    pub reward_box_kept_between_screen_x_ratio_min: f64,
    pub reward_box_kept_between_screen_x_ratio_max: f64,
    pub camera_missing_icon_move_x: i32,
    pub camera_retry_interval_ms: u64,
    pub climb_detection_rect: Rect,
    pub climb_escape_drop_delay_ms: u64,
    pub climb_escape_left_hold_ms: u64,
    pub move_forward_burst_ms: u64,
    pub jump_every_forward_bursts: u64,
    pub post_jump_delay_ms: u64,
    pub post_forward_release_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossRewardRule {
    pub interact_reward_rect: Rect,
    pub interact_wait_ms: u64,
    pub use_original_resin_rect: Rect,
    pub use_original_resin_timeout_ms: u64,
    pub post_use_original_resin_delay_ms: u64,
    pub supplement_prompt_wait_ms: u64,
    pub reward_recognition_enabled: bool,
    pub reward_ready_close_rect: Rect,
    pub reward_ready_retry_attempts: u64,
    pub reward_ready_retry_interval_ms: u64,
    pub close_result_retry_attempts: u64,
    pub close_result_retry_interval_ms: u64,
    pub click_center_after_attempt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossRepositionRule {
    pub talk_to_start_uses_after_fight_quick_route: bool,
    pub no_pathing_support_reruns_special_navigation: bool,
    pub normal_boss_replays_last_route_position: bool,
    pub normal_boss_post_reposition_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossLocators {
    pub original_resin_top_icon: AutoBossTemplateLocator,
    pub reward_box: AutoBossTemplateLocator,
    pub open_resin_supplement_pane_button: AutoBossTemplateLocator,
    pub transient_resin: AutoBossTemplateLocator,
    pub fragile_resin: AutoBossTemplateLocator,
    pub increase_resin_usage_quantity_button: AutoBossTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoBossTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoBossTaskStep {
    pub phase: AutoBossTaskPhase,
    pub action: AutoBossTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossTaskPhase {
    Startup,
    Prepare,
    Resin,
    Navigation,
    Combat,
    Reward,
    Reposition,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossTaskAction {
    ValidateAndParseCombatStrategy,
    LogScreenResolution,
    ReturnMainUiAndSwitchParty,
    CheckOriginalResin,
    UseSupplementalResinWhenAllowed,
    NavigateToBoss,
    RunAutoFight,
    MoveToRewardFlower,
    TakeReward,
    RecognizeRewardWhenEnabled,
    RepositionForNextRound,
    ReleaseInputsAndNotifyEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossExecutionStatus {
    Completed,
    StartupFailed,
    PrepareFailed,
    ResinExhausted,
    RewardSkipped,
    NavigationFailed,
    CombatFailed,
    RewardFailed,
    RepositionFailed,
    Cancelled,
    RoundLimitReached,
    CleanupFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossRuntimeActionStatus {
    Succeeded,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossRuntimeActionKind {
    Startup,
    PrepareRound,
    CheckOriginalResin,
    UseSupplementalResin,
    SkipReward,
    NavigateToBoss,
    RunAutoFight,
    MoveToRewardFlower,
    TakeReward,
    RecognizeReward,
    RepositionForNextRound,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossNavigationKind {
    FirstNavigation,
    RepositionForNextRound,
    ReturnToStatue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBossSkipReason {
    InsufficientResin,
    SupplementalResinUnavailable,
    RewardDisabledByRuntime,
    RewardPromptMissing,
    Cancelled,
    RoundLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRuntimeRoundContext {
    pub round_index: u32,
    pub boss_name: String,
    pub target_reward_count: Option<u32>,
    pub claimed_rewards: u32,
    pub is_first_round: bool,
    pub should_claim_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossNavigationRequest {
    pub kind: AutoBossNavigationKind,
    pub boss_name: String,
    pub route_files: Vec<String>,
    pub no_pathing_support: bool,
    pub talk_to_start: bool,
    pub return_to_statue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossStartupOutcome {
    pub completed: bool,
    pub combat_strategy_parsed: bool,
    pub screen_resolution_logged: bool,
    pub start_notification_sent: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossPrepareOutcome {
    pub completed: bool,
    pub main_ui_ready: bool,
    pub party_switched: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossResinCheckOutcome {
    pub precheck_succeeded: bool,
    pub original_resin: Option<i32>,
    pub can_claim_reward: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossSupplementalResinOutcome {
    pub attempted: bool,
    pub used_transient_resin: i32,
    pub used_fragile_resin: i32,
    pub original_resin_after: Option<i32>,
    pub can_claim_reward: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossNavigationOutcome {
    pub completed: bool,
    pub teleport_used: bool,
    pub pathing_used: bool,
    pub route_files: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossCombatOutcome {
    pub completed: bool,
    pub victory: bool,
    pub normal_end: bool,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRewardNavigationOutcome {
    pub completed: bool,
    pub reward_prompt_found: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRewardOutcome {
    pub claimed: bool,
    pub original_resin_spent: i32,
    pub skip_reason: Option<AutoBossSkipReason>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRewardRecognitionOutcome {
    pub attempted: bool,
    pub recognized: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRepositionOutcome {
    pub completed: bool,
    pub returned_to_statue: bool,
    pub route_files: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossCleanupOutcome {
    pub completed: bool,
    pub released_all_keys: bool,
    pub released_left_mouse: bool,
    pub end_notification_sent: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoBossRuntimeActionOutcome {
    Startup(AutoBossStartupOutcome),
    Prepare(AutoBossPrepareOutcome),
    ResinCheck(AutoBossResinCheckOutcome),
    SupplementalResin(AutoBossSupplementalResinOutcome),
    Navigation(AutoBossNavigationOutcome),
    Combat(AutoBossCombatOutcome),
    RewardNavigation(AutoBossRewardNavigationOutcome),
    Reward(AutoBossRewardOutcome),
    RewardRecognition(AutoBossRewardRecognitionOutcome),
    Reposition(AutoBossRepositionOutcome),
    Cleanup(AutoBossCleanupOutcome),
    Skipped(AutoBossSkipReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossRuntimeActionReport {
    pub phase: AutoBossTaskPhase,
    pub action_kind: AutoBossRuntimeActionKind,
    pub status: AutoBossRuntimeActionStatus,
    pub round_index: Option<u32>,
    pub detail: String,
    pub outcome: AutoBossRuntimeActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossSkippedStep {
    pub action_kind: AutoBossRuntimeActionKind,
    pub round_index: Option<u32>,
    pub reason: AutoBossSkipReason,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossExecutorState {
    pub startup_completed: bool,
    pub current_round: u32,
    pub target_reward_count: Option<u32>,
    pub rounds_started: u32,
    pub navigation_attempts: u32,
    pub fights_attempted: u32,
    pub fights_succeeded: u32,
    pub combat_failures: u32,
    pub reward_navigation_attempts: u32,
    pub rewards_claimed: u32,
    pub rewards_skipped: u32,
    pub reward_recognition_attempts: u32,
    pub original_resin_spent: i32,
    pub supplemental_resin_used: i32,
    pub last_observed_original_resin: Option<i32>,
    pub stopped_by_resin: bool,
    pub cancelled: bool,
    pub cleanup_completed: bool,
    pub last_skip_reason: Option<AutoBossSkipReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBossExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoBossExecutionStatus,
    pub state: AutoBossExecutorState,
    pub executed_actions: Vec<AutoBossRuntimeActionReport>,
    pub skipped_steps: Vec<AutoBossSkippedStep>,
}

pub trait AutoBossRuntime {
    fn start_auto_boss(&mut self, plan: &AutoBossExecutionPlan) -> Result<AutoBossStartupOutcome>;

    fn prepare_auto_boss_round(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossPrepareOutcome>;

    fn check_auto_boss_resin(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossResinCheckOutcome>;

    fn use_auto_boss_supplemental_resin(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
        resin_check: &AutoBossResinCheckOutcome,
    ) -> Result<AutoBossSupplementalResinOutcome>;

    fn navigate_auto_boss_to_boss(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
        request: &AutoBossNavigationRequest,
    ) -> Result<AutoBossNavigationOutcome>;

    fn run_auto_boss_fight(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossCombatOutcome>;

    fn move_auto_boss_to_reward(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossRewardNavigationOutcome>;

    fn take_auto_boss_reward(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossRewardOutcome>;

    fn recognize_auto_boss_reward(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
    ) -> Result<AutoBossRewardRecognitionOutcome>;

    fn reposition_auto_boss_for_next_round(
        &mut self,
        plan: &AutoBossExecutionPlan,
        context: &AutoBossRuntimeRoundContext,
        request: &AutoBossNavigationRequest,
    ) -> Result<AutoBossRepositionOutcome>;

    fn cleanup_auto_boss(&mut self, plan: &AutoBossExecutionPlan)
        -> Result<AutoBossCleanupOutcome>;

    fn is_auto_boss_cancelled(&mut self) -> bool {
        false
    }
}

pub fn plan_auto_boss(
    working_directory: impl AsRef<Path>,
    config: AutoBossExecutionConfig,
) -> Result<AutoBossExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let mut param = config.param;
    normalize_auto_boss_param_strategy(&mut param);
    validate_auto_boss_param(working_directory, &param)?;

    let required_route_files = auto_boss_required_route_files(&param.boss_name);
    let first_navigation_files = auto_boss_first_navigation_files(&param.boss_name);

    Ok(AutoBossExecutionPlan {
        task_key: AUTO_BOSS_TASK_KEY.to_string(),
        display_name: AUTO_BOSS_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        boss_data: auto_boss_data_plan(&param.boss_name),
        validation_rule: AutoBossValidationRule {
            requires_boss_name: true,
            requires_supported_boss: true,
            requires_non_negative_revive_retry_count: true,
            requires_positive_run_count_when_specified: true,
            supplemental_resin_only_allowed_with_specified_run_count: true,
            requires_existing_combat_strategy_file_or_directory: true,
            requires_existing_route_files: required_route_files.clone(),
            requires_16_to_9_resolution: true,
            warns_below_1920x1080: true,
        },
        startup_rule: AutoBossStartupRule {
            parses_combat_script_bag_before_start: true,
            logs_screen_resolution: true,
            sends_start_notification: true,
            sends_end_notification: true,
            outer_retry_exception_respects_revive_retry_count: true,
            retry_delay_ms: 2_000,
            releases_all_keys_on_finish: true,
            releases_left_mouse_on_finish: true,
        },
        loop_rule: AutoBossLoopRule {
            prepares_main_ui_before_loop: true,
            switches_party_when_team_name_configured: !param.team_name.trim().is_empty(),
            specified_run_count_stops_after_rewards: param.specify_run_count,
            unspecified_run_count_runs_until_resin_exhausted: !param.specify_run_count,
            reward_count_increments_only_after_successful_reward: true,
            return_to_statue_after_each_round_option: param.return_to_statue_after_each_round,
            statue_delay_ms: 3_000,
        },
        pathing_rule: AutoBossPathingRule {
            pathing_asset_directory: AUTO_BOSS_PATHING_ASSET_DIR.to_string(),
            required_route_files,
            first_navigation_files,
            no_pathing_support_uses_force_teleport_and_key_mouse: auto_boss_is_no_pathing_support(
                &param.boss_name,
            ),
            normal_boss_uses_go_to_route: !auto_boss_is_no_pathing_support(&param.boss_name),
            pathing_party_skip_party_switch: true,
            pathing_party_auto_fight_enabled: false,
            runs_return_main_ui_before_pathing_file: true,
        },
        resin_rule: AutoBossResinRule {
            original_resin_cost: AUTO_BOSS_ORIGINAL_RESIN_COST,
            resin_recovery_interval_minutes: AUTO_BOSS_ORIGINAL_RESIN_RECOVERY_INTERVAL_MINUTES,
            precheck_opens_big_map: true,
            precheck_returns_main_ui_finally: true,
            resin_icon_search_rect: Rect {
                x: 1200,
                y: 25,
                width: 250,
                height: 50,
            },
            resin_count_ocr_rect_offset_from_icon_right: Rect {
                x: 25,
                y: 37,
                width: 120,
                height: 24,
            },
            recovery_detail_rect_offset_from_icon: Rect {
                x: -13,
                y: 29,
                width: 220,
                height: 150,
            },
            precheck_failure_falls_back_to_reward_prompt: true,
            insufficient_resin_stops_when_not_specified_run_count: true,
        },
        supplemental_resin_rule: AutoBossSupplementalResinRule {
            enabled_resin_options: supplemental_resin_options(&param),
            target_quantity_formula: "max(0, (resin_limit - resin_count) / 60)".to_string(),
            max_quick_use_quantity: AUTO_BOSS_MAX_QUICK_USE_QUANTITY,
            title_rect: Rect {
                x: 834,
                y: 247,
                width: 256,
                height: 60,
            },
            open_button_roi: Rect {
                x: 1200,
                y: 25,
                width: 250,
                height: 50,
            },
            icon_roi: Rect {
                x: 644,
                y: 378,
                width: 620,
                height: 192,
            },
            selected_name_rect: Rect {
                x: 906,
                y: 587,
                width: 110,
                height: 31,
            },
            quick_use_title_rect: Rect {
                x: 875,
                y: 269,
                width: 184,
                height: 63,
            },
            quick_use_available_count_rect: Rect {
                x: 1191,
                y: 633,
                width: 72,
                height: 29,
            },
            quick_use_quantity_rect: Rect {
                x: 915,
                y: 540,
                width: 93,
                height: 81,
            },
            increase_button_roi: Rect {
                x: 1265,
                y: 620,
                width: 59,
                height: 55,
            },
            quick_use_retry_base_attempts: 6,
            quick_use_retry_multiplier: 3,
        },
        combat_rule: AutoBossCombatRule {
            initializes_team_with_retry: true,
            team_initialization_retry_attempts: 5,
            team_initialization_retry_interval_ms: 1_000,
            switches_to_first_script_avatar_before_fight: true,
            switch_avatar_sleep_ms: 200,
            auto_fight_finish_detection_enabled: true,
            pick_drops_after_fight_enabled: false,
            kazuha_pickup_enabled: false,
            qin_double_pickup_enabled: false,
            exp_based_pickup_enabled: false,
            battle_threshold_for_loot: -1,
            only_pick_elite_drops_mode: "DisableAutoPickupForNonElite".to_string(),
            normal_end_exception_is_logged: true,
            calls_combat_scenes_after_task: true,
        },
        reward_navigation_rule: AutoBossRewardNavigationRule {
            navigation_timeout_seconds: 15,
            reward_prompt_ocr_rect: Rect {
                x: 1210,
                y: 300,
                width: 200,
                height: 400,
            },
            reward_box_kept_between_screen_x_ratio_min: 0.45,
            reward_box_kept_between_screen_x_ratio_max: 0.55,
            camera_missing_icon_move_x: 200,
            camera_retry_interval_ms: 250,
            climb_detection_rect: Rect {
                x: 1686,
                y: 1030,
                width: 60,
                height: 23,
            },
            climb_escape_drop_delay_ms: 1_000,
            climb_escape_left_hold_ms: 800,
            move_forward_burst_ms: 1_000,
            jump_every_forward_bursts: 2,
            post_jump_delay_ms: 100,
            post_forward_release_delay_ms: 200,
        },
        reward_rule: AutoBossRewardRule {
            interact_reward_rect: Rect {
                x: 1210,
                y: 515,
                width: 200,
                height: 50,
            },
            interact_wait_ms: 800,
            use_original_resin_rect: Rect {
                x: 850,
                y: 740,
                width: 250,
                height: 35,
            },
            use_original_resin_timeout_ms: 3_000,
            post_use_original_resin_delay_ms: 1_000,
            supplement_prompt_wait_ms: 1_000,
            reward_recognition_enabled: param.reward_recognition_enabled,
            reward_ready_close_rect: Rect {
                x: 850,
                y: 960,
                width: 220,
                height: 35,
            },
            reward_ready_retry_attempts: 20,
            reward_ready_retry_interval_ms: 300,
            close_result_retry_attempts: 20,
            close_result_retry_interval_ms: 300,
            click_center_after_attempt: 5,
        },
        reposition_rule: AutoBossRepositionRule {
            talk_to_start_uses_after_fight_quick_route: auto_boss_is_talk_to_start(
                &param.boss_name,
            ),
            no_pathing_support_reruns_special_navigation: auto_boss_is_no_pathing_support(
                &param.boss_name,
            ),
            normal_boss_replays_last_route_position: !auto_boss_is_talk_to_start(&param.boss_name)
                && !auto_boss_is_no_pathing_support(&param.boss_name),
            normal_boss_post_reposition_delay_ms: 4_000,
        },
        locators: auto_boss_locators(),
        steps: auto_boss_steps(),
        executor_ready: true,
        pending_native: vec![
            "AutoBoss now has a Rust injectable execution boundary for startup, per-round main-UI/team preparation, resin decisions, supplemental-resin decisions, boss navigation requests, fight dispatch, reward collection, reposition, cancellation checks, and cleanup".to_string(),
            "desktop live adapters remain pending: live capture/OCR/template probing, ReturnMainUi/SwitchParty/TpTask wiring, supplemental resin UI clicking/OCR, notifications, and cancellation integration".to_string(),
            "full pathing/fight adapters remain pending: PathExecutor, KeyMouseMacroPlayer, CombatScenes/AutoFightTask dispatch, reward-flower camera/input navigation, and RewardResultRecognizer live wiring".to_string(),
        ],
        param,
    })
}

pub fn execute_auto_boss_plan<R>(
    plan: &AutoBossExecutionPlan,
    runtime: &mut R,
) -> Result<AutoBossExecutionReport>
where
    R: AutoBossRuntime,
{
    let mut state = AutoBossExecutorState {
        target_reward_count: auto_boss_target_reward_count(&plan.param),
        ..AutoBossExecutorState::default()
    };
    let mut executed_actions = Vec::new();
    let mut skipped_steps = Vec::new();

    let execution_result = execute_auto_boss_plan_inner(
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
        &mut skipped_steps,
    );

    let status = match execution_result {
        Ok(status) => status,
        Err(error) => {
            let cleanup_error =
                execute_auto_boss_cleanup(plan, runtime, &mut state, &mut executed_actions).err();
            return Err(cleanup_error.unwrap_or(error));
        }
    };

    let cleanup_status =
        execute_auto_boss_cleanup(plan, runtime, &mut state, &mut executed_actions)?;
    let status = if cleanup_status == AutoBossExecutionStatus::CleanupFailed {
        AutoBossExecutionStatus::CleanupFailed
    } else {
        status
    };

    Ok(auto_boss_report(
        plan,
        status,
        state,
        executed_actions,
        skipped_steps,
    ))
}

fn execute_auto_boss_plan_inner<R>(
    plan: &AutoBossExecutionPlan,
    runtime: &mut R,
    state: &mut AutoBossExecutorState,
    executed_actions: &mut Vec<AutoBossRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoBossSkippedStep>,
) -> Result<AutoBossExecutionStatus>
where
    R: AutoBossRuntime,
{
    let startup = runtime.start_auto_boss(plan)?;
    state.startup_completed = startup.completed;
    executed_actions.push(auto_boss_action_report(
        AutoBossTaskPhase::Startup,
        AutoBossRuntimeActionKind::Startup,
        if startup.completed {
            AutoBossRuntimeActionStatus::Succeeded
        } else {
            AutoBossRuntimeActionStatus::Failed
        },
        None,
        startup
            .message
            .clone()
            .unwrap_or_else(|| "startup boundary completed".to_string()),
        AutoBossRuntimeActionOutcome::Startup(startup.clone()),
    ));
    if !startup.completed {
        return Ok(AutoBossExecutionStatus::StartupFailed);
    }

    loop {
        if let Some(target_reward_count) = state.target_reward_count {
            if state.rewards_claimed >= target_reward_count {
                return Ok(AutoBossExecutionStatus::Completed);
            }
        }

        if runtime.is_auto_boss_cancelled() {
            state.cancelled = true;
            return Ok(auto_boss_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoBossTaskPhase::Prepare,
                AutoBossRuntimeActionKind::SkipReward,
                None,
                AutoBossSkipReason::Cancelled,
                AutoBossExecutionStatus::Cancelled,
            ));
        }

        state.current_round += 1;
        state.rounds_started += 1;
        let round_index = state.current_round;
        let context = auto_boss_round_context(plan, state, true);

        let prepare = runtime.prepare_auto_boss_round(plan, &context)?;
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Prepare,
            AutoBossRuntimeActionKind::PrepareRound,
            if prepare.completed {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Failed
            },
            Some(round_index),
            prepare
                .message
                .clone()
                .unwrap_or_else(|| "round preparation completed".to_string()),
            AutoBossRuntimeActionOutcome::Prepare(prepare.clone()),
        ));
        if !prepare.completed {
            return Ok(AutoBossExecutionStatus::PrepareFailed);
        }

        let resin_check = runtime.check_auto_boss_resin(plan, &context)?;
        state.last_observed_original_resin = resin_check.original_resin;
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Resin,
            AutoBossRuntimeActionKind::CheckOriginalResin,
            AutoBossRuntimeActionStatus::Succeeded,
            Some(round_index),
            resin_check.message.clone().unwrap_or_else(|| {
                if resin_check.can_claim_reward {
                    "original resin can claim reward".to_string()
                } else {
                    "original resin cannot claim reward".to_string()
                }
            }),
            AutoBossRuntimeActionOutcome::ResinCheck(resin_check.clone()),
        ));

        let should_claim_reward = auto_boss_resolve_reward_claim(
            plan,
            runtime,
            state,
            executed_actions,
            &context,
            resin_check,
        )?;
        if !should_claim_reward {
            let status = if state.target_reward_count.is_none() {
                state.stopped_by_resin = true;
                AutoBossExecutionStatus::ResinExhausted
            } else {
                AutoBossExecutionStatus::RewardSkipped
            };
            let reason = state
                .last_skip_reason
                .unwrap_or(AutoBossSkipReason::InsufficientResin);
            return Ok(auto_boss_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoBossTaskPhase::Resin,
                AutoBossRuntimeActionKind::SkipReward,
                Some(round_index),
                reason,
                status,
            ));
        }

        let context = auto_boss_round_context(plan, state, true);
        let navigation_request =
            auto_boss_navigation_request(plan, AutoBossNavigationKind::FirstNavigation);
        state.navigation_attempts += 1;
        let navigation = runtime.navigate_auto_boss_to_boss(plan, &context, &navigation_request)?;
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Navigation,
            AutoBossRuntimeActionKind::NavigateToBoss,
            if navigation.completed {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Failed
            },
            Some(round_index),
            navigation
                .message
                .clone()
                .unwrap_or_else(|| "boss navigation boundary completed".to_string()),
            AutoBossRuntimeActionOutcome::Navigation(navigation.clone()),
        ));
        if !navigation.completed {
            return Ok(AutoBossExecutionStatus::NavigationFailed);
        }

        state.fights_attempted += 1;
        let combat = runtime.run_auto_boss_fight(plan, &context)?;
        let combat_succeeded = combat.completed && combat.victory;
        if combat_succeeded {
            state.fights_succeeded += 1;
        } else {
            state.combat_failures += 1;
        }
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Combat,
            AutoBossRuntimeActionKind::RunAutoFight,
            if combat_succeeded {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Failed
            },
            Some(round_index),
            combat
                .message
                .clone()
                .unwrap_or_else(|| "auto fight boundary completed".to_string()),
            AutoBossRuntimeActionOutcome::Combat(combat),
        ));
        if !combat_succeeded {
            return Ok(AutoBossExecutionStatus::CombatFailed);
        }

        let reward_navigation = runtime.move_auto_boss_to_reward(plan, &context)?;
        state.reward_navigation_attempts += 1;
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Reward,
            AutoBossRuntimeActionKind::MoveToRewardFlower,
            if reward_navigation.completed {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Failed
            },
            Some(round_index),
            reward_navigation
                .message
                .clone()
                .unwrap_or_else(|| "reward navigation boundary completed".to_string()),
            AutoBossRuntimeActionOutcome::RewardNavigation(reward_navigation.clone()),
        ));
        if !reward_navigation.completed {
            if !reward_navigation.reward_prompt_found {
                return Ok(auto_boss_skip(
                    state,
                    executed_actions,
                    skipped_steps,
                    AutoBossTaskPhase::Reward,
                    AutoBossRuntimeActionKind::SkipReward,
                    Some(round_index),
                    AutoBossSkipReason::RewardPromptMissing,
                    AutoBossExecutionStatus::RewardSkipped,
                ));
            }
            return Ok(AutoBossExecutionStatus::RewardFailed);
        }

        let reward = runtime.take_auto_boss_reward(plan, &context)?;
        if reward.claimed {
            state.rewards_claimed += 1;
            state.original_resin_spent += reward.original_resin_spent;
        } else {
            state.rewards_skipped += 1;
            state.last_skip_reason = reward.skip_reason;
        }
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Reward,
            AutoBossRuntimeActionKind::TakeReward,
            if reward.claimed {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Skipped
            },
            Some(round_index),
            reward.message.clone().unwrap_or_else(|| {
                if reward.claimed {
                    "reward claimed".to_string()
                } else {
                    "reward skipped".to_string()
                }
            }),
            AutoBossRuntimeActionOutcome::Reward(reward.clone()),
        ));
        if !reward.claimed {
            let reason = reward
                .skip_reason
                .unwrap_or(AutoBossSkipReason::RewardDisabledByRuntime);
            skipped_steps.push(AutoBossSkippedStep {
                action_kind: AutoBossRuntimeActionKind::TakeReward,
                round_index: Some(round_index),
                reason,
            });
            return Ok(AutoBossExecutionStatus::RewardSkipped);
        }

        if plan.reward_rule.reward_recognition_enabled {
            let recognition = runtime.recognize_auto_boss_reward(plan, &context)?;
            if recognition.attempted {
                state.reward_recognition_attempts += 1;
            }
            executed_actions.push(auto_boss_action_report(
                AutoBossTaskPhase::Reward,
                AutoBossRuntimeActionKind::RecognizeReward,
                if !recognition.attempted {
                    AutoBossRuntimeActionStatus::Skipped
                } else if recognition.recognized {
                    AutoBossRuntimeActionStatus::Succeeded
                } else {
                    AutoBossRuntimeActionStatus::Failed
                },
                Some(round_index),
                recognition
                    .message
                    .clone()
                    .unwrap_or_else(|| "reward recognition boundary completed".to_string()),
                AutoBossRuntimeActionOutcome::RewardRecognition(recognition),
            ));
        }

        if let Some(target_reward_count) = state.target_reward_count {
            if state.rewards_claimed >= target_reward_count {
                return Ok(AutoBossExecutionStatus::Completed);
            }
        }

        if runtime.is_auto_boss_cancelled() {
            state.cancelled = true;
            return Ok(auto_boss_skip(
                state,
                executed_actions,
                skipped_steps,
                AutoBossTaskPhase::Reposition,
                AutoBossRuntimeActionKind::SkipReward,
                Some(round_index),
                AutoBossSkipReason::Cancelled,
                AutoBossExecutionStatus::Cancelled,
            ));
        }

        let reposition_request =
            auto_boss_navigation_request(plan, auto_boss_next_reposition_kind(plan));
        let reposition =
            runtime.reposition_auto_boss_for_next_round(plan, &context, &reposition_request)?;
        executed_actions.push(auto_boss_action_report(
            AutoBossTaskPhase::Reposition,
            AutoBossRuntimeActionKind::RepositionForNextRound,
            if reposition.completed {
                AutoBossRuntimeActionStatus::Succeeded
            } else {
                AutoBossRuntimeActionStatus::Failed
            },
            Some(round_index),
            reposition
                .message
                .clone()
                .unwrap_or_else(|| "reposition boundary completed".to_string()),
            AutoBossRuntimeActionOutcome::Reposition(reposition),
        ));
        if !executed_actions
            .last()
            .is_some_and(|report| report.status == AutoBossRuntimeActionStatus::Succeeded)
        {
            return Ok(AutoBossExecutionStatus::RepositionFailed);
        }
    }
}

fn auto_boss_resolve_reward_claim<R>(
    plan: &AutoBossExecutionPlan,
    runtime: &mut R,
    state: &mut AutoBossExecutorState,
    executed_actions: &mut Vec<AutoBossRuntimeActionReport>,
    context: &AutoBossRuntimeRoundContext,
    resin_check: AutoBossResinCheckOutcome,
) -> Result<bool>
where
    R: AutoBossRuntime,
{
    if resin_check.can_claim_reward {
        return Ok(true);
    }

    if !resin_check.precheck_succeeded
        && plan.resin_rule.precheck_failure_falls_back_to_reward_prompt
    {
        return Ok(true);
    }

    if plan
        .supplemental_resin_rule
        .enabled_resin_options
        .is_empty()
    {
        state.last_skip_reason = Some(AutoBossSkipReason::InsufficientResin);
        return Ok(false);
    }

    let supplemental = runtime.use_auto_boss_supplemental_resin(plan, context, &resin_check)?;
    state.supplemental_resin_used +=
        supplemental.used_transient_resin + supplemental.used_fragile_resin;
    state.last_observed_original_resin = supplemental
        .original_resin_after
        .or(state.last_observed_original_resin);
    let can_claim_reward = supplemental.can_claim_reward;
    executed_actions.push(auto_boss_action_report(
        AutoBossTaskPhase::Resin,
        AutoBossRuntimeActionKind::UseSupplementalResin,
        if can_claim_reward {
            AutoBossRuntimeActionStatus::Succeeded
        } else if supplemental.attempted {
            AutoBossRuntimeActionStatus::Failed
        } else {
            AutoBossRuntimeActionStatus::Skipped
        },
        Some(context.round_index),
        supplemental.message.clone().unwrap_or_else(|| {
            if can_claim_reward {
                "supplemental resin made reward claim possible".to_string()
            } else {
                "supplemental resin did not make reward claim possible".to_string()
            }
        }),
        AutoBossRuntimeActionOutcome::SupplementalResin(supplemental),
    ));

    if !can_claim_reward {
        state.last_skip_reason = Some(AutoBossSkipReason::SupplementalResinUnavailable);
    }
    Ok(can_claim_reward)
}

fn execute_auto_boss_cleanup<R>(
    plan: &AutoBossExecutionPlan,
    runtime: &mut R,
    state: &mut AutoBossExecutorState,
    executed_actions: &mut Vec<AutoBossRuntimeActionReport>,
) -> Result<AutoBossExecutionStatus>
where
    R: AutoBossRuntime,
{
    let cleanup = runtime.cleanup_auto_boss(plan)?;
    state.cleanup_completed = cleanup.completed;
    let status = if cleanup.completed {
        AutoBossRuntimeActionStatus::Succeeded
    } else {
        AutoBossRuntimeActionStatus::Failed
    };
    executed_actions.push(auto_boss_action_report(
        AutoBossTaskPhase::Cleanup,
        AutoBossRuntimeActionKind::Cleanup,
        status,
        None,
        cleanup
            .message
            .clone()
            .unwrap_or_else(|| "cleanup boundary completed".to_string()),
        AutoBossRuntimeActionOutcome::Cleanup(cleanup.clone()),
    ));

    if cleanup.completed {
        Ok(AutoBossExecutionStatus::Completed)
    } else {
        Ok(AutoBossExecutionStatus::CleanupFailed)
    }
}

fn auto_boss_target_reward_count(param: &AutoBossParam) -> Option<u32> {
    if param.specify_run_count {
        Some(param.run_count.max(1) as u32)
    } else {
        None
    }
}

fn auto_boss_round_context(
    plan: &AutoBossExecutionPlan,
    state: &AutoBossExecutorState,
    should_claim_reward: bool,
) -> AutoBossRuntimeRoundContext {
    AutoBossRuntimeRoundContext {
        round_index: state.current_round,
        boss_name: plan.param.boss_name.clone(),
        target_reward_count: state.target_reward_count,
        claimed_rewards: state.rewards_claimed,
        is_first_round: state.current_round <= 1,
        should_claim_reward,
    }
}

fn auto_boss_navigation_request(
    plan: &AutoBossExecutionPlan,
    kind: AutoBossNavigationKind,
) -> AutoBossNavigationRequest {
    let boss_name = plan.param.boss_name.clone();
    let route_files = match kind {
        AutoBossNavigationKind::FirstNavigation => plan.pathing_rule.first_navigation_files.clone(),
        AutoBossNavigationKind::ReturnToStatue => Vec::new(),
        AutoBossNavigationKind::RepositionForNextRound => {
            if auto_boss_is_no_pathing_support(&boss_name) {
                auto_boss_first_navigation_files(&boss_name)
            } else if auto_boss_is_talk_to_start(&boss_name) {
                vec![format!("{boss_name}战斗后快速前往.json")]
            } else {
                vec![format!("{boss_name}前往.json")]
            }
        }
    };

    AutoBossNavigationRequest {
        kind,
        boss_name,
        route_files,
        no_pathing_support: plan
            .pathing_rule
            .no_pathing_support_uses_force_teleport_and_key_mouse,
        talk_to_start: plan
            .reposition_rule
            .talk_to_start_uses_after_fight_quick_route,
        return_to_statue: kind == AutoBossNavigationKind::ReturnToStatue,
    }
}

fn auto_boss_next_reposition_kind(plan: &AutoBossExecutionPlan) -> AutoBossNavigationKind {
    if plan.loop_rule.return_to_statue_after_each_round_option {
        AutoBossNavigationKind::ReturnToStatue
    } else {
        AutoBossNavigationKind::RepositionForNextRound
    }
}

fn auto_boss_skip(
    state: &mut AutoBossExecutorState,
    executed_actions: &mut Vec<AutoBossRuntimeActionReport>,
    skipped_steps: &mut Vec<AutoBossSkippedStep>,
    phase: AutoBossTaskPhase,
    action_kind: AutoBossRuntimeActionKind,
    round_index: Option<u32>,
    reason: AutoBossSkipReason,
    status: AutoBossExecutionStatus,
) -> AutoBossExecutionStatus {
    if matches!(
        reason,
        AutoBossSkipReason::InsufficientResin
            | AutoBossSkipReason::SupplementalResinUnavailable
            | AutoBossSkipReason::RewardDisabledByRuntime
            | AutoBossSkipReason::RewardPromptMissing
    ) {
        state.rewards_skipped += 1;
    }
    state.last_skip_reason = Some(reason);
    skipped_steps.push(AutoBossSkippedStep {
        action_kind,
        round_index,
        reason,
    });
    executed_actions.push(auto_boss_action_report(
        phase,
        action_kind,
        AutoBossRuntimeActionStatus::Skipped,
        round_index,
        format!("skipped AutoBoss step: {:?}", reason),
        AutoBossRuntimeActionOutcome::Skipped(reason),
    ));
    status
}

fn auto_boss_report(
    plan: &AutoBossExecutionPlan,
    status: AutoBossExecutionStatus,
    state: AutoBossExecutorState,
    executed_actions: Vec<AutoBossRuntimeActionReport>,
    skipped_steps: Vec<AutoBossSkippedStep>,
) -> AutoBossExecutionReport {
    AutoBossExecutionReport {
        task_key: plan.task_key.clone(),
        completed: status == AutoBossExecutionStatus::Completed,
        status,
        state,
        executed_actions,
        skipped_steps,
    }
}

fn auto_boss_action_report(
    phase: AutoBossTaskPhase,
    action_kind: AutoBossRuntimeActionKind,
    status: AutoBossRuntimeActionStatus,
    round_index: Option<u32>,
    detail: impl Into<String>,
    outcome: AutoBossRuntimeActionOutcome,
) -> AutoBossRuntimeActionReport {
    AutoBossRuntimeActionReport {
        phase,
        action_kind,
        status,
        round_index,
        detail: detail.into(),
        outcome,
    }
}

pub fn auto_boss_supported_boss_names() -> Vec<String> {
    AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .flat_map(|(_, bosses)| bosses.iter().copied())
        .map(str::to_string)
        .collect()
}

pub fn auto_boss_is_supported(boss_name: &str) -> bool {
    AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .any(|(_, bosses)| bosses.contains(&boss_name))
}

pub fn auto_boss_is_talk_to_start(boss_name: &str) -> bool {
    AUTO_BOSS_TALK_TO_START_BOSSES.contains(&boss_name)
}

pub fn auto_boss_is_no_pathing_support(boss_name: &str) -> bool {
    AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES.contains(&boss_name)
}

pub fn auto_boss_required_route_files(boss_name: &str) -> Vec<String> {
    if auto_boss_is_no_pathing_support(boss_name) {
        return vec![
            format!("{boss_name}强制传送.json"),
            format!("{boss_name}键鼠前往.json"),
        ];
    }

    let mut routes = vec![format!("{boss_name}前往.json")];
    if auto_boss_is_talk_to_start(boss_name) {
        routes.push(format!("{boss_name}战斗后快速前往.json"));
    }
    routes
}

pub fn auto_boss_first_navigation_files(boss_name: &str) -> Vec<String> {
    if auto_boss_is_no_pathing_support(boss_name) {
        vec![
            format!("{boss_name}强制传送.json"),
            format!("{boss_name}键鼠前往.json"),
        ]
    } else {
        vec![format!("{boss_name}前往.json")]
    }
}

fn validate_auto_boss_param(working_directory: &Path, param: &AutoBossParam) -> Result<()> {
    if param.boss_name.trim().is_empty() {
        return invalid_auto_boss_config("请选择需要讨伐的首领");
    }
    if !auto_boss_is_supported(&param.boss_name) {
        return invalid_auto_boss_config(format!("暂不支持首领：{}", param.boss_name));
    }
    if param.revive_retry_count < 0 {
        return invalid_auto_boss_config("角色死亡后重试次数不能小于 0");
    }
    if param.specify_run_count && param.run_count < 1 {
        return invalid_auto_boss_config("指定讨伐次数必须大于 0");
    }
    if !param.specify_run_count && (param.use_transient_resin || param.use_fragile_resin) {
        return invalid_auto_boss_config("只有指定讨伐次数模式才能开启须臾树脂或脆弱树脂补充");
    }

    let strategy_path = working_directory.join(&param.combat_strategy_path);
    if !strategy_path.exists() {
        return invalid_auto_boss_config("当前选择的自动战斗策略文件不存在");
    }

    for route in auto_boss_required_route_files(&param.boss_name) {
        let path = auto_boss_pathing_asset_path(working_directory, &route);
        if !path.exists() {
            return Err(TaskError::InvalidTaskConfig {
                key: AUTO_BOSS_TASK_KEY.to_string(),
                message: format!("未找到首领路线文件：{route}"),
            });
        }
    }
    Ok(())
}

fn auto_boss_pathing_asset_path(working_directory: &Path, route: &str) -> PathBuf {
    working_directory
        .join(AUTO_BOSS_PATHING_ASSET_DIR)
        .join(route)
}

fn invalid_auto_boss_config<T>(message: impl Into<String>) -> Result<T> {
    Err(TaskError::InvalidTaskConfig {
        key: AUTO_BOSS_TASK_KEY.to_string(),
        message: message.into(),
    })
}

fn normalize_auto_boss_param_strategy(param: &mut AutoBossParam) {
    if param.strategy_name.trim().is_empty() {
        param.strategy_name = AUTO_STRATEGY_NAME.to_string();
    }
    if param.combat_strategy_path.trim().is_empty() {
        param.combat_strategy_path = combat_strategy_path(Some(&param.strategy_name));
    }
}

fn auto_boss_data_plan(selected_boss: &str) -> AutoBossDataPlan {
    let country_to_bosses = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .map(|(country, bosses)| AutoBossCountryBosses {
            country: (*country).to_string(),
            bosses: bosses.iter().map(|boss| (*boss).to_string()).collect(),
        })
        .collect::<Vec<_>>();
    let selected_boss_country = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .find(|(_, bosses)| bosses.contains(&selected_boss))
        .map(|(country, _)| (*country).to_string());
    let supported_boss_count = AUTO_BOSS_COUNTRY_TO_BOSSES
        .iter()
        .map(|(_, bosses)| bosses.len())
        .sum();

    AutoBossDataPlan {
        country_to_bosses,
        supported_boss_count,
        selected_boss_supported: auto_boss_is_supported(selected_boss),
        selected_boss_country,
        selected_boss_talk_to_start: auto_boss_is_talk_to_start(selected_boss),
        selected_boss_no_pathing_support: auto_boss_is_no_pathing_support(selected_boss),
        talk_to_start_bosses: AUTO_BOSS_TALK_TO_START_BOSSES
            .iter()
            .map(|boss| (*boss).to_string())
            .collect(),
        no_pathing_support_bosses: AUTO_BOSS_NO_PATHING_SUPPORT_BOSSES
            .iter()
            .map(|boss| (*boss).to_string())
            .collect(),
    }
}

fn supplemental_resin_options(param: &AutoBossParam) -> Vec<AutoBossSupplementalResinOption> {
    let mut options = Vec::new();
    if param.use_transient_resin {
        options.push(AutoBossSupplementalResinOption {
            name: "须臾树脂".to_string(),
            asset: AUTO_BOSS_TRANSIENT_RESIN_ASSET.to_string(),
        });
    }
    if param.use_fragile_resin {
        options.push(AutoBossSupplementalResinOption {
            name: "脆弱树脂".to_string(),
            asset: AUTO_BOSS_FRAGILE_RESIN_ASSET.to_string(),
        });
    }
    options
}

fn auto_boss_locators() -> AutoBossLocators {
    AutoBossLocators {
        original_resin_top_icon: template_locator(
            "AutoBossOriginalResinTopIcon",
            AUTO_BOSS_ORIGINAL_RESIN_TOP_ICON_ASSET,
        ),
        reward_box: template_locator("AutoBossRewardBox", AUTO_BOSS_REWARD_BOX_ASSET),
        open_resin_supplement_pane_button: template_locator(
            "AutoBossOpenResinSupplementPaneButton",
            AUTO_BOSS_OPEN_RESIN_SUPPLEMENT_PANE_BUTTON_ASSET,
        ),
        transient_resin: template_locator(
            "AutoBossTransientResinInSupplementPane",
            AUTO_BOSS_TRANSIENT_RESIN_ASSET,
        ),
        fragile_resin: template_locator(
            "AutoBossFragileResinInSupplementPane",
            AUTO_BOSS_FRAGILE_RESIN_ASSET,
        ),
        increase_resin_usage_quantity_button: template_locator(
            "AutoBossIncreaseResinUsageQuantityButton",
            AUTO_BOSS_INCREASE_RESIN_QUANTITY_BUTTON_ASSET,
        ),
    }
}

fn template_locator(name: &str, asset: &str) -> AutoBossTemplateLocator {
    AutoBossTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi: None,
        threshold: 0.8,
        match_mode: TemplateMatchMode::CCoeffNormed,
        draw_on_window: false,
    }
}

fn auto_boss_steps() -> Vec<AutoBossTaskStep> {
    use AutoBossTaskAction::*;
    use AutoBossTaskPhase::*;
    vec![
        AutoBossTaskStep {
            phase: Startup,
            action: ValidateAndParseCombatStrategy,
        },
        AutoBossTaskStep {
            phase: Startup,
            action: LogScreenResolution,
        },
        AutoBossTaskStep {
            phase: Prepare,
            action: ReturnMainUiAndSwitchParty,
        },
        AutoBossTaskStep {
            phase: Resin,
            action: CheckOriginalResin,
        },
        AutoBossTaskStep {
            phase: Resin,
            action: UseSupplementalResinWhenAllowed,
        },
        AutoBossTaskStep {
            phase: Navigation,
            action: NavigateToBoss,
        },
        AutoBossTaskStep {
            phase: Combat,
            action: RunAutoFight,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: MoveToRewardFlower,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: TakeReward,
        },
        AutoBossTaskStep {
            phase: Reward,
            action: RecognizeRewardWhenEnabled,
        },
        AutoBossTaskStep {
            phase: Reposition,
            action: RepositionForNextRound,
        },
        AutoBossTaskStep {
            phase: Cleanup,
            action: ReleaseInputsAndNotifyEnd,
        },
    ]
}

fn apply_auto_boss_config(param: &mut AutoBossParam, config: &AutoBossConfig) {
    param.boss_name = config.boss_name.clone();
    param.set_strategy_name(Some(&config.strategy_name));
    param.team_name = config.team_name.clone();
    param.specify_run_count = config.specify_run_count;
    param.run_count = config.run_count as i32;
    param.use_transient_resin = config.use_transient_resin;
    param.use_fragile_resin = config.use_fragile_resin;
    param.revive_retry_count = config.revive_retry_count as i32;
    param.return_to_statue_after_each_round = config.return_to_statue_after_each_round;
    param.reward_recognition_enabled = config.reward_recognition_enabled;
}

fn overlay_auto_boss_param_members(param: &mut AutoBossParam, value: &Value) {
    if let Some(boss_name) = string_member(
        value,
        [
            "bossName",
            "BossName",
            "boss_name",
            "autoBossName",
            "AutoBossName",
        ],
    ) {
        param.boss_name = boss_name;
    }
    if let Some(strategy_name) =
        string_member(value, ["strategyName", "StrategyName", "strategy_name"])
    {
        param.set_strategy_name(Some(&strategy_name));
    }
    if let Some(path) = string_member(
        value,
        [
            "combatStrategyPath",
            "CombatStrategyPath",
            "combat_strategy_path",
        ],
    ) {
        param.combat_strategy_path = path;
    }
    if let Some(team_name) = string_member(value, ["teamName", "TeamName", "team_name"]) {
        param.team_name = team_name;
    }
    if let Some(value) = bool_member(
        value,
        ["specifyRunCount", "SpecifyRunCount", "specify_run_count"],
    ) {
        param.specify_run_count = value;
    }
    if let Some(value) = i32_member(value, ["runCount", "RunCount", "run_count"]) {
        param.run_count = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "useTransientResin",
            "UseTransientResin",
            "use_transient_resin",
        ],
    ) {
        param.use_transient_resin = value;
    }
    if let Some(value) = bool_member(
        value,
        ["useFragileResin", "UseFragileResin", "use_fragile_resin"],
    ) {
        param.use_fragile_resin = value;
    }
    if let Some(value) = i32_member(
        value,
        ["reviveRetryCount", "ReviveRetryCount", "revive_retry_count"],
    ) {
        param.revive_retry_count = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "returnToStatueAfterEachRound",
            "ReturnToStatueAfterEachRound",
            "return_to_statue_after_each_round",
        ],
    ) {
        param.return_to_statue_after_each_round = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "rewardRecognitionEnabled",
            "RewardRecognitionEnabled",
            "reward_recognition_enabled",
        ],
    ) {
        param.reward_recognition_enabled = value;
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn string_member_from<const N: usize>(
    primary: Option<&Value>,
    fallback: &Value,
    names: &[&str; N],
) -> Option<String> {
    primary
        .and_then(|value| {
            names
                .iter()
                .find_map(|name| value.get(*name))
                .and_then(|value| value.as_str().map(str::to_string))
        })
        .or_else(|| {
            names
                .iter()
                .find_map(|name| fallback.get(*name))
                .and_then(|value| value.as_str().map(str::to_string))
        })
}

fn bool_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<bool> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_bool)
}

fn i32_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<i32> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(|value| value.as_i64())
        .and_then(|value| i32::try_from(value).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RuntimeCall {
        Startup,
        Prepare(u32),
        CheckResin(u32),
        UseSupplementalResin(u32),
        Navigate(u32, AutoBossNavigationKind, Vec<String>),
        Fight(u32),
        MoveToReward(u32),
        TakeReward(u32),
        RecognizeReward(u32),
        Reposition(u32, AutoBossNavigationKind, Vec<String>),
        Cleanup,
    }

    #[derive(Debug, Clone, Default)]
    struct FakeAutoBossRuntime {
        calls: Vec<RuntimeCall>,
        resin_checks: Vec<AutoBossResinCheckOutcome>,
        supplemental_outcomes: Vec<AutoBossSupplementalResinOutcome>,
        combat_outcomes: Vec<AutoBossCombatOutcome>,
        reward_outcomes: Vec<AutoBossRewardOutcome>,
        reward_navigation_outcomes: Vec<AutoBossRewardNavigationOutcome>,
        cleanup_count: u32,
    }

    impl AutoBossRuntime for FakeAutoBossRuntime {
        fn start_auto_boss(
            &mut self,
            _plan: &AutoBossExecutionPlan,
        ) -> Result<AutoBossStartupOutcome> {
            self.calls.push(RuntimeCall::Startup);
            Ok(AutoBossStartupOutcome {
                completed: true,
                combat_strategy_parsed: true,
                screen_resolution_logged: true,
                start_notification_sent: true,
                message: None,
            })
        }

        fn prepare_auto_boss_round(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossPrepareOutcome> {
            self.calls.push(RuntimeCall::Prepare(context.round_index));
            Ok(AutoBossPrepareOutcome {
                completed: true,
                main_ui_ready: true,
                party_switched: true,
                message: None,
            })
        }

        fn check_auto_boss_resin(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossResinCheckOutcome> {
            self.calls
                .push(RuntimeCall::CheckResin(context.round_index));
            Ok(if self.resin_checks.is_empty() {
                AutoBossResinCheckOutcome {
                    precheck_succeeded: true,
                    original_resin: Some(160),
                    can_claim_reward: true,
                    message: None,
                }
            } else {
                self.resin_checks.remove(0)
            })
        }

        fn use_auto_boss_supplemental_resin(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
            _resin_check: &AutoBossResinCheckOutcome,
        ) -> Result<AutoBossSupplementalResinOutcome> {
            self.calls
                .push(RuntimeCall::UseSupplementalResin(context.round_index));
            Ok(if self.supplemental_outcomes.is_empty() {
                AutoBossSupplementalResinOutcome {
                    attempted: false,
                    used_transient_resin: 0,
                    used_fragile_resin: 0,
                    original_resin_after: None,
                    can_claim_reward: false,
                    message: None,
                }
            } else {
                self.supplemental_outcomes.remove(0)
            })
        }

        fn navigate_auto_boss_to_boss(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
            request: &AutoBossNavigationRequest,
        ) -> Result<AutoBossNavigationOutcome> {
            self.calls.push(RuntimeCall::Navigate(
                context.round_index,
                request.kind,
                request.route_files.clone(),
            ));
            Ok(AutoBossNavigationOutcome {
                completed: true,
                teleport_used: request.no_pathing_support,
                pathing_used: !request.route_files.is_empty(),
                route_files: request.route_files.clone(),
                message: None,
            })
        }

        fn run_auto_boss_fight(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossCombatOutcome> {
            self.calls.push(RuntimeCall::Fight(context.round_index));
            Ok(if self.combat_outcomes.is_empty() {
                AutoBossCombatOutcome {
                    completed: true,
                    victory: true,
                    normal_end: true,
                    duration_ms: Some(1_000),
                    message: None,
                }
            } else {
                self.combat_outcomes.remove(0)
            })
        }

        fn move_auto_boss_to_reward(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossRewardNavigationOutcome> {
            self.calls
                .push(RuntimeCall::MoveToReward(context.round_index));
            Ok(if self.reward_navigation_outcomes.is_empty() {
                AutoBossRewardNavigationOutcome {
                    completed: true,
                    reward_prompt_found: true,
                    message: None,
                }
            } else {
                self.reward_navigation_outcomes.remove(0)
            })
        }

        fn take_auto_boss_reward(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossRewardOutcome> {
            self.calls
                .push(RuntimeCall::TakeReward(context.round_index));
            Ok(if self.reward_outcomes.is_empty() {
                AutoBossRewardOutcome {
                    claimed: true,
                    original_resin_spent: AUTO_BOSS_ORIGINAL_RESIN_COST,
                    skip_reason: None,
                    message: None,
                }
            } else {
                self.reward_outcomes.remove(0)
            })
        }

        fn recognize_auto_boss_reward(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
        ) -> Result<AutoBossRewardRecognitionOutcome> {
            self.calls
                .push(RuntimeCall::RecognizeReward(context.round_index));
            Ok(AutoBossRewardRecognitionOutcome {
                attempted: true,
                recognized: true,
                message: None,
            })
        }

        fn reposition_auto_boss_for_next_round(
            &mut self,
            _plan: &AutoBossExecutionPlan,
            context: &AutoBossRuntimeRoundContext,
            request: &AutoBossNavigationRequest,
        ) -> Result<AutoBossRepositionOutcome> {
            self.calls.push(RuntimeCall::Reposition(
                context.round_index,
                request.kind,
                request.route_files.clone(),
            ));
            Ok(AutoBossRepositionOutcome {
                completed: true,
                returned_to_statue: request.return_to_statue,
                route_files: request.route_files.clone(),
                message: None,
            })
        }

        fn cleanup_auto_boss(
            &mut self,
            _plan: &AutoBossExecutionPlan,
        ) -> Result<AutoBossCleanupOutcome> {
            self.calls.push(RuntimeCall::Cleanup);
            self.cleanup_count += 1;
            Ok(AutoBossCleanupOutcome {
                completed: true,
                released_all_keys: true,
                released_left_mouse: true,
                end_notification_sent: true,
                message: None,
            })
        }
    }

    #[test]
    fn auto_boss_execute_single_boss_success_loop() {
        let root = test_root("auto-boss-execute-success");
        let plan = test_plan(
            &root,
            serde_json::json!({
                "bossName": "爆炎树",
                "strategyName": "boss",
                "teamName": "Boss Team",
                "specifyRunCount": true,
                "runCount": 1,
                "rewardRecognitionEnabled": true
            }),
        );
        assert!(plan.executor_ready);
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("desktop live adapters remain pending")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("full pathing/fight adapters remain pending")));

        let mut runtime = FakeAutoBossRuntime::default();
        let report = execute_auto_boss_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoBossExecutionStatus::Completed);
        assert!(report.completed);
        assert_eq!(report.state.rounds_started, 1);
        assert_eq!(report.state.fights_succeeded, 1);
        assert_eq!(report.state.rewards_claimed, 1);
        assert_eq!(report.state.original_resin_spent, 40);
        assert!(report.state.cleanup_completed);
        assert_eq!(runtime.cleanup_count, 1);
        assert!(runtime.calls.contains(&RuntimeCall::Navigate(
            1,
            AutoBossNavigationKind::FirstNavigation,
            vec!["爆炎树前往.json".to_string()]
        )));
        assert!(runtime.calls.contains(&RuntimeCall::RecognizeReward(1)));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auto_boss_execute_insufficient_resin_skips_without_reward() {
        let root = test_root("auto-boss-execute-no-resin");
        let plan = test_plan(
            &root,
            serde_json::json!({
                "bossName": "爆炎树",
                "strategyName": "boss",
                "specifyRunCount": false
            }),
        );
        let mut runtime = FakeAutoBossRuntime {
            resin_checks: vec![AutoBossResinCheckOutcome {
                precheck_succeeded: true,
                original_resin: Some(20),
                can_claim_reward: false,
                message: Some("not enough resin".to_string()),
            }],
            ..FakeAutoBossRuntime::default()
        };

        let report = execute_auto_boss_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoBossExecutionStatus::ResinExhausted);
        assert!(!report.completed);
        assert!(report.state.stopped_by_resin);
        assert_eq!(report.state.rewards_claimed, 0);
        assert_eq!(report.state.rewards_skipped, 1);
        assert_eq!(
            report.state.last_skip_reason,
            Some(AutoBossSkipReason::InsufficientResin)
        );
        assert_eq!(runtime.cleanup_count, 1);
        assert!(!runtime
            .calls
            .iter()
            .any(|call| matches!(call, RuntimeCall::Fight(_))));
        assert!(!runtime
            .calls
            .iter()
            .any(|call| matches!(call, RuntimeCall::TakeReward(_))));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auto_boss_execute_combat_failure_records_failure_and_stops() {
        let root = test_root("auto-boss-execute-combat-failure");
        let plan = test_plan(
            &root,
            serde_json::json!({
                "bossName": "爆炎树",
                "strategyName": "boss",
                "specifyRunCount": true,
                "runCount": 2
            }),
        );
        let mut runtime = FakeAutoBossRuntime {
            combat_outcomes: vec![AutoBossCombatOutcome {
                completed: true,
                victory: false,
                normal_end: false,
                duration_ms: Some(500),
                message: Some("fight failed".to_string()),
            }],
            ..FakeAutoBossRuntime::default()
        };

        let report = execute_auto_boss_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoBossExecutionStatus::CombatFailed);
        assert!(!report.completed);
        assert_eq!(report.state.fights_attempted, 1);
        assert_eq!(report.state.fights_succeeded, 0);
        assert_eq!(report.state.combat_failures, 1);
        assert_eq!(report.state.rewards_claimed, 0);
        assert_eq!(runtime.cleanup_count, 1);
        assert!(report.executed_actions.iter().any(|action| {
            action.action_kind == AutoBossRuntimeActionKind::RunAutoFight
                && action.status == AutoBossRuntimeActionStatus::Failed
        }));
        assert!(!runtime
            .calls
            .iter()
            .any(|call| matches!(call, RuntimeCall::MoveToReward(_))));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn auto_boss_execute_cleanup_runs_after_reward_skip() {
        let root = test_root("auto-boss-execute-cleanup-skip");
        let plan = test_plan(
            &root,
            serde_json::json!({
                "bossName": "爆炎树",
                "strategyName": "boss",
                "specifyRunCount": true,
                "runCount": 1
            }),
        );
        let mut runtime = FakeAutoBossRuntime {
            reward_outcomes: vec![AutoBossRewardOutcome {
                claimed: false,
                original_resin_spent: 0,
                skip_reason: Some(AutoBossSkipReason::RewardDisabledByRuntime),
                message: Some("runtime declined reward".to_string()),
            }],
            ..FakeAutoBossRuntime::default()
        };

        let report = execute_auto_boss_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoBossExecutionStatus::RewardSkipped);
        assert!(!report.completed);
        assert_eq!(report.state.rewards_claimed, 0);
        assert_eq!(report.state.rewards_skipped, 1);
        assert!(report.state.cleanup_completed);
        assert_eq!(runtime.cleanup_count, 1);
        assert_eq!(runtime.calls.last(), Some(&RuntimeCall::Cleanup));
        assert_eq!(report.skipped_steps.len(), 1);
        assert_eq!(
            report.skipped_steps[0].reason,
            AutoBossSkipReason::RewardDisabledByRuntime
        );

        let _ = fs::remove_dir_all(root);
    }

    fn test_plan(root: &Path, auto_boss_config: Value) -> AutoBossExecutionPlan {
        write_test_file(
            &root.join("User").join("AutoFight").join("boss.txt"),
            "钟离 e, wait(0.2)",
        );
        let boss_name = auto_boss_config
            .get("bossName")
            .and_then(Value::as_str)
            .unwrap_or("爆炎树");
        for route in auto_boss_required_route_files(boss_name) {
            write_test_file(
                &root
                    .join("GameTask")
                    .join("AutoBoss")
                    .join("Assets")
                    .join("Pathing")
                    .join(route),
                r#"{"positions":[]}"#,
            );
        }
        let config = AutoBossExecutionConfig::from_value(Some(&serde_json::json!({
            "autoBossConfig": auto_boss_config
        })));
        plan_auto_boss(root, config).unwrap()
    }

    fn test_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bgi-task-{name}-{nonce}"))
    }

    fn write_test_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}
