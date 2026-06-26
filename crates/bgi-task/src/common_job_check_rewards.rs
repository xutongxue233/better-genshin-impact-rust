use super::{task_vision_result, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::{GenshinAction, NotificationPayload};
use bgi_vision::{BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CHECK_REWARDS_TASK_KEY: &str = "CheckRewards";
pub const CHECK_REWARDS_NOTIFICATION_EVENT: &str = "daily.reward";
pub const CHECK_REWARDS_DEFAULT_DAILY_REWARD_TEXT: &str = "每日委托奖励";
pub const CHECK_REWARDS_DEFAULT_COMMISSIONS_TEXT: &str = "委托";
pub const CHECK_REWARDS_DEFAULT_CLAIMED_TEXT: &str = "今日奖励已领取";
pub const CHECK_REWARDS_SUCCESS_MESSAGE: &str = "检查每日奖励：已领取";
pub const CHECK_REWARDS_FAILURE_MESSAGE: &str = "检查到每日奖励未领取，请手动查看！";

const DEFAULT_OPEN_RETRIES: u8 = 4;
const DEFAULT_OPEN_RETRY_INTERVAL_MS: u32 = 1_000;
const DEFAULT_CLAIM_CHECK_RETRIES: u8 = 4;
const DEFAULT_CLAIM_CHECK_INTERVAL_MS: u32 = 500;
const DEFAULT_FINAL_DELAY_MS: u32 = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub localized_texts: CheckRewardsLocalizedTexts,
    pub locators: CheckRewardsLocators,
    pub open_handbook_rule: CheckRewardsRetryRule,
    pub claim_check_rule: CheckRewardsRetryRule,
    pub notifications: CheckRewardsNotifications,
    pub steps: Vec<CheckRewardsStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRewardsLocalizedTexts {
    pub daily_reward_text: String,
    pub commissions_text: String,
    pub claimed_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsLocators {
    pub daily_reward_title: BvLocatorPlan,
    pub commissions_ocr: BvLocatorPlan,
    pub claimed_text: BvLocatorPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRewardsRetryRule {
    pub max_retries: u8,
    pub interval_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsNotifications {
    pub claimed: NotificationPayload,
    pub unclaimed: NotificationPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRewardsStep {
    pub phase: CheckRewardsStepPhase,
    pub condition: CheckRewardsStepCondition,
    pub label: String,
    pub action: CheckRewardsStepAction,
}

impl CheckRewardsStep {
    fn new(
        phase: CheckRewardsStepPhase,
        condition: CheckRewardsStepCondition,
        label: impl Into<String>,
        action: CheckRewardsStepAction,
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
pub enum CheckRewardsStepPhase {
    Setup,
    OpenHandbook,
    ClaimStatus,
    Notify,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckRewardsStepCondition {
    Always,
    EachOpenRetry,
    WhenCommissionsTextMatched,
    WhenDailyRewardTitleDetected,
    WhenClaimedTextDetected,
    WhenClaimedTextMissing,
    AfterStatusCheck,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CheckRewardsStepAction {
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
        text: String,
        click_first_match: bool,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Notify {
        payload: NotificationPayload,
    },
    ReturnResult {
        result: CheckRewardsStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckRewardsStepResult {
    Claimed,
    UnclaimedManualCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckRewardsExecutionConfig {
    pub capture_size: Size,
    pub daily_reward_text: String,
    pub commissions_text: String,
    pub claimed_text: String,
    pub max_open_retries: u8,
    pub open_retry_interval_ms: u32,
    pub max_claim_check_retries: u8,
    pub claim_check_interval_ms: u32,
    pub final_delay_ms: u32,
}

impl Default for CheckRewardsExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
            daily_reward_text: CHECK_REWARDS_DEFAULT_DAILY_REWARD_TEXT.to_string(),
            commissions_text: CHECK_REWARDS_DEFAULT_COMMISSIONS_TEXT.to_string(),
            claimed_text: CHECK_REWARDS_DEFAULT_CLAIMED_TEXT.to_string(),
            max_open_retries: DEFAULT_OPEN_RETRIES,
            open_retry_interval_ms: DEFAULT_OPEN_RETRY_INTERVAL_MS,
            max_claim_check_retries: DEFAULT_CLAIM_CHECK_RETRIES,
            claim_check_interval_ms: DEFAULT_CLAIM_CHECK_INTERVAL_MS,
            final_delay_ms: DEFAULT_FINAL_DELAY_MS,
        }
    }
}

impl CheckRewardsExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        let defaults = CheckRewardsExecutionConfig::default();
        if config.daily_reward_text.trim().is_empty() {
            config.daily_reward_text = defaults.daily_reward_text;
        }
        if config.commissions_text.trim().is_empty() {
            config.commissions_text = defaults.commissions_text;
        }
        if config.claimed_text.trim().is_empty() {
            config.claimed_text = defaults.claimed_text;
        }
        if config.max_open_retries == 0 {
            config.max_open_retries = defaults.max_open_retries;
        }
        if config.open_retry_interval_ms == 0 {
            config.open_retry_interval_ms = defaults.open_retry_interval_ms;
        }
        if config.max_claim_check_retries == 0 {
            config.max_claim_check_retries = defaults.max_claim_check_retries;
        }
        if config.claim_check_interval_ms == 0 {
            config.claim_check_interval_ms = defaults.claim_check_interval_ms;
        }
        if config.final_delay_ms == 0 {
            config.final_delay_ms = defaults.final_delay_ms;
        }
        config
    }
}

pub fn plan_check_rewards(
    config: CheckRewardsExecutionConfig,
) -> Result<CheckRewardsExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let roi = check_rewards_roi(config.capture_size)?;
    let localized_texts = CheckRewardsLocalizedTexts {
        daily_reward_text: config.daily_reward_text,
        commissions_text: config.commissions_text,
        claimed_text: config.claimed_text,
    };
    let locators = CheckRewardsLocators {
        daily_reward_title: text_locator(
            &page,
            &localized_texts.daily_reward_text,
            Some(roi),
            BvLocatorOperation::WaitFor,
            Some(config.open_retry_interval_ms),
        ),
        commissions_ocr: page.locator_for_text("", Some(roi)).plan(
            BvLocatorOperation::FindAll,
            Some(config.open_retry_interval_ms),
        ),
        claimed_text: text_locator(
            &page,
            &localized_texts.claimed_text,
            Some(roi),
            BvLocatorOperation::WaitFor,
            Some(config.claim_check_interval_ms),
        ),
    };
    let open_handbook_rule = CheckRewardsRetryRule {
        max_retries: config.max_open_retries,
        interval_ms: config.open_retry_interval_ms,
    };
    let claim_check_rule = CheckRewardsRetryRule {
        max_retries: config.max_claim_check_retries,
        interval_ms: config.claim_check_interval_ms,
    };
    let notifications = CheckRewardsNotifications {
        claimed: NotificationPayload::success(
            CHECK_REWARDS_NOTIFICATION_EVENT,
            CHECK_REWARDS_SUCCESS_MESSAGE,
        ),
        unclaimed: NotificationPayload::fail(
            CHECK_REWARDS_NOTIFICATION_EVENT,
            CHECK_REWARDS_FAILURE_MESSAGE,
        ),
    };
    let steps = vec![
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Setup,
            CheckRewardsStepCondition::Always,
            "log check rewards start",
            CheckRewardsStepAction::Log {
                message: "start CheckRewards common job plan".to_string(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Setup,
            CheckRewardsStepCondition::Always,
            "return to main UI before checking rewards",
            CheckRewardsStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::OpenHandbook,
            CheckRewardsStepCondition::EachOpenRetry,
            "open adventurer handbook",
            CheckRewardsStepAction::GenshinAction {
                action: GenshinAction::OpenAdventurerHandbook,
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::OpenHandbook,
            CheckRewardsStepCondition::EachOpenRetry,
            "OCR adventurer handbook left panel",
            CheckRewardsStepAction::Ocr {
                command: BvPageCommand::Ocr {
                    locator: locators.commissions_ocr.clone(),
                },
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::OpenHandbook,
            CheckRewardsStepCondition::EachOpenRetry,
            "match and click commissions tab",
            CheckRewardsStepAction::MatchCommissions {
                text: localized_texts.commissions_text.clone(),
                click_first_match: true,
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::OpenHandbook,
            CheckRewardsStepCondition::WhenCommissionsTextMatched,
            "wait for daily commission reward title",
            CheckRewardsStepAction::Locator {
                locator: locators.daily_reward_title.clone(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::ClaimStatus,
            CheckRewardsStepCondition::WhenDailyRewardTitleDetected,
            "wait for claimed reward text",
            CheckRewardsStepAction::Locator {
                locator: locators.claimed_text.clone(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Notify,
            CheckRewardsStepCondition::WhenClaimedTextDetected,
            "notify claimed daily reward status",
            CheckRewardsStepAction::Notify {
                payload: notifications.claimed.clone(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Notify,
            CheckRewardsStepCondition::WhenClaimedTextDetected,
            "return claimed result",
            CheckRewardsStepAction::ReturnResult {
                result: CheckRewardsStepResult::Claimed,
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Notify,
            CheckRewardsStepCondition::WhenClaimedTextMissing,
            "notify unclaimed daily reward status",
            CheckRewardsStepAction::Notify {
                payload: notifications.unclaimed.clone(),
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Notify,
            CheckRewardsStepCondition::WhenClaimedTextMissing,
            "return unclaimed result",
            CheckRewardsStepAction::ReturnResult {
                result: CheckRewardsStepResult::UnclaimedManualCheck,
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Cleanup,
            CheckRewardsStepCondition::AfterStatusCheck,
            "wait before returning to main UI",
            CheckRewardsStepAction::Page {
                command: task_vision_result(page.wait(config.final_delay_ms))?,
            },
        ),
        CheckRewardsStep::new(
            CheckRewardsStepPhase::Cleanup,
            CheckRewardsStepCondition::AfterStatusCheck,
            "return to main UI after checking rewards",
            CheckRewardsStepAction::CommonJob {
                task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            },
        ),
    ];

    Ok(CheckRewardsExecutionPlan {
        task_key: CHECK_REWARDS_TASK_KEY.to_string(),
        display_name: "Check Rewards".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        localized_texts,
        locators,
        open_handbook_rule,
        claim_check_rule,
        notifications,
        steps,
        notes: "Legacy daily reward check is represented as a handbook OCR and notification plan with an injectable OCR/click/notification executor and desktop WinRT OCR/capture-click adapter; live OCR still depends on installed Windows OCR language support.".to_string(),
    })
}

fn text_locator(
    page: &BvPage,
    text: &str,
    roi: Option<Rect>,
    operation: BvLocatorOperation,
    timeout_ms: Option<u32>,
) -> BvLocatorPlan {
    page.locator_for_text(text, roi).plan(operation, timeout_ms)
}

fn check_rewards_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 10) as i32,
        (size.height / 10) as i32,
        (size.width * 3 / 10) as i32,
        (size.height * 7 / 10) as i32,
    ))
}
