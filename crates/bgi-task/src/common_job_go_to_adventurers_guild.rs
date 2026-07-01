use super::{
    plan_one_key_expedition_with_locators, task_vision_result, OneKeyExpeditionExecutionPlan,
    CHOOSE_TALK_OPTION_TASK_KEY, CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY,
    ONE_KEY_EXPEDITION_COLLECT, ONE_KEY_EXPEDITION_RE_DISPATCH, RETURN_MAIN_UI_PAIMON_MENU,
    RETURN_MAIN_UI_TASK_KEY, SWITCH_PARTY_TASK_KEY,
};
use crate::{Result, TaskPortState};
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const GO_TO_ADVENTURERS_GUILD_TASK_KEY: &str = "GoToAdventurersGuild";
pub const GO_TO_ADVENTURERS_GUILD_EXPEDITION_COLLECT: &str = ONE_KEY_EXPEDITION_COLLECT;
pub const GO_TO_ADVENTURERS_GUILD_EXPEDITION_RE: &str = ONE_KEY_EXPEDITION_RE_DISPATCH;
pub const GO_TO_ADVENTURERS_GUILD_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const GO_TO_ADVENTURERS_GUILD_DEFAULT_RETRY_TIMES: u8 = 1;
pub const GO_TO_ADVENTURERS_GUILD_DEFAULT_TALK_RETRY_TIMES: u8 = 3;
pub const GO_TO_ADVENTURERS_GUILD_DEFAULT_DAILY_SKIP_TIMES: u32 = 10;

const PATHING_JSON_PREFIX: &str = "GameTask/Common/Element/Assets/Json/冒险家协会_";
const PATHING_JSON_SUFFIX: &str = ".json";
const AFTER_PATHING_DELAY_MS: u32 = 600;
const TALK_RETRY_DELAY_MS: u32 = 500;
const DAILY_AFTER_CLICK_DELAY_MS: u32 = 800;
const DAILY_SELECT_LAST_OPTION_TIMES: u8 = 3;
const AFTER_PAIMON_MENU_DELAY_MS: u32 = 500;
const REOPEN_DIALOG_DELAY_MS: u32 = 1_200;
const EXPEDITION_AFTER_CLICK_DELAY_MS: u32 = 500;
const EXPEDITION_COLLECT_ATTEMPTS: u8 = 2;
const EXPEDITION_WAIT_BEFORE_RETRY_MS: u32 = 1_000;
const EXPEDITION_AFTER_COLLECT_DELAY_MS: u32 = 1_100;
const EXPEDITION_RETRY_ATTEMPTS: u8 = 3;
const EXPEDITION_RETRY_WINDOW_MS: u32 = 1_000;
const EXPEDITION_AFTER_RE_DISPATCH_DELAY_MS: u32 = 500;
const DEFAULT_INTERACT_VK: u16 = 0x46;
const VK_ESCAPE: u16 = 0x1B;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub retry_times: u8,
    pub country: String,
    pub daily_reward_party_name: Option<String>,
    pub only_do_once: bool,
    pub localized_texts: GoToAdventurersGuildLocalizedTexts,
    pub locators: GoToAdventurersGuildLocators,
    pub pathing_rule: GoToAdventurersGuildPathingRule,
    pub interaction_rule: GoToAdventurersGuildInteractionRule,
    pub daily_reward_rule: GoToAdventurersGuildDailyRewardRule,
    pub expedition_rule: GoToAdventurersGuildExpeditionRule,
    pub steps: Vec<GoToAdventurersGuildStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildLocalizedTexts {
    pub daily: String,
    pub catherine: String,
    pub expedition: String,
}

impl Default for GoToAdventurersGuildLocalizedTexts {
    fn default() -> Self {
        Self {
            daily: "每日".to_string(),
            catherine: "凯瑟琳".to_string(),
            expedition: "探索".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildLocators {
    pub black_confirm: BvLocatorPlan,
    pub paimon_menu: BvLocatorPlan,
    pub expedition_collect: BvLocatorPlan,
    pub expedition_re_dispatch: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildPathingRule {
    pub pathing_json: String,
    pub fail_when_task_missing: bool,
    pub party_config: GoToAdventurersGuildPathingPartyConfig,
    pub end_action_text: String,
    pub after_pathing_delay_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildPathingPartyConfig {
    pub enabled: bool,
    pub auto_skip_enabled: bool,
    pub auto_run_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildInteractionRule {
    pub retry_talk_times: u8,
    pub retry_delay_ms: u32,
    pub interact_vk: u16,
    pub interact_text: String,
    pub fail_when_talk_ui_missing_after_retries: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildDailyRewardRule {
    pub skip_times: u32,
    pub is_orange: bool,
    pub after_click_delay_ms: u32,
    pub black_confirm_optional: bool,
    pub select_last_option_until_end_times: u8,
    pub wait_paimon_menu_after_daily: bool,
    pub after_paimon_menu_delay_ms: u32,
    pub reopen_dialog_delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildExpeditionRule {
    pub one_key_plan: OneKeyExpeditionExecutionPlan,
    pub collect_locator: BvLocatorPlan,
    pub re_dispatch_locator: BvLocatorPlan,
    pub collect_attempts: u8,
    pub wait_before_retry_ms: u32,
    pub after_collect_delay_ms: u32,
    pub re_dispatch_retry_attempts: u8,
    pub re_dispatch_retry_window_ms: u32,
    pub after_re_dispatch_delay_ms: u32,
    pub exit_with_escape: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToAdventurersGuildStep {
    pub phase: GoToAdventurersGuildStepPhase,
    pub condition: GoToAdventurersGuildStepCondition,
    pub label: String,
    pub action: GoToAdventurersGuildStepAction,
}

impl GoToAdventurersGuildStep {
    fn new(
        phase: GoToAdventurersGuildStepPhase,
        condition: GoToAdventurersGuildStepCondition,
        label: impl Into<String>,
        action: GoToAdventurersGuildStepAction,
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
pub enum GoToAdventurersGuildStepPhase {
    Setup,
    Party,
    EncounterPoints,
    Pathing,
    DailyReward,
    Expedition,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToAdventurersGuildStepCondition {
    Always,
    WhenDailyRewardPartyConfigured,
    WhenOnlyDoOnceFalse,
    WhenDailyRewardOptionFound,
    AfterDailyRewardDialogueFinished,
    WhenExpeditionOptionFound,
    WhenTalkUiStillOpen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GoToAdventurersGuildStepAction {
    CommonJob {
        task_key: String,
        config: Option<Value>,
    },
    Pathing {
        rule: GoToAdventurersGuildPathingRule,
    },
    InteractionRetry {
        rule: GoToAdventurersGuildInteractionRule,
    },
    SelectLastTalkOptionUntilEnd {
        max_times: Option<u8>,
        until_paimon_menu: bool,
    },
    OneKeyExpedition {
        rule: GoToAdventurersGuildExpeditionRule,
    },
    Page {
        command: BvPageCommand,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Input {
        events: Vec<InputEvent>,
    },
    ReturnResult {
        result: GoToAdventurersGuildStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToAdventurersGuildStepResult {
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoToAdventurersGuildExecutionConfig {
    #[serde(alias = "Country")]
    pub country: String,
    #[serde(alias = "dailyRewardPartyName")]
    #[serde(alias = "DailyRewardPartyName")]
    pub daily_reward_party_name: Option<String>,
    #[serde(alias = "onlyDoOnce")]
    #[serde(alias = "OnlyDoOnce")]
    pub only_do_once: bool,
    #[serde(alias = "dailyText")]
    #[serde(alias = "DailyText")]
    pub daily_text: String,
    #[serde(alias = "catherineText")]
    #[serde(alias = "CatherineText")]
    pub catherine_text: String,
    #[serde(alias = "expeditionText")]
    #[serde(alias = "ExpeditionText")]
    pub expedition_text: String,
    #[serde(alias = "interactVk")]
    #[serde(alias = "InteractVk")]
    pub interact_vk: u16,
    pub capture_size: Size,
}

impl Default for GoToAdventurersGuildExecutionConfig {
    fn default() -> Self {
        let texts = GoToAdventurersGuildLocalizedTexts::default();
        Self {
            country: String::new(),
            daily_reward_party_name: None,
            only_do_once: false,
            daily_text: texts.daily,
            catherine_text: texts.catherine,
            expedition_text: texts.expedition,
            interact_vk: DEFAULT_INTERACT_VK,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl GoToAdventurersGuildExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.interact_vk == 0 {
            config.interact_vk = DEFAULT_INTERACT_VK;
        }
        if config.daily_text.is_empty() {
            config.daily_text = GoToAdventurersGuildLocalizedTexts::default().daily;
        }
        if config.catherine_text.is_empty() {
            config.catherine_text = GoToAdventurersGuildLocalizedTexts::default().catherine;
        }
        if config.expedition_text.is_empty() {
            config.expedition_text = GoToAdventurersGuildLocalizedTexts::default().expedition;
        }
        config.daily_reward_party_name = config
            .daily_reward_party_name
            .filter(|name| !name.trim().is_empty());
        config
    }

    pub(super) fn localized_texts(&self) -> GoToAdventurersGuildLocalizedTexts {
        GoToAdventurersGuildLocalizedTexts {
            daily: self.daily_text.clone(),
            catherine: self.catherine_text.clone(),
            expedition: self.expedition_text.clone(),
        }
    }
}

pub fn plan_go_to_adventurers_guild(
    capture_size: Size,
    country: impl Into<String>,
    daily_reward_party_name: Option<String>,
    only_do_once: bool,
    localized_texts: GoToAdventurersGuildLocalizedTexts,
    interact_vk: u16,
) -> Result<GoToAdventurersGuildExecutionPlan> {
    let country = country.into();
    let interact_vk = if interact_vk == 0 {
        DEFAULT_INTERACT_VK
    } else {
        interact_vk
    };
    let page = BvPage {
        capture_size,
        ..BvPage::default()
    };
    let locators = go_to_adventurers_guild_locators(&page)?;
    let pathing_rule = GoToAdventurersGuildPathingRule {
        pathing_json: format!("{PATHING_JSON_PREFIX}{country}{PATHING_JSON_SUFFIX}"),
        fail_when_task_missing: true,
        party_config: GoToAdventurersGuildPathingPartyConfig {
            enabled: true,
            auto_skip_enabled: true,
            auto_run_enabled: true,
        },
        end_action_text: localized_texts.catherine.clone(),
        after_pathing_delay_ms: AFTER_PATHING_DELAY_MS,
    };
    let interaction_rule = GoToAdventurersGuildInteractionRule {
        retry_talk_times: GO_TO_ADVENTURERS_GUILD_DEFAULT_TALK_RETRY_TIMES,
        retry_delay_ms: TALK_RETRY_DELAY_MS,
        interact_vk,
        interact_text: localized_texts.catherine.clone(),
        fail_when_talk_ui_missing_after_retries: true,
    };
    let daily_reward_rule = GoToAdventurersGuildDailyRewardRule {
        skip_times: GO_TO_ADVENTURERS_GUILD_DEFAULT_DAILY_SKIP_TIMES,
        is_orange: true,
        after_click_delay_ms: DAILY_AFTER_CLICK_DELAY_MS,
        black_confirm_optional: true,
        select_last_option_until_end_times: DAILY_SELECT_LAST_OPTION_TIMES,
        wait_paimon_menu_after_daily: true,
        after_paimon_menu_delay_ms: AFTER_PAIMON_MENU_DELAY_MS,
        reopen_dialog_delay_ms: REOPEN_DIALOG_DELAY_MS,
    };
    let expedition_rule = GoToAdventurersGuildExpeditionRule {
        one_key_plan: plan_one_key_expedition_with_locators(
            capture_size,
            locators.expedition_collect.clone(),
            locators.expedition_re_dispatch.clone(),
        )?,
        collect_locator: locators.expedition_collect.clone(),
        re_dispatch_locator: locators.expedition_re_dispatch.clone(),
        collect_attempts: EXPEDITION_COLLECT_ATTEMPTS,
        wait_before_retry_ms: EXPEDITION_WAIT_BEFORE_RETRY_MS,
        after_collect_delay_ms: EXPEDITION_AFTER_COLLECT_DELAY_MS,
        re_dispatch_retry_attempts: EXPEDITION_RETRY_ATTEMPTS,
        re_dispatch_retry_window_ms: EXPEDITION_RETRY_WINDOW_MS,
        after_re_dispatch_delay_ms: EXPEDITION_AFTER_RE_DISPATCH_DELAY_MS,
        exit_with_escape: true,
    };
    let mut steps = Vec::new();

    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Setup,
        GoToAdventurersGuildStepCondition::Always,
        "log go-to-adventurers-guild start",
        GoToAdventurersGuildStepAction::Log {
            message: format!("start GoToAdventurersGuild common job plan for {country}"),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Party,
        GoToAdventurersGuildStepCondition::WhenDailyRewardPartyConfigured,
        "switch to daily reward party",
        GoToAdventurersGuildStepAction::CommonJob {
            task_key: SWITCH_PARTY_TASK_KEY.to_string(),
            config: daily_reward_party_name
                .as_ref()
                .map(|party_name| json!({ "partyName": party_name, "captureSize": capture_size })),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::EncounterPoints,
        GoToAdventurersGuildStepCondition::WhenOnlyDoOnceFalse,
        "claim encounter points before talking to Catherine",
        GoToAdventurersGuildStepAction::CommonJob {
            task_key: CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY.to_string(),
            config: Some(json!({ "captureSize": capture_size })),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Pathing,
        GoToAdventurersGuildStepCondition::Always,
        "path to adventurers guild and press interaction",
        GoToAdventurersGuildStepAction::Pathing {
            rule: pathing_rule.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Pathing,
        GoToAdventurersGuildStepCondition::Always,
        "wait after pathing",
        GoToAdventurersGuildStepAction::Page {
            command: task_vision_result(page.wait(AFTER_PATHING_DELAY_MS))?,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Pathing,
        GoToAdventurersGuildStepCondition::Always,
        "retry Catherine interaction until talk UI opens",
        GoToAdventurersGuildStepAction::InteractionRetry {
            rule: interaction_rule.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::Always,
        "select daily reward talk option",
        GoToAdventurersGuildStepAction::CommonJob {
            task_key: CHOOSE_TALK_OPTION_TASK_KEY.to_string(),
            config: Some(json!({
                "option": localized_texts.daily.clone(),
                "skipTimes": daily_reward_rule.skip_times,
                "isOrange": daily_reward_rule.is_orange,
                "captureSize": capture_size
            })),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::WhenDailyRewardOptionFound,
        "wait after clicking daily reward option",
        GoToAdventurersGuildStepAction::Page {
            command: task_vision_result(page.wait(DAILY_AFTER_CLICK_DELAY_MS))?,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::WhenDailyRewardOptionFound,
        "click optional daily reward black confirm prompt",
        GoToAdventurersGuildStepAction::Locator {
            locator: locators.black_confirm.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::WhenDailyRewardOptionFound,
        "select trailing dialogue options after daily reward",
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd {
            max_times: Some(DAILY_SELECT_LAST_OPTION_TIMES),
            until_paimon_menu: true,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "wait until main UI is visible after daily reward",
        GoToAdventurersGuildStepAction::Locator {
            locator: locators.paimon_menu.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "wait after Paimon menu appears",
        GoToAdventurersGuildStepAction::Page {
            command: task_vision_result(page.wait(AFTER_PAIMON_MENU_DELAY_MS))?,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "press Escape after daily reward dialogue",
        GoToAdventurersGuildStepAction::Input {
            events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "return to main UI after daily reward",
        GoToAdventurersGuildStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: Some(json!({ "captureSize": capture_size })),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "wait before reopening Catherine dialogue",
        GoToAdventurersGuildStepAction::Page {
            command: task_vision_result(page.wait(REOPEN_DIALOG_DELAY_MS))?,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::DailyReward,
        GoToAdventurersGuildStepCondition::AfterDailyRewardDialogueFinished,
        "reopen Catherine dialogue",
        GoToAdventurersGuildStepAction::InteractionRetry {
            rule: interaction_rule.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Expedition,
        GoToAdventurersGuildStepCondition::Always,
        "select expedition talk option",
        GoToAdventurersGuildStepAction::CommonJob {
            task_key: CHOOSE_TALK_OPTION_TASK_KEY.to_string(),
            config: Some(json!({
                "option": localized_texts.expedition.clone(),
                "skipTimes": daily_reward_rule.skip_times,
                "isOrange": daily_reward_rule.is_orange,
                "captureSize": capture_size
            })),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Expedition,
        GoToAdventurersGuildStepCondition::WhenExpeditionOptionFound,
        "wait after clicking expedition option",
        GoToAdventurersGuildStepAction::Page {
            command: task_vision_result(page.wait(EXPEDITION_AFTER_CLICK_DELAY_MS))?,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Expedition,
        GoToAdventurersGuildStepCondition::WhenExpeditionOptionFound,
        "run one-key expedition",
        GoToAdventurersGuildStepAction::OneKeyExpedition {
            rule: expedition_rule.clone(),
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Cleanup,
        GoToAdventurersGuildStepCondition::WhenTalkUiStillOpen,
        "select last option to exit remaining dialogue",
        GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd {
            max_times: None,
            until_paimon_menu: false,
        },
    ));
    steps.push(GoToAdventurersGuildStep::new(
        GoToAdventurersGuildStepPhase::Cleanup,
        GoToAdventurersGuildStepCondition::Always,
        "return completed result",
        GoToAdventurersGuildStepAction::ReturnResult {
            result: GoToAdventurersGuildStepResult::Completed,
        },
    ));

    Ok(GoToAdventurersGuildExecutionPlan {
        task_key: GO_TO_ADVENTURERS_GUILD_TASK_KEY.to_string(),
        display_name: "Go To Adventurers Guild".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        retry_times: GO_TO_ADVENTURERS_GUILD_DEFAULT_RETRY_TIMES,
        country,
        daily_reward_party_name,
        only_do_once,
        localized_texts,
        locators,
        pathing_rule,
        interaction_rule,
        daily_reward_rule,
        expedition_rule,
        steps,
        notes: "Legacy Adventurers' Guild pathing, Catherine interaction, daily reward dialogue, and one-key expedition flow are represented and executable as a Rust state machine through injectable hooks; bundled PathExecutor JSON can be converted through the shared AutoPathing action boundary with legacy TrackMap conversion, and the desktop live route consumes the shared movement contract before handing teleport, nested common-job, interaction, OCR, and click work to live adapters. Existing nested common-job bridges plus capture/OCR/send-input adapters for Catherine F-interaction retry and legacy-style talk-option draining with Paimon-menu stop checks are wired while full desktop movement dispatch and later real-game OCR/expedition click regression gaps remain pending.".to_string(),
    })
}

fn go_to_adventurers_guild_locators(page: &BvPage) -> Result<GoToAdventurersGuildLocators> {
    Ok(GoToAdventurersGuildLocators {
        black_confirm: image_locator(
            page,
            GO_TO_ADVENTURERS_GUILD_BLACK_CONFIRM,
            None,
            0.8,
            true,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        paimon_menu: image_locator(
            page,
            RETURN_MAIN_UI_PAIMON_MENU,
            Some(top_left_quarter_rect(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::WaitFor,
            Some(10_000),
        )?,
        expedition_collect: image_locator(
            page,
            GO_TO_ADVENTURERS_GUILD_EXPEDITION_COLLECT,
            Some(expedition_collect_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        expedition_re_dispatch: image_locator(
            page,
            GO_TO_ADVENTURERS_GUILD_EXPEDITION_RE,
            Some(expedition_re_dispatch_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
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

fn top_left_quarter_rect(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}

fn expedition_collect_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        (size.height - size.height / 3) as i32,
        (size.width / 4) as i32,
        (size.height / 3) as i32,
    ))
}

fn expedition_re_dispatch_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        (size.height - size.height / 4) as i32,
        (size.width / 4) as i32,
        (size.height / 4) as i32,
    ))
}
