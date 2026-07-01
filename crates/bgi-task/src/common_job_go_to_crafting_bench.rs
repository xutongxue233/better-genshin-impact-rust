use super::{task_vision_result, RETURN_MAIN_UI_TASK_KEY};
use crate::{Result, TaskPortState};
use bgi_core::GenshinAction;
use bgi_input::{InputEvent, InputSequence};
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const GO_TO_CRAFTING_BENCH_TASK_KEY: &str = "GoToCraftingBench";
pub const GO_TO_CRAFTING_BENCH_TALK_UI: &str = "AutoSkip:disabled_ui.png";
pub const GO_TO_CRAFTING_BENCH_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";
pub const GO_TO_CRAFTING_BENCH_BLACK_CONFIRM: &str = "Common/Element:btn_black_confirm.png";
pub const GO_TO_CRAFTING_BENCH_CONDENSED_RESIN: &str = "Common/Element:craft_condensed_resin.png";
pub const GO_TO_CRAFTING_BENCH_FRAGILE_RESIN_COUNT: &str = "Common/Element:fragile_resin_count.png";
pub const GO_TO_CRAFTING_BENCH_CONDENSED_RESIN_COUNT: &str =
    "Common/Element:condensed_resin_count.png";
pub const GO_TO_CRAFTING_BENCH_KEY_REDUCE: &str = "Common/Element:key_reduce.png";
pub const GO_TO_CRAFTING_BENCH_KEY_INCREASE: &str = "Common/Element:key_increase.png";
pub const GO_TO_CRAFTING_BENCH_DEFAULT_RETRY_TIMES: u8 = 2;

const PATHING_JSON_PREFIX: &str = "GameTask/Common/Element/Assets/Json/合成台_";
const PATHING_JSON_SUFFIX: &str = ".json";
const DEFAULT_CRAFT_TEXT: &str = "合成";
const DEFAULT_INTERACT_VK: u16 = 0x46;
const VK_ESCAPE: u16 = 0x1B;
const AFTER_RETRY_FAILURE_DELAY_MS: u32 = 1_000;
const AFTER_PATHING_DELAY_MS: u32 = 700;
const INTERACT_SUCCESS_DELAY_MS: u32 = 1_000;
const MOVE_BACKWARD_HOLD_MS: u32 = 200;
const AFTER_CRAFTING_PAGE_DELAY_MS: u32 = 800;
const CONDENSED_COUNT_RETRIES: u8 = 3;
const CONDENSED_COUNT_RETRY_DELAY_MS: u32 = 200;
const RESIN_CONSUMED_PER_CRAFT: u16 = 60;
const MAX_CONDENSED_RESIN_COUNT: u8 = 5;
const REDUCE_CLICKS_BEFORE_ADD: u8 = 5;
const REDUCE_CLICK_DELAY_MS: u32 = 150;
const AFTER_REDUCE_DELAY_MS: u32 = 300;
const ADD_CLICK_DELAY_MS: u32 = 150;
const AFTER_ADD_DELAY_MS: u32 = 200;
const AFTER_WHITE_CONFIRM_DELAY_MS: u32 = 300;
const AFTER_CRAFT_DELAY_MS: u32 = 1_300;
const FRAGILE_RESIN_REGEX: &str = r"(\d+)\s*[/17]\s*(6|60)";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub retry_times: u8,
    pub retry_failure_delay_ms: u32,
    pub country: String,
    pub localized_texts: GoToCraftingBenchLocalizedTexts,
    pub min_resin_to_keep: i32,
    pub locators: GoToCraftingBenchLocators,
    pub pathing_rule: GoToCraftingBenchPathingRule,
    pub interaction_rule: GoToCraftingBenchInteractionRule,
    pub crafting_page_rule: GoToCraftingBenchCraftingPageRule,
    pub resin_recognition_rule: GoToCraftingBenchResinRecognitionRule,
    pub resin_craft_rule: GoToCraftingBenchResinCraftRule,
    pub steps: Vec<GoToCraftingBenchStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchLocalizedTexts {
    pub craft: String,
}

impl Default for GoToCraftingBenchLocalizedTexts {
    fn default() -> Self {
        Self {
            craft: DEFAULT_CRAFT_TEXT.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchLocators {
    pub talk_ui: BvLocatorPlan,
    pub white_confirm: BvLocatorPlan,
    pub black_confirm: BvLocatorPlan,
    pub condensed_resin: BvLocatorPlan,
    pub fragile_resin_count: BvLocatorPlan,
    pub condensed_resin_count: BvLocatorPlan,
    pub key_reduce: BvLocatorPlan,
    pub key_increase: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchPathingRule {
    pub pathing_json: String,
    pub fail_when_task_missing: bool,
    pub party_config: GoToCraftingBenchPathingPartyConfig,
    pub end_action_text: String,
    pub after_pathing_delay_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchPathingPartyConfig {
    pub enabled: bool,
    pub auto_skip_enabled: bool,
    pub auto_run_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoToCraftingBenchInteractionRule {
    pub interact_vk: u16,
    pub interact_text: String,
    pub interact_success_delay_ms: u32,
    pub move_backward_hold_ms: u32,
    pub fail_message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchCraftingPageRule {
    pub end_locator: BvLocatorPlan,
    pub after_page_delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchResinRecognitionRule {
    pub craft_condensed_resin_locator: BvLocatorPlan,
    pub fragile_resin_count_locator: BvLocatorPlan,
    pub condensed_resin_count_locator: BvLocatorPlan,
    pub fragile_count_ocr_crop: GoToCraftingBenchRelativeCrop,
    pub condensed_count_ocr_crop: GoToCraftingBenchRelativeCrop,
    pub fragile_resin_regex: String,
    pub condensed_count_retries: u8,
    pub condensed_count_retry_delay_ms: u32,
    pub initial_condensed_count: i32,
    pub valid_condensed_count_min: i32,
    pub valid_condensed_count_max: i32,
    pub return_main_ui_on_count_failure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchRelativeCrop {
    pub anchor: GoToCraftingBenchCropAnchor,
    pub x_offset_width_multiplier: f64,
    pub y_offset_height_multiplier: f64,
    pub width_multiplier: f64,
    pub height_multiplier: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchCropAnchor {
    MatchedLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchResinCraftRule {
    pub min_resin_to_keep: i32,
    pub resin_consumed_per_craft: u16,
    pub max_condensed_resin_count: u8,
    pub reduce_clicks_before_add: u8,
    pub reduce_click_delay_ms: u32,
    pub after_reduce_delay_ms: u32,
    pub add_click_delay_ms: u32,
    pub after_add_delay_ms: u32,
    pub after_white_confirm_delay_ms: u32,
    pub after_craft_delay_ms: u32,
    pub direct_confirm_when_min_resin_to_keep_disabled: bool,
    pub escape_after_craft: bool,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoToCraftingBenchStep {
    pub phase: GoToCraftingBenchStepPhase,
    pub condition: GoToCraftingBenchStepCondition,
    pub label: String,
    pub action: GoToCraftingBenchStepAction,
}

impl GoToCraftingBenchStep {
    fn new(
        phase: GoToCraftingBenchStepPhase,
        condition: GoToCraftingBenchStepCondition,
        label: impl Into<String>,
        action: GoToCraftingBenchStepAction,
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
pub enum GoToCraftingBenchStepPhase {
    Setup,
    Pathing,
    Interaction,
    CraftingPage,
    ResinDetection,
    ResinCraft,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchStepCondition {
    Always,
    WhenTalkUiMissing,
    WhenTalkUiStillMissing,
    WhenCraftingPageOpen,
    WhenCondensedResinVisible,
    WhenMinResinToKeepEnabled,
    WhenResinCountRecognitionFailed,
    WhenCraftsNeeded,
    WhenMinResinToKeepDisabled,
    WhenCrafted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GoToCraftingBenchStepAction {
    Pathing {
        rule: GoToCraftingBenchPathingRule,
    },
    InteractionRetry {
        rule: GoToCraftingBenchInteractionRule,
    },
    GenshinAction {
        action: GenshinAction,
        press: GoToCraftingBenchActionPress,
    },
    SelectLastTalkOptionUntilEnd {
        until_locator: BvLocatorPlan,
    },
    DetectResin {
        locator: BvLocatorPlan,
    },
    RecognizeResinCounts {
        rule: GoToCraftingBenchResinRecognitionRule,
    },
    ComputeCraftsNeeded {
        rule: GoToCraftingBenchResinCraftRule,
    },
    CraftCondensedResin {
        rule: GoToCraftingBenchResinCraftRule,
    },
    CommonJob {
        task_key: String,
        config: Option<Value>,
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
        result: GoToCraftingBenchStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchActionPress {
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoToCraftingBenchStepResult {
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GoToCraftingBenchExecutionConfig {
    #[serde(alias = "Country")]
    pub country: String,
    #[serde(alias = "craftText")]
    #[serde(alias = "CraftText")]
    pub craft_text: String,
    #[serde(alias = "minResinToKeep")]
    #[serde(alias = "MinResinToKeep")]
    pub min_resin_to_keep: i32,
    #[serde(alias = "interactVk")]
    #[serde(alias = "InteractVk")]
    pub interact_vk: u16,
    pub capture_size: Size,
}

impl Default for GoToCraftingBenchExecutionConfig {
    fn default() -> Self {
        Self {
            country: String::new(),
            craft_text: DEFAULT_CRAFT_TEXT.to_string(),
            min_resin_to_keep: 0,
            interact_vk: DEFAULT_INTERACT_VK,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl GoToCraftingBenchExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config: Self = value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default();
        if config.craft_text.is_empty() {
            config.craft_text = DEFAULT_CRAFT_TEXT.to_string();
        }
        if config.interact_vk == 0 {
            config.interact_vk = DEFAULT_INTERACT_VK;
        }
        config
    }

    pub(super) fn localized_texts(&self) -> GoToCraftingBenchLocalizedTexts {
        GoToCraftingBenchLocalizedTexts {
            craft: self.craft_text.clone(),
        }
    }
}

pub fn plan_go_to_crafting_bench(
    capture_size: Size,
    country: impl Into<String>,
    localized_texts: GoToCraftingBenchLocalizedTexts,
    min_resin_to_keep: i32,
    interact_vk: u16,
) -> Result<GoToCraftingBenchExecutionPlan> {
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
    let locators = go_to_crafting_bench_locators(&page)?;
    let pathing_rule = GoToCraftingBenchPathingRule {
        pathing_json: format!("{PATHING_JSON_PREFIX}{country}{PATHING_JSON_SUFFIX}"),
        fail_when_task_missing: true,
        party_config: GoToCraftingBenchPathingPartyConfig {
            enabled: true,
            auto_skip_enabled: true,
            auto_run_enabled: country != "枫丹",
        },
        end_action_text: localized_texts.craft.clone(),
        after_pathing_delay_ms: AFTER_PATHING_DELAY_MS,
    };
    let interaction_rule = GoToCraftingBenchInteractionRule {
        interact_vk,
        interact_text: localized_texts.craft.clone(),
        interact_success_delay_ms: INTERACT_SUCCESS_DELAY_MS,
        move_backward_hold_ms: MOVE_BACKWARD_HOLD_MS,
        fail_message: "未进入和合成台交互对话界面".to_string(),
    };
    let crafting_page_rule = GoToCraftingBenchCraftingPageRule {
        end_locator: locators.white_confirm.clone(),
        after_page_delay_ms: AFTER_CRAFTING_PAGE_DELAY_MS,
    };
    let resin_recognition_rule = GoToCraftingBenchResinRecognitionRule {
        craft_condensed_resin_locator: locators.condensed_resin.clone(),
        fragile_resin_count_locator: locators.fragile_resin_count.clone(),
        condensed_resin_count_locator: locators.condensed_resin_count.clone(),
        fragile_count_ocr_crop: GoToCraftingBenchRelativeCrop {
            anchor: GoToCraftingBenchCropAnchor::MatchedLocator,
            x_offset_width_multiplier: 0.0,
            y_offset_height_multiplier: 1.0,
            width_multiplier: 1.0,
            height_multiplier: 1.0,
        },
        condensed_count_ocr_crop: GoToCraftingBenchRelativeCrop {
            anchor: GoToCraftingBenchCropAnchor::MatchedLocator,
            x_offset_width_multiplier: 1.0,
            y_offset_height_multiplier: 0.0,
            width_multiplier: 5.0 / 3.0,
            height_multiplier: 1.0,
        },
        fragile_resin_regex: FRAGILE_RESIN_REGEX.to_string(),
        condensed_count_retries: CONDENSED_COUNT_RETRIES,
        condensed_count_retry_delay_ms: CONDENSED_COUNT_RETRY_DELAY_MS,
        initial_condensed_count: 0,
        valid_condensed_count_min: 0,
        valid_condensed_count_max: MAX_CONDENSED_RESIN_COUNT as i32,
        return_main_ui_on_count_failure: true,
    };
    let resin_craft_rule = GoToCraftingBenchResinCraftRule {
        min_resin_to_keep,
        resin_consumed_per_craft: RESIN_CONSUMED_PER_CRAFT,
        max_condensed_resin_count: MAX_CONDENSED_RESIN_COUNT,
        reduce_clicks_before_add: REDUCE_CLICKS_BEFORE_ADD,
        reduce_click_delay_ms: REDUCE_CLICK_DELAY_MS,
        after_reduce_delay_ms: AFTER_REDUCE_DELAY_MS,
        add_click_delay_ms: ADD_CLICK_DELAY_MS,
        after_add_delay_ms: AFTER_ADD_DELAY_MS,
        after_white_confirm_delay_ms: AFTER_WHITE_CONFIRM_DELAY_MS,
        after_craft_delay_ms: AFTER_CRAFT_DELAY_MS,
        direct_confirm_when_min_resin_to_keep_disabled: true,
        escape_after_craft: true,
        formula: "craftsNeeded = min(5 - condensedResinCount, max(0, (fragileResinCount - minResinToKeep) / 60))".to_string(),
    };
    let mut steps = Vec::new();

    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Setup,
        GoToCraftingBenchStepCondition::Always,
        "log go-to-crafting-bench start",
        GoToCraftingBenchStepAction::Log {
            message: format!("start GoToCraftingBench common job plan for {country}"),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Pathing,
        GoToCraftingBenchStepCondition::Always,
        "path to crafting bench and press interaction",
        GoToCraftingBenchStepAction::Pathing {
            rule: pathing_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Pathing,
        GoToCraftingBenchStepCondition::Always,
        "wait after pathing",
        GoToCraftingBenchStepAction::Page {
            command: task_vision_result(page.wait(AFTER_PATHING_DELAY_MS))?,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::Always,
        "detect crafting talk UI",
        GoToCraftingBenchStepAction::Locator {
            locator: locators.talk_ui.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::WhenTalkUiMissing,
        "press crafting interaction",
        GoToCraftingBenchStepAction::InteractionRetry {
            rule: interaction_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::WhenTalkUiStillMissing,
        "step backward before retrying crafting interaction",
        GoToCraftingBenchStepAction::GenshinAction {
            action: GenshinAction::MoveBackward,
            press: GoToCraftingBenchActionPress::KeyDown,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::WhenTalkUiStillMissing,
        "hold backward briefly",
        GoToCraftingBenchStepAction::Page {
            command: task_vision_result(page.wait(MOVE_BACKWARD_HOLD_MS))?,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::WhenTalkUiStillMissing,
        "release backward before retrying crafting interaction",
        GoToCraftingBenchStepAction::GenshinAction {
            action: GenshinAction::MoveBackward,
            press: GoToCraftingBenchActionPress::KeyUp,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Interaction,
        GoToCraftingBenchStepCondition::WhenTalkUiStillMissing,
        "retry crafting interaction after stepping back",
        GoToCraftingBenchStepAction::InteractionRetry {
            rule: interaction_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::CraftingPage,
        GoToCraftingBenchStepCondition::Always,
        "select last dialogue option until crafting page confirm appears",
        GoToCraftingBenchStepAction::SelectLastTalkOptionUntilEnd {
            until_locator: locators.white_confirm.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::CraftingPage,
        GoToCraftingBenchStepCondition::WhenCraftingPageOpen,
        "wait after crafting page opens",
        GoToCraftingBenchStepAction::Page {
            command: task_vision_result(page.wait(AFTER_CRAFTING_PAGE_DELAY_MS))?,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinDetection,
        GoToCraftingBenchStepCondition::WhenCraftingPageOpen,
        "detect condensed resin recipe",
        GoToCraftingBenchStepAction::DetectResin {
            locator: locators.condensed_resin.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinDetection,
        GoToCraftingBenchStepCondition::WhenCondensedResinVisible,
        "load one-dragon resin config",
        GoToCraftingBenchStepAction::Log {
            message: "load User/OneDragon selected config to resolve MinResinToKeep".to_string(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinDetection,
        GoToCraftingBenchStepCondition::WhenMinResinToKeepEnabled,
        "recognize current resin counts",
        GoToCraftingBenchStepAction::RecognizeResinCounts {
            rule: resin_recognition_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Cleanup,
        GoToCraftingBenchStepCondition::WhenResinCountRecognitionFailed,
        "press Escape after resin-count recognition failure",
        GoToCraftingBenchStepAction::Input {
            events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Cleanup,
        GoToCraftingBenchStepCondition::WhenResinCountRecognitionFailed,
        "return to main UI after resin-count recognition failure",
        GoToCraftingBenchStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: Some(json!({ "captureSize": capture_size })),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinCraft,
        GoToCraftingBenchStepCondition::WhenMinResinToKeepEnabled,
        "compute condensed resin craft count",
        GoToCraftingBenchStepAction::ComputeCraftsNeeded {
            rule: resin_craft_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinCraft,
        GoToCraftingBenchStepCondition::WhenCraftsNeeded,
        "craft requested condensed resin count",
        GoToCraftingBenchStepAction::CraftCondensedResin {
            rule: resin_craft_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinCraft,
        GoToCraftingBenchStepCondition::WhenMinResinToKeepDisabled,
        "craft condensed resin with default confirm flow",
        GoToCraftingBenchStepAction::CraftCondensedResin {
            rule: resin_craft_rule.clone(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinCraft,
        GoToCraftingBenchStepCondition::WhenCrafted,
        "wait after crafting condensed resin",
        GoToCraftingBenchStepAction::Page {
            command: task_vision_result(page.wait(AFTER_CRAFT_DELAY_MS))?,
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::ResinCraft,
        GoToCraftingBenchStepCondition::WhenCrafted,
        "press Escape after crafting condensed resin",
        GoToCraftingBenchStepAction::Input {
            events: InputSequence::new().key_press(VK_ESCAPE).events().to_vec(),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Cleanup,
        GoToCraftingBenchStepCondition::Always,
        "return to main UI",
        GoToCraftingBenchStepAction::CommonJob {
            task_key: RETURN_MAIN_UI_TASK_KEY.to_string(),
            config: Some(json!({ "captureSize": capture_size })),
        },
    ));
    steps.push(GoToCraftingBenchStep::new(
        GoToCraftingBenchStepPhase::Cleanup,
        GoToCraftingBenchStepCondition::Always,
        "return completed result",
        GoToCraftingBenchStepAction::ReturnResult {
            result: GoToCraftingBenchStepResult::Completed,
        },
    ));

    Ok(GoToCraftingBenchExecutionPlan {
        task_key: GO_TO_CRAFTING_BENCH_TASK_KEY.to_string(),
        display_name: "Go To Crafting Bench".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size,
        retry_times: GO_TO_CRAFTING_BENCH_DEFAULT_RETRY_TIMES,
        retry_failure_delay_ms: AFTER_RETRY_FAILURE_DELAY_MS,
        country,
        localized_texts,
        min_resin_to_keep,
        locators,
        pathing_rule,
        interaction_rule,
        crafting_page_rule,
        resin_recognition_rule,
        resin_craft_rule,
        steps,
        notes: "Legacy crafting-bench pathing, dialogue interaction, condensed-resin recognition, and crafting flow are represented and executable through injectable pathing/OCR/craft hooks; bundled PathExecutor JSON can be converted through the shared AutoPathing action boundary with legacy TrackMap conversion, and the desktop live route now consumes the shared movement contract before handing teleport, interaction, OCR, and click work to live adapters. Capture/OCR/send-input adapters for the crafting-bench F interaction retry, legacy-style talk-option draining with crafting-page and main-UI stop checks, and condensed-resin reduce/increase/confirm clicks are wired; full desktop movement dispatch, one-dragon config loading, and MinResinToKeep resin-count OCR remain pending.".to_string(),
    })
}

fn go_to_crafting_bench_locators(page: &BvPage) -> Result<GoToCraftingBenchLocators> {
    Ok(GoToCraftingBenchLocators {
        talk_ui: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_TALK_UI,
            Some(talk_ui_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        white_confirm: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_WHITE_CONFIRM,
            None,
            0.8,
            true,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        black_confirm: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_BLACK_CONFIRM,
            None,
            0.8,
            true,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        condensed_resin: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_CONDENSED_RESIN,
            Some(craft_condensed_resin_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        fragile_resin_count: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_FRAGILE_RESIN_COUNT,
            Some(fragile_resin_count_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        condensed_resin_count: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_CONDENSED_RESIN_COUNT,
            Some(condensed_resin_count_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::IsExist,
            Some(1_000),
        )?,
        key_reduce: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_KEY_REDUCE,
            Some(craft_adjust_button_roi(page.capture_size)?),
            0.8,
            false,
            BvLocatorOperation::Click,
            Some(1_000),
        )?,
        key_increase: image_locator(
            page,
            GO_TO_CRAFTING_BENCH_KEY_INCREASE,
            Some(craft_adjust_button_roi(page.capture_size)?),
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

fn talk_ui_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        (size.width / 3) as i32,
        (size.height / 8) as i32,
    ))
}

fn craft_condensed_resin_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        0,
        (size.width / 2) as i32,
        (size.height / 3 * 2) as i32,
    ))
}

fn fragile_resin_count_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        (size.height * 3 / 4) as i32,
        (size.width / 3) as i32,
        (size.height / 6) as i32,
    ))
}

fn condensed_resin_count_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        0,
        (size.width / 4) as i32,
        (size.height / 15) as i32,
    ))
}

fn craft_adjust_button_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        (size.width / 2) as i32,
        (size.height / 2) as i32,
        (size.width / 2) as i32,
        (size.height / 2) as i32,
    ))
}
