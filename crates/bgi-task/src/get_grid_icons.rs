use bgi_core::config::GetGridIconsConfig;
use bgi_vision::{OcrResult, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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
        if !std::ptr::eq(get_grid_icons_value, value) {
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
    pub x_from_capture_width: f64,
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
pub struct GetGridIconsArtifactSetFlowerOcrPlan {
    pub flower_with_glyph_text: String,
    pub flower_with_glyph_rect: Rect,
    pub flower_without_glyph_roi_in_name_region: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsArtifactSetFlowerNameDecision {
    pub flower_name: String,
    pub flower_with_glyph_text: String,
    pub flower_without_glyph_ocr_text: String,
    pub flower_with_glyph_rect: Rect,
    pub flower_without_glyph_roi_in_name_region: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GetGridIconsArtifactSetIconCropPlan {
    pub source_rect_in_item_region: Rect,
    pub normalized_size: Size,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetGridIconsStepPhase {
    Setup,
    OpenGrid,
    ScanGrid,
    CaptureItem,
    Save,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetGridIconsStepAction {
    CreateTimestampedOutputDirectory,
    ReturnMainUi,
    OpenInventoryTab,
    RequireManualGridOpen,
    EnumerateGridItems,
    StopWhenGridScanIncomplete,
    ClickItemAndWait,
    OcrItemName,
    CountStarsWhenConfigured,
    OcrArtifactSetFlowerName,
    CropArtifactSetIcon,
    SaveUniquePng,
    StopWhenMaxCountReached,
    ClearVisionOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsGridItem {
    pub page_index: u32,
    pub item_index: u32,
    pub rect: Rect,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsPngData {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsGridEnumeration {
    pub items: Vec<GetGridIconsGridItem>,
    pub scan_complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetGridIconsRuntimeActionStatus {
    Executed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetGridIconsSkipReason {
    MaxNumToGetZero,
    MaxNumToGetReached,
    GridScanIncomplete,
    MissingOcrItemName,
    DuplicateFileName,
    SaveFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsRuntimeActionReport {
    pub phase: GetGridIconsStepPhase,
    pub action: GetGridIconsStepAction,
    pub status: GetGridIconsRuntimeActionStatus,
    pub item: Option<GetGridIconsGridItem>,
    pub output_file_name: Option<String>,
    pub skip_reason: Option<GetGridIconsSkipReason>,
    pub message: String,
}

impl GetGridIconsRuntimeActionReport {
    fn executed(
        phase: GetGridIconsStepPhase,
        action: GetGridIconsStepAction,
        item: Option<GetGridIconsGridItem>,
        output_file_name: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            action,
            status: GetGridIconsRuntimeActionStatus::Executed,
            item,
            output_file_name,
            skip_reason: None,
            message: message.into(),
        }
    }

    fn skipped(
        phase: GetGridIconsStepPhase,
        action: GetGridIconsStepAction,
        item: Option<GetGridIconsGridItem>,
        output_file_name: Option<String>,
        reason: GetGridIconsSkipReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            action,
            status: GetGridIconsRuntimeActionStatus::Skipped,
            item,
            output_file_name,
            skip_reason: Some(reason),
            message: message.into(),
        }
    }

    fn failed(
        phase: GetGridIconsStepPhase,
        action: GetGridIconsStepAction,
        item: Option<GetGridIconsGridItem>,
        output_file_name: Option<String>,
        reason: GetGridIconsSkipReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            action,
            status: GetGridIconsRuntimeActionStatus::Failed,
            item,
            output_file_name,
            skip_reason: Some(reason),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsSavedPng {
    pub item: GetGridIconsGridItem,
    pub item_name: String,
    pub star_suffix: String,
    pub file_name: String,
    pub output_path: PathBuf,
    pub png_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsSkippedPng {
    pub item: Option<GetGridIconsGridItem>,
    pub reason: GetGridIconsSkipReason,
    pub item_name: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsExecutorState {
    pub output_dir: Option<PathBuf>,
    pub auto_open_inventory_completed: Option<bool>,
    pub manual_open_completed: Option<bool>,
    pub grid_items: Vec<GetGridIconsGridItem>,
    pub grid_scan_complete: bool,
    pub clicked_items: u64,
    pub saved_icons: Vec<GetGridIconsSavedPng>,
    pub skipped_icons: Vec<GetGridIconsSkippedPng>,
    pub max_num_reached: bool,
    pub cleanup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetGridIconsExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: GetGridIconsExecutorState,
    pub action_reports: Vec<GetGridIconsRuntimeActionReport>,
}

pub trait GetGridIconsRuntime {
    fn create_get_grid_icons_output_dir(
        &mut self,
        output_rule: &GetGridIconsOutputRule,
        grid_screen_name: &GridScreenName,
    ) -> Result<PathBuf>;

    fn return_get_grid_icons_main_ui(&mut self, grid_screen_name: &GridScreenName) -> Result<()>;

    fn open_get_grid_icons_inventory_tab(
        &mut self,
        grid_screen_name: &GridScreenName,
        inventory_tab_assets: &InventoryTabAssetPair,
    ) -> Result<()>;

    fn require_get_grid_icons_manual_open(
        &mut self,
        grid_screen_name: &GridScreenName,
        message: &str,
    ) -> Result<()>;

    fn enumerate_get_grid_icons_grid_items(
        &mut self,
        grid_rule: &GetGridIconsGridRule,
        artifact_set_filter_rule: Option<&GetGridIconsArtifactSetFilterRule>,
    ) -> Result<GetGridIconsGridEnumeration>;

    fn click_get_grid_icons_item(
        &mut self,
        item: &GetGridIconsGridItem,
        wait_after_click_ms: u64,
    ) -> Result<()>;

    fn ocr_get_grid_icons_item_name(
        &mut self,
        item: &GetGridIconsGridItem,
        roi: &GetGridIconsWidthRelativeRect,
    ) -> Result<Option<String>>;

    fn count_get_grid_icons_star_suffix(
        &mut self,
        item: &GetGridIconsGridItem,
        rule: &GetGridIconsStarSuffixRule,
    ) -> Result<u8>;

    fn capture_get_grid_icons_item_icon_png(
        &mut self,
        item: &GetGridIconsGridItem,
    ) -> Result<GetGridIconsPngData>;

    fn ocr_get_grid_icons_artifact_set_flower_name(
        &mut self,
        item: &GetGridIconsGridItem,
        rule: &GetGridIconsArtifactSetFilterRule,
    ) -> Result<Option<String>>;

    fn crop_get_grid_icons_artifact_set_icon_png(
        &mut self,
        item: &GetGridIconsGridItem,
        rule: &GetGridIconsArtifactSetFilterRule,
    ) -> Result<GetGridIconsPngData>;

    fn save_get_grid_icons_png(
        &mut self,
        output_dir: &Path,
        file_name: &str,
        png: &GetGridIconsPngData,
    ) -> Result<()>;

    fn clear_get_grid_icons_vision_overlay(&mut self) -> Result<()>;
}

pub fn execute_get_grid_icons_plan<R>(
    plan: &GetGridIconsExecutionPlan,
    runtime: &mut R,
) -> Result<GetGridIconsExecutionReport>
where
    R: GetGridIconsRuntime,
{
    let mut state = GetGridIconsExecutorState::default();
    let mut action_reports = Vec::new();

    let execution_result =
        execute_get_grid_icons_plan_inner(plan, runtime, &mut state, &mut action_reports);
    let cleanup_result = execute_get_grid_icons_cleanup(runtime, &mut state, &mut action_reports);

    match (execution_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(GetGridIconsExecutionReport {
            task_key: plan.task_key.clone(),
            completed: state.cleanup_completed,
            state,
            action_reports,
        }),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) | (Err(_), Err(error)) => Err(error),
    }
}

fn execute_get_grid_icons_plan_inner<R>(
    plan: &GetGridIconsExecutionPlan,
    runtime: &mut R,
    state: &mut GetGridIconsExecutorState,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
) -> Result<()>
where
    R: GetGridIconsRuntime,
{
    let output_dir = runtime
        .create_get_grid_icons_output_dir(&plan.output_rule, &plan.config_rule.grid_screen_name)?;
    state.output_dir = Some(output_dir.clone());
    action_reports.push(GetGridIconsRuntimeActionReport::executed(
        GetGridIconsStepPhase::Setup,
        GetGridIconsStepAction::CreateTimestampedOutputDirectory,
        None,
        None,
        output_dir.display().to_string(),
    ));

    if plan.config_rule.max_num_to_get == 0 {
        state.max_num_reached = true;
        state.skipped_icons.push(GetGridIconsSkippedPng {
            item: None,
            reason: GetGridIconsSkipReason::MaxNumToGetZero,
            item_name: None,
            file_name: None,
        });
        action_reports.push(GetGridIconsRuntimeActionReport::skipped(
            GetGridIconsStepPhase::ScanGrid,
            GetGridIconsStepAction::EnumerateGridItems,
            None,
            None,
            GetGridIconsSkipReason::MaxNumToGetZero,
            "maxNumToGet is 0; grid scan skipped",
        ));
        return Ok(());
    }

    execute_get_grid_icons_open_grid(plan, runtime, state, action_reports)?;

    let enumeration = runtime.enumerate_get_grid_icons_grid_items(
        &plan.grid_rule,
        plan.artifact_set_filter_rule.as_ref(),
    )?;
    state.grid_items = enumeration.items;
    state.grid_scan_complete = enumeration.scan_complete;
    action_reports.push(GetGridIconsRuntimeActionReport::executed(
        GetGridIconsStepPhase::ScanGrid,
        GetGridIconsStepAction::EnumerateGridItems,
        None,
        None,
        format!(
            "{} grid item(s); scan_complete={}",
            state.grid_items.len(),
            state.grid_scan_complete
        ),
    ));

    if !state.grid_scan_complete
        && (state.grid_items.len() as u64) < plan.config_rule.max_num_to_get
    {
        return fail_get_grid_icons_incomplete_scan(state, action_reports);
    }

    let mut existing_file_names = HashSet::new();
    for item in state.grid_items.clone() {
        if state.clicked_items >= plan.config_rule.max_num_to_get {
            state.max_num_reached = true;
            state.skipped_icons.push(GetGridIconsSkippedPng {
                item: Some(item),
                reason: GetGridIconsSkipReason::MaxNumToGetReached,
                item_name: None,
                file_name: None,
            });
            action_reports.push(GetGridIconsRuntimeActionReport::skipped(
                GetGridIconsStepPhase::Save,
                GetGridIconsStepAction::StopWhenMaxCountReached,
                Some(item),
                None,
                GetGridIconsSkipReason::MaxNumToGetReached,
                format!("maxNumToGet {} reached", plan.config_rule.max_num_to_get),
            ));
            break;
        }

        execute_get_grid_icons_item(
            plan,
            runtime,
            state,
            action_reports,
            &output_dir,
            &mut existing_file_names,
            item,
        )?;
    }

    if state.clicked_items >= plan.config_rule.max_num_to_get {
        state.max_num_reached = true;
    }

    if !state.grid_scan_complete && !state.max_num_reached {
        return fail_get_grid_icons_incomplete_scan(state, action_reports);
    }

    Ok(())
}

fn fail_get_grid_icons_incomplete_scan(
    state: &mut GetGridIconsExecutorState,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
) -> Result<()> {
    state.skipped_icons.push(GetGridIconsSkippedPng {
        item: None,
        reason: GetGridIconsSkipReason::GridScanIncomplete,
        item_name: None,
        file_name: None,
    });
    let message =
        "GetGridIcons grid enumeration did not complete; full desktop GridScroller adapter remains pending after visible-page enumeration";
    action_reports.push(GetGridIconsRuntimeActionReport::failed(
        GetGridIconsStepPhase::ScanGrid,
        GetGridIconsStepAction::StopWhenGridScanIncomplete,
        None,
        None,
        GetGridIconsSkipReason::GridScanIncomplete,
        message,
    ));
    Err(TaskError::CommonJobExecution(message.to_string()))
}

fn execute_get_grid_icons_open_grid<R>(
    plan: &GetGridIconsExecutionPlan,
    runtime: &mut R,
    state: &mut GetGridIconsExecutorState,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
) -> Result<()>
where
    R: GetGridIconsRuntime,
{
    if plan.open_rule.auto_open_inventory {
        runtime.return_get_grid_icons_main_ui(&plan.config_rule.grid_screen_name)?;
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::ReturnMainUi,
            None,
            None,
            "returned to main UI before opening inventory",
        ));

        let inventory_tab_assets =
            plan.open_rule
                .inventory_tab_assets
                .as_ref()
                .ok_or_else(|| TaskError::InvalidTaskConfig {
                    key: GET_GRID_ICONS_TASK_KEY.to_string(),
                    message: "auto-open inventory grid has no inventory tab assets".to_string(),
                })?;
        runtime.open_get_grid_icons_inventory_tab(
            &plan.config_rule.grid_screen_name,
            inventory_tab_assets,
        )?;
        state.auto_open_inventory_completed = Some(true);
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::OpenInventoryTab,
            None,
            None,
            inventory_tab_assets.unchecked_asset.clone(),
        ));
    } else {
        runtime.require_get_grid_icons_manual_open(
            &plan.config_rule.grid_screen_name,
            &plan.open_rule.manual_open_message,
        )?;
        state.manual_open_completed = Some(true);
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::OpenGrid,
            GetGridIconsStepAction::RequireManualGridOpen,
            None,
            None,
            plan.open_rule.manual_open_message.clone(),
        ));
    }

    Ok(())
}

fn execute_get_grid_icons_item<R>(
    plan: &GetGridIconsExecutionPlan,
    runtime: &mut R,
    state: &mut GetGridIconsExecutorState,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
    output_dir: &Path,
    existing_file_names: &mut HashSet<String>,
    item: GetGridIconsGridItem,
) -> Result<()>
where
    R: GetGridIconsRuntime,
{
    runtime.click_get_grid_icons_item(&item, plan.capture_rule.item_click_delay_ms)?;
    state.clicked_items += 1;
    action_reports.push(GetGridIconsRuntimeActionReport::executed(
        GetGridIconsStepPhase::CaptureItem,
        GetGridIconsStepAction::ClickItemAndWait,
        Some(item),
        None,
        format!("waited {} ms", plan.capture_rule.item_click_delay_ms),
    ));

    let (item_name, star_suffix, png) =
        capture_get_grid_icons_item_payload(plan, runtime, action_reports, item)?;

    let Some(item_name) = item_name else {
        state.skipped_icons.push(GetGridIconsSkippedPng {
            item: Some(item),
            reason: GetGridIconsSkipReason::MissingOcrItemName,
            item_name: None,
            file_name: None,
        });
        action_reports.push(GetGridIconsRuntimeActionReport::skipped(
            GetGridIconsStepPhase::Save,
            GetGridIconsStepAction::SaveUniquePng,
            Some(item),
            None,
            GetGridIconsSkipReason::MissingOcrItemName,
            "item name OCR returned no text",
        ));
        return Ok(());
    };

    let file_name = format!("{item_name}{star_suffix}.png");
    if !existing_file_names.insert(file_name.clone()) {
        state.skipped_icons.push(GetGridIconsSkippedPng {
            item: Some(item),
            reason: GetGridIconsSkipReason::DuplicateFileName,
            item_name: Some(item_name),
            file_name: Some(file_name.clone()),
        });
        action_reports.push(GetGridIconsRuntimeActionReport::skipped(
            GetGridIconsStepPhase::Save,
            GetGridIconsStepAction::SaveUniquePng,
            Some(item),
            Some(file_name),
            GetGridIconsSkipReason::DuplicateFileName,
            "duplicate file name skipped",
        ));
        return Ok(());
    }

    match runtime.save_get_grid_icons_png(output_dir, &file_name, &png) {
        Ok(()) => {
            let output_path = output_dir.join(&file_name);
            state.saved_icons.push(GetGridIconsSavedPng {
                item,
                item_name,
                star_suffix,
                file_name: file_name.clone(),
                output_path,
                png_len: png.bytes.len(),
            });
            action_reports.push(GetGridIconsRuntimeActionReport::executed(
                GetGridIconsStepPhase::Save,
                GetGridIconsStepAction::SaveUniquePng,
                Some(item),
                Some(file_name),
                format!("{} byte(s)", png.bytes.len()),
            ));
        }
        Err(error) if plan.output_rule.save_failures_are_logged_not_fatal => {
            state.skipped_icons.push(GetGridIconsSkippedPng {
                item: Some(item),
                reason: GetGridIconsSkipReason::SaveFailed,
                item_name: Some(item_name),
                file_name: Some(file_name.clone()),
            });
            action_reports.push(GetGridIconsRuntimeActionReport::failed(
                GetGridIconsStepPhase::Save,
                GetGridIconsStepAction::SaveUniquePng,
                Some(item),
                Some(file_name),
                GetGridIconsSkipReason::SaveFailed,
                error.to_string(),
            ));
        }
        Err(error) => return Err(error),
    }

    Ok(())
}

fn capture_get_grid_icons_item_payload<R>(
    plan: &GetGridIconsExecutionPlan,
    runtime: &mut R,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
    item: GetGridIconsGridItem,
) -> Result<(Option<String>, String, GetGridIconsPngData)>
where
    R: GetGridIconsRuntime,
{
    if let Some(artifact_set_filter_rule) = plan.artifact_set_filter_rule.as_ref() {
        let item_name =
            runtime.ocr_get_grid_icons_artifact_set_flower_name(&item, artifact_set_filter_rule)?;
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::OcrArtifactSetFlowerName,
            Some(item),
            None,
            item_name.clone().unwrap_or_default(),
        ));

        let png =
            runtime.crop_get_grid_icons_artifact_set_icon_png(&item, artifact_set_filter_rule)?;
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::CropArtifactSetIcon,
            Some(item),
            None,
            format!("{} byte(s)", png.bytes.len()),
        ));
        return Ok((item_name, String::new(), png));
    }

    let item_name =
        runtime.ocr_get_grid_icons_item_name(&item, &plan.capture_rule.item_name_ocr_roi)?;
    action_reports.push(GetGridIconsRuntimeActionReport::executed(
        GetGridIconsStepPhase::CaptureItem,
        GetGridIconsStepAction::OcrItemName,
        Some(item),
        None,
        item_name.clone().unwrap_or_default(),
    ));

    let star_suffix = if let Some(star_suffix_rule) = plan.capture_rule.star_suffix_rule.as_ref() {
        let star_count = runtime.count_get_grid_icons_star_suffix(&item, star_suffix_rule)?;
        let suffix = star_suffix_rule.glyph.repeat(star_count as usize);
        action_reports.push(GetGridIconsRuntimeActionReport::executed(
            GetGridIconsStepPhase::CaptureItem,
            GetGridIconsStepAction::CountStarsWhenConfigured,
            Some(item),
            None,
            suffix.clone(),
        ));
        suffix
    } else {
        String::new()
    };

    let png = runtime.capture_get_grid_icons_item_icon_png(&item)?;
    Ok((item_name, star_suffix, png))
}

fn execute_get_grid_icons_cleanup<R>(
    runtime: &mut R,
    state: &mut GetGridIconsExecutorState,
    action_reports: &mut Vec<GetGridIconsRuntimeActionReport>,
) -> Result<()>
where
    R: GetGridIconsRuntime,
{
    runtime.clear_get_grid_icons_vision_overlay()?;
    state.cleanup_completed = true;
    action_reports.push(GetGridIconsRuntimeActionReport::executed(
        GetGridIconsStepPhase::Cleanup,
        GetGridIconsStepAction::ClearVisionOverlay,
        None,
        None,
        "vision overlay cleared",
    ));
    Ok(())
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
        executor_ready: true,
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
            "desktop live adapters are not wired yet for TaskRunner/ISoloTask cancellation lifecycle".to_string(),
            "desktop live adapters are partially wired for ordinary inventory ReturnMainUi/OpenInventory/tab handling and visible-page cell clicks; manual-open prompts and special-grid input dispatch remain pending".to_string(),
            "desktop live adapters are partially wired for ordinary inventory current-visible-page enumeration/crop with scan_complete=false contract protection; full GridScroller page scrolling, OpenCV ArtifactSetFilterScreen contour enumeration, first-page de-highlight, anti-recycling clicks, and phase-correlation parity remain pending".to_string(),
            "desktop live adapters are not wired yet for Paddle OCR item names, artifact-set flower names, optional star contour suffix detection, live cropped-icon PNG encoding, and overlay cleanup".to_string(),
            "optional GridIconsAccuracyTestTask ONNX/prototype inference live adapter remains pending".to_string(),
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
            x_from_capture_width: 0.028,
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

pub fn plan_get_grid_icons_artifact_set_flower_without_glyph_ocr(
    rule: &GetGridIconsArtifactSetFilterRule,
    capture_size: Size,
    name_region_ocr: &OcrResult,
) -> Option<GetGridIconsArtifactSetFlowerOcrPlan> {
    let mut regions = name_region_ocr.regions.iter().collect::<Vec<_>>();
    regions.sort_by_key(|region| region.rect.center().y);
    let flower_with_glyph = regions
        .into_iter()
        .skip_while(|region| !region.text.contains(&rule.anchor_text))
        .nth(1)?;

    let roi_rule = &rule.flower_without_glyph_roi;
    let capture_width = capture_size.width as f64;
    let line_rect = flower_with_glyph.rect;
    let flower_without_glyph_roi_in_name_region = Rect::new(
        (capture_width * roi_rule.x_from_capture_width) as i32,
        (line_rect.y as f64 - line_rect.height as f64 * roi_rule.y_from_detected_line_height)
            as i32,
        (capture_width * roi_rule.width_from_capture_width) as i32,
        (line_rect.height as f64 * roi_rule.height_from_detected_line_height) as i32,
    )
    .ok()?;

    Some(GetGridIconsArtifactSetFlowerOcrPlan {
        flower_with_glyph_text: flower_with_glyph.text.clone(),
        flower_with_glyph_rect: line_rect,
        flower_without_glyph_roi_in_name_region,
    })
}

pub fn resolve_get_grid_icons_artifact_set_flower_name(
    flower_with_glyph_text: &str,
    flower_without_glyph_ocr_text: &str,
) -> Option<String> {
    let source = flower_with_glyph_text.trim();
    let recognized_without_glyph = flower_without_glyph_ocr_text.trim();
    let source_len = source.chars().count();
    let suffix_len = recognized_without_glyph.chars().count();
    if suffix_len > source_len {
        return None;
    }

    Some(source.chars().skip(source_len - suffix_len).collect())
}

pub fn decide_get_grid_icons_artifact_set_flower_name(
    rule: &GetGridIconsArtifactSetFilterRule,
    capture_size: Size,
    name_region_ocr: &OcrResult,
    flower_without_glyph_ocr_text: &str,
) -> Option<GetGridIconsArtifactSetFlowerNameDecision> {
    let ocr_plan = plan_get_grid_icons_artifact_set_flower_without_glyph_ocr(
        rule,
        capture_size,
        name_region_ocr,
    )?;
    let flower_name = resolve_get_grid_icons_artifact_set_flower_name(
        &ocr_plan.flower_with_glyph_text,
        flower_without_glyph_ocr_text,
    )?;

    Some(GetGridIconsArtifactSetFlowerNameDecision {
        flower_name,
        flower_with_glyph_text: ocr_plan.flower_with_glyph_text,
        flower_without_glyph_ocr_text: flower_without_glyph_ocr_text.to_string(),
        flower_with_glyph_rect: ocr_plan.flower_with_glyph_rect,
        flower_without_glyph_roi_in_name_region: ocr_plan.flower_without_glyph_roi_in_name_region,
    })
}

pub fn plan_get_grid_icons_artifact_set_icon_crop(
    rule: &ArtifactSetFilterIconCropRule,
    item_region_size: Size,
    asset_scale: f64,
) -> Option<GetGridIconsArtifactSetIconCropPlan> {
    let source_width = rule.source_width as f64;
    let source_height = rule.source_height as f64;
    let source_rect = Rect::new(
        (item_region_size.width as f64 / 2.0 - 237.0 * asset_scale - source_width / 2.0) as i32,
        (item_region_size.height as f64 / 2.0 - source_height / 2.0) as i32,
        rule.source_width as i32,
        rule.source_height as i32,
    )
    .ok()?
    .clamp_to(item_region_size)
    .ok()?;

    Some(GetGridIconsArtifactSetIconCropPlan {
        source_rect_in_item_region: source_rect,
        normalized_size: Size::new(rule.normalized_width, rule.normalized_height),
    })
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
        GetGridIconsStepPhase::ScanGrid,
        GetGridIconsStepAction::StopWhenGridScanIncomplete,
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

#[cfg(test)]
mod get_grid_icons_runtime_tests {
    use super::*;

    #[test]
    fn get_grid_icons_executor_skips_scan_when_max_is_zero() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "Weapons",
                "maxNumToGet": 0
            }),
        )))
        .unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![grid_item(0)]);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(runtime.create_output_dir_calls, 1);
        assert_eq!(runtime.return_main_ui_calls, 0);
        assert_eq!(runtime.open_inventory_tab_calls, 0);
        assert_eq!(runtime.enumerate_calls, 0);
        assert_eq!(runtime.save_calls, 0);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(report.state.max_num_reached);
        assert!(report.state.grid_items.is_empty());
        assert_eq!(
            report.state.skipped_icons[0].reason,
            GetGridIconsSkipReason::MaxNumToGetZero
        );
        assert!(report.action_reports.iter().any(|report| {
            report.action == GetGridIconsStepAction::EnumerateGridItems
                && report.status == GetGridIconsRuntimeActionStatus::Skipped
        }));
    }

    #[test]
    fn get_grid_icons_executor_scans_after_manual_open_branch() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "ArtifactSalvage",
                "maxNumToGet": 2
            }),
        )))
        .unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![grid_item(0)])
            .with_names(vec![Some("ManualItem".to_string())]);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(runtime.manual_open_calls, 1);
        assert_eq!(runtime.return_main_ui_calls, 0);
        assert_eq!(runtime.open_inventory_tab_calls, 0);
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.click_calls, 1);
        assert_eq!(runtime.save_calls, 1);
        assert_eq!(runtime.cleanup_calls, 1);
        assert_eq!(report.state.manual_open_completed, Some(true));
        assert_eq!(report.state.saved_icons.len(), 1);
        assert_eq!(report.state.saved_icons[0].file_name, "ManualItem.png");
    }

    #[test]
    fn get_grid_icons_executor_auto_opens_inventory_saves_unique_pngs_and_truncates_at_max() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "Weapons",
                "starAsSuffix": true,
                "maxNumToGet": 2
            }),
        )))
        .unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![
            grid_item(0),
            grid_item(1),
            grid_item(2),
            grid_item(3),
        ])
        .with_names(vec![
            Some("Sword".to_string()),
            Some("Sword".to_string()),
            Some("Bow".to_string()),
            Some("Polearm".to_string()),
        ])
        .with_star_counts(vec![1, 1, 0, 0]);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(runtime.return_main_ui_calls, 1);
        assert_eq!(runtime.open_inventory_tab_calls, 1);
        assert_eq!(runtime.manual_open_calls, 0);
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.click_calls, 2);
        assert_eq!(runtime.star_suffix_calls, 2);
        assert_eq!(runtime.save_calls, 1);
        assert_eq!(runtime.cleanup_calls, 1);
        assert_eq!(runtime.saved_file_names, vec!["Sword★.png".to_string()]);
        assert_eq!(report.state.clicked_items, 2);
        assert_eq!(report.state.saved_icons.len(), 1);
        assert_eq!(report.state.saved_icons[0].star_suffix, "★");
        assert!(report.state.max_num_reached);
        assert!(report.state.skipped_icons.iter().any(|skipped| {
            skipped.reason == GetGridIconsSkipReason::DuplicateFileName
                && skipped.file_name.as_deref() == Some("Sword★.png")
        }));
        assert!(report.state.skipped_icons.iter().any(|skipped| {
            skipped.reason == GetGridIconsSkipReason::MaxNumToGetReached
                && skipped.item == Some(grid_item(2))
        }));
    }

    #[test]
    fn get_grid_icons_executor_uses_artifact_set_filter_capture_branch() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "ArtifactSetFilter",
                "maxNumToGet": 1
            }),
        )))
        .unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![grid_item(0)])
            .with_names(vec![Some("昔日宗室之花".to_string())]);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(runtime.manual_open_calls, 1);
        assert_eq!(runtime.return_main_ui_calls, 0);
        assert_eq!(runtime.open_inventory_tab_calls, 0);
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.click_calls, 1);
        assert_eq!(runtime.item_name_ocr_calls, 0);
        assert_eq!(runtime.star_suffix_calls, 0);
        assert_eq!(runtime.capture_icon_calls, 0);
        assert_eq!(runtime.artifact_flower_ocr_calls, 1);
        assert_eq!(runtime.artifact_icon_crop_calls, 1);
        assert_eq!(runtime.save_calls, 1);
        assert_eq!(runtime.cleanup_calls, 1);
        assert_eq!(runtime.saved_file_names, vec!["昔日宗室之花.png"]);
        assert_eq!(report.state.saved_icons.len(), 1);
        assert_eq!(report.state.saved_icons[0].file_name, "昔日宗室之花.png");
        assert!(report.action_reports.iter().any(|report| {
            report.action == GetGridIconsStepAction::OcrArtifactSetFlowerName
                && report.status == GetGridIconsRuntimeActionStatus::Executed
        }));
        assert!(report.action_reports.iter().any(|report| {
            report.action == GetGridIconsStepAction::CropArtifactSetIcon
                && report.status == GetGridIconsRuntimeActionStatus::Executed
        }));
    }

    #[test]
    fn get_grid_icons_executor_runs_cleanup_when_scan_fails() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::default()).unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![grid_item(0)]);
        runtime.fail_enumerate = true;

        let error = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap_err();

        assert!(
            matches!(error, TaskError::CommonJobExecution(message) if message == "enumerate failed")
        );
        assert_eq!(runtime.create_output_dir_calls, 1);
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.cleanup_calls, 1);
    }

    #[test]
    fn get_grid_icons_executor_rejects_incomplete_grid_scan_without_max_reached() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "Weapons",
                "maxNumToGet": 2
            }),
        )))
        .unwrap();
        let mut runtime =
            FakeGetGridIconsRuntime::with_items(vec![grid_item(0)]).with_grid_scan_complete(false);

        let error = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("grid enumeration did not complete")
                    && message.contains("GridScroller adapter remains pending")
        ));
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.click_calls, 0);
        assert_eq!(runtime.save_calls, 0);
        assert_eq!(runtime.cleanup_calls, 1);
    }

    #[test]
    fn get_grid_icons_executor_allows_incomplete_grid_scan_when_max_reached() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "Weapons",
                "maxNumToGet": 1
            }),
        )))
        .unwrap();
        let mut runtime = FakeGetGridIconsRuntime::with_items(vec![grid_item(0), grid_item(1)])
            .with_grid_scan_complete(false);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert!(!report.state.grid_scan_complete);
        assert!(report.state.max_num_reached);
        assert_eq!(runtime.enumerate_calls, 1);
        assert_eq!(runtime.click_calls, 1);
        assert_eq!(runtime.save_calls, 1);
        assert_eq!(runtime.cleanup_calls, 1);
        assert!(report.action_reports.iter().any(|report| {
            report.action == GetGridIconsStepAction::StopWhenMaxCountReached
                && report.status == GetGridIconsRuntimeActionStatus::Skipped
        }));
    }

    #[test]
    fn get_grid_icons_executor_marks_exact_incomplete_visible_count_as_max_reached() {
        let plan = plan_get_grid_icons(GetGridIconsExecutionConfig::from_value(Some(
            &serde_json::json!({
                "gridName": "Weapons",
                "maxNumToGet": 1
            }),
        )))
        .unwrap();
        let mut runtime =
            FakeGetGridIconsRuntime::with_items(vec![grid_item(0)]).with_grid_scan_complete(false);

        let report = execute_get_grid_icons_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert!(!report.state.grid_scan_complete);
        assert!(report.state.max_num_reached);
        assert_eq!(runtime.click_calls, 1);
        assert_eq!(runtime.save_calls, 1);
        assert!(!report
            .action_reports
            .iter()
            .any(|report| { report.action == GetGridIconsStepAction::StopWhenMaxCountReached }));
    }

    #[derive(Debug, Default)]
    struct FakeGetGridIconsRuntime {
        output_dir: PathBuf,
        grid_items: Vec<GetGridIconsGridItem>,
        grid_scan_complete: bool,
        names: Vec<Option<String>>,
        star_counts: Vec<u8>,
        saved_file_names: Vec<String>,
        create_output_dir_calls: u32,
        return_main_ui_calls: u32,
        open_inventory_tab_calls: u32,
        manual_open_calls: u32,
        enumerate_calls: u32,
        click_calls: u32,
        item_name_ocr_calls: u32,
        star_suffix_calls: u32,
        capture_icon_calls: u32,
        artifact_flower_ocr_calls: u32,
        artifact_icon_crop_calls: u32,
        save_calls: u32,
        cleanup_calls: u32,
        fail_enumerate: bool,
    }

    impl FakeGetGridIconsRuntime {
        fn with_items(grid_items: Vec<GetGridIconsGridItem>) -> Self {
            Self {
                output_dir: PathBuf::from("target/get-grid-icons-test"),
                grid_items,
                grid_scan_complete: true,
                ..Self::default()
            }
        }

        fn with_grid_scan_complete(mut self, scan_complete: bool) -> Self {
            self.grid_scan_complete = scan_complete;
            self
        }

        fn with_names(mut self, names: Vec<Option<String>>) -> Self {
            self.names = names;
            self
        }

        fn with_star_counts(mut self, star_counts: Vec<u8>) -> Self {
            self.star_counts = star_counts;
            self
        }

        fn name_for(&self, item: &GetGridIconsGridItem) -> Option<String> {
            self.names
                .get(item.item_index as usize)
                .cloned()
                .unwrap_or_else(|| Some(format!("Item{}", item.item_index)))
        }

        fn star_count_for(&self, item: &GetGridIconsGridItem) -> u8 {
            self.star_counts
                .get(item.item_index as usize)
                .copied()
                .unwrap_or_default()
        }
    }

    impl GetGridIconsRuntime for FakeGetGridIconsRuntime {
        fn create_get_grid_icons_output_dir(
            &mut self,
            _output_rule: &GetGridIconsOutputRule,
            _grid_screen_name: &GridScreenName,
        ) -> Result<PathBuf> {
            self.create_output_dir_calls += 1;
            Ok(self.output_dir.clone())
        }

        fn return_get_grid_icons_main_ui(
            &mut self,
            _grid_screen_name: &GridScreenName,
        ) -> Result<()> {
            self.return_main_ui_calls += 1;
            Ok(())
        }

        fn open_get_grid_icons_inventory_tab(
            &mut self,
            _grid_screen_name: &GridScreenName,
            _inventory_tab_assets: &InventoryTabAssetPair,
        ) -> Result<()> {
            self.open_inventory_tab_calls += 1;
            Ok(())
        }

        fn require_get_grid_icons_manual_open(
            &mut self,
            _grid_screen_name: &GridScreenName,
            _message: &str,
        ) -> Result<()> {
            self.manual_open_calls += 1;
            Ok(())
        }

        fn enumerate_get_grid_icons_grid_items(
            &mut self,
            _grid_rule: &GetGridIconsGridRule,
            _artifact_set_filter_rule: Option<&GetGridIconsArtifactSetFilterRule>,
        ) -> Result<GetGridIconsGridEnumeration> {
            self.enumerate_calls += 1;
            if self.fail_enumerate {
                return Err(TaskError::CommonJobExecution(
                    "enumerate failed".to_string(),
                ));
            }
            Ok(GetGridIconsGridEnumeration {
                items: self.grid_items.clone(),
                scan_complete: self.grid_scan_complete,
            })
        }

        fn click_get_grid_icons_item(
            &mut self,
            _item: &GetGridIconsGridItem,
            _wait_after_click_ms: u64,
        ) -> Result<()> {
            self.click_calls += 1;
            Ok(())
        }

        fn ocr_get_grid_icons_item_name(
            &mut self,
            item: &GetGridIconsGridItem,
            _roi: &GetGridIconsWidthRelativeRect,
        ) -> Result<Option<String>> {
            self.item_name_ocr_calls += 1;
            Ok(self.name_for(item))
        }

        fn count_get_grid_icons_star_suffix(
            &mut self,
            item: &GetGridIconsGridItem,
            _rule: &GetGridIconsStarSuffixRule,
        ) -> Result<u8> {
            self.star_suffix_calls += 1;
            Ok(self.star_count_for(item))
        }

        fn capture_get_grid_icons_item_icon_png(
            &mut self,
            item: &GetGridIconsGridItem,
        ) -> Result<GetGridIconsPngData> {
            self.capture_icon_calls += 1;
            Ok(GetGridIconsPngData {
                bytes: vec![item.item_index as u8],
            })
        }

        fn ocr_get_grid_icons_artifact_set_flower_name(
            &mut self,
            item: &GetGridIconsGridItem,
            _rule: &GetGridIconsArtifactSetFilterRule,
        ) -> Result<Option<String>> {
            self.artifact_flower_ocr_calls += 1;
            Ok(self.name_for(item))
        }

        fn crop_get_grid_icons_artifact_set_icon_png(
            &mut self,
            item: &GetGridIconsGridItem,
            _rule: &GetGridIconsArtifactSetFilterRule,
        ) -> Result<GetGridIconsPngData> {
            self.artifact_icon_crop_calls += 1;
            Ok(GetGridIconsPngData {
                bytes: vec![item.item_index as u8, 1],
            })
        }

        fn save_get_grid_icons_png(
            &mut self,
            _output_dir: &Path,
            file_name: &str,
            _png: &GetGridIconsPngData,
        ) -> Result<()> {
            self.save_calls += 1;
            self.saved_file_names.push(file_name.to_string());
            Ok(())
        }

        fn clear_get_grid_icons_vision_overlay(&mut self) -> Result<()> {
            self.cleanup_calls += 1;
            Ok(())
        }
    }

    fn grid_item(item_index: u32) -> GetGridIconsGridItem {
        GetGridIconsGridItem {
            page_index: 0,
            item_index,
            rect: Rect {
                x: 10 + item_index as i32,
                y: 20,
                width: 30,
                height: 40,
            },
        }
    }
}
