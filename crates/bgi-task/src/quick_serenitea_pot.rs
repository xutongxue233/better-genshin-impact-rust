use crate::{Result, TaskError, TaskPortState};
use bgi_core::GenshinAction;
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, BvPageCommand, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const QUICK_SERENITEA_POT_TASK_KEY: &str = "QuickSereniteaPot";
pub const QUICK_SERENITEA_POT_DISPLAY_NAME: &str = "Quick Serenitea Pot";
pub const QUICK_SERENITEA_POT_BAG_CLOSE_BUTTON: &str = "QuickTeleport:MapCloseButton.png";
pub const QUICK_SERENITEA_POT_ICON: &str = "QuickSereniteaPot:SereniteaPotIcon.png";
pub const QUICK_SERENITEA_POT_WHITE_CONFIRM: &str = "Common/Element:btn_white_confirm.png";

const TEMPLATE_THRESHOLD: f64 = 0.8;
const BAG_CLOSE_ROI_RIGHT_OFFSET_1080P: f64 = 107.0;
const BAG_CLOSE_ROI_Y_1080P: f64 = 19.0;
const BAG_CLOSE_ROI_SIZE_1080P: f64 = 58.0;
const POT_ICON_ROI_X_1080P: f64 = 100.0;
const POT_ICON_ROI_Y_1080P: f64 = 100.0;
const POT_ICON_ROI_W_1080P: f64 = 1190.0;
const POT_ICON_ROI_H_1080P: f64 = 860.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickSereniteaPotExecutionConfig {
    pub capture_size: Size,
}

impl Default for QuickSereniteaPotExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl QuickSereniteaPotExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub preflight_rule: QuickSereniteaPotPreflightRule,
    pub locators: QuickSereniteaPotLocators,
    pub bag_rule: QuickSereniteaPotBagRule,
    pub placement_rule: QuickSereniteaPotPlacementRule,
    pub interaction_rule: QuickSereniteaPotInteractionRule,
    pub steps: Vec<QuickSereniteaPotStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickSereniteaPotPreflightRule {
    pub requires_initialized_task_context: bool,
    pub shows_toast_when_uninitialized: bool,
    pub toast_message: String,
    pub requires_active_genshin_process: bool,
    pub inactive_process_returns_without_warning: bool,
    pub destroys_asset_singleton_before_run: bool,
    pub catches_and_logs_exceptions: bool,
    pub clears_vision_drawings_finally: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotLocators {
    pub bag_close_button: BvLocatorPlan,
    pub serenitea_pot_icon: BvLocatorPlan,
    pub white_confirm_button: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotBagRule {
    pub open_inventory_action: GenshinAction,
    pub wait_after_open_inventory_ms: u32,
    pub bag_open_retry_interval_ms: u32,
    pub bag_open_attempts: u8,
    pub gadget_tab_click: QuickSereniteaPotScreenPoint,
    pub wait_after_gadget_tab_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotPlacementRule {
    pub find_pot_retry_interval_ms: u32,
    pub find_pot_attempts: u8,
    pub white_confirm_pre_click_delay_ms: u32,
    pub ignore_missing_white_confirm: bool,
    pub wait_after_icon_click_ms: u32,
    pub wait_after_confirm_ms: u32,
    pub main_ui_success_checks: u8,
    pub big_map_reopen_attempts: u8,
    pub reopen_inventory_action: GenshinAction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotInteractionRule {
    pub enter_text: String,
    pub leave_text: String,
    pub object_text: String,
    pub interact_action: GenshinAction,
    pub wait_after_interact_ms: u32,
    pub confirm_click: QuickSereniteaPotScreenPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct QuickSereniteaPotScreenPoint {
    pub x_1080p: f64,
    pub y_1080p: f64,
    pub screen_x: f64,
    pub screen_y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotStep {
    pub phase: QuickSereniteaPotStepPhase,
    pub condition: QuickSereniteaPotStepCondition,
    pub label: String,
    pub action: QuickSereniteaPotStepAction,
}

impl QuickSereniteaPotStep {
    fn new(
        phase: QuickSereniteaPotStepPhase,
        condition: QuickSereniteaPotStepCondition,
        label: impl Into<String>,
        action: QuickSereniteaPotStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotStepPhase {
    Preflight,
    OpenBag,
    PlaceGadget,
    VerifyPlacement,
    Interact,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotStepCondition {
    Always,
    WhenBagOpened,
    WhenPotIconFound,
    WhenPlacementDidNotReachMainUi,
    WhenBigMapDetected,
    WhenEnterOrLeaveFound,
    WhenEnterOrLeaveMissing,
    Finally,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickSereniteaPotStepAction {
    Preflight {
        rule: QuickSereniteaPotPreflightRule,
    },
    GenshinAction {
        action: GenshinAction,
    },
    Locator {
        locator: BvLocatorPlan,
    },
    Click1080p {
        point: QuickSereniteaPotScreenPoint,
    },
    Page {
        command: BvPageCommand,
    },
    VerifyPlacement {
        rule: QuickSereniteaPotPlacementRule,
    },
    ClickWhiteConfirmButton {
        locator: BvLocatorPlan,
        pre_click_delay_ms: u32,
        missing_is_ok: bool,
    },
    FindFInteraction {
        rule: QuickSereniteaPotInteractionRule,
    },
    ClearVisionDrawings,
    Return,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct QuickSereniteaPotPlacementOutcome {
    pub main_ui_reached: bool,
    pub big_map_detected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotInteractionOutcome {
    Enter,
    Leave,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotExecutionResult {
    Completed,
    PreflightSkipped,
    BagMissing,
    PotIconMissing,
    WhiteConfirmMissing,
    ReturnedFromBigMap,
    InteractionMissing,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct QuickSereniteaPotExecutorState {
    pub preflight_passed: bool,
    pub bag_opened: Option<bool>,
    pub pot_icon_found: Option<bool>,
    pub white_confirm_clicked: Option<bool>,
    pub placement_outcome: Option<QuickSereniteaPotPlacementOutcome>,
    pub interaction_outcome: Option<QuickSereniteaPotInteractionOutcome>,
    pub interaction_action_dispatched: bool,
    pub confirmation_clicked: bool,
    pub vision_drawings_cleared: bool,
    pub result: Option<QuickSereniteaPotExecutionResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotRuntimeActionKind {
    Preflight,
    GenshinAction,
    Locator,
    Click1080p,
    Page,
    VerifyPlacement,
    ClickWhiteConfirmButton,
    FindFInteraction,
    ClearVisionDrawings,
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickSereniteaPotSkipReason {
    PreflightSkipped,
    BagMissing,
    PotIconMissing,
    WhiteConfirmMissing,
    PlacementNotVerified,
    BigMapMissing,
    EnterOrLeaveFound,
    EnterOrLeaveMissing,
    ResultAlreadySet,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotRuntimeStepReport {
    pub phase: QuickSereniteaPotStepPhase,
    pub condition: QuickSereniteaPotStepCondition,
    pub label: String,
    pub action_kind: QuickSereniteaPotRuntimeActionKind,
}

impl QuickSereniteaPotRuntimeStepReport {
    fn executed(step: &QuickSereniteaPotStep) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            action_kind: quick_serenitea_pot_action_kind(&step.action),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotSkippedStep {
    pub phase: QuickSereniteaPotStepPhase,
    pub condition: QuickSereniteaPotStepCondition,
    pub label: String,
    pub reason: QuickSereniteaPotSkipReason,
}

impl QuickSereniteaPotSkippedStep {
    fn new(step: &QuickSereniteaPotStep, reason: QuickSereniteaPotSkipReason) -> Self {
        Self {
            phase: step.phase,
            condition: step.condition,
            label: step.label.clone(),
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickSereniteaPotExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub state: QuickSereniteaPotExecutorState,
    pub executed_steps: Vec<QuickSereniteaPotRuntimeStepReport>,
    pub skipped_steps: Vec<QuickSereniteaPotSkippedStep>,
}

pub trait QuickSereniteaPotRuntime {
    fn quick_serenitea_pot_preflight(
        &mut self,
        rule: &QuickSereniteaPotPreflightRule,
    ) -> Result<bool>;

    fn dispatch_quick_serenitea_pot_action(&mut self, action: GenshinAction) -> Result<()>;

    fn locate_quick_serenitea_pot_template(&mut self, locator: &BvLocatorPlan) -> Result<bool>;

    fn click_quick_serenitea_pot_point(
        &mut self,
        point: &QuickSereniteaPotScreenPoint,
    ) -> Result<()>;

    fn execute_quick_serenitea_pot_page_command(&mut self, command: &BvPageCommand) -> Result<()>;

    fn verify_quick_serenitea_pot_placement(
        &mut self,
        rule: &QuickSereniteaPotPlacementRule,
    ) -> Result<QuickSereniteaPotPlacementOutcome>;

    fn click_quick_serenitea_pot_white_confirm(
        &mut self,
        locator: &BvLocatorPlan,
        pre_click_delay_ms: u32,
        missing_is_ok: bool,
    ) -> Result<bool>;

    fn find_quick_serenitea_pot_interaction(
        &mut self,
        rule: &QuickSereniteaPotInteractionRule,
    ) -> Result<QuickSereniteaPotInteractionOutcome>;

    fn clear_quick_serenitea_pot_vision_drawings(&mut self) -> Result<()>;
}

pub fn execute_quick_serenitea_pot_plan<R>(
    plan: &QuickSereniteaPotExecutionPlan,
    runtime: &mut R,
) -> Result<QuickSereniteaPotExecutionReport>
where
    R: QuickSereniteaPotRuntime,
{
    let mut state = QuickSereniteaPotExecutorState::default();
    let mut executed_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for step in &plan.steps {
        match should_execute_quick_serenitea_pot_step(step, &state) {
            Ok(()) => {
                execute_quick_serenitea_pot_step(step, runtime, &mut state)?;
                executed_steps.push(QuickSereniteaPotRuntimeStepReport::executed(step));
            }
            Err(reason) => skipped_steps.push(QuickSereniteaPotSkippedStep::new(step, reason)),
        }
    }

    if state.result.is_none() && state.preflight_passed {
        state.result = Some(QuickSereniteaPotExecutionResult::Completed);
    }

    let completed = matches!(
        state.result,
        Some(
            QuickSereniteaPotExecutionResult::Completed
                | QuickSereniteaPotExecutionResult::ReturnedFromBigMap
                | QuickSereniteaPotExecutionResult::InteractionMissing
        )
    );

    Ok(QuickSereniteaPotExecutionReport {
        task_key: plan.task_key.clone(),
        completed,
        state,
        executed_steps,
        skipped_steps,
    })
}

pub fn plan_quick_serenitea_pot(
    config: QuickSereniteaPotExecutionConfig,
) -> Result<QuickSereniteaPotExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let locators = plan_locators(&page, config.capture_size)?;
    let preflight_rule = QuickSereniteaPotPreflightRule {
        requires_initialized_task_context: true,
        shows_toast_when_uninitialized: true,
        toast_message: "请先启动".to_string(),
        requires_active_genshin_process: true,
        inactive_process_returns_without_warning: true,
        destroys_asset_singleton_before_run: true,
        catches_and_logs_exceptions: true,
        clears_vision_drawings_finally: true,
    };
    let bag_rule = QuickSereniteaPotBagRule {
        open_inventory_action: GenshinAction::OpenInventory,
        wait_after_open_inventory_ms: 500,
        bag_open_retry_interval_ms: 500,
        bag_open_attempts: 5,
        gadget_tab_click: screen_point(config.capture_size, 1050.0, 50.0),
        wait_after_gadget_tab_ms: 200,
    };
    let placement_rule = QuickSereniteaPotPlacementRule {
        find_pot_retry_interval_ms: 200,
        find_pot_attempts: 3,
        white_confirm_pre_click_delay_ms: 500,
        ignore_missing_white_confirm: true,
        wait_after_icon_click_ms: 200,
        wait_after_confirm_ms: 800,
        main_ui_success_checks: 5,
        big_map_reopen_attempts: 5,
        reopen_inventory_action: GenshinAction::OpenInventory,
    };
    let interaction_rule = QuickSereniteaPotInteractionRule {
        enter_text: "进入".to_string(),
        leave_text: "离开".to_string(),
        object_text: "尘歌壶".to_string(),
        interact_action: GenshinAction::PickUpOrInteract,
        wait_after_interact_ms: 200,
        confirm_click: screen_point(config.capture_size, 1010.0, 760.0),
    };
    let steps = vec![
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Preflight,
            QuickSereniteaPotStepCondition::Always,
            "check task context and active Genshin process",
            QuickSereniteaPotStepAction::Preflight {
                rule: preflight_rule.clone(),
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::OpenBag,
            QuickSereniteaPotStepCondition::Always,
            "open inventory",
            QuickSereniteaPotStepAction::GenshinAction {
                action: bag_rule.open_inventory_action,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::OpenBag,
            QuickSereniteaPotStepCondition::Always,
            "wait after opening inventory",
            QuickSereniteaPotStepAction::Page {
                command: page_wait(&page, bag_rule.wait_after_open_inventory_ms)?,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::OpenBag,
            QuickSereniteaPotStepCondition::Always,
            "wait for bag close button",
            QuickSereniteaPotStepAction::Locator {
                locator: locators.bag_close_button.clone(),
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::OpenBag,
            QuickSereniteaPotStepCondition::WhenBagOpened,
            "click gadget tab",
            QuickSereniteaPotStepAction::Click1080p {
                point: bag_rule.gadget_tab_click,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::OpenBag,
            QuickSereniteaPotStepCondition::WhenBagOpened,
            "wait after gadget tab click",
            QuickSereniteaPotStepAction::Page {
                command: page_wait(&page, bag_rule.wait_after_gadget_tab_ms)?,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::PlaceGadget,
            QuickSereniteaPotStepCondition::WhenBagOpened,
            "find and click Serenitea Pot icon",
            QuickSereniteaPotStepAction::Locator {
                locator: locators.serenitea_pot_icon.clone(),
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::PlaceGadget,
            QuickSereniteaPotStepCondition::WhenPotIconFound,
            "wait after pot icon click",
            QuickSereniteaPotStepAction::Page {
                command: page_wait(&page, placement_rule.wait_after_icon_click_ms)?,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::PlaceGadget,
            QuickSereniteaPotStepCondition::WhenPotIconFound,
            "click white confirm button to place gadget",
            QuickSereniteaPotStepAction::ClickWhiteConfirmButton {
                locator: locators.white_confirm_button.clone(),
                pre_click_delay_ms: placement_rule.white_confirm_pre_click_delay_ms,
                missing_is_ok: placement_rule.ignore_missing_white_confirm,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::PlaceGadget,
            QuickSereniteaPotStepCondition::WhenPotIconFound,
            "wait after placing gadget",
            QuickSereniteaPotStepAction::Page {
                command: page_wait(&page, placement_rule.wait_after_confirm_ms)?,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::VerifyPlacement,
            QuickSereniteaPotStepCondition::WhenPotIconFound,
            "verify main UI or reopen inventory until big map",
            QuickSereniteaPotStepAction::VerifyPlacement {
                rule: placement_rule.clone(),
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::VerifyPlacement,
            QuickSereniteaPotStepCondition::WhenBigMapDetected,
            "return when placement leads to big map",
            QuickSereniteaPotStepAction::Return,
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Interact,
            QuickSereniteaPotStepCondition::Always,
            "detect enter or leave Serenitea Pot F interaction",
            QuickSereniteaPotStepAction::FindFInteraction {
                rule: interaction_rule.clone(),
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Interact,
            QuickSereniteaPotStepCondition::WhenEnterOrLeaveFound,
            "press F to trigger Serenitea Pot interaction",
            QuickSereniteaPotStepAction::GenshinAction {
                action: interaction_rule.interact_action,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Interact,
            QuickSereniteaPotStepCondition::WhenEnterOrLeaveFound,
            "wait after F interaction",
            QuickSereniteaPotStepAction::Page {
                command: page_wait(&page, interaction_rule.wait_after_interact_ms)?,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Interact,
            QuickSereniteaPotStepCondition::WhenEnterOrLeaveFound,
            "click enter or leave confirmation",
            QuickSereniteaPotStepAction::Click1080p {
                point: interaction_rule.confirm_click,
            },
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Interact,
            QuickSereniteaPotStepCondition::WhenEnterOrLeaveMissing,
            "log missing Serenitea Pot F interaction",
            QuickSereniteaPotStepAction::Return,
        ),
        QuickSereniteaPotStep::new(
            QuickSereniteaPotStepPhase::Cleanup,
            QuickSereniteaPotStepCondition::Finally,
            "clear vision drawings",
            QuickSereniteaPotStepAction::ClearVisionDrawings,
        ),
    ];

    Ok(QuickSereniteaPotExecutionPlan {
        task_key: QUICK_SERENITEA_POT_TASK_KEY.to_string(),
        display_name: QUICK_SERENITEA_POT_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        preflight_rule,
        locators,
        bag_rule,
        placement_rule,
        interaction_rule,
        steps,
        pending_native: vec![
            "live TaskContext/SystemControl preflight, Toast, and asset singleton reset adapter"
                .to_string(),
            "live capture, template matching, Bv main-UI/big-map/F-interaction recognition adapter"
                .to_string(),
            "live Genshin action dispatch and GameCaptureRegion coordinate click adapter"
                .to_string(),
            "live Vision overlay cleanup adapter".to_string(),
        ],
    })
}

fn should_execute_quick_serenitea_pot_step(
    step: &QuickSereniteaPotStep,
    state: &QuickSereniteaPotExecutorState,
) -> std::result::Result<(), QuickSereniteaPotSkipReason> {
    if step.condition == QuickSereniteaPotStepCondition::Finally {
        return if state.preflight_passed {
            Ok(())
        } else {
            Err(QuickSereniteaPotSkipReason::PreflightSkipped)
        };
    }
    if state.result.is_some() {
        return Err(QuickSereniteaPotSkipReason::ResultAlreadySet);
    }

    match step.condition {
        QuickSereniteaPotStepCondition::Always => {
            if step.phase == QuickSereniteaPotStepPhase::Preflight || state.preflight_passed {
                Ok(())
            } else {
                Err(QuickSereniteaPotSkipReason::PreflightSkipped)
            }
        }
        QuickSereniteaPotStepCondition::WhenBagOpened => match state.bag_opened {
            Some(true) => Ok(()),
            _ => Err(QuickSereniteaPotSkipReason::BagMissing),
        },
        QuickSereniteaPotStepCondition::WhenPotIconFound => match state.pot_icon_found {
            Some(true) => Ok(()),
            Some(false) => Err(QuickSereniteaPotSkipReason::PotIconMissing),
            None => Err(QuickSereniteaPotSkipReason::BagMissing),
        },
        QuickSereniteaPotStepCondition::WhenPlacementDidNotReachMainUi => {
            match state.placement_outcome {
                Some(outcome) if !outcome.main_ui_reached => Ok(()),
                Some(_) => Err(QuickSereniteaPotSkipReason::PlacementNotVerified),
                None => Err(QuickSereniteaPotSkipReason::PlacementNotVerified),
            }
        }
        QuickSereniteaPotStepCondition::WhenBigMapDetected => match state.placement_outcome {
            Some(outcome) if outcome.big_map_detected => Ok(()),
            Some(_) => Err(QuickSereniteaPotSkipReason::BigMapMissing),
            None => Err(QuickSereniteaPotSkipReason::PlacementNotVerified),
        },
        QuickSereniteaPotStepCondition::WhenEnterOrLeaveFound => match state.interaction_outcome {
            Some(QuickSereniteaPotInteractionOutcome::Enter)
            | Some(QuickSereniteaPotInteractionOutcome::Leave) => Ok(()),
            Some(QuickSereniteaPotInteractionOutcome::Missing) => {
                Err(QuickSereniteaPotSkipReason::EnterOrLeaveMissing)
            }
            None => Err(QuickSereniteaPotSkipReason::EnterOrLeaveMissing),
        },
        QuickSereniteaPotStepCondition::WhenEnterOrLeaveMissing => {
            match state.interaction_outcome {
                Some(QuickSereniteaPotInteractionOutcome::Missing) => Ok(()),
                Some(_) => Err(QuickSereniteaPotSkipReason::EnterOrLeaveFound),
                None => Err(QuickSereniteaPotSkipReason::EnterOrLeaveMissing),
            }
        }
        QuickSereniteaPotStepCondition::Finally => Ok(()),
    }
}

fn execute_quick_serenitea_pot_step<R>(
    step: &QuickSereniteaPotStep,
    runtime: &mut R,
    state: &mut QuickSereniteaPotExecutorState,
) -> Result<()>
where
    R: QuickSereniteaPotRuntime,
{
    match &step.action {
        QuickSereniteaPotStepAction::Preflight { rule } => {
            let passed = runtime.quick_serenitea_pot_preflight(rule)?;
            state.preflight_passed = passed;
            if !passed {
                state.result = Some(QuickSereniteaPotExecutionResult::PreflightSkipped);
            }
        }
        QuickSereniteaPotStepAction::GenshinAction { action } => {
            runtime.dispatch_quick_serenitea_pot_action(*action)?;
            if *action == GenshinAction::PickUpOrInteract {
                state.interaction_action_dispatched = true;
            }
        }
        QuickSereniteaPotStepAction::Locator { locator } => {
            let matched = runtime.locate_quick_serenitea_pot_template(locator)?;
            match step.phase {
                QuickSereniteaPotStepPhase::OpenBag => {
                    state.bag_opened = Some(matched);
                    if !matched {
                        state.result = Some(QuickSereniteaPotExecutionResult::BagMissing);
                    }
                }
                QuickSereniteaPotStepPhase::PlaceGadget
                    if step.label.contains("Serenitea Pot icon") =>
                {
                    state.pot_icon_found = Some(matched);
                    if !matched {
                        state.result = Some(QuickSereniteaPotExecutionResult::PotIconMissing);
                    }
                }
                _ => {}
            }
        }
        QuickSereniteaPotStepAction::ClickWhiteConfirmButton {
            locator,
            pre_click_delay_ms,
            missing_is_ok,
        } => {
            let clicked = runtime.click_quick_serenitea_pot_white_confirm(
                locator,
                *pre_click_delay_ms,
                *missing_is_ok,
            )?;
            state.white_confirm_clicked = Some(clicked);
            if !clicked && !missing_is_ok {
                state.result = Some(QuickSereniteaPotExecutionResult::WhiteConfirmMissing);
            }
        }
        QuickSereniteaPotStepAction::Click1080p { point } => {
            runtime.click_quick_serenitea_pot_point(point)?;
            if step.phase == QuickSereniteaPotStepPhase::Interact {
                state.confirmation_clicked = true;
            }
        }
        QuickSereniteaPotStepAction::Page { command } => {
            runtime.execute_quick_serenitea_pot_page_command(command)?;
        }
        QuickSereniteaPotStepAction::VerifyPlacement { rule } => {
            state.placement_outcome = Some(runtime.verify_quick_serenitea_pot_placement(rule)?);
        }
        QuickSereniteaPotStepAction::FindFInteraction { rule } => {
            state.interaction_outcome = Some(runtime.find_quick_serenitea_pot_interaction(rule)?);
        }
        QuickSereniteaPotStepAction::ClearVisionDrawings => {
            runtime.clear_quick_serenitea_pot_vision_drawings()?;
            state.vision_drawings_cleared = true;
        }
        QuickSereniteaPotStepAction::Return => match step.condition {
            QuickSereniteaPotStepCondition::WhenBigMapDetected => {
                state.result = Some(QuickSereniteaPotExecutionResult::ReturnedFromBigMap);
            }
            QuickSereniteaPotStepCondition::WhenEnterOrLeaveMissing => {
                state.result = Some(QuickSereniteaPotExecutionResult::InteractionMissing);
            }
            _ => {}
        },
    }
    Ok(())
}

fn quick_serenitea_pot_action_kind(
    action: &QuickSereniteaPotStepAction,
) -> QuickSereniteaPotRuntimeActionKind {
    match action {
        QuickSereniteaPotStepAction::Preflight { .. } => {
            QuickSereniteaPotRuntimeActionKind::Preflight
        }
        QuickSereniteaPotStepAction::GenshinAction { .. } => {
            QuickSereniteaPotRuntimeActionKind::GenshinAction
        }
        QuickSereniteaPotStepAction::Locator { .. } => QuickSereniteaPotRuntimeActionKind::Locator,
        QuickSereniteaPotStepAction::Click1080p { .. } => {
            QuickSereniteaPotRuntimeActionKind::Click1080p
        }
        QuickSereniteaPotStepAction::Page { .. } => QuickSereniteaPotRuntimeActionKind::Page,
        QuickSereniteaPotStepAction::VerifyPlacement { .. } => {
            QuickSereniteaPotRuntimeActionKind::VerifyPlacement
        }
        QuickSereniteaPotStepAction::ClickWhiteConfirmButton { .. } => {
            QuickSereniteaPotRuntimeActionKind::ClickWhiteConfirmButton
        }
        QuickSereniteaPotStepAction::FindFInteraction { .. } => {
            QuickSereniteaPotRuntimeActionKind::FindFInteraction
        }
        QuickSereniteaPotStepAction::ClearVisionDrawings => {
            QuickSereniteaPotRuntimeActionKind::ClearVisionDrawings
        }
        QuickSereniteaPotStepAction::Return => QuickSereniteaPotRuntimeActionKind::Return,
    }
}

fn plan_locators(page: &BvPage, capture_size: Size) -> Result<QuickSereniteaPotLocators> {
    let bag_close_roi = right_offset_rect(
        capture_size,
        BAG_CLOSE_ROI_RIGHT_OFFSET_1080P,
        BAG_CLOSE_ROI_Y_1080P,
        BAG_CLOSE_ROI_SIZE_1080P,
        BAG_CLOSE_ROI_SIZE_1080P,
    )?;
    let serenitea_pot_roi = scaled_rect(
        capture_size,
        POT_ICON_ROI_X_1080P,
        POT_ICON_ROI_Y_1080P,
        POT_ICON_ROI_W_1080P,
        POT_ICON_ROI_H_1080P,
    )?;
    let mut white_confirm_button = image_locator_with_retry(
        page,
        QUICK_SERENITEA_POT_WHITE_CONFIRM,
        None,
        TEMPLATE_THRESHOLD,
        BvLocatorOperation::Click,
        1,
        250,
    )?;
    white_confirm_button
        .recognition_object
        .template
        .use_3_channels = true;
    white_confirm_button
        .recognition_object
        .template
        .draw_on_window = false;
    Ok(QuickSereniteaPotLocators {
        bag_close_button: image_locator_with_retry(
            page,
            QUICK_SERENITEA_POT_BAG_CLOSE_BUTTON,
            Some(bag_close_roi),
            TEMPLATE_THRESHOLD,
            BvLocatorOperation::WaitFor,
            2_500,
            500,
        )?,
        serenitea_pot_icon: image_locator_with_retry(
            page,
            QUICK_SERENITEA_POT_ICON,
            Some(serenitea_pot_roi),
            TEMPLATE_THRESHOLD,
            BvLocatorOperation::Click,
            600,
            200,
        )?,
        white_confirm_button,
    })
}

fn image_locator_with_retry(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    operation: BvLocatorOperation,
    timeout_ms: u32,
    retry_interval_ms: u32,
) -> Result<BvLocatorPlan> {
    let image = BvImage::new(asset).map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let locator = page
        .locator_for_image(&image, roi, threshold)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?
        .with_retry_interval(retry_interval_ms)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let mut plan = locator.plan(operation, Some(timeout_ms));
    plan.recognition_object.template.draw_on_window = true;
    Ok(plan)
}

fn page_wait(page: &BvPage, milliseconds: u32) -> Result<BvPageCommand> {
    page.wait(milliseconds)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn right_offset_rect(
    size: Size,
    right_offset: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<Rect> {
    let scale = scale(size);
    Rect::new(
        size.width as i32 - (right_offset * scale).round() as i32,
        (y * scale).round() as i32,
        (width * scale).round() as i32,
        (height * scale).round() as i32,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn scaled_rect(size: Size, x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    let scale = scale(size);
    Rect::new(
        (x * scale).round() as i32,
        (y * scale).round() as i32,
        (width * scale).round() as i32,
        (height * scale).round() as i32,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn screen_point(size: Size, x_1080p: f64, y_1080p: f64) -> QuickSereniteaPotScreenPoint {
    let scale = scale(size);
    QuickSereniteaPotScreenPoint {
        x_1080p,
        y_1080p,
        screen_x: x_1080p * scale,
        screen_y: y_1080p * scale,
    }
}

fn scale(size: Size) -> f64 {
    size.width as f64 / 1920.0
}
