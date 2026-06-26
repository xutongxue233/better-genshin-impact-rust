use super::{image_locator, task_vision_result};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Scalar4, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CHOOSE_TALK_OPTION_TASK_KEY: &str = "ChooseTalkOption";
pub const CHOOSE_TALK_OPTION_DISABLED_UI: &str = "AutoSkip:disabled_ui.png";
pub const CHOOSE_TALK_OPTION_ICON: &str = "AutoSkip:icon_option.png";
pub const CHOOSE_TALK_OPTION_VK_SPACE: u16 = 0x20;

const DEFAULT_SKIP_TIMES: u32 = 10;
const TALK_UI_WAIT_TIMEOUT_MS: u32 = 10_000;
const FIRST_OCR_STABILIZE_DELAY_MS: u32 = 1_000;
const OPTION_NOT_FOUND_SKIP_DELAY_MS: u32 = 500;
const CLICK_MATCHED_OPTION_DELAY_MS: u32 = 300;
const ORANGE_MIN_PIXEL_RATE: f64 = 0.1;
const OCR_TEXT_WIDTH_1080P: f64 = 535.0;
const OCR_TEXT_X_PADDING_1080P: f64 = 8.0;
const OCR_TEXT_BOTTOM_PADDING_1080P: f64 = 30.0;
const OCR_LARGE_Y_GAP_1080P: f64 = 150.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub option: String,
    pub skip_times: u32,
    pub is_orange: bool,
    pub talk_ui_locator: BvLocatorPlan,
    pub option_icon_locator: BvLocatorPlan,
    pub ocr_rule: ChooseTalkOptionOcrRule,
    pub orange_rule: Option<ChooseTalkOptionOrangeRule>,
    pub steps: Vec<ChooseTalkOptionStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionOcrRule {
    pub option_icon_roi: Rect,
    pub ocr_y: i32,
    pub ocr_width: i32,
    pub ocr_x_padding: i32,
    pub ocr_bottom_padding: i32,
    pub ignore_short_alphanumeric_text: bool,
    pub short_alphanumeric_max_len: usize,
    pub ignored_large_y_gap: i32,
    pub sort_icons_by_y_descending: bool,
    pub sort_ocr_results_by_y_ascending: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionOrangeRule {
    pub hsv_lower: Scalar4,
    pub hsv_upper: Scalar4,
    pub min_pixel_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChooseTalkOptionStep {
    pub condition: ChooseTalkOptionStepCondition,
    pub attempt: Option<u32>,
    pub label: String,
    pub action: ChooseTalkOptionStepAction,
}

impl ChooseTalkOptionStep {
    fn new(label: impl Into<String>, action: ChooseTalkOptionStepAction) -> Self {
        Self {
            condition: ChooseTalkOptionStepCondition::Always,
            attempt: None,
            label: label.into(),
            action,
        }
    }

    fn conditional(
        condition: ChooseTalkOptionStepCondition,
        label: impl Into<String>,
        action: ChooseTalkOptionStepAction,
    ) -> Self {
        Self {
            condition,
            attempt: None,
            label: label.into(),
            action,
        }
    }

    fn for_attempt(
        attempt: u32,
        condition: ChooseTalkOptionStepCondition,
        label: impl Into<String>,
        action: ChooseTalkOptionStepAction,
    ) -> Self {
        Self {
            condition,
            attempt: Some(attempt),
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChooseTalkOptionStepCondition {
    Always,
    FirstOcrPass,
    WhenOptionIconMissing,
    WhenOptionTextMatched,
    WhenOrangeRequired,
    WhenOrangeRejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ChooseTalkOptionStepAction {
    Page { command: BvPageCommand },
    Input { events: Vec<InputEvent> },
    Locator { locator: BvLocatorPlan },
    RecognizeOptions { rule: ChooseTalkOptionOcrRule },
    MatchText { option: String },
    CheckOrange { rule: ChooseTalkOptionOrangeRule },
    ClickMatchedOption,
    ReturnResult { result: TalkOptionPlanResult },
    Log { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TalkOptionPlanResult {
    NotFound,
    FoundButNotOrange,
    FoundAndClick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChooseTalkOptionExecutionConfig {
    #[serde(alias = "Option")]
    pub option: String,
    #[serde(alias = "skipTimes")]
    #[serde(alias = "SkipTimes")]
    pub skip_times: u32,
    #[serde(alias = "isOrange")]
    #[serde(alias = "IsOrange")]
    pub is_orange: bool,
    pub capture_size: Size,
}

impl Default for ChooseTalkOptionExecutionConfig {
    fn default() -> Self {
        Self {
            option: String::new(),
            skip_times: DEFAULT_SKIP_TIMES,
            is_orange: false,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl ChooseTalkOptionExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.skip_times == 0 {
            config.skip_times = DEFAULT_SKIP_TIMES;
        }
        config
    }
}

pub fn plan_choose_talk_option(
    capture_size: Size,
    option: impl Into<String>,
    skip_times: u32,
    is_orange: bool,
) -> Result<ChooseTalkOptionExecutionPlan> {
    let option = option.into();
    let skip_times = if skip_times == 0 {
        DEFAULT_SKIP_TIMES
    } else {
        skip_times
    };
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let talk_ui_locator = image_locator(
        &page,
        CHOOSE_TALK_OPTION_DISABLED_UI,
        Some(disabled_ui_roi(capture_size)?),
        0.8,
        BvLocatorOperation::WaitFor,
        Some(TALK_UI_WAIT_TIMEOUT_MS),
    )?;
    let option_icon_locator = image_locator(
        &page,
        CHOOSE_TALK_OPTION_ICON,
        Some(option_icon_roi(capture_size)?),
        0.8,
        BvLocatorOperation::FindAll,
        Some(1_000),
    )?;
    let ocr_rule = choose_talk_option_ocr_rule(capture_size)?;
    let orange_rule = is_orange.then(choose_talk_option_orange_rule);
    let mut steps = Vec::new();

    steps.push(ChooseTalkOptionStep::new(
        "log choose talk option start",
        ChooseTalkOptionStepAction::Log {
            message: format!("choose talk option: {option}"),
        },
    ));
    steps.push(ChooseTalkOptionStep::new(
        "wait for talk UI",
        ChooseTalkOptionStepAction::Locator {
            locator: talk_ui_locator.clone(),
        },
    ));
    steps.push(ChooseTalkOptionStep::new(
        "wait before first option OCR",
        ChooseTalkOptionStepAction::Page {
            command: task_vision_result(page.wait(500))?,
        },
    ));

    for attempt in 1..=skip_times {
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::Always,
            "find talk option icons",
            ChooseTalkOptionStepAction::Locator {
                locator: option_icon_locator.clone(),
            },
        ));
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::WhenOptionIconMissing,
            "press Space when option icons are missing",
            ChooseTalkOptionStepAction::Input {
                events: InputSequence::new()
                    .key_press(CHOOSE_TALK_OPTION_VK_SPACE)
                    .events()
                    .to_vec(),
            },
        ));
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::WhenOptionIconMissing,
            "wait after Space retry",
            ChooseTalkOptionStepAction::Page {
                command: task_vision_result(page.wait(OPTION_NOT_FOUND_SKIP_DELAY_MS))?,
            },
        ));
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::Always,
            "recognize option text from lowest option icon",
            ChooseTalkOptionStepAction::RecognizeOptions {
                rule: ocr_rule.clone(),
            },
        ));
        if attempt == 1 {
            steps.push(ChooseTalkOptionStep::for_attempt(
                attempt,
                ChooseTalkOptionStepCondition::FirstOcrPass,
                "wait for fully displayed option text after first OCR",
                ChooseTalkOptionStepAction::Page {
                    command: task_vision_result(page.wait(FIRST_OCR_STABILIZE_DELAY_MS))?,
                },
            ));
        }
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::Always,
            "match configured option text",
            ChooseTalkOptionStepAction::MatchText {
                option: option.clone(),
            },
        ));
        if let Some(rule) = &orange_rule {
            steps.push(ChooseTalkOptionStep::for_attempt(
                attempt,
                ChooseTalkOptionStepCondition::WhenOrangeRequired,
                "check matched option is orange",
                ChooseTalkOptionStepAction::CheckOrange { rule: rule.clone() },
            ));
            steps.push(ChooseTalkOptionStep::for_attempt(
                attempt,
                ChooseTalkOptionStepCondition::WhenOrangeRejected,
                "return found but not orange",
                ChooseTalkOptionStepAction::ReturnResult {
                    result: TalkOptionPlanResult::FoundButNotOrange,
                },
            ));
        }
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::WhenOptionTextMatched,
            "click matched option",
            ChooseTalkOptionStepAction::ClickMatchedOption,
        ));
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::WhenOptionTextMatched,
            "wait after clicking matched option",
            ChooseTalkOptionStepAction::Page {
                command: task_vision_result(page.wait(CLICK_MATCHED_OPTION_DELAY_MS))?,
            },
        ));
        steps.push(ChooseTalkOptionStep::for_attempt(
            attempt,
            ChooseTalkOptionStepCondition::WhenOptionTextMatched,
            "return found and clicked",
            ChooseTalkOptionStepAction::ReturnResult {
                result: TalkOptionPlanResult::FoundAndClick,
            },
        ));
    }

    steps.push(ChooseTalkOptionStep::conditional(
        ChooseTalkOptionStepCondition::Always,
        "return not found after retries",
        ChooseTalkOptionStepAction::ReturnResult {
            result: TalkOptionPlanResult::NotFound,
        },
    ));

    Ok(ChooseTalkOptionExecutionPlan {
        task_key: CHOOSE_TALK_OPTION_TASK_KEY.to_string(),
        display_name: "Choose Talk Option".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        option,
        skip_times,
        is_orange,
        talk_ui_locator,
        option_icon_locator,
        ocr_rule,
        orange_rule,
        steps,
        notes: "Legacy ChooseTalkOption OCR/icon retry flow is represented as a Rust locator/OCR/input plan with desktop WinRT OCR/click/color adapter coverage; live OCR still depends on installed Windows OCR language support.".to_string(),
    })
}

fn choose_talk_option_ocr_rule(capture_size: Size) -> Result<ChooseTalkOptionOcrRule> {
    let scale = capture_size.width as f64 / 1920.0;
    Ok(ChooseTalkOptionOcrRule {
        option_icon_roi: option_icon_roi(capture_size)?,
        ocr_y: (capture_size.height / 8) as i32,
        ocr_width: scaled_i32(OCR_TEXT_WIDTH_1080P, scale),
        ocr_x_padding: scaled_i32(OCR_TEXT_X_PADDING_1080P, scale),
        ocr_bottom_padding: scaled_i32(OCR_TEXT_BOTTOM_PADDING_1080P, scale),
        ignore_short_alphanumeric_text: true,
        short_alphanumeric_max_len: 5,
        ignored_large_y_gap: scaled_i32(OCR_LARGE_Y_GAP_1080P, scale),
        sort_icons_by_y_descending: true,
        sort_ocr_results_by_y_ascending: true,
    })
}

fn choose_talk_option_orange_rule() -> ChooseTalkOptionOrangeRule {
    ChooseTalkOptionOrangeRule {
        hsv_lower: Scalar4 {
            v0: 10.0,
            v1: 150.0,
            v2: 150.0,
            v3: 0.0,
        },
        hsv_upper: Scalar4 {
            v0: 25.0,
            v1: 255.0,
            v2: 255.0,
            v3: 0.0,
        },
        min_pixel_rate: ORANGE_MIN_PIXEL_RATE,
    }
}

fn disabled_ui_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 3) as i32,
        (size.height / 8) as i32,
    ))
}

fn option_icon_roi(size: Size) -> Result<Rect> {
    let x = (size.width / 2) as i32;
    let y = (size.height / 12) as i32;
    let width = (size.width - size.width / 2 - size.width / 6) as i32;
    let height = (size.height - size.height / 12 - 10) as i32;
    task_vision_result(Rect::new(x, y, width, height))
}

fn scaled_i32(value_1080p: f64, scale: f64) -> i32 {
    (value_1080p * scale).round() as i32
}
