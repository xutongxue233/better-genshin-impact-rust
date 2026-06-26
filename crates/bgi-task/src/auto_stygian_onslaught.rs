use bgi_core::config::{AutoArtifactSalvageConfig, AutoStygianOnslaughtConfig};
use bgi_core::GenshinAction;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auto_artifact_salvage::AutoArtifactSalvageParam;
use crate::auto_domain::AutoDomainResinUseRecord;
use crate::quick_teleport::QUICK_TELEPORT_GO_TELEPORT;
use crate::task_params::{combat_strategy_path, AutoStygianOnslaughtParam};
use crate::{Result, TaskError, TaskPortState};

pub const AUTO_STYGIAN_ONSLAUGHT_TASK_KEY: &str = "AutoStygianOnslaught";
pub const AUTO_STYGIAN_ONSLAUGHT_DISPLAY_NAME: &str = "自动幽境危战";
pub const AUTO_STYGIAN_ONSLAUGHT_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_STYGIAN_ONSLAUGHT_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_STYGIAN_ONSLAUGHT_LOOP_LIMIT: u32 = 9_999;
pub const AUTO_STYGIAN_ONSLAUGHT_EVENT_NAME: &str = "幽境危战";
pub const AUTO_STYGIAN_ONSLAUGHT_HARD_DIFFICULTY: &str = "困难";
pub const AUTO_STYGIAN_ONSLAUGHT_ULTIMATE_CHALLENGE: &str = "至危挑战";
pub const AUTO_STYGIAN_ONSLAUGHT_NORMAL_CHALLENGE: &str = "常规挑战";
pub const AUTO_STYGIAN_ONSLAUGHT_CONFIRM_ASSET: &str = "AutoFight:confirm.png";
pub const AUTO_STYGIAN_ONSLAUGHT_EXIT_ASSET: &str = "AutoFight:exit.png";
pub const AUTO_STYGIAN_ONSLAUGHT_WHITE_CONFIRM_ASSET: &str = "Common/Element:btn_white_confirm.png";
pub const AUTO_STYGIAN_ONSLAUGHT_WHITE_CANCEL_ASSET: &str = "Common/Element:btn_white_cancel.png";
pub const AUTO_STYGIAN_ONSLAUGHT_EXIT_DOOR_ASSET: &str = "Common/Element:btn_exit_door.png";
pub const AUTO_STYGIAN_ONSLAUGHT_PAIMON_MENU_ASSET: &str = "Common/Element:paimon_menu.png";
pub const AUTO_STYGIAN_ONSLAUGHT_INVENTORY_ASSET: &str = "Common/Element:bag.png";
pub const AUTO_STYGIAN_ONSLAUGHT_LEYLINE_DISORDER_ASSET: &str =
    "Common/Element:leyline_disorder_icon.png";
pub const AUTO_STYGIAN_ONSLAUGHT_PICK_KEY_ASSET: &str = "AutoPick:F.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoStygianOnslaughtParam,
    pub param_rule: AutoStygianOnslaughtParamRule,
    pub startup_rule: AutoStygianOnslaughtStartupRule,
    pub state_machine_rule: AutoStygianOnslaughtStateMachineRule,
    pub detector_rule: AutoStygianOnslaughtDetectorRule,
    pub navigation_rule: AutoStygianOnslaughtNavigationRule,
    pub difficulty_rule: AutoStygianOnslaughtDifficultyRule,
    pub boss_rule: AutoStygianOnslaughtBossRule,
    pub team_rule: AutoStygianOnslaughtTeamRule,
    pub combat_rule: AutoStygianOnslaughtCombatRule,
    pub reward_rule: AutoStygianOnslaughtRewardRule,
    pub resin_rule: AutoStygianOnslaughtResinRule,
    pub exit_rule: AutoStygianOnslaughtExitRule,
    pub artifact_salvage_rule: AutoStygianOnslaughtArtifactSalvageRule,
    pub steps: Vec<AutoStygianOnslaughtStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoStygianOnslaughtExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub param: AutoStygianOnslaughtParam,
    pub auto_stygian_onslaught_config: AutoStygianOnslaughtConfig,
    pub artifact_salvage_max_artifact_star: String,
}

impl Default for AutoStygianOnslaughtExecutionConfig {
    fn default() -> Self {
        let auto_stygian_onslaught_config = AutoStygianOnslaughtConfig::default();
        let mut param = AutoStygianOnslaughtParam::default();
        apply_auto_stygian_config(&mut param, &auto_stygian_onslaught_config);
        Self {
            capture_size: Size::new(
                AUTO_STYGIAN_ONSLAUGHT_DEFAULT_CAPTURE_WIDTH,
                AUTO_STYGIAN_ONSLAUGHT_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            param,
            auto_stygian_onslaught_config,
            artifact_salvage_max_artifact_star: AutoArtifactSalvageConfig::default()
                .max_artifact_star,
        }
    }
}

impl AutoStygianOnslaughtExecutionConfig {
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

        let stygian_value = value
            .get("autoStygianOnslaughtConfig")
            .or_else(|| value.get("AutoStygianOnslaughtConfig"))
            .or_else(|| value.get("auto_stygian_onslaught_config"))
            .unwrap_or(value);
        config.auto_stygian_onslaught_config =
            serde_json::from_value(stygian_value.clone()).unwrap_or_default();
        apply_auto_stygian_config(&mut config.param, &config.auto_stygian_onslaught_config);
        overlay_param(&mut config.param, stygian_value);
        if stygian_value as *const Value != value as *const Value {
            overlay_param(&mut config.param, value);
        }

        if let Some(param_value) = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("autoStygianOnslaughtParam"))
            .or_else(|| value.get("AutoStygianOnslaughtParam"))
            .or_else(|| value.get("auto_stygian_onslaught_param"))
        {
            overlay_param(&mut config.param, param_value);
        }

        if let Some(max_artifact_star) = string_member(
            value,
            [
                "artifactSalvageMaxArtifactStar",
                "maxArtifactStar",
                "MaxArtifactStar",
                "max_artifact_star",
            ],
        ) {
            config.artifact_salvage_max_artifact_star = max_artifact_star;
        }
        if let Some(artifact_salvage_value) = value
            .get("autoArtifactSalvageConfig")
            .or_else(|| value.get("AutoArtifactSalvageConfig"))
            .or_else(|| value.get("auto_artifact_salvage_config"))
        {
            if let Some(max_artifact_star) = string_member(
                artifact_salvage_value,
                ["maxArtifactStar", "MaxArtifactStar", "max_artifact_star"],
            ) {
                config.artifact_salvage_max_artifact_star = max_artifact_star;
            }
        }

        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtParamRule {
    pub boss_num: i32,
    pub selected_boss_num: i32,
    pub invalid_boss_num_falls_back_to: i32,
    pub combat_script_bag_path: String,
    pub auto_team_strategy_directory: String,
    pub fight_team_name: String,
    pub task_runner_skips_main_ui_wait_due_to_legacy_name: bool,
    pub task_still_returns_main_ui_on_start: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtStartupRule {
    pub destroys_auto_fight_assets_before_start: bool,
    pub parses_combat_script_bag_before_start: bool,
    pub builds_specified_resin_records_before_start: bool,
    pub creates_lower_head_then_walk_to_task: LowerHeadThenWalkToDependencyRule,
    pub logs_screen_resolution: bool,
    pub requires_16_to_9_resolution: bool,
    pub warns_below_1920x1080: bool,
    pub sends_domain_start_notification: bool,
    pub sends_domain_end_notification: bool,
    pub catches_task_cancelled_without_error: bool,
    pub catches_other_exceptions_as_log_information: bool,
    pub delay_before_artifact_salvage_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LowerHeadThenWalkToDependencyRule {
    pub target_asset: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtStateMachineRule {
    pub initial_state: StygianState,
    pub navigation_target_state: StygianState,
    pub battle_loop_limit: u32,
    pub transitions: Vec<StygianStateTransition>,
    pub unknown_state_returns_main_ui: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StygianStateTransition {
    pub from: StygianState,
    pub to: Vec<StygianState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StygianState {
    Unknown,
    MainWorld,
    EventMenu,
    StygianOnslaughtPage,
    TeleportMap,
    DomainEntrance,
    DifficultySelect,
    DomainLoading,
    DomainLobby,
    BossSelect,
    BattleArena,
    BattleLoading,
    InBattle,
    BattleResultWin,
    BattleResultLose,
    LeylineFlowerPrompt,
    ResinSelect,
    ContinueOrExit,
    Exiting,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtDetectorRule {
    pub detectors: Vec<StygianStateDetector>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StygianStateDetector {
    pub state: StygianState,
    pub order: u16,
    pub rule: StygianStateDetectorKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum StygianStateDetectorKind {
    TemplatePair {
        required_assets: Vec<String>,
        missing_assets: Vec<String>,
    },
    Template {
        asset: String,
    },
    TemplateAndOcr {
        asset: String,
        roi: StygianRoiRule,
        contains_any: Vec<String>,
    },
    Ocr {
        roi: StygianRoiRule,
        contains_all: Vec<String>,
        contains_any: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum StygianRoiRule {
    Absolute1080p(Rect),
    Relative {
        x_ratio: f64,
        y_ratio: f64,
        width_ratio: f64,
        height_ratio: f64,
    },
    CutRight {
        width_ratio: f64,
    },
    CutLeft {
        width_ratio: f64,
    },
    CutLeftTop {
        width_ratio: f64,
        height_ratio: f64,
    },
    CutRightTop {
        width_ratio: f64,
        height_ratio: f64,
    },
    CutRightBottom {
        width_ratio: f64,
        height_ratio: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtNavigationRule {
    pub return_main_ui_before_navigation: bool,
    pub open_event_menu_action: GenshinAction,
    pub open_event_menu_delay_ms: u64,
    pub event_menu_title_text: String,
    pub event_list_roi_1080p: Rect,
    pub event_list_drag: StygianDragRule,
    pub event_search_attempts: u8,
    pub event_name: String,
    pub go_challenge_text: String,
    pub go_challenge_roi: StygianRoiRule,
    pub teleport_button_asset: String,
    pub teleport_click_delay_ms: u64,
    pub domain_entrance_text_roi_1080p: Rect,
    pub domain_entrance_interact_action: GenshinAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StygianDragRule {
    pub start_x_1080p: i32,
    pub start_y_1080p: i32,
    pub end_y_1080p: i32,
    pub step_y_1080p: i32,
    pub mouse_down_delay_ms: u64,
    pub step_delay_ms: u64,
    pub after_drag_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtDifficultyRule {
    pub target_difficulty: String,
    pub single_player_text: String,
    pub target_detect_roi: StygianRoiRule,
    pub ultimate_challenge_text: String,
    pub normal_challenge_text: String,
    pub normal_challenge_menu_click_offset_x_1080p: i32,
    pub retry_attempts: u8,
    pub retry_interval_ms: u64,
    pub mode_switch_wait_ms: u64,
    pub difficulty_click_wait_ms: u64,
    pub confirm_asset: String,
    pub after_confirm_wait_ms: u64,
    pub continue_when_switch_failed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtBossRule {
    pub requested_boss_num: i32,
    pub selected_boss_num: i32,
    pub invalid_boss_num_falls_back_to: i32,
    pub boss_positions_1080p: Vec<StygianBossPosition>,
    pub start_challenge_text: String,
    pub character_preview_text: String,
    pub start_challenge_confirm_asset: String,
    pub after_start_wait_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StygianBossPosition {
    pub boss_num: i32,
    pub x_1080p: i32,
    pub y_1080p: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtTeamRule {
    pub enabled: bool,
    pub fight_team_name: String,
    pub open_panel_text: String,
    pub panel_open_roi: StygianRoiRule,
    pub panel_button_roi: StygianRoiRule,
    pub open_retry_interval_ms: u64,
    pub search_start_point_1080p: StygianScreenPoint,
    pub search_step_y_1080p: i32,
    pub max_retries: u8,
    pub click_found_team_times: u8,
    pub click_found_team_offset_x_1080p: i32,
    pub close_with_paimon_menu_when_not_found: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StygianScreenPoint {
    pub x_1080p: i32,
    pub y_1080p: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtCombatRule {
    pub initializes_combat_scenes_retry_attempts: u8,
    pub initializes_combat_scenes_retry_interval_ms: u64,
    pub selects_first_script_avatar: bool,
    pub after_avatar_switch_wait_ms: u64,
    pub pre_fight_move_forward_ms: u64,
    pub combat_script_loop_until_domain_end: bool,
    pub domain_end_detection: StygianDomainEndDetectionRule,
    pub releases_all_keys_after_fight: bool,
    pub fight_status_flag_is_set: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StygianDomainEndDetectionRule {
    pub white_cancel_asset: String,
    pub button_roi: StygianRoiRule,
    pub ocr_offset_x_scaled: i32,
    pub ocr_offset_y_scaled: i32,
    pub ocr_width_scaled: i32,
    pub ocr_height_multiplier: f64,
    pub text_contains: String,
    pub retry_attempts: u16,
    pub retry_interval_ms: u64,
    pub result_transition_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtRewardRule {
    pub battle_win_click_asset: String,
    pub battle_lose_click_asset: String,
    pub click_result_wait_ms: u64,
    pub move_forward_after_win_ms: u64,
    pub wait_after_move_forward_ms: u64,
    pub f_key_activation_text: String,
    pub lower_head_then_walk_when_no_activation_text: bool,
    pub leyline_interact_action: GenshinAction,
    pub leyline_interact_retry_attempts: u8,
    pub leyline_interact_retry_interval_ms: u64,
    pub reward_prompt_text: String,
    pub reward_prompt_transition_timeout_ms: u64,
    pub no_reward_prompt_continues_loop: bool,
    pub no_resin_texts: Vec<String>,
    pub sends_reward_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtResinRule {
    pub specify_resin_use: bool,
    pub configured_priority_list: Vec<String>,
    pub configured_priority_list_is_not_used_by_legacy_task: bool,
    pub default_auto_use_priority: Vec<String>,
    pub specified_records: Vec<AutoDomainResinUseRecord>,
    pub condensed_resin_name: String,
    pub original_resin_name: String,
    pub transient_resin_name: String,
    pub fragile_resin_name: String,
    pub default_insufficient_condition: String,
    pub original_resin_minimum_per_claim: u8,
    pub use_button_double_click_gap_ms: u64,
    pub continue_double_click_gap_ms: u64,
    pub continuation_transition_timeout_ms: u64,
    pub specified_unavailable_continues_without_exit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtExitRule {
    pub open_paimon_menu_until_exit_door_appears: bool,
    pub exit_door_asset: String,
    pub paimon_menu_asset: String,
    pub exit_complete_poll_interval_ms: u64,
    pub exit_complete_retry_attempts: u16,
    pub after_exit_complete_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoStygianOnslaughtArtifactSalvageRule {
    pub enabled: bool,
    pub max_artifact_star: String,
    pub invalid_star_falls_back_to: u8,
    pub starts_auto_artifact_salvage_task: bool,
    pub passes_java_script_none: bool,
    pub passes_artifact_set_filter_none: bool,
    pub passes_max_num_to_check_none: bool,
    pub passes_recognition_failure_policy_none: bool,
    pub quick_salvage_param: Option<AutoArtifactSalvageParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoStygianOnslaughtStep {
    pub phase: AutoStygianOnslaughtStepPhase,
    pub action: AutoStygianOnslaughtStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoStygianOnslaughtStepPhase {
    Startup,
    Navigate,
    SelectChallenge,
    BattleLoop,
    Reward,
    Exit,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoStygianOnslaughtStepAction {
    DestroyAutoFightAssetsAndParseCombatScriptBag,
    NotifyDomainStart,
    ReturnMainUi,
    OpenEventsMenu,
    FindStygianOnslaughtInEventMenu,
    ClickGoChallenge,
    ClickTeleport,
    InteractDomainEntrance,
    SwitchToHardMode,
    ConfirmSinglePlayerChallenge,
    WalkToKey,
    SelectBoss,
    SwitchConfiguredTeam,
    StartChallenge,
    InitializeCombatScenes,
    SelectCombatScriptAndFirstAvatar,
    MoveForwardBeforeFight,
    RunCombatScriptUntilResult,
    HandleBattleLose,
    HandleBattleWin,
    MoveAwayFromLeylineFlower,
    FindAndInteractLeylineFlower,
    DetectRewardPrompt,
    ChooseAndUseResin,
    ContinueOrExitByRemainingResin,
    ExitDomain,
    RunAutoArtifactSalvageWhenEnabled,
    NotifyDomainEnd,
}

pub fn plan_auto_stygian_onslaught(
    config: AutoStygianOnslaughtExecutionConfig,
) -> Result<AutoStygianOnslaughtExecutionPlan> {
    let specified_records = build_stygian_resin_records(&config.param)?;
    let selected_boss_num = selected_boss_num(config.param.boss_num);
    let artifact_salvage_rule =
        artifact_salvage_rule(&config.param, &config.artifact_salvage_max_artifact_star);

    Ok(AutoStygianOnslaughtExecutionPlan {
        task_key: AUTO_STYGIAN_ONSLAUGHT_TASK_KEY.to_string(),
        display_name: AUTO_STYGIAN_ONSLAUGHT_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: false,
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        param_rule: AutoStygianOnslaughtParamRule {
            boss_num: config.param.boss_num,
            selected_boss_num,
            invalid_boss_num_falls_back_to: 1,
            combat_script_bag_path: config.param.combat_script_bag_path.clone(),
            auto_team_strategy_directory: "User/AutoFight/".to_string(),
            fight_team_name: config.param.fight_team_name.clone(),
            task_runner_skips_main_ui_wait_due_to_legacy_name: true,
            task_still_returns_main_ui_on_start: true,
        },
        startup_rule: startup_rule(),
        state_machine_rule: state_machine_rule(),
        detector_rule: detector_rule(),
        navigation_rule: navigation_rule(),
        difficulty_rule: difficulty_rule(),
        boss_rule: boss_rule(config.param.boss_num),
        team_rule: team_rule(&config.param.fight_team_name),
        combat_rule: combat_rule(),
        reward_rule: reward_rule(),
        resin_rule: AutoStygianOnslaughtResinRule {
            specify_resin_use: config.param.specify_resin_use,
            configured_priority_list: config.param.resin_priority_list.clone(),
            configured_priority_list_is_not_used_by_legacy_task: true,
            default_auto_use_priority: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
            specified_records,
            condensed_resin_name: "浓缩树脂".to_string(),
            original_resin_name: "原粹树脂".to_string(),
            transient_resin_name: "须臾树脂".to_string(),
            fragile_resin_name: "脆弱树脂".to_string(),
            default_insufficient_condition: "CondensedResinCount <= 0 && OriginalResinCount < 20"
                .to_string(),
            original_resin_minimum_per_claim: 20,
            use_button_double_click_gap_ms: 60,
            continue_double_click_gap_ms: 60,
            continuation_transition_timeout_ms: 10_000,
            specified_unavailable_continues_without_exit: true,
        },
        exit_rule: exit_rule(),
        artifact_salvage_rule,
        steps: stygian_steps(config.param.auto_artifact_salvage),
        pending_native: pending_native(
            config.param.auto_artifact_salvage,
            !config.param.fight_team_name.is_empty(),
        ),
        param: config.param,
    })
}

pub fn build_stygian_resin_records(
    param: &AutoStygianOnslaughtParam,
) -> Result<Vec<AutoDomainResinUseRecord>> {
    if !param.specify_resin_use {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    push_resin_record(&mut records, "浓缩树脂", param.condensed_resin_use_count);
    push_resin_record(&mut records, "原粹树脂", param.original_resin_use_count);
    push_resin_record(&mut records, "须臾树脂", param.transient_resin_use_count);
    push_resin_record(&mut records, "脆弱树脂", param.fragile_resin_use_count);

    if records.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_STYGIAN_ONSLAUGHT_TASK_KEY.to_string(),
            message: "指定树脂刷取次数时至少需要配置一种树脂的刷取次数".to_string(),
        });
    }
    Ok(records)
}

fn push_resin_record(records: &mut Vec<AutoDomainResinUseRecord>, name: &str, count: i32) {
    if count > 0 {
        records.push(AutoDomainResinUseRecord {
            name: name.to_string(),
            remain_count: count,
            max_count: count,
        });
    }
}

fn apply_auto_stygian_config(
    param: &mut AutoStygianOnslaughtParam,
    config: &AutoStygianOnslaughtConfig,
) {
    param.boss_num = config.boss_num as i32;
    param.auto_artifact_salvage = config.auto_artifact_salvage;
    param.specify_resin_use = config.specify_resin_use;
    param.resin_priority_list = if config.resin_priority_list.is_empty() {
        vec!["浓缩树脂".to_string(), "原粹树脂".to_string()]
    } else {
        config.resin_priority_list.clone()
    };
    param.original_resin_use_count = config.original_resin_use_count as i32;
    param.condensed_resin_use_count = config.condensed_resin_use_count as i32;
    param.transient_resin_use_count = config.transient_resin_use_count as i32;
    param.fragile_resin_use_count = config.fragile_resin_use_count as i32;
    param.fight_team_name = config.fight_team_name.clone();
    param.combat_script_bag_path = combat_strategy_path(
        (!config.strategy_name.trim().is_empty()).then_some(config.strategy_name.as_str()),
    );
}

fn selected_boss_num(boss_num: i32) -> i32 {
    if (1..=3).contains(&boss_num) {
        boss_num
    } else {
        1
    }
}

fn startup_rule() -> AutoStygianOnslaughtStartupRule {
    AutoStygianOnslaughtStartupRule {
        destroys_auto_fight_assets_before_start: true,
        parses_combat_script_bag_before_start: true,
        builds_specified_resin_records_before_start: true,
        creates_lower_head_then_walk_to_task: LowerHeadThenWalkToDependencyRule {
            target_asset: "chest_tip.png".to_string(),
            timeout_ms: 20_000,
        },
        logs_screen_resolution: true,
        requires_16_to_9_resolution: true,
        warns_below_1920x1080: true,
        sends_domain_start_notification: true,
        sends_domain_end_notification: true,
        catches_task_cancelled_without_error: true,
        catches_other_exceptions_as_log_information: true,
        delay_before_artifact_salvage_ms: 3_000,
    }
}

fn state_machine_rule() -> AutoStygianOnslaughtStateMachineRule {
    AutoStygianOnslaughtStateMachineRule {
        initial_state: StygianState::Unknown,
        navigation_target_state: StygianState::BattleArena,
        battle_loop_limit: AUTO_STYGIAN_ONSLAUGHT_LOOP_LIMIT,
        transitions: vec![
            transition(
                StygianState::MainWorld,
                [StygianState::EventMenu, StygianState::StygianOnslaughtPage],
            ),
            transition(
                StygianState::EventMenu,
                [StygianState::StygianOnslaughtPage],
            ),
            transition(
                StygianState::StygianOnslaughtPage,
                [StygianState::TeleportMap, StygianState::DomainEntrance],
            ),
            transition(StygianState::TeleportMap, [StygianState::DomainEntrance]),
            transition(
                StygianState::DomainEntrance,
                [StygianState::DifficultySelect],
            ),
            transition(StygianState::DifficultySelect, [StygianState::DomainLobby]),
            transition(
                StygianState::DomainLobby,
                [StygianState::BossSelect, StygianState::LeylineFlowerPrompt],
            ),
            transition(StygianState::BossSelect, [StygianState::BattleArena]),
            transition(
                StygianState::BattleArena,
                [
                    StygianState::BattleResultWin,
                    StygianState::BattleResultLose,
                ],
            ),
            transition(StygianState::BattleResultWin, [StygianState::DomainLobby]),
            transition(StygianState::BattleResultLose, [StygianState::BossSelect]),
            transition(
                StygianState::LeylineFlowerPrompt,
                [StygianState::ResinSelect],
            ),
            transition(
                StygianState::ResinSelect,
                [StygianState::ContinueOrExit, StygianState::DomainLobby],
            ),
            transition(
                StygianState::ContinueOrExit,
                [StygianState::BattleArena, StygianState::MainWorld],
            ),
        ],
        unknown_state_returns_main_ui: true,
    }
}

fn transition<const N: usize>(from: StygianState, to: [StygianState; N]) -> StygianStateTransition {
    StygianStateTransition {
        from,
        to: to.into_iter().collect(),
    }
}

fn detector_rule() -> AutoStygianOnslaughtDetectorRule {
    AutoStygianOnslaughtDetectorRule {
        detectors: vec![
            detector(
                StygianState::ContinueOrExit,
                10,
                StygianStateDetectorKind::TemplatePair {
                    required_assets: vec![
                        AUTO_STYGIAN_ONSLAUGHT_CONFIRM_ASSET.to_string(),
                        AUTO_STYGIAN_ONSLAUGHT_EXIT_ASSET.to_string(),
                    ],
                    missing_assets: Vec::new(),
                },
            ),
            detector(
                StygianState::TeleportMap,
                20,
                StygianStateDetectorKind::Template {
                    asset: QUICK_TELEPORT_GO_TELEPORT.to_string(),
                },
            ),
            detector(
                StygianState::DomainLobby,
                30,
                StygianStateDetectorKind::TemplatePair {
                    required_assets: vec![
                        AUTO_STYGIAN_ONSLAUGHT_LEYLINE_DISORDER_ASSET.to_string(),
                        AUTO_STYGIAN_ONSLAUGHT_INVENTORY_ASSET.to_string(),
                    ],
                    missing_assets: Vec::new(),
                },
            ),
            detector(
                StygianState::BattleArena,
                40,
                StygianStateDetectorKind::TemplatePair {
                    required_assets: vec![AUTO_STYGIAN_ONSLAUGHT_LEYLINE_DISORDER_ASSET.to_string()],
                    missing_assets: vec![AUTO_STYGIAN_ONSLAUGHT_INVENTORY_ASSET.to_string()],
                },
            ),
            detector(
                StygianState::MainWorld,
                50,
                StygianStateDetectorKind::Template {
                    asset: AUTO_STYGIAN_ONSLAUGHT_PAIMON_MENU_ASSET.to_string(),
                },
            ),
            detector(
                StygianState::BattleResultWin,
                60,
                StygianStateDetectorKind::TemplateAndOcr {
                    asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CANCEL_ASSET.to_string(),
                    roi: StygianRoiRule::Relative {
                        x_ratio: 0.35,
                        y_ratio: 0.7,
                        width_ratio: 0.3,
                        height_ratio: 0.2,
                    },
                    contains_any: vec!["返回".to_string()],
                },
            ),
            detector(
                StygianState::BattleResultLose,
                70,
                StygianStateDetectorKind::TemplateAndOcr {
                    asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CONFIRM_ASSET.to_string(),
                    roi: StygianRoiRule::Relative {
                        x_ratio: 0.2,
                        y_ratio: 0.3,
                        width_ratio: 0.6,
                        height_ratio: 0.3,
                    },
                    contains_any: vec!["挑战失败".to_string(), "重新挑战".to_string()],
                },
            ),
            detector(
                StygianState::ResinSelect,
                80,
                StygianStateDetectorKind::Ocr {
                    roi: central_reward_roi(),
                    contains_all: vec!["地脉之花".to_string()],
                    contains_any: vec!["浓缩树脂".to_string(), "原粹树脂".to_string()],
                },
            ),
            detector(
                StygianState::LeylineFlowerPrompt,
                90,
                StygianStateDetectorKind::Ocr {
                    roi: central_reward_roi(),
                    contains_all: vec!["地脉之花".to_string()],
                    contains_any: Vec::new(),
                },
            ),
            detector(
                StygianState::BossSelect,
                100,
                StygianStateDetectorKind::Ocr {
                    roi: StygianRoiRule::Relative {
                        x_ratio: 0.5,
                        y_ratio: 0.0,
                        width_ratio: 0.5,
                        height_ratio: 1.0,
                    },
                    contains_all: vec!["角色预览".to_string(), "开始挑战".to_string()],
                    contains_any: Vec::new(),
                },
            ),
            detector(
                StygianState::DifficultySelect,
                110,
                StygianStateDetectorKind::Ocr {
                    roi: StygianRoiRule::Relative {
                        x_ratio: 0.5,
                        y_ratio: 0.7,
                        width_ratio: 0.5,
                        height_ratio: 0.3,
                    },
                    contains_all: vec!["单人挑战".to_string()],
                    contains_any: Vec::new(),
                },
            ),
            detector(
                StygianState::DomainEntrance,
                120,
                StygianStateDetectorKind::Ocr {
                    roi: StygianRoiRule::Absolute1080p(Rect {
                        x: 1223,
                        y: 510,
                        width: 153,
                        height: 56,
                    }),
                    contains_all: vec![AUTO_STYGIAN_ONSLAUGHT_EVENT_NAME.to_string()],
                    contains_any: Vec::new(),
                },
            ),
            detector(
                StygianState::EventMenu,
                130,
                StygianStateDetectorKind::Ocr {
                    roi: StygianRoiRule::Absolute1080p(Rect {
                        x: 125,
                        y: 142,
                        width: 113,
                        height: 28,
                    }),
                    contains_all: vec!["活动一览".to_string()],
                    contains_any: Vec::new(),
                },
            ),
            detector(
                StygianState::StygianOnslaughtPage,
                140,
                StygianStateDetectorKind::Ocr {
                    roi: StygianRoiRule::Relative {
                        x_ratio: 0.55,
                        y_ratio: 0.3,
                        width_ratio: 0.4,
                        height_ratio: 0.6,
                    },
                    contains_all: vec!["前往挑战".to_string()],
                    contains_any: Vec::new(),
                },
            ),
        ],
    }
}

fn detector(
    state: StygianState,
    order: u16,
    rule: StygianStateDetectorKind,
) -> StygianStateDetector {
    StygianStateDetector { state, order, rule }
}

fn central_reward_roi() -> StygianRoiRule {
    StygianRoiRule::Relative {
        x_ratio: 0.2,
        y_ratio: 0.2,
        width_ratio: 0.6,
        height_ratio: 0.6,
    }
}

fn navigation_rule() -> AutoStygianOnslaughtNavigationRule {
    AutoStygianOnslaughtNavigationRule {
        return_main_ui_before_navigation: true,
        open_event_menu_action: GenshinAction::OpenTheEventsMenu,
        open_event_menu_delay_ms: 500,
        event_menu_title_text: "活动一览".to_string(),
        event_list_roi_1080p: Rect {
            x: 195,
            y: 201,
            width: 296,
            height: 654,
        },
        event_list_drag: StygianDragRule {
            start_x_1080p: 343,
            start_y_1080p: 328,
            end_y_1080p: 728,
            step_y_1080p: 50,
            mouse_down_delay_ms: 100,
            step_delay_ms: 30,
            after_drag_wait_ms: 500,
        },
        event_search_attempts: 2,
        event_name: AUTO_STYGIAN_ONSLAUGHT_EVENT_NAME.to_string(),
        go_challenge_text: "前往挑战".to_string(),
        go_challenge_roi: StygianRoiRule::CutRight { width_ratio: 0.5 },
        teleport_button_asset: QUICK_TELEPORT_GO_TELEPORT.to_string(),
        teleport_click_delay_ms: 300,
        domain_entrance_text_roi_1080p: Rect {
            x: 1223,
            y: 510,
            width: 153,
            height: 56,
        },
        domain_entrance_interact_action: GenshinAction::PickUpOrInteract,
    }
}

fn difficulty_rule() -> AutoStygianOnslaughtDifficultyRule {
    AutoStygianOnslaughtDifficultyRule {
        target_difficulty: AUTO_STYGIAN_ONSLAUGHT_HARD_DIFFICULTY.to_string(),
        single_player_text: "单人挑战".to_string(),
        target_detect_roi: StygianRoiRule::CutRightTop {
            width_ratio: 0.5,
            height_ratio: 0.2,
        },
        ultimate_challenge_text: AUTO_STYGIAN_ONSLAUGHT_ULTIMATE_CHALLENGE.to_string(),
        normal_challenge_text: AUTO_STYGIAN_ONSLAUGHT_NORMAL_CHALLENGE.to_string(),
        normal_challenge_menu_click_offset_x_1080p: 400,
        retry_attempts: 10,
        retry_interval_ms: 500,
        mode_switch_wait_ms: 500,
        difficulty_click_wait_ms: 300,
        confirm_asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CONFIRM_ASSET.to_string(),
        after_confirm_wait_ms: 300,
        continue_when_switch_failed: true,
    }
}

fn boss_rule(requested_boss_num: i32) -> AutoStygianOnslaughtBossRule {
    AutoStygianOnslaughtBossRule {
        requested_boss_num,
        selected_boss_num: selected_boss_num(requested_boss_num),
        invalid_boss_num_falls_back_to: 1,
        boss_positions_1080p: vec![
            StygianBossPosition {
                boss_num: 1,
                x_1080p: 196,
                y_1080p: 346,
            },
            StygianBossPosition {
                boss_num: 2,
                x_1080p: 237,
                y_1080p: 541,
            },
            StygianBossPosition {
                boss_num: 3,
                x_1080p: 203,
                y_1080p: 728,
            },
        ],
        start_challenge_text: "开始挑战".to_string(),
        character_preview_text: "角色预览".to_string(),
        start_challenge_confirm_asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CONFIRM_ASSET.to_string(),
        after_start_wait_ms: 300,
    }
}

fn team_rule(fight_team_name: &str) -> AutoStygianOnslaughtTeamRule {
    AutoStygianOnslaughtTeamRule {
        enabled: !fight_team_name.is_empty(),
        fight_team_name: fight_team_name.to_string(),
        open_panel_text: "预设队伍".to_string(),
        panel_open_roi: StygianRoiRule::CutLeftTop {
            width_ratio: 0.15,
            height_ratio: 0.075,
        },
        panel_button_roi: StygianRoiRule::CutRightBottom {
            width_ratio: 0.3,
            height_ratio: 0.1,
        },
        open_retry_interval_ms: 300,
        search_start_point_1080p: StygianScreenPoint {
            x_1080p: 936,
            y_1080p: 150,
        },
        search_step_y_1080p: 100,
        max_retries: 30,
        click_found_team_times: 5,
        click_found_team_offset_x_1080p: 250,
        close_with_paimon_menu_when_not_found: true,
    }
}

fn combat_rule() -> AutoStygianOnslaughtCombatRule {
    AutoStygianOnslaughtCombatRule {
        initializes_combat_scenes_retry_attempts: 10,
        initializes_combat_scenes_retry_interval_ms: 500,
        selects_first_script_avatar: true,
        after_avatar_switch_wait_ms: 200,
        pre_fight_move_forward_ms: 1_200,
        combat_script_loop_until_domain_end: true,
        domain_end_detection: StygianDomainEndDetectionRule {
            white_cancel_asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CANCEL_ASSET.to_string(),
            button_roi: StygianRoiRule::Relative {
                x_ratio: 1.0 / 3.0,
                y_ratio: 0.78,
                width_ratio: 1.0 / 3.0,
                height_ratio: 0.22,
            },
            ocr_offset_x_scaled: 40,
            ocr_offset_y_scaled: -20,
            ocr_width_scaled: 270,
            ocr_height_multiplier: 2.0,
            text_contains: "返回".to_string(),
            retry_attempts: 300,
            retry_interval_ms: 1_000,
            result_transition_timeout_ms: 60_000,
        },
        releases_all_keys_after_fight: true,
        fight_status_flag_is_set: true,
    }
}

fn reward_rule() -> AutoStygianOnslaughtRewardRule {
    AutoStygianOnslaughtRewardRule {
        battle_win_click_asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CANCEL_ASSET.to_string(),
        battle_lose_click_asset: AUTO_STYGIAN_ONSLAUGHT_WHITE_CONFIRM_ASSET.to_string(),
        click_result_wait_ms: 300,
        move_forward_after_win_ms: 200,
        wait_after_move_forward_ms: 2_000,
        f_key_activation_text: "激活".to_string(),
        lower_head_then_walk_when_no_activation_text: true,
        leyline_interact_action: GenshinAction::PickUpOrInteract,
        leyline_interact_retry_attempts: 10,
        leyline_interact_retry_interval_ms: 300,
        reward_prompt_text: "地脉之花".to_string(),
        reward_prompt_transition_timeout_ms: 10_000,
        no_reward_prompt_continues_loop: true,
        no_resin_texts: vec!["数量不足".to_string(), "补充原粹树脂".to_string()],
        sends_reward_notification: true,
    }
}

fn exit_rule() -> AutoStygianOnslaughtExitRule {
    AutoStygianOnslaughtExitRule {
        open_paimon_menu_until_exit_door_appears: true,
        exit_door_asset: AUTO_STYGIAN_ONSLAUGHT_EXIT_DOOR_ASSET.to_string(),
        paimon_menu_asset: AUTO_STYGIAN_ONSLAUGHT_PAIMON_MENU_ASSET.to_string(),
        exit_complete_poll_interval_ms: 200,
        exit_complete_retry_attempts: 300,
        after_exit_complete_wait_ms: 1_000,
    }
}

fn artifact_salvage_rule(
    param: &AutoStygianOnslaughtParam,
    max_artifact_star: &str,
) -> AutoStygianOnslaughtArtifactSalvageRule {
    let star = max_artifact_star.trim().parse::<u8>().unwrap_or(4);
    let quick_salvage_param = param
        .auto_artifact_salvage
        .then(|| AutoArtifactSalvageParam::quick_only(star));
    AutoStygianOnslaughtArtifactSalvageRule {
        enabled: param.auto_artifact_salvage,
        max_artifact_star: max_artifact_star.to_string(),
        invalid_star_falls_back_to: 4,
        starts_auto_artifact_salvage_task: param.auto_artifact_salvage,
        passes_java_script_none: true,
        passes_artifact_set_filter_none: true,
        passes_max_num_to_check_none: true,
        passes_recognition_failure_policy_none: true,
        quick_salvage_param,
    }
}

fn stygian_steps(artifact_salvage_enabled: bool) -> Vec<AutoStygianOnslaughtStep> {
    let mut steps = vec![
        step(
            AutoStygianOnslaughtStepPhase::Startup,
            AutoStygianOnslaughtStepAction::DestroyAutoFightAssetsAndParseCombatScriptBag,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Startup,
            AutoStygianOnslaughtStepAction::NotifyDomainStart,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::ReturnMainUi,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::OpenEventsMenu,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::FindStygianOnslaughtInEventMenu,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::ClickGoChallenge,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::ClickTeleport,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Navigate,
            AutoStygianOnslaughtStepAction::InteractDomainEntrance,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::SwitchToHardMode,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::ConfirmSinglePlayerChallenge,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::WalkToKey,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::SelectBoss,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::SwitchConfiguredTeam,
        ),
        step(
            AutoStygianOnslaughtStepPhase::SelectChallenge,
            AutoStygianOnslaughtStepAction::StartChallenge,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::InitializeCombatScenes,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::SelectCombatScriptAndFirstAvatar,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::MoveForwardBeforeFight,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::RunCombatScriptUntilResult,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::HandleBattleLose,
        ),
        step(
            AutoStygianOnslaughtStepPhase::BattleLoop,
            AutoStygianOnslaughtStepAction::HandleBattleWin,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Reward,
            AutoStygianOnslaughtStepAction::MoveAwayFromLeylineFlower,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Reward,
            AutoStygianOnslaughtStepAction::FindAndInteractLeylineFlower,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Reward,
            AutoStygianOnslaughtStepAction::DetectRewardPrompt,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Reward,
            AutoStygianOnslaughtStepAction::ChooseAndUseResin,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Reward,
            AutoStygianOnslaughtStepAction::ContinueOrExitByRemainingResin,
        ),
        step(
            AutoStygianOnslaughtStepPhase::Exit,
            AutoStygianOnslaughtStepAction::ExitDomain,
        ),
    ];
    if artifact_salvage_enabled {
        steps.push(step(
            AutoStygianOnslaughtStepPhase::Cleanup,
            AutoStygianOnslaughtStepAction::RunAutoArtifactSalvageWhenEnabled,
        ));
    }
    steps.push(step(
        AutoStygianOnslaughtStepPhase::Cleanup,
        AutoStygianOnslaughtStepAction::NotifyDomainEnd,
    ));
    steps
}

fn step(
    phase: AutoStygianOnslaughtStepPhase,
    action: AutoStygianOnslaughtStepAction,
) -> AutoStygianOnslaughtStep {
    AutoStygianOnslaughtStep { phase, action }
}

fn pending_native(artifact_salvage_enabled: bool, team_switch_enabled: bool) -> Vec<String> {
    let mut pending = vec![
        "StateMachineBase runtime, BvPage locators, live capture, OCR, template matching, and transition timeouts".to_string(),
        "ReturnMainUiTask, OpenEventsMenu/OpenPaimonMenu/PickUpOrInteract/MoveForward input dispatch, mouse drag, and click execution".to_string(),
        "CombatScriptParser, CombatScenes team recognition, avatar switching, combat command loop, and AutoFight FightStatusFlag lifecycle".to_string(),
        "domain-end detection thread, cancellation coordination, key release, and result-state OCR".to_string(),
        "ResinStatus OCR, AutoDomainTask.PressUseResin button matching, continuation/exit clicks, and reward notifications".to_string(),
        "WalkToFTask and LowerHeadThenWalkToTask movement/camera execution for key and leyline flower interaction".to_string(),
    ];
    if team_switch_enabled {
        pending.push("preset-team panel OCR, scroll/drag selection, repeated team-name click, and fallback panel close".to_string());
    }
    if artifact_salvage_enabled {
        pending.push("post-domain AutoArtifactSalvage native quick-salvage execution remains disabled behind its Rust plan".to_string());
    }
    pending
}

fn overlay_param(param: &mut AutoStygianOnslaughtParam, value: &Value) {
    if let Some(value) = i32_member(value, ["bossNum", "BossNum", "boss_num"]) {
        param.boss_num = value;
    }
    if let Some(value) = bool_member(
        value,
        [
            "autoArtifactSalvage",
            "AutoArtifactSalvage",
            "auto_artifact_salvage",
        ],
    ) {
        param.auto_artifact_salvage = value;
    }
    if let Some(value) = bool_member(
        value,
        ["specifyResinUse", "SpecifyResinUse", "specify_resin_use"],
    ) {
        param.specify_resin_use = value;
    }
    if let Some(value) = string_vec_member(
        value,
        [
            "resinPriorityList",
            "ResinPriorityList",
            "resin_priority_list",
        ],
    ) {
        param.resin_priority_list = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "originalResinUseCount",
            "OriginalResinUseCount",
            "original_resin_use_count",
        ],
    ) {
        param.original_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "condensedResinUseCount",
            "CondensedResinUseCount",
            "condensed_resin_use_count",
        ],
    ) {
        param.condensed_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "transientResinUseCount",
            "TransientResinUseCount",
            "transient_resin_use_count",
        ],
    ) {
        param.transient_resin_use_count = value;
    }
    if let Some(value) = i32_member(
        value,
        [
            "fragileResinUseCount",
            "FragileResinUseCount",
            "fragile_resin_use_count",
        ],
    ) {
        param.fragile_resin_use_count = value;
    }
    if let Some(value) = string_member(value, ["fightTeamName", "FightTeamName", "fight_team_name"])
    {
        param.fight_team_name = value;
    }
    if let Some(value) = string_member(
        value,
        [
            "combatScriptBagPath",
            "CombatScriptBagPath",
            "combat_script_bag_path",
        ],
    ) {
        param.combat_script_bag_path = value;
    } else if let Some(value) = string_member(
        value,
        [
            "strategyName",
            "StrategyName",
            "strategy_name",
            "combatStrategyName",
            "CombatStrategyName",
        ],
    ) {
        param.combat_script_bag_path =
            combat_strategy_path((!value.trim().is_empty()).then_some(value.as_str()));
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    let width =
        u64_member(value, ["captureWidth", "CaptureWidth", "capture_width"]).or_else(|| {
            value
                .get("captureSize")
                .and_then(|size| u64_member(size, ["width", "Width"]))
        })? as u32;
    let height =
        u64_member(value, ["captureHeight", "CaptureHeight", "capture_height"]).or_else(|| {
            value
                .get("captureSize")
                .and_then(|size| u64_member(size, ["height", "Height"]))
        })? as u32;
    Some(Size::new(width, height))
}

fn bool_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<bool> {
    member(value, keys).and_then(|value| value.as_bool())
}

fn i32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<i32> {
    member(value, keys).and_then(|value| {
        value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .or_else(|| value.as_str().and_then(|value| value.trim().parse().ok()))
    })
}

fn u64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u64> {
    member(value, keys).and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|value| value.trim().parse().ok()))
    })
}

fn f64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<f64> {
    member(value, keys).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|value| value.trim().parse().ok()))
    })
}

fn string_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<String> {
    member(value, keys).and_then(|value| value.as_str().map(str::to_string))
}

fn string_vec_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<Vec<String>> {
    member(value, keys).and_then(|value| {
        value.as_array().map(|array| {
            array
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
    })
}

fn member<'a, const N: usize>(value: &'a Value, keys: [&str; N]) -> Option<&'a Value> {
    keys.into_iter().find_map(|key| value.get(key))
}
