use bgi_core::SkillCdConfig;
use bgi_vision::{BgrPixel, Rect, Size};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const SKILL_CD_TASK_KEY: &str = "SkillCd";
pub const SKILL_CD_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const SKILL_CD_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const SKILL_CD_OVERLAY_KEY: &str = "SkillCdText";
pub const SKILL_CD_OCR_REGEX: &str = r"\d+(\.\d+)?";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: SkillCdConfigRule,
    pub clamp_rule: SkillCdClampRule,
    pub context_rule: SkillCdContextRule,
    pub team_sync_rule: SkillCdTeamSyncRule,
    pub input_rule: SkillCdInputRule,
    pub cooldown_rule: SkillCdCooldownRule,
    pub overlay_rule: SkillCdOverlayRule,
    pub steps: Vec<SkillCdTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCdExecutionConfig {
    pub capture_size: Size,
    pub skill_cd_config: SkillCdConfig,
    pub trigger_interval_ms: u64,
}

impl Default for SkillCdExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                SKILL_CD_DEFAULT_CAPTURE_WIDTH,
                SKILL_CD_DEFAULT_CAPTURE_HEIGHT,
            ),
            skill_cd_config: SkillCdConfig::default(),
            trigger_interval_ms: 50,
        }
    }
}

impl SkillCdExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let skill_cd_value = value
            .get("skillCdConfig")
            .or_else(|| value.get("SkillCdConfig"))
            .or_else(|| value.get("skill_cd_config"))
            .unwrap_or(value);
        config.skill_cd_config = serde_json::from_value(skill_cd_value.clone()).unwrap_or_default();

        if let Some(trigger_interval_ms) = u64_member(
            value,
            [
                "triggerInterval",
                "TriggerInterval",
                "triggerIntervalMs",
                "trigger_interval_ms",
            ],
        ) {
            config.trigger_interval_ms = trigger_interval_ms;
        }

        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdConfigRule {
    pub enabled: bool,
    pub custom_cd_rules: Vec<SkillCdCustomCdRule>,
    pub trigger_on_skill_use: bool,
    pub hide_when_zero: bool,
    pub p_x: f64,
    pub p_y: f64,
    pub gap: f64,
    pub scale: f64,
    pub background_normal_color: String,
    pub text_normal_color: String,
    pub background_ready_color: String,
    pub text_ready_color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdCustomCdRule {
    pub role_name: String,
    pub cd_value_text: Option<String>,
    pub cd_value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdClampRule {
    pub p_x: SkillCdRange,
    pub p_y: SkillCdRange,
    pub gap: SkillCdRange,
    pub scale: SkillCdRange,
    pub blank_color_fallbacks_to_defaults: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdContextRule {
    pub accepted_contexts: Vec<SkillCdGameContext>,
    pub leave_debounce_ms: u64,
    pub enter_stabilization_ms: u64,
    pub multi_game_auto_disables_trigger: bool,
    pub disabled_clears_overlay: bool,
    pub leave_context_clears_frame_cache: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdGameContext {
    MainUi,
    Domain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdTeamSyncRule {
    pub sync_on_context_enter: bool,
    pub initial_delay_ms: u64,
    pub recent_index_press_delay_ms: u64,
    pub require_initialized_combat_scenes: bool,
    pub reset_cds_when_team_changes: bool,
    pub clear_team_when_sync_fails: bool,
    pub overlay_requires_exact_avatar_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdInputRule {
    pub elemental_skill_binding_source: String,
    pub trigger_on_skill_use: bool,
    pub skill_use_recent_window_ms: u64,
    pub digit_keys: Vec<String>,
    pub tracks_key_down_edges: bool,
    pub frame_source: SkillCdFrameSource,
    pub last_press_index_time_update: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdFrameSource {
    PenultimateImageThenLastImage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdCooldownRule {
    pub slot_count: usize,
    pub countdown_max_delta_seconds: f64,
    pub clamp_to_zero: bool,
    pub active_index_detector: String,
    pub skill_ready_detector: String,
    pub e_cooldown_roi: Rect,
    pub ocr_rule: SkillCdOcrRule,
    pub fallback_rule: SkillCdFallbackRule,
    pub default_avatar_index_rects: Vec<Rect>,
    pub switch_protect_window_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdFallbackResolution {
    pub seconds: f64,
    pub source: SkillCdFallbackSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdFallbackSource {
    CustomRule,
    CustomRuleDefaultAvatarMap,
    CustomRuleMissingDefaultAvatar,
    DefaultAvatarMap,
    MissingDefaultAvatar,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdRuntimeState {
    pub cooldowns: [f64; 4],
    pub prev_digit_keys: [bool; 4],
    pub prev_elemental_skill_key: bool,
    pub last_elemental_skill_press_elapsed_ms: Option<u64>,
    pub last_press_index_elapsed_ms: Option<u64>,
    pub was_in_context: bool,
    pub context_enter_elapsed_ms: Option<u64>,
    pub context_leave_elapsed_ms: Option<u64>,
    pub last_active_index: i32,
    pub last_switch_from_slot: Option<usize>,
    pub last_switch_elapsed_ms: Option<u64>,
    pub is_syncing_team: bool,
    pub team_avatar_names: [String; 4],
    pub last_frame_available: bool,
    pub penultimate_frame_available: bool,
}

impl Default for SkillCdRuntimeState {
    fn default() -> Self {
        Self {
            cooldowns: [0.0; 4],
            prev_digit_keys: [false; 4],
            prev_elemental_skill_key: false,
            last_elemental_skill_press_elapsed_ms: None,
            last_press_index_elapsed_ms: None,
            was_in_context: false,
            context_enter_elapsed_ms: None,
            context_leave_elapsed_ms: None,
            last_active_index: -1,
            last_switch_from_slot: None,
            last_switch_elapsed_ms: None,
            is_syncing_team: false,
            team_avatar_names: empty_skill_cd_team_names(),
            last_frame_available: false,
            penultimate_frame_available: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdTickObservation {
    pub delta_seconds: f64,
    pub raw_in_context: bool,
    pub multi_game_detected: bool,
    pub elemental_skill_key_down: bool,
    pub digit_key_down: [bool; 4],
    pub current_frame_available: bool,
    pub active_index: Option<usize>,
    pub ocr_cooldown_seconds: Option<f64>,
    pub visual_skill_ready: Option<bool>,
    pub fallback_cooldown: Option<SkillCdFallbackResolution>,
    pub team_sync: Option<SkillCdTeamSyncUpdate>,
    pub screen_scale_factor: f64,
}

impl Default for SkillCdTickObservation {
    fn default() -> Self {
        Self {
            delta_seconds: 0.05,
            raw_in_context: true,
            multi_game_detected: false,
            elemental_skill_key_down: false,
            digit_key_down: [false; 4],
            current_frame_available: true,
            active_index: None,
            ocr_cooldown_seconds: None,
            visual_skill_ready: None,
            fallback_cooldown: None,
            team_sync: None,
            screen_scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdTeamSyncUpdate {
    pub initialized: bool,
    pub avatar_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdTickReduction {
    pub state: SkillCdRuntimeState,
    pub effects: Vec<SkillCdTickEffect>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdTickExecutionReport {
    pub observation: SkillCdTickObservation,
    pub effects: Vec<SkillCdTickEffect>,
    pub executed_actions: Vec<SkillCdExecutedAction>,
    pub state: SkillCdRuntimeState,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SkillCdExecutedAction {
    ClearOverlay {
        reason: SkillCdClearOverlayReason,
    },
    AutoDisableTrigger,
    RequestTeamSync {
        delay_ms: u64,
        wait_after_recent_index_press_ms: u64,
    },
    ClearFrameCache,
    RotateFrameCache,
    RenderOverlay {
        entries: Vec<SkillCdOverlayEntry>,
    },
}

pub trait SkillCdRuntime {
    fn observe_skill_cd_tick(
        &mut self,
        plan: &SkillCdExecutionPlan,
        state: &SkillCdRuntimeState,
    ) -> SkillCdTickObservation;

    fn clear_skill_cd_overlay(
        &mut self,
        plan: &SkillCdExecutionPlan,
        reason: SkillCdClearOverlayReason,
    );

    fn auto_disable_skill_cd_trigger(&mut self, plan: &SkillCdExecutionPlan);

    fn request_skill_cd_team_sync(
        &mut self,
        plan: &SkillCdExecutionPlan,
        delay_ms: u64,
        wait_after_recent_index_press_ms: u64,
    );

    fn clear_skill_cd_frame_cache(&mut self, plan: &SkillCdExecutionPlan);

    fn rotate_skill_cd_frame_cache(&mut self, plan: &SkillCdExecutionPlan);

    fn render_skill_cd_overlay(
        &mut self,
        plan: &SkillCdExecutionPlan,
        entries: &[SkillCdOverlayEntry],
    );
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SkillCdTickEffect {
    ClearOverlay {
        reason: SkillCdClearOverlayReason,
    },
    DecrementCooldowns {
        delta_seconds: f64,
    },
    AutoDisableTrigger,
    RequestTeamSync {
        delay_ms: u64,
        wait_after_recent_index_press_ms: u64,
    },
    TeamSyncApplied {
        team_changed: bool,
    },
    ClearTeam,
    SkipTick {
        reason: SkillCdSkipReason,
    },
    ClearFrameCache,
    PollKeyStates,
    RecordElementalSkillPress,
    IdentifyActiveIndex {
        trigger: SkillCdActionTrigger,
        pressed_target: Option<usize>,
    },
    SetCooldown {
        slot: usize,
        seconds: f64,
        source: SkillCdCooldownUpdateSource,
    },
    KeepCooldown {
        slot: usize,
        reason: SkillCdKeepCooldownReason,
    },
    RotateFrameCache,
    RenderOverlay {
        entries: Vec<SkillCdOverlayEntry>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdClearOverlayReason {
    Disabled,
    LeftContext,
    SyncingTeam,
    InvalidAvatarCount,
    EmptyTextList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdSkipReason {
    Disabled,
    MultiGameDetected,
    OutOfContext,
    EnterStabilization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdActionTrigger {
    DigitKey,
    ElementalSkillKey,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SkillCdCooldownUpdateSource {
    Ocr,
    Fallback(SkillCdFallbackSource),
    VisualReadyWithoutExistingCooldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdKeepCooldownReason {
    VisualReadyWithExistingCooldown,
    OcrMissingAndNoFallback,
    ActiveSlotMatchesPressedTarget,
    MissingActiveIndex,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdOverlayEntry {
    pub slot: usize,
    pub text: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdOcrRule {
    pub white_lower_bgr: BgrPixel,
    pub white_upper_bgr: BgrPixel,
    pub ocr_engine: String,
    pub regex: String,
    pub compensation_trigger_intervals: u64,
    pub compensation_seconds: f64,
    pub accepted_min_exclusive: f64,
    pub accepted_max_exclusive: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdFallbackRule {
    pub custom_rule_precedes_default_avatar_map: bool,
    pub empty_custom_value_uses_default_avatar_map: bool,
    pub matched_custom_name_without_default_returns_zero: bool,
    pub default_source: String,
    pub only_applies_when_skill_used_recently: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCdOverlayRule {
    pub overlay_key: String,
    pub position_x: f64,
    pub position_y: f64,
    pub gap: f64,
    pub scale: f64,
    pub hide_when_zero: bool,
    pub requires_exact_avatar_count: usize,
    pub clear_while_syncing_team: bool,
    pub clear_when_empty: bool,
    pub cd_text_format: String,
    pub position_round_decimal_places: u8,
    pub screen_scale_source: String,
    pub avatar_side_icon_rects: Vec<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCdTickStep {
    pub phase: SkillCdTickPhase,
    pub condition: SkillCdTickCondition,
    pub action: SkillCdTickAction,
}

impl SkillCdTickStep {
    fn new(
        phase: SkillCdTickPhase,
        condition: SkillCdTickCondition,
        action: SkillCdTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdTickPhase {
    Disabled,
    Countdown,
    ContextGate,
    TeamSync,
    InputPolling,
    ActionTrigger,
    FrameCache,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdTickCondition {
    WhenDisabled,
    WhenDeltaBetweenZeroAndFiveSeconds,
    WhenMainUiOrDomain,
    WhenMultiGameDetected,
    WhenLeavingContext,
    WhenEnteringContext,
    WhenEnterStabilizationNotElapsed,
    WhenDigitKeyDownEdge,
    WhenElementalSkillKeyDownAndEnabled,
    WhenActiveIndexDetected,
    WhenOcrCooldownRecognized,
    WhenOcrFailsAndFallbackApplies,
    EveryTick,
    WhenOverlayShouldRender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkillCdTickAction {
    ClearOverlay,
    DecrementCooldowns,
    KeepContextForDebounceWindow,
    AutoDisableTrigger,
    ClearFrameCacheAndHideOverlay,
    SyncCombatScenesAfterDelay,
    SkipTick,
    PollKeyStates,
    IdentifyActiveIndex,
    RecognizeCooldownWithOcr,
    ApplyFallbackCooldown,
    RotateLastTwoFrames,
    RenderTextOverlay,
}

pub fn plan_skill_cd(config: SkillCdExecutionConfig) -> SkillCdExecutionPlan {
    let capture_size = config.capture_size;
    let skill_cd_config = config.skill_cd_config;
    let trigger_interval_ms = config.trigger_interval_ms;
    let config_rule = SkillCdConfigRule {
        enabled: skill_cd_config.enabled,
        custom_cd_rules: custom_cd_rules(&skill_cd_config.custom_cd_list),
        trigger_on_skill_use: skill_cd_config.trigger_on_skill_use,
        hide_when_zero: skill_cd_config.hide_when_zero,
        p_x: clamp(skill_cd_config.p_x, 0.0, 1920.0),
        p_y: clamp(skill_cd_config.p_y, 0.0, 1080.0),
        gap: clamp(skill_cd_config.gap, 0.0, 200.0),
        scale: clamp(skill_cd_config.scale, 0.0, 10.0),
        background_normal_color: fallback_color(
            skill_cd_config.background_normal_color,
            "#FFFFFFFF",
        ),
        text_normal_color: fallback_color(skill_cd_config.text_normal_color, "#DA4A23FF"),
        background_ready_color: fallback_color(skill_cd_config.background_ready_color, "#FFFFFFFF"),
        text_ready_color: fallback_color(skill_cd_config.text_ready_color, "#5DCC17FF"),
    };

    SkillCdExecutionPlan {
        task_key: SKILL_CD_TASK_KEY.to_string(),
        display_name: "Skill Cooldown".to_string(),
        capture_size,
        clamp_rule: SkillCdClampRule {
            p_x: SkillCdRange {
                min: 0.0,
                max: 1920.0,
            },
            p_y: SkillCdRange {
                min: 0.0,
                max: 1080.0,
            },
            gap: SkillCdRange {
                min: 0.0,
                max: 200.0,
            },
            scale: SkillCdRange {
                min: 0.0,
                max: 10.0,
            },
            blank_color_fallbacks_to_defaults: true,
        },
        context_rule: SkillCdContextRule {
            accepted_contexts: vec![SkillCdGameContext::MainUi, SkillCdGameContext::Domain],
            leave_debounce_ms: 800,
            enter_stabilization_ms: 500,
            multi_game_auto_disables_trigger: true,
            disabled_clears_overlay: true,
            leave_context_clears_frame_cache: true,
        },
        team_sync_rule: SkillCdTeamSyncRule {
            sync_on_context_enter: true,
            initial_delay_ms: 500,
            recent_index_press_delay_ms: 1_100,
            require_initialized_combat_scenes: true,
            reset_cds_when_team_changes: true,
            clear_team_when_sync_fails: true,
            overlay_requires_exact_avatar_count: 4,
        },
        input_rule: SkillCdInputRule {
            elemental_skill_binding_source: "KeyBindingsConfig.ElementalSkill".to_string(),
            trigger_on_skill_use: config_rule.trigger_on_skill_use,
            skill_use_recent_window_ms: 1_100,
            digit_keys: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
            tracks_key_down_edges: true,
            frame_source: SkillCdFrameSource::PenultimateImageThenLastImage,
            last_press_index_time_update: "updated during digit-key polling like SkillCdTrigger"
                .to_string(),
        },
        cooldown_rule: SkillCdCooldownRule {
            slot_count: 4,
            countdown_max_delta_seconds: 5.0,
            clamp_to_zero: true,
            active_index_detector: "PartyAvatarSideIndexHelper.GetAvatarIndexIsActiveWithContext"
                .to_string(),
            skill_ready_detector: "Bv.IsSkillReady(activeIdx, false)".to_string(),
            e_cooldown_roi: e_cooldown_rect(capture_size),
            ocr_rule: SkillCdOcrRule {
                white_lower_bgr: BgrPixel {
                    b: 230,
                    g: 230,
                    r: 230,
                },
                white_upper_bgr: BgrPixel {
                    b: 255,
                    g: 255,
                    r: 255,
                },
                ocr_engine: "Paddle.OcrWithoutDetector".to_string(),
                regex: SKILL_CD_OCR_REGEX.to_string(),
                compensation_trigger_intervals: 2,
                compensation_seconds: (trigger_interval_ms * 2) as f64 / 1000.0,
                accepted_min_exclusive: 0.0,
                accepted_max_exclusive: 60.0,
            },
            fallback_rule: SkillCdFallbackRule {
                custom_rule_precedes_default_avatar_map: true,
                empty_custom_value_uses_default_avatar_map: true,
                matched_custom_name_without_default_returns_zero: true,
                default_source: "DefaultAutoFightConfig.CombatAvatarMap.SkillCd".to_string(),
                only_applies_when_skill_used_recently: true,
            },
            default_avatar_index_rects: avatar_index_rects(capture_size),
            switch_protect_window_ms: 1_000,
        },
        overlay_rule: SkillCdOverlayRule {
            overlay_key: SKILL_CD_OVERLAY_KEY.to_string(),
            position_x: config_rule.p_x,
            position_y: config_rule.p_y,
            gap: config_rule.gap,
            scale: config_rule.scale,
            hide_when_zero: config_rule.hide_when_zero,
            requires_exact_avatar_count: 4,
            clear_while_syncing_team: true,
            clear_when_empty: true,
            cd_text_format: "F1".to_string(),
            position_round_decimal_places: 1,
            screen_scale_source:
                "SystemInfo.GameScreenSize.Width / SystemInfo.ScaleMax1080PCaptureRect.Width"
                    .to_string(),
            avatar_side_icon_rects: avatar_side_icon_rects(capture_size),
        },
        config_rule,
        steps: skill_cd_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop adapters for live MainUi/Domain recognition and multiplayer BV detection"
                .to_string(),
            "desktop adapter for async RunnerContext.TrySyncCombatScenesSilent party synchronization"
                .to_string(),
            "desktop adapters for GetAsyncKeyState, frame cache ownership, active-avatar detection, Paddle OCR, and SkillCdText overlay rendering".to_string(),
        ],
    }
}

fn skill_cd_steps() -> Vec<SkillCdTickStep> {
    vec![
        SkillCdTickStep::new(
            SkillCdTickPhase::Disabled,
            SkillCdTickCondition::WhenDisabled,
            SkillCdTickAction::ClearOverlay,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::Countdown,
            SkillCdTickCondition::WhenDeltaBetweenZeroAndFiveSeconds,
            SkillCdTickAction::DecrementCooldowns,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ContextGate,
            SkillCdTickCondition::WhenMainUiOrDomain,
            SkillCdTickAction::KeepContextForDebounceWindow,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ContextGate,
            SkillCdTickCondition::WhenMultiGameDetected,
            SkillCdTickAction::AutoDisableTrigger,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ContextGate,
            SkillCdTickCondition::WhenLeavingContext,
            SkillCdTickAction::ClearFrameCacheAndHideOverlay,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::TeamSync,
            SkillCdTickCondition::WhenEnteringContext,
            SkillCdTickAction::SyncCombatScenesAfterDelay,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::TeamSync,
            SkillCdTickCondition::WhenEnterStabilizationNotElapsed,
            SkillCdTickAction::SkipTick,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::InputPolling,
            SkillCdTickCondition::EveryTick,
            SkillCdTickAction::PollKeyStates,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ActionTrigger,
            SkillCdTickCondition::WhenDigitKeyDownEdge,
            SkillCdTickAction::IdentifyActiveIndex,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ActionTrigger,
            SkillCdTickCondition::WhenElementalSkillKeyDownAndEnabled,
            SkillCdTickAction::IdentifyActiveIndex,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ActionTrigger,
            SkillCdTickCondition::WhenActiveIndexDetected,
            SkillCdTickAction::RecognizeCooldownWithOcr,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ActionTrigger,
            SkillCdTickCondition::WhenOcrCooldownRecognized,
            SkillCdTickAction::RecognizeCooldownWithOcr,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::ActionTrigger,
            SkillCdTickCondition::WhenOcrFailsAndFallbackApplies,
            SkillCdTickAction::ApplyFallbackCooldown,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::FrameCache,
            SkillCdTickCondition::EveryTick,
            SkillCdTickAction::RotateLastTwoFrames,
        ),
        SkillCdTickStep::new(
            SkillCdTickPhase::Overlay,
            SkillCdTickCondition::WhenOverlayShouldRender,
            SkillCdTickAction::RenderTextOverlay,
        ),
    ]
}

pub fn reduce_skill_cd_tick(
    plan: &SkillCdExecutionPlan,
    state: &SkillCdRuntimeState,
    observation: SkillCdTickObservation,
) -> SkillCdTickReduction {
    let mut next = state.clone();
    let mut effects = Vec::new();
    let delta_ms = seconds_to_ms(observation.delta_seconds);

    increment_skill_cd_elapsed(&mut next.last_elemental_skill_press_elapsed_ms, delta_ms);
    increment_skill_cd_elapsed(&mut next.last_press_index_elapsed_ms, delta_ms);
    increment_skill_cd_elapsed(&mut next.last_switch_elapsed_ms, delta_ms);
    if let Some(elapsed) = next.context_enter_elapsed_ms.as_mut() {
        *elapsed = elapsed.saturating_add(delta_ms);
    }
    if let Some(elapsed) = next.context_leave_elapsed_ms.as_mut() {
        *elapsed = elapsed.saturating_add(delta_ms);
    }

    if !plan.config_rule.enabled {
        effects.push(SkillCdTickEffect::ClearOverlay {
            reason: SkillCdClearOverlayReason::Disabled,
        });
        effects.push(SkillCdTickEffect::SkipTick {
            reason: SkillCdSkipReason::Disabled,
        });
        return SkillCdTickReduction {
            state: next,
            effects,
        };
    }

    if observation.delta_seconds >= 0.0
        && observation.delta_seconds < plan.cooldown_rule.countdown_max_delta_seconds
    {
        let mut decremented = false;
        for cooldown in &mut next.cooldowns {
            if *cooldown > 0.0 {
                *cooldown -= observation.delta_seconds;
                if plan.cooldown_rule.clamp_to_zero && *cooldown < 0.0 {
                    *cooldown = 0.0;
                }
                decremented = true;
            }
        }
        if decremented {
            effects.push(SkillCdTickEffect::DecrementCooldowns {
                delta_seconds: observation.delta_seconds,
            });
        }
    }

    if observation.raw_in_context {
        if observation.multi_game_detected && plan.context_rule.multi_game_auto_disables_trigger {
            effects.push(SkillCdTickEffect::AutoDisableTrigger);
            effects.push(SkillCdTickEffect::SkipTick {
                reason: SkillCdSkipReason::MultiGameDetected,
            });
            return SkillCdTickReduction {
                state: next,
                effects,
            };
        }

        next.context_leave_elapsed_ms = None;
        if !next.was_in_context {
            next.was_in_context = true;
            next.context_enter_elapsed_ms = Some(0);
            next.is_syncing_team = true;
            if plan.team_sync_rule.sync_on_context_enter {
                let wait_after_recent_index_press_ms = next
                    .last_press_index_elapsed_ms
                    .map(|elapsed| {
                        plan.team_sync_rule
                            .recent_index_press_delay_ms
                            .saturating_sub(
                                elapsed.saturating_add(plan.team_sync_rule.initial_delay_ms),
                            )
                    })
                    .unwrap_or(0);
                effects.push(SkillCdTickEffect::RequestTeamSync {
                    delay_ms: plan
                        .team_sync_rule
                        .initial_delay_ms
                        .saturating_add(wait_after_recent_index_press_ms),
                    wait_after_recent_index_press_ms,
                });
            }
        }
    } else {
        if next.was_in_context && next.context_leave_elapsed_ms.is_none() {
            next.context_leave_elapsed_ms = Some(0);
        }
        let in_debounce_window = next
            .context_leave_elapsed_ms
            .map(|elapsed| elapsed < plan.context_rule.leave_debounce_ms)
            .unwrap_or(false);

        if !in_debounce_window {
            if next.was_in_context {
                next.was_in_context = false;
                next.context_enter_elapsed_ms = None;
                next.last_active_index = -1;
                effects.push(SkillCdTickEffect::ClearOverlay {
                    reason: SkillCdClearOverlayReason::LeftContext,
                });
            }
            if plan.context_rule.leave_context_clears_frame_cache
                && (next.last_frame_available || next.penultimate_frame_available)
            {
                next.last_frame_available = false;
                next.penultimate_frame_available = false;
                effects.push(SkillCdTickEffect::ClearFrameCache);
            }
            effects.push(SkillCdTickEffect::SkipTick {
                reason: SkillCdSkipReason::OutOfContext,
            });
            return SkillCdTickReduction {
                state: next,
                effects,
            };
        }
    }

    if let Some(sync) = observation.team_sync.as_ref() {
        apply_skill_cd_team_sync(plan, &mut next, sync, &mut effects);
    }

    if next
        .context_enter_elapsed_ms
        .map(|elapsed| elapsed < plan.context_rule.enter_stabilization_ms)
        .unwrap_or(false)
    {
        effects.push(SkillCdTickEffect::SkipTick {
            reason: SkillCdSkipReason::EnterStabilization,
        });
        return SkillCdTickReduction {
            state: next,
            effects,
        };
    }

    effects.push(SkillCdTickEffect::PollKeyStates);

    let e_key_edge = observation.elemental_skill_key_down && !next.prev_elemental_skill_key;
    if e_key_edge {
        next.last_elemental_skill_press_elapsed_ms = Some(0);
        effects.push(SkillCdTickEffect::RecordElementalSkillPress);
    }
    next.prev_elemental_skill_key = observation.elemental_skill_key_down;

    let mut pressed_target = None;
    for (slot, is_down) in observation.digit_key_down.into_iter().enumerate() {
        if is_down && !next.prev_digit_keys[slot] {
            pressed_target = Some(slot);
        }
        next.prev_digit_keys[slot] = is_down;
    }
    next.last_press_index_elapsed_ms = Some(0);

    if next.last_frame_available {
        if let Some(target) = pressed_target {
            effects.push(SkillCdTickEffect::IdentifyActiveIndex {
                trigger: SkillCdActionTrigger::DigitKey,
                pressed_target: Some(target),
            });
            apply_skill_cd_action_trigger(
                plan,
                &mut next,
                &observation,
                Some(target),
                &mut effects,
            );
        }

        if next.prev_elemental_skill_key && plan.input_rule.trigger_on_skill_use {
            effects.push(SkillCdTickEffect::IdentifyActiveIndex {
                trigger: SkillCdActionTrigger::ElementalSkillKey,
                pressed_target,
            });
            apply_skill_cd_action_trigger(
                plan,
                &mut next,
                &observation,
                pressed_target,
                &mut effects,
            );
        }
    }

    next.penultimate_frame_available = next.last_frame_available;
    next.last_frame_available = observation.current_frame_available;
    if observation.current_frame_available {
        effects.push(SkillCdTickEffect::RotateFrameCache);
    }

    push_skill_cd_overlay_effect(plan, &next, observation.screen_scale_factor, &mut effects);

    SkillCdTickReduction {
        state: next,
        effects,
    }
}

pub fn execute_skill_cd_tick_plan<R: SkillCdRuntime>(
    plan: &SkillCdExecutionPlan,
    state: &SkillCdRuntimeState,
    runtime: &mut R,
) -> SkillCdTickExecutionReport {
    let observation = runtime.observe_skill_cd_tick(plan, state);
    let reduction = reduce_skill_cd_tick(plan, state, observation.clone());
    let mut executed_actions = Vec::new();

    for effect in &reduction.effects {
        match effect {
            SkillCdTickEffect::ClearOverlay { reason } => {
                runtime.clear_skill_cd_overlay(plan, *reason);
                executed_actions.push(SkillCdExecutedAction::ClearOverlay { reason: *reason });
            }
            SkillCdTickEffect::AutoDisableTrigger => {
                runtime.auto_disable_skill_cd_trigger(plan);
                executed_actions.push(SkillCdExecutedAction::AutoDisableTrigger);
            }
            SkillCdTickEffect::RequestTeamSync {
                delay_ms,
                wait_after_recent_index_press_ms,
            } => {
                runtime.request_skill_cd_team_sync(
                    plan,
                    *delay_ms,
                    *wait_after_recent_index_press_ms,
                );
                executed_actions.push(SkillCdExecutedAction::RequestTeamSync {
                    delay_ms: *delay_ms,
                    wait_after_recent_index_press_ms: *wait_after_recent_index_press_ms,
                });
            }
            SkillCdTickEffect::ClearFrameCache => {
                runtime.clear_skill_cd_frame_cache(plan);
                executed_actions.push(SkillCdExecutedAction::ClearFrameCache);
            }
            SkillCdTickEffect::RotateFrameCache => {
                runtime.rotate_skill_cd_frame_cache(plan);
                executed_actions.push(SkillCdExecutedAction::RotateFrameCache);
            }
            SkillCdTickEffect::RenderOverlay { entries } => {
                runtime.render_skill_cd_overlay(plan, entries);
                executed_actions.push(SkillCdExecutedAction::RenderOverlay {
                    entries: entries.clone(),
                });
            }
            SkillCdTickEffect::DecrementCooldowns { .. }
            | SkillCdTickEffect::TeamSyncApplied { .. }
            | SkillCdTickEffect::ClearTeam
            | SkillCdTickEffect::SkipTick { .. }
            | SkillCdTickEffect::PollKeyStates
            | SkillCdTickEffect::RecordElementalSkillPress
            | SkillCdTickEffect::IdentifyActiveIndex { .. }
            | SkillCdTickEffect::SetCooldown { .. }
            | SkillCdTickEffect::KeepCooldown { .. } => {}
        }
    }

    SkillCdTickExecutionReport {
        observation,
        effects: reduction.effects,
        executed_actions,
        state: reduction.state,
    }
}

fn apply_skill_cd_team_sync(
    plan: &SkillCdExecutionPlan,
    state: &mut SkillCdRuntimeState,
    sync: &SkillCdTeamSyncUpdate,
    effects: &mut Vec<SkillCdTickEffect>,
) {
    state.is_syncing_team = false;
    if !sync.initialized {
        state.team_avatar_names = empty_skill_cd_team_names();
        effects.push(SkillCdTickEffect::ClearTeam);
        return;
    }

    let mut next_names = empty_skill_cd_team_names();
    for (index, name) in sync.avatar_names.iter().take(4).enumerate() {
        next_names[index] = name.clone();
    }
    let team_changed = state.team_avatar_names != next_names;
    if team_changed && plan.team_sync_rule.reset_cds_when_team_changes {
        state.cooldowns = [0.0; 4];
        state.last_active_index = -1;
    }
    state.team_avatar_names = next_names;
    effects.push(SkillCdTickEffect::TeamSyncApplied { team_changed });
}

fn apply_skill_cd_action_trigger(
    plan: &SkillCdExecutionPlan,
    state: &mut SkillCdRuntimeState,
    observation: &SkillCdTickObservation,
    pressed_target: Option<usize>,
    effects: &mut Vec<SkillCdTickEffect>,
) {
    let Some(active_index) = observation.active_index else {
        effects.push(SkillCdTickEffect::KeepCooldown {
            slot: 0,
            reason: SkillCdKeepCooldownReason::MissingActiveIndex,
        });
        return;
    };
    if active_index == 0 || active_index > plan.cooldown_rule.slot_count {
        effects.push(SkillCdTickEffect::KeepCooldown {
            slot: 0,
            reason: SkillCdKeepCooldownReason::MissingActiveIndex,
        });
        return;
    }

    let slot = active_index - 1;
    if pressed_target == Some(slot) {
        effects.push(SkillCdTickEffect::KeepCooldown {
            slot,
            reason: SkillCdKeepCooldownReason::ActiveSlotMatchesPressedTarget,
        });
    } else if let Some(seconds) = observation
        .ocr_cooldown_seconds
        .filter(|seconds| *seconds > 0.0)
    {
        state.cooldowns[slot] = seconds;
        state.last_switch_from_slot = Some(slot);
        state.last_switch_elapsed_ms = Some(0);
        effects.push(SkillCdTickEffect::SetCooldown {
            slot,
            seconds,
            source: SkillCdCooldownUpdateSource::Ocr,
        });
    } else {
        let just_used_e = state
            .last_elemental_skill_press_elapsed_ms
            .map(|elapsed| elapsed < plan.input_rule.skill_use_recent_window_ms)
            .unwrap_or(false);
        if observation.visual_skill_ready.unwrap_or(false) {
            if just_used_e {
                apply_skill_cd_fallback(state, slot, observation, effects);
            } else if state.cooldowns[slot] > 0.0 {
                effects.push(SkillCdTickEffect::KeepCooldown {
                    slot,
                    reason: SkillCdKeepCooldownReason::VisualReadyWithExistingCooldown,
                });
            } else {
                state.cooldowns[slot] = 0.0;
                effects.push(SkillCdTickEffect::SetCooldown {
                    slot,
                    seconds: 0.0,
                    source: SkillCdCooldownUpdateSource::VisualReadyWithoutExistingCooldown,
                });
            }
        } else if just_used_e {
            apply_skill_cd_fallback(state, slot, observation, effects);
        } else {
            effects.push(SkillCdTickEffect::KeepCooldown {
                slot,
                reason: SkillCdKeepCooldownReason::OcrMissingAndNoFallback,
            });
        }
    }

    state.last_active_index = pressed_target.map(|target| target as i32).unwrap_or(-1) + 1;
}

fn apply_skill_cd_fallback(
    state: &mut SkillCdRuntimeState,
    slot: usize,
    observation: &SkillCdTickObservation,
    effects: &mut Vec<SkillCdTickEffect>,
) {
    if let Some(fallback) = &observation.fallback_cooldown {
        state.cooldowns[slot] = fallback.seconds;
        effects.push(SkillCdTickEffect::SetCooldown {
            slot,
            seconds: fallback.seconds,
            source: SkillCdCooldownUpdateSource::Fallback(fallback.source),
        });
    } else {
        effects.push(SkillCdTickEffect::KeepCooldown {
            slot,
            reason: SkillCdKeepCooldownReason::OcrMissingAndNoFallback,
        });
    }
}

fn push_skill_cd_overlay_effect(
    plan: &SkillCdExecutionPlan,
    state: &SkillCdRuntimeState,
    screen_scale_factor: f64,
    effects: &mut Vec<SkillCdTickEffect>,
) {
    if state.is_syncing_team && plan.overlay_rule.clear_while_syncing_team {
        effects.push(SkillCdTickEffect::ClearOverlay {
            reason: SkillCdClearOverlayReason::SyncingTeam,
        });
        return;
    }

    let valid_avatar_count = state
        .team_avatar_names
        .iter()
        .filter(|name| !name.is_empty())
        .count();
    if valid_avatar_count != plan.overlay_rule.requires_exact_avatar_count {
        effects.push(SkillCdTickEffect::ClearOverlay {
            reason: SkillCdClearOverlayReason::InvalidAvatarCount,
        });
        return;
    }

    let base_x = round_skill_cd_overlay_value(plan.overlay_rule.position_x) * screen_scale_factor;
    let base_y = round_skill_cd_overlay_value(plan.overlay_rule.position_y) * screen_scale_factor;
    let gap = round_skill_cd_overlay_value(plan.overlay_rule.gap) * screen_scale_factor;
    let entries = state
        .cooldowns
        .iter()
        .enumerate()
        .filter_map(|(slot, seconds)| {
            if plan.overlay_rule.hide_when_zero && *seconds <= 0.0 {
                return None;
            }
            Some(SkillCdOverlayEntry {
                slot,
                text: format!("{seconds:.1}"),
                x: base_x,
                y: base_y + gap * slot as f64,
            })
        })
        .collect::<Vec<_>>();

    if entries.is_empty() && plan.overlay_rule.clear_when_empty {
        effects.push(SkillCdTickEffect::ClearOverlay {
            reason: SkillCdClearOverlayReason::EmptyTextList,
        });
    } else {
        effects.push(SkillCdTickEffect::RenderOverlay { entries });
    }
}

pub fn parse_skill_cd_ocr_text(text: &str, rule: &SkillCdOcrRule) -> Option<f64> {
    if text.trim().is_empty() {
        return None;
    }

    let regex = Regex::new(&rule.regex).ok()?;
    let value = regex
        .find(text)
        .and_then(|matched| matched.as_str().parse::<f64>().ok())?
        - rule.compensation_seconds;

    if value > rule.accepted_min_exclusive && value < rule.accepted_max_exclusive {
        Some(value)
    } else {
        None
    }
}

pub fn resolve_skill_cd_fallback<'a>(
    role_name: &str,
    custom_rules: &[SkillCdCustomCdRule],
    default_cd_lookup: impl IntoIterator<Item = (&'a str, f64)>,
) -> SkillCdFallbackResolution {
    let default_cd_by_name = default_cd_lookup.into_iter().collect::<BTreeMap<_, _>>();

    if !role_name.is_empty() {
        if let Some(rule) = custom_rules
            .iter()
            .find(|rule| rule.role_name.as_str() == role_name)
        {
            if let Some(seconds) = rule.cd_value {
                return SkillCdFallbackResolution {
                    seconds,
                    source: SkillCdFallbackSource::CustomRule,
                };
            }
            if let Some(seconds) = default_cd_by_name.get(role_name) {
                return SkillCdFallbackResolution {
                    seconds: *seconds,
                    source: SkillCdFallbackSource::CustomRuleDefaultAvatarMap,
                };
            }
            return SkillCdFallbackResolution {
                seconds: 0.0,
                source: SkillCdFallbackSource::CustomRuleMissingDefaultAvatar,
            };
        }

        if let Some(seconds) = default_cd_by_name.get(role_name) {
            return SkillCdFallbackResolution {
                seconds: *seconds,
                source: SkillCdFallbackSource::DefaultAvatarMap,
            };
        }
    }

    SkillCdFallbackResolution {
        seconds: 0.0,
        source: SkillCdFallbackSource::MissingDefaultAvatar,
    }
}

fn e_cooldown_rect(size: Size) -> Rect {
    Rect {
        x: size.width as i32 - scaled(241, size),
        y: size.height as i32 - scaled(97, size),
        width: scaled(41, size),
        height: scaled(18, size),
    }
}

fn avatar_index_rects(size: Size) -> Vec<Rect> {
    [256, 352, 448, 544]
        .into_iter()
        .map(|y| Rect {
            x: size.width as i32 - scaled(61, size),
            y: scaled(y, size),
            width: scaled(28, size),
            height: scaled(24, size),
        })
        .collect()
}

fn avatar_side_icon_rects(size: Size) -> Vec<Rect> {
    [225, 315, 410, 500]
        .into_iter()
        .map(|y| Rect {
            x: size.width as i32 - scaled(155, size),
            y: scaled(y, size),
            width: scaled(76, size),
            height: scaled(76, size),
        })
        .collect()
}

fn custom_cd_rules(values: &[Value]) -> Vec<SkillCdCustomCdRule> {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .filter_map(|value| {
            let role_name = string_member(value, ["roleName", "RoleName", "role_name"])?;
            if role_name.trim().is_empty() || !seen.insert(role_name.clone()) {
                return None;
            }
            let cd_value_text =
                string_member(value, ["cdValueText", "CdValueText", "cd_value_text"])
                    .filter(|text| !text.trim().is_empty());
            let cd_value = cd_value_text
                .as_ref()
                .and_then(|text| text.trim().parse::<f64>().ok());
            Some(SkillCdCustomCdRule {
                role_name,
                cd_value_text,
                cd_value,
            })
        })
        .collect()
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

fn fallback_color(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / SKILL_CD_DEFAULT_CAPTURE_WIDTH as i64) as i32
}

fn seconds_to_ms(seconds: f64) -> u64 {
    if seconds <= 0.0 {
        0
    } else {
        (seconds * 1000.0).round() as u64
    }
}

fn increment_skill_cd_elapsed(value: &mut Option<u64>, delta_ms: u64) {
    if let Some(elapsed) = value.as_mut() {
        *elapsed = elapsed.saturating_add(delta_ms);
    }
}

fn empty_skill_cd_team_names() -> [String; 4] {
    std::array::from_fn(|_| String::new())
}

fn round_skill_cd_overlay_value(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    let capture = value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .unwrap_or(value);
    let width = u32_member(capture, ["width", "Width", "captureWidth", "CaptureWidth"])?;
    let height = u32_member(
        capture,
        ["height", "Height", "captureHeight", "CaptureHeight"],
    )?;
    Some(Size::new(width, height))
}

fn string_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<String> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
}

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}

fn u64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u64> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(Value::as_u64)
}
