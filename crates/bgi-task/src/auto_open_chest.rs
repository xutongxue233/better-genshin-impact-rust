use crate::{Result, TaskError, TaskPortState};
use bgi_core::GenshinAction;
use bgi_vision::{BvImage, BvLocatorOperation, BvLocatorPlan, BvPage, Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUTO_OPEN_CHEST_TASK_KEY: &str = "AutoOpenChest";
pub const AUTO_OPEN_CHEST_DISPLAY_NAME: &str = "Auto Open Chest";
pub const AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_OPEN_CHEST_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_OPEN_CHEST_CHEST_ASSET: &str = "AutoOpenChest:chest.png";
pub const AUTO_OPEN_CHEST_CHEST_F_ASSET: &str = "AutoOpenChest:chest_F_icon.png";
pub const AUTO_OPEN_CHEST_FLOWER_F_ASSET: &str = "AutoOpenChest:flower_F_icon.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoOpenChestExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub locators: AutoOpenChestLocators,
    pub search_rule: AutoOpenChestSearchRule,
    pub steps: Vec<AutoOpenChestStep>,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOpenChestExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
}

impl Default for AutoOpenChestExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH,
                AUTO_OPEN_CHEST_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
        }
    }
}

impl AutoOpenChestExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };
        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        if let Some(asset_scale) = f64_member(value, ["assetScale", "AssetScale", "asset_scale"]) {
            config.asset_scale = asset_scale.max(0.0);
        } else {
            config.asset_scale =
                config.capture_size.width as f64 / AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH as f64;
        }
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoOpenChestLocators {
    pub chest_icon: BvLocatorPlan,
    pub chest_f_icon: BvLocatorPlan,
    pub flower_f_icon: BvLocatorPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoOpenChestSearchRule {
    pub requires_initial_chest_f_prompt: bool,
    pub time_limit_seconds: u64,
    pub loop_delay_ms: u64,
    pub backward_y_threshold_1080p: i32,
    pub scaled_backward_y_threshold: i32,
    pub backward_delay_ms: u64,
    pub center_gap_divisor: i32,
    pub move_forward_when_half_width_gap_below_icon_width: bool,
    pub always_releases_forward_on_loop_exit: bool,
    pub flower_handler_action: GenshinAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoOpenChestStep {
    pub phase: AutoOpenChestStepPhase,
    pub label: String,
    pub action: AutoOpenChestStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoOpenChestStepPhase {
    InitialPrompt,
    SearchLoop,
    Interaction,
    FlowerHandling,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutoOpenChestStepAction {
    DetectChestFIcon,
    DetectChestIcon,
    DetectInteractionPrompt,
    MoveForwardWhenCentered,
    BackwardAndCameraResetWhenBehind,
    TurnTowardChest,
    Interact,
    OpenPaimonMenuForFlower,
    ReleaseMoveForward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoOpenChestActionPress {
    KeyPress,
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoOpenChestAction {
    GenshinAction {
        action: GenshinAction,
        press: AutoOpenChestActionPress,
    },
    MouseMoveBy {
        delta_x: i32,
        delta_y: i32,
    },
    MouseMiddleClick,
    Delay {
        duration_ms: u64,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoOpenChestObservation {
    pub initial_chest_f_icon_exists: bool,
    pub chest_icon: Option<Rect>,
    pub chest_f_icon_exists: bool,
    pub flower_f_icon_exists: bool,
    pub capture_width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoOpenChestDecisionResult {
    InitialPromptMissing,
    ChestIconMissing,
    SearchingContinue,
    ChestInteracted,
    FlowerInteracted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoOpenChestDecision {
    pub result: AutoOpenChestDecisionResult,
    pub is_terminal: bool,
    pub is_flower: bool,
    pub actions: Vec<AutoOpenChestAction>,
    pub post_loop_actions: Vec<AutoOpenChestAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoOpenChestExecutionStatus {
    InitialPromptMissing,
    ChestIconMissing,
    ChestInteracted,
    FlowerInteracted,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoOpenChestExecutionState {
    pub iterations: usize,
    pub elapsed_ms: u64,
    pub timed_out: bool,
    pub cancelled: bool,
    pub final_result: Option<AutoOpenChestDecisionResult>,
    pub flower_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoOpenChestExecutionReport {
    pub task_key: String,
    pub completed: bool,
    pub status: AutoOpenChestExecutionStatus,
    pub state: AutoOpenChestExecutionState,
    pub decisions: Vec<AutoOpenChestDecision>,
    pub dispatched_actions: Vec<AutoOpenChestAction>,
    pub cleanup_actions: Vec<AutoOpenChestAction>,
    pub post_loop_actions: Vec<AutoOpenChestAction>,
}

pub trait AutoOpenChestRuntime {
    fn elapsed_auto_open_chest_ms(&mut self) -> Result<u64>;

    fn is_auto_open_chest_cancelled(&mut self) -> Result<bool>;

    fn observe_auto_open_chest(
        &mut self,
        plan: &AutoOpenChestExecutionPlan,
    ) -> Result<AutoOpenChestObservation>;

    fn dispatch_auto_open_chest_action(&mut self, action: &AutoOpenChestAction) -> Result<()>;
}

pub fn plan_auto_open_chest(
    config: AutoOpenChestExecutionConfig,
) -> Result<AutoOpenChestExecutionPlan> {
    let page = BvPage {
        capture_size: config.capture_size,
        ..BvPage::default()
    };
    let locators = AutoOpenChestLocators {
        chest_icon: image_locator(
            &page,
            AUTO_OPEN_CHEST_CHEST_ASSET,
            Some(scaled_rect(
                config.capture_size,
                330.0,
                130.0,
                1250.0,
                840.0,
            )?),
            BvLocatorOperation::IsExist,
        )?,
        chest_f_icon: image_locator(
            &page,
            AUTO_OPEN_CHEST_CHEST_F_ASSET,
            Some(scaled_rect(
                config.capture_size,
                1150.0,
                450.0,
                100.0,
                300.0,
            )?),
            BvLocatorOperation::IsExist,
        )?,
        flower_f_icon: image_locator(
            &page,
            AUTO_OPEN_CHEST_FLOWER_F_ASSET,
            Some(scaled_rect(
                config.capture_size,
                1150.0,
                450.0,
                100.0,
                300.0,
            )?),
            BvLocatorOperation::IsExist,
        )?,
    };
    let search_rule = AutoOpenChestSearchRule {
        requires_initial_chest_f_prompt: true,
        time_limit_seconds: 60,
        loop_delay_ms: 500,
        backward_y_threshold_1080p: 600,
        scaled_backward_y_threshold: (600.0 * config.asset_scale).round() as i32,
        backward_delay_ms: 30,
        center_gap_divisor: 2,
        move_forward_when_half_width_gap_below_icon_width: true,
        always_releases_forward_on_loop_exit: true,
        flower_handler_action: GenshinAction::OpenPaimonMenu,
    };

    Ok(AutoOpenChestExecutionPlan {
        task_key: AUTO_OPEN_CHEST_TASK_KEY.to_string(),
        display_name: AUTO_OPEN_CHEST_DISPLAY_NAME.to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        locators,
        search_rule,
        steps: auto_open_chest_steps(),
        pending_native: vec![
            "desktop live capture/template/input execution is wired; TaskContext/SystemControl preflight and real-game regression remain pending, while legacy draw-disabled locator overlay cleanup is a no-op"
                .to_string(),
        ],
    })
}

pub fn execute_auto_open_chest_plan<R>(
    plan: &AutoOpenChestExecutionPlan,
    runtime: &mut R,
) -> Result<AutoOpenChestExecutionReport>
where
    R: AutoOpenChestRuntime,
{
    let timeout_ms = plan.search_rule.time_limit_seconds.saturating_mul(1000);
    let mut state = AutoOpenChestExecutionState {
        iterations: 0,
        elapsed_ms: 0,
        timed_out: false,
        cancelled: false,
        final_result: None,
        flower_detected: false,
    };
    let mut decisions = Vec::new();
    let mut dispatched_actions = Vec::new();
    let mut cleanup_actions = Vec::new();
    let mut post_loop_actions = Vec::new();
    let mut initial_prompt_checked = false;
    let mut initial_prompt_seen = false;

    loop {
        state.elapsed_ms = runtime.elapsed_auto_open_chest_ms()?;
        if timeout_ms > 0 && state.elapsed_ms >= timeout_ms {
            state.timed_out = true;
            let cleanup = dispatch_auto_open_chest_cleanup(plan, runtime)?;
            cleanup_actions.extend(cleanup);
            return Ok(auto_open_chest_report(
                plan,
                false,
                AutoOpenChestExecutionStatus::TimedOut,
                state,
                decisions,
                dispatched_actions,
                cleanup_actions,
                post_loop_actions,
            ));
        }

        if runtime.is_auto_open_chest_cancelled()? {
            state.cancelled = true;
            let cleanup = dispatch_auto_open_chest_cleanup(plan, runtime)?;
            cleanup_actions.extend(cleanup);
            return Ok(auto_open_chest_report(
                plan,
                false,
                AutoOpenChestExecutionStatus::Cancelled,
                state,
                decisions,
                dispatched_actions,
                cleanup_actions,
                post_loop_actions,
            ));
        }

        let mut observation = runtime.observe_auto_open_chest(plan)?;
        if plan.search_rule.requires_initial_chest_f_prompt {
            if initial_prompt_checked {
                observation.initial_chest_f_icon_exists = initial_prompt_seen;
            } else {
                initial_prompt_checked = true;
                initial_prompt_seen = observation.initial_chest_f_icon_exists;
            }
        }

        state.iterations += 1;
        let decision = decide_auto_open_chest_step(plan, observation);

        for action in &decision.actions {
            runtime.dispatch_auto_open_chest_action(action)?;
            dispatched_actions.push(action.clone());
        }

        if decision.is_terminal {
            state.final_result = Some(decision.result);
            state.flower_detected = decision.is_flower;
            let status = auto_open_chest_status_from_decision(decision.result);
            let completed = true;
            let terminal_needs_cleanup =
                decision.result != AutoOpenChestDecisionResult::InitialPromptMissing;
            let pending_post_loop_actions = decision.post_loop_actions.clone();
            decisions.push(decision);

            if terminal_needs_cleanup {
                let cleanup = dispatch_auto_open_chest_cleanup(plan, runtime)?;
                cleanup_actions.extend(cleanup);
            }

            for action in &pending_post_loop_actions {
                runtime.dispatch_auto_open_chest_action(action)?;
                post_loop_actions.push(action.clone());
            }

            return Ok(auto_open_chest_report(
                plan,
                completed,
                status,
                state,
                decisions,
                dispatched_actions,
                cleanup_actions,
                post_loop_actions,
            ));
        }

        decisions.push(decision);
    }
}

pub fn decide_auto_open_chest_step(
    plan: &AutoOpenChestExecutionPlan,
    observation: AutoOpenChestObservation,
) -> AutoOpenChestDecision {
    if plan.search_rule.requires_initial_chest_f_prompt && !observation.initial_chest_f_icon_exists
    {
        return AutoOpenChestDecision {
            result: AutoOpenChestDecisionResult::InitialPromptMissing,
            is_terminal: true,
            is_flower: false,
            actions: Vec::new(),
            post_loop_actions: Vec::new(),
        };
    }

    let Some(chest_icon) = observation.chest_icon else {
        return AutoOpenChestDecision {
            result: AutoOpenChestDecisionResult::ChestIconMissing,
            is_terminal: true,
            is_flower: false,
            actions: vec![
                AutoOpenChestAction::Log {
                    message: "未找到宝箱图标".to_string(),
                },
                genshin_action(GenshinAction::MoveForward, AutoOpenChestActionPress::KeyUp),
            ],
            post_loop_actions: Vec::new(),
        };
    };

    if observation.chest_f_icon_exists || observation.flower_f_icon_exists {
        let actions = vec![
            genshin_action(
                GenshinAction::PickUpOrInteract,
                AutoOpenChestActionPress::KeyPress,
            ),
            genshin_action(GenshinAction::MoveForward, AutoOpenChestActionPress::KeyUp),
        ];
        let post_loop_actions = if observation.flower_f_icon_exists {
            vec![genshin_action(
                plan.search_rule.flower_handler_action,
                AutoOpenChestActionPress::KeyPress,
            )]
        } else {
            Vec::new()
        };
        return AutoOpenChestDecision {
            result: if observation.flower_f_icon_exists {
                AutoOpenChestDecisionResult::FlowerInteracted
            } else {
                AutoOpenChestDecisionResult::ChestInteracted
            },
            is_terminal: true,
            is_flower: observation.flower_f_icon_exists,
            actions,
            post_loop_actions,
        };
    }

    let mut actions = Vec::new();
    let center_gap = chest_icon.width / 2 - chest_icon.x;
    if plan
        .search_rule
        .move_forward_when_half_width_gap_below_icon_width
        && center_gap.abs() < chest_icon.width
    {
        actions.push(genshin_action(
            GenshinAction::MoveForward,
            AutoOpenChestActionPress::KeyDown,
        ));
    }

    if chest_icon.y > plan.search_rule.scaled_backward_y_threshold {
        actions.push(genshin_action(
            GenshinAction::MoveBackward,
            AutoOpenChestActionPress::KeyPress,
        ));
        actions.push(AutoOpenChestAction::Delay {
            duration_ms: plan.search_rule.backward_delay_ms,
        });
        actions.push(AutoOpenChestAction::MouseMiddleClick);
    } else {
        let gap = observation.capture_width as i32 / 2 - chest_icon.x;
        actions.push(AutoOpenChestAction::MouseMoveBy {
            delta_x: gap / plan.search_rule.center_gap_divisor,
            delta_y: 0,
        });
    }
    actions.push(AutoOpenChestAction::Delay {
        duration_ms: plan.search_rule.loop_delay_ms,
    });

    AutoOpenChestDecision {
        result: AutoOpenChestDecisionResult::SearchingContinue,
        is_terminal: false,
        is_flower: false,
        actions,
        post_loop_actions: Vec::new(),
    }
}

fn auto_open_chest_status_from_decision(
    result: AutoOpenChestDecisionResult,
) -> AutoOpenChestExecutionStatus {
    match result {
        AutoOpenChestDecisionResult::InitialPromptMissing => {
            AutoOpenChestExecutionStatus::InitialPromptMissing
        }
        AutoOpenChestDecisionResult::ChestIconMissing => {
            AutoOpenChestExecutionStatus::ChestIconMissing
        }
        AutoOpenChestDecisionResult::SearchingContinue => AutoOpenChestExecutionStatus::TimedOut,
        AutoOpenChestDecisionResult::ChestInteracted => {
            AutoOpenChestExecutionStatus::ChestInteracted
        }
        AutoOpenChestDecisionResult::FlowerInteracted => {
            AutoOpenChestExecutionStatus::FlowerInteracted
        }
    }
}

fn dispatch_auto_open_chest_cleanup<R>(
    plan: &AutoOpenChestExecutionPlan,
    runtime: &mut R,
) -> Result<Vec<AutoOpenChestAction>>
where
    R: AutoOpenChestRuntime,
{
    if !plan.search_rule.always_releases_forward_on_loop_exit {
        return Ok(Vec::new());
    }
    let action = genshin_action(GenshinAction::MoveForward, AutoOpenChestActionPress::KeyUp);
    runtime.dispatch_auto_open_chest_action(&action)?;
    Ok(vec![action])
}

#[allow(clippy::too_many_arguments)]
fn auto_open_chest_report(
    plan: &AutoOpenChestExecutionPlan,
    completed: bool,
    status: AutoOpenChestExecutionStatus,
    state: AutoOpenChestExecutionState,
    decisions: Vec<AutoOpenChestDecision>,
    dispatched_actions: Vec<AutoOpenChestAction>,
    cleanup_actions: Vec<AutoOpenChestAction>,
    post_loop_actions: Vec<AutoOpenChestAction>,
) -> AutoOpenChestExecutionReport {
    AutoOpenChestExecutionReport {
        task_key: plan.task_key.clone(),
        completed,
        status,
        state,
        decisions,
        dispatched_actions,
        cleanup_actions,
        post_loop_actions,
    }
}

fn auto_open_chest_steps() -> Vec<AutoOpenChestStep> {
    vec![
        step(
            AutoOpenChestStepPhase::InitialPrompt,
            "detect initial chest F prompt",
            AutoOpenChestStepAction::DetectChestFIcon,
        ),
        step(
            AutoOpenChestStepPhase::SearchLoop,
            "detect chest icon",
            AutoOpenChestStepAction::DetectChestIcon,
        ),
        step(
            AutoOpenChestStepPhase::SearchLoop,
            "detect chest or flower F prompt",
            AutoOpenChestStepAction::DetectInteractionPrompt,
        ),
        step(
            AutoOpenChestStepPhase::SearchLoop,
            "hold forward when chest icon is centered",
            AutoOpenChestStepAction::MoveForwardWhenCentered,
        ),
        step(
            AutoOpenChestStepPhase::SearchLoop,
            "step backward and middle-click when icon is below threshold",
            AutoOpenChestStepAction::BackwardAndCameraResetWhenBehind,
        ),
        step(
            AutoOpenChestStepPhase::SearchLoop,
            "turn toward chest icon",
            AutoOpenChestStepAction::TurnTowardChest,
        ),
        step(
            AutoOpenChestStepPhase::Interaction,
            "press interact when F prompt is found",
            AutoOpenChestStepAction::Interact,
        ),
        step(
            AutoOpenChestStepPhase::FlowerHandling,
            "open Paimon menu after flower interaction",
            AutoOpenChestStepAction::OpenPaimonMenuForFlower,
        ),
        step(
            AutoOpenChestStepPhase::Cleanup,
            "release move forward on loop exit",
            AutoOpenChestStepAction::ReleaseMoveForward,
        ),
    ]
}

fn step(
    phase: AutoOpenChestStepPhase,
    label: impl Into<String>,
    action: AutoOpenChestStepAction,
) -> AutoOpenChestStep {
    AutoOpenChestStep {
        phase,
        label: label.into(),
        action,
    }
}

fn image_locator(
    page: &BvPage,
    asset: &str,
    roi: Option<Rect>,
    operation: BvLocatorOperation,
) -> Result<BvLocatorPlan> {
    let image = BvImage::new(asset).map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let locator = page
        .locator_for_image(&image, roi, 0.8)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    Ok(locator.plan(operation, Some(1)))
}

fn genshin_action(action: GenshinAction, press: AutoOpenChestActionPress) -> AutoOpenChestAction {
    AutoOpenChestAction::GenshinAction { action, press }
}

fn scaled_rect(size: Size, x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    let scale = size.width as f64 / AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH as f64;
    Rect::new(
        (x * scale).round() as i32,
        (y * scale).round() as i32,
        (width * scale).round() as i32,
        (height * scale).round() as i32,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn f64_member(value: &Value, names: [&str; 3]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(Value::as_f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_open_chest_pending_native_tracks_real_desktop_gaps() {
        let plan = plan_auto_open_chest(AutoOpenChestExecutionConfig::default()).unwrap();

        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("TaskContext/SystemControl preflight")));
        assert!(plan
            .pending_native
            .iter()
            .any(|item| item.contains("overlay cleanup is a no-op")));
        assert!(!plan
            .pending_native
            .iter()
            .any(|item| item.contains("overlay drawing adapters remain pending")));
    }
}
