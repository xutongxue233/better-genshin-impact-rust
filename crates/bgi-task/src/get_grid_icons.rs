use bgi_core::config::GetGridIconsConfig;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common_job::{
    GridIconClassifierRule, GridItemDetectionRule, GridScreenName, GridScrollRule, GridTemplate,
    InventoryTabAssetPair, GRID_ICON_INPUT_NAME, GRID_ICON_MODEL_NAME, GRID_ICON_MODEL_PATH,
    GRID_ICON_PROTOTYPE_CSV_PATH,
};
use crate::{Result, TaskError, TaskPortState};

pub const GET_GRID_ICONS_TASK_KEY: &str = "GetGridIcons";
pub const GET_GRID_ICONS_DISPLAY_NAME: &str = "获取Grid界面物品图标";
pub const GET_GRID_ICONS_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const GET_GRID_ICONS_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const GET_GRID_ICONS_OUTPUT_ROOT: &str = "log/gridIcons";
pub const GET_GRID_ICONS_OUTPUT_DIRECTORY_PATTERN: &str =
    "log/gridIcons/{GridScreenName}{yyyyMMddHHmmss}";
pub const GET_GRID_ICONS_MAX_DEFAULT: u64 = i32::MAX as u64;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub config_rule: GetGridIconsConfigRule,
    pub open_rule: GetGridIconsOpenRule,
    pub grid_rule: GetGridIconsGridRule,
    pub capture_rule: GetGridIconsCaptureRule,
    pub artifact_set_filter_rule: Option<GetGridIconsArtifactSetFilterRule>,
    pub output_rule: GetGridIconsOutputRule,
    pub model_accuracy_rule: GetGridIconsModelAccuracyRule,
    pub steps: Vec<GetGridIconsStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGridIconsExecutionConfig {
    pub capture_size: Size,
    pub get_grid_icons_config: GetGridIconsConfig,
}

impl Default for GetGridIconsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                GET_GRID_ICONS_DEFAULT_CAPTURE_WIDTH,
                GET_GRID_ICONS_DEFAULT_CAPTURE_HEIGHT,
            ),
            get_grid_icons_config: GetGridIconsConfig::default(),
        }
    }
}

impl GetGridIconsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let get_grid_icons_value = value
            .get("getGridIconsConfig")
            .or_else(|| value.get("GetGridIconsConfig"))
            .or_else(|| value.get("get_grid_icons_config"))
            .unwrap_or(value);
        config.get_grid_icons_config =
            serde_json::from_value(get_grid_icons_value.clone()).unwrap_or_default();

        overlay_get_grid_icons_config(&mut config.get_grid_icons_config, value);
        if get_grid_icons_value as *const Value != value as *const Value {
            overlay_get_grid_icons_config(&mut config.get_grid_icons_config, get_grid_icons_value);
        }
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsConfigRule {
    pub grid_name_raw: Value,
    pub grid_screen_name: GridScreenName,
    pub grid_screen_description: String,
    pub star_as_suffix: bool,
    pub lv_as_suffix: bool,
    pub lv_as_suffix_legacy_config_present_but_unused: bool,
    pub max_num_to_get: u64,
    pub zero_max_count_stops_without_scanning: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsOpenRule {
    pub auto_open_inventory: bool,
    pub requires_manual_open: bool,
    pub unsupported_auto_open_still_attempts_grid_scan: bool,
    pub return_main_ui_before_inventory_open: bool,
    pub inventory_tab_assets: Option<InventoryTabAssetPair>,
    pub manual_open_message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsGridRule {
    pub grid_template: GridTemplate,
    pub detection_rule: GridItemDetectionRule,
    pub scroll_rule: GridScrollRule,
    pub uses_artifact_set_filter_screen: bool,
    pub on_after_turn_draws_items: bool,
    pub on_before_scroll_clears_overlay: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsCaptureRule {
    pub item_click_delay_ms: u64,
    pub item_name_ocr_roi: GetGridIconsWidthRelativeRect,
    pub item_name_ocr_engine: String,
    pub star_suffix_rule: Option<GetGridIconsStarSuffixRule>,
    pub lv_suffix_config_is_ignored_by_legacy_task: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsWidthRelativeRect {
    pub x_from_capture_width: f64,
    pub y_from_capture_width: f64,
    pub width_from_capture_width: f64,
    pub height_from_capture_width: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsStarSuffixRule {
    pub star_ocr_roi: GetGridIconsWidthRelativeRect,
    pub yellow_bgr_lower: GetGridIconsBgrColor,
    pub yellow_bgr_upper: GetGridIconsBgrColor,
    pub contour_count_maps_to_star_glyphs: bool,
    pub glyph: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GetGridIconsBgrColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsArtifactSetFilterRule {
    pub grid_template: GridTemplate,
    pub item_shape_ratio_target: f64,
    pub item_shape_ratio_tolerance: f64,
    pub close_kernel_width: i32,
    pub close_kernel_height: i32,
    pub flower_name_roi: GetGridIconsWidthRelativeRect,
    pub flower_without_glyph_roi: GetGridIconsFlowerWithoutGlyphRule,
    pub anchor_text: String,
    pub retry_scroll_rounds: u64,
    pub retry_scroll_delta_per_round: i32,
    pub retry_scroll_interval_ms: u64,
    pub retry_wait_ms: u64,
    pub icon_crop_rule: ArtifactSetFilterIconCropRule,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsFlowerWithoutGlyphRule {
    pub x_from_name_region_width: f64,
    pub y_from_detected_line_height: f64,
    pub width_from_capture_width: f64,
    pub height_from_detected_line_height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArtifactSetFilterIconCropRule {
    pub source_width: u32,
    pub source_height: u32,
    pub normalized_width: u32,
    pub normalized_height: u32,
    pub x_formula: String,
    pub y_formula: String,
    pub clamps_to_item_region: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsOutputRule {
    pub output_root: String,
    pub output_directory_pattern: String,
    pub file_name_pattern: String,
    pub duplicate_file_names_are_skipped: bool,
    pub file_name_is_not_sanitized_by_legacy_task: bool,
    pub saves_png_on_background_thread: bool,
    pub save_failures_are_logged_not_fatal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsModelAccuracyRule {
    pub related_task_name: String,
    pub model_rule: GridIconClassifierRule,
    pub loads_prefix_list_metadata: bool,
    pub prototype_csv_skips_header: bool,
    pub star_output_uses_argmax_of_third_output: bool,
    pub main_collection_does_not_require_model: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetGridIconsStep {
    pub phase: GetGridIconsStepPhase,
    pub action: GetGridIconsStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GetGridIconsStepPhase {
    Setup,
    OpenGrid,
    ScanGrid,
    CaptureItem,
    Save,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum GetGridIconsStepAction {
    CreateTimestampedOutputDirectory,
    ReturnMainUi,
    OpenInventoryTab,
    RequireManualGridOpen,
    EnumerateGridItems,
    ClickItemAndWait,
    OcrItemName,
    CountStarsWhenConfigured,
    OcrArtifactSetFlowerName,
    CropArtifactSetIcon,
    SaveUniquePng,
    StopWhenMaxCountReached,
    ClearVisionOverlay,
}

pub fn plan_get_grid_icons(
    config: GetGridIconsExecutionConfig,
) -> Result<GetGridIconsExecutionPlan> {
    let grid_screen_name = parse_grid_screen_name(&config.get_grid_icons_config.grid_name)?;
    let max_num_to_get = config.get_grid_icons_config.max_num_to_get;
    let open_rule = open_rule_for_grid_screen(&grid_screen_name);
    let grid_rule = grid_rule_for_grid_screen(&grid_screen_name);
    let artifact_set_filter_rule =
        (grid_screen_name == GridScreenName::ArtifactSetFilter).then(artifact_set_filter_rule);

    Ok(GetGridIconsExecutionPlan {
        task_key: GET_GRID_ICONS_TASK_KEY.to_string(),
        display_name: GET_GRID_ICONS_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: false,
        capture_size: config.capture_size,
        config_rule: GetGridIconsConfigRule {
            grid_name_raw: config.get_grid_icons_config.grid_name.clone(),
            grid_screen_description: grid_screen_description(&grid_screen_name).to_string(),
            grid_screen_name: grid_screen_name.clone(),
            star_as_suffix: config.get_grid_icons_config.star_as_suffix,
            lv_as_suffix: config.get_grid_icons_config.lv_as_suffix,
            lv_as_suffix_legacy_config_present_but_unused: true,
            max_num_to_get,
            zero_max_count_stops_without_scanning: max_num_to_get == 0,
        },
        open_rule,
        grid_rule,
        capture_rule: capture_rule(config.get_grid_icons_config.star_as_suffix),
        artifact_set_filter_rule,
        output_rule: GetGridIconsOutputRule {
            output_root: GET_GRID_ICONS_OUTPUT_ROOT.to_string(),
            output_directory_pattern: GET_GRID_ICONS_OUTPUT_DIRECTORY_PATTERN.to_string(),
            file_name_pattern: "{ocr_item_name}{optional_star_glyphs}.png".to_string(),
            duplicate_file_names_are_skipped: true,
            file_name_is_not_sanitized_by_legacy_task: true,
            saves_png_on_background_thread: true,
            save_failures_are_logged_not_fatal: true,
        },
        model_accuracy_rule: model_accuracy_rule(),
        steps: get_grid_icons_steps(&grid_screen_name, config.get_grid_icons_config.star_as_suffix),
        pending_native: vec![
            "TaskRunner/ISoloTask cancellation lifecycle".to_string(),
            "ReturnMainUiTask and AutoArtifactSalvageTask.OpenInventory input execution".to_string(),
            "GameCaptureRegion live capture and ImageRegion crop/click behavior".to_string(),
            "OpenCV GridScreen/ArtifactSetFilterScreen contour detection, GridCell clustering, and phase-correlation scrolling".to_string(),
            "Paddle OCR for item names and artifact-set flower names".to_string(),
            "SendInput mouse clicks, wheel scrolling, and inventory hotkey dispatch".to_string(),
            "PNG filesystem writes on background threads and overlay clearing".to_string(),
            "Optional GridIconsAccuracyTestTask ONNX/prototype inference".to_string(),
        ],
    })
}

fn parse_grid_screen_name(value: &Value) -> Result<GridScreenName> {
    if let Some(name) = value.as_str() {
        return grid_screen_name_from_str(name).ok_or_else(|| invalid_grid_name(value));
    }
    if let Some(index) = value.as_i64() {
        return grid_screen_name_from_index(index).ok_or_else(|| invalid_grid_name(value));
    }
    invalid_grid_name_result(value)
}

fn grid_screen_name_from_str(value: &str) -> Option<GridScreenName> {
    match value {
        "Weapons" | "武器" => Some(GridScreenName::Weapons),
        "Artifacts" | "圣遗物" => Some(GridScreenName::Artifacts),
        "CharacterDevelopmentItems" | "养成道具" => {
            Some(GridScreenName::CharacterDevelopmentItems)
        }
        "Food" | "食物" => Some(GridScreenName::Food),
        "Materials" | "材料" => Some(GridScreenName::Materials),
        "Gadget" | "小道具" => Some(GridScreenName::Gadget),
        "Quest" | "任务" => Some(GridScreenName::Quest),
        "PreciousItems" | "贵重道具" => Some(GridScreenName::PreciousItems),
        "Furnishings" | "摆设" => Some(GridScreenName::Furnishings),
        "ArtifactSalvage" | "圣遗物分解" => Some(GridScreenName::ArtifactSalvage),
        "ArtifactSetFilter" | "圣遗物套装筛选" => Some(GridScreenName::ArtifactSetFilter),
        _ => None,
    }
}

fn grid_screen_name_from_index(value: i64) -> Option<GridScreenName> {
    match value {
        0 => Some(GridScreenName::Weapons),
        1 => Some(GridScreenName::Artifacts),
        2 => Some(GridScreenName::CharacterDevelopmentItems),
        3 => Some(GridScreenName::Food),
        4 => Some(GridScreenName::Materials),
        5 => Some(GridScreenName::Gadget),
        6 => Some(GridScreenName::Quest),
        7 => Some(GridScreenName::PreciousItems),
        8 => Some(GridScreenName::Furnishings),
        9 => Some(GridScreenName::ArtifactSalvage),
        10 => Some(GridScreenName::ArtifactSetFilter),
        _ => None,
    }
}

fn invalid_grid_name(value: &Value) -> TaskError {
    TaskError::InvalidTaskConfig {
        key: GET_GRID_ICONS_TASK_KEY.to_string(),
        message: format!("unsupported gridName: {value}"),
    }
}

fn invalid_grid_name_result<T>(value: &Value) -> Result<T> {
    Err(invalid_grid_name(value))
}

fn grid_screen_description(grid: &GridScreenName) -> &'static str {
    match grid {
        GridScreenName::Weapons => "武器",
        GridScreenName::Artifacts => "圣遗物",
        GridScreenName::CharacterDevelopmentItems => "养成道具",
        GridScreenName::Food => "食物",
        GridScreenName::Materials => "材料",
        GridScreenName::Gadget => "小道具",
        GridScreenName::Quest => "任务",
        GridScreenName::PreciousItems => "贵重道具",
        GridScreenName::Furnishings => "摆设",
        GridScreenName::ArtifactSalvage => "圣遗物分解",
        GridScreenName::ArtifactSetFilter => "圣遗物套装筛选",
    }
}

fn open_rule_for_grid_screen(grid: &GridScreenName) -> GetGridIconsOpenRule {
    let inventory_tab_assets = grid.inventory_tab_assets();
    let auto_open_inventory = inventory_tab_assets.is_some();
    let requires_manual_open = matches!(
        grid,
        GridScreenName::ArtifactSetFilter | GridScreenName::ArtifactSalvage
    );
    GetGridIconsOpenRule {
        auto_open_inventory,
        requires_manual_open,
        unsupported_auto_open_still_attempts_grid_scan: *grid == GridScreenName::ArtifactSalvage,
        return_main_ui_before_inventory_open: auto_open_inventory,
        inventory_tab_assets,
        manual_open_message: format!(
            "{}暂不支持自动打开，请提前手动打开界面",
            grid_screen_description(grid)
        ),
    }
}

fn grid_rule_for_grid_screen(grid: &GridScreenName) -> GetGridIconsGridRule {
    let grid_template = if *grid == GridScreenName::ArtifactSetFilter {
        artifact_set_filter_grid_template()
    } else {
        grid.grid_template()
    };
    GetGridIconsGridRule {
        grid_template: grid_template.clone(),
        detection_rule: grid_detection_rule_for_grid_screen(grid),
        scroll_rule: scroll_rule_for_grid_template(&grid_template),
        uses_artifact_set_filter_screen: *grid == GridScreenName::ArtifactSetFilter,
        on_after_turn_draws_items: *grid != GridScreenName::ArtifactSetFilter,
        on_before_scroll_clears_overlay: true,
    }
}

fn grid_detection_rule_for_grid_screen(grid: &GridScreenName) -> GridItemDetectionRule {
    let (close_kernel_width, close_kernel_height, shape_ratio_target, shape_ratio_tolerance) =
        if *grid == GridScreenName::ArtifactSetFilter {
            (3, 3, 8.63, 0.4)
        } else {
            (5, 5, 0.81, 0.03)
        };
    GridItemDetectionRule {
        min_width_per_column_ratio: 0.66,
        shape_ratio_target,
        shape_ratio_tolerance,
        top_right_exclusion_x_ratio: 0.60,
        top_right_exclusion_y_ratio: 0.28,
        canny_low_threshold: 20.0,
        canny_high_threshold: 40.0,
        close_kernel_width,
        close_kernel_height,
        fill_missing_threshold_roi_height_ratio: 0.025,
        phantom_cell_bgr: crate::common_job::GridBgrColor {
            b: 0xdc,
            g: 0xe5,
            r: 0xe9,
        },
        phantom_cell_tolerance: 30,
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

fn capture_rule(star_as_suffix: bool) -> GetGridIconsCaptureRule {
    GetGridIconsCaptureRule {
        item_click_delay_ms: 300,
        item_name_ocr_roi: GetGridIconsWidthRelativeRect {
            x_from_capture_width: 0.682,
            y_from_capture_width: 0.0625,
            width_from_capture_width: 0.256,
            height_from_capture_width: 0.03125,
        },
        item_name_ocr_engine: "Paddle".to_string(),
        star_suffix_rule: star_as_suffix.then(|| GetGridIconsStarSuffixRule {
            star_ocr_roi: GetGridIconsWidthRelativeRect {
                x_from_capture_width: 0.682,
                y_from_capture_width: 0.1823,
                width_from_capture_width: 0.105,
                height_from_capture_width: 0.02345,
            },
            yellow_bgr_lower: GetGridIconsBgrColor {
                b: 45,
                g: 199,
                r: 250,
            },
            yellow_bgr_upper: GetGridIconsBgrColor {
                b: 55,
                g: 209,
                r: 255,
            },
            contour_count_maps_to_star_glyphs: true,
            glyph: "★".to_string(),
        }),
        lv_suffix_config_is_ignored_by_legacy_task: true,
    }
}

fn artifact_set_filter_rule() -> GetGridIconsArtifactSetFilterRule {
    GetGridIconsArtifactSetFilterRule {
        grid_template: artifact_set_filter_grid_template(),
        item_shape_ratio_target: 8.63,
        item_shape_ratio_tolerance: 0.4,
        close_kernel_width: 3,
        close_kernel_height: 3,
        flower_name_roi: GetGridIconsWidthRelativeRect {
            x_from_capture_width: 0.714,
            y_from_capture_width: 0.284,
            width_from_capture_width: 0.256,
            height_from_capture_width: 0.208,
        },
        flower_without_glyph_roi: GetGridIconsFlowerWithoutGlyphRule {
            x_from_name_region_width: 0.028,
            y_from_detected_line_height: 0.0,
            width_from_capture_width: 0.228,
            height_from_detected_line_height: 1.0,
        },
        anchor_text: "套装包含".to_string(),
        retry_scroll_rounds: 5,
        retry_scroll_delta_per_round: -2,
        retry_scroll_interval_ms: 40,
        retry_wait_ms: 300,
        icon_crop_rule: ArtifactSetFilterIconCropRule {
            source_width: 60,
            source_height: 60,
            normalized_width: 125,
            normalized_height: 125,
            x_formula: "item_width / 2 - 237 * asset_scale - source_width / 2".to_string(),
            y_formula: "item_height / 2 - source_height / 2".to_string(),
            clamps_to_item_region: true,
        },
    }
}

fn model_accuracy_rule() -> GetGridIconsModelAccuracyRule {
    GetGridIconsModelAccuracyRule {
        related_task_name: "GridIconsAccuracyTestTask".to_string(),
        model_rule: GridIconClassifierRule {
            model_name: GRID_ICON_MODEL_NAME.to_string(),
            model_path: GRID_ICON_MODEL_PATH.to_string(),
            prototype_csv_path: GRID_ICON_PROTOTYPE_CSV_PATH.to_string(),
            input_name: GRID_ICON_INPUT_NAME.to_string(),
            feature_dimensions: 64,
            max_distance_squared: 100.0,
        },
        loads_prefix_list_metadata: true,
        prototype_csv_skips_header: true,
        star_output_uses_argmax_of_third_output: true,
        main_collection_does_not_require_model: true,
    }
}

fn get_grid_icons_steps(grid: &GridScreenName, star_as_suffix: bool) -> Vec<GetGridIconsStep> {
    let mut steps = vec![step(
        GetGridIconsStepPhase::Setup,
        GetGridIconsStepAction::CreateTimestampedOutputDirectory,
    )];
    if grid.inventory_tab_assets().is_some() {
        steps.push(step(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::ReturnMainUi,
        ));
        steps.push(step(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::OpenInventoryTab,
        ));
    } else {
        steps.push(step(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::RequireManualGridOpen,
        ));
    }
    steps.push(step(
        GetGridIconsStepPhase::ScanGrid,
        GetGridIconsStepAction::EnumerateGridItems,
    ));
    steps.push(step(
        GetGridIconsStepPhase::CaptureItem,
        GetGridIconsStepAction::ClickItemAndWait,
    ));
    if *grid == GridScreenName::ArtifactSetFilter {
        steps.push(step(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::OcrArtifactSetFlowerName,
        ));
        steps.push(step(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::CropArtifactSetIcon,
        ));
    } else {
        steps.push(step(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::OcrItemName,
        ));
        if star_as_suffix {
            steps.push(step(
                GetGridIconsStepPhase::CaptureItem,
                GetGridIconsStepAction::CountStarsWhenConfigured,
            ));
        }
    }
    steps.push(step(
        GetGridIconsStepPhase::Save,
        GetGridIconsStepAction::SaveUniquePng,
    ));
    steps.push(step(
        GetGridIconsStepPhase::Save,
        GetGridIconsStepAction::StopWhenMaxCountReached,
    ));
    steps.push(step(
        GetGridIconsStepPhase::Cleanup,
        GetGridIconsStepAction::ClearVisionOverlay,
    ));
    steps
}

fn step(phase: GetGridIconsStepPhase, action: GetGridIconsStepAction) -> GetGridIconsStep {
    GetGridIconsStep { phase, action }
}

fn overlay_get_grid_icons_config(config: &mut GetGridIconsConfig, value: &Value) {
    if let Some(grid_name) = value
        .get("gridName")
        .or_else(|| value.get("GridName"))
        .or_else(|| value.get("grid_name"))
    {
        config.grid_name = grid_name.clone();
    }
    if let Some(value) = bool_member(value, ["starAsSuffix", "StarAsSuffix", "star_as_suffix"]) {
        config.star_as_suffix = value;
    }
    if let Some(value) = bool_member(value, ["lvAsSuffix", "LvAsSuffix", "lv_as_suffix"]) {
        config.lv_as_suffix = value;
    }
    if let Some(value) = u64_member(value, ["maxNumToGet", "MaxNumToGet", "max_num_to_get"]) {
        config.max_num_to_get = value;
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn bool_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<bool> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_bool)
}

fn u64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<u64> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_u64)
}
