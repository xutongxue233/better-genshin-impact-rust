use super::{image_locator, task_vision_result, RETURN_MAIN_UI_TASK_KEY, TELEPORT_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, MouseButton};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GO_TO_SERENITEA_POT_TASK_KEY: &str = "GoToSereniteaPot";
pub const GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME: &str = "默认配置";
pub const GO_TO_SERENITEA_POT_MAP_TP_TYPE: &str = "地图传送";
pub const GO_TO_SERENITEA_POT_BAG_TP_TYPE: &str = "尘歌壶道具";
pub const GO_TO_SERENITEA_POT_AREA_NAME: &str = "尘歌壶";
pub const GO_TO_SERENITEA_POT_FINAL_TP_X: f64 = 4508.97509765625;
pub const GO_TO_SERENITEA_POT_FINAL_TP_Y: f64 = 3630.557373046875;
pub const GO_TO_SERENITEA_POT_ONE_DRAGON_FOLDER: &str = "User/OneDragon";

pub const GO_TO_SERENITEA_POT_HOME: &str = "Common/Element:sereniteapot_home.png";
pub const GO_TO_SERENITEA_POT_TELEPORT_HOME: &str = "Common/Element:sereniteapot_home.png";
pub const GO_TO_SERENITEA_POT_TELEPORT_BUTTON: &str = "QuickTeleport:GoTeleport.png";
pub const GO_TO_SERENITEA_POT_BAG_CLOSE_BUTTON: &str = "QuickTeleport:MapCloseButton.png";
pub const GO_TO_SERENITEA_POT_ICON: &str = "QuickSereniteaPot:SereniteaPotIcon.png";
pub const GO_TO_SERENITEA_POT_FINGER: &str = "Common/Element:finger.png";
pub const GO_TO_SERENITEA_POT_PAGE_CLOSE_WHITE: &str = "Common/Element:page_close_white.png";
pub const GO_TO_SERENITEA_POT_POT_PAGE_CLOSE: &str = "Common/Element:sereniteapot_page_close.png";
pub const GO_TO_SERENITEA_POT_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const GO_TO_SERENITEA_POT_LOVE: &str = "Common/Element:sereniteapot_love.png";
pub const GO_TO_SERENITEA_POT_MONEY: &str = "Common/Element:sereniteapot_money.png";

const DONG_TIAN_NAME_ATTEMPTS: u8 = 5;
const MAP_HOME_ZOOM_ATTEMPTS: u8 = 5;
const MAP_TELEPORT_ATTEMPTS: u8 = 10;
const BAG_MAIN_UI_WAIT_MS: u32 = 5_000;
const FIND_AYUAN_FAIL_COUNT: u16 = 180;
const FIND_AYUAN_MISSING_ROTATE_WIDTH_RATIO: f64 = 0.1;
const FIND_AYUAN_TARGET_Y_RATIO: f64 = 0.25;
const FIND_AYUAN_TARGET_Y_OFFSET: i32 = 100;
const FIND_AYUAN_HORIZONTAL_TOLERANCE_WIDTH_MULTIPLIER: f64 = 1.4;
const FIND_AYUAN_DROP_INTERVAL_MS: u32 = 50;
const SHOP_PURCHASE_RETRIES: u8 = 2;
const SHOP_ITEM_AFTER_CLICK_MS: u32 = 600;
const SHOP_BUY_ANIMATION_MS: u32 = 1_000;
const SHOP_ITEM_MISSING_DELAY_MS: u32 = 700;
const SHOP_BETWEEN_ITEMS_DELAY_MS: u32 = 700;
const SHOP_SOLD_OUT_TEXT: &str = "已售";
const NO_COMPANION_EXP_TEXT: &str = "无法领取好感经验";
const COMPANION_AVAILABLE_REGEX: &str = r"(\d+)\s*[/17]\s*(8)";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub selected_config_name: String,
    pub legacy_tp_type: String,
    pub entry_mode: GoToSereniteaPotEntryMode,
    pub secret_treasure_objects: Vec<String>,
    pub localized_texts: GoToSereniteaPotLocalizedTexts,
    pub locators: GoToSereniteaPotLocators,
    pub config_rule: GoToSereniteaPotConfigRule,
    pub map_entry_rule: GoToSereniteaPotMapEntryRule,
    pub bag_entry_rule: GoToSereniteaPotBagEntryRule,
    pub find_ayuan_rule: GoToSereniteaPotFindAYuanRule,
    pub reward_rule: GoToSereniteaPotRewardRule,
    pub shop_rule: GoToSereniteaPotShopRule,
    pub finish_rule: GoToSereniteaPotFinishRule,
    pub steps: Vec<GoToSereniteaPotStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotEntryMode {
    MapTeleport,
    BagGadget,
}

impl GoToSereniteaPotEntryMode {
    fn from_legacy(value: &str) -> Self {
        if value.trim() == GO_TO_SERENITEA_POT_MAP_TP_TYPE {
            Self::MapTeleport
        } else {
            Self::BagGadget
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotLocalizedTexts {
    pub ayuan: String,
    pub spirit: String,
    pub spirit_bracketed: String,
    pub trust_rank: String,
    pub realm_depot: String,
    pub goodbye: String,
    pub sold_out: String,
    pub no_companion_exp: String,
}

impl Default for GoToSereniteaPotLocalizedTexts {
    fn default() -> Self {
        Self {
            ayuan: "阿圆".to_string(),
            spirit: "壶灵".to_string(),
            spirit_bracketed: "<壶灵>".to_string(),
            trust_rank: "信任".to_string(),
            realm_depot: "洞天百宝".to_string(),
            goodbye: "再见".to_string(),
            sold_out: SHOP_SOLD_OUT_TEXT.to_string(),
            no_companion_exp: NO_COMPANION_EXP_TEXT.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotLocators {
    pub serenitea_pot_home: BvLocatorPlan,
    pub teleport_serenitea_pot_home: BvLocatorPlan,
    pub teleport_button: BvLocatorPlan,
    pub bag_close_button: BvLocatorPlan,
    pub serenitea_pot_icon: BvLocatorPlan,
    pub finger_icon: BvLocatorPlan,
    pub page_close_white: BvLocatorPlan,
    pub pot_page_close: BvLocatorPlan,
    pub white_confirm: BvLocatorPlan,
    pub love: BvLocatorPlan,
    pub money: BvLocatorPlan,
    pub shop_items: Vec<GoToSereniteaPotShopItemLocator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotShopItemLocator {
    pub item: GoToSereniteaPotShopItem,
    pub asset: String,
    pub locator: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotShopItem {
    Cloth,
    TransientResin,
    HeroWit,
    AdventurersExperience,
    MysticEnhancementOre,
    Mora,
    SanctifyingEssence,
    SanctifyingUnction,
}

impl GoToSereniteaPotShopItem {
    pub fn legacy_name(self) -> &'static str {
        match self {
            Self::Cloth => "布匹",
            Self::TransientResin => "须臾树脂",
            Self::HeroWit => "大英雄的经验",
            Self::AdventurersExperience => "流浪者的经验",
            Self::MysticEnhancementOre => "精锻用魔矿",
            Self::Mora => "摩拉",
            Self::SanctifyingEssence => "祝圣精华",
            Self::SanctifyingUnction => "祝圣油膏",
        }
    }

    pub fn asset(self) -> &'static str {
        match self {
            Self::Cloth => "Common/Element:ayuan_cloth.png",
            Self::TransientResin => "Common/Element:ayuan_resin.png",
            Self::HeroWit => "Common/Element:exp_book.png",
            Self::AdventurersExperience => "Common/Element:exp_book_small.png",
            Self::MysticEnhancementOre => "Common/Element:ayuan_magicmineralprecision.png",
            Self::Mora => "Common/Element:ayuan_mola.png",
            Self::SanctifyingEssence => "Common/Element:exp_bottle_big.png",
            Self::SanctifyingUnction => "Common/Element:exp_bottle_small.png",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotConfigRule {
    pub one_dragon_config_folder: String,
    pub selected_config_name: String,
    pub default_config_name: String,
    pub fallback_to_first_config: bool,
    pub fallback_to_default_config: bool,
    pub json_serializer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotMapEntryRule {
    pub return_main_ui_task_key: String,
    pub open_map_wait_ms: u32,
    pub area_name: String,
    pub dong_tian_name_ocr: GoToSereniteaPotOcrRule,
    pub home_locator: BvLocatorPlan,
    pub home_zoom_attempts: u8,
    pub zoom_start_level: f64,
    pub zoom_level_step: f64,
    pub wait_before_zoom_ms: u32,
    pub wait_after_zoom_ms: u32,
    pub teleport_button_locator: BvLocatorPlan,
    pub teleport_home_locator: BvLocatorPlan,
    pub teleport_attempts: u8,
    pub wait_before_teleport_click_ms: u32,
    pub wait_after_teleport_click_ms: u32,
    pub teleport_button_disappear_checks: u8,
    pub teleport_button_disappear_wait_ms: u32,
    pub wait_after_home_click_ms: u32,
    pub wait_retry_ms: u32,
    pub wait_main_ui_after_teleport: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotBagEntryRule {
    pub quick_serenitea_pot_task: String,
    pub wait_after_quick_task_ms: u32,
    pub wait_main_ui: bool,
    pub finger_locator: BvLocatorPlan,
    pub open_map_wait_ms: u32,
    pub dong_tian_name_ocr: GoToSereniteaPotOcrRule,
    pub close_map_toggle_attempts: u8,
    pub close_map_toggle_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotFindAYuanRule {
    pub realm_adjustments: Vec<GoToSereniteaPotRealmAdjustment>,
    pub middle_click_events: Vec<InputEvent>,
    pub wait_after_middle_click_ms: u32,
    pub search_ocr: GoToSereniteaPotOcrRule,
    pub accepted_texts: Vec<String>,
    pub target_y_max_ratio: f64,
    pub target_y_offset_px: i32,
    pub horizontal_tolerance_width_multiplier: f64,
    pub missing_rotate_width_ratio: f64,
    pub align_delay_ms: u32,
    pub blur_safe_delay_ms: u32,
    pub max_missing_rotations: u16,
    pub approach_action: GenshinAction,
    pub approach_dialog_text: String,
    pub drop_action: GenshinAction,
    pub drop_interval_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotRealmAdjustment {
    pub realm_name: String,
    pub actions: Vec<GoToSereniteaPotTimedAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotTimedAction {
    pub action: GoToSereniteaPotTimedActionKind,
    pub hold_ms: Option<u32>,
    pub wait_after_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GoToSereniteaPotTimedActionKind {
    GenshinAction { action: GenshinAction },
    MouseMiddleClick,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotRewardRule {
    pub start_dialog_text: String,
    pub trust_option_text: String,
    pub after_trust_click_ms: u32,
    pub companion_available_ocr_crop: GoToSereniteaPotRelativeCrop,
    pub companion_available_regex: String,
    pub love_locator: BvLocatorPlan,
    pub no_companion_exp_ocr: GoToSereniteaPotOcrRule,
    pub no_companion_exp_text: String,
    pub pot_page_close_locator: BvLocatorPlan,
    pub money_locator: BvLocatorPlan,
    pub page_close_white_locator: BvLocatorPlan,
    pub reward_click_delay_ms: u32,
    pub after_reward_flow_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotShopRule {
    pub configured_objects: Vec<String>,
    pub day_selector_index: usize,
    pub daily_repeat_label: String,
    pub server_day_start_hour: u8,
    pub valid_day_labels: Vec<GoToSereniteaPotShopDay>,
    pub shop_option_text: String,
    pub after_shop_option_click_ms: u32,
    pub items: Vec<GoToSereniteaPotShopItemLocator>,
    pub sold_out_ocr: GoToSereniteaPotOcrRule,
    pub sold_out_text: String,
    pub purchase_retries: u8,
    pub after_item_click_ms: u32,
    pub buy_max_rule: GoToSereniteaPotBuyMaxRule,
    pub after_buy_animation_ms: u32,
    pub item_missing_delay_ms: u32,
    pub between_items_delay_ms: u32,
    pub close_after_purchase_locator: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToSereniteaPotShopDay {
    pub label: String,
    pub day_of_week: GoToSereniteaPotDayOfWeek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotDayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotBuyMaxRule {
    pub sold_out_ocr: GoToSereniteaPotOcrRule,
    pub white_confirm_locator: BvLocatorPlan,
    pub after_confirm_ms: u32,
    pub escape_after_confirm: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotFinishRule {
    pub page_close_white_locator: BvLocatorPlan,
    pub goodbye_option_text: String,
    pub goodbye_skip_times: u32,
    pub wait_after_goodbye_ms: u32,
    pub click_until_main_ui: bool,
    pub teleport_task_key: String,
    pub final_teleport_x: f64,
    pub final_teleport_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotOcrRule {
    pub roi: Rect,
    pub recognition_type: GoToSereniteaPotRecognitionType,
    pub attempts: u8,
    pub retry_delay_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotRecognitionType {
    Ocr,
    OcrWithoutDetector,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotRelativeCrop {
    pub x_ratio: f64,
    pub y_ratio: f64,
    pub width_ratio: f64,
    pub height_from_width_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToSereniteaPotStep {
    pub phase: GoToSereniteaPotStepPhase,
    pub condition: GoToSereniteaPotStepCondition,
    pub label: String,
    pub action: GoToSereniteaPotStepAction,
}

impl GoToSereniteaPotStep {
    fn new(
        phase: GoToSereniteaPotStepPhase,
        condition: GoToSereniteaPotStepCondition,
        label: impl Into<String>,
        action: GoToSereniteaPotStepAction,
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
pub enum GoToSereniteaPotStepPhase {
    Config,
    EnterPot,
    FindAYuan,
    Reward,
    Shop,
    Finish,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotStepCondition {
    Always,
    WhenMapTeleportConfigured,
    WhenBagGadgetConfigured,
    WhenEntryFailed,
    WhenEntrySucceeded,
    WhenAYuanFound,
    WhenAYuanMissing,
    WhenTrustRewardAvailable,
    WhenShopConfiguredAndDue,
    WhenShopMissingOrNotDue,
    Finally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotActionPress {
    KeyDown,
    KeyUp,
    KeyPress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GoToSereniteaPotStepAction {
    Log {
        message: String,
    },
    CommonJob {
        task_key: String,
        config: Option<Value>,
    },
    GenshinAction {
        action: GenshinAction,
        press: GoToSereniteaPotActionPress,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Page {
        command: BvPageCommand,
    },
    MapEntry {
        rule: GoToSereniteaPotMapEntryRule,
    },
    BagEntry {
        rule: GoToSereniteaPotBagEntryRule,
    },
    FindAYuan {
        rule: GoToSereniteaPotFindAYuanRule,
    },
    Reward {
        rule: GoToSereniteaPotRewardRule,
    },
    ShopPurchase {
        rule: GoToSereniteaPotShopRule,
    },
    Finish {
        rule: GoToSereniteaPotFinishRule,
    },
    ReleaseAllKeys,
    ClearVisionDrawings,
    ReturnResult {
        result: GoToSereniteaPotStepResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToSereniteaPotStepResult {
    Completed,
    EntryFailed,
    AYuanNotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoToSereniteaPotExecutionConfig {
    pub capture_size: Size,
    pub selected_config_name: String,
    pub serenitea_pot_tp_type: String,
    pub secret_treasure_objects: Vec<String>,
    pub localized_texts: GoToSereniteaPotLocalizedTexts,
}

impl Default for GoToSereniteaPotExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            selected_config_name: GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME.to_string(),
            serenitea_pot_tp_type: GO_TO_SERENITEA_POT_MAP_TP_TYPE.to_string(),
            secret_treasure_objects: Vec::new(),
            localized_texts: GoToSereniteaPotLocalizedTexts::default(),
        }
    }
}

impl GoToSereniteaPotExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.selected_config_name.trim().is_empty() {
            config.selected_config_name = GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME.to_string();
        }
        if config.serenitea_pot_tp_type.trim().is_empty() {
            config.serenitea_pot_tp_type = GO_TO_SERENITEA_POT_MAP_TP_TYPE.to_string();
        }
        config
    }
}

pub fn plan_go_to_serenitea_pot(
    config: GoToSereniteaPotExecutionConfig,
) -> Result<GoToSereniteaPotExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let locators = plan_locators(&page, config.capture_size)?;
    let config_rule = GoToSereniteaPotConfigRule {
        one_dragon_config_folder: GO_TO_SERENITEA_POT_ONE_DRAGON_FOLDER.to_string(),
        selected_config_name: config.selected_config_name.clone(),
        default_config_name: GO_TO_SERENITEA_POT_DEFAULT_CONFIG_NAME.to_string(),
        fallback_to_first_config: true,
        fallback_to_default_config: true,
        json_serializer: "Newtonsoft.Json".to_string(),
    };
    let dong_tian_name_ocr = GoToSereniteaPotOcrRule {
        roi: ratio_rect(config.capture_size, 0.86, 0.9, 0.073, 0.04)?,
        recognition_type: GoToSereniteaPotRecognitionType::Ocr,
        attempts: DONG_TIAN_NAME_ATTEMPTS,
        retry_delay_ms: 100,
    };
    let map_entry_rule = GoToSereniteaPotMapEntryRule {
        return_main_ui_task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        open_map_wait_ms: 900,
        area_name: GO_TO_SERENITEA_POT_AREA_NAME.to_string(),
        dong_tian_name_ocr: dong_tian_name_ocr.clone(),
        home_locator: locators.serenitea_pot_home.clone(),
        home_zoom_attempts: MAP_HOME_ZOOM_ATTEMPTS,
        zoom_start_level: 2.5,
        zoom_level_step: -0.2,
        wait_before_zoom_ms: 1_000,
        wait_after_zoom_ms: 1_000,
        teleport_button_locator: locators.teleport_button.clone(),
        teleport_home_locator: locators.teleport_serenitea_pot_home.clone(),
        teleport_attempts: MAP_TELEPORT_ATTEMPTS,
        wait_before_teleport_click_ms: 300,
        wait_after_teleport_click_ms: 500,
        teleport_button_disappear_checks: 10,
        teleport_button_disappear_wait_ms: 500,
        wait_after_home_click_ms: 800,
        wait_retry_ms: 800,
        wait_main_ui_after_teleport: true,
    };
    let bag_entry_rule = GoToSereniteaPotBagEntryRule {
        quick_serenitea_pot_task: "QuickSereniteaPotTask.Done".to_string(),
        wait_after_quick_task_ms: BAG_MAIN_UI_WAIT_MS,
        wait_main_ui: true,
        finger_locator: locators.finger_icon.clone(),
        open_map_wait_ms: 1_000,
        dong_tian_name_ocr: dong_tian_name_ocr.clone(),
        close_map_toggle_attempts: 4,
        close_map_toggle_wait_ms: 1_000,
    };
    let find_ayuan_rule = GoToSereniteaPotFindAYuanRule {
        realm_adjustments: realm_adjustments(),
        middle_click_events: middle_click_events(),
        wait_after_middle_click_ms: 900,
        search_ocr: GoToSereniteaPotOcrRule {
            roi: task_vision_result(Rect::new(
                (config.capture_size.width / 5) as i32,
                (config.capture_size.height / 15) as i32,
                (config.capture_size.width as f64 * 0.65).round() as i32,
                (config.capture_size.height / 2) as i32,
            ))?,
            recognition_type: GoToSereniteaPotRecognitionType::Ocr,
            attempts: 1,
            retry_delay_ms: 0,
        },
        accepted_texts: vec![
            config.localized_texts.ayuan.clone(),
            config.localized_texts.spirit.clone(),
            config.localized_texts.spirit_bracketed.clone(),
        ],
        target_y_max_ratio: FIND_AYUAN_TARGET_Y_RATIO,
        target_y_offset_px: FIND_AYUAN_TARGET_Y_OFFSET,
        horizontal_tolerance_width_multiplier: FIND_AYUAN_HORIZONTAL_TOLERANCE_WIDTH_MULTIPLIER,
        missing_rotate_width_ratio: FIND_AYUAN_MISSING_ROTATE_WIDTH_RATIO,
        align_delay_ms: 300,
        blur_safe_delay_ms: 500,
        max_missing_rotations: FIND_AYUAN_FAIL_COUNT,
        approach_action: GenshinAction::MoveForward,
        approach_dialog_text: config.localized_texts.ayuan.clone(),
        drop_action: GenshinAction::Drop,
        drop_interval_ms: FIND_AYUAN_DROP_INTERVAL_MS,
    };
    let sold_out_ocr = GoToSereniteaPotOcrRule {
        roi: ratio_rect(config.capture_size, 0.7, 0.35, 0.2, 0.15)?,
        recognition_type: GoToSereniteaPotRecognitionType::Ocr,
        attempts: 1,
        retry_delay_ms: 0,
    };
    let reward_rule = GoToSereniteaPotRewardRule {
        start_dialog_text: config.localized_texts.ayuan.clone(),
        trust_option_text: config.localized_texts.trust_rank.clone(),
        after_trust_click_ms: 1_000,
        companion_available_ocr_crop: GoToSereniteaPotRelativeCrop {
            x_ratio: 1801.0 / 1920.0,
            y_ratio: 609.0 / 1080.0,
            width_ratio: 75.0 / 1920.0,
            height_from_width_ratio: 46.0 / 1920.0,
        },
        companion_available_regex: COMPANION_AVAILABLE_REGEX.to_string(),
        love_locator: locators.love.clone(),
        no_companion_exp_ocr: GoToSereniteaPotOcrRule {
            roi: ratio_rect(config.capture_size, 0.35, 0.45, 0.3, 0.05)?,
            recognition_type: GoToSereniteaPotRecognitionType::Ocr,
            attempts: 1,
            retry_delay_ms: 0,
        },
        no_companion_exp_text: config.localized_texts.no_companion_exp.clone(),
        pot_page_close_locator: locators.pot_page_close.clone(),
        money_locator: locators.money.clone(),
        page_close_white_locator: locators.page_close_white.clone(),
        reward_click_delay_ms: 500,
        after_reward_flow_ms: 900,
    };
    let shop_rule = GoToSereniteaPotShopRule {
        configured_objects: config.secret_treasure_objects.clone(),
        day_selector_index: 0,
        daily_repeat_label: "每天重复".to_string(),
        server_day_start_hour: 4,
        valid_day_labels: shop_days(),
        shop_option_text: config.localized_texts.realm_depot.clone(),
        after_shop_option_click_ms: 500,
        items: locators.shop_items.clone(),
        sold_out_ocr: sold_out_ocr.clone(),
        sold_out_text: config.localized_texts.sold_out.clone(),
        purchase_retries: SHOP_PURCHASE_RETRIES,
        after_item_click_ms: SHOP_ITEM_AFTER_CLICK_MS,
        buy_max_rule: GoToSereniteaPotBuyMaxRule {
            sold_out_ocr,
            white_confirm_locator: locators.white_confirm.clone(),
            after_confirm_ms: 600,
            escape_after_confirm: true,
        },
        after_buy_animation_ms: SHOP_BUY_ANIMATION_MS,
        item_missing_delay_ms: SHOP_ITEM_MISSING_DELAY_MS,
        between_items_delay_ms: SHOP_BETWEEN_ITEMS_DELAY_MS,
        close_after_purchase_locator: locators.page_close_white.clone(),
    };
    let finish_rule = GoToSereniteaPotFinishRule {
        page_close_white_locator: locators.page_close_white.clone(),
        goodbye_option_text: config.localized_texts.goodbye.clone(),
        goodbye_skip_times: 20,
        wait_after_goodbye_ms: 300,
        click_until_main_ui: true,
        teleport_task_key: TELEPORT_TASK_KEY.to_string(),
        final_teleport_x: GO_TO_SERENITEA_POT_FINAL_TP_X,
        final_teleport_y: GO_TO_SERENITEA_POT_FINAL_TP_Y,
    };
    let steps = vec![
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Config,
            GoToSereniteaPotStepCondition::Always,
            "resolve selected OneDragon config",
            GoToSereniteaPotStepAction::Log {
                message: format!(
                    "resolve OneDragon config {} from {}",
                    config_rule.selected_config_name, config_rule.one_dragon_config_folder
                ),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::EnterPot,
            GoToSereniteaPotStepCondition::WhenMapTeleportConfigured,
            "enter Serenitea Pot through map teleport",
            GoToSereniteaPotStepAction::MapEntry {
                rule: map_entry_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::EnterPot,
            GoToSereniteaPotStepCondition::WhenBagGadgetConfigured,
            "enter Serenitea Pot through bag gadget",
            GoToSereniteaPotStepAction::BagEntry {
                rule: bag_entry_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Finish,
            GoToSereniteaPotStepCondition::WhenEntryFailed,
            "finish safely when entering the Serenitea Pot fails",
            GoToSereniteaPotStepAction::Finish {
                rule: finish_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::FindAYuan,
            GoToSereniteaPotStepCondition::WhenEntrySucceeded,
            "find and approach A Yuan",
            GoToSereniteaPotStepAction::FindAYuan {
                rule: find_ayuan_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Finish,
            GoToSereniteaPotStepCondition::WhenAYuanMissing,
            "finish safely when A Yuan search fails",
            GoToSereniteaPotStepAction::Finish {
                rule: finish_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Reward,
            GoToSereniteaPotStepCondition::WhenAYuanFound,
            "claim trust-rank companion and realm currency rewards",
            GoToSereniteaPotStepAction::Reward {
                rule: reward_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Shop,
            GoToSereniteaPotStepCondition::WhenShopConfiguredAndDue,
            "buy configured realm depot items",
            GoToSereniteaPotStepAction::ShopPurchase {
                rule: shop_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Shop,
            GoToSereniteaPotStepCondition::WhenShopMissingOrNotDue,
            "skip realm depot purchase",
            GoToSereniteaPotStepAction::Log {
                message:
                    "skip Serenitea Pot shop purchase when no item is configured or day is not due"
                        .to_string(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Finish,
            GoToSereniteaPotStepCondition::Always,
            "exit A Yuan dialog and teleport back to Teyvat",
            GoToSereniteaPotStepAction::Finish {
                rule: finish_rule.clone(),
            },
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Cleanup,
            GoToSereniteaPotStepCondition::Finally,
            "release all keys",
            GoToSereniteaPotStepAction::ReleaseAllKeys,
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Cleanup,
            GoToSereniteaPotStepCondition::Finally,
            "clear vision drawings",
            GoToSereniteaPotStepAction::ClearVisionDrawings,
        ),
        GoToSereniteaPotStep::new(
            GoToSereniteaPotStepPhase::Cleanup,
            GoToSereniteaPotStepCondition::Finally,
            "return completion",
            GoToSereniteaPotStepAction::ReturnResult {
                result: GoToSereniteaPotStepResult::Completed,
            },
        ),
    ];

    Ok(GoToSereniteaPotExecutionPlan {
        task_key: GO_TO_SERENITEA_POT_TASK_KEY.to_string(),
        display_name: "Go To Serenitea Pot".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        selected_config_name: config.selected_config_name,
        legacy_tp_type: config.serenitea_pot_tp_type.clone(),
        entry_mode: GoToSereniteaPotEntryMode::from_legacy(&config.serenitea_pot_tp_type),
        secret_treasure_objects: config.secret_treasure_objects,
        localized_texts: config.localized_texts,
        locators,
        config_rule,
        map_entry_rule,
        bag_entry_rule,
        find_ayuan_rule,
        reward_rule,
        shop_rule,
        finish_rule,
        steps,
        notes: "Serenitea Pot entry, A Yuan search, reward claim, realm-depot purchase, and final teleport are represented and executable as a Rust state machine through injectable hooks; direct map entry and real-game OCR/click regression remain pending."
            .to_string(),
    })
}

fn plan_locators(page: &BvPage, capture_size: Size) -> Result<GoToSereniteaPotLocators> {
    Ok(GoToSereniteaPotLocators {
        serenitea_pot_home: image_locator(
            page,
            GO_TO_SERENITEA_POT_HOME,
            Some(full_rect(capture_size)?),
            0.8,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        teleport_serenitea_pot_home: image_locator(
            page,
            GO_TO_SERENITEA_POT_TELEPORT_HOME,
            Some(bottom_right_half_rect(capture_size)?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        teleport_button: image_locator(
            page,
            GO_TO_SERENITEA_POT_TELEPORT_BUTTON,
            Some(scaled_rect(capture_size, 1440, 960, 100, 120)?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        bag_close_button: image_locator(
            page,
            GO_TO_SERENITEA_POT_BAG_CLOSE_BUTTON,
            Some(scaled_rect(capture_size, 1813, 19, 58, 58)?),
            0.8,
            BvLocatorOperation::IsExist,
            Some(500),
        )?,
        serenitea_pot_icon: image_locator(
            page,
            GO_TO_SERENITEA_POT_ICON,
            Some(scaled_rect(capture_size, 100, 100, 1190, 860)?),
            0.8,
            BvLocatorOperation::Click,
            Some(600),
        )?,
        finger_icon: image_locator(
            page,
            GO_TO_SERENITEA_POT_FINGER,
            Some(scaled_rect(capture_size, 1320, 0, 80, 80)?),
            0.8,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        page_close_white: image_locator(
            page,
            GO_TO_SERENITEA_POT_PAGE_CLOSE_WHITE,
            Some(task_vision_result(Rect::new(
                (capture_size.width - capture_size.width / 8) as i32,
                0,
                (capture_size.width / 8) as i32,
                (capture_size.height / 8) as i32,
            ))?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        pot_page_close: image_locator(
            page,
            GO_TO_SERENITEA_POT_POT_PAGE_CLOSE,
            Some(ratio_rect(capture_size, 0.5, 0.2, 0.25, 0.125)?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        white_confirm: image_locator(
            page,
            GO_TO_SERENITEA_POT_WHITE_CONFIRM,
            None,
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        love: image_locator(
            page,
            GO_TO_SERENITEA_POT_LOVE,
            Some(task_vision_result(Rect::new(
                (capture_size.width - capture_size.width / 8) as i32,
                (capture_size.height / 2) as i32,
                (capture_size.width / 8) as i32,
                (capture_size.height / 4) as i32,
            ))?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        money: image_locator(
            page,
            GO_TO_SERENITEA_POT_MONEY,
            Some(task_vision_result(Rect::new(
                (capture_size.width / 2) as i32,
                (capture_size.height - capture_size.height / 4) as i32,
                (capture_size.width / 4) as i32,
                (capture_size.height / 4) as i32,
            ))?),
            0.8,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        shop_items: shop_item_locators(page, capture_size)?,
    })
}

fn shop_item_locators(
    page: &BvPage,
    capture_size: Size,
) -> Result<Vec<GoToSereniteaPotShopItemLocator>> {
    let roi = task_vision_result(Rect::new(
        0,
        0,
        (capture_size.width * 7 / 10) as i32,
        capture_size.height as i32,
    ))?;
    [
        GoToSereniteaPotShopItem::Cloth,
        GoToSereniteaPotShopItem::TransientResin,
        GoToSereniteaPotShopItem::HeroWit,
        GoToSereniteaPotShopItem::AdventurersExperience,
        GoToSereniteaPotShopItem::MysticEnhancementOre,
        GoToSereniteaPotShopItem::Mora,
        GoToSereniteaPotShopItem::SanctifyingEssence,
        GoToSereniteaPotShopItem::SanctifyingUnction,
    ]
    .into_iter()
    .map(|item| {
        Ok(GoToSereniteaPotShopItemLocator {
            item,
            asset: item.asset().to_string(),
            locator: image_locator(
                page,
                item.asset(),
                Some(roi),
                0.8,
                BvLocatorOperation::Click,
                Some(1_000),
            )?,
        })
    })
    .collect()
}

fn realm_adjustments() -> Vec<GoToSereniteaPotRealmAdjustment> {
    vec![
        GoToSereniteaPotRealmAdjustment {
            realm_name: "妙香林".to_string(),
            actions: vec![timed_genshin_action(GenshinAction::MoveForward, 200, 0)],
        },
        GoToSereniteaPotRealmAdjustment {
            realm_name: "清琼岛".to_string(),
            actions: vec![
                timed_genshin_action(GenshinAction::MoveLeft, 100, 300),
                timed_middle_click(500),
            ],
        },
        GoToSereniteaPotRealmAdjustment {
            realm_name: "绘绮庭".to_string(),
            actions: vec![
                timed_genshin_action(GenshinAction::MoveLeft, 1_300, 500),
                timed_genshin_action(GenshinAction::MoveBackward, 600, 300),
                timed_middle_click(800),
            ],
        },
        GoToSereniteaPotRealmAdjustment {
            realm_name: "旋流屿".to_string(),
            actions: vec![
                timed_genshin_action(GenshinAction::MoveBackward, 900, 300),
                timed_middle_click(800),
            ],
        },
    ]
}

fn timed_genshin_action(
    action: GenshinAction,
    hold_ms: u32,
    wait_after_ms: u32,
) -> GoToSereniteaPotTimedAction {
    GoToSereniteaPotTimedAction {
        action: GoToSereniteaPotTimedActionKind::GenshinAction { action },
        hold_ms: Some(hold_ms),
        wait_after_ms,
    }
}

fn timed_middle_click(wait_after_ms: u32) -> GoToSereniteaPotTimedAction {
    GoToSereniteaPotTimedAction {
        action: GoToSereniteaPotTimedActionKind::MouseMiddleClick,
        hold_ms: None,
        wait_after_ms,
    }
}

fn middle_click_events() -> Vec<InputEvent> {
    vec![
        InputEvent::MouseButtonDown {
            button: MouseButton::Middle,
        },
        InputEvent::MouseButtonUp {
            button: MouseButton::Middle,
        },
    ]
}

fn shop_days() -> Vec<GoToSereniteaPotShopDay> {
    vec![
        GoToSereniteaPotShopDay {
            label: "星期一".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Monday,
        },
        GoToSereniteaPotShopDay {
            label: "星期二".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Tuesday,
        },
        GoToSereniteaPotShopDay {
            label: "星期三".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Wednesday,
        },
        GoToSereniteaPotShopDay {
            label: "星期四".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Thursday,
        },
        GoToSereniteaPotShopDay {
            label: "星期五".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Friday,
        },
        GoToSereniteaPotShopDay {
            label: "星期六".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Saturday,
        },
        GoToSereniteaPotShopDay {
            label: "星期日".to_string(),
            day_of_week: GoToSereniteaPotDayOfWeek::Sunday,
        },
    ]
}

fn ratio_rect(size: Size, x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width as f64 * x).round() as i32,
        (size.height as f64 * y).round() as i32,
        (size.width as f64 * width).round() as i32,
        (size.height as f64 * height).round() as i32,
    ))
}

fn bottom_right_half_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        (size.height / 2) as i32,
        (size.width / 2) as i32,
        (size.height / 2) as i32,
    ))
}

fn full_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(0, 0, size.width as i32, size.height as i32))
}

fn scaled_rect(
    size: Size,
    x_1080p: u32,
    y_1080p: u32,
    width_1080p: u32,
    height_1080p: u32,
) -> Result<Rect> {
    task_vision_result(Rect::new(
        scaled_width(size, x_1080p) as i32,
        scaled_height(size, y_1080p) as i32,
        scaled_width(size, width_1080p) as i32,
        scaled_height(size, height_1080p) as i32,
    ))
}

fn scaled_width(size: Size, value_1080p: u32) -> u32 {
    ((size.width as u64 * value_1080p as u64 + 960) / 1920) as u32
}

fn scaled_height(size: Size, value_1080p: u32) -> u32 {
    ((size.height as u64 * value_1080p as u64 + 540) / 1080) as u32
}
