use super::{image_locator, task_vision_result, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CLAIM_MAIL_REWARDS_TASK_KEY: &str = "ClaimMailRewards";
pub const CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD: &str = "Common/Element:esc_mail_reward.png";
pub const CLAIM_MAIL_REWARDS_COLLECT: &str = "Common/Element:collect.png";

const VK_ESCAPE: u16 = 0x1B;
const BEFORE_OPEN_DELAY_MS: u32 = 200;
const AFTER_OPEN_DELAY_MS: u32 = 1_300;
const AFTER_MAIL_CLICK_DELAY_MS: u32 = 1_000;
const AFTER_COLLECT_CLICK_DELAY_MS: u32 = 200;
const LOCATOR_TIMEOUT_MS: u32 = 1_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimMailRewardsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub locators: ClaimMailRewardsLocators,
    pub steps: Vec<ClaimMailRewardsStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimMailRewardsLocators {
    pub mail_reward_detect: BvLocatorPlan,
    pub mail_reward_click: BvLocatorPlan,
    pub collect_all_detect: BvLocatorPlan,
    pub collect_all_click: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimMailRewardsStep {
    pub phase: ClaimMailRewardsStepPhase,
    pub condition: ClaimMailRewardsStepCondition,
    pub label: String,
    pub action: ClaimMailRewardsStepAction,
}

impl ClaimMailRewardsStep {
    fn new(
        phase: ClaimMailRewardsStepPhase,
        condition: ClaimMailRewardsStepCondition,
        label: impl Into<String>,
        action: ClaimMailRewardsStepAction,
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
pub enum ClaimMailRewardsStepPhase {
    Setup,
    PaimonMenu,
    MailClaim,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimMailRewardsStepCondition {
    Always,
    WhenMailRewardDetected,
    WhenMailRewardMissing,
    WhenCollectAllDetected,
    WhenCollectAllMissing,
    AfterClaimAttempt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ClaimMailRewardsStepAction {
    CommonJob { task_key: String },
    Page { command: BvPageCommand },
    GenshinAction { action: GenshinAction },
    Locator { locator: BvLocatorPlan },
    Input { events: Vec<InputEvent> },
    ReturnResult { result: ClaimMailRewardsStepResult },
    Log { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimMailRewardsStepResult {
    Claimed,
    NoMailRewards,
    MailOpenedWithoutClaimAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaimMailRewardsExecutionConfig {
    pub capture_size: Size,
}

impl Default for ClaimMailRewardsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl ClaimMailRewardsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn plan_claim_mail_rewards(capture_size: Size) -> Result<ClaimMailRewardsExecutionPlan> {
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = claim_mail_rewards_locators(&page)?;
    let escape_events = InputSequence::new().key_press(VK_ESCAPE).events().to_vec();
    let steps = vec![
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Setup,
            ClaimMailRewardsStepCondition::Always,
            "log claim mail rewards start",
            ClaimMailRewardsStepAction::Log {
                message: "start ClaimMailRewards common job plan".to_string(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Setup,
            ClaimMailRewardsStepCondition::Always,
            "return to main UI before opening Paimon menu",
            ClaimMailRewardsStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Setup,
            ClaimMailRewardsStepCondition::Always,
            "wait before opening Paimon menu",
            ClaimMailRewardsStepAction::Page {
                command: task_vision_result(page.wait(BEFORE_OPEN_DELAY_MS))?,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::PaimonMenu,
            ClaimMailRewardsStepCondition::Always,
            "open Paimon menu",
            ClaimMailRewardsStepAction::GenshinAction {
                action: GenshinAction::OpenPaimonMenu,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::PaimonMenu,
            ClaimMailRewardsStepCondition::Always,
            "wait after opening Paimon menu",
            ClaimMailRewardsStepAction::Page {
                command: task_vision_result(page.wait(AFTER_OPEN_DELAY_MS))?,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::PaimonMenu,
            ClaimMailRewardsStepCondition::Always,
            "detect mail reward icon",
            ClaimMailRewardsStepAction::Locator {
                locator: locators.mail_reward_detect.clone(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::PaimonMenu,
            ClaimMailRewardsStepCondition::WhenMailRewardDetected,
            "click mail reward icon",
            ClaimMailRewardsStepAction::Locator {
                locator: locators.mail_reward_click.clone(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::MailClaim,
            ClaimMailRewardsStepCondition::WhenMailRewardDetected,
            "wait after opening mail reward page",
            ClaimMailRewardsStepAction::Page {
                command: task_vision_result(page.wait(AFTER_MAIL_CLICK_DELAY_MS))?,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::MailClaim,
            ClaimMailRewardsStepCondition::WhenMailRewardDetected,
            "detect collect-all button",
            ClaimMailRewardsStepAction::Locator {
                locator: locators.collect_all_detect.clone(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::MailClaim,
            ClaimMailRewardsStepCondition::WhenCollectAllDetected,
            "click collect-all button",
            ClaimMailRewardsStepAction::Locator {
                locator: locators.collect_all_click.clone(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::MailClaim,
            ClaimMailRewardsStepCondition::WhenCollectAllDetected,
            "wait after collect-all click",
            ClaimMailRewardsStepAction::Page {
                command: task_vision_result(page.wait(AFTER_COLLECT_CLICK_DELAY_MS))?,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::MailClaim,
            ClaimMailRewardsStepCondition::WhenCollectAllDetected,
            "press Escape after mail rewards claim",
            ClaimMailRewardsStepAction::Input {
                events: escape_events,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Cleanup,
            ClaimMailRewardsStepCondition::AfterClaimAttempt,
            "return to main UI after mail rewards flow",
            ClaimMailRewardsStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Cleanup,
            ClaimMailRewardsStepCondition::WhenCollectAllDetected,
            "return claimed result",
            ClaimMailRewardsStepAction::ReturnResult {
                result: ClaimMailRewardsStepResult::Claimed,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Cleanup,
            ClaimMailRewardsStepCondition::WhenMailRewardMissing,
            "return no mail rewards result",
            ClaimMailRewardsStepAction::ReturnResult {
                result: ClaimMailRewardsStepResult::NoMailRewards,
            },
        ),
        ClaimMailRewardsStep::new(
            ClaimMailRewardsStepPhase::Cleanup,
            ClaimMailRewardsStepCondition::WhenCollectAllMissing,
            "return mail opened without collect-all result",
            ClaimMailRewardsStepAction::ReturnResult {
                result: ClaimMailRewardsStepResult::MailOpenedWithoutClaimAll,
            },
        ),
    ];

    Ok(ClaimMailRewardsExecutionPlan {
        task_key: CLAIM_MAIL_REWARDS_TASK_KEY.to_string(),
        display_name: "Claim Mail Rewards".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        locators,
        steps,
        notes: "Legacy mail rewards flow is represented as a return-menu-template-input plan with a Rust template/input executor boundary and desktop live menu/template/click/input adapter.".to_string(),
    })
}

fn claim_mail_rewards_locators(page: &BvPage) -> Result<ClaimMailRewardsLocators> {
    let mail_reward_roi = mail_reward_roi(page.capture_size)?;
    let collect_roi = collect_all_roi(page.capture_size)?;
    Ok(ClaimMailRewardsLocators {
        mail_reward_detect: image_locator(
            page,
            CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD,
            Some(mail_reward_roi),
            0.8,
            BvLocatorOperation::IsExist,
            Some(LOCATOR_TIMEOUT_MS),
        )?,
        mail_reward_click: image_locator(
            page,
            CLAIM_MAIL_REWARDS_ESC_MAIL_REWARD,
            Some(mail_reward_roi),
            0.8,
            BvLocatorOperation::Click,
            Some(LOCATOR_TIMEOUT_MS),
        )?,
        collect_all_detect: image_locator(
            page,
            CLAIM_MAIL_REWARDS_COLLECT,
            Some(collect_roi),
            0.8,
            BvLocatorOperation::IsExist,
            Some(LOCATOR_TIMEOUT_MS),
        )?,
        collect_all_click: image_locator(
            page,
            CLAIM_MAIL_REWARDS_COLLECT,
            Some(collect_roi),
            0.8,
            BvLocatorOperation::Click,
            Some(LOCATOR_TIMEOUT_MS),
        )?,
    })
}

fn mail_reward_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height / 2) as i32,
        (size.width / 10) as i32,
        (size.height / 2) as i32,
    ))
}

fn collect_all_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height - size.height / 3) as i32,
        (size.width / 4) as i32,
        (size.height / 3) as i32,
    ))
}
