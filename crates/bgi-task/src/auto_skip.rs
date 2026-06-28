use bgi_core::AutoSkipConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::Result;

pub const AUTO_SKIP_TASK_KEY: &str = "AutoSkip";
pub const AUTO_SKIP_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_SKIP_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_SKIP_STOP_AUTO: &str = "AutoSkip:stop_auto.png";
pub const AUTO_SKIP_DISABLED_UI: &str = "AutoSkip:disabled_ui.png";
pub const AUTO_SKIP_OPTION_ICON: &str = "AutoSkip:icon_option.png";
pub const AUTO_SKIP_DAILY_REWARD_ICON: &str = "AutoSkip:icon_daily_reward.png";
pub const AUTO_SKIP_EXPLORE_ICON: &str = "AutoSkip:icon_explore.png";
pub const AUTO_SKIP_EXCLAMATION_ICON: &str = "AutoSkip:icon_exclamation.png";
pub const AUTO_SKIP_PAGE_CLOSE: &str = "AutoSkip:page_close.png";
pub const AUTO_SKIP_COOK: &str = "AutoSkip:cook.png";
pub const AUTO_SKIP_PAGE_CLOSE_MAIN: &str = "AutoSkip:page_close_main.png";
pub const AUTO_SKIP_COLLECT: &str = "AutoSkip:collect.png";
pub const AUTO_SKIP_RE_DISPATCH: &str = "AutoSkip:re.png";
pub const AUTO_SKIP_PRIMOGEM: &str = "AutoSkip:primogem.png";
pub const AUTO_SKIP_SUBMIT_EXCLAMATION: &str = "AutoSkip:submit_icon_exclamation.png";
pub const AUTO_SKIP_SUBMIT_GOODS: &str = "AutoSkip:submit_goods.png";
pub const AUTO_SKIP_HANGOUT_SELECTED: &str = "AutoSkip:hangout_selected.png";
pub const AUTO_SKIP_HANGOUT_UNSELECTED: &str = "AutoSkip:hangout_unselected.png";
pub const AUTO_SKIP_HANGOUT_SKIP: &str = "AutoSkip:hangout_skip.png";
pub const AUTO_SKIP_CHAT_REVIEW: &str = "AutoSkip:chat_review.png";
pub const AUTO_SKIP_CONFIRM_BUTTON_1: &str = "AutoSkip:comfirm_btn1.png";
pub const AUTO_SKIP_CONFIRM_BUTTON_2: &str = "AutoSkip:comfirm_btn2.png";
pub const AUTO_SKIP_DEFAULT_PAUSE_OPTIONS_JSON: &str =
    "Assets/Config/Skip/default_pause_options.json";
pub const AUTO_SKIP_PAUSE_OPTIONS_JSON: &str = "Assets/Config/Skip/pause_options.json";
pub const AUTO_SKIP_SELECT_OPTIONS_JSON: &str = "Assets/Config/Skip/select_options.json";
pub const AUTO_SKIP_HANGOUT_JSON: &str = "GameTask/AutoSkip/Assets/hangout.json";
pub const AUTO_SKIP_COMMON_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const AUTO_SKIP_COMMON_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const AUTO_SKIP_AUTO_PICK_CHAT_PICK: &str = "AutoPick:F.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: AutoSkipConfigRule,
    pub assets: AutoSkipAssetsPlan,
    pub timing_rule: AutoSkipTimingRule,
    pub option_rule: AutoSkipOptionRule,
    pub audio_wait_rule: AutoSkipAudioWaitRule,
    pub black_screen_rule: AutoSkipBlackScreenRule,
    pub popup_rule: AutoSkipPopupRule,
    pub submit_goods_rule: AutoSkipSubmitGoodsRule,
    pub daily_reward_rule: AutoSkipDailyRewardRule,
    pub expedition_rule: AutoSkipExpeditionRule,
    pub hangout_rule: AutoSkipHangoutRule,
    pub steps: Vec<AutoSkipTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoSkipExecutionConfig {
    pub capture_size: Size,
    pub auto_skip_config: AutoSkipConfig,
}

impl Default for AutoSkipExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_SKIP_DEFAULT_CAPTURE_WIDTH,
                AUTO_SKIP_DEFAULT_CAPTURE_HEIGHT,
            ),
            auto_skip_config: AutoSkipConfig::default(),
        }
    }
}

impl AutoSkipExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let auto_skip_value = value
            .get("autoSkipConfig")
            .or_else(|| value.get("AutoSkipConfig"))
            .or_else(|| value.get("auto_skip_config"))
            .unwrap_or(value);
        config.auto_skip_config =
            serde_json::from_value(auto_skip_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipConfigRule {
    pub enabled: bool,
    pub quickly_skip_conversations_enabled: bool,
    pub before_click_confirm_delay_ms: u64,
    pub after_choose_option_sleep_delay_ms: u64,
    pub auto_wait_dialogue_option_voice_enabled: bool,
    pub dialogue_option_voice_max_wait_seconds: u64,
    pub auto_get_daily_rewards_enabled: bool,
    pub auto_re_explore_enabled: bool,
    pub auto_re_explore_character: String,
    pub click_chat_option: AutoSkipClickChatOption,
    pub click_chat_option_raw: String,
    pub custom_priority_options_enabled: bool,
    pub custom_priority_options: String,
    pub auto_hangout_event_enabled: bool,
    pub auto_hangout_end_choose: String,
    pub auto_hangout_choose_option_sleep_delay_ms: u64,
    pub auto_hangout_press_skip_enabled: bool,
    pub run_background_enabled: bool,
    pub bring_game_to_front_after_background_dialog_enabled: bool,
    pub submit_goods_enabled: bool,
    pub picture_in_picture_enabled: bool,
    pub picture_in_picture_source_type: String,
    pub close_popup_paged_enabled: bool,
    pub skip_built_in_click_options: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoSkipClickChatOption {
    First,
    Last,
    Random,
    None,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipAssetsPlan {
    pub stop_auto_button: AutoSkipTemplateLocator,
    pub disabled_ui_button: AutoSkipTemplateLocator,
    pub playing_text: AutoSkipOcrMatchLocator,
    pub option_icon: AutoSkipTemplateLocator,
    pub daily_reward_icon: AutoSkipTemplateLocator,
    pub explore_icon: AutoSkipTemplateLocator,
    pub exclamation_icon: AutoSkipTemplateLocator,
    pub page_close: AutoSkipTemplateLocator,
    pub cook: AutoSkipTemplateLocator,
    pub page_close_main: AutoSkipTemplateLocator,
    pub collect: AutoSkipTemplateLocator,
    pub re_dispatch: AutoSkipTemplateLocator,
    pub primogem: AutoSkipTemplateLocator,
    pub submit_exclamation: AutoSkipTemplateLocator,
    pub submit_goods: AutoSkipTemplateLocator,
    pub hangout_selected: AutoSkipTemplateLocator,
    pub hangout_unselected: AutoSkipTemplateLocator,
    pub hangout_skip: AutoSkipTemplateLocator,
    pub chat_pick_asset: String,
    pub chat_review_asset: String,
    pub confirm_button_assets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipOcrMatchLocator {
    pub name: String,
    pub roi: Rect,
    pub one_contain_match_text: Vec<String>,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipTimingRule {
    pub tick_min_interval_ms: u64,
    pub hangout_interval_ms: u64,
    pub playing_disappear_grace_ms: u64,
    pub submit_goods_window_ms: u64,
    pub black_click_interval_ms: u64,
    pub option_wait_recheck_ms: u64,
    pub bring_to_front_after_background_interval_ms: u64,
    pub close_item_popup_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipOptionRule {
    pub option_roi: Rect,
    pub text_region_x_offset_1080p: i32,
    pub text_region_width_1080p: i32,
    pub text_region_bottom_padding_1080p: i32,
    pub ignore_if_next_text_y_gap_greater_than: i32,
    pub skip_empty_text: bool,
    pub skip_alnum_text_shorter_than: usize,
    pub custom_priority_splitters: Vec<String>,
    pub default_pause_options_json: String,
    pub pause_options_json: String,
    pub select_options_json: String,
    pub decision_priority: Vec<AutoSkipOptionDecision>,
    pub orange_rule: AutoSkipOrangeOptionRule,
    pub use_key_rule: AutoSkipUseInteractionKeyRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipOptionDecision {
    CustomPriorityKeyword,
    ClickNoneExit,
    BuiltInSelectKeyword,
    BuiltInPauseKeyword,
    OrangeText,
    BuiltInDefaultPauseKeyword,
    ConfiguredDefaultChoice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipDialogueOptionCandidate {
    pub text: String,
    pub y: i32,
    pub is_orange: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipDialogueOptionDecisionReport {
    pub has_dialogue_option: bool,
    pub candidates: Vec<AutoSkipDialogueOptionCandidate>,
    pub action: AutoSkipDialogueOptionAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoSkipDialogueOptionAction {
    Click {
        candidate_index: usize,
        text: String,
        reason: AutoSkipDialogueOptionClickReason,
    },
    NoClick {
        reason: AutoSkipDialogueOptionNoClickReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipDialogueOptionClickReason {
    CustomPriorityKeyword,
    BuiltInSelectKeyword,
    OrangeDailyReward,
    OrangeExpedition,
    OrangeText,
    ConfiguredFirst,
    ConfiguredLast,
    ConfiguredRandom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipDialogueOptionNoClickReason {
    NoCandidates,
    ClickNoneConfigured,
    BuiltInPauseKeyword,
    OrangeReservedKeyword,
    BuiltInDefaultPauseKeyword,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipOrangeOptionRule {
    pub lower_bgr: AutoSkipBgrColor,
    pub upper_bgr: AutoSkipBgrColor,
    pub min_white_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipUseInteractionKeyRule {
    pub enabled_by_runtime_flag: bool,
    pub first_option_key_asset: String,
    pub random_scroll_key: String,
    pub previous_option_key: String,
    pub random_scroll_count_min: u8,
    pub random_scroll_count_max_exclusive: u8,
    pub scroll_interval_ms: u64,
    pub delay_before_confirm_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoSkipBgrColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipAudioWaitRule {
    pub sample_rate: u32,
    pub frame_sample_count: u32,
    pub speech_probability_threshold: f64,
    pub maybe_speech_probability_threshold: f64,
    pub speech_rise_duration_ms: u64,
    pub speech_start_grace_ms: u64,
    pub silence_duration_ms: u64,
    pub no_speech_quiet_duration_ms: u64,
    pub detector_retry_delay_ms: u64,
    pub max_wait_clamp_seconds: u64,
    pub fallback_delay_source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipBlackScreenRule {
    pub gray_roi: Rect,
    pub black_gray_value: u8,
    pub min_black_rate: f64,
    pub max_black_rate_exclusive: f64,
    pub click_background_when_background_operation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipPopupRule {
    pub close_popup_page_enabled: bool,
    pub bottom_triangle_crop: Rect,
    pub yellow_triangle_hsv: AutoSkipHsvRange,
    pub blue_triangle_hsv: AutoSkipHsvRange,
    pub triangle_area_min: f64,
    pub triangle_area_max: f64,
    pub triangle_vertices: usize,
    pub character_popup_area_ratio_min: f64,
    pub character_popup_area_ratio_max: f64,
    pub character_popup_aspect_ratio_min: f64,
    pub character_popup_aspect_ratio_max: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoSkipHsvRange {
    pub lower: AutoSkipHsvColor,
    pub upper: AutoSkipHsvColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoSkipHsvColor {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipSubmitGoodsRule {
    pub enabled: bool,
    pub active_after_playing_disappears_ms: u64,
    pub exclamation_locator: AutoSkipTemplateLocator,
    pub submit_goods_locator: AutoSkipTemplateLocator,
    pub selected_goods_color_bgr: AutoSkipBgrColor,
    pub selected_goods_color_tolerance: u8,
    pub selected_goods_min_area: u32,
    pub morphology_kernel_width: u8,
    pub morphology_kernel_height: u8,
    pub morphology_close_iterations: u8,
    pub after_select_goods_delay_ms: u64,
    pub after_black_confirm_delay_ms: u64,
    pub before_white_confirm_delay_ms: u64,
    pub black_confirm_asset: String,
    pub white_confirm_asset: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipDailyRewardRule {
    pub enabled: bool,
    pub keywords: Vec<String>,
    pub after_click_delay_ms: u64,
    pub black_confirm_asset: String,
    pub primogem_locator: AutoSkipTemplateLocator,
    pub primogem_wait_window_ms: u64,
    pub primogem_click: AutoSkipScreenPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoSkipScreenPoint {
    pub x_1080p: i32,
    pub y_1080p: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipExpeditionRule {
    pub enabled: bool,
    pub keywords: Vec<String>,
    pub open_panel_delay_ms: u64,
    pub collect_locator: AutoSkipTemplateLocator,
    pub re_dispatch_locator: AutoSkipTemplateLocator,
    pub collect_wait_ms: u64,
    pub re_dispatch_retry_interval_ms: u64,
    pub re_dispatch_retry_count: u8,
    pub after_re_dispatch_delay_ms: u64,
    pub final_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoSkipHangoutRule {
    pub enabled: bool,
    pub interval_ms: u64,
    pub selected_locator: AutoSkipTemplateLocator,
    pub unselected_locator: AutoSkipTemplateLocator,
    pub skip_locator: AutoSkipTemplateLocator,
    pub branch_config_json: String,
    pub configured_branch: String,
    pub choose_option_sleep_delay_ms: u64,
    pub press_skip_enabled: bool,
    pub decision_priority: Vec<AutoSkipHangoutDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipHangoutDecision {
    ConfiguredBranchKeyword,
    FirstUnselected,
    FirstSelected,
    SkipButtonWhenNoOption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipHangoutOptionCandidate {
    pub text: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipHangoutOptionDecisionReport {
    pub option_icons_detected: bool,
    pub has_text_candidates: bool,
    pub skip_button_detected: bool,
    pub candidates: Vec<AutoSkipHangoutOptionCandidate>,
    pub action: AutoSkipHangoutOptionAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoSkipHangoutOptionAction {
    ClickOption {
        candidate_index: usize,
        text: String,
        matched_keyword: Option<String>,
        reason: AutoSkipHangoutOptionClickReason,
    },
    ClickSkip {
        reason: AutoSkipHangoutSkipClickReason,
    },
    NoClick {
        reason: AutoSkipHangoutOptionNoClickReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipHangoutOptionClickReason {
    ConfiguredBranchKeyword,
    FirstUnselected,
    FirstSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipHangoutSkipClickReason {
    SkipButtonWhenNoOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipHangoutOptionNoClickReason {
    HangoutDisabled,
    OptionIconsDetectedButNoTextRegion,
    SkipDisabled,
    SkipButtonMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipTickStep {
    pub phase: AutoSkipTickPhase,
    pub condition: AutoSkipTickCondition,
    pub action: AutoSkipTickAction,
}

impl AutoSkipTickStep {
    fn new(
        phase: AutoSkipTickPhase,
        condition: AutoSkipTickCondition,
        action: AutoSkipTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipTickPhase {
    OperationMode,
    AudioWait,
    Throttle,
    DailyReward,
    PopupAfterPlaying,
    PlayingDialog,
    OptionChoose,
    Hangout,
    BlackScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipTickCondition {
    Always,
    WhenVoiceWaitDisabled,
    WhenChooseOptionWaitActive,
    WhenTickIntervalNotElapsed,
    WhenDailyRewardWaitWindowActive,
    WhenNotPlayingWithinGraceWindow,
    WhenSubmitGoodsWindowActive,
    WhenPlaying,
    WhenQuicklySkipConversationsEnabled,
    WhenOptionIconOrInteractionKeyDetected,
    WhenHangoutEnabledAndNoChatOption,
    WhenNotPlaying,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoSkipTickAction {
    RefreshBackgroundOperationMode,
    CancelOrReleaseAudioWaiter,
    UpdateChooseOptionWait,
    SkipTick,
    DetectAndClaimDailyPrimogem,
    ClosePopupPage,
    CloseItemPopupByBottomTriangle,
    CloseCharacterPopup,
    SubmitGoods,
    PressSpaceOrInteractionKey,
    ChooseDialogueOption,
    ChooseHangoutOption,
    ClickBlackScreen,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoSkipTriggerState {
    pub prev_tick_ms: Option<u64>,
    pub last_playing_ms: Option<u64>,
    pub choose_option_wait_until_ms: Option<u64>,
    pub choose_option_audio_recheck_until_ms: Option<u64>,
    pub previous_daily_reward_click_ms: Option<u64>,
    pub last_hangout_option_ms: Option<u64>,
    pub last_black_screen_click_ms: Option<u64>,
    pub last_close_item_popup_ms: Option<u64>,
    pub pending_bring_to_front_since_ms: Option<u64>,
    pub last_bring_to_front_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AutoSkipTickObservation {
    pub now_ms: u64,
    pub is_playing: bool,
    pub option_icon_detected: bool,
    pub interaction_key_detected: bool,
    pub daily_primogem_detected: bool,
    pub popup_page_close_detected: bool,
    pub bottom_triangle_detected: bool,
    pub character_popup_detected: bool,
    pub submit_goods_detected: bool,
    pub black_screen_detected: bool,
    pub dialogue_candidates: Vec<AutoSkipDialogueOptionCandidate>,
    pub hangout_option_icons_detected: bool,
    pub hangout_candidates: Vec<AutoSkipHangoutOptionCandidate>,
    pub hangout_skip_detected: bool,
    pub random_choice_index: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AutoSkipDialogueKeywordLists {
    pub select_keywords: Vec<String>,
    pub pause_keywords: Vec<String>,
    pub default_pause_keywords: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AutoSkipAudioWaitPreparation {
    pub recheck_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipTickExecutionReport {
    pub task_key: String,
    pub decision: AutoSkipTickDecisionReport,
    pub executed_actions: Vec<AutoSkipExecutedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipTickDecisionReport {
    pub processed: bool,
    pub skip_reason: Option<AutoSkipTickSkipReason>,
    pub background_operation_mode: Option<bool>,
    pub is_playing: Option<bool>,
    pub dialogue_decision: Option<AutoSkipDialogueOptionDecisionReport>,
    pub hangout_decision: Option<AutoSkipHangoutOptionDecisionReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipTickSkipReason {
    Disabled,
    ChooseOptionWaitActive,
    TickIntervalNotElapsed,
    DailyRewardWaitWindowHandled,
    AudioWaitPrepared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoSkipExecutedAction {
    RefreshBackgroundOperationMode {
        background_operation_mode: bool,
    },
    CancelOrReleaseAudioWaiter,
    PrepareAudioWait {
        reason: AutoSkipDialogueOptionClickReason,
        recheck_delay_ms: u64,
    },
    PressKey {
        key: String,
        reason: AutoSkipKeyPressReason,
    },
    ClickDialogueOption {
        candidate_index: usize,
        text: String,
        reason: AutoSkipDialogueOptionClickReason,
    },
    SelectDialogueOptionByKey {
        reason: AutoSkipDialogueOptionClickReason,
        key_presses: Vec<String>,
    },
    ClickHangoutOption {
        candidate_index: usize,
        text: String,
        reason: AutoSkipHangoutOptionClickReason,
    },
    ClickHangoutSkip {
        reason: AutoSkipHangoutSkipClickReason,
    },
    ClickDailyPrimogem {
        point: AutoSkipScreenPoint,
    },
    ClickDailyBlackConfirm {
        asset: String,
    },
    ClickPopupPageClose {
        asset: String,
    },
    BringGameToFront,
    ClickBottomTrianglePopup,
    CloseCharacterPopup,
    SubmitGoods,
    RunExpedition {
        text: String,
    },
    ClickBlackScreen {
        background_operation_mode: bool,
    },
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoSkipKeyPressReason {
    QuicklySkipConversation,
    InteractionKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoSkipKeySelectionReport {
    pub reason: AutoSkipDialogueOptionClickReason,
    pub key_presses: Vec<String>,
}

pub trait AutoSkipRuntime {
    fn refresh_auto_skip_background_operation_mode(
        &mut self,
        plan: &AutoSkipExecutionPlan,
    ) -> Result<bool>;

    fn observe_auto_skip_tick(
        &mut self,
        plan: &AutoSkipExecutionPlan,
    ) -> Result<AutoSkipTickObservation>;

    fn cancel_or_release_auto_skip_audio_waiter(
        &mut self,
        plan: &AutoSkipExecutionPlan,
    ) -> Result<()>;

    fn prepare_auto_skip_audio_wait(
        &mut self,
        plan: &AutoSkipExecutionPlan,
        reason: AutoSkipDialogueOptionClickReason,
    ) -> Result<AutoSkipAudioWaitPreparation>;

    fn auto_skip_dialogue_keywords(
        &mut self,
        plan: &AutoSkipExecutionPlan,
    ) -> Result<AutoSkipDialogueKeywordLists>;

    fn auto_skip_hangout_branch_keywords(
        &mut self,
        plan: &AutoSkipExecutionPlan,
    ) -> Result<Vec<String>>;

    fn press_auto_skip_key(&mut self, key: &str) -> Result<()>;

    fn click_auto_skip_dialogue_option(
        &mut self,
        candidate_index: usize,
        candidate: &AutoSkipDialogueOptionCandidate,
    ) -> Result<()>;

    fn select_auto_skip_dialogue_option_by_key(
        &mut self,
        plan: &AutoSkipExecutionPlan,
        random_choice_index: Option<usize>,
    ) -> Result<AutoSkipKeySelectionReport>;

    fn click_auto_skip_hangout_option(
        &mut self,
        candidate_index: usize,
        candidate: &AutoSkipHangoutOptionCandidate,
    ) -> Result<()>;

    fn click_auto_skip_hangout_skip(&mut self, plan: &AutoSkipExecutionPlan) -> Result<()>;

    fn click_auto_skip_daily_primogem(&mut self, point: AutoSkipScreenPoint) -> Result<()>;

    fn click_auto_skip_daily_black_confirm(&mut self, asset: &str) -> Result<bool>;

    fn click_auto_skip_popup_page_close(&mut self, locator: &AutoSkipTemplateLocator)
        -> Result<()>;

    fn bring_auto_skip_game_to_front(&mut self, plan: &AutoSkipExecutionPlan) -> Result<()>;

    fn click_auto_skip_bottom_triangle_popup(&mut self, plan: &AutoSkipExecutionPlan)
        -> Result<()>;

    fn close_auto_skip_character_popup(&mut self, plan: &AutoSkipExecutionPlan) -> Result<()>;

    fn execute_auto_skip_submit_goods(&mut self, plan: &AutoSkipExecutionPlan) -> Result<()>;

    fn execute_auto_skip_expedition(
        &mut self,
        plan: &AutoSkipExecutionPlan,
        text: &str,
    ) -> Result<()>;

    fn click_auto_skip_black_screen(&mut self, background_operation_mode: bool) -> Result<()>;

    fn delay_auto_skip(&mut self, duration_ms: u64) -> Result<()>;
}

pub fn execute_auto_skip_tick_plan<R>(
    plan: &AutoSkipExecutionPlan,
    state: &mut AutoSkipTriggerState,
    runtime: &mut R,
) -> Result<AutoSkipTickExecutionReport>
where
    R: AutoSkipRuntime,
{
    let mut decision = AutoSkipTickDecisionReport {
        processed: false,
        skip_reason: None,
        background_operation_mode: None,
        is_playing: None,
        dialogue_decision: None,
        hangout_decision: None,
    };
    let mut executed_actions = Vec::new();

    if !plan.config_rule.enabled {
        decision.skip_reason = Some(AutoSkipTickSkipReason::Disabled);
        return Ok(AutoSkipTickExecutionReport {
            task_key: plan.task_key.clone(),
            decision,
            executed_actions,
        });
    }

    let background_operation_mode = runtime.refresh_auto_skip_background_operation_mode(plan)?;
    executed_actions.push(AutoSkipExecutedAction::RefreshBackgroundOperationMode {
        background_operation_mode,
    });
    decision.background_operation_mode = Some(background_operation_mode);

    let observation = runtime.observe_auto_skip_tick(plan)?;
    decision.processed = true;
    decision.is_playing = Some(observation.is_playing);

    if !plan.config_rule.auto_wait_dialogue_option_voice_enabled {
        runtime.cancel_or_release_auto_skip_audio_waiter(plan)?;
        executed_actions.push(AutoSkipExecutedAction::CancelOrReleaseAudioWaiter);
        state.choose_option_audio_recheck_until_ms = None;
    }

    let mut audio_wait_recheck_active = state
        .choose_option_audio_recheck_until_ms
        .map(|until| observation.now_ms <= until)
        .unwrap_or(false);
    if !audio_wait_recheck_active {
        state.choose_option_audio_recheck_until_ms = None;
    }

    if let Some(wait_until_ms) = state.choose_option_wait_until_ms {
        if observation.now_ms < wait_until_ms {
            decision.skip_reason = Some(AutoSkipTickSkipReason::ChooseOptionWaitActive);
            return Ok(AutoSkipTickExecutionReport {
                task_key: plan.task_key.clone(),
                decision,
                executed_actions,
            });
        }
        state.choose_option_wait_until_ms = None;
        state.choose_option_audio_recheck_until_ms =
            Some(observation.now_ms + plan.timing_rule.option_wait_recheck_ms);
        audio_wait_recheck_active = true;
    }

    if elapsed_ms_since(state.prev_tick_ms, observation.now_ms)
        <= plan.timing_rule.tick_min_interval_ms
    {
        decision.skip_reason = Some(AutoSkipTickSkipReason::TickIntervalNotElapsed);
        return Ok(AutoSkipTickExecutionReport {
            task_key: plan.task_key.clone(),
            decision,
            executed_actions,
        });
    }
    state.prev_tick_ms = Some(observation.now_ms);

    if is_auto_skip_daily_reward_wait_window_active(plan, state, observation.now_ms)
        && observation.daily_primogem_detected
    {
        runtime.click_auto_skip_daily_primogem(plan.daily_reward_rule.primogem_click)?;
        state.previous_daily_reward_click_ms = None;
        executed_actions.push(AutoSkipExecutedAction::ClickDailyPrimogem {
            point: plan.daily_reward_rule.primogem_click,
        });
        decision.skip_reason = Some(AutoSkipTickSkipReason::DailyRewardWaitWindowHandled);
        return Ok(AutoSkipTickExecutionReport {
            task_key: plan.task_key.clone(),
            decision,
            executed_actions,
        });
    }

    if !observation.is_playing {
        execute_auto_skip_not_playing_branches(
            plan,
            state,
            &observation,
            background_operation_mode,
            runtime,
            &mut executed_actions,
        )?;
        return Ok(AutoSkipTickExecutionReport {
            task_key: plan.task_key.clone(),
            decision,
            executed_actions,
        });
    }

    state.last_playing_ms = Some(observation.now_ms);
    if background_operation_mode
        && plan
            .config_rule
            .bring_game_to_front_after_background_dialog_enabled
    {
        state
            .pending_bring_to_front_since_ms
            .get_or_insert(observation.now_ms);
    }

    if plan.config_rule.quickly_skip_conversations_enabled {
        let (key, reason) = if observation.interaction_key_detected {
            ("F", AutoSkipKeyPressReason::InteractionKey)
        } else {
            ("SPACE", AutoSkipKeyPressReason::QuicklySkipConversation)
        };
        if plan.config_rule.before_click_confirm_delay_ms > 0 {
            runtime.delay_auto_skip(plan.config_rule.before_click_confirm_delay_ms)?;
            executed_actions.push(AutoSkipExecutedAction::Delay {
                duration_ms: plan.config_rule.before_click_confirm_delay_ms,
            });
        }
        runtime.press_auto_skip_key(key)?;
        executed_actions.push(AutoSkipExecutedAction::PressKey {
            key: key.to_string(),
            reason,
        });
    }

    let chat_option_possible = observation.option_icon_detected
        || observation.interaction_key_detected
        || !observation.dialogue_candidates.is_empty();
    let mut has_chat_option = false;
    if chat_option_possible {
        let keyword_lists = runtime.auto_skip_dialogue_keywords(plan)?;
        let dialogue_decision = decide_auto_skip_dialogue_option(
            &observation.dialogue_candidates,
            &plan.config_rule,
            &plan.option_rule,
            &plan.daily_reward_rule,
            &plan.expedition_rule,
            keyword_lists.select_keywords.as_slice(),
            keyword_lists.pause_keywords.as_slice(),
            keyword_lists.default_pause_keywords.as_slice(),
            observation.random_choice_index,
        );
        has_chat_option = dialogue_decision.has_dialogue_option;

        if let AutoSkipDialogueOptionAction::Click {
            candidate_index,
            text,
            reason,
        } = &dialogue_decision.action
        {
            if plan.config_rule.auto_wait_dialogue_option_voice_enabled
                && !auto_skip_dialogue_click_bypasses_audio_wait(*reason)
                && !audio_wait_recheck_active
            {
                let wait = runtime.prepare_auto_skip_audio_wait(plan, *reason)?;
                state.choose_option_wait_until_ms =
                    Some(observation.now_ms + wait.recheck_delay_ms);
                state.choose_option_audio_recheck_until_ms = None;
                executed_actions.push(AutoSkipExecutedAction::PrepareAudioWait {
                    reason: *reason,
                    recheck_delay_ms: wait.recheck_delay_ms,
                });
                decision.dialogue_decision = Some(dialogue_decision);
                decision.skip_reason = Some(AutoSkipTickSkipReason::AudioWaitPrepared);
                return Ok(AutoSkipTickExecutionReport {
                    task_key: plan.task_key.clone(),
                    decision,
                    executed_actions,
                });
            }

            let candidate = &dialogue_decision.candidates[*candidate_index];
            if !auto_skip_dialogue_click_bypasses_audio_wait(*reason)
                && plan.config_rule.after_choose_option_sleep_delay_ms > 0
            {
                runtime.delay_auto_skip(plan.config_rule.after_choose_option_sleep_delay_ms)?;
                executed_actions.push(AutoSkipExecutedAction::Delay {
                    duration_ms: plan.config_rule.after_choose_option_sleep_delay_ms,
                });
            }
            runtime.click_auto_skip_dialogue_option(*candidate_index, candidate)?;
            state.choose_option_audio_recheck_until_ms = None;
            executed_actions.push(AutoSkipExecutedAction::ClickDialogueOption {
                candidate_index: *candidate_index,
                text: text.clone(),
                reason: *reason,
            });

            match reason {
                AutoSkipDialogueOptionClickReason::OrangeDailyReward => {
                    state.previous_daily_reward_click_ms = Some(observation.now_ms);
                    if plan.daily_reward_rule.after_click_delay_ms > 0 {
                        runtime.delay_auto_skip(plan.daily_reward_rule.after_click_delay_ms)?;
                        executed_actions.push(AutoSkipExecutedAction::Delay {
                            duration_ms: plan.daily_reward_rule.after_click_delay_ms,
                        });
                    }
                    if runtime.click_auto_skip_daily_black_confirm(
                        plan.daily_reward_rule.black_confirm_asset.as_str(),
                    )? {
                        executed_actions.push(AutoSkipExecutedAction::ClickDailyBlackConfirm {
                            asset: plan.daily_reward_rule.black_confirm_asset.clone(),
                        });
                    }
                }
                AutoSkipDialogueOptionClickReason::OrangeExpedition => {
                    if plan.expedition_rule.open_panel_delay_ms > 0 {
                        runtime.delay_auto_skip(plan.expedition_rule.open_panel_delay_ms)?;
                        executed_actions.push(AutoSkipExecutedAction::Delay {
                            duration_ms: plan.expedition_rule.open_panel_delay_ms,
                        });
                    }
                    runtime.execute_auto_skip_expedition(plan, text)?;
                    executed_actions
                        .push(AutoSkipExecutedAction::RunExpedition { text: text.clone() });
                }
                _ => {}
            }
        }

        decision.dialogue_decision = Some(dialogue_decision);

        if !has_chat_option
            && observation.interaction_key_detected
            && plan.config_rule.click_chat_option != AutoSkipClickChatOption::None
        {
            let reason = auto_skip_key_selection_reason(&plan.config_rule);
            if plan.config_rule.auto_wait_dialogue_option_voice_enabled
                && !audio_wait_recheck_active
            {
                let wait = runtime.prepare_auto_skip_audio_wait(plan, reason)?;
                state.choose_option_wait_until_ms =
                    Some(observation.now_ms + wait.recheck_delay_ms);
                state.choose_option_audio_recheck_until_ms = None;
                executed_actions.push(AutoSkipExecutedAction::PrepareAudioWait {
                    reason,
                    recheck_delay_ms: wait.recheck_delay_ms,
                });
                decision.skip_reason = Some(AutoSkipTickSkipReason::AudioWaitPrepared);
                return Ok(AutoSkipTickExecutionReport {
                    task_key: plan.task_key.clone(),
                    decision,
                    executed_actions,
                });
            }

            if plan.config_rule.after_choose_option_sleep_delay_ms > 0 {
                runtime.delay_auto_skip(plan.config_rule.after_choose_option_sleep_delay_ms)?;
                executed_actions.push(AutoSkipExecutedAction::Delay {
                    duration_ms: plan.config_rule.after_choose_option_sleep_delay_ms,
                });
            }
            let key_report = runtime
                .select_auto_skip_dialogue_option_by_key(plan, observation.random_choice_index)?;
            state.choose_option_audio_recheck_until_ms = None;
            has_chat_option = true;
            executed_actions.push(AutoSkipExecutedAction::SelectDialogueOptionByKey {
                reason: key_report.reason,
                key_presses: key_report.key_presses,
            });
        }
    }

    if plan.hangout_rule.enabled
        && !has_chat_option
        && elapsed_ms_since(state.last_hangout_option_ms, observation.now_ms)
            >= plan.timing_rule.hangout_interval_ms
    {
        state.last_hangout_option_ms = Some(observation.now_ms);
        let branch_keywords = runtime.auto_skip_hangout_branch_keywords(plan)?;
        let hangout_decision = decide_auto_skip_hangout_option(
            observation.hangout_option_icons_detected,
            &observation.hangout_candidates,
            observation.hangout_skip_detected,
            &plan.hangout_rule,
            branch_keywords.as_slice(),
        );
        match &hangout_decision.action {
            AutoSkipHangoutOptionAction::ClickOption {
                candidate_index,
                text,
                reason,
                ..
            } => {
                let candidate = &hangout_decision.candidates[*candidate_index];
                if plan.hangout_rule.choose_option_sleep_delay_ms > 0 {
                    runtime.delay_auto_skip(plan.hangout_rule.choose_option_sleep_delay_ms)?;
                    executed_actions.push(AutoSkipExecutedAction::Delay {
                        duration_ms: plan.hangout_rule.choose_option_sleep_delay_ms,
                    });
                }
                runtime.click_auto_skip_hangout_option(*candidate_index, candidate)?;
                executed_actions.push(AutoSkipExecutedAction::ClickHangoutOption {
                    candidate_index: *candidate_index,
                    text: text.clone(),
                    reason: *reason,
                });
            }
            AutoSkipHangoutOptionAction::ClickSkip { reason } => {
                runtime.click_auto_skip_hangout_skip(plan)?;
                executed_actions.push(AutoSkipExecutedAction::ClickHangoutSkip { reason: *reason });
            }
            AutoSkipHangoutOptionAction::NoClick { .. } => {}
        }
        decision.hangout_decision = Some(hangout_decision);
    }

    Ok(AutoSkipTickExecutionReport {
        task_key: plan.task_key.clone(),
        decision,
        executed_actions,
    })
}

pub fn plan_auto_skip(config: AutoSkipExecutionConfig) -> AutoSkipExecutionPlan {
    let capture_size = config.capture_size;
    let auto_skip_config = config.auto_skip_config;
    let assets = auto_skip_assets(capture_size);

    AutoSkipExecutionPlan {
        task_key: AUTO_SKIP_TASK_KEY.to_string(),
        display_name: "Auto Skip".to_string(),
        capture_size,
        config_rule: AutoSkipConfigRule {
            enabled: auto_skip_config.enabled,
            quickly_skip_conversations_enabled: auto_skip_config.quickly_skip_conversations_enabled,
            before_click_confirm_delay_ms: auto_skip_config.before_click_confirm_delay,
            after_choose_option_sleep_delay_ms: auto_skip_config.after_choose_option_sleep_delay,
            auto_wait_dialogue_option_voice_enabled: auto_skip_config
                .auto_wait_dialogue_option_voice_enabled,
            dialogue_option_voice_max_wait_seconds: auto_skip_config
                .dialogue_option_voice_max_wait_seconds,
            auto_get_daily_rewards_enabled: auto_skip_config.auto_get_daily_rewards_enabled,
            auto_re_explore_enabled: auto_skip_config.auto_re_explore_enabled,
            auto_re_explore_character: auto_skip_config.auto_re_explore_character,
            click_chat_option: click_chat_option_from_str(&auto_skip_config.click_chat_option),
            click_chat_option_raw: auto_skip_config.click_chat_option,
            custom_priority_options_enabled: auto_skip_config.custom_priority_options_enabled,
            custom_priority_options: auto_skip_config.custom_priority_options,
            auto_hangout_event_enabled: auto_skip_config.auto_hangout_event_enabled,
            auto_hangout_end_choose: auto_skip_config.auto_hangout_end_choose.clone(),
            auto_hangout_choose_option_sleep_delay_ms: auto_skip_config
                .auto_hangout_choose_option_sleep_delay,
            auto_hangout_press_skip_enabled: auto_skip_config.auto_hangout_press_skip_enabled,
            run_background_enabled: auto_skip_config.run_background_enabled,
            bring_game_to_front_after_background_dialog_enabled: auto_skip_config
                .bring_game_to_front_after_background_dialog_enabled,
            submit_goods_enabled: auto_skip_config.submit_goods_enabled,
            picture_in_picture_enabled: auto_skip_config.picture_in_picture_enabled,
            picture_in_picture_source_type: auto_skip_config.picture_in_picture_source_type,
            close_popup_paged_enabled: auto_skip_config.close_popup_paged_enabled,
            skip_built_in_click_options: auto_skip_config.skip_built_in_click_options,
        },
        timing_rule: AutoSkipTimingRule {
            tick_min_interval_ms: 200,
            hangout_interval_ms: 1_200,
            playing_disappear_grace_ms: 10_000,
            submit_goods_window_ms: 3_000,
            black_click_interval_ms: 1_200,
            option_wait_recheck_ms: 1_200,
            bring_to_front_after_background_interval_ms: 3_000,
            close_item_popup_interval_ms: 1_000,
        },
        option_rule: auto_skip_option_rule(capture_size),
        audio_wait_rule: AutoSkipAudioWaitRule {
            sample_rate: 16_000,
            frame_sample_count: 512,
            speech_probability_threshold: 0.60,
            maybe_speech_probability_threshold: 0.35,
            speech_rise_duration_ms: 160,
            speech_start_grace_ms: 5_000,
            silence_duration_ms: 2_000,
            no_speech_quiet_duration_ms: 1_200,
            detector_retry_delay_ms: 5_000,
            max_wait_clamp_seconds: 600,
            fallback_delay_source: "afterChooseOptionSleepDelay".to_string(),
        },
        black_screen_rule: AutoSkipBlackScreenRule {
            gray_roi: Rect {
                x: 0,
                y: capture_size.height as i32 / 3,
                width: capture_size.width as i32,
                height: capture_size.height as i32 / 3,
            },
            black_gray_value: 0,
            min_black_rate: 0.5,
            max_black_rate_exclusive: 0.98999,
            click_background_when_background_operation: true,
        },
        popup_rule: auto_skip_popup_rule(capture_size, auto_skip_config.close_popup_paged_enabled),
        submit_goods_rule: AutoSkipSubmitGoodsRule {
            enabled: auto_skip_config.submit_goods_enabled,
            active_after_playing_disappears_ms: 3_000,
            exclamation_locator: assets.submit_exclamation.clone(),
            submit_goods_locator: assets.submit_goods.clone(),
            selected_goods_color_bgr: AutoSkipBgrColor {
                b: 233,
                g: 229,
                r: 220,
            },
            selected_goods_color_tolerance: 100,
            selected_goods_min_area: 20,
            morphology_kernel_width: 5,
            morphology_kernel_height: 5,
            morphology_close_iterations: 2,
            after_select_goods_delay_ms: 800,
            after_black_confirm_delay_ms: 200,
            before_white_confirm_delay_ms: 500,
            black_confirm_asset: AUTO_SKIP_COMMON_BLACK_CONFIRM.to_string(),
            white_confirm_asset: AUTO_SKIP_COMMON_WHITE_CONFIRM.to_string(),
        },
        daily_reward_rule: AutoSkipDailyRewardRule {
            enabled: auto_skip_config.auto_get_daily_rewards_enabled,
            keywords: vec!["每日".to_string(), "委托".to_string()],
            after_click_delay_ms: 800,
            black_confirm_asset: AUTO_SKIP_COMMON_BLACK_CONFIRM.to_string(),
            primogem_locator: assets.primogem.clone(),
            primogem_wait_window_ms: 10_000,
            primogem_click: AutoSkipScreenPoint {
                x_1080p: 960,
                y_1080p: 900,
            },
        },
        expedition_rule: AutoSkipExpeditionRule {
            enabled: auto_skip_config.auto_re_explore_enabled,
            keywords: vec!["探索".to_string(), "派遣".to_string()],
            open_panel_delay_ms: 800,
            collect_locator: assets.collect.clone(),
            re_dispatch_locator: assets.re_dispatch.clone(),
            collect_wait_ms: 1_100,
            re_dispatch_retry_interval_ms: 1_000,
            re_dispatch_retry_count: 3,
            after_re_dispatch_delay_ms: 500,
            final_key: "ESC".to_string(),
        },
        hangout_rule: AutoSkipHangoutRule {
            enabled: auto_skip_config.auto_hangout_event_enabled,
            interval_ms: 1_200,
            selected_locator: assets.hangout_selected.clone(),
            unselected_locator: assets.hangout_unselected.clone(),
            skip_locator: assets.hangout_skip.clone(),
            branch_config_json: AUTO_SKIP_HANGOUT_JSON.to_string(),
            configured_branch: auto_skip_config.auto_hangout_end_choose,
            choose_option_sleep_delay_ms: auto_skip_config.auto_hangout_choose_option_sleep_delay,
            press_skip_enabled: auto_skip_config.auto_hangout_press_skip_enabled,
            decision_priority: vec![
                AutoSkipHangoutDecision::ConfiguredBranchKeyword,
                AutoSkipHangoutDecision::FirstUnselected,
                AutoSkipHangoutDecision::FirstSelected,
                AutoSkipHangoutDecision::SkipButtonWhenNoOption,
            ],
        },
        assets,
        steps: auto_skip_steps(),
        executor_ready: true,
        pending_native: vec![
            "live adapter for capture, BV talk/main/big-map UI detection, and draw overlay updates".to_string(),
            "live adapter for Paddle OCR dialogue options, hangout text, and expedition text".to_string(),
            "live adapter for PostMessage/background input dispatch, window activation, and keyboard/mouse input"
                .to_string(),
            "live adapter for Silero VAD ONNX inference over process loopback audio".to_string(),
            "live adapter for OpenCV black-screen, orange-text, bottom-triangle, and character-popup detection"
                .to_string(),
            "live adapter for OneKeyExpeditionTask, submit-goods item selection, and confirm-button execution"
                .to_string(),
            "Picture-in-picture capture source and foreground/background mode coordination"
                .to_string(),
        ],
    }
}

fn auto_skip_assets(capture_size: Size) -> AutoSkipAssetsPlan {
    let option_roi = option_roi(capture_size);
    AutoSkipAssetsPlan {
        stop_auto_button: template(
            "StopAutoButton",
            AUTO_SKIP_STOP_AUTO,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32 / 5,
                height: capture_size.height as i32 / 8,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        disabled_ui_button: template(
            "DisabledUiButton",
            AUTO_SKIP_DISABLED_UI,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32 / 3,
                height: capture_size.height as i32 / 8,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        playing_text: AutoSkipOcrMatchLocator {
            name: "PlayingText".to_string(),
            roi: Rect {
                x: scaled(100, capture_size),
                y: scaled(35, capture_size),
                width: scaled(85, capture_size),
                height: scaled(35, capture_size),
            },
            one_contain_match_text: vec![
                "播".to_string(),
                "番".to_string(),
                "放".to_string(),
                "中".to_string(),
            ],
            draw_on_window: true,
        },
        option_icon: template(
            "OptionIcon",
            AUTO_SKIP_OPTION_ICON,
            option_roi,
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        daily_reward_icon: template(
            "DailyRewardIcon",
            AUTO_SKIP_DAILY_REWARD_ICON,
            option_roi,
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        explore_icon: template(
            "ExploreIcon",
            AUTO_SKIP_EXPLORE_ICON,
            option_roi,
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        exclamation_icon: template(
            "IconExclamation",
            AUTO_SKIP_EXCLAMATION_ICON,
            option_roi,
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        page_close: template(
            "PageClose",
            AUTO_SKIP_PAGE_CLOSE,
            Rect {
                x: capture_size.width as i32 - capture_size.width as i32 / 8,
                y: 0,
                width: capture_size.width as i32 / 8,
                height: capture_size.height as i32 / 8,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        cook: template(
            "Cook",
            AUTO_SKIP_COOK,
            Rect {
                x: capture_size.width as i32 / 15,
                y: 0,
                width: capture_size.width as i32 / 14,
                height: capture_size.height as i32 / 14,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        page_close_main: template(
            "PageCloseMain",
            AUTO_SKIP_PAGE_CLOSE_MAIN,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32 / 25,
                height: capture_size.height as i32 / 14,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        collect: template(
            "Collect",
            AUTO_SKIP_COLLECT,
            Rect {
                x: 0,
                y: capture_size.height as i32 - capture_size.height as i32 / 3,
                width: capture_size.width as i32 / 4,
                height: capture_size.height as i32 / 3,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        re_dispatch: template(
            "Re",
            AUTO_SKIP_RE_DISPATCH,
            Rect {
                x: capture_size.width as i32 / 2,
                y: capture_size.height as i32 - capture_size.height as i32 / 4,
                width: capture_size.width as i32 / 4,
                height: capture_size.height as i32 / 4,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        primogem: template(
            "Primogem",
            AUTO_SKIP_PRIMOGEM,
            Rect {
                x: 0,
                y: capture_size.height as i32 / 3,
                width: capture_size.width as i32,
                height: capture_size.height as i32 / 3,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        submit_exclamation: template(
            "SubmitExclamationIconRo",
            AUTO_SKIP_SUBMIT_EXCLAMATION,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32,
                height: capture_size.height as i32 / 4,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        submit_goods: template(
            "SubmitGoods",
            AUTO_SKIP_SUBMIT_GOODS,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32 / 2,
                height: capture_size.height as i32 / 3,
            },
            0.9,
            TemplateMatchMode::CCorrNormed,
            true,
            true,
        ),
        hangout_selected: template(
            "HangoutSelected",
            AUTO_SKIP_HANGOUT_SELECTED,
            full_capture(capture_size),
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        hangout_unselected: template(
            "HangoutUnselected",
            AUTO_SKIP_HANGOUT_UNSELECTED,
            full_capture(capture_size),
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            true,
        ),
        hangout_skip: template(
            "HangoutSkip",
            AUTO_SKIP_HANGOUT_SKIP,
            Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32 / 5,
                height: capture_size.height as i32 / 8,
            },
            0.8,
            TemplateMatchMode::CCoeffNormed,
            false,
            false,
        ),
        chat_pick_asset: AUTO_SKIP_AUTO_PICK_CHAT_PICK.to_string(),
        chat_review_asset: AUTO_SKIP_CHAT_REVIEW.to_string(),
        confirm_button_assets: vec![
            AUTO_SKIP_CONFIRM_BUTTON_1.to_string(),
            AUTO_SKIP_CONFIRM_BUTTON_2.to_string(),
        ],
    }
}

fn auto_skip_option_rule(capture_size: Size) -> AutoSkipOptionRule {
    AutoSkipOptionRule {
        option_roi: option_roi(capture_size),
        text_region_x_offset_1080p: 8,
        text_region_width_1080p: 535,
        text_region_bottom_padding_1080p: 30,
        ignore_if_next_text_y_gap_greater_than: 150,
        skip_empty_text: true,
        skip_alnum_text_shorter_than: 5,
        custom_priority_splitters: vec![
            "\\r".to_string(),
            "\\n".to_string(),
            ";".to_string(),
            "；".to_string(),
        ],
        default_pause_options_json: AUTO_SKIP_DEFAULT_PAUSE_OPTIONS_JSON.to_string(),
        pause_options_json: AUTO_SKIP_PAUSE_OPTIONS_JSON.to_string(),
        select_options_json: AUTO_SKIP_SELECT_OPTIONS_JSON.to_string(),
        decision_priority: vec![
            AutoSkipOptionDecision::CustomPriorityKeyword,
            AutoSkipOptionDecision::ClickNoneExit,
            AutoSkipOptionDecision::BuiltInSelectKeyword,
            AutoSkipOptionDecision::BuiltInPauseKeyword,
            AutoSkipOptionDecision::OrangeText,
            AutoSkipOptionDecision::BuiltInDefaultPauseKeyword,
            AutoSkipOptionDecision::ConfiguredDefaultChoice,
        ],
        orange_rule: AutoSkipOrangeOptionRule {
            lower_bgr: AutoSkipBgrColor {
                b: 243,
                g: 195,
                r: 48,
            },
            upper_bgr: AutoSkipBgrColor {
                b: 255,
                g: 205,
                r: 55,
            },
            min_white_rate: 0.06,
        },
        use_key_rule: AutoSkipUseInteractionKeyRule {
            enabled_by_runtime_flag: true,
            first_option_key_asset: AUTO_SKIP_AUTO_PICK_CHAT_PICK.to_string(),
            random_scroll_key: "S".to_string(),
            previous_option_key: "W".to_string(),
            random_scroll_count_min: 0,
            random_scroll_count_max_exclusive: 5,
            scroll_interval_ms: 100,
            delay_before_confirm_ms: 50,
        },
    }
}

fn auto_skip_popup_rule(capture_size: Size, close_popup_page_enabled: bool) -> AutoSkipPopupRule {
    AutoSkipPopupRule {
        close_popup_page_enabled,
        bottom_triangle_crop: Rect {
            x: scaled(900, capture_size),
            y: scaled(960, capture_size),
            width: scaled(120, capture_size),
            height: scaled(120, capture_size),
        },
        yellow_triangle_hsv: AutoSkipHsvRange {
            lower: AutoSkipHsvColor {
                h: 0,
                s: 222,
                v: 173,
            },
            upper: AutoSkipHsvColor {
                h: 33,
                s: 255,
                v: 255,
            },
        },
        blue_triangle_hsv: AutoSkipHsvRange {
            lower: AutoSkipHsvColor {
                h: 87,
                s: 131,
                v: 142,
            },
            upper: AutoSkipHsvColor {
                h: 124,
                s: 255,
                v: 255,
            },
        },
        triangle_area_min: 10.0,
        triangle_area_max: 50.0,
        triangle_vertices: 3,
        character_popup_area_ratio_min: 0.24,
        character_popup_area_ratio_max: 0.3,
        character_popup_aspect_ratio_min: 5.6,
        character_popup_aspect_ratio_max: 7.2,
    }
}

fn auto_skip_steps() -> Vec<AutoSkipTickStep> {
    vec![
        AutoSkipTickStep::new(
            AutoSkipTickPhase::OperationMode,
            AutoSkipTickCondition::Always,
            AutoSkipTickAction::RefreshBackgroundOperationMode,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::AudioWait,
            AutoSkipTickCondition::WhenVoiceWaitDisabled,
            AutoSkipTickAction::CancelOrReleaseAudioWaiter,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::AudioWait,
            AutoSkipTickCondition::WhenChooseOptionWaitActive,
            AutoSkipTickAction::UpdateChooseOptionWait,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::Throttle,
            AutoSkipTickCondition::WhenTickIntervalNotElapsed,
            AutoSkipTickAction::SkipTick,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::DailyReward,
            AutoSkipTickCondition::WhenDailyRewardWaitWindowActive,
            AutoSkipTickAction::DetectAndClaimDailyPrimogem,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::PopupAfterPlaying,
            AutoSkipTickCondition::WhenNotPlayingWithinGraceWindow,
            AutoSkipTickAction::ClosePopupPage,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::PopupAfterPlaying,
            AutoSkipTickCondition::WhenNotPlayingWithinGraceWindow,
            AutoSkipTickAction::CloseItemPopupByBottomTriangle,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::PopupAfterPlaying,
            AutoSkipTickCondition::WhenNotPlayingWithinGraceWindow,
            AutoSkipTickAction::CloseCharacterPopup,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::PopupAfterPlaying,
            AutoSkipTickCondition::WhenSubmitGoodsWindowActive,
            AutoSkipTickAction::SubmitGoods,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::PlayingDialog,
            AutoSkipTickCondition::WhenPlaying,
            AutoSkipTickAction::PressSpaceOrInteractionKey,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::OptionChoose,
            AutoSkipTickCondition::WhenOptionIconOrInteractionKeyDetected,
            AutoSkipTickAction::ChooseDialogueOption,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::Hangout,
            AutoSkipTickCondition::WhenHangoutEnabledAndNoChatOption,
            AutoSkipTickAction::ChooseHangoutOption,
        ),
        AutoSkipTickStep::new(
            AutoSkipTickPhase::BlackScreen,
            AutoSkipTickCondition::WhenNotPlaying,
            AutoSkipTickAction::ClickBlackScreen,
        ),
    ]
}

fn template(
    name: &str,
    asset: &str,
    roi: Rect,
    threshold: f64,
    match_mode: TemplateMatchMode,
    use_3_channels: bool,
    draw_on_window: bool,
) -> AutoSkipTemplateLocator {
    AutoSkipTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode,
        use_3_channels,
        draw_on_window,
    }
}

fn click_chat_option_from_str(value: &str) -> AutoSkipClickChatOption {
    match value {
        "优先选择第一个选项" => AutoSkipClickChatOption::First,
        "优先选择最后一个选项" => AutoSkipClickChatOption::Last,
        "随机选择选项" => AutoSkipClickChatOption::Random,
        "不选择选项" => AutoSkipClickChatOption::None,
        other => AutoSkipClickChatOption::Other(other.to_string()),
    }
}

fn option_roi(size: Size) -> Rect {
    Rect {
        x: size.width as i32 / 2,
        y: size.height as i32 / 12,
        width: size.width as i32 - size.width as i32 / 2 - size.width as i32 / 6,
        height: size.height as i32 - size.height as i32 / 12 - 10,
    }
}

fn full_capture(size: Size) -> Rect {
    Rect {
        x: 0,
        y: 0,
        width: size.width as i32,
        height: size.height as i32,
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / AUTO_SKIP_DEFAULT_CAPTURE_WIDTH as i64) as i32
}

pub fn parse_auto_skip_custom_priority_options(text: &str) -> Vec<String> {
    text.split(['\r', '\n', ';', '；'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn filter_auto_skip_dialogue_option_candidates(
    candidates: &[AutoSkipDialogueOptionCandidate],
    rule: &AutoSkipOptionRule,
) -> Vec<AutoSkipDialogueOptionCandidate> {
    let mut sorted = candidates.to_vec();
    sorted.sort_by_key(|candidate| candidate.y);

    sorted
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            if rule.skip_empty_text && candidate.text.is_empty() {
                return None;
            }
            if candidate.text.chars().count() < rule.skip_alnum_text_shorter_than
                && candidate.text.chars().all(|c| c.is_ascii_alphanumeric())
            {
                return None;
            }
            if let Some(next) = sorted.get(index + 1) {
                if next.y - candidate.y > rule.ignore_if_next_text_y_gap_greater_than {
                    return None;
                }
            }

            Some(candidate.clone())
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn decide_auto_skip_dialogue_option(
    candidates: &[AutoSkipDialogueOptionCandidate],
    config_rule: &AutoSkipConfigRule,
    option_rule: &AutoSkipOptionRule,
    daily_reward_rule: &AutoSkipDailyRewardRule,
    expedition_rule: &AutoSkipExpeditionRule,
    select_keywords: &[String],
    pause_keywords: &[String],
    default_pause_keywords: &[String],
    random_choice_index: Option<usize>,
) -> AutoSkipDialogueOptionDecisionReport {
    let candidates = filter_auto_skip_dialogue_option_candidates(candidates, option_rule);
    if candidates.is_empty() {
        return auto_skip_dialogue_no_click(
            false,
            candidates,
            AutoSkipDialogueOptionNoClickReason::NoCandidates,
        );
    }

    if config_rule.custom_priority_options_enabled
        && !config_rule.custom_priority_options.is_empty()
    {
        let custom_options =
            parse_auto_skip_custom_priority_options(config_rule.custom_priority_options.as_str());
        for (index, candidate) in candidates.iter().enumerate() {
            if custom_options
                .iter()
                .any(|keyword| candidate.text.contains(keyword))
            {
                return auto_skip_dialogue_click(
                    candidates,
                    index,
                    AutoSkipDialogueOptionClickReason::CustomPriorityKeyword,
                );
            }
        }
    }

    if config_rule.click_chat_option == AutoSkipClickChatOption::None {
        return auto_skip_dialogue_no_click(
            false,
            candidates,
            AutoSkipDialogueOptionNoClickReason::ClickNoneConfigured,
        );
    }

    if !config_rule.skip_built_in_click_options {
        for (index, candidate) in candidates.iter().enumerate() {
            if auto_skip_text_contains_any(candidate.text.as_str(), select_keywords) {
                return auto_skip_dialogue_click(
                    candidates,
                    index,
                    AutoSkipDialogueOptionClickReason::BuiltInSelectKeyword,
                );
            }

            if auto_skip_text_contains_any(candidate.text.as_str(), pause_keywords) {
                return auto_skip_dialogue_no_click(
                    true,
                    candidates,
                    AutoSkipDialogueOptionNoClickReason::BuiltInPauseKeyword,
                );
            }
        }

        for (index, candidate) in candidates.iter().enumerate() {
            if !candidate.is_orange {
                continue;
            }

            let is_daily = auto_skip_text_contains_any(
                candidate.text.as_str(),
                daily_reward_rule.keywords.as_slice(),
            );
            let is_expedition = auto_skip_text_contains_any(
                candidate.text.as_str(),
                expedition_rule.keywords.as_slice(),
            );

            if daily_reward_rule.enabled && is_daily {
                return auto_skip_dialogue_click(
                    candidates,
                    index,
                    AutoSkipDialogueOptionClickReason::OrangeDailyReward,
                );
            }
            if expedition_rule.enabled && is_expedition {
                return auto_skip_dialogue_click(
                    candidates,
                    index,
                    AutoSkipDialogueOptionClickReason::OrangeExpedition,
                );
            }
            if !is_daily && !is_expedition {
                return auto_skip_dialogue_click(
                    candidates,
                    index,
                    AutoSkipDialogueOptionClickReason::OrangeText,
                );
            }

            return auto_skip_dialogue_no_click(
                true,
                candidates,
                AutoSkipDialogueOptionNoClickReason::OrangeReservedKeyword,
            );
        }

        for candidate in &candidates {
            if auto_skip_text_contains_any(candidate.text.as_str(), default_pause_keywords) {
                return auto_skip_dialogue_no_click(
                    true,
                    candidates,
                    AutoSkipDialogueOptionNoClickReason::BuiltInDefaultPauseKeyword,
                );
            }
        }
    }

    let (candidate_index, reason) = match config_rule.click_chat_option {
        AutoSkipClickChatOption::First => (0, AutoSkipDialogueOptionClickReason::ConfiguredFirst),
        AutoSkipClickChatOption::Random => (
            random_choice_index.unwrap_or(0) % candidates.len(),
            AutoSkipDialogueOptionClickReason::ConfiguredRandom,
        ),
        AutoSkipClickChatOption::Last
        | AutoSkipClickChatOption::Other(_)
        | AutoSkipClickChatOption::None => (
            candidates.len() - 1,
            AutoSkipDialogueOptionClickReason::ConfiguredLast,
        ),
    };

    auto_skip_dialogue_click(candidates, candidate_index, reason)
}

fn auto_skip_dialogue_click(
    candidates: Vec<AutoSkipDialogueOptionCandidate>,
    candidate_index: usize,
    reason: AutoSkipDialogueOptionClickReason,
) -> AutoSkipDialogueOptionDecisionReport {
    let text = candidates[candidate_index].text.clone();
    AutoSkipDialogueOptionDecisionReport {
        has_dialogue_option: true,
        candidates,
        action: AutoSkipDialogueOptionAction::Click {
            candidate_index,
            text,
            reason,
        },
    }
}

fn auto_skip_dialogue_no_click(
    has_dialogue_option: bool,
    candidates: Vec<AutoSkipDialogueOptionCandidate>,
    reason: AutoSkipDialogueOptionNoClickReason,
) -> AutoSkipDialogueOptionDecisionReport {
    AutoSkipDialogueOptionDecisionReport {
        has_dialogue_option,
        candidates,
        action: AutoSkipDialogueOptionAction::NoClick { reason },
    }
}

fn auto_skip_text_contains_any(text: &str, keywords: &[String]) -> bool {
    keywords.iter().any(|keyword| text.contains(keyword))
}

fn execute_auto_skip_not_playing_branches<R>(
    plan: &AutoSkipExecutionPlan,
    state: &mut AutoSkipTriggerState,
    observation: &AutoSkipTickObservation,
    background_operation_mode: bool,
    runtime: &mut R,
    executed_actions: &mut Vec<AutoSkipExecutedAction>,
) -> Result<()>
where
    R: AutoSkipRuntime,
{
    if state.pending_bring_to_front_since_ms.is_some()
        && plan
            .config_rule
            .bring_game_to_front_after_background_dialog_enabled
        && elapsed_ms_since(state.last_bring_to_front_ms, observation.now_ms)
            >= plan.timing_rule.bring_to_front_after_background_interval_ms
    {
        runtime.bring_auto_skip_game_to_front(plan)?;
        state.pending_bring_to_front_since_ms = None;
        state.last_bring_to_front_ms = Some(observation.now_ms);
        executed_actions.push(AutoSkipExecutedAction::BringGameToFront);
    }

    let within_playing_grace = state
        .last_playing_ms
        .map(|last| {
            observation.now_ms.saturating_sub(last) <= plan.timing_rule.playing_disappear_grace_ms
        })
        .unwrap_or(false);

    if within_playing_grace && plan.popup_rule.close_popup_page_enabled {
        if observation.popup_page_close_detected {
            runtime.click_auto_skip_popup_page_close(&plan.assets.page_close)?;
            executed_actions.push(AutoSkipExecutedAction::ClickPopupPageClose {
                asset: plan.assets.page_close.asset.clone(),
            });
        }

        if observation.bottom_triangle_detected
            && elapsed_ms_since(state.last_close_item_popup_ms, observation.now_ms)
                >= plan.timing_rule.close_item_popup_interval_ms
        {
            runtime.click_auto_skip_bottom_triangle_popup(plan)?;
            state.last_close_item_popup_ms = Some(observation.now_ms);
            executed_actions.push(AutoSkipExecutedAction::ClickBottomTrianglePopup);
        }

        if observation.character_popup_detected {
            runtime.close_auto_skip_character_popup(plan)?;
            executed_actions.push(AutoSkipExecutedAction::CloseCharacterPopup);
        }
    }

    let within_submit_goods_window = state
        .last_playing_ms
        .map(|last| {
            observation.now_ms.saturating_sub(last) <= plan.timing_rule.submit_goods_window_ms
        })
        .unwrap_or(false);
    if within_submit_goods_window {
        if plan.submit_goods_rule.enabled && observation.submit_goods_detected {
            runtime.execute_auto_skip_submit_goods(plan)?;
            executed_actions.push(AutoSkipExecutedAction::SubmitGoods);
        }
        return Ok(());
    }

    if observation.black_screen_detected
        && elapsed_ms_since(state.last_black_screen_click_ms, observation.now_ms)
            >= plan.timing_rule.black_click_interval_ms
    {
        runtime.click_auto_skip_black_screen(background_operation_mode)?;
        state.last_black_screen_click_ms = Some(observation.now_ms);
        executed_actions.push(AutoSkipExecutedAction::ClickBlackScreen {
            background_operation_mode,
        });
    }

    Ok(())
}

fn is_auto_skip_daily_reward_wait_window_active(
    plan: &AutoSkipExecutionPlan,
    state: &AutoSkipTriggerState,
    now_ms: u64,
) -> bool {
    if !plan.daily_reward_rule.enabled {
        return false;
    }

    state
        .previous_daily_reward_click_ms
        .map(|previous| {
            now_ms.saturating_sub(previous) <= plan.daily_reward_rule.primogem_wait_window_ms
        })
        .unwrap_or(false)
}

fn auto_skip_key_selection_reason(
    config_rule: &AutoSkipConfigRule,
) -> AutoSkipDialogueOptionClickReason {
    match config_rule.click_chat_option {
        AutoSkipClickChatOption::First => AutoSkipDialogueOptionClickReason::ConfiguredFirst,
        AutoSkipClickChatOption::Random => AutoSkipDialogueOptionClickReason::ConfiguredRandom,
        AutoSkipClickChatOption::Last
        | AutoSkipClickChatOption::Other(_)
        | AutoSkipClickChatOption::None => AutoSkipDialogueOptionClickReason::ConfiguredLast,
    }
}

fn auto_skip_dialogue_click_bypasses_audio_wait(reason: AutoSkipDialogueOptionClickReason) -> bool {
    matches!(
        reason,
        AutoSkipDialogueOptionClickReason::OrangeDailyReward
            | AutoSkipDialogueOptionClickReason::OrangeExpedition
    )
}

fn elapsed_ms_since(previous_ms: Option<u64>, now_ms: u64) -> u64 {
    previous_ms
        .map(|previous| now_ms.saturating_sub(previous))
        .unwrap_or(u64::MAX)
}

pub fn parse_auto_skip_hangout_branch_options(
    json: &str,
) -> std::result::Result<BTreeMap<String, Vec<String>>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn auto_skip_hangout_configured_branch_keywords<'a>(
    branches: &'a BTreeMap<String, Vec<String>>,
    configured_branch: &str,
) -> &'a [String] {
    branches
        .get(configured_branch)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn decide_auto_skip_hangout_option(
    option_icons_detected: bool,
    candidates: &[AutoSkipHangoutOptionCandidate],
    skip_button_detected: bool,
    hangout_rule: &AutoSkipHangoutRule,
    configured_branch_keywords: &[String],
) -> AutoSkipHangoutOptionDecisionReport {
    let candidates = candidates.to_vec();
    if !hangout_rule.enabled {
        return auto_skip_hangout_no_click(
            option_icons_detected,
            skip_button_detected,
            candidates,
            AutoSkipHangoutOptionNoClickReason::HangoutDisabled,
        );
    }

    if option_icons_detected {
        if candidates.is_empty() {
            return auto_skip_hangout_no_click(
                true,
                skip_button_detected,
                candidates,
                AutoSkipHangoutOptionNoClickReason::OptionIconsDetectedButNoTextRegion,
            );
        }

        if !hangout_rule.configured_branch.is_empty() {
            for (index, candidate) in candidates.iter().enumerate() {
                if let Some(keyword) = configured_branch_keywords
                    .iter()
                    .find(|keyword| candidate.text.contains(keyword.as_str()))
                {
                    return auto_skip_hangout_click_option(
                        option_icons_detected,
                        skip_button_detected,
                        candidates,
                        index,
                        Some(keyword.clone()),
                        AutoSkipHangoutOptionClickReason::ConfiguredBranchKeyword,
                    );
                }
            }
        }

        if let Some((index, _)) = candidates
            .iter()
            .enumerate()
            .find(|(_, candidate)| !candidate.is_selected)
        {
            return auto_skip_hangout_click_option(
                option_icons_detected,
                skip_button_detected,
                candidates,
                index,
                None,
                AutoSkipHangoutOptionClickReason::FirstUnselected,
            );
        }

        return auto_skip_hangout_click_option(
            option_icons_detected,
            skip_button_detected,
            candidates,
            0,
            None,
            AutoSkipHangoutOptionClickReason::FirstSelected,
        );
    }

    if !hangout_rule.press_skip_enabled {
        return auto_skip_hangout_no_click(
            false,
            skip_button_detected,
            candidates,
            AutoSkipHangoutOptionNoClickReason::SkipDisabled,
        );
    }

    if skip_button_detected {
        return AutoSkipHangoutOptionDecisionReport {
            option_icons_detected: false,
            has_text_candidates: false,
            skip_button_detected,
            candidates,
            action: AutoSkipHangoutOptionAction::ClickSkip {
                reason: AutoSkipHangoutSkipClickReason::SkipButtonWhenNoOption,
            },
        };
    }

    auto_skip_hangout_no_click(
        false,
        skip_button_detected,
        candidates,
        AutoSkipHangoutOptionNoClickReason::SkipButtonMissing,
    )
}

fn auto_skip_hangout_click_option(
    option_icons_detected: bool,
    skip_button_detected: bool,
    candidates: Vec<AutoSkipHangoutOptionCandidate>,
    candidate_index: usize,
    matched_keyword: Option<String>,
    reason: AutoSkipHangoutOptionClickReason,
) -> AutoSkipHangoutOptionDecisionReport {
    let text = candidates[candidate_index].text.clone();
    AutoSkipHangoutOptionDecisionReport {
        option_icons_detected,
        has_text_candidates: true,
        skip_button_detected,
        candidates,
        action: AutoSkipHangoutOptionAction::ClickOption {
            candidate_index,
            text,
            matched_keyword,
            reason,
        },
    }
}

fn auto_skip_hangout_no_click(
    option_icons_detected: bool,
    skip_button_detected: bool,
    candidates: Vec<AutoSkipHangoutOptionCandidate>,
    reason: AutoSkipHangoutOptionNoClickReason,
) -> AutoSkipHangoutOptionDecisionReport {
    let has_text_candidates = !candidates.is_empty();
    AutoSkipHangoutOptionDecisionReport {
        option_icons_detected,
        has_text_candidates,
        skip_button_detected,
        candidates,
        action: AutoSkipHangoutOptionAction::NoClick { reason },
    }
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

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}
