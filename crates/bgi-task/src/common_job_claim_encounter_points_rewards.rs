use super::{image_locator, task_vision_result, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY: &str = "ClaimEncounterPointsRewards";
pub const CLAIM_ENCOUNTER_POINTS_REWARDS_BUTTON: &str =
    "Common/Element:btn_claim_encounter_points_rewards.png";
pub const CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES: u8 = 5;

const DEFAULT_COMMISSIONS_TEXT: &str = "委托";
const BEFORE_OPEN_DELAY_MS: u32 = 200;
const AFTER_OPEN_DELAY_MS: u32 = 1_000;
const AFTER_CLAIM_DELAY_MS: u32 = 1_000;
const CLAIM_BUTTON_TIMEOUT_MS: u32 = 1_000;
const LEFT_OCR_WIDTH_1080P: f64 = 380.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub commissions_text: String,
    pub max_open_retries: u8,
    pub ocr_rule: ClaimEncounterPointsRewardsOcrRule,
    pub claim_button_locator: BvLocatorPlan,
    pub steps: Vec<ClaimEncounterPointsRewardsStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsOcrRule {
    pub commissions_text: String,
    pub left_panel_roi: Rect,
    pub max_open_retries: u8,
    pub match_exact_text: bool,
    pub click_first_match: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimEncounterPointsRewardsStep {
    pub condition: ClaimEncounterPointsRewardsStepCondition,
    pub label: String,
    pub action: ClaimEncounterPointsRewardsStepAction,
}

impl ClaimEncounterPointsRewardsStep {
    fn new(
        condition: ClaimEncounterPointsRewardsStepCondition,
        label: impl Into<String>,
        action: ClaimEncounterPointsRewardsStepAction,
    ) -> Self {
        Self {
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimEncounterPointsRewardsStepCondition {
    Always,
    WhenCommissionsTextMatched,
    WhenEarlyClaimButtonDetected,
    WhenEarlyClaimButtonMissing,
    WhenClaimButtonDetected,
    AfterOpenRetryLimit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ClaimEncounterPointsRewardsStepAction {
    CommonJob {
        task_key: String,
    },
    Page {
        command: BvPageCommand,
    },
    GenshinAction {
        action: GenshinAction,
    },
    Ocr {
        command: BvPageCommand,
    },
    MatchCommissions {
        rule: ClaimEncounterPointsRewardsOcrRule,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    ClickMatchedText,
    ReturnResult {
        result: ClaimEncounterPointsRewardsStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimEncounterPointsRewardsStepResult {
    ClaimedFromVisibleButton,
    ClaimedAfterOpeningCommissions,
    CommissionsTabNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaimEncounterPointsRewardsExecutionConfig {
    pub capture_size: Size,
    #[serde(alias = "commissionsText")]
    #[serde(alias = "CommissionsText")]
    pub commissions_text: String,
    #[serde(alias = "maxOpenRetries")]
    #[serde(alias = "MaxOpenRetries")]
    pub max_open_retries: u8,
}

impl Default for ClaimEncounterPointsRewardsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            commissions_text: DEFAULT_COMMISSIONS_TEXT.to_string(),
            max_open_retries: CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES,
        }
    }
}

impl ClaimEncounterPointsRewardsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.commissions_text.trim().is_empty() {
            config.commissions_text = DEFAULT_COMMISSIONS_TEXT.to_string();
        }
        if config.max_open_retries == 0 {
            config.max_open_retries = CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES;
        }
        config
    }
}

pub fn plan_claim_encounter_points_rewards(
    capture_size: Size,
    commissions_text: impl Into<String>,
    max_open_retries: u8,
) -> Result<ClaimEncounterPointsRewardsExecutionPlan> {
    let mut commissions_text = commissions_text.into();
    if commissions_text.trim().is_empty() {
        commissions_text = DEFAULT_COMMISSIONS_TEXT.to_string();
    }
    let max_open_retries = if max_open_retries == 0 {
        CLAIM_ENCOUNTER_POINTS_REWARDS_DEFAULT_RETRIES
    } else {
        max_open_retries
    };
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let claim_button_locator = image_locator(
        &page,
        CLAIM_ENCOUNTER_POINTS_REWARDS_BUTTON,
        Some(right_bottom_roi(capture_size, 0.3, 0.5)?),
        0.8,
        BvLocatorOperation::Click,
        Some(CLAIM_BUTTON_TIMEOUT_MS),
    )?;
    let ocr_rule = ClaimEncounterPointsRewardsOcrRule {
        commissions_text: commissions_text.clone(),
        left_panel_roi: left_panel_ocr_roi(capture_size)?,
        max_open_retries,
        match_exact_text: true,
        click_first_match: true,
    };
    let left_panel_ocr = page.ocr(Some(ocr_rule.left_panel_roi));
    let mut steps = Vec::new();

    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "log claim encounter points rewards start",
        ClaimEncounterPointsRewardsStepAction::Log {
            message: "start ClaimEncounterPointsRewards common job plan".to_string(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "return to main UI before opening adventurer handbook",
        ClaimEncounterPointsRewardsStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "wait before opening adventurer handbook",
        ClaimEncounterPointsRewardsStepAction::Page {
            command: task_vision_result(page.wait(BEFORE_OPEN_DELAY_MS))?,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "open adventurer handbook",
        ClaimEncounterPointsRewardsStepAction::GenshinAction {
            action: GenshinAction::OpenAdventurerHandbook,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "wait after opening adventurer handbook",
        ClaimEncounterPointsRewardsStepAction::Page {
            command: task_vision_result(page.wait(AFTER_OPEN_DELAY_MS))?,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "OCR adventurer handbook left panel",
        ClaimEncounterPointsRewardsStepAction::Ocr {
            command: left_panel_ocr,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::Always,
        "match commissions tab text",
        ClaimEncounterPointsRewardsStepAction::MatchCommissions {
            rule: ocr_rule.clone(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenCommissionsTextMatched,
        "try direct claim before clicking commissions tab",
        ClaimEncounterPointsRewardsStepAction::Locator {
            locator: claim_button_locator.clone(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonDetected,
        "wait after early claim click",
        ClaimEncounterPointsRewardsStepAction::Page {
            command: task_vision_result(page.wait(AFTER_CLAIM_DELAY_MS))?,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonDetected,
        "return early claim result",
        ClaimEncounterPointsRewardsStepAction::ReturnResult {
            result: ClaimEncounterPointsRewardsStepResult::ClaimedFromVisibleButton,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonMissing,
        "click matched commissions tab text",
        ClaimEncounterPointsRewardsStepAction::ClickMatchedText,
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonMissing,
        "wait after commissions tab click",
        ClaimEncounterPointsRewardsStepAction::Page {
            command: task_vision_result(page.wait(AFTER_OPEN_DELAY_MS))?,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenEarlyClaimButtonMissing,
        "click claim encounter points rewards button",
        ClaimEncounterPointsRewardsStepAction::Locator {
            locator: claim_button_locator.clone(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenClaimButtonDetected,
        "wait after claim encounter points rewards click",
        ClaimEncounterPointsRewardsStepAction::Page {
            command: task_vision_result(page.wait(AFTER_CLAIM_DELAY_MS))?,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenClaimButtonDetected,
        "return to main UI after claiming encounter points rewards",
        ClaimEncounterPointsRewardsStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::WhenClaimButtonDetected,
        "return claimed after opening commissions result",
        ClaimEncounterPointsRewardsStepAction::ReturnResult {
            result: ClaimEncounterPointsRewardsStepResult::ClaimedAfterOpeningCommissions,
        },
    ));
    steps.push(ClaimEncounterPointsRewardsStep::new(
        ClaimEncounterPointsRewardsStepCondition::AfterOpenRetryLimit,
        "return commissions tab not found result",
        ClaimEncounterPointsRewardsStepAction::ReturnResult {
            result: ClaimEncounterPointsRewardsStepResult::CommissionsTabNotFound,
        },
    ));

    Ok(ClaimEncounterPointsRewardsExecutionPlan {
        task_key: CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY.to_string(),
        display_name: "Claim Encounter Points Rewards".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        commissions_text,
        max_open_retries,
        ocr_rule,
        claim_button_locator,
        steps,
        notes: "Legacy encounter-points reward claiming is represented as a Rust handbook/OCR/locator plan with an injectable OCR retry executor; desktop live OCR remains pending.".to_string(),
    })
}

fn left_panel_ocr_roi(size: Size) -> Result<Rect> {
    let scale = size.width as f64 / 1920.0;
    task_vision_result(Rect::new(
        0,
        0,
        (LEFT_OCR_WIDTH_1080P * scale).round() as i32,
        size.height as i32,
    ))
}

fn right_bottom_roi(size: Size, width_ratio: f64, height_ratio: f64) -> Result<Rect> {
    let width = (size.width as f64 * width_ratio).round() as i32;
    let height = (size.height as f64 * height_ratio).round() as i32;
    task_vision_result(Rect::new(
        size.width as i32 - width,
        size.height as i32 - height,
        width,
        height,
    ))
}
