use bgi_core::config::AutoArtifactSalvageConfig;
use bgi_core::GenshinAction;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common_job::{
    GridBgrColor, GridIconClassifierRule, GridItemDetectionRule, GridScreenName, GridScrollRule,
    GridTemplate, InventoryTabAssetPair, GRID_ICON_INPUT_NAME, GRID_ICON_MODEL_NAME,
    GRID_ICON_MODEL_PATH, GRID_ICON_PROTOTYPE_CSV_PATH,
};
use crate::{Result, TaskError, TaskPortState};

pub const AUTO_ARTIFACT_SALVAGE_TASK_KEY: &str = "AutoArtifactSalvage";
pub const AUTO_ARTIFACT_SALVAGE_DISPLAY_NAME: &str = "圣遗物分解独立任务";
pub const AUTO_ARTIFACT_SALVAGE_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_ARTIFACT_SALVAGE_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_ARTIFACT_SALVAGE_BTN_ASSET: &str = "Common/Element:btn_artifact_salvage.png";
pub const AUTO_ARTIFACT_SALVAGE_CONFIRM_ASSET: &str =
    "Common/Element:btn_artifact_salvage_confirm.png";
pub const AUTO_ARTIFACT_SALVAGE_WHITE_CONFIRM_ASSET: &str = "Common/Element:btn_white_confirm.png";
pub const AUTO_ARTIFACT_SALVAGE_BLACK_CONFIRM_ASSET: &str = "Common/Element:btn_black_confirm.png";
pub const AUTO_ARTIFACT_SALVAGE_DEFAULT_JS: &str = "var hasATK = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'ATK');\nvar hasDEF = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'DEF');\nvar hasHP = Array.from(ArtifactStat.MinorAffixes).some(affix => affix.Type == 'HP');\nOutput = (hasATK && hasDEF) || (hasHP && hasDEF);";
pub const AUTO_ARTIFACT_SALVAGE_LEGACY_REGEX: &str =
    r"(?=[\S\s]*攻击力\+[\d]*\n)(?=[\S\s]*防御力\+[\d]*\n)";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoArtifactSalvageExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub config_rule: AutoArtifactSalvageConfigRule,
    pub open_inventory_rule: AutoArtifactSalvageOpenInventoryRule,
    pub quick_salvage_rule: QuickArtifactSalvageRule,
    pub five_star_rule: Option<FiveStarArtifactFilterRule>,
    pub steps: Vec<AutoArtifactSalvageStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoArtifactSalvageExecutionConfig {
    pub capture_size: Size,
    pub param: AutoArtifactSalvageParam,
}

impl Default for AutoArtifactSalvageExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_ARTIFACT_SALVAGE_DEFAULT_CAPTURE_WIDTH,
                AUTO_ARTIFACT_SALVAGE_DEFAULT_CAPTURE_HEIGHT,
            ),
            param: AutoArtifactSalvageParam::from_core_config(AutoArtifactSalvageConfig::default()),
        }
    }
}

impl AutoArtifactSalvageExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let core_config_value = value
            .get("autoArtifactSalvageConfig")
            .or_else(|| value.get("AutoArtifactSalvageConfig"))
            .or_else(|| value.get("auto_artifact_salvage_config"))
            .unwrap_or(value);
        let core_config: AutoArtifactSalvageConfig =
            serde_json::from_value(core_config_value.clone()).unwrap_or_default();
        config.param = AutoArtifactSalvageParam::from_core_config(core_config);

        overlay_param(&mut config.param, core_config_value);
        if core_config_value as *const Value != value as *const Value {
            overlay_param(&mut config.param, value);
        }
        if let Some(param_value) = value
            .get("param")
            .or_else(|| value.get("Param"))
            .or_else(|| value.get("taskParam"))
            .or_else(|| value.get("TaskParam"))
        {
            overlay_param(&mut config.param, param_value);
        }
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoArtifactSalvageParam {
    pub star: u8,
    pub java_script: Option<String>,
    pub artifact_set_filter: Option<String>,
    pub max_num_to_check: Option<u64>,
    pub recognition_failure_policy: Option<AutoArtifactRecognitionFailurePolicy>,
    pub regular_expression: String,
}

impl AutoArtifactSalvageParam {
    pub fn quick_only(star: u8) -> Self {
        Self {
            star,
            java_script: None,
            artifact_set_filter: None,
            max_num_to_check: None,
            recognition_failure_policy: None,
            regular_expression: AUTO_ARTIFACT_SALVAGE_LEGACY_REGEX.to_string(),
        }
    }

    fn from_core_config(config: AutoArtifactSalvageConfig) -> Self {
        Self {
            star: parse_star_value(&Value::String(config.max_artifact_star)).unwrap_or(4),
            java_script: Some(config.java_script),
            artifact_set_filter: Some(config.artifact_set_filter),
            max_num_to_check: Some(config.max_num_to_check),
            recognition_failure_policy: parse_recognition_failure_policy(
                &config.recognition_failure_policy,
            )
            .or(Some(AutoArtifactRecognitionFailurePolicy::Skip)),
            regular_expression: config.regular_expression,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoArtifactRecognitionFailurePolicy {
    Skip,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoArtifactSalvageConfigRule {
    pub star: u8,
    pub selected_quick_salvage_stars: Vec<u8>,
    pub unselected_after_quick_select_stars: Vec<u8>,
    pub java_script_present: bool,
    pub java_script_is_blank: bool,
    pub artifact_set_filter_present: bool,
    pub artifact_set_filter_contains_predicted_name: bool,
    pub max_num_to_check: Option<u64>,
    pub zero_max_count_still_checks_first_selected_item: bool,
    pub recognition_failure_policy: Option<AutoArtifactRecognitionFailurePolicy>,
    pub recognition_failure_skip_does_not_consume_check_count: bool,
    pub regular_expression_legacy_unused: String,
    pub valid_star_range: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoArtifactSalvageOpenInventoryRule {
    pub return_main_ui_before_open: bool,
    pub grid_screen_name: GridScreenName,
    pub open_inventory_action: GenshinAction,
    pub open_wait_ms: u64,
    pub retry_attempts: u8,
    pub retry_when_main_ui_detected: bool,
    pub expired_item_prompt_confirm_asset: String,
    pub expired_item_prompt_crop_bottom_ratio: f64,
    pub expired_item_prompt_wait_ms: u64,
    pub inventory_tab_assets: InventoryTabAssetPair,
    pub after_tab_ready_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickArtifactSalvageRule {
    pub opens_salvage_button: ArtifactSalvageTemplateLocator,
    pub quick_select_ocr_rule: ArtifactSalvageOcrButtonRule,
    pub star_option_ocr_rule: ArtifactSalvageStarOptionRule,
    pub quick_select_confirm_asset: String,
    pub quick_select_confirm_reused_as_filter_button: bool,
    pub quick_select_confirm_wait_ms: u64,
    pub salvage_confirm_asset: String,
    pub after_salvage_confirm_wait_ms: u64,
    pub final_confirm_asset: String,
    pub final_confirm_kind: ArtifactSalvageConfirmKind,
    pub after_final_confirm_wait_ms: u64,
    pub post_quick_salvage_click_when_js_present: bool,
    pub no_quick_items_is_not_fatal: bool,
    pub destructive_native_action: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSalvageTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: ArtifactSalvageRelativeRoi,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSalvageRelativeRoi {
    pub cut: String,
    pub width_ratio: Option<f64>,
    pub height_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSalvageOcrButtonRule {
    pub text_key: String,
    pub default_regex: String,
    pub roi: ArtifactSalvageRelativeRoi,
    pub click_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSalvageStarOptionRule {
    pub localized_regexes_by_star: Vec<ArtifactSalvageStarText>,
    pub roi: ArtifactSalvageRelativeRoi,
    pub unselect_wait_ms: u64,
    pub legacy_inverse_selection_since_5_5: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactSalvageStarText {
    pub star: u8,
    pub text_key: String,
    pub default_regex: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactSalvageConfirmKind {
    White,
    Black,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FiveStarArtifactFilterRule {
    pub artifact_set_filter_rule: Option<ArtifactSetFilterSelectionRule>,
    pub salvage_grid_rule: ArtifactSalvageGridRule,
    pub artifact_status_rule: ArtifactStatusDetectionRule,
    pub artifact_stat_ocr_rule: ArtifactStatOcrRule,
    pub java_script_rule: ArtifactJavaScriptRule,
    pub finish_rule: FiveStarFinishRule,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSetFilterSelectionRule {
    pub open_filter_by_reusing_quick_select_confirm_region: bool,
    pub after_open_filter_wait_ms: u64,
    pub set_category_click: ArtifactSalvageScreenPoint,
    pub after_set_category_click_wait_ms: u64,
    pub grid_template: GridTemplate,
    pub detection_rule: GridItemDetectionRule,
    pub scroll_rule: GridScrollRule,
    pub classifier_rule: GridIconClassifierRule,
    pub match_policy: ArtifactSetFilterMatchPolicy,
    pub on_before_scroll_clears_overlay: bool,
    pub failed_prediction_draw_text: String,
    pub click_matched_item_wait_ms: u64,
    pub confirm_filter_asset: String,
    pub after_confirm_filter_wait_ms: u64,
    pub confirm_panel_asset: String,
    pub after_confirm_panel_wait_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ArtifactSalvageScreenPoint {
    pub x_1080p: i32,
    pub y_1080p: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSetFilterMatchPolicy {
    pub uses_string_contains: bool,
    pub filter_text: String,
    pub predicted_name_null_is_drawn_as_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSalvageGridRule {
    pub grid_screen_name: GridScreenName,
    pub grid_template: GridTemplate,
    pub detection_rule: GridItemDetectionRule,
    pub scroll_rule: GridScrollRule,
    pub on_after_turn_draws_items: bool,
    pub on_before_scroll_clears_overlay: bool,
    pub only_none_status_items_are_checked: bool,
    pub select_click_wait_ms: u64,
    pub deselect_click_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactStatusDetectionRule {
    pub upper_line_height_ratio: f64,
    pub locked_rule: ArtifactStatusColorRule,
    pub selected_rule: ArtifactStatusColorRule,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactStatusColorRule {
    pub status: ArtifactGridStatus,
    pub common_hsv: ArtifactCommonHsv,
    pub hue_margin: f64,
    pub saturation_margin: f64,
    pub value_margin: f64,
    pub contour_x_max_ratio: Option<f64>,
    pub bounding_width_min_ratio: f64,
    pub bounding_height_min_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactGridStatus {
    None,
    Locked,
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ArtifactCommonHsv {
    pub h: f64,
    pub s: f64,
    pub v: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactStatOcrRule {
    pub card_roi: ArtifactSalvageWidthRelativeRect,
    pub name_roi: ArtifactSalvageWidthRelativeRect,
    pub type_roi: ArtifactSalvageWidthRelativeRect,
    pub main_affix_roi: ArtifactSalvageWidthRelativeRect,
    pub level_and_minor_affix_roi: ArtifactSalvageWidthRelativeRect,
    pub top_hat_kernel: ArtifactSalvageKernelRule,
    pub main_affix_binary_threshold: u8,
    pub ocr_engine: String,
    pub ocr_score_threshold: f64,
    pub minor_affix_left_bound_ratio: f64,
    pub main_affix_value_regex: String,
    pub minor_affix_regex: String,
    pub level_regex: String,
    pub level_min: u8,
    pub level_max: u8,
    pub unactivated_affix_histogram_rule: UnactivatedAffixHistogramRule,
    pub parse_with_game_culture_info: bool,
    pub artifact_stat_js_binding_name: String,
    pub affix_type_names: Vec<ArtifactAffixTypeName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ArtifactSalvageWidthRelativeRect {
    pub x_from_capture_width: f64,
    pub y_from_capture_width: f64,
    pub width_from_capture_width: f64,
    pub height_from_capture_width: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ArtifactSalvageKernelRule {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnactivatedAffixHistogramRule {
    pub enabled_after_recognized_minor_affixes: usize,
    pub background_intensity: u8,
    pub foreground_intensity: u8,
    pub background_must_exceed_foreground: bool,
    pub reject_other_intensity_above_min_background_foreground: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactAffixTypeName {
    pub kind: String,
    pub default_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactJavaScriptRule {
    pub source: String,
    pub engine: String,
    pub flags: Vec<String>,
    pub timeout_ms: u64,
    pub timeout_interrupts_engine: bool,
    pub input_binding: String,
    pub output_binding: String,
    pub output_must_exist: bool,
    pub output_must_be_bool: bool,
    pub true_keeps_artifact_selected: bool,
    pub false_deselects_artifact: bool,
    pub script_engine_exception_is_wrapped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FiveStarFinishRule {
    pub logs_manual_review_required: bool,
    pub manual_review_message: String,
    pub clears_overlay_in_finally: bool,
    pub does_not_click_salvage_confirm_for_five_star: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoArtifactSalvageStep {
    pub phase: AutoArtifactSalvageStepPhase,
    pub action: AutoArtifactSalvageStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoArtifactSalvageStepPhase {
    Setup,
    OpenSalvage,
    QuickSalvage,
    ArtifactSetFilter,
    FiveStarScan,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoArtifactSalvageStepAction {
    ReturnMainUi,
    OpenArtifactsInventory,
    ClickOpenSalvage,
    ClickQuickSelect,
    UnselectStarsAboveConfiguredMaximum,
    ConfirmQuickSelection,
    ClickQuickSalvageConfirm,
    ClickQuickSalvageBlackConfirm,
    ClickBlankAfterQuickSalvageWhenJavaScriptPresent,
    OpenArtifactSetFilter,
    ClickArtifactSetCategory,
    ClassifyAndSelectArtifactSets,
    ConfirmArtifactSetFilter,
    ScanArtifactSalvageGrid,
    DetectLockedOrSelectedState,
    ClickArtifactAndReadDetailCard,
    OcrArtifactStat,
    EvaluateJavaScriptOutput,
    DeselectWhenJavaScriptReturnsFalse,
    ApplyRecognitionFailurePolicy,
    StopWhenMaxCheckCountReached,
    PromptManualReview,
    EscapeAndReturnMainUiWhenQuickOnly,
    ClearVisionOverlay,
}

pub fn plan_auto_artifact_salvage(
    config: AutoArtifactSalvageExecutionConfig,
) -> Result<AutoArtifactSalvageExecutionPlan> {
    validate_param(&config.param)?;
    let java_script_present = config.param.java_script.is_some();
    let artifact_set_filter_enabled = config
        .param
        .artifact_set_filter
        .as_deref()
        .is_some_and(|filter| !filter.trim().is_empty());

    Ok(AutoArtifactSalvageExecutionPlan {
        task_key: AUTO_ARTIFACT_SALVAGE_TASK_KEY.to_string(),
        display_name: AUTO_ARTIFACT_SALVAGE_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: false,
        capture_size: config.capture_size,
        config_rule: config_rule(&config.param),
        open_inventory_rule: open_inventory_rule(),
        quick_salvage_rule: quick_salvage_rule(),
        five_star_rule: java_script_present.then(|| five_star_rule(&config.param)),
        steps: steps(java_script_present, artifact_set_filter_enabled),
        pending_native: pending_native(java_script_present, artifact_set_filter_enabled),
    })
}

fn validate_param(param: &AutoArtifactSalvageParam) -> Result<()> {
    if !(1..=4).contains(&param.star) {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_ARTIFACT_SALVAGE_TASK_KEY.to_string(),
            message: format!(
                "max artifact star must be between 1 and 4, got {}",
                param.star
            ),
        });
    }
    if param.java_script.is_some()
        && (param.max_num_to_check.is_none() || param.recognition_failure_policy.is_none())
    {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_ARTIFACT_SALVAGE_TASK_KEY.to_string(),
            message: "javaScript mode requires maxNumToCheck and recognitionFailurePolicy"
                .to_string(),
        });
    }
    Ok(())
}

fn config_rule(param: &AutoArtifactSalvageParam) -> AutoArtifactSalvageConfigRule {
    let selected_quick_salvage_stars = (1..=param.star).collect();
    let unselected_after_quick_select_stars = if param.star < 4 {
        ((param.star + 1)..=4).collect()
    } else {
        Vec::new()
    };
    AutoArtifactSalvageConfigRule {
        star: param.star,
        selected_quick_salvage_stars,
        unselected_after_quick_select_stars,
        java_script_present: param.java_script.is_some(),
        java_script_is_blank: param
            .java_script
            .as_deref()
            .is_some_and(|script| script.trim().is_empty()),
        artifact_set_filter_present: param
            .artifact_set_filter
            .as_deref()
            .is_some_and(|filter| !filter.trim().is_empty()),
        artifact_set_filter_contains_predicted_name: true,
        max_num_to_check: param.max_num_to_check,
        zero_max_count_still_checks_first_selected_item: param.max_num_to_check == Some(0),
        recognition_failure_policy: param.recognition_failure_policy,
        recognition_failure_skip_does_not_consume_check_count: true,
        regular_expression_legacy_unused: param.regular_expression.clone(),
        valid_star_range: "1..=4".to_string(),
    }
}

fn open_inventory_rule() -> AutoArtifactSalvageOpenInventoryRule {
    AutoArtifactSalvageOpenInventoryRule {
        return_main_ui_before_open: true,
        grid_screen_name: GridScreenName::Artifacts,
        open_inventory_action: GenshinAction::OpenInventory,
        open_wait_ms: 1_200,
        retry_attempts: 5,
        retry_when_main_ui_detected: true,
        expired_item_prompt_confirm_asset: AUTO_ARTIFACT_SALVAGE_WHITE_CONFIRM_ASSET.to_string(),
        expired_item_prompt_crop_bottom_ratio: 0.2,
        expired_item_prompt_wait_ms: 300,
        inventory_tab_assets: GridScreenName::Artifacts
            .inventory_tab_assets()
            .expect("Artifacts inventory tab assets should exist"),
        after_tab_ready_wait_ms: 800,
    }
}

fn quick_salvage_rule() -> QuickArtifactSalvageRule {
    QuickArtifactSalvageRule {
        opens_salvage_button: ArtifactSalvageTemplateLocator {
            name: "BtnArtifactSalvage".to_string(),
            asset: AUTO_ARTIFACT_SALVAGE_BTN_ASSET.to_string(),
            roi: ArtifactSalvageRelativeRoi {
                cut: "CutBottom".to_string(),
                width_ratio: None,
                height_ratio: Some(0.1),
            },
            draw_on_window: false,
        },
        quick_select_ocr_rule: ArtifactSalvageOcrButtonRule {
            text_key: "快速选择".to_string(),
            default_regex: "快速选择".to_string(),
            roi: ArtifactSalvageRelativeRoi {
                cut: "CutLeftBottom".to_string(),
                width_ratio: Some(0.25),
                height_ratio: Some(0.1),
            },
            click_wait_ms: 500,
        },
        star_option_ocr_rule: ArtifactSalvageStarOptionRule {
            localized_regexes_by_star: (1..=4)
                .map(|star| ArtifactSalvageStarText {
                    star,
                    text_key: format!("{star}星圣遗物"),
                    default_regex: format!("{star}星圣遗物"),
                })
                .collect(),
            roi: ArtifactSalvageRelativeRoi {
                cut: "CutLeft".to_string(),
                width_ratio: Some(0.20),
                height_ratio: None,
            },
            unselect_wait_ms: 500,
            legacy_inverse_selection_since_5_5: true,
        },
        quick_select_confirm_asset: AUTO_ARTIFACT_SALVAGE_WHITE_CONFIRM_ASSET.to_string(),
        quick_select_confirm_reused_as_filter_button: true,
        quick_select_confirm_wait_ms: 1_500,
        salvage_confirm_asset: AUTO_ARTIFACT_SALVAGE_CONFIRM_ASSET.to_string(),
        after_salvage_confirm_wait_ms: 1_000,
        final_confirm_asset: AUTO_ARTIFACT_SALVAGE_BLACK_CONFIRM_ASSET.to_string(),
        final_confirm_kind: ArtifactSalvageConfirmKind::Black,
        after_final_confirm_wait_ms: 400,
        post_quick_salvage_click_when_js_present: true,
        no_quick_items_is_not_fatal: true,
        destructive_native_action: true,
    }
}

fn five_star_rule(param: &AutoArtifactSalvageParam) -> FiveStarArtifactFilterRule {
    let artifact_set_filter = param.artifact_set_filter.clone().unwrap_or_default();
    FiveStarArtifactFilterRule {
        artifact_set_filter_rule: (!artifact_set_filter.trim().is_empty()).then(|| {
            ArtifactSetFilterSelectionRule {
                open_filter_by_reusing_quick_select_confirm_region: true,
                after_open_filter_wait_ms: 400,
                set_category_click: ArtifactSalvageScreenPoint {
                    x_1080p: 315,
                    y_1080p: 190,
                },
                after_set_category_click_wait_ms: 1_000,
                grid_template: artifact_set_filter_grid_template(),
                detection_rule: artifact_set_filter_detection_rule(),
                scroll_rule: scroll_rule_for_grid_template(&artifact_set_filter_grid_template()),
                classifier_rule: grid_icon_classifier_rule(),
                match_policy: ArtifactSetFilterMatchPolicy {
                    uses_string_contains: true,
                    filter_text: artifact_set_filter,
                    predicted_name_null_is_drawn_as_failure: true,
                },
                on_before_scroll_clears_overlay: true,
                failed_prediction_draw_text: "识别失败".to_string(),
                click_matched_item_wait_ms: 100,
                confirm_filter_asset: AUTO_ARTIFACT_SALVAGE_WHITE_CONFIRM_ASSET.to_string(),
                after_confirm_filter_wait_ms: 1_500,
                confirm_panel_asset: AUTO_ARTIFACT_SALVAGE_WHITE_CONFIRM_ASSET.to_string(),
                after_confirm_panel_wait_ms: 600,
            }
        }),
        salvage_grid_rule: ArtifactSalvageGridRule {
            grid_screen_name: GridScreenName::ArtifactSalvage,
            grid_template: GridTemplate::artifact_salvage(),
            detection_rule: standard_grid_detection_rule(),
            scroll_rule: scroll_rule_for_grid_template(&GridTemplate::artifact_salvage()),
            on_after_turn_draws_items: true,
            on_before_scroll_clears_overlay: true,
            only_none_status_items_are_checked: true,
            select_click_wait_ms: 300,
            deselect_click_wait_ms: 100,
        },
        artifact_status_rule: artifact_status_detection_rule(),
        artifact_stat_ocr_rule: artifact_stat_ocr_rule(),
        java_script_rule: ArtifactJavaScriptRule {
            source: param.java_script.clone().unwrap_or_default(),
            engine: "ClearScript V8ScriptEngine".to_string(),
            flags: vec![
                "UseCaseInsensitiveMemberBinding".to_string(),
                "DisableGlobalMembers".to_string(),
            ],
            timeout_ms: 3_000,
            timeout_interrupts_engine: true,
            input_binding: "ArtifactStat".to_string(),
            output_binding: "Output".to_string(),
            output_must_exist: true,
            output_must_be_bool: true,
            true_keeps_artifact_selected: true,
            false_deselects_artifact: true,
            script_engine_exception_is_wrapped: true,
        },
        finish_rule: FiveStarFinishRule {
            logs_manual_review_required: true,
            manual_review_message: "筛选完毕，请复查并手动分解".to_string(),
            clears_overlay_in_finally: true,
            does_not_click_salvage_confirm_for_five_star: true,
        },
    }
}

fn steps(
    java_script_present: bool,
    artifact_set_filter_enabled: bool,
) -> Vec<AutoArtifactSalvageStep> {
    let mut steps = vec![
        step(
            AutoArtifactSalvageStepPhase::Setup,
            AutoArtifactSalvageStepAction::ReturnMainUi,
        ),
        step(
            AutoArtifactSalvageStepPhase::OpenSalvage,
            AutoArtifactSalvageStepAction::OpenArtifactsInventory,
        ),
        step(
            AutoArtifactSalvageStepPhase::OpenSalvage,
            AutoArtifactSalvageStepAction::ClickOpenSalvage,
        ),
        step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::ClickQuickSelect,
        ),
        step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::UnselectStarsAboveConfiguredMaximum,
        ),
        step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::ConfirmQuickSelection,
        ),
        step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::ClickQuickSalvageConfirm,
        ),
        step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::ClickQuickSalvageBlackConfirm,
        ),
    ];

    if java_script_present {
        steps.push(step(
            AutoArtifactSalvageStepPhase::QuickSalvage,
            AutoArtifactSalvageStepAction::ClickBlankAfterQuickSalvageWhenJavaScriptPresent,
        ));
        if artifact_set_filter_enabled {
            steps.extend([
                step(
                    AutoArtifactSalvageStepPhase::ArtifactSetFilter,
                    AutoArtifactSalvageStepAction::OpenArtifactSetFilter,
                ),
                step(
                    AutoArtifactSalvageStepPhase::ArtifactSetFilter,
                    AutoArtifactSalvageStepAction::ClickArtifactSetCategory,
                ),
                step(
                    AutoArtifactSalvageStepPhase::ArtifactSetFilter,
                    AutoArtifactSalvageStepAction::ClassifyAndSelectArtifactSets,
                ),
                step(
                    AutoArtifactSalvageStepPhase::ArtifactSetFilter,
                    AutoArtifactSalvageStepAction::ConfirmArtifactSetFilter,
                ),
            ]);
        }
        steps.extend([
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::ScanArtifactSalvageGrid,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::DetectLockedOrSelectedState,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::ClickArtifactAndReadDetailCard,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::OcrArtifactStat,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::EvaluateJavaScriptOutput,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::DeselectWhenJavaScriptReturnsFalse,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::ApplyRecognitionFailurePolicy,
            ),
            step(
                AutoArtifactSalvageStepPhase::FiveStarScan,
                AutoArtifactSalvageStepAction::StopWhenMaxCheckCountReached,
            ),
            step(
                AutoArtifactSalvageStepPhase::Cleanup,
                AutoArtifactSalvageStepAction::PromptManualReview,
            ),
            step(
                AutoArtifactSalvageStepPhase::Cleanup,
                AutoArtifactSalvageStepAction::ClearVisionOverlay,
            ),
        ]);
    } else {
        steps.extend([
            step(
                AutoArtifactSalvageStepPhase::Cleanup,
                AutoArtifactSalvageStepAction::EscapeAndReturnMainUiWhenQuickOnly,
            ),
            step(
                AutoArtifactSalvageStepPhase::Cleanup,
                AutoArtifactSalvageStepAction::ClearVisionOverlay,
            ),
        ]);
    }

    steps
}

fn step(
    phase: AutoArtifactSalvageStepPhase,
    action: AutoArtifactSalvageStepAction,
) -> AutoArtifactSalvageStep {
    AutoArtifactSalvageStep { phase, action }
}

fn pending_native(java_script_present: bool, artifact_set_filter_enabled: bool) -> Vec<String> {
    let mut pending = vec![
        "ReturnMainUiTask, SendInput OpenInventory/Escape, and mouse click dispatch".to_string(),
        "live capture, template matching, prompt dialog detection, and BV confirm button clicks"
            .to_string(),
        "destructive 1-4 star quick salvage confirmation remains intentionally executor-disabled"
            .to_string(),
        "localized OCR for quick-select and star option buttons".to_string(),
    ];
    if artifact_set_filter_enabled {
        pending.push(
            "ArtifactSetFilter grid enumeration, overlay drawing, ONNX gridIcon inference, and filter confirmation"
                .to_string(),
        );
    }
    if java_script_present {
        pending.extend([
            "ArtifactSalvage grid enumeration and locked/selected color-state OpenCV recognition"
                .to_string(),
            "Paddle OCR artifact detail parsing, localized affix mapping, and unactivated affix histogram detection"
                .to_string(),
            "ClearScript V8 JavaScript evaluation with timeout, Output validation, and recognition failure policy"
                .to_string(),
            "five-star result is selection-only and requires manual review before actual salvage".to_string(),
        ]);
    }
    pending
}

fn artifact_set_filter_grid_template() -> GridTemplate {
    GridTemplate {
        roi_1080p: Rect {
            x: 40,
            y: 100,
            width: 1300,
            height: 852,
        },
        columns: 2,
        s1_round: 3,
        round_milliseconds: 40,
        s2_round: 40,
        s3_scale: 0.024,
    }
}

fn standard_grid_detection_rule() -> GridItemDetectionRule {
    GridItemDetectionRule {
        min_width_per_column_ratio: 0.66,
        shape_ratio_target: 0.81,
        shape_ratio_tolerance: 0.03,
        top_right_exclusion_x_ratio: 0.60,
        top_right_exclusion_y_ratio: 0.28,
        canny_low_threshold: 20.0,
        canny_high_threshold: 40.0,
        close_kernel_width: 5,
        close_kernel_height: 5,
        fill_missing_threshold_roi_height_ratio: 0.025,
        phantom_cell_bgr: GridBgrColor {
            b: 0xdc,
            g: 0xe5,
            r: 0xe9,
        },
        phantom_cell_tolerance: 30,
    }
}

fn artifact_set_filter_detection_rule() -> GridItemDetectionRule {
    GridItemDetectionRule {
        shape_ratio_target: 8.63,
        shape_ratio_tolerance: 0.4,
        close_kernel_width: 3,
        close_kernel_height: 3,
        ..standard_grid_detection_rule()
    }
}

fn scroll_rule_for_grid_template(grid_template: &GridTemplate) -> GridScrollRule {
    GridScrollRule {
        test_scroll_rounds: grid_template.s1_round,
        page_scroll_rounds: grid_template.s2_round,
        scroll_delta_per_round: -2,
        fine_scroll_delta: -1,
        round_wait_ms: grid_template.round_milliseconds,
        settle_wait_ms: 300,
        fine_scroll_check_interval_ms: 60,
        fine_scroll_timeout_ms: 2_000,
        phase_correlation_lower_threshold: 0.5,
        phase_correlation_upper_threshold: 0.95,
    }
}

fn grid_icon_classifier_rule() -> GridIconClassifierRule {
    GridIconClassifierRule {
        model_name: GRID_ICON_MODEL_NAME.to_string(),
        model_path: GRID_ICON_MODEL_PATH.to_string(),
        prototype_csv_path: GRID_ICON_PROTOTYPE_CSV_PATH.to_string(),
        input_name: GRID_ICON_INPUT_NAME.to_string(),
        feature_dimensions: 64,
        max_distance_squared: 100.0,
    }
}

fn artifact_status_detection_rule() -> ArtifactStatusDetectionRule {
    ArtifactStatusDetectionRule {
        upper_line_height_ratio: 0.19,
        locked_rule: ArtifactStatusColorRule {
            status: ArtifactGridStatus::Locked,
            common_hsv: ArtifactCommonHsv {
                h: 9.0,
                s: 0.54,
                v: 1.00,
            },
            hue_margin: 3.0,
            saturation_margin: 25.0,
            value_margin: 25.0,
            contour_x_max_ratio: Some(0.2),
            bounding_width_min_ratio: 0.07,
            bounding_height_min_ratio: 0.3,
        },
        selected_rule: ArtifactStatusColorRule {
            status: ArtifactGridStatus::Selected,
            common_hsv: ArtifactCommonHsv {
                h: 80.0,
                s: 0.76,
                v: 1.00,
            },
            hue_margin: 3.0,
            saturation_margin: 10.0,
            value_margin: 5.0,
            contour_x_max_ratio: None,
            bounding_width_min_ratio: 0.2,
            bounding_height_min_ratio: 0.8,
        },
    }
}

fn artifact_stat_ocr_rule() -> ArtifactStatOcrRule {
    ArtifactStatOcrRule {
        card_roi: ArtifactSalvageWidthRelativeRect {
            x_from_capture_width: 0.70,
            y_from_capture_width: 0.112,
            width_from_capture_width: 0.275,
            height_from_capture_width: 0.50,
        },
        name_roi: ArtifactSalvageWidthRelativeRect {
            x_from_capture_width: 0.0,
            y_from_capture_width: 0.0,
            width_from_capture_width: 1.0,
            height_from_capture_width: 0.106,
        },
        type_roi: ArtifactSalvageWidthRelativeRect {
            x_from_capture_width: 0.0,
            y_from_capture_width: 0.106,
            width_from_capture_width: 1.0,
            height_from_capture_width: 0.106,
        },
        main_affix_roi: ArtifactSalvageWidthRelativeRect {
            x_from_capture_width: 0.0,
            y_from_capture_width: 0.22,
            width_from_capture_width: 0.55,
            height_from_capture_width: 0.30,
        },
        level_and_minor_affix_roi: ArtifactSalvageWidthRelativeRect {
            x_from_capture_width: 0.0,
            y_from_capture_width: 0.52,
            width_from_capture_width: 1.0,
            height_from_capture_width: 0.48,
        },
        top_hat_kernel: ArtifactSalvageKernelRule {
            width: 15,
            height: 15,
        },
        main_affix_binary_threshold: 30,
        ocr_engine: "Paddle".to_string(),
        ocr_score_threshold: 0.5,
        minor_affix_left_bound_ratio: 0.1,
        main_affix_value_regex: r"^([\d., ]*)(%?)$".to_string(),
        minor_affix_regex: r"^([^+:：]+)\+([\d., 。]*)(%?).*$".to_string(),
        level_regex: r"^\+(\d*)$".to_string(),
        level_min: 0,
        level_max: 20,
        unactivated_affix_histogram_rule: UnactivatedAffixHistogramRule {
            enabled_after_recognized_minor_affixes: 3,
            background_intensity: 222,
            foreground_intensity: 152,
            background_must_exceed_foreground: true,
            reject_other_intensity_above_min_background_foreground: true,
        },
        parse_with_game_culture_info: true,
        artifact_stat_js_binding_name: "ArtifactStat".to_string(),
        affix_type_names: artifact_affix_type_names(),
    }
}

fn artifact_affix_type_names() -> Vec<ArtifactAffixTypeName> {
    [
        ("ATK", "攻击力"),
        ("ATKPercent", "攻击力"),
        ("DEF", "防御力"),
        ("DEFPercent", "防御力"),
        ("HP", "生命值"),
        ("HPPercent", "生命值"),
        ("CRITRate", "暴击率"),
        ("CRITDMG", "暴击伤害"),
        ("ElementalMastery", "元素精通"),
        ("EnergyRecharge", "元素充能效率"),
        ("HealingBonus", "治疗加成"),
        ("PhysicalDMGBonus", "物理伤害加成"),
        ("PyroDMGBonus", "火元素伤害加成"),
        ("HydroDMGBonus", "水元素伤害加成"),
        ("DendroDMGBonus", "草元素伤害加成"),
        ("ElectroDMGBonus", "雷元素伤害加成"),
        ("AnemoDMGBonus", "风元素伤害加成"),
        ("CryoDMGBonus", "冰元素伤害加成"),
        ("GeoDMGBonus", "岩元素伤害加成"),
    ]
    .into_iter()
    .map(|(kind, default_text)| ArtifactAffixTypeName {
        kind: kind.to_string(),
        default_text: default_text.to_string(),
    })
    .collect()
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

fn overlay_param(param: &mut AutoArtifactSalvageParam, value: &Value) {
    if let Some(value) = member(
        value,
        [
            "star",
            "Star",
            "maxArtifactStar",
            "MaxArtifactStar",
            "max_artifact_star",
        ],
    ) {
        if let Some(star) = parse_star_value(value) {
            param.star = star;
        }
    }
    if let Some(value) = member(value, ["javaScript", "JavaScript", "java_script"]) {
        param.java_script = string_or_null(value);
    }
    if let Some(value) = member(
        value,
        [
            "artifactSetFilter",
            "ArtifactSetFilter",
            "artifact_set_filter",
        ],
    ) {
        param.artifact_set_filter = string_or_null(value);
    }
    if let Some(value) = member(
        value,
        [
            "regularExpression",
            "RegularExpression",
            "regular_expression",
        ],
    ) {
        if let Some(regular_expression) = value.as_str() {
            param.regular_expression = regular_expression.to_string();
        }
    }
    if let Some(value) = member(
        value,
        ["maxNumToCheck", "MaxNumToCheck", "max_num_to_check"],
    ) {
        param.max_num_to_check = u64_or_null(value);
    }
    if let Some(value) = member(
        value,
        [
            "recognitionFailurePolicy",
            "RecognitionFailurePolicy",
            "recognition_failure_policy",
        ],
    ) {
        param.recognition_failure_policy = parse_optional_recognition_failure_policy(value);
    }
}

fn parse_star_value(value: &Value) -> Option<u8> {
    match value {
        Value::Number(number) => number.as_u64().and_then(|value| u8::try_from(value).ok()),
        Value::String(value) => value.trim().parse::<u8>().ok(),
        _ => None,
    }
}

fn parse_optional_recognition_failure_policy(
    value: &Value,
) -> Option<AutoArtifactRecognitionFailurePolicy> {
    if value.is_null() {
        return None;
    }
    parse_recognition_failure_policy(value)
}

fn parse_recognition_failure_policy(value: &Value) -> Option<AutoArtifactRecognitionFailurePolicy> {
    match value {
        Value::Number(number) => match number.as_u64()? {
            0 => Some(AutoArtifactRecognitionFailurePolicy::Skip),
            1 => Some(AutoArtifactRecognitionFailurePolicy::Abort),
            _ => None,
        },
        Value::String(value) => match value.trim() {
            "Skip" | "skip" | "跳过" => Some(AutoArtifactRecognitionFailurePolicy::Skip),
            "Abort" | "abort" | "终止" => Some(AutoArtifactRecognitionFailurePolicy::Abort),
            "0" => Some(AutoArtifactRecognitionFailurePolicy::Skip),
            "1" => Some(AutoArtifactRecognitionFailurePolicy::Abort),
            _ => None,
        },
        _ => None,
    }
}

fn string_or_null(value: &Value) -> Option<String> {
    if value.is_null() {
        return None;
    }
    value.as_str().map(str::to_string)
}

fn u64_or_null(value: &Value) -> Option<u64> {
    if value.is_null() {
        return None;
    }
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|value| u64::try_from(value).ok()))
        .or_else(|| value.as_str().and_then(|value| value.trim().parse().ok()))
}

fn u64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u64> {
    member(value, keys).and_then(u64_or_null)
}

fn member<'a, const N: usize>(value: &'a Value, keys: [&str; N]) -> Option<&'a Value> {
    keys.into_iter().find_map(|key| value.get(key))
}
