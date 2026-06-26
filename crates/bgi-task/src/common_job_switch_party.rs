use super::{task_vision_result, RETURN_MAIN_UI_PAIMON_MENU, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, InputSequence, MouseButton};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SWITCH_PARTY_TASK_KEY: &str = "SwitchParty";
pub const SWITCH_PARTY_CHOOSE_VIEW: &str = "Common/Element:party_btn_choose_view.png";
pub const SWITCH_PARTY_DELETE: &str = "Common/Element:party_btn_delete.png";
pub const SWITCH_PARTY_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const SWITCH_PARTY_DEFAULT_OPEN_ATTEMPTS: u8 = 2;
pub const SWITCH_PARTY_DEFAULT_OPEN_CHECKS_PER_ATTEMPT: u8 = 7;
pub const SWITCH_PARTY_DEFAULT_LIST_SCAN_PAGES: u8 = 16;

const AFTER_RETURN_MAIN_UI_DELAY_MS: u32 = 200;
const OPEN_CHECK_INTERVAL_MS: u32 = 600;
const AFTER_PARTY_VIEW_DELAY_MS: u32 = 500;
const OPEN_CHOOSE_ATTEMPTS: u8 = 4;
const OPEN_CHOOSE_INTERVAL_MS: u32 = 500;
const DELETE_VERIFY_ATTEMPTS: u8 = 5;
const TOP_RESET_CLICK_X_1080P: f64 = 700.0;
const TOP_RESET_CLICK_Y_1080P: f64 = 125.0;
const TOP_RESET_PRE_DELAY_MS: u64 = 50;
const TOP_RESET_HOLD_MS: u64 = 450;
const TOP_RESET_POST_DELAY_MS: u64 = 100;
const CURRENT_TEAM_TEXT_WIDTH_1080P: f64 = 350.0;
const PARTY_LIST_TOP_Y_1080P: f64 = 80.0;
const PARTY_LIST_LOWEST_X_MIN_1080P: f64 = 35.0;
const PARTY_LIST_LOWEST_X_MAX_1080P: f64 = 100.0;
const PARTY_LIST_LAST_ITEM_THRESHOLD_Y_1080P: f64 = 777.0;
const FIRST_PAGE_PRECLICK_X_1080P: f64 = 600.0;
const FIRST_PAGE_PRECLICK_Y_1080P: f64 = 200.0;
const FIRST_PAGE_PRECLICK_DELAY_MS: u32 = 300;
const PAGE_SCROLL_DELAY_MS: u32 = 400;
const MATCHED_PARTY_CLICK_X_MULTIPLIER: f64 = 2.0;
const MATCHED_PARTY_CLICK_Y_ANCHOR: PartyTextClickYAnchor = PartyTextClickYAnchor::Bottom;
const AFTER_MATCHED_PARTY_CLICK_DELAY_MS: u32 = 200;
const CONFIRM_CLOSE_CHECKS: u8 = 10;
const AFTER_FIRST_CONFIRM_DELAY_MS: u32 = 200;
const AFTER_SECOND_CONFIRM_DELAY_MS: u32 = 500;
const MATCH_CURRENT_ESCAPE_DELAY_MS: u32 = 500;
const VK_ESCAPE: u16 = 0x1B;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub party_name: String,
    pub locators: SwitchPartyLocators,
    pub open_rule: SwitchPartyOpenRule,
    pub current_party_rule: SwitchPartyCurrentPartyRule,
    pub choose_menu_rule: SwitchPartyChooseMenuRule,
    pub list_scan_rule: SwitchPartyListScanRule,
    pub confirm_rule: SwitchPartyConfirmRule,
    pub steps: Vec<SwitchPartyStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyLocators {
    pub main_ui: BvLocatorPlan,
    pub party_choose_view: BvLocatorPlan,
    pub party_delete: BvLocatorPlan,
    pub white_confirm_left: BvLocatorPlan,
    pub white_confirm_right: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyOpenRule {
    pub max_attempts: u8,
    pub checks_per_attempt: u8,
    pub check_interval_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyCurrentPartyRule {
    pub ocr_roi: Rect,
    pub text_width: i32,
    pub strip_double_quotes: bool,
    pub remove_crlf: bool,
    pub truncate_at_first_lf: bool,
    pub trim: bool,
    pub match_as_regex: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchPartyChooseMenuRule {
    pub open_attempts: u8,
    pub open_interval_ms: u32,
    pub delete_verify_attempts: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyListScanRule {
    pub ocr_roi: Rect,
    pub max_pages: u8,
    pub lowest_item_x_min: i32,
    pub lowest_item_x_max: i32,
    pub last_item_threshold_y: i32,
    pub first_page_preclick: SwitchPartyScreenPoint,
    pub first_page_preclick_delay_ms: u32,
    pub page_scroll_delay_ms: u32,
    pub matched_party_click_x_multiplier: f64,
    pub matched_party_click_y_anchor: PartyTextClickYAnchor,
    pub after_matched_party_click_delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyConfirmRule {
    pub left_confirm_locator: BvLocatorPlan,
    pub right_confirm_locator: BvLocatorPlan,
    pub party_delete_locator: BvLocatorPlan,
    pub close_check_attempts: u8,
    pub after_first_confirm_delay_ms: u32,
    pub after_second_confirm_delay_ms: u32,
    pub return_main_ui_when_opened_from_main: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyScreenPoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub screen_x: f64,
    pub screen_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyTextClickYAnchor {
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchPartyStep {
    pub phase: SwitchPartyStepPhase,
    pub condition: SwitchPartyStepCondition,
    pub label: String,
    pub action: SwitchPartyStepAction,
}

impl SwitchPartyStep {
    fn new(
        phase: SwitchPartyStepPhase,
        condition: SwitchPartyStepCondition,
        label: impl Into<String>,
        action: SwitchPartyStepAction,
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
pub enum SwitchPartyStepPhase {
    Setup,
    OpenPartyView,
    CurrentPartyCheck,
    PartyList,
    Confirm,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchPartyStepCondition {
    Always,
    WhenMainUiMissing,
    WhenPartyViewMissing,
    WhenCurrentPartyMatched,
    WhenCurrentPartyNotMatched,
    WhenPartyMatchedInList,
    WhenPartyNotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum SwitchPartyStepAction {
    CommonJob {
        task_key: String,
    },
    Page {
        command: BvPageCommand,
    },
    GenshinAction {
        action: GenshinAction,
    },
    Input {
        events: Vec<InputEvent>,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Ocr {
        command: BvPageCommand,
    },
    NormalizeCurrentPartyName {
        rule: SwitchPartyCurrentPartyRule,
    },
    MatchCurrentParty {
        party_name: String,
    },
    OpenPartyChooseMenu {
        rule: SwitchPartyChooseMenuRule,
        choose_locator: BvLocatorPlan,
        delete_locator: BvLocatorPlan,
    },
    ScanPartyList {
        rule: SwitchPartyListScanRule,
        party_name: String,
    },
    ConfirmParty {
        rule: SwitchPartyConfirmRule,
    },
    ClearCombatScenes,
    ReturnResult {
        result: SwitchPartyStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchPartyStepResult {
    AlreadySelected,
    Switched,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SwitchPartyExecutionConfig {
    #[serde(alias = "partyName")]
    #[serde(alias = "PartyName")]
    pub party_name: String,
    pub capture_size: Size,
}

impl Default for SwitchPartyExecutionConfig {
    fn default() -> Self {
        Self {
            party_name: String::new(),
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl SwitchPartyExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn plan_switch_party(
    capture_size: Size,
    party_name: impl Into<String>,
) -> Result<SwitchPartyExecutionPlan> {
    let party_name = party_name.into();
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = switch_party_locators(&page)?;
    let open_rule = SwitchPartyOpenRule {
        max_attempts: SWITCH_PARTY_DEFAULT_OPEN_ATTEMPTS,
        checks_per_attempt: SWITCH_PARTY_DEFAULT_OPEN_CHECKS_PER_ATTEMPT,
        check_interval_ms: OPEN_CHECK_INTERVAL_MS,
    };
    let current_party_rule = current_party_rule(capture_size)?;
    let choose_menu_rule = SwitchPartyChooseMenuRule {
        open_attempts: OPEN_CHOOSE_ATTEMPTS,
        open_interval_ms: OPEN_CHOOSE_INTERVAL_MS,
        delete_verify_attempts: DELETE_VERIFY_ATTEMPTS,
    };
    let list_scan_rule = list_scan_rule(capture_size)?;
    let confirm_rule = SwitchPartyConfirmRule {
        left_confirm_locator: locators.white_confirm_left.clone(),
        right_confirm_locator: locators.white_confirm_right.clone(),
        party_delete_locator: locators.party_delete.clone(),
        close_check_attempts: CONFIRM_CLOSE_CHECKS,
        after_first_confirm_delay_ms: AFTER_FIRST_CONFIRM_DELAY_MS,
        after_second_confirm_delay_ms: AFTER_SECOND_CONFIRM_DELAY_MS,
        return_main_ui_when_opened_from_main: true,
    };
    let current_party_ocr = page.ocr(Some(current_party_rule.ocr_roi));
    let party_list_ocr = page.ocr(Some(list_scan_rule.ocr_roi));
    let top_reset_events = top_reset_events(capture_size);
    let escape_events = InputSequence::new().key_press(VK_ESCAPE).events().to_vec();
    let mut steps = Vec::new();

    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Setup,
        SwitchPartyStepCondition::Always,
        "log switch party start",
        SwitchPartyStepAction::Log {
            message: format!("start SwitchParty common job plan for {party_name}"),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Setup,
        SwitchPartyStepCondition::WhenMainUiMissing,
        "return to main UI before opening party setup",
        SwitchPartyStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Setup,
        SwitchPartyStepCondition::WhenMainUiMissing,
        "wait after returning to main UI",
        SwitchPartyStepAction::Page {
            command: task_vision_result(page.wait(AFTER_RETURN_MAIN_UI_DELAY_MS))?,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::OpenPartyView,
        SwitchPartyStepCondition::WhenPartyViewMissing,
        "open party setup screen",
        SwitchPartyStepAction::GenshinAction {
            action: GenshinAction::OpenPartySetupScreen,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::OpenPartyView,
        SwitchPartyStepCondition::WhenPartyViewMissing,
        "wait for party setup screen",
        SwitchPartyStepAction::Locator {
            locator: locators.party_choose_view.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::OpenPartyView,
        SwitchPartyStepCondition::Always,
        "wait after party view is available",
        SwitchPartyStepAction::Page {
            command: task_vision_result(page.wait(AFTER_PARTY_VIEW_DELAY_MS))?,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::Always,
        "OCR current party name",
        SwitchPartyStepAction::Ocr {
            command: current_party_ocr,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::Always,
        "normalize current party OCR text",
        SwitchPartyStepAction::NormalizeCurrentPartyName {
            rule: current_party_rule.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::Always,
        "match current party by regex",
        SwitchPartyStepAction::MatchCurrentParty {
            party_name: party_name.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::WhenCurrentPartyMatched,
        "press Escape when current party is already selected",
        SwitchPartyStepAction::Input {
            events: escape_events.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::WhenCurrentPartyMatched,
        "wait after closing party setup",
        SwitchPartyStepAction::Page {
            command: task_vision_result(page.wait(MATCH_CURRENT_ESCAPE_DELAY_MS))?,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::WhenCurrentPartyMatched,
        "return to main UI after already-selected party",
        SwitchPartyStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::CurrentPartyCheck,
        SwitchPartyStepCondition::WhenCurrentPartyMatched,
        "return already-selected result",
        SwitchPartyStepAction::ReturnResult {
            result: SwitchPartyStepResult::AlreadySelected,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::PartyList,
        SwitchPartyStepCondition::WhenCurrentPartyNotMatched,
        "open party choose menu",
        SwitchPartyStepAction::OpenPartyChooseMenu {
            rule: choose_menu_rule,
            choose_locator: locators.party_choose_view.clone(),
            delete_locator: locators.party_delete.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::PartyList,
        SwitchPartyStepCondition::WhenCurrentPartyNotMatched,
        "reset party list to top",
        SwitchPartyStepAction::Input {
            events: top_reset_events,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::PartyList,
        SwitchPartyStepCondition::WhenCurrentPartyNotMatched,
        "OCR party list",
        SwitchPartyStepAction::Ocr {
            command: party_list_ocr,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::PartyList,
        SwitchPartyStepCondition::WhenCurrentPartyNotMatched,
        "scan party list pages by regex",
        SwitchPartyStepAction::ScanPartyList {
            rule: list_scan_rule.clone(),
            party_name: party_name.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Confirm,
        SwitchPartyStepCondition::WhenPartyMatchedInList,
        "confirm selected party",
        SwitchPartyStepAction::ConfirmParty {
            rule: confirm_rule.clone(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Cleanup,
        SwitchPartyStepCondition::WhenPartyMatchedInList,
        "clear combat scenes after party switch",
        SwitchPartyStepAction::ClearCombatScenes,
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Cleanup,
        SwitchPartyStepCondition::WhenPartyMatchedInList,
        "return switched result",
        SwitchPartyStepAction::ReturnResult {
            result: SwitchPartyStepResult::Switched,
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Cleanup,
        SwitchPartyStepCondition::WhenPartyNotFound,
        "return to main UI when party was not found",
        SwitchPartyStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(SwitchPartyStep::new(
        SwitchPartyStepPhase::Cleanup,
        SwitchPartyStepCondition::WhenPartyNotFound,
        "return not-found result",
        SwitchPartyStepAction::ReturnResult {
            result: SwitchPartyStepResult::NotFound,
        },
    ));

    Ok(SwitchPartyExecutionPlan {
        task_key: SWITCH_PARTY_TASK_KEY.to_string(),
        display_name: "Switch Party".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        party_name,
        locators,
        open_rule,
        current_party_rule,
        choose_menu_rule,
        list_scan_rule,
        confirm_rule,
        steps,
        notes: "Legacy SwitchParty OCR/list-scan/confirm flow is represented and executable through injectable OCR, list scan, confirm, and combat-scene cleanup hooks; desktop live WinRT OCR/template/capture-click execution is wired, with OCR depending on installed Windows OCR language support.".to_string(),
    })
}

fn switch_party_locators(page: &BvPage) -> Result<SwitchPartyLocators> {
    Ok(SwitchPartyLocators {
        main_ui: image_locator(
            page,
            RETURN_MAIN_UI_PAIMON_MENU,
            Some(top_left_quarter_rect(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        party_choose_view: image_locator(
            page,
            SWITCH_PARTY_CHOOSE_VIEW,
            Some(party_choose_view_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::WaitFor,
            Some(OPEN_CHECK_INTERVAL_MS * SWITCH_PARTY_DEFAULT_OPEN_CHECKS_PER_ATTEMPT as u32),
        )?,
        party_delete: image_locator(
            page,
            SWITCH_PARTY_DELETE,
            Some(party_delete_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        white_confirm_left: image_locator(
            page,
            SWITCH_PARTY_WHITE_CONFIRM,
            Some(confirm_left_roi(page.capture_size)?),
            0.8,
            true,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        white_confirm_right: image_locator(
            page,
            SWITCH_PARTY_WHITE_CONFIRM,
            Some(confirm_right_roi(page.capture_size)?),
            0.8,
            true,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
    })
}

fn current_party_rule(size: Size) -> Result<SwitchPartyCurrentPartyRule> {
    let choose_roi = party_choose_view_roi(size)?;
    let width = scaled_i32(CURRENT_TEAM_TEXT_WIDTH_1080P, size);
    let ocr_roi = task_vision_result(Rect::new(
        choose_roi.right(),
        choose_roi.y,
        width,
        choose_roi.height,
    ))?;
    Ok(SwitchPartyCurrentPartyRule {
        ocr_roi,
        text_width: width,
        strip_double_quotes: true,
        remove_crlf: true,
        truncate_at_first_lf: true,
        trim: true,
        match_as_regex: true,
    })
}

fn list_scan_rule(size: Size) -> Result<SwitchPartyListScanRule> {
    let delete_roi = party_delete_roi(size)?;
    let top_y = scaled_i32(PARTY_LIST_TOP_Y_1080P, size);
    let ocr_roi = task_vision_result(Rect::new(
        0,
        top_y,
        delete_roi.right(),
        delete_roi.y - top_y,
    ))?;
    Ok(SwitchPartyListScanRule {
        ocr_roi,
        max_pages: SWITCH_PARTY_DEFAULT_LIST_SCAN_PAGES,
        lowest_item_x_min: scaled_i32(PARTY_LIST_LOWEST_X_MIN_1080P, size),
        lowest_item_x_max: scaled_i32(PARTY_LIST_LOWEST_X_MAX_1080P, size),
        last_item_threshold_y: scaled_i32(PARTY_LIST_LAST_ITEM_THRESHOLD_Y_1080P, size),
        first_page_preclick: point_1080p(
            size,
            FIRST_PAGE_PRECLICK_X_1080P,
            FIRST_PAGE_PRECLICK_Y_1080P,
        ),
        first_page_preclick_delay_ms: FIRST_PAGE_PRECLICK_DELAY_MS,
        page_scroll_delay_ms: PAGE_SCROLL_DELAY_MS,
        matched_party_click_x_multiplier: MATCHED_PARTY_CLICK_X_MULTIPLIER,
        matched_party_click_y_anchor: MATCHED_PARTY_CLICK_Y_ANCHOR,
        after_matched_party_click_delay_ms: AFTER_MATCHED_PARTY_CLICK_DELAY_MS,
    })
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    use_3_channels: bool,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> Result<BvLocatorPlan> {
    let image = task_vision_result(BvImage::new(asset))?;
    let mut locator = task_vision_result(page.locator_for_image(&image, roi, threshold))?;
    locator.recognition_object.template.use_3_channels = use_3_channels;
    Ok(locator.plan(operation, timeout_ms))
}

fn top_reset_events(size: Size) -> Vec<InputEvent> {
    let point = point_1080p(size, TOP_RESET_CLICK_X_1080P, TOP_RESET_CLICK_Y_1080P);
    InputSequence::new()
        .move_mouse_to(point.screen_x.round() as i32, point.screen_y.round() as i32)
        .mouse_click(MouseButton::Left)
        .delay(TOP_RESET_PRE_DELAY_MS)
        .mouse_down(MouseButton::Left)
        .delay(TOP_RESET_HOLD_MS)
        .mouse_up(MouseButton::Left)
        .delay(TOP_RESET_POST_DELAY_MS)
        .events()
        .to_vec()
}

fn point_1080p(size: Size, x_1080p: f64, y_1080p: f64) -> SwitchPartyScreenPoint {
    let scale = size.width as f64 / 1920.0;
    SwitchPartyScreenPoint {
        x_1080p,
        y_1080p,
        screen_x: x_1080p * scale,
        screen_y: y_1080p * scale,
    }
}

fn party_choose_view_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        size.height as i32 - scaled_i32(120.0, size),
        (size.width / 7) as i32,
        scaled_i32(120.0, size),
    ))
}

fn party_delete_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 4) as i32,
        size.height as i32 - scaled_i32(120.0, size),
        (size.width / 2) as i32,
        scaled_i32(120.0, size),
    ))
}

fn confirm_left_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height / 4) as i32,
        (size.width / 4) as i32,
        (size.height - size.height / 4) as i32,
    ))
}

fn confirm_right_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width - size.width / 4) as i32,
        (size.height / 4) as i32,
        (size.width / 4) as i32,
        (size.height - size.height / 4) as i32,
    ))
}

fn top_left_quarter_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}

fn scaled_i32(value_1080p: f64, size: Size) -> i32 {
    (value_1080p * size.width as f64 / 1920.0).round() as i32
}
