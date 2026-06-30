use bgi_core::AutoEatConfig;
use bgi_vision::{Point, Rect, RgbPixel, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    plan_count_inventory_item, CommonJobRuntimeOutcome, CountInventoryGridIconMatch,
    CountInventoryGridItemFrame, CountInventoryItemExecutionConfig,
    CountInventoryItemExecutionPlan, CountInventoryOpenInventoryOutcome,
    CountInventoryOpenInventoryRule, GridIconClassifierRule, GridIconCropRule,
    GridItemCountOcrRule, GridItemDetectionRule, GridScreenName, GridScrollRule, GridTemplate,
    Result, TaskError, TaskPortState, COUNT_INVENTORY_OCR_FAILED, COUNT_INVENTORY_SINGLE_NOT_FOUND,
    RETURN_MAIN_UI_TASK_KEY,
};

pub const AUTO_EAT_TASK_KEY: &str = "AutoEat";
pub const AUTO_EAT_FOOD_TASK_KEY: &str = "AutoEatFood";
pub const AUTO_EAT_SCRIPT_TASK_NAME: &str = "AutoEat";
pub const AUTO_EAT_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_EAT_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_EAT_RECOVERY_ASSET: &str = "AutoEat:Recovery.png";
pub const AUTO_EAT_RESURRECTION_ASSET: &str = "AutoEat:Resurrection.png";
pub const AUTO_EAT_QUICK_USE_GADGET_ACTION: &str = "QuickUseGadget";
pub const AUTO_EAT_FOOD_WHITE_CONFIRM_ASSET: &str = "Common/Element:btn_white_confirm.png";
pub const AUTO_EAT_FOOD_CONFIRM_DELAY_MS: u64 = 300;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: AutoEatConfigRule,
    pub detection_rule: AutoEatDetectionRule,
    pub state_rule: AutoEatStateRule,
    pub locators: AutoEatLocators,
    pub steps: Vec<AutoEatTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoEatExecutionConfig {
    pub capture_size: Size,
    pub auto_eat_config: AutoEatConfig,
}

impl Default for AutoEatExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_EAT_DEFAULT_CAPTURE_WIDTH,
                AUTO_EAT_DEFAULT_CAPTURE_HEIGHT,
            ),
            auto_eat_config: AutoEatConfig::default(),
        }
    }
}

impl AutoEatExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let auto_eat_value = value
            .get("autoEatConfig")
            .or_else(|| value.get("AutoEatConfig"))
            .or_else(|| value.get("auto_eat_config"))
            .unwrap_or(value);
        config.auto_eat_config = serde_json::from_value(auto_eat_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoEatConfigRule {
    pub enabled: bool,
    pub show_notification: bool,
    pub check_interval_ms: u64,
    pub eat_interval_ms: u64,
    pub test_food_name: Option<String>,
    pub default_atk_boosting_dish_name: Option<String>,
    pub default_adventurers_dish_name: Option<String>,
    pub default_def_boosting_dish_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatDetectionRule {
    pub low_hp_source: String,
    pub low_hp_pixel_probe: AutoEatLowHpPixelProbe,
    pub recovery_cache_ttl_ms: u64,
    pub resurrection_cooldown_ms: u64,
    pub eat_action: String,
    pub resurrection_action: String,
    pub exceptions_are_logged_and_ignored: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoEatLowHpPixelProbe {
    pub point: Point,
    pub expected_rgb: RgbPixel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoEatStateRule {
    pub prev_execute_time_field: String,
    pub last_recovery_check_time_field: String,
    pub last_resurrection_time_field: String,
    pub last_eat_time_field: String,
    pub recovery_detected_field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatLocators {
    pub recovery_icon: AutoEatTemplateLocator,
    pub resurrection_icon: AutoEatTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoEatTickStep {
    pub phase: AutoEatTickPhase,
    pub condition: AutoEatTickCondition,
    pub action: AutoEatTickAction,
}

impl AutoEatTickStep {
    fn new(
        phase: AutoEatTickPhase,
        condition: AutoEatTickCondition,
        action: AutoEatTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoEatTickPhase {
    Throttle,
    LowHpRecovery,
    Resurrection,
    ErrorHandling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoEatTickCondition {
    WhenCheckIntervalNotElapsed,
    WhenCurrentAvatarIsLowHp,
    WhenRecoveryCacheExpired,
    WhenRecoveryCachedOrDetected,
    WhenEatIntervalElapsed,
    WhenResurrectionIconDetected,
    WhenResurrectionCooldownElapsed,
    OnDetectionError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoEatTickAction {
    SkipTick,
    DetectCurrentAvatarLowHp,
    ClearRecoveryCache,
    DetectRecoveryIcon,
    SimulateGenshinAction { action: String },
    DetectResurrectionIcon,
    LogDebugAndContinue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatTriggerState {
    pub prev_execute_ms: Option<u64>,
    pub last_recovery_check_time_ms: Option<u64>,
    pub last_resurrection_time_ms: Option<u64>,
    pub last_eat_time_ms: Option<u64>,
    pub recovery_detected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatTickObservation {
    pub now_ms: u64,
    pub current_avatar_low_hp: bool,
    pub recovery_icon_detected: bool,
    pub resurrection_icon_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatTickDecisionReport {
    pub processed: bool,
    pub skip_reason: Option<AutoEatTickSkipReason>,
    pub low_hp_detected: bool,
    pub recovery_cache_cleared: bool,
    pub recovery_cache_updated: bool,
    pub recovery_available: bool,
    pub resurrection_available: bool,
    pub actions: Vec<AutoEatTriggeredAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoEatTickSkipReason {
    Disabled,
    CheckIntervalNotElapsed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoEatTriggeredAction {
    Eat { action: String },
    Resurrect { action: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatTickExecutionReport {
    pub task_key: String,
    pub decision: AutoEatTickDecisionReport,
    pub dispatched_actions: Vec<AutoEatTriggeredAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AutoEatFoodEffectType {
    RecoveryDish,
    ATKBoostingDish,
    AdventurersDish,
    DEFBoostingDish,
    Potion,
    Other,
}

impl AutoEatFoodEffectType {
    fn from_legacy_index(index: i64) -> Option<Self> {
        match index {
            0 => Some(Self::RecoveryDish),
            1 => Some(Self::ATKBoostingDish),
            2 => Some(Self::AdventurersDish),
            3 => Some(Self::DEFBoostingDish),
            4 => Some(Self::Potion),
            5 => Some(Self::Other),
            _ => None,
        }
    }

    fn from_legacy_name(name: &str) -> Option<Self> {
        match name.trim() {
            "RecoveryDish" | "recoveryDish" | "recovery" | "恢复类料理" => {
                Some(Self::RecoveryDish)
            }
            "ATKBoostingDish" | "AtkBoostingDish" | "atkBoostingDish" | "attack" | "攻击类料理" => {
                Some(Self::ATKBoostingDish)
            }
            "AdventurersDish" | "adventurersDish" | "adventurer" | "冒险类料理" => {
                Some(Self::AdventurersDish)
            }
            "DEFBoostingDish" | "DefBoostingDish" | "defBoostingDish" | "defense"
            | "防御类料理" => Some(Self::DEFBoostingDish),
            "Potion" | "potion" | "药剂" => Some(Self::Potion),
            "Other" | "other" | "其他" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoEatFoodExecutionConfig {
    pub capture_size: Size,
    pub food_name: Option<String>,
    pub food_effect_type: Option<AutoEatFoodEffectType>,
    pub default_atk_boosting_dish_name: Option<String>,
    pub default_adventurers_dish_name: Option<String>,
    pub default_def_boosting_dish_name: Option<String>,
}

impl Default for AutoEatFoodExecutionConfig {
    fn default() -> Self {
        let auto_eat_config = AutoEatConfig::default();
        Self {
            capture_size: Size::new(
                AUTO_EAT_DEFAULT_CAPTURE_WIDTH,
                AUTO_EAT_DEFAULT_CAPTURE_HEIGHT,
            ),
            food_name: None,
            food_effect_type: None,
            default_atk_boosting_dish_name: auto_eat_config.default_atk_boosting_dish_name,
            default_adventurers_dish_name: auto_eat_config.default_adventurers_dish_name,
            default_def_boosting_dish_name: auto_eat_config.default_def_boosting_dish_name,
        }
    }
}

impl AutoEatFoodExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Result<Self> {
        let mut config = Self::default();
        let Some(value) = value else {
            return Ok(config);
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let sources = auto_eat_food_config_sources(value);
        if let Some(auto_eat_value) = sources.iter().find_map(|source| {
            source
                .get("autoEatConfig")
                .or_else(|| source.get("AutoEatConfig"))
                .or_else(|| source.get("auto_eat_config"))
        }) {
            if let Ok(auto_eat_config) =
                serde_json::from_value::<AutoEatConfig>(auto_eat_value.clone())
            {
                config.default_atk_boosting_dish_name =
                    auto_eat_config.default_atk_boosting_dish_name;
                config.default_adventurers_dish_name =
                    auto_eat_config.default_adventurers_dish_name;
                config.default_def_boosting_dish_name =
                    auto_eat_config.default_def_boosting_dish_name;
            }
        }

        config.food_name =
            optional_string_from_sources(&sources, &["foodName", "FoodName", "food_name"])
                .flatten();
        config.food_effect_type = optional_food_effect_type_from_sources(
            &sources,
            &["foodEffectType", "FoodEffectType", "food_effect_type"],
        )?;
        if let Some(value) = optional_string_from_sources(
            &sources,
            &[
                "defaultAtkBoostingDishName",
                "DefaultAtkBoostingDishName",
                "default_atk_boosting_dish_name",
            ],
        ) {
            config.default_atk_boosting_dish_name = value;
        }
        if let Some(value) = optional_string_from_sources(
            &sources,
            &[
                "defaultAdventurersDishName",
                "DefaultAdventurersDishName",
                "default_adventurers_dish_name",
            ],
        ) {
            config.default_adventurers_dish_name = value;
        }
        if let Some(value) = optional_string_from_sources(
            &sources,
            &[
                "defaultDefBoostingDishName",
                "DefaultDefBoostingDishName",
                "default_def_boosting_dish_name",
            ],
        ) {
            config.default_def_boosting_dish_name = value;
        }

        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatFoodExecutionPlan {
    pub task_key: String,
    pub script_task_name: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub mode: AutoEatFoodPlanMode,
    pub food_name: Option<String>,
    pub food_effect_type: Option<AutoEatFoodEffectType>,
    pub inventory_plan: Option<CountInventoryItemExecutionPlan>,
    pub use_rule: AutoEatFoodUseRule,
    pub result_contract: AutoEatFoodResultContract,
    pub steps: Vec<AutoEatFoodStep>,
    pub pending_native: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoEatFoodPlanMode {
    InventoryFood {
        food_name: String,
        source: AutoEatFoodNameSource,
    },
    PortableNutritionBagLoop,
    MissingDefaultFood {
        effect_type: AutoEatFoodEffectType,
        config_label: String,
        log_message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoEatFoodNameSource {
    FoodNameConfig,
    FoodEffectDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatFoodUseRule {
    pub target_grid_screen_name: GridScreenName,
    pub click_matched_item: bool,
    pub count_before_use: bool,
    pub after_item_click_wait_ms: u64,
    pub confirm_button_asset: String,
    pub click_confirm_if_visible: bool,
    pub clear_drawings_in_finally: bool,
    pub return_main_ui_after_use: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatFoodResultContract {
    pub not_found_value: i32,
    pub ocr_failed_value: i32,
    pub success_count_offset: i32,
    pub missing_default_returns_none: bool,
    pub portable_bag_loop_returns_none: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoEatFoodStep {
    pub phase: AutoEatFoodStepPhase,
    pub condition: AutoEatFoodStepCondition,
    pub action: AutoEatFoodStepAction,
}

impl AutoEatFoodStep {
    fn new(
        phase: AutoEatFoodStepPhase,
        condition: AutoEatFoodStepCondition,
        action: AutoEatFoodStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoEatFoodStepPhase {
    Setup,
    OpenFoodInventory,
    ScanGrid,
    UseFood,
    Cleanup,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoEatFoodStepCondition {
    Always,
    WhenInventoryFoodMode,
    WhenPortableNutritionBagMode,
    WhenDefaultFoodMissing,
    WhenClassifierMatchesFood,
    WhenCountOcrFails,
    Finally,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoEatFoodStepAction {
    LogInfo {
        message: String,
    },
    LogWarning {
        message: String,
    },
    ReturnMainUi,
    PortableNutritionBagLoop,
    OpenFoodInventoryTab {
        rule: CountInventoryOpenInventoryRule,
    },
    LoadGridIconClassifier {
        rule: GridIconClassifierRule,
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
    ClickMatchedFoodItem {
        food_name: String,
    },
    OcrMatchedFoodCount {
        rule: GridItemCountOcrRule,
    },
    DelayAfterItemClick {
        delay_ms: u64,
    },
    ConfirmUseFoodIfVisible {
        asset: String,
    },
    ClearVisionDrawings,
    ReturnResult {
        contract: AutoEatFoodResultContract,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatFoodUseObservation {
    pub matched_food_name: Option<String>,
    pub count_ocr_text: Option<String>,
    pub confirm_button_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoEatFoodUseDecisionReport {
    pub outcome: AutoEatFoodUseOutcome,
    pub return_value: Option<i32>,
    pub normalized_count_text: Option<String>,
    pub actions: Vec<AutoEatFoodUseAction>,
    pub logs: Vec<AutoEatFoodUseLog>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoEatFoodUseOutcome {
    Consumed,
    OcrFailedButConsumed,
    NotFound,
    MissingDefaultSkipped,
    PortableNutritionBagLoopPending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoEatFoodUseAction {
    ClickMatchedFoodItem { food_name: String },
    DelayAfterItemClick { delay_ms: u64 },
    ClickWhiteConfirmIfPresent { asset: String, detected: bool },
    ClearVisionDrawings,
    ReturnMainUi,
    Skip { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "level", content = "message")]
pub enum AutoEatFoodUseLog {
    Info(String),
    Warning(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoEatFoodRuntimeActionKind {
    CommonJob,
    OpenInventory,
    ConfirmExpiredItemPrompt,
    OpenInventoryTab,
    LoadGridIconClassifier,
    EnumerateGridItems,
    CropGridIcon,
    InferGridIcon,
    OcrGridItemCount,
    ClickMatchedFoodItem,
    DelayAfterItemClick,
    ClickWhiteConfirmIfPresent,
    ClearVisionDrawings,
    ReturnMainUi,
    Log,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoEatFoodRuntimeActionReport {
    pub action_kind: AutoEatFoodRuntimeActionKind,
    pub outcome: CommonJobRuntimeOutcome,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoEatFoodExecutorState {
    pub initial_return_main_ui_completed: Option<bool>,
    pub open_inventory_outcome: Option<CountInventoryOpenInventoryOutcome>,
    pub expired_item_prompt_confirmed: Option<bool>,
    pub inventory_tab_opened: Option<bool>,
    pub classifier_loaded: bool,
    pub grid_items: Vec<CountInventoryGridItemFrame>,
    pub grid_icons_cropped: bool,
    pub inferred_icons: Vec<CountInventoryGridIconMatch>,
    pub target_match: Option<CountInventoryGridIconMatch>,
    pub ocr_count_text: Option<String>,
    pub confirm_button_detected: Option<bool>,
    pub decision: Option<AutoEatFoodUseDecisionReport>,
    pub final_return_main_ui_completed: Option<bool>,
    pub vision_drawings_cleared: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoEatFoodExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: AutoEatFoodExecutorState,
    pub executed_actions: Vec<AutoEatFoodRuntimeActionReport>,
}

pub trait AutoEatRuntime {
    fn observe_auto_eat_tick(
        &mut self,
        plan: &AutoEatExecutionPlan,
    ) -> Result<AutoEatTickObservation>;

    fn dispatch_auto_eat_action(&mut self, action: &AutoEatTriggeredAction) -> Result<()>;
}

pub trait AutoEatFoodRuntime {
    fn execute_auto_eat_food_common_job(
        &mut self,
        task_key: &str,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn open_auto_eat_food_inventory(
        &mut self,
        rule: &CountInventoryOpenInventoryRule,
    ) -> Result<CountInventoryOpenInventoryOutcome>;

    fn confirm_auto_eat_food_expired_item_prompt(
        &mut self,
        confirm_asset: &str,
        crop_bottom_ratio: f64,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn open_auto_eat_food_inventory_tab(
        &mut self,
        rule: &CountInventoryOpenInventoryRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn load_auto_eat_food_grid_icon_classifier(
        &mut self,
        rule: &GridIconClassifierRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn enumerate_auto_eat_food_grid_items(
        &mut self,
        template: &GridTemplate,
        detection_rule: &GridItemDetectionRule,
        scroll_rule: &GridScrollRule,
    ) -> Result<Vec<CountInventoryGridItemFrame>>;

    fn crop_auto_eat_food_grid_icons(
        &mut self,
        items: &[CountInventoryGridItemFrame],
        rule: &GridIconCropRule,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn infer_auto_eat_food_grid_icons(
        &mut self,
        items: &[CountInventoryGridItemFrame],
        rule: &GridIconClassifierRule,
    ) -> Result<Vec<CountInventoryGridIconMatch>>;

    fn ocr_auto_eat_food_item_count(
        &mut self,
        matched: &CountInventoryGridIconMatch,
        rule: &GridItemCountOcrRule,
    ) -> Result<Option<String>>;

    fn click_auto_eat_food_item(
        &mut self,
        matched: &CountInventoryGridIconMatch,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn delay_auto_eat_food_after_item_click(
        &mut self,
        duration_ms: u64,
    ) -> Result<CommonJobRuntimeOutcome>;

    fn click_auto_eat_food_white_confirm_if_present(&mut self, asset: &str) -> Result<bool>;

    fn clear_auto_eat_food_vision_drawings(&mut self) -> Result<CommonJobRuntimeOutcome>;

    fn log_auto_eat_food(&mut self, message: &str) -> Result<CommonJobRuntimeOutcome>;
}

pub fn plan_auto_eat(config: AutoEatExecutionConfig) -> AutoEatExecutionPlan {
    let capture_size = config.capture_size;
    let auto_eat_config = config.auto_eat_config;
    AutoEatExecutionPlan {
        task_key: AUTO_EAT_TASK_KEY.to_string(),
        display_name: "Auto Eat".to_string(),
        capture_size,
        config_rule: AutoEatConfigRule {
            enabled: auto_eat_config.enabled,
            show_notification: auto_eat_config.show_notification,
            check_interval_ms: auto_eat_config.check_interval,
            eat_interval_ms: auto_eat_config.eat_interval,
            test_food_name: auto_eat_config.test_food_name,
            default_atk_boosting_dish_name: auto_eat_config.default_atk_boosting_dish_name,
            default_adventurers_dish_name: auto_eat_config.default_adventurers_dish_name,
            default_def_boosting_dish_name: auto_eat_config.default_def_boosting_dish_name,
        },
        detection_rule: AutoEatDetectionRule {
            low_hp_source: "Bv.CurrentAvatarIsLowHp".to_string(),
            low_hp_pixel_probe: AutoEatLowHpPixelProbe {
                point: Point {
                    x: scaled(808, capture_size),
                    y: scaled(1010, capture_size),
                },
                expected_rgb: RgbPixel {
                    r: 255,
                    g: 90,
                    b: 90,
                },
            },
            recovery_cache_ttl_ms: 30_000,
            resurrection_cooldown_ms: 2_000,
            eat_action: AUTO_EAT_QUICK_USE_GADGET_ACTION.to_string(),
            resurrection_action: AUTO_EAT_QUICK_USE_GADGET_ACTION.to_string(),
            exceptions_are_logged_and_ignored: true,
        },
        state_rule: AutoEatStateRule {
            prev_execute_time_field: "_prevExecute".to_string(),
            last_recovery_check_time_field: "_lastRecoveryCheckTime".to_string(),
            last_resurrection_time_field: "_lastResurrectionTime".to_string(),
            last_eat_time_field: "_lastEatTime".to_string(),
            recovery_detected_field: "_recoveryDetected".to_string(),
        },
        locators: auto_eat_locators(capture_size),
        steps: auto_eat_steps(),
        executor_ready: true,
        pending_native: vec![
            "full BV current-avatar low-HP parity beyond the desktop pixel probe remains pending"
                .to_string(),
            "optional notification behavior and solo AutoEatTask food inventory flow".to_string(),
        ],
    }
}

pub fn plan_auto_eat_food(config: AutoEatFoodExecutionConfig) -> Result<AutoEatFoodExecutionPlan> {
    let capture_size = config.capture_size;
    let mode = resolve_auto_eat_food_mode(&config)?;
    let food_name = match &mode {
        AutoEatFoodPlanMode::InventoryFood { food_name, .. } => Some(food_name.clone()),
        _ => None,
    };
    let inventory_plan = match &food_name {
        Some(food_name) => Some(plan_count_inventory_item(
            CountInventoryItemExecutionConfig {
                capture_size,
                grid_screen_name: Some(GridScreenName::Food),
                item_name: Some(food_name.clone()),
                item_names: None,
            },
        )?),
        None => None,
    };
    let use_rule = AutoEatFoodUseRule {
        target_grid_screen_name: GridScreenName::Food,
        click_matched_item: true,
        count_before_use: true,
        after_item_click_wait_ms: AUTO_EAT_FOOD_CONFIRM_DELAY_MS,
        confirm_button_asset: AUTO_EAT_FOOD_WHITE_CONFIRM_ASSET.to_string(),
        click_confirm_if_visible: true,
        clear_drawings_in_finally: true,
        return_main_ui_after_use: true,
    };
    let result_contract = AutoEatFoodResultContract {
        not_found_value: COUNT_INVENTORY_SINGLE_NOT_FOUND,
        ocr_failed_value: COUNT_INVENTORY_OCR_FAILED,
        success_count_offset: -1,
        missing_default_returns_none: true,
        portable_bag_loop_returns_none: true,
    };
    let steps = auto_eat_food_steps(&mode, inventory_plan.as_ref(), &use_rule, &result_contract);

    Ok(AutoEatFoodExecutionPlan {
        task_key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
        script_task_name: AUTO_EAT_SCRIPT_TASK_NAME.to_string(),
        display_name: "Auto Eat Food".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        mode,
        food_name,
        food_effect_type: config.food_effect_type,
        inventory_plan,
        use_rule,
        result_contract,
        steps,
        pending_native: vec![
            "desktop inventory-food live routing now shares game-window, BitBlt, capture-size, and CountInventory adapter-boundary preflight; live ReturnMainUi, opening Food inventory, expired-item prompt confirmation, Food tab selection, and GridIcon classifier asset preflight are wired through the shared common-job runtime, while grid enumeration, ONNX/prototype classifier loading and inference, count OCR, matched food click, white confirm click, overlay cleanup, and final ReturnMainUi remain pending".to_string(),
            "portable nutrition bag loop for script-dispatched AutoEat without foodName remains pending"
                .to_string(),
        ],
        notes: "Rust models the script-dispatched AutoEatTask foodName/foodEffectType resolution, Food inventory grid constants through CountInventoryItem, icon classifier/OCR contracts, use-confirm rule, full-width digit normalization, legacy int? result semantics, and an injectable Rust executor for the inventory-food branch; desktop live routing shares CountInventory game-window, BitBlt, capture-size, inventory opening, expired-prompt confirmation, tab-selection, classifier asset checks, and first-active-adapter preflight before the remaining live grid/ONNX/OCR/click adapters are wired.".to_string(),
    })
}

pub fn decide_auto_eat_food_use(
    plan: &AutoEatFoodExecutionPlan,
    observation: AutoEatFoodUseObservation,
) -> AutoEatFoodUseDecisionReport {
    match &plan.mode {
        AutoEatFoodPlanMode::MissingDefaultFood { log_message, .. } => {
            return AutoEatFoodUseDecisionReport {
                outcome: AutoEatFoodUseOutcome::MissingDefaultSkipped,
                return_value: None,
                normalized_count_text: None,
                actions: vec![AutoEatFoodUseAction::Skip {
                    reason: log_message.clone(),
                }],
                logs: vec![AutoEatFoodUseLog::Info(log_message.clone())],
            };
        }
        AutoEatFoodPlanMode::PortableNutritionBagLoop => {
            return AutoEatFoodUseDecisionReport {
                outcome: AutoEatFoodUseOutcome::PortableNutritionBagLoopPending,
                return_value: None,
                normalized_count_text: None,
                actions: vec![AutoEatFoodUseAction::Skip {
                    reason: "portable nutrition bag loop remains native-pending".to_string(),
                }],
                logs: Vec::new(),
            };
        }
        AutoEatFoodPlanMode::InventoryFood { .. } => {}
    }

    let Some(food_name) = plan.food_name.as_deref() else {
        return AutoEatFoodUseDecisionReport {
            outcome: AutoEatFoodUseOutcome::NotFound,
            return_value: Some(plan.result_contract.not_found_value),
            normalized_count_text: None,
            actions: Vec::new(),
            logs: Vec::new(),
        };
    };

    if observation.matched_food_name.as_deref() != Some(food_name) {
        return AutoEatFoodUseDecisionReport {
            outcome: AutoEatFoodUseOutcome::NotFound,
            return_value: Some(plan.result_contract.not_found_value),
            normalized_count_text: None,
            actions: vec![
                AutoEatFoodUseAction::ClearVisionDrawings,
                AutoEatFoodUseAction::ReturnMainUi,
            ],
            logs: vec![AutoEatFoodUseLog::Info(format!("没有找到{food_name}"))],
        };
    }

    let normalized_count_text = observation
        .count_ocr_text
        .as_deref()
        .map(normalize_auto_eat_food_count_text);
    let parsed_count = normalized_count_text
        .as_deref()
        .and_then(|text| text.trim().parse::<i32>().ok());
    let mut logs = Vec::new();
    let (outcome, return_value) = match parsed_count {
        Some(count) => (
            AutoEatFoodUseOutcome::Consumed,
            count + plan.result_contract.success_count_offset,
        ),
        None => {
            let raw_text = observation.count_ocr_text.unwrap_or_default();
            logs.push(AutoEatFoodUseLog::Warning(format!(
                "无法识别食物数量：{raw_text}，依然尝试使用"
            )));
            (
                AutoEatFoodUseOutcome::OcrFailedButConsumed,
                plan.result_contract.ocr_failed_value,
            )
        }
    };
    logs.push(AutoEatFoodUseLog::Info(format!(
        "吃了一份{food_name}，真香！"
    )));

    AutoEatFoodUseDecisionReport {
        outcome,
        return_value: Some(return_value),
        normalized_count_text,
        actions: vec![
            AutoEatFoodUseAction::ClickMatchedFoodItem {
                food_name: food_name.to_string(),
            },
            AutoEatFoodUseAction::DelayAfterItemClick {
                delay_ms: plan.use_rule.after_item_click_wait_ms,
            },
            AutoEatFoodUseAction::ClickWhiteConfirmIfPresent {
                asset: plan.use_rule.confirm_button_asset.clone(),
                detected: observation.confirm_button_detected,
            },
            AutoEatFoodUseAction::ClearVisionDrawings,
            AutoEatFoodUseAction::ReturnMainUi,
        ],
        logs,
    }
}

pub fn execute_auto_eat_food_plan<R>(
    plan: &AutoEatFoodExecutionPlan,
    runtime: &mut R,
) -> Result<AutoEatFoodExecutionReport>
where
    R: AutoEatFoodRuntime,
{
    let mut state = AutoEatFoodExecutorState::default();
    let mut executed_actions = Vec::new();

    match &plan.mode {
        AutoEatFoodPlanMode::MissingDefaultFood { .. }
        | AutoEatFoodPlanMode::PortableNutritionBagLoop => {
            let decision = decide_auto_eat_food_use(
                plan,
                AutoEatFoodUseObservation {
                    matched_food_name: None,
                    count_ocr_text: None,
                    confirm_button_detected: false,
                },
            );
            execute_auto_eat_food_decision_actions(
                &decision,
                plan,
                runtime,
                &mut state,
                &mut executed_actions,
            )?;
            state.decision = Some(decision);
            return Ok(auto_eat_food_execution_report(
                plan,
                state,
                executed_actions,
            ));
        }
        AutoEatFoodPlanMode::InventoryFood { .. } => {}
    }

    let inventory_plan =
        plan.inventory_plan
            .as_ref()
            .ok_or_else(|| TaskError::InvalidTaskConfig {
                key: plan.task_key.clone(),
                message: "AutoEatFood inventory mode requires an inventory plan".to_string(),
            })?;

    execute_auto_eat_food_inventory_scan(
        inventory_plan,
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
    )?;
    let matched_food_name = state
        .target_match
        .as_ref()
        .map(|matched| matched.item_name.clone());
    let decision = decide_auto_eat_food_use(
        plan,
        AutoEatFoodUseObservation {
            matched_food_name,
            count_ocr_text: state.ocr_count_text.clone(),
            confirm_button_detected: state.confirm_button_detected.unwrap_or(false),
        },
    );
    execute_auto_eat_food_decision_actions(
        &decision,
        plan,
        runtime,
        &mut state,
        &mut executed_actions,
    )?;
    state.decision = Some(decision);

    Ok(auto_eat_food_execution_report(
        plan,
        state,
        executed_actions,
    ))
}

fn execute_auto_eat_food_inventory_scan<R>(
    inventory_plan: &CountInventoryItemExecutionPlan,
    plan: &AutoEatFoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoEatFoodExecutorState,
    executed_actions: &mut Vec<AutoEatFoodRuntimeActionReport>,
) -> Result<()>
where
    R: AutoEatFoodRuntime,
{
    let outcome = runtime.execute_auto_eat_food_common_job(RETURN_MAIN_UI_TASK_KEY)?;
    state.initial_return_main_ui_completed = Some(auto_eat_food_outcome_succeeded(outcome));
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::CommonJob,
        outcome,
    ));

    let open_outcome = runtime.open_auto_eat_food_inventory(&inventory_plan.open_inventory_rule)?;
    state.open_inventory_outcome = Some(open_outcome);
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::OpenInventory,
        CommonJobRuntimeOutcome::Matched(!open_outcome.still_on_main_ui),
    ));

    if open_outcome.expired_item_prompt_detected {
        let outcome = runtime.confirm_auto_eat_food_expired_item_prompt(
            &inventory_plan
                .open_inventory_rule
                .expired_item_prompt_confirm_asset,
            inventory_plan
                .open_inventory_rule
                .expired_item_prompt_crop_bottom_ratio,
        )?;
        state.expired_item_prompt_confirmed = Some(auto_eat_food_outcome_succeeded(outcome));
        executed_actions.push(auto_eat_food_action_report(
            AutoEatFoodRuntimeActionKind::ConfirmExpiredItemPrompt,
            outcome,
        ));
    }

    if !open_outcome.inventory_tab_checked {
        let outcome =
            runtime.open_auto_eat_food_inventory_tab(&inventory_plan.open_inventory_rule)?;
        state.inventory_tab_opened = Some(auto_eat_food_outcome_succeeded(outcome));
        executed_actions.push(auto_eat_food_action_report(
            AutoEatFoodRuntimeActionKind::OpenInventoryTab,
            outcome,
        ));
    }

    let outcome =
        runtime.load_auto_eat_food_grid_icon_classifier(&inventory_plan.classifier_rule)?;
    state.classifier_loaded = auto_eat_food_outcome_succeeded(outcome);
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::LoadGridIconClassifier,
        outcome,
    ));

    state.grid_items = runtime.enumerate_auto_eat_food_grid_items(
        &inventory_plan.grid_template,
        &inventory_plan.grid_item_detection_rule,
        &inventory_plan.scroll_rule,
    )?;
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::EnumerateGridItems,
        CommonJobRuntimeOutcome::Matched(!state.grid_items.is_empty()),
    ));

    let outcome = runtime
        .crop_auto_eat_food_grid_icons(&state.grid_items, &inventory_plan.grid_icon_crop_rule)?;
    state.grid_icons_cropped = auto_eat_food_outcome_succeeded(outcome);
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::CropGridIcon,
        outcome,
    ));

    state.inferred_icons = runtime
        .infer_auto_eat_food_grid_icons(&state.grid_items, &inventory_plan.classifier_rule)?;
    state.target_match = state
        .inferred_icons
        .iter()
        .find(|matched| plan.food_name.as_deref() == Some(matched.item_name.as_str()))
        .cloned();
    executed_actions.push(auto_eat_food_action_report(
        AutoEatFoodRuntimeActionKind::InferGridIcon,
        CommonJobRuntimeOutcome::Matched(state.target_match.is_some()),
    ));

    if let Some(matched) = state.target_match.as_ref() {
        state.ocr_count_text =
            runtime.ocr_auto_eat_food_item_count(matched, &inventory_plan.count_ocr_rule)?;
        executed_actions.push(auto_eat_food_action_report(
            AutoEatFoodRuntimeActionKind::OcrGridItemCount,
            CommonJobRuntimeOutcome::Matched(state.ocr_count_text.is_some()),
        ));
    }

    Ok(())
}

fn execute_auto_eat_food_decision_actions<R>(
    decision: &AutoEatFoodUseDecisionReport,
    plan: &AutoEatFoodExecutionPlan,
    runtime: &mut R,
    state: &mut AutoEatFoodExecutorState,
    executed_actions: &mut Vec<AutoEatFoodRuntimeActionReport>,
) -> Result<()>
where
    R: AutoEatFoodRuntime,
{
    for log in &decision.logs {
        let message = match log {
            AutoEatFoodUseLog::Info(message) | AutoEatFoodUseLog::Warning(message) => message,
        };
        let outcome = runtime.log_auto_eat_food(message)?;
        executed_actions.push(auto_eat_food_action_report(
            AutoEatFoodRuntimeActionKind::Log,
            outcome,
        ));
    }

    for action in &decision.actions {
        match action {
            AutoEatFoodUseAction::ClickMatchedFoodItem { .. } => {
                let Some(matched) = state.target_match.as_ref() else {
                    return Err(TaskError::CommonJobExecution(
                        "AutoEatFood has no matched food item to click".to_string(),
                    ));
                };
                let outcome = runtime.click_auto_eat_food_item(matched)?;
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::ClickMatchedFoodItem,
                    outcome,
                ));
            }
            AutoEatFoodUseAction::DelayAfterItemClick { delay_ms } => {
                let outcome = runtime.delay_auto_eat_food_after_item_click(*delay_ms)?;
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::DelayAfterItemClick,
                    outcome,
                ));
            }
            AutoEatFoodUseAction::ClickWhiteConfirmIfPresent { asset, .. } => {
                let detected = runtime.click_auto_eat_food_white_confirm_if_present(asset)?;
                state.confirm_button_detected = Some(detected);
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::ClickWhiteConfirmIfPresent,
                    CommonJobRuntimeOutcome::Matched(detected),
                ));
            }
            AutoEatFoodUseAction::ClearVisionDrawings => {
                let outcome = runtime.clear_auto_eat_food_vision_drawings()?;
                state.vision_drawings_cleared = auto_eat_food_outcome_succeeded(outcome);
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::ClearVisionDrawings,
                    outcome,
                ));
            }
            AutoEatFoodUseAction::ReturnMainUi => {
                let outcome = runtime.execute_auto_eat_food_common_job(RETURN_MAIN_UI_TASK_KEY)?;
                state.final_return_main_ui_completed =
                    Some(auto_eat_food_outcome_succeeded(outcome));
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::ReturnMainUi,
                    outcome,
                ));
            }
            AutoEatFoodUseAction::Skip { reason } => {
                let outcome = runtime.log_auto_eat_food(reason)?;
                executed_actions.push(auto_eat_food_action_report(
                    AutoEatFoodRuntimeActionKind::Skip,
                    outcome,
                ));
            }
        }
    }

    if plan.use_rule.clear_drawings_in_finally && !state.vision_drawings_cleared {
        let outcome = runtime.clear_auto_eat_food_vision_drawings()?;
        state.vision_drawings_cleared = auto_eat_food_outcome_succeeded(outcome);
        executed_actions.push(auto_eat_food_action_report(
            AutoEatFoodRuntimeActionKind::ClearVisionDrawings,
            outcome,
        ));
    }

    Ok(())
}

fn auto_eat_food_execution_report(
    plan: &AutoEatFoodExecutionPlan,
    state: AutoEatFoodExecutorState,
    executed_actions: Vec<AutoEatFoodRuntimeActionReport>,
) -> AutoEatFoodExecutionReport {
    AutoEatFoodExecutionReport {
        task_key: plan.task_key.clone(),
        completed: state
            .decision
            .as_ref()
            .is_some_and(|decision| decision.return_value.is_some()),
        state,
        executed_actions,
    }
}

fn auto_eat_food_action_report(
    action_kind: AutoEatFoodRuntimeActionKind,
    outcome: CommonJobRuntimeOutcome,
) -> AutoEatFoodRuntimeActionReport {
    AutoEatFoodRuntimeActionReport {
        action_kind,
        outcome,
    }
}

fn auto_eat_food_outcome_succeeded(outcome: CommonJobRuntimeOutcome) -> bool {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => value,
        CommonJobRuntimeOutcome::None => true,
    }
}

pub fn normalize_auto_eat_food_count_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if ('\u{ff10}'..='\u{ff19}').contains(&character) {
                char::from_u32(character as u32 - '\u{ff10}' as u32 + '0' as u32)
                    .unwrap_or(character)
            } else {
                character
            }
        })
        .collect()
}

pub fn execute_auto_eat_tick_plan<R>(
    plan: &AutoEatExecutionPlan,
    state: &mut AutoEatTriggerState,
    runtime: &mut R,
) -> Result<AutoEatTickExecutionReport>
where
    R: AutoEatRuntime,
{
    let observation = runtime.observe_auto_eat_tick(plan)?;
    let decision = decide_auto_eat_tick(state, observation, plan);
    let mut dispatched_actions = Vec::new();
    for action in &decision.actions {
        runtime.dispatch_auto_eat_action(action)?;
        dispatched_actions.push(action.clone());
    }

    Ok(AutoEatTickExecutionReport {
        task_key: plan.task_key.clone(),
        decision,
        dispatched_actions,
    })
}

pub fn decide_auto_eat_tick(
    state: &mut AutoEatTriggerState,
    observation: AutoEatTickObservation,
    plan: &AutoEatExecutionPlan,
) -> AutoEatTickDecisionReport {
    if !plan.config_rule.enabled {
        return AutoEatTickDecisionReport {
            processed: false,
            skip_reason: Some(AutoEatTickSkipReason::Disabled),
            low_hp_detected: observation.current_avatar_low_hp,
            recovery_cache_cleared: false,
            recovery_cache_updated: false,
            recovery_available: state.recovery_detected,
            resurrection_available: observation.resurrection_icon_detected,
            actions: Vec::new(),
        };
    }

    if elapsed_ms_since(state.prev_execute_ms, observation.now_ms)
        <= plan.config_rule.check_interval_ms
    {
        return AutoEatTickDecisionReport {
            processed: false,
            skip_reason: Some(AutoEatTickSkipReason::CheckIntervalNotElapsed),
            low_hp_detected: observation.current_avatar_low_hp,
            recovery_cache_cleared: false,
            recovery_cache_updated: false,
            recovery_available: state.recovery_detected,
            resurrection_available: observation.resurrection_icon_detected,
            actions: Vec::new(),
        };
    }

    state.prev_execute_ms = Some(observation.now_ms);
    let mut recovery_cache_cleared = false;
    let mut recovery_cache_updated = false;
    let mut recovery_available = state.recovery_detected;
    let mut actions = Vec::new();

    if observation.current_avatar_low_hp {
        if elapsed_ms_since(state.last_recovery_check_time_ms, observation.now_ms)
            >= plan.detection_rule.recovery_cache_ttl_ms
        {
            state.recovery_detected = false;
            state.last_recovery_check_time_ms = Some(observation.now_ms);
            recovery_cache_cleared = true;
        }

        if state.recovery_detected || observation.recovery_icon_detected {
            if !state.recovery_detected {
                state.recovery_detected = true;
                state.last_recovery_check_time_ms = Some(observation.now_ms);
                recovery_cache_updated = true;
            }
            recovery_available = true;

            if elapsed_ms_since(state.last_eat_time_ms, observation.now_ms)
                >= plan.config_rule.eat_interval_ms
            {
                actions.push(AutoEatTriggeredAction::Eat {
                    action: plan.detection_rule.eat_action.clone(),
                });
                state.last_eat_time_ms = Some(observation.now_ms);
            }
        } else {
            recovery_available = false;
        }
    }

    let resurrection_available = observation.resurrection_icon_detected;
    if resurrection_available
        && elapsed_ms_since(state.last_resurrection_time_ms, observation.now_ms)
            >= plan.detection_rule.resurrection_cooldown_ms
    {
        actions.push(AutoEatTriggeredAction::Resurrect {
            action: plan.detection_rule.resurrection_action.clone(),
        });
        state.last_resurrection_time_ms = Some(observation.now_ms);
    }

    AutoEatTickDecisionReport {
        processed: true,
        skip_reason: None,
        low_hp_detected: observation.current_avatar_low_hp,
        recovery_cache_cleared,
        recovery_cache_updated,
        recovery_available,
        resurrection_available,
        actions,
    }
}

fn auto_eat_locators(capture_size: Size) -> AutoEatLocators {
    AutoEatLocators {
        recovery_icon: template(
            "RecoveryIcon",
            AUTO_EAT_RECOVERY_ASSET,
            Rect {
                x: scaled(1810, capture_size),
                y: scaled(778, capture_size),
                width: scaled(23, capture_size),
                height: scaled(23, capture_size),
            },
        ),
        resurrection_icon: template(
            "ResurrectionIcon",
            AUTO_EAT_RESURRECTION_ASSET,
            Rect {
                x: scaled(1810, capture_size),
                y: scaled(778, capture_size),
                width: scaled(18, capture_size),
                height: scaled(19, capture_size),
            },
        ),
    }
}

fn auto_eat_steps() -> Vec<AutoEatTickStep> {
    vec![
        AutoEatTickStep::new(
            AutoEatTickPhase::Throttle,
            AutoEatTickCondition::WhenCheckIntervalNotElapsed,
            AutoEatTickAction::SkipTick,
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::LowHpRecovery,
            AutoEatTickCondition::WhenCurrentAvatarIsLowHp,
            AutoEatTickAction::DetectCurrentAvatarLowHp,
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::LowHpRecovery,
            AutoEatTickCondition::WhenRecoveryCacheExpired,
            AutoEatTickAction::ClearRecoveryCache,
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::LowHpRecovery,
            AutoEatTickCondition::WhenCurrentAvatarIsLowHp,
            AutoEatTickAction::DetectRecoveryIcon,
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::LowHpRecovery,
            AutoEatTickCondition::WhenRecoveryCachedOrDetected,
            AutoEatTickAction::SimulateGenshinAction {
                action: AUTO_EAT_QUICK_USE_GADGET_ACTION.to_string(),
            },
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::Resurrection,
            AutoEatTickCondition::WhenResurrectionIconDetected,
            AutoEatTickAction::DetectResurrectionIcon,
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::Resurrection,
            AutoEatTickCondition::WhenResurrectionCooldownElapsed,
            AutoEatTickAction::SimulateGenshinAction {
                action: AUTO_EAT_QUICK_USE_GADGET_ACTION.to_string(),
            },
        ),
        AutoEatTickStep::new(
            AutoEatTickPhase::ErrorHandling,
            AutoEatTickCondition::OnDetectionError,
            AutoEatTickAction::LogDebugAndContinue,
        ),
    ]
}

fn resolve_auto_eat_food_mode(config: &AutoEatFoodExecutionConfig) -> Result<AutoEatFoodPlanMode> {
    if config.food_name.is_some() && config.food_effect_type.is_some() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
            message: "不能同时指定foodName和foodEffectType".to_string(),
        });
    }

    if let Some(food_name) = config.food_name.clone() {
        return Ok(AutoEatFoodPlanMode::InventoryFood {
            food_name,
            source: AutoEatFoodNameSource::FoodNameConfig,
        });
    }

    let Some(effect_type) = config.food_effect_type.clone() else {
        return Ok(AutoEatFoodPlanMode::PortableNutritionBagLoop);
    };

    let (food_name, config_label) = match effect_type {
        AutoEatFoodEffectType::ATKBoostingDish => (
            config.default_atk_boosting_dish_name.clone(),
            "默认的攻击类料理",
        ),
        AutoEatFoodEffectType::AdventurersDish => (
            config.default_adventurers_dish_name.clone(),
            "默认的冒险类料理",
        ),
        AutoEatFoodEffectType::DEFBoostingDish => (
            config.default_def_boosting_dish_name.clone(),
            "默认的防御类料理",
        ),
        AutoEatFoodEffectType::RecoveryDish
        | AutoEatFoodEffectType::Potion
        | AutoEatFoodEffectType::Other => {
            return Err(TaskError::InvalidTaskConfig {
                key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
                message: "JS脚本入参错误：错误的foodEffectType".to_string(),
            });
        }
    };

    match food_name {
        Some(food_name) if !food_name.trim().is_empty() => Ok(AutoEatFoodPlanMode::InventoryFood {
            food_name: food_name.trim().to_string(),
            source: AutoEatFoodNameSource::FoodEffectDefault,
        }),
        _ => {
            let log_message = format!("缺少{config_label}配置，跳过吃Buff");
            Ok(AutoEatFoodPlanMode::MissingDefaultFood {
                effect_type,
                config_label: config_label.to_string(),
                log_message,
            })
        }
    }
}

fn auto_eat_food_steps(
    mode: &AutoEatFoodPlanMode,
    inventory_plan: Option<&CountInventoryItemExecutionPlan>,
    use_rule: &AutoEatFoodUseRule,
    result_contract: &AutoEatFoodResultContract,
) -> Vec<AutoEatFoodStep> {
    let mut steps = vec![AutoEatFoodStep::new(
        AutoEatFoodStepPhase::Setup,
        AutoEatFoodStepCondition::Always,
        AutoEatFoodStepAction::LogInfo {
            message: "自动吃药任务启动".to_string(),
        },
    )];

    match mode {
        AutoEatFoodPlanMode::InventoryFood { food_name, .. } => {
            let inventory_plan =
                inventory_plan.expect("inventory plan should exist for InventoryFood mode");
            steps.extend([
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::Setup,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::LogInfo {
                        message: format!("打开背包寻找{food_name}……"),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::OpenFoodInventory,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::ReturnMainUi,
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::OpenFoodInventory,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::OpenFoodInventoryTab {
                        rule: inventory_plan.open_inventory_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::ScanGrid,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::LoadGridIconClassifier {
                        rule: inventory_plan.classifier_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::ScanGrid,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::EnumerateGridItems {
                        template: inventory_plan.grid_template.clone(),
                        detection_rule: inventory_plan.grid_item_detection_rule.clone(),
                        scroll_rule: inventory_plan.scroll_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::ScanGrid,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::CropGridIcon {
                        rule: inventory_plan.grid_icon_crop_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::ScanGrid,
                    AutoEatFoodStepCondition::WhenInventoryFoodMode,
                    AutoEatFoodStepAction::InferGridIcon {
                        rule: inventory_plan.classifier_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::UseFood,
                    AutoEatFoodStepCondition::WhenClassifierMatchesFood,
                    AutoEatFoodStepAction::ClickMatchedFoodItem {
                        food_name: food_name.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::UseFood,
                    AutoEatFoodStepCondition::WhenClassifierMatchesFood,
                    AutoEatFoodStepAction::OcrMatchedFoodCount {
                        rule: inventory_plan.count_ocr_rule.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::UseFood,
                    AutoEatFoodStepCondition::WhenCountOcrFails,
                    AutoEatFoodStepAction::LogWarning {
                        message: "无法识别食物数量：{text}，依然尝试使用".to_string(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::UseFood,
                    AutoEatFoodStepCondition::WhenClassifierMatchesFood,
                    AutoEatFoodStepAction::DelayAfterItemClick {
                        delay_ms: use_rule.after_item_click_wait_ms,
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::UseFood,
                    AutoEatFoodStepCondition::WhenClassifierMatchesFood,
                    AutoEatFoodStepAction::ConfirmUseFoodIfVisible {
                        asset: use_rule.confirm_button_asset.clone(),
                    },
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::Cleanup,
                    AutoEatFoodStepCondition::Finally,
                    AutoEatFoodStepAction::ClearVisionDrawings,
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::Cleanup,
                    AutoEatFoodStepCondition::Always,
                    AutoEatFoodStepAction::ReturnMainUi,
                ),
                AutoEatFoodStep::new(
                    AutoEatFoodStepPhase::Result,
                    AutoEatFoodStepCondition::Always,
                    AutoEatFoodStepAction::ReturnResult {
                        contract: result_contract.clone(),
                    },
                ),
            ]);
        }
        AutoEatFoodPlanMode::PortableNutritionBagLoop => {
            steps.push(AutoEatFoodStep::new(
                AutoEatFoodStepPhase::Setup,
                AutoEatFoodStepCondition::WhenPortableNutritionBagMode,
                AutoEatFoodStepAction::PortableNutritionBagLoop,
            ));
        }
        AutoEatFoodPlanMode::MissingDefaultFood { log_message, .. } => {
            steps.push(AutoEatFoodStep::new(
                AutoEatFoodStepPhase::Result,
                AutoEatFoodStepCondition::WhenDefaultFoodMissing,
                AutoEatFoodStepAction::LogInfo {
                    message: log_message.clone(),
                },
            ));
        }
    }

    steps
}

fn template(name: &str, asset: &str, roi: Rect) -> AutoEatTemplateLocator {
    AutoEatTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold: 0.8,
        match_mode: TemplateMatchMode::CCoeffNormed,
        use_3_channels: false,
        draw_on_window: false,
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / AUTO_EAT_DEFAULT_CAPTURE_WIDTH as i64) as i32
}

fn elapsed_ms_since(previous_ms: Option<u64>, now_ms: u64) -> u64 {
    previous_ms
        .map(|previous| now_ms.saturating_sub(previous))
        .unwrap_or(u64::MAX)
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

fn auto_eat_food_config_sources(value: &Value) -> Vec<&Value> {
    let mut sources = vec![value];
    for key in [
        "taskParam",
        "TaskParam",
        "task_param",
        "param",
        "Param",
        "config",
        "Config",
    ] {
        if let Some(source) = value.get(key) {
            sources.push(source);
        }
    }
    sources
}

fn optional_string_from_sources(sources: &[&Value], keys: &[&str]) -> Option<Option<String>> {
    for source in sources {
        for key in keys {
            let Some(value) = source.get(*key) else {
                continue;
            };
            if value.is_null() {
                return Some(None);
            }
            if let Some(text) = value.as_str() {
                let text = text.trim();
                return Some((!text.is_empty()).then(|| text.to_string()));
            }
        }
    }
    None
}

fn optional_food_effect_type_from_sources(
    sources: &[&Value],
    keys: &[&str],
) -> Result<Option<AutoEatFoodEffectType>> {
    for source in sources {
        for key in keys {
            let Some(value) = source.get(*key) else {
                continue;
            };
            if value.is_null() {
                return Ok(None);
            }
            if let Some(index) = value.as_i64() {
                return AutoEatFoodEffectType::from_legacy_index(index)
                    .map(Some)
                    .ok_or_else(|| TaskError::InvalidTaskConfig {
                        key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
                        message: "JS脚本入参错误：错误的foodEffectType".to_string(),
                    });
            }
            if let Some(name) = value.as_str() {
                return AutoEatFoodEffectType::from_legacy_name(name)
                    .map(Some)
                    .ok_or_else(|| TaskError::InvalidTaskConfig {
                        key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
                        message: "JS脚本入参错误：错误的foodEffectType".to_string(),
                    });
            }
            return Err(TaskError::InvalidTaskConfig {
                key: AUTO_EAT_FOOD_TASK_KEY.to_string(),
                message: "JS脚本入参错误：错误的foodEffectType".to_string(),
            });
        }
    }
    Ok(None)
}
