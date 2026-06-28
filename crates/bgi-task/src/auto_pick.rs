use bgi_core::AutoPickConfig;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

use crate::Result;

pub const AUTO_PICK_TASK_KEY: &str = "AutoPick";
pub const AUTO_PICK_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_PICK_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_PICK_PICK_KEY_ASSET: &str = "AutoPick:F.png";
pub const AUTO_PICK_CHAT_ICON_ASSET: &str = "AutoSkip:icon_option.png";
pub const AUTO_PICK_SETTINGS_ICON_ASSET: &str = "AutoPick:icon_settings.png";
pub const AUTO_PICK_L_KEY_ASSET: &str = "AutoPick:L.png";
pub const AUTO_PICK_DEFAULT_BLACK_LIST_JSON: &str =
    "Assets/Config/Pick/default_pick_black_lists.json";
pub const AUTO_PICK_USER_BLACK_LIST_TXT: &str = "User/pick_black_lists.txt";
pub const AUTO_PICK_USER_FUZZY_BLACK_LIST_TXT: &str = "User/pick_fuzzy_black_lists.txt";
pub const AUTO_PICK_USER_WHITE_LIST_TXT: &str = "User/pick_white_lists.txt";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPickExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: AutoPickConfigRule,
    pub external_config: AutoPickExternalConfig,
    pub template_rule: AutoPickTemplateRule,
    pub text_region_rule: AutoPickTextRegionRule,
    pub text_extraction_rule: AutoPickTextExtractionRule,
    pub in_progress_rule: AutoPickInProgressRule,
    pub scroll_rule: AutoPickScrollRule,
    pub ocr_cleanup_rule: AutoPickOcrCleanupRule,
    pub decision_rule: AutoPickDecisionRule,
    pub tick_steps: Vec<AutoPickTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoPickExecutionConfig {
    pub capture_size: Size,
    pub auto_pick_config: AutoPickConfig,
    pub external_config: AutoPickExternalConfig,
}

impl Default for AutoPickExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_PICK_DEFAULT_CAPTURE_WIDTH,
                AUTO_PICK_DEFAULT_CAPTURE_HEIGHT,
            ),
            auto_pick_config: AutoPickConfig::default(),
            external_config: AutoPickExternalConfig::default(),
        }
    }
}

impl AutoPickExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        if let Some(auto_pick_value) = value
            .get("autoPickConfig")
            .or_else(|| value.get("AutoPickConfig"))
            .or_else(|| value.get("auto_pick_config"))
        {
            config.auto_pick_config = auto_pick_config_from_value(auto_pick_value);
        } else {
            config.auto_pick_config = auto_pick_config_from_value(value);
        }

        if let Some(external_value) = value
            .get("externalConfig")
            .or_else(|| value.get("ExternalConfig"))
            .or_else(|| value.get("external_config"))
        {
            config.external_config = AutoPickExternalConfig::from_value(Some(external_value));
        } else {
            config.external_config = AutoPickExternalConfig::from_value(Some(value));
        }

        config
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoPickExternalConfig {
    pub text_list: Vec<String>,
    pub force_interaction: bool,
}

impl AutoPickExternalConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let Some(Value::Object(map)) = value else {
            return Self::default();
        };
        let text_list = map
            .get("textList")
            .or_else(|| map.get("TextList"))
            .or_else(|| map.get("text_list"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let force_interaction = bool_member(
            map,
            ["forceInteraction", "ForceInteraction", "force_interaction"],
        )
        .unwrap_or(false);

        Self {
            text_list,
            force_interaction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPickConfigRule {
    pub enabled: bool,
    pub pick_key: String,
    pub pick_key_asset: String,
    pub pick_key_region: Rect,
    pub custom_chat_pick_region: Rect,
    pub ocr_engine: AutoPickOcrEngine,
    pub fast_mode_enabled: bool,
    pub black_list_enabled: bool,
    pub white_list_enabled: bool,
    pub list_files: AutoPickListFiles,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoPickOcrEngine {
    Paddle,
    Yap,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickListFiles {
    pub default_black_list_json: String,
    pub user_black_list_txt: String,
    pub user_fuzzy_black_list_txt: String,
    pub user_white_list_txt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPickTemplateRule {
    pub pick_template: AutoPickTemplateLocator,
    pub l_key_template: AutoPickTemplateLocator,
    pub chat_icon_template: AutoPickRelativeTemplateLocator,
    pub settings_icon_template: AutoPickRelativeTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickTemplateLocator {
    pub name: String,
    pub asset: String,
    pub region_of_interest: Rect,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickRelativeTemplateLocator {
    pub name: String,
    pub asset: String,
    pub region: AutoPickRelativeRegion,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickRelativeRegion {
    pub anchor: AutoPickRegionAnchor,
    pub x_offset_1080p: i32,
    pub y_offset_1080p: i32,
    pub width_1080p: i32,
    pub height_source: AutoPickRelativeHeightSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoPickRegionAnchor {
    FoundPickKeyTopLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoPickRelativeHeightSource {
    FoundPickKeyHeight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickTextRegionRule {
    pub item_icon_left_offset_1080p: i32,
    pub item_text_left_offset_1080p: i32,
    pub item_text_right_offset_1080p: i32,
    pub text_width_1080p: i32,
    pub bounds_check_against_capture: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPickTextExtractionRule {
    pub binary_threshold_min: f64,
    pub binary_threshold_max: f64,
    pub morphology_kernel_width: i32,
    pub morphology_kernel_height: i32,
    pub erode_iterations: u32,
    pub dilate_iterations: u32,
    pub projection_max_gap: i32,
    pub valid_text_x_less_than: i32,
    pub valid_text_min_width: i32,
    pub valid_text_min_height: i32,
    pub paddle_detector_fallback: bool,
    pub text_only_right_padding: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoPickInProgressRule {
    pub sobel_dx: i32,
    pub sobel_dy: i32,
    pub sample_height_max: i32,
    pub skip_when_average_gradient_below: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickScrollRule {
    pub enabled_when_pick_key_missing: bool,
    pub probe_points: Vec<AutoPickColorProbe>,
    pub vertical_scroll_delta: i32,
    pub wait_after_scroll_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickColorProbe {
    pub x_1080p: i32,
    pub y_1080p: i32,
    pub rgb: AutoPickRgbColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoPickRgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickOcrCleanupRule {
    pub remove_whitespace: bool,
    pub replace_left_brackets_with_corner_quote: Vec<char>,
    pub replace_right_brackets_with_corner_quote: Vec<char>,
    pub trim_left_to_chinese_or_left_quote: bool,
    pub trim_right_to_chinese_right_quote_or_exclamation: bool,
    pub auto_pair_corner_quotes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickDecisionRule {
    pub force_interaction: bool,
    pub min_text_len_to_pick: usize,
    pub direct_pick_when_no_black_white_list_and_icon_not_excluded: bool,
    pub white_list_enabled: bool,
    pub white_list_can_pick_excluded_icon: bool,
    pub excluded_icon_blocks_after_white_list_check: bool,
    pub exact_black_list_enabled: bool,
    pub fuzzy_black_list_enabled: bool,
    pub do_not_pick_rules: Vec<AutoPickDoNotPickRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoPickDoNotPickRule {
    Contains(String),
    ContainsAll(Vec<String>),
    ContainsOnePrefixAndAny { prefix: String, any: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickTextDecision {
    Pick,
    Skip(AutoPickTextSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickTextSkipReason {
    EmptyText,
    StaticDoNotPick,
    TextTooShort,
    ExcludedIcon,
    ExactBlackList,
    FuzzyBlackList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickPreOcrDecision {
    ContinueToOcr,
    Pick,
    Skip(AutoPickPreOcrSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickPreOcrSkipReason {
    ExcludedIconWithoutWhiteList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoPickTickObservation {
    pub runner_pause_count: u32,
    pub found_pick_rect: Option<Rect>,
    pub scroll_icon_detected: bool,
    pub l_key_detected: bool,
    pub excluded_icon_detected: bool,
    pub average_text_gradient: Option<f64>,
    pub raw_ocr_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoPickTickDecisionReport {
    pub text_rect: Option<Rect>,
    pub cleaned_text: Option<String>,
    pub pre_ocr_decision: Option<AutoPickPreOcrDecision>,
    pub text_decision: Option<AutoPickTextDecision>,
    pub action: AutoPickTickDecisionAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoPickTickDecisionAction {
    Pick {
        key: String,
        reason: AutoPickTickPickReason,
        text: Option<String>,
    },
    Scroll {
        vertical_delta: i32,
        wait_after_scroll_ms: u64,
    },
    Skip {
        reason: AutoPickTickSkipReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickTickPickReason {
    ForceInteraction,
    DirectNoLists,
    TextAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPickTickSkipReason {
    Disabled,
    Paused,
    PickKeyMissing,
    LKeyDetected,
    ExcludedIconWithoutWhiteList,
    TextRegionOutOfRange,
    InProgress,
    MissingOcrText,
    TextRejected(AutoPickTextSkipReason),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoPickRuntimeLists {
    pub white_list: Vec<String>,
    pub exact_black_list: Vec<String>,
    pub fuzzy_black_list: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoPickTickExecutionReport {
    pub task_key: String,
    pub decision: AutoPickTickDecisionReport,
    pub executed_actions: Vec<AutoPickExecutedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoPickExecutedAction {
    KeyPress {
        key: String,
        reason: AutoPickTickPickReason,
        text: Option<String>,
    },
    MouseScroll {
        vertical_delta: i32,
    },
    Delay {
        duration_ms: u64,
    },
}

pub trait AutoPickRuntime {
    fn observe_auto_pick_tick(
        &mut self,
        plan: &AutoPickExecutionPlan,
    ) -> Result<AutoPickTickObservation>;

    fn auto_pick_lists(&mut self, plan: &AutoPickExecutionPlan) -> Result<AutoPickRuntimeLists>;

    fn press_auto_pick_key(&mut self, key: &str) -> Result<()>;

    fn scroll_auto_pick(&mut self, vertical_delta: i32) -> Result<()>;

    fn delay_auto_pick(&mut self, duration_ms: u64) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickTickStep {
    pub phase: AutoPickTickPhase,
    pub action: AutoPickTickAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoPickTickPhase {
    PauseGate,
    PickKeyDetection,
    ScrollFallback,
    ForceInteraction,
    LKeyGuard,
    ExcludedIconDetection,
    DirectPickWithoutLists,
    TextRegion,
    InProgressGuard,
    Ocr,
    TextCleanup,
    StaticDoNotPickFilter,
    WhiteList,
    ExcludedIconGuard,
    BlackList,
    Pick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoPickTickAction {
    WaitForRunnerPauseCount,
    TemplateMatch { asset: String },
    ScrollWhenScrollIconDetected,
    KeyPress { key: String },
    SkipWhenDetected { asset: String },
    DetectRelativeIcon { assets: Vec<String> },
    ComputeTextRegion,
    SkipWhenSobelShowsActivePickup,
    Ocr { engine: AutoPickOcrEngine },
    ProcessOcrText,
    SkipWhenTextMatchesStaticRule,
    PickWhenTextInWhiteList,
    SkipWhenIconExcluded,
    SkipWhenTextInBlackList,
    PickRecognizedText,
}

pub fn plan_auto_pick(config: AutoPickExecutionConfig) -> AutoPickExecutionPlan {
    let scale = scale_1080p(config.capture_size);
    let pick_key = normalize_pick_key(&config.auto_pick_config.pick_key);
    let pick_key_asset = format!("AutoPick:{pick_key}.png");
    let pick_key_region = rect_scaled(1090.0, 330.0, 60.0, 420.0, scale);
    let custom_chat_pick_region = Rect {
        x: scaled_i32(1200.0, scale),
        y: scaled_i32(350.0, scale),
        width: scaled_i32(50.0, scale),
        height: (config.capture_size.height as i32
            - scaled_i32(220.0, scale)
            - scaled_i32(350.0, scale))
        .max(0),
    };
    let l_key_region = Rect {
        x: config.capture_size.width as i32 - scaled_i32(110.0, scale),
        y: scaled_i32(550.0, scale),
        width: scaled_i32(70.0, scale),
        height: scaled_i32(100.0, scale),
    };
    let icon_width = (config.auto_pick_config.item_text_left_offset
        - config.auto_pick_config.item_icon_left_offset)
        .max(0);
    let text_width = (config.auto_pick_config.item_text_right_offset
        - config.auto_pick_config.item_text_left_offset)
        .max(0);
    let engine = ocr_engine(&config.auto_pick_config.ocr_engine);

    AutoPickExecutionPlan {
        task_key: AUTO_PICK_TASK_KEY.to_string(),
        display_name: "Auto Pick".to_string(),
        capture_size: config.capture_size,
        config_rule: AutoPickConfigRule {
            enabled: config.auto_pick_config.enabled,
            pick_key: pick_key.clone(),
            pick_key_asset: pick_key_asset.clone(),
            pick_key_region,
            custom_chat_pick_region,
            ocr_engine: engine.clone(),
            fast_mode_enabled: config.auto_pick_config.fast_mode_enabled,
            black_list_enabled: config.auto_pick_config.black_list_enabled,
            white_list_enabled: config.auto_pick_config.white_list_enabled,
            list_files: AutoPickListFiles {
                default_black_list_json: AUTO_PICK_DEFAULT_BLACK_LIST_JSON.to_string(),
                user_black_list_txt: AUTO_PICK_USER_BLACK_LIST_TXT.to_string(),
                user_fuzzy_black_list_txt: AUTO_PICK_USER_FUZZY_BLACK_LIST_TXT.to_string(),
                user_white_list_txt: AUTO_PICK_USER_WHITE_LIST_TXT.to_string(),
            },
        },
        external_config: config.external_config.clone(),
        template_rule: AutoPickTemplateRule {
            pick_template: AutoPickTemplateLocator {
                name: pick_key.clone(),
                asset: pick_key_asset,
                region_of_interest: pick_key_region,
                draw_on_window: false,
            },
            l_key_template: AutoPickTemplateLocator {
                name: "L".to_string(),
                asset: AUTO_PICK_L_KEY_ASSET.to_string(),
                region_of_interest: l_key_region,
                draw_on_window: false,
            },
            chat_icon_template: AutoPickRelativeTemplateLocator {
                name: "ChatIcon".to_string(),
                asset: AUTO_PICK_CHAT_ICON_ASSET.to_string(),
                region: AutoPickRelativeRegion {
                    anchor: AutoPickRegionAnchor::FoundPickKeyTopLeft,
                    x_offset_1080p: config.auto_pick_config.item_icon_left_offset,
                    y_offset_1080p: 0,
                    width_1080p: icon_width,
                    height_source: AutoPickRelativeHeightSource::FoundPickKeyHeight,
                },
                draw_on_window: false,
            },
            settings_icon_template: AutoPickRelativeTemplateLocator {
                name: "SettingsIcon".to_string(),
                asset: AUTO_PICK_SETTINGS_ICON_ASSET.to_string(),
                region: AutoPickRelativeRegion {
                    anchor: AutoPickRegionAnchor::FoundPickKeyTopLeft,
                    x_offset_1080p: config.auto_pick_config.item_icon_left_offset,
                    y_offset_1080p: 0,
                    width_1080p: icon_width,
                    height_source: AutoPickRelativeHeightSource::FoundPickKeyHeight,
                },
                draw_on_window: false,
            },
        },
        text_region_rule: AutoPickTextRegionRule {
            item_icon_left_offset_1080p: config.auto_pick_config.item_icon_left_offset,
            item_text_left_offset_1080p: config.auto_pick_config.item_text_left_offset,
            item_text_right_offset_1080p: config.auto_pick_config.item_text_right_offset,
            text_width_1080p: text_width,
            bounds_check_against_capture: true,
        },
        text_extraction_rule: AutoPickTextExtractionRule {
            binary_threshold_min: 160.0,
            binary_threshold_max: 255.0,
            morphology_kernel_width: 3,
            morphology_kernel_height: 3,
            erode_iterations: 1,
            dilate_iterations: 2,
            projection_max_gap: 30,
            valid_text_x_less_than: 20,
            valid_text_min_width: 5,
            valid_text_min_height: 5,
            paddle_detector_fallback: true,
            text_only_right_padding: 5,
        },
        in_progress_rule: AutoPickInProgressRule {
            sobel_dx: 1,
            sobel_dy: 0,
            sample_height_max: 3,
            skip_when_average_gradient_below: -3.0,
        },
        scroll_rule: AutoPickScrollRule {
            enabled_when_pick_key_missing: true,
            probe_points: vec![
                AutoPickColorProbe {
                    x_1080p: 1062,
                    y_1080p: 537,
                    rgb: AutoPickRgbColor {
                        r: 255,
                        g: 233,
                        b: 44,
                    },
                },
                AutoPickColorProbe {
                    x_1080p: 1062,
                    y_1080p: 524,
                    rgb: AutoPickRgbColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                },
                AutoPickColorProbe {
                    x_1080p: 1062,
                    y_1080p: 554,
                    rgb: AutoPickRgbColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                },
            ],
            vertical_scroll_delta: 2,
            wait_after_scroll_ms: 50,
        },
        ocr_cleanup_rule: AutoPickOcrCleanupRule {
            remove_whitespace: true,
            replace_left_brackets_with_corner_quote: vec!['【', '['],
            replace_right_brackets_with_corner_quote: vec!['】', ']'],
            trim_left_to_chinese_or_left_quote: true,
            trim_right_to_chinese_right_quote_or_exclamation: true,
            auto_pair_corner_quotes: true,
        },
        decision_rule: AutoPickDecisionRule {
            force_interaction: config.external_config.force_interaction,
            min_text_len_to_pick: 2,
            direct_pick_when_no_black_white_list_and_icon_not_excluded: true,
            white_list_enabled: config.auto_pick_config.white_list_enabled,
            white_list_can_pick_excluded_icon: true,
            excluded_icon_blocks_after_white_list_check: true,
            exact_black_list_enabled: config.auto_pick_config.black_list_enabled,
            fuzzy_black_list_enabled: config.auto_pick_config.black_list_enabled,
            do_not_pick_rules: vec![
                AutoPickDoNotPickRule::Contains("长时间".to_string()),
                AutoPickDoNotPickRule::ContainsOnePrefixAndAny {
                    prefix: "我在".to_string(),
                    any: vec![
                        "声望".to_string(),
                        "回声".to_string(),
                        "悬木人".to_string(),
                        "流泉".to_string(),
                    ],
                },
                AutoPickDoNotPickRule::Contains("聚所".to_string()),
                AutoPickDoNotPickRule::ContainsAll(vec!["霜月".to_string(), "坊".to_string()]),
                AutoPickDoNotPickRule::Contains("叮铃".to_string()),
                AutoPickDoNotPickRule::Contains("眶螂".to_string()),
                AutoPickDoNotPickRule::ContainsAll(vec!["蛋卷".to_string(), "坊".to_string()]),
                AutoPickDoNotPickRule::Contains("西风成垒".to_string()),
                AutoPickDoNotPickRule::Contains("望崖营壁".to_string()),
                AutoPickDoNotPickRule::Contains("魔女的花园".to_string()),
                AutoPickDoNotPickRule::Contains("月谕圣牌".to_string()),
            ],
        },
        tick_steps: auto_pick_tick_steps(engine, pick_key),
        executor_ready: true,
        pending_native: vec![
            "desktop live adapter now covers capture, pause-count gate, F/L/chat/settings template matching, scroll icon color probing, list loading, pick-key input, scroll input, and delay dispatch".to_string(),
            "Sobel/morphology text extraction and Paddle/Yap OCR remain pending".to_string(),
        ],
    }
}

pub fn execute_auto_pick_tick_plan<R>(
    plan: &AutoPickExecutionPlan,
    runtime: &mut R,
) -> Result<AutoPickTickExecutionReport>
where
    R: AutoPickRuntime,
{
    let observation = runtime.observe_auto_pick_tick(plan)?;
    let lists = if auto_pick_tick_requires_runtime_lists(plan, &observation) {
        runtime.auto_pick_lists(plan)?
    } else {
        AutoPickRuntimeLists::default()
    };
    let decision = decide_auto_pick_tick(
        plan,
        observation,
        lists.white_list.iter().map(String::as_str),
        lists.exact_black_list.iter().map(String::as_str),
        lists.fuzzy_black_list.iter().map(String::as_str),
    );
    let mut executed_actions = Vec::new();

    match &decision.action {
        AutoPickTickDecisionAction::Pick { key, reason, text } => {
            runtime.press_auto_pick_key(key)?;
            executed_actions.push(AutoPickExecutedAction::KeyPress {
                key: key.clone(),
                reason: *reason,
                text: text.clone(),
            });
        }
        AutoPickTickDecisionAction::Scroll {
            vertical_delta,
            wait_after_scroll_ms,
        } => {
            runtime.scroll_auto_pick(*vertical_delta)?;
            executed_actions.push(AutoPickExecutedAction::MouseScroll {
                vertical_delta: *vertical_delta,
            });
            if *wait_after_scroll_ms > 0 {
                runtime.delay_auto_pick(*wait_after_scroll_ms)?;
                executed_actions.push(AutoPickExecutedAction::Delay {
                    duration_ms: *wait_after_scroll_ms,
                });
            }
        }
        AutoPickTickDecisionAction::Skip { .. } => {}
    }

    Ok(AutoPickTickExecutionReport {
        task_key: plan.task_key.clone(),
        decision,
        executed_actions,
    })
}

pub fn auto_pick_tick_requires_runtime_lists(
    plan: &AutoPickExecutionPlan,
    observation: &AutoPickTickObservation,
) -> bool {
    if !plan.config_rule.enabled || observation.runner_pause_count > 0 {
        return false;
    }

    let Some(found_pick_rect) = observation.found_pick_rect else {
        return false;
    };

    if plan.decision_rule.force_interaction || observation.l_key_detected {
        return false;
    }

    if decide_auto_pick_pre_ocr(observation.excluded_icon_detected, &plan.decision_rule)
        != AutoPickPreOcrDecision::ContinueToOcr
    {
        return false;
    }

    let Some(text_rect) =
        compute_auto_pick_text_rect(found_pick_rect, plan.capture_size, &plan.text_region_rule)
    else {
        return false;
    };

    if observation
        .average_text_gradient
        .map(|gradient| should_auto_pick_skip_in_progress(gradient, &plan.in_progress_rule))
        .unwrap_or(false)
    {
        return false;
    }

    let Some(raw_text) = observation.raw_ocr_text.as_deref() else {
        return false;
    };
    let _ = text_rect;
    let cleaned_text = process_auto_pick_ocr_text(raw_text, &plan.ocr_cleanup_rule);
    if cleaned_text.is_empty()
        || should_auto_pick_skip_text(&cleaned_text, &plan.decision_rule.do_not_pick_rules)
        || cleaned_text.chars().count() < plan.decision_rule.min_text_len_to_pick
    {
        return false;
    }

    plan.decision_rule.white_list_enabled
        || plan.decision_rule.exact_black_list_enabled
        || plan.decision_rule.fuzzy_black_list_enabled
}

pub fn decide_auto_pick_tick<'a>(
    plan: &AutoPickExecutionPlan,
    observation: AutoPickTickObservation,
    white_list: impl IntoIterator<Item = &'a str>,
    exact_black_list: impl IntoIterator<Item = &'a str>,
    fuzzy_black_list: impl IntoIterator<Item = &'a str>,
) -> AutoPickTickDecisionReport {
    if !plan.config_rule.enabled {
        return auto_pick_tick_skip(AutoPickTickSkipReason::Disabled);
    }
    if observation.runner_pause_count > 0 {
        return auto_pick_tick_skip(AutoPickTickSkipReason::Paused);
    }

    let Some(found_pick_rect) = observation.found_pick_rect else {
        if observation.scroll_icon_detected && plan.scroll_rule.enabled_when_pick_key_missing {
            return AutoPickTickDecisionReport {
                text_rect: None,
                cleaned_text: None,
                pre_ocr_decision: None,
                text_decision: None,
                action: AutoPickTickDecisionAction::Scroll {
                    vertical_delta: plan.scroll_rule.vertical_scroll_delta,
                    wait_after_scroll_ms: plan.scroll_rule.wait_after_scroll_ms,
                },
            };
        }
        return auto_pick_tick_skip(AutoPickTickSkipReason::PickKeyMissing);
    };

    if plan.decision_rule.force_interaction {
        return auto_pick_tick_pick(
            AutoPickTickPickReason::ForceInteraction,
            &plan.config_rule.pick_key,
            None,
            None,
            None,
            None,
        );
    }

    if observation.l_key_detected {
        return auto_pick_tick_skip(AutoPickTickSkipReason::LKeyDetected);
    }

    let pre_ocr_decision =
        decide_auto_pick_pre_ocr(observation.excluded_icon_detected, &plan.decision_rule);
    match pre_ocr_decision {
        AutoPickPreOcrDecision::Pick => {
            return auto_pick_tick_pick(
                AutoPickTickPickReason::DirectNoLists,
                &plan.config_rule.pick_key,
                None,
                Some(pre_ocr_decision),
                None,
                None,
            );
        }
        AutoPickPreOcrDecision::Skip(AutoPickPreOcrSkipReason::ExcludedIconWithoutWhiteList) => {
            return AutoPickTickDecisionReport {
                text_rect: None,
                cleaned_text: None,
                pre_ocr_decision: Some(pre_ocr_decision),
                text_decision: None,
                action: AutoPickTickDecisionAction::Skip {
                    reason: AutoPickTickSkipReason::ExcludedIconWithoutWhiteList,
                },
            };
        }
        AutoPickPreOcrDecision::ContinueToOcr => {}
    }

    let Some(text_rect) =
        compute_auto_pick_text_rect(found_pick_rect, plan.capture_size, &plan.text_region_rule)
    else {
        return AutoPickTickDecisionReport {
            text_rect: None,
            cleaned_text: None,
            pre_ocr_decision: Some(pre_ocr_decision),
            text_decision: None,
            action: AutoPickTickDecisionAction::Skip {
                reason: AutoPickTickSkipReason::TextRegionOutOfRange,
            },
        };
    };

    if observation
        .average_text_gradient
        .map(|gradient| should_auto_pick_skip_in_progress(gradient, &plan.in_progress_rule))
        .unwrap_or(false)
    {
        return AutoPickTickDecisionReport {
            text_rect: Some(text_rect),
            cleaned_text: None,
            pre_ocr_decision: Some(pre_ocr_decision),
            text_decision: None,
            action: AutoPickTickDecisionAction::Skip {
                reason: AutoPickTickSkipReason::InProgress,
            },
        };
    }

    let Some(raw_text) = observation.raw_ocr_text.as_deref() else {
        return AutoPickTickDecisionReport {
            text_rect: Some(text_rect),
            cleaned_text: None,
            pre_ocr_decision: Some(pre_ocr_decision),
            text_decision: None,
            action: AutoPickTickDecisionAction::Skip {
                reason: AutoPickTickSkipReason::MissingOcrText,
            },
        };
    };
    let cleaned_text = process_auto_pick_ocr_text(raw_text, &plan.ocr_cleanup_rule);
    let text_decision = decide_auto_pick_text(
        &cleaned_text,
        observation.excluded_icon_detected,
        &plan.decision_rule,
        white_list,
        exact_black_list,
        fuzzy_black_list,
    );
    match text_decision {
        AutoPickTextDecision::Pick => auto_pick_tick_pick(
            AutoPickTickPickReason::TextAccepted,
            &plan.config_rule.pick_key,
            Some(cleaned_text),
            Some(pre_ocr_decision),
            Some(text_decision),
            Some(text_rect),
        ),
        AutoPickTextDecision::Skip(reason) => AutoPickTickDecisionReport {
            text_rect: Some(text_rect),
            cleaned_text: Some(cleaned_text),
            pre_ocr_decision: Some(pre_ocr_decision),
            text_decision: Some(text_decision),
            action: AutoPickTickDecisionAction::Skip {
                reason: AutoPickTickSkipReason::TextRejected(reason),
            },
        },
    }
}

fn auto_pick_tick_pick(
    reason: AutoPickTickPickReason,
    key: &str,
    text: Option<String>,
    pre_ocr_decision: Option<AutoPickPreOcrDecision>,
    text_decision: Option<AutoPickTextDecision>,
    text_rect: Option<Rect>,
) -> AutoPickTickDecisionReport {
    AutoPickTickDecisionReport {
        text_rect,
        cleaned_text: text.clone(),
        pre_ocr_decision,
        text_decision,
        action: AutoPickTickDecisionAction::Pick {
            key: key.to_string(),
            reason,
            text,
        },
    }
}

fn auto_pick_tick_skip(reason: AutoPickTickSkipReason) -> AutoPickTickDecisionReport {
    AutoPickTickDecisionReport {
        text_rect: None,
        cleaned_text: None,
        pre_ocr_decision: None,
        text_decision: None,
        action: AutoPickTickDecisionAction::Skip { reason },
    }
}

pub fn should_auto_pick_skip_text(text: &str, rules: &[AutoPickDoNotPickRule]) -> bool {
    rules.iter().any(|rule| match rule {
        AutoPickDoNotPickRule::Contains(value) => text.contains(value),
        AutoPickDoNotPickRule::ContainsAll(values) => {
            values.iter().all(|value| text.contains(value))
        }
        AutoPickDoNotPickRule::ContainsOnePrefixAndAny { prefix, any } => {
            text.contains(prefix) && any.iter().any(|value| text.contains(value))
        }
    })
}

pub fn process_auto_pick_ocr_text(text: &str, rule: &AutoPickOcrCleanupRule) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut chars = Vec::with_capacity(text.chars().count());
    for c in text.chars() {
        if rule.remove_whitespace && c.is_whitespace() {
            continue;
        }

        if rule.replace_left_brackets_with_corner_quote.contains(&c) {
            chars.push('「');
        } else if rule.replace_right_brackets_with_corner_quote.contains(&c) {
            chars.push('」');
        } else {
            chars.push(c);
        }
    }

    let Some(start) = chars
        .iter()
        .position(|c| !rule.trim_left_to_chinese_or_left_quote || is_chinese_or_left_quote(*c))
    else {
        return String::new();
    };
    let Some(end) = chars.iter().rposition(|c| {
        !rule.trim_right_to_chinese_right_quote_or_exclamation || {
            is_chinese_right_quote_or_exclamation(*c)
        }
    }) else {
        return String::new();
    };
    if start > end {
        return String::new();
    }

    let mut cleaned: String = chars[start..=end].iter().collect();
    if rule.auto_pair_corner_quotes {
        let has_left_quote = cleaned.contains('「');
        let has_right_quote = cleaned.contains('」');
        if has_left_quote && !has_right_quote {
            cleaned.push('」');
        } else if has_right_quote && !has_left_quote {
            cleaned.insert(0, '「');
        }
    }

    cleaned
}

pub fn decide_auto_pick_text<'a>(
    text: &str,
    excluded_icon: bool,
    rule: &AutoPickDecisionRule,
    white_list: impl IntoIterator<Item = &'a str>,
    exact_black_list: impl IntoIterator<Item = &'a str>,
    fuzzy_black_list: impl IntoIterator<Item = &'a str>,
) -> AutoPickTextDecision {
    if text.is_empty() {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::EmptyText);
    }

    if should_auto_pick_skip_text(text, &rule.do_not_pick_rules) {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::StaticDoNotPick);
    }

    if text.chars().count() < rule.min_text_len_to_pick {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::TextTooShort);
    }

    if rule.white_list_enabled
        && white_list.into_iter().any(|item| item == text)
        && (!excluded_icon || rule.white_list_can_pick_excluded_icon)
    {
        return AutoPickTextDecision::Pick;
    }

    if excluded_icon && rule.excluded_icon_blocks_after_white_list_check {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::ExcludedIcon);
    }

    if rule.exact_black_list_enabled && exact_black_list.into_iter().any(|item| item == text) {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::ExactBlackList);
    }

    if rule.fuzzy_black_list_enabled
        && fuzzy_black_list
            .into_iter()
            .any(|item| !item.is_empty() && text.contains(item))
    {
        return AutoPickTextDecision::Skip(AutoPickTextSkipReason::FuzzyBlackList);
    }

    AutoPickTextDecision::Pick
}

pub fn decide_auto_pick_pre_ocr(
    excluded_icon: bool,
    rule: &AutoPickDecisionRule,
) -> AutoPickPreOcrDecision {
    if rule.force_interaction {
        return AutoPickPreOcrDecision::Pick;
    }

    if excluded_icon && !rule.white_list_enabled {
        return AutoPickPreOcrDecision::Skip(
            AutoPickPreOcrSkipReason::ExcludedIconWithoutWhiteList,
        );
    }

    if rule.direct_pick_when_no_black_white_list_and_icon_not_excluded
        && !excluded_icon
        && !rule.white_list_enabled
        && !rule.exact_black_list_enabled
        && !rule.fuzzy_black_list_enabled
    {
        return AutoPickPreOcrDecision::Pick;
    }

    AutoPickPreOcrDecision::ContinueToOcr
}

pub fn compute_auto_pick_text_rect(
    found_pick_rect: Rect,
    capture_size: Size,
    rule: &AutoPickTextRegionRule,
) -> Option<Rect> {
    let scale = scale_1080p(capture_size);
    let rect = Rect {
        x: found_pick_rect.x + scaled_i32(rule.item_text_left_offset_1080p as f64, scale),
        y: found_pick_rect.y,
        width: scaled_i32(rule.text_width_1080p as f64, scale),
        height: found_pick_rect.height,
    };

    if !rule.bounds_check_against_capture {
        return Some(rect);
    }

    if rect.x < 0
        || rect.y < 0
        || rect.width <= 0
        || rect.height <= 0
        || rect.x + rect.width > capture_size.width as i32
        || rect.y + rect.height > capture_size.height as i32
    {
        return None;
    }

    Some(rect)
}

pub fn should_auto_pick_skip_in_progress(
    average_gradient: f64,
    rule: &AutoPickInProgressRule,
) -> bool {
    average_gradient < rule.skip_when_average_gradient_below
}

pub fn is_valid_auto_pick_text_bounds(bounds: Rect, rule: &AutoPickTextExtractionRule) -> bool {
    bounds.x < rule.valid_text_x_less_than
        && bounds.width > rule.valid_text_min_width
        && bounds.height > rule.valid_text_min_height
}

pub fn auto_pick_text_only_crop_width(
    bounds: Rect,
    text_mat_width: i32,
    rule: &AutoPickTextExtractionRule,
) -> i32 {
    let text_mat_width = text_mat_width.max(0);
    let right_with_padding = (bounds.x + bounds.width + rule.text_only_right_padding).max(0);
    right_with_padding.min(text_mat_width)
}

pub fn parse_auto_pick_text_list(text: &str) -> Vec<String> {
    text.split(['\r', '\n'])
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn parse_auto_pick_text_set(text: &str) -> BTreeSet<String> {
    parse_auto_pick_text_list(text).into_iter().collect()
}

fn is_chinese_or_left_quote(c: char) -> bool {
    c == '「' || is_common_chinese(c)
}

fn is_chinese_right_quote_or_exclamation(c: char) -> bool {
    c == '」' || c == '！' || is_common_chinese(c)
}

fn is_common_chinese(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

fn auto_pick_tick_steps(engine: AutoPickOcrEngine, pick_key: String) -> Vec<AutoPickTickStep> {
    vec![
        AutoPickTickStep {
            phase: AutoPickTickPhase::PauseGate,
            action: AutoPickTickAction::WaitForRunnerPauseCount,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::PickKeyDetection,
            action: AutoPickTickAction::TemplateMatch {
                asset: format!("AutoPick:{pick_key}.png"),
            },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::ScrollFallback,
            action: AutoPickTickAction::ScrollWhenScrollIconDetected,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::ForceInteraction,
            action: AutoPickTickAction::KeyPress {
                key: pick_key.clone(),
            },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::LKeyGuard,
            action: AutoPickTickAction::SkipWhenDetected {
                asset: AUTO_PICK_L_KEY_ASSET.to_string(),
            },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::ExcludedIconDetection,
            action: AutoPickTickAction::DetectRelativeIcon {
                assets: vec![
                    AUTO_PICK_CHAT_ICON_ASSET.to_string(),
                    AUTO_PICK_SETTINGS_ICON_ASSET.to_string(),
                ],
            },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::DirectPickWithoutLists,
            action: AutoPickTickAction::KeyPress {
                key: pick_key.clone(),
            },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::TextRegion,
            action: AutoPickTickAction::ComputeTextRegion,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::InProgressGuard,
            action: AutoPickTickAction::SkipWhenSobelShowsActivePickup,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::Ocr,
            action: AutoPickTickAction::Ocr { engine },
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::TextCleanup,
            action: AutoPickTickAction::ProcessOcrText,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::StaticDoNotPickFilter,
            action: AutoPickTickAction::SkipWhenTextMatchesStaticRule,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::WhiteList,
            action: AutoPickTickAction::PickWhenTextInWhiteList,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::ExcludedIconGuard,
            action: AutoPickTickAction::SkipWhenIconExcluded,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::BlackList,
            action: AutoPickTickAction::SkipWhenTextInBlackList,
        },
        AutoPickTickStep {
            phase: AutoPickTickPhase::Pick,
            action: AutoPickTickAction::PickRecognizedText,
        },
    ]
}

fn auto_pick_config_from_value(value: &Value) -> AutoPickConfig {
    serde_json::from_value(value.clone()).unwrap_or_default()
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

fn normalize_pick_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        "F".to_string()
    } else {
        key.to_string()
    }
}

fn ocr_engine(value: &str) -> AutoPickOcrEngine {
    match value {
        "Paddle" => AutoPickOcrEngine::Paddle,
        "Yap" => AutoPickOcrEngine::Yap,
        other => AutoPickOcrEngine::Other(other.to_string()),
    }
}

fn scale_1080p(size: Size) -> f64 {
    size.height as f64 / AUTO_PICK_DEFAULT_CAPTURE_HEIGHT as f64
}

fn rect_scaled(x: f64, y: f64, width: f64, height: f64, scale: f64) -> Rect {
    Rect {
        x: scaled_i32(x, scale),
        y: scaled_i32(y, scale),
        width: scaled_i32(width, scale),
        height: scaled_i32(height, scale),
    }
}

fn scaled_i32(value: f64, scale: f64) -> i32 {
    (value * scale).trunc() as i32
}

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}

fn bool_member<const N: usize>(
    map: &serde_json::Map<String, Value>,
    keys: [&str; N],
) -> Option<bool> {
    keys.into_iter()
        .filter_map(|key| map.get(key))
        .find_map(Value::as_bool)
}
