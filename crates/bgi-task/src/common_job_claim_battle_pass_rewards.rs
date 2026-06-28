use super::{task_vision_result, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLAIM_BATTLE_PASS_REWARDS_TASK_KEY: &str = "ClaimBattlePassRewards";
pub const CLAIM_BATTLE_PASS_PROMPT_STAR: &str = "Common/Element:prompt_dialog_left_bottom_star.png";
pub const CLAIM_BATTLE_PASS_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const CLAIM_BATTLE_PASS_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const CLAIM_BATTLE_PASS_BLACK_CANCEL: &str = "Common/Element:btn_black_cancel.png";
pub const CLAIM_BATTLE_PASS_WHITE_CANCEL: &str = "Common/Element:btn_white_cancel.png";
pub const CLAIM_BATTLE_PASS_PRIMOGEM: &str = "Common/Element:primogem.png";

const DEFAULT_CLAIM_TEXT_PATTERNS: [&str; 2] = ["一键", "领取"];
const VK_ESCAPE: u16 = 0x1B;
const BEFORE_OPEN_DELAY_MS: u32 = 200;
const AFTER_OPEN_DELAY_MS: u32 = 1_000;
const BEFORE_POINTS_CLAIM_DELAY_MS: u32 = 500;
const AFTER_CLAIM_DELAY_MS: u32 = 1_000;
const UPGRADE_ANIMATION_DELAY_MS: u32 = 2_500;
const BEFORE_REWARDS_CLAIM_DELAY_MS: u32 = 1_500;
const POINTS_TAB_X_1080P: f64 = 960.0;
const POINTS_TAB_Y_1080P: f64 = 45.0;
const REWARDS_TAB_X_1080P: f64 = 858.0;
const REWARDS_TAB_Y_1080P: f64 = 45.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimBattlePassRewardsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub claim_all_rule: BattlePassClaimAllRule,
    pub manual_selection_rule: BattlePassManualSelectionDialogRule,
    pub locators: BattlePassRewardLocators,
    pub steps: Vec<BattlePassRewardStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattlePassClaimAllRule {
    pub claim_text_patterns: Vec<String>,
    pub ocr_roi: Rect,
    pub match_as_regex: bool,
    pub click_first_match: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattlePassManualSelectionDialogRule {
    pub prompt_star_locator: BvLocatorPlan,
    pub cancel_locators: Vec<BvLocatorPlan>,
    pub confirm_locators: Vec<BvLocatorPlan>,
    pub prompt_star_or_cancel_and_confirm: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattlePassRewardLocators {
    pub primogem: BvLocatorPlan,
    pub prompt_star: BvLocatorPlan,
    pub black_cancel: BvLocatorPlan,
    pub white_cancel: BvLocatorPlan,
    pub black_confirm: BvLocatorPlan,
    pub white_confirm: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattlePassRewardStep {
    pub phase: BattlePassRewardStepPhase,
    pub condition: BattlePassRewardStepCondition,
    pub scope: Option<BattlePassClaimScope>,
    pub label: String,
    pub action: BattlePassRewardStepAction,
}

impl BattlePassRewardStep {
    fn new(
        phase: BattlePassRewardStepPhase,
        condition: BattlePassRewardStepCondition,
        scope: Option<BattlePassClaimScope>,
        label: impl Into<String>,
        action: BattlePassRewardStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            scope,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePassClaimScope {
    Points,
    Rewards,
    UpgradeAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePassRewardStepPhase {
    Setup,
    PointsClaim,
    RewardsClaim,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePassRewardStepCondition {
    Always,
    WhenClaimAllTextMatched,
    AfterClaimClick,
    WhenPrimogemDetected,
    WhenManualSelectionDialogDetected,
    AfterClaimStages,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum BattlePassRewardStepAction {
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
    MatchClaimAll {
        rule: BattlePassClaimAllRule,
    },
    ClickMatchedText,
    DetectManualSelectionDialog {
        rule: BattlePassManualSelectionDialogRule,
    },
    DismissPrimogemIfVisible {
        locator: BvLocatorPlan,
        events: Vec<InputEvent>,
    },
    ReturnResult {
        result: BattlePassRewardStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePassRewardStepResult {
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaimBattlePassRewardsExecutionConfig {
    pub capture_size: Size,
    #[serde(alias = "claimTextPatterns")]
    #[serde(alias = "ClaimTextPatterns")]
    pub claim_text_patterns: Vec<String>,
}

impl Default for ClaimBattlePassRewardsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            claim_text_patterns: DEFAULT_CLAIM_TEXT_PATTERNS
                .iter()
                .map(|text| text.to_string())
                .collect(),
        }
    }
}

impl ClaimBattlePassRewardsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config
            .claim_text_patterns
            .iter()
            .all(|pattern| pattern.trim().is_empty())
        {
            config.claim_text_patterns = DEFAULT_CLAIM_TEXT_PATTERNS
                .iter()
                .map(|text| text.to_string())
                .collect();
        }
        config
            .claim_text_patterns
            .retain(|pattern| !pattern.trim().is_empty());
        config
    }
}

pub fn plan_claim_battle_pass_rewards(
    capture_size: Size,
    claim_text_patterns: impl IntoIterator<Item = impl Into<String>>,
) -> Result<ClaimBattlePassRewardsExecutionPlan> {
    let claim_text_patterns = normalized_claim_text_patterns(claim_text_patterns);
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = battle_pass_reward_locators(&page)?;
    let claim_all_rule = BattlePassClaimAllRule {
        claim_text_patterns,
        ocr_roi: right_bottom_roi(capture_size, 0.3, 0.2)?,
        match_as_regex: true,
        click_first_match: true,
    };
    let manual_selection_rule = BattlePassManualSelectionDialogRule {
        prompt_star_locator: locators.prompt_star.clone(),
        cancel_locators: vec![locators.black_cancel.clone(), locators.white_cancel.clone()],
        confirm_locators: vec![
            locators.black_confirm.clone(),
            locators.white_confirm.clone(),
        ],
        prompt_star_or_cancel_and_confirm: true,
    };
    let claim_all_ocr = page.ocr(Some(claim_all_rule.ocr_roi));
    let escape_events = InputSequence::new().key_press(VK_ESCAPE).events().to_vec();
    let mut steps = vec![
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::Setup,
            BattlePassRewardStepCondition::Always,
            None,
            "log battle pass reward claim start",
            BattlePassRewardStepAction::Log {
                message: "start ClaimBattlePassRewards common job plan".to_string(),
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::Setup,
            BattlePassRewardStepCondition::Always,
            None,
            "return to main UI before opening battle pass",
            BattlePassRewardStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::Setup,
            BattlePassRewardStepCondition::Always,
            None,
            "wait before opening battle pass",
            BattlePassRewardStepAction::Page {
                command: task_vision_result(page.wait(BEFORE_OPEN_DELAY_MS))?,
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::Setup,
            BattlePassRewardStepCondition::Always,
            None,
            "open battle pass screen",
            BattlePassRewardStepAction::GenshinAction {
                action: GenshinAction::OpenBattlePassScreen,
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::Setup,
            BattlePassRewardStepCondition::Always,
            None,
            "wait after opening battle pass",
            BattlePassRewardStepAction::Page {
                command: task_vision_result(page.wait(AFTER_OPEN_DELAY_MS))?,
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::PointsClaim,
            BattlePassRewardStepCondition::Always,
            Some(BattlePassClaimScope::Points),
            "click battle pass points tab",
            BattlePassRewardStepAction::Page {
                command: page.click_1080p(POINTS_TAB_X_1080P, POINTS_TAB_Y_1080P),
            },
        ),
        BattlePassRewardStep::new(
            BattlePassRewardStepPhase::PointsClaim,
            BattlePassRewardStepCondition::Always,
            Some(BattlePassClaimScope::Points),
            "wait before claiming battle pass points",
            BattlePassRewardStepAction::Page {
                command: task_vision_result(page.wait(BEFORE_POINTS_CLAIM_DELAY_MS))?,
            },
        ),
    ];
    append_claim_all_steps(
        &mut steps,
        BattlePassRewardStepPhase::PointsClaim,
        BattlePassClaimScope::Points,
        claim_all_ocr.clone(),
        &claim_all_rule,
        &manual_selection_rule,
        &locators,
        &escape_events,
        &page,
    )?;
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::RewardsClaim,
        BattlePassRewardStepCondition::Always,
        Some(BattlePassClaimScope::UpgradeAnimation),
        "wait for battle pass upgrade animation",
        BattlePassRewardStepAction::Page {
            command: task_vision_result(page.wait(UPGRADE_ANIMATION_DELAY_MS))?,
        },
    ));
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::RewardsClaim,
        BattlePassRewardStepCondition::WhenPrimogemDetected,
        Some(BattlePassClaimScope::UpgradeAnimation),
        "dismiss primogem popup before opening rewards tab",
        BattlePassRewardStepAction::DismissPrimogemIfVisible {
            locator: locators.primogem.clone(),
            events: escape_events.clone(),
        },
    ));
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::RewardsClaim,
        BattlePassRewardStepCondition::Always,
        Some(BattlePassClaimScope::Rewards),
        "click battle pass rewards tab",
        BattlePassRewardStepAction::Page {
            command: page.click_1080p(REWARDS_TAB_X_1080P, REWARDS_TAB_Y_1080P),
        },
    ));
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::RewardsClaim,
        BattlePassRewardStepCondition::Always,
        Some(BattlePassClaimScope::Rewards),
        "wait before claiming battle pass rewards",
        BattlePassRewardStepAction::Page {
            command: task_vision_result(page.wait(BEFORE_REWARDS_CLAIM_DELAY_MS))?,
        },
    ));
    append_claim_all_steps(
        &mut steps,
        BattlePassRewardStepPhase::RewardsClaim,
        BattlePassClaimScope::Rewards,
        claim_all_ocr,
        &claim_all_rule,
        &manual_selection_rule,
        &locators,
        &escape_events,
        &page,
    )?;
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::Cleanup,
        BattlePassRewardStepCondition::AfterClaimStages,
        None,
        "return to main UI after battle pass rewards",
        BattlePassRewardStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
        },
    ));
    steps.push(BattlePassRewardStep::new(
        BattlePassRewardStepPhase::Cleanup,
        BattlePassRewardStepCondition::AfterClaimStages,
        None,
        "return completed result",
        BattlePassRewardStepAction::ReturnResult {
            result: BattlePassRewardStepResult::Completed,
        },
    ));

    Ok(ClaimBattlePassRewardsExecutionPlan {
        task_key: CLAIM_BATTLE_PASS_REWARDS_TASK_KEY.to_string(),
        display_name: "Claim Battle Pass Rewards".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        claim_all_rule,
        manual_selection_rule,
        locators,
        steps,
        notes: "Legacy battle-pass reward claiming is represented as a Rust battle-pass page/OCR/dialog/locator plan with an injectable two-stage claim executor and desktop WinRT OCR/capture-click adapter; live OCR depends on installed Windows OCR language support.".to_string(),
    })
}

fn append_claim_all_steps(
    steps: &mut Vec<BattlePassRewardStep>,
    phase: BattlePassRewardStepPhase,
    scope: BattlePassClaimScope,
    claim_all_ocr: BvPageCommand,
    claim_all_rule: &BattlePassClaimAllRule,
    manual_selection_rule: &BattlePassManualSelectionDialogRule,
    locators: &BattlePassRewardLocators,
    escape_events: &[InputEvent],
    page: &BvPage,
) -> Result<()> {
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::Always,
        Some(scope),
        "OCR claim-all button text",
        BattlePassRewardStepAction::Ocr {
            command: claim_all_ocr,
        },
    ));
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::Always,
        Some(scope),
        "match localized claim-all text",
        BattlePassRewardStepAction::MatchClaimAll {
            rule: claim_all_rule.clone(),
        },
    ));
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::WhenClaimAllTextMatched,
        Some(scope),
        "click matched claim-all text",
        BattlePassRewardStepAction::ClickMatchedText,
    ));
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::AfterClaimClick,
        Some(scope),
        "wait after claim-all click",
        BattlePassRewardStepAction::Page {
            command: task_vision_result(page.wait(AFTER_CLAIM_DELAY_MS))?,
        },
    ));
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::AfterClaimClick,
        Some(scope),
        "detect manual-selection reward dialog",
        BattlePassRewardStepAction::DetectManualSelectionDialog {
            rule: manual_selection_rule.clone(),
        },
    ));
    steps.push(BattlePassRewardStep::new(
        phase,
        BattlePassRewardStepCondition::WhenPrimogemDetected,
        Some(scope),
        "dismiss primogem popup after claim-all",
        BattlePassRewardStepAction::DismissPrimogemIfVisible {
            locator: locators.primogem.clone(),
            events: escape_events.to_vec(),
        },
    ));
    Ok(())
}

fn battle_pass_reward_locators(page: &BvPage) -> Result<BattlePassRewardLocators> {
    Ok(BattlePassRewardLocators {
        primogem: image_locator(
            page,
            CLAIM_BATTLE_PASS_PRIMOGEM,
            Some(middle_third_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        prompt_star: image_locator(
            page,
            CLAIM_BATTLE_PASS_PROMPT_STAR,
            Some(left_bottom_half_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        black_cancel: dialog_button_locator(page, CLAIM_BATTLE_PASS_BLACK_CANCEL)?,
        white_cancel: dialog_button_locator(page, CLAIM_BATTLE_PASS_WHITE_CANCEL)?,
        black_confirm: dialog_button_locator(page, CLAIM_BATTLE_PASS_BLACK_CONFIRM)?,
        white_confirm: dialog_button_locator(page, CLAIM_BATTLE_PASS_WHITE_CONFIRM)?,
    })
}

fn dialog_button_locator(page: &BvPage, asset: &str) -> Result<BvLocatorPlan> {
    image_locator(
        page,
        asset,
        None,
        0.8,
        true,
        BvLocatorOperation::IsExist,
        Some(1_000),
    )
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

fn normalized_claim_text_patterns(
    patterns: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<String> {
    let patterns: Vec<String> = patterns
        .into_iter()
        .map(Into::into)
        .filter(|pattern| !pattern.trim().is_empty())
        .collect();
    if patterns.is_empty() {
        DEFAULT_CLAIM_TEXT_PATTERNS
            .iter()
            .map(|text| text.to_string())
            .collect()
    } else {
        patterns
    }
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

fn left_bottom_half_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height / 2) as i32,
        (size.width / 2) as i32,
        (size.height / 2) as i32,
    ))
}

fn middle_third_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height / 3) as i32,
        size.width as i32,
        (size.height / 3) as i32,
    ))
}
