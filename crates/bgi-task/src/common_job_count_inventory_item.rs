use super::RETURN_MAIN_UI_TASK_KEY;
use crate::{Result, TaskError, TaskPortState};
use bgi_core::GenshinAction;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const COUNT_INVENTORY_ITEM_TASK_KEY: &str = "CountInventoryItem";
pub const GRID_ICON_MODEL_NAME: &str = "GridIcon";
pub const GRID_ICON_MODEL_PATH: &str = "Assets/Model/Item/gridIcon.onnx";
pub const GRID_ICON_PROTOTYPE_CSV_PATH: &str = "Assets/Model/Item/items.csv";
pub const GRID_ICON_INPUT_NAME: &str = "input_image";
pub const COUNT_INVENTORY_SINGLE_NOT_FOUND: i32 = -1;
pub const COUNT_INVENTORY_OCR_FAILED: i32 = -2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryItemExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub grid_screen_name: GridScreenName,
    pub search_mode: CountInventorySearchMode,
    pub open_inventory_rule: CountInventoryOpenInventoryRule,
    pub grid_template: GridTemplate,
    pub grid_item_detection_rule: GridItemDetectionRule,
    pub grid_icon_crop_rule: GridIconCropRule,
    pub classifier_rule: GridIconClassifierRule,
    pub count_ocr_rule: GridItemCountOcrRule,
    pub scroll_rule: GridScrollRule,
    pub weapon_ore_prescroll_rule: WeaponOrePrescrollRule,
    pub result_contract: CountInventoryResultContract,
    pub steps: Vec<CountInventoryItemStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum GridScreenName {
    Weapons,
    Artifacts,
    CharacterDevelopmentItems,
    Food,
    Materials,
    Gadget,
    Quest,
    PreciousItems,
    Furnishings,
    ArtifactSalvage,
    ArtifactSetFilter,
}

impl GridScreenName {
    pub fn inventory_tab_assets(&self) -> Option<InventoryTabAssetPair> {
        let (checked, unchecked) = match self {
            GridScreenName::Weapons => ("bag_weapon_checked.png", "bag_weapon_unchecked.png"),
            GridScreenName::Artifacts => ("bag_artifact_checked.png", "bag_artifact_unchecked.png"),
            GridScreenName::CharacterDevelopmentItems => (
                "bag_characterdevelopmentitem_checked.png",
                "bag_characterdevelopmentitem_unchecked.png",
            ),
            GridScreenName::Food => ("bag_food_checked.png", "bag_food_unchecked.png"),
            GridScreenName::Materials => ("bag_material_checked.png", "bag_material_unchecked.png"),
            GridScreenName::Gadget => ("bag_gadget_checked.png", "bag_gadget_unchecked.png"),
            GridScreenName::Quest => ("bag_quest_checked.png", "bag_quest_unchecked.png"),
            GridScreenName::PreciousItems => (
                "bag_preciousitem_checked.png",
                "bag_preciousitem_unchecked.png",
            ),
            GridScreenName::Furnishings => {
                ("bag_furnishing_checked.png", "bag_furnishing_unchecked.png")
            }
            GridScreenName::ArtifactSalvage | GridScreenName::ArtifactSetFilter => return None,
        };

        Some(InventoryTabAssetPair {
            checked_asset: format!("Common/Element:{checked}"),
            unchecked_asset: format!("Common/Element:{unchecked}"),
            checked_threshold: 0.8,
            unchecked_threshold: 0.87,
            roi_top_ratio: 0.1,
        })
    }

    pub fn grid_template(&self) -> GridTemplate {
        match self {
            GridScreenName::Artifacts => GridTemplate::artifacts(),
            GridScreenName::ArtifactSalvage => GridTemplate::artifact_salvage(),
            _ => GridTemplate::inventory_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CountInventoryItemExecutionConfig {
    pub capture_size: Size,
    #[serde(alias = "grid")]
    #[serde(alias = "gridScreenName")]
    #[serde(alias = "GridScreenName")]
    pub grid_screen_name: Option<GridScreenName>,
    #[serde(alias = "itemName")]
    #[serde(alias = "ItemName")]
    pub item_name: Option<String>,
    #[serde(alias = "itemNames")]
    #[serde(alias = "ItemNames")]
    pub item_names: Option<Vec<String>>,
}

impl Default for CountInventoryItemExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            grid_screen_name: None,
            item_name: None,
            item_names: None,
        }
    }
}

impl CountInventoryItemExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Result<Self> {
        let config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        validate_count_inventory_config(config)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CountInventorySearchMode {
    Single { item_name: String },
    Multiple { item_names: Vec<String> },
}

impl CountInventorySearchMode {
    pub fn item_names(&self) -> Vec<String> {
        match self {
            CountInventorySearchMode::Single { item_name } => vec![item_name.clone()],
            CountInventorySearchMode::Multiple { item_names } => item_names.clone(),
        }
    }

    pub fn needs_weapon_ore_prescroll(&self, grid_screen_name: &GridScreenName) -> bool {
        *grid_screen_name == GridScreenName::Weapons
            && self
                .item_names()
                .iter()
                .any(|name| name.starts_with("精锻用"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryOpenInventoryRule {
    pub open_inventory_action: GenshinAction,
    pub open_wait_ms: u64,
    pub retry_attempts: u8,
    pub retry_when_main_ui_detected: bool,
    pub expired_item_prompt_confirm_asset: String,
    pub expired_item_prompt_crop_bottom_ratio: f64,
    pub expired_item_prompt_wait_ms: u64,
    pub tab_assets: InventoryTabAssetPair,
    pub after_tab_ready_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryTabAssetPair {
    pub checked_asset: String,
    pub unchecked_asset: String,
    pub checked_threshold: f64,
    pub unchecked_threshold: f64,
    pub roi_top_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridTemplate {
    pub roi_1080p: Rect,
    pub columns: u8,
    pub s1_round: u8,
    pub round_milliseconds: u64,
    pub s2_round: u8,
    pub s3_scale: f64,
}

impl GridTemplate {
    pub fn inventory_default() -> Self {
        Self {
            roi_1080p: Rect {
                x: 106,
                y: 110,
                width: 1171,
                height: 845,
            },
            columns: 8,
            s1_round: 3,
            round_milliseconds: 40,
            s2_round: 32,
            s3_scale: 0.024,
        }
    }

    pub fn artifacts() -> Self {
        Self {
            roi_1080p: Rect {
                x: 106,
                y: 162,
                width: 1171,
                height: 783,
            },
            columns: 8,
            s1_round: 3,
            round_milliseconds: 40,
            s2_round: 32,
            s3_scale: 0.024,
        }
    }

    pub fn artifact_salvage() -> Self {
        Self {
            roi_1080p: Rect {
                x: 48,
                y: 106,
                width: 1267,
                height: 768,
            },
            columns: 9,
            s1_round: 3,
            round_milliseconds: 40,
            s2_round: 28,
            s3_scale: 0.018,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridItemDetectionRule {
    pub min_width_per_column_ratio: f64,
    pub shape_ratio_target: f64,
    pub shape_ratio_tolerance: f64,
    pub top_right_exclusion_x_ratio: f64,
    pub top_right_exclusion_y_ratio: f64,
    pub canny_low_threshold: f64,
    pub canny_high_threshold: f64,
    pub close_kernel_width: i32,
    pub close_kernel_height: i32,
    pub fill_missing_threshold_roi_height_ratio: f64,
    pub phantom_cell_bgr: GridBgrColor,
    pub phantom_cell_tolerance: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridBgrColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridIconCropRule {
    pub normalized_width: u32,
    pub normalized_height: u32,
    pub icon_crop: Rect,
    pub bottom_crop: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridIconClassifierRule {
    pub model_name: String,
    pub model_path: String,
    pub prototype_csv_path: String,
    pub input_name: String,
    pub feature_dimensions: u32,
    pub max_distance_squared: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridItemCountOcrRule {
    pub crop_top_numerator: u32,
    pub crop_bottom_numerator: u32,
    pub crop_left_numerator: u32,
    pub crop_right_numerator: u32,
    pub crop_height_denominator: u32,
    pub crop_width_denominator: u32,
    pub resize_scale: u32,
    pub ocr_engine: String,
    pub convert_full_width_digits: bool,
    pub ocr_failed_value: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridScrollRule {
    pub test_scroll_rounds: u8,
    pub page_scroll_rounds: u8,
    pub scroll_delta_per_round: i32,
    pub fine_scroll_delta: i32,
    pub round_wait_ms: u64,
    pub settle_wait_ms: u64,
    pub fine_scroll_check_interval_ms: u64,
    pub fine_scroll_timeout_ms: u64,
    pub phase_correlation_lower_threshold: f64,
    pub phase_correlation_upper_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponOrePrescrollRule {
    pub enabled: bool,
    pub item_name_prefix: String,
    pub hold_scrollbar_bottom_x_1080p: i32,
    pub hold_scrollbar_bottom_y_1080p: i32,
    pub hold_ms: u64,
    pub after_scroll_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountInventoryResultContract {
    pub single_not_found_value: i32,
    pub ocr_failed_value: i32,
    pub multiple_missing_items_are_omitted: bool,
    pub multiple_duplicate_names_keep_first: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInventoryItemStep {
    pub phase: CountInventoryItemStepPhase,
    pub condition: CountInventoryItemStepCondition,
    pub label: String,
    pub action: CountInventoryItemStepAction,
}

impl CountInventoryItemStep {
    fn new(
        phase: CountInventoryItemStepPhase,
        label: impl Into<String>,
        action: CountInventoryItemStepAction,
    ) -> Self {
        Self {
            phase,
            condition: CountInventoryItemStepCondition::Always,
            label: label.into(),
            action,
        }
    }

    fn conditional(
        phase: CountInventoryItemStepPhase,
        condition: CountInventoryItemStepCondition,
        label: impl Into<String>,
        action: CountInventoryItemStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountInventoryItemStepPhase {
    Setup,
    OpenInventory,
    PreScroll,
    ScanGrid,
    Count,
    Cleanup,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountInventoryItemStepCondition {
    Always,
    WhenExpiredItemPromptDetected,
    WhenInventoryTabUnchecked,
    WhenStillOnMainUi,
    WhenWeaponOreRequested,
    WhenClassifierMatchesTarget,
    WhenAllRequestedItemsFound,
    WhenScanComplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CountInventoryItemStepAction {
    CommonJob {
        task_key: String,
    },
    GenshinAction {
        action: GenshinAction,
    },
    OpenInventoryTab {
        rule: CountInventoryOpenInventoryRule,
    },
    ConfirmExpiredItemPrompt {
        confirm_asset: String,
        crop_bottom_ratio: f64,
    },
    LoadGridIconClassifier {
        rule: GridIconClassifierRule,
    },
    PreScrollWeaponOre {
        rule: WeaponOrePrescrollRule,
    },
    EnumerateGridItems {
        template: GridTemplate,
        detection_rule: GridItemDetectionRule,
        scroll_rule: GridScrollRule,
    },
    CropGridIcon {
        rule: GridIconCropRule,
    },
    InferGridIcon {
        rule: GridIconClassifierRule,
    },
    OcrGridItemCount {
        rule: GridItemCountOcrRule,
    },
    ReturnResult {
        contract: CountInventoryResultContract,
    },
    ClearVisionDrawings,
    Log {
        message: String,
    },
}

pub fn plan_count_inventory_item(
    config: CountInventoryItemExecutionConfig,
) -> Result<CountInventoryItemExecutionPlan> {
    let config = validate_count_inventory_config(config)?;
    let grid_screen_name = config.grid_screen_name.unwrap();
    let tab_assets =
        grid_screen_name
            .inventory_tab_assets()
            .ok_or_else(|| TaskError::InvalidTaskConfig {
                key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
                message: format!("unsupported inventory grid screen: {grid_screen_name:?}"),
            })?;
    let search_mode = match (config.item_name, config.item_names) {
        (Some(item_name), None) => CountInventorySearchMode::Single { item_name },
        (None, Some(item_names)) => CountInventorySearchMode::Multiple { item_names },
        _ => unreachable!("validated CountInventoryItem search mode"),
    };
    let grid_template = grid_screen_name.grid_template();
    let classifier_rule = GridIconClassifierRule {
        model_name: GRID_ICON_MODEL_NAME.to_string(),
        model_path: GRID_ICON_MODEL_PATH.to_string(),
        prototype_csv_path: GRID_ICON_PROTOTYPE_CSV_PATH.to_string(),
        input_name: GRID_ICON_INPUT_NAME.to_string(),
        feature_dimensions: 64,
        max_distance_squared: 100.0,
    };
    let grid_item_detection_rule = GridItemDetectionRule {
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
    };
    let grid_icon_crop_rule = GridIconCropRule {
        normalized_width: 125,
        normalized_height: 153,
        icon_crop: Rect {
            x: 0,
            y: 0,
            width: 125,
            height: 125,
        },
        bottom_crop: Rect {
            x: 0,
            y: 126,
            width: 125,
            height: 27,
        },
    };
    let count_ocr_rule = GridItemCountOcrRule {
        crop_top_numerator: 128,
        crop_bottom_numerator: 150,
        crop_left_numerator: 5,
        crop_right_numerator: 120,
        crop_height_denominator: 153,
        crop_width_denominator: 125,
        resize_scale: 2,
        ocr_engine: "Paddle".to_string(),
        convert_full_width_digits: true,
        ocr_failed_value: COUNT_INVENTORY_OCR_FAILED,
    };
    let scroll_rule = GridScrollRule {
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
    };
    let weapon_ore_prescroll_rule = WeaponOrePrescrollRule {
        enabled: search_mode.needs_weapon_ore_prescroll(&grid_screen_name),
        item_name_prefix: "精锻用".to_string(),
        hold_scrollbar_bottom_x_1080p: 1289,
        hold_scrollbar_bottom_y_1080p: 936,
        hold_ms: 2_000,
        after_scroll_wait_ms: 300,
    };
    let result_contract = CountInventoryResultContract {
        single_not_found_value: COUNT_INVENTORY_SINGLE_NOT_FOUND,
        ocr_failed_value: COUNT_INVENTORY_OCR_FAILED,
        multiple_missing_items_are_omitted: true,
        multiple_duplicate_names_keep_first: true,
    };
    let open_inventory_rule = CountInventoryOpenInventoryRule {
        open_inventory_action: GenshinAction::OpenInventory,
        open_wait_ms: 1_200,
        retry_attempts: 5,
        retry_when_main_ui_detected: true,
        expired_item_prompt_confirm_asset: "Common/Element:btn_white_confirm.png".to_string(),
        expired_item_prompt_crop_bottom_ratio: 0.2,
        expired_item_prompt_wait_ms: 300,
        tab_assets,
        after_tab_ready_wait_ms: 800,
    };

    let steps = count_inventory_steps(
        &open_inventory_rule,
        &grid_template,
        &grid_item_detection_rule,
        &grid_icon_crop_rule,
        &classifier_rule,
        &count_ocr_rule,
        &scroll_rule,
        &weapon_ore_prescroll_rule,
        &result_contract,
    );

    Ok(CountInventoryItemExecutionPlan {
        task_key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
        display_name: "Count Inventory Item".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        grid_screen_name,
        search_mode,
        open_inventory_rule,
        grid_template,
        grid_item_detection_rule,
        grid_icon_crop_rule,
        classifier_rule,
        count_ocr_rule,
        scroll_rule,
        weapon_ore_prescroll_rule,
        result_contract,
        steps,
        notes: "Inventory open/tab selection, grid constants, icon crop, ONNX prototype matching, count OCR, weapon-ore prescroll, and single/multiple result contracts are modeled and executable as a Rust state machine through injectable hooks; desktop live routing now performs shared game-window, BitBlt, and capture-size preflight, executes OpenInventory through SendInput with post-open template state probes, confirms expired-item prompts, clicks inventory tabs, verifies GridIcon classifier asset availability, dispatches the weapon-ore scrollbar-bottom hold through capture-relative input, can enumerate visible-page grid candidates and crop enumerated grid cells into normalized icon images from the current frame, and keeps the runtime boundary blocked until full GridScroller-style scan_complete semantics are available, while OpenCV-exact grid enumeration with scroll-to-bottom, ONNX/prototype classifier loading and inference, Paddle OCR, and script return object execution remain pending.".to_string(),
    })
}

fn validate_count_inventory_config(
    config: CountInventoryItemExecutionConfig,
) -> Result<CountInventoryItemExecutionConfig> {
    if config.grid_screen_name.is_none() {
        return Err(TaskError::InvalidTaskConfig {
            key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
            message: "gridScreenName is required".to_string(),
        });
    }
    if config.item_name.is_some() && config.item_names.is_some() {
        return Err(TaskError::InvalidTaskConfig {
            key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
            message: "itemName and itemNames cannot both be provided".to_string(),
        });
    }
    if config.item_name.is_none() && config.item_names.is_none() {
        return Err(TaskError::InvalidTaskConfig {
            key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
            message: "itemName or itemNames is required".to_string(),
        });
    }
    if matches!(config.item_names.as_ref(), Some(names) if names.is_empty()) {
        return Err(TaskError::InvalidTaskConfig {
            key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
            message: "itemNames cannot be empty".to_string(),
        });
    }
    if matches!(
        config.grid_screen_name,
        Some(GridScreenName::ArtifactSalvage | GridScreenName::ArtifactSetFilter)
    ) {
        return Err(TaskError::InvalidTaskConfig {
            key: COUNT_INVENTORY_ITEM_TASK_KEY.to_string(),
            message: "gridScreenName is not an inventory tab supported by OpenInventory"
                .to_string(),
        });
    }
    Ok(config)
}

fn count_inventory_steps(
    open_inventory_rule: &CountInventoryOpenInventoryRule,
    grid_template: &GridTemplate,
    grid_item_detection_rule: &GridItemDetectionRule,
    grid_icon_crop_rule: &GridIconCropRule,
    classifier_rule: &GridIconClassifierRule,
    count_ocr_rule: &GridItemCountOcrRule,
    scroll_rule: &GridScrollRule,
    weapon_ore_prescroll_rule: &WeaponOrePrescrollRule,
    result_contract: &CountInventoryResultContract,
) -> Vec<CountInventoryItemStep> {
    vec![
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::Setup,
            "return to main UI before opening inventory",
            CountInventoryItemStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::OpenInventory,
            "press inventory action and wait for bag page",
            CountInventoryItemStepAction::GenshinAction {
                action: GenshinAction::OpenInventory,
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::OpenInventory,
            CountInventoryItemStepCondition::WhenExpiredItemPromptDetected,
            "confirm expired item prompt while opening inventory",
            CountInventoryItemStepAction::ConfirmExpiredItemPrompt {
                confirm_asset: open_inventory_rule
                    .expired_item_prompt_confirm_asset
                    .clone(),
                crop_bottom_ratio: open_inventory_rule.expired_item_prompt_crop_bottom_ratio,
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::OpenInventory,
            CountInventoryItemStepCondition::WhenInventoryTabUnchecked,
            "click requested inventory tab when unchecked",
            CountInventoryItemStepAction::OpenInventoryTab {
                rule: open_inventory_rule.clone(),
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::OpenInventory,
            CountInventoryItemStepCondition::WhenStillOnMainUi,
            "retry inventory action when still on main UI",
            CountInventoryItemStepAction::GenshinAction {
                action: GenshinAction::OpenInventory,
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::Setup,
            "load grid icon ONNX classifier and prototypes",
            CountInventoryItemStepAction::LoadGridIconClassifier {
                rule: classifier_rule.clone(),
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::PreScroll,
            CountInventoryItemStepCondition::WhenWeaponOreRequested,
            "pre-scroll weapon ore page to bottom",
            CountInventoryItemStepAction::PreScrollWeaponOre {
                rule: weapon_ore_prescroll_rule.clone(),
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::ScanGrid,
            "enumerate grid cells across scrolled pages",
            CountInventoryItemStepAction::EnumerateGridItems {
                template: grid_template.clone(),
                detection_rule: grid_item_detection_rule.clone(),
                scroll_rule: scroll_rule.clone(),
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::ScanGrid,
            "crop normalized grid item icon",
            CountInventoryItemStepAction::CropGridIcon {
                rule: grid_icon_crop_rule.clone(),
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::ScanGrid,
            "infer item name from grid icon prototype distance",
            CountInventoryItemStepAction::InferGridIcon {
                rule: classifier_rule.clone(),
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::Count,
            CountInventoryItemStepCondition::WhenClassifierMatchesTarget,
            "OCR item count and convert full-width digits",
            CountInventoryItemStepAction::OcrGridItemCount {
                rule: count_ocr_rule.clone(),
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::Result,
            CountInventoryItemStepCondition::WhenAllRequestedItemsFound,
            "stop scanning once all requested items are found",
            CountInventoryItemStepAction::ReturnResult {
                contract: result_contract.clone(),
            },
        ),
        CountInventoryItemStep::conditional(
            CountInventoryItemStepPhase::Result,
            CountInventoryItemStepCondition::WhenScanComplete,
            "return missing or partial inventory count result",
            CountInventoryItemStepAction::ReturnResult {
                contract: result_contract.clone(),
            },
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::Cleanup,
            "clear grid overlay drawings",
            CountInventoryItemStepAction::ClearVisionDrawings,
        ),
        CountInventoryItemStep::new(
            CountInventoryItemStepPhase::Cleanup,
            "return to main UI after inventory scan",
            CountInventoryItemStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
    ]
}
