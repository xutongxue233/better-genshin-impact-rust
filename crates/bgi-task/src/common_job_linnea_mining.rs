use crate::{Result, TaskError, TaskPortState};
use bgi_core::{plan_linnea_mining_action, GenshinAction};
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LINNEA_MINING_TASK_KEY: &str = "LinneaMining";
pub const LINNEA_MINING_DEFAULT_MINE_COUNT: i32 = 1;
pub const LINNEA_MINING_DEFAULT_SCAN_ROUNDS: i32 = 1;
pub const LINNEA_MINING_MODEL_NAME: &str = "BgiMine";
pub const LINNEA_MINING_MODEL_PATH: &str = "Assets/Model/Mine/bgi_mine.onnx";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub action_code: String,
    pub raw_params: Option<String>,
    pub mine_count: i32,
    pub scan_rounds: i32,
    pub prefer_right: bool,
    pub avatar_rule: LinneaMiningAvatarRule,
    pub aiming_rule: LinneaMiningAimingRule,
    pub detection_rule: LinneaMiningDetectionRule,
    pub cluster_rule: LinneaMiningClusterRule,
    pub alignment_rule: LinneaMiningAlignmentRule,
    pub scan_rule: LinneaMiningScanRule,
    pub mine_rule: LinneaMiningMineRule,
    pub cleanup_rule: LinneaMiningCleanupRule,
    pub steps: Vec<LinneaMiningStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LinneaMiningExecutionConfig {
    pub action_params: Option<String>,
    pub mine_count: Option<i32>,
    pub scan_rounds: Option<i32>,
}

impl LinneaMiningExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let Some(value) = value else {
            return Self::default();
        };

        if let Some(params) = value.as_str() {
            return Self {
                action_params: Some(params.to_string()),
                ..Self::default()
            };
        }

        Self {
            action_params: string_member(
                value,
                [
                    "actionParams",
                    "ActionParams",
                    "action_params",
                    "rawParams",
                    "raw_params",
                ],
            ),
            mine_count: i32_member(value, ["mineCount", "MineCount", "mine_count", "mines"]),
            scan_rounds: i32_member(value, ["scanRounds", "ScanRounds", "scan_rounds", "rounds"]),
        }
    }

    fn effective_action_params(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(mine_count) = self.mine_count {
            parts.push(format!("mines={mine_count}"));
        }
        if let Some(scan_rounds) = self.scan_rounds {
            parts.push(format!("rounds={scan_rounds}"));
        }
        if parts.is_empty() {
            self.action_params.clone()
        } else {
            Some(parts.join(","))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningAvatarRule {
    pub avatar_name: String,
    pub switch_avatar_before_mining: bool,
    pub switch_wait_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningAimingRule {
    pub aiming_mode_action: GenshinAction,
    pub enter_aim_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningDetectionRule {
    pub model_name: String,
    pub model_relative_path: String,
    pub accepted_label: String,
    pub confidence_threshold: f32,
    pub source: LinneaMiningDetectionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinneaMiningDetectionSource {
    FullCapture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningClusterRule {
    pub base_cluster_distance_1080p: f64,
    pub base_cluster_area_1080p: f64,
    pub base_alignment_expansion_1080p: f64,
    pub base_edge_ignore_1080p: f64,
    pub area_ratio_threshold: f64,
    pub prefer_right_when_scan_rounds_gt_one: bool,
    pub target_selection: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningAlignmentRule {
    pub max_inner_retry: u8,
    pub element_sight_refresh_ms: u32,
    pub refresh_release_ms: u32,
    pub refresh_hold_ms: u32,
    pub aim_sensitivity_factor_x: f64,
    pub aim_sensitivity_factor_y: f64,
    pub aim_move_delay_ms: u32,
    pub fallback_shot_on_last_successful_detection: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningScanRule {
    pub middle_button_hold_ms: u32,
    pub middle_button_release_ms: u32,
    pub compensate_detection_hold_ms: u32,
    pub compensate_move_wait_ms: u32,
    pub left_turn_step_1080p: i32,
    pub left_turn_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningMineRule {
    pub compensate_up_pixels: i32,
    pub compensate_up_wait_ms: u32,
    pub attack_button: String,
    pub after_attack_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningCleanupRule {
    pub leave_aiming_mode_action: GenshinAction,
    pub middle_button_up: bool,
    pub clear_vision_drawings: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningStep {
    pub phase: LinneaMiningStepPhase,
    pub condition: LinneaMiningStepCondition,
    pub label: String,
    pub action: LinneaMiningStepAction,
}

impl LinneaMiningStep {
    fn new(
        phase: LinneaMiningStepPhase,
        condition: LinneaMiningStepCondition,
        label: impl Into<String>,
        action: LinneaMiningStepAction,
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
pub enum LinneaMiningStepPhase {
    Setup,
    ScanLoop,
    Detection,
    Alignment,
    Mine,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningStepCondition {
    Always,
    PerScanRound,
    WhenClusterDetected,
    WhenClusterMissing,
    WhenAlignedOrFallback,
    WhenDetectionLost,
    BetweenScanRounds,
    Finally,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum LinneaMiningStepAction {
    SwitchAvatar {
        avatar_name: String,
        wait_ms: u32,
    },
    ToggleAimingMode {
        action: GenshinAction,
        wait_ms: u32,
    },
    HoldElementalSight {
        hold_ms: u32,
    },
    DetectMineralCluster,
    AlignTarget {
        max_retries: u8,
    },
    MineTarget {
        attack_button: String,
        after_attack_wait_ms: u32,
    },
    CompensateLostDetection,
    TurnForNextScanRound {
        left_turn_step_1080p: i32,
        wait_ms: u32,
    },
    ReleaseMiddleButton,
    ClearVisionDrawings,
    MarkNativePending {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningRuntimeActionKind {
    SwitchAvatar,
    AimingMode,
    ElementalSight,
    DetectMineralCluster,
    AlignTarget,
    MineAttack,
    TurnForNextScanRound,
    Cleanup,
    Delay,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningRuntimeOutcome {
    Completed,
    Detected,
    Missing,
    Aligned,
    Fallback,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningRuntimeActionReport {
    pub phase: LinneaMiningStepPhase,
    pub action_kind: LinneaMiningRuntimeActionKind,
    pub scan_round: Option<i32>,
    pub mine_index: Option<i32>,
    pub retry_index: Option<u8>,
    pub label: String,
    pub outcome: LinneaMiningRuntimeOutcome,
    pub target: Option<LinneaMiningTarget>,
    pub message: Option<String>,
}

impl LinneaMiningRuntimeActionReport {
    fn new(
        phase: LinneaMiningStepPhase,
        action_kind: LinneaMiningRuntimeActionKind,
        label: impl Into<String>,
        outcome: LinneaMiningRuntimeOutcome,
    ) -> Self {
        Self {
            phase,
            action_kind,
            scan_round: None,
            mine_index: None,
            retry_index: None,
            label: label.into(),
            outcome,
            target: None,
            message: None,
        }
    }

    fn scan_round(mut self, scan_round: i32) -> Self {
        self.scan_round = Some(scan_round);
        self
    }

    fn mine_index(mut self, mine_index: i32) -> Self {
        self.mine_index = Some(mine_index);
        self
    }

    fn retry_index(mut self, retry_index: u8) -> Self {
        self.retry_index = Some(retry_index);
        self
    }

    fn target(mut self, target: LinneaMiningTarget) -> Self {
        self.target = Some(target);
        self
    }

    fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningExecutorState {
    pub switched_avatar: bool,
    pub aiming_mode_entered: bool,
    pub middle_button_held: bool,
    pub cleanup_leave_aiming_attempted: bool,
    pub cleanup_middle_button_released: bool,
    pub cleanup_drawings_cleared: bool,
    pub scan_rounds_completed: i32,
    pub mined_count: i32,
    pub detections: u32,
    pub missing_detections: u32,
    pub alignment_attempts: u32,
    pub alignment_successes: u32,
    pub fallback_attacks: u32,
    pub left_turns: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningExecutionStatus {
    Completed,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningDecisionKind {
    Continue,
    Mine,
    RetryAlignment,
    FallbackMine,
    TurnLeftAndContinue,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningDecision {
    pub kind: LinneaMiningDecisionKind,
    pub scan_round: i32,
    pub mine_index: i32,
    pub target: Option<LinneaMiningTarget>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningExecutionReport {
    pub task_key: String,
    pub status: LinneaMiningExecutionStatus,
    pub completed: bool,
    pub state: LinneaMiningExecutorState,
    pub decisions: Vec<LinneaMiningDecision>,
    pub actions: Vec<LinneaMiningRuntimeActionReport>,
    pub cleanup_actions: Vec<LinneaMiningRuntimeActionReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningScreenSize {
    pub width: u32,
    pub height: u32,
}

impl Default for LinneaMiningScreenSize {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

impl From<Size> for LinneaMiningScreenSize {
    fn from(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<LinneaMiningScreenSize> for Size {
    fn from(size: LinneaMiningScreenSize) -> Self {
        Size::new(size.width, size.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl LinneaMiningRect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width: width.max(0),
            height: height.max(0),
        }
    }

    fn center(self) -> LinneaMiningPoint {
        LinneaMiningPoint {
            x: self.x + self.width / 2,
            y: self.y + self.height / 2,
        }
    }

    fn area(self) -> f64 {
        (self.width.max(0) as f64) * (self.height.max(0) as f64)
    }

    fn expanded(self, expansion: f64, screen: LinneaMiningScreenSize) -> Self {
        let x_expansion = ((self.width as f64) * expansion).round() as i32;
        let y_expansion = ((self.height as f64) * expansion).round() as i32;
        let x = (self.x - x_expansion).max(0);
        let y = (self.y - y_expansion).max(0);
        let right = (self.x + self.width + x_expansion).min(screen.width as i32);
        let bottom = (self.y + self.height + y_expansion).min(screen.height as i32);
        Self::new(x, y, (right - x).max(0), (bottom - y).max(0))
    }

    fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        Self::new(x, y, right - x, bottom - y)
    }

    fn is_inside_horizontal_margin(self, screen: LinneaMiningScreenSize, margin: f64) -> bool {
        let margin = margin.round() as i32;
        self.x >= margin && self.x + self.width <= screen.width as i32 - margin
    }
}

impl From<Rect> for LinneaMiningRect {
    fn from(rect: Rect) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }
}

impl From<LinneaMiningRect> for Rect {
    fn from(rect: LinneaMiningRect) -> Self {
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningDetection {
    pub rect: LinneaMiningRect,
    pub label: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningCluster {
    pub rects: Vec<LinneaMiningRect>,
    pub bounds: LinneaMiningRect,
    pub center: LinneaMiningPoint,
    pub area: f64,
}

impl LinneaMiningCluster {
    fn new(rects: Vec<LinneaMiningRect>) -> Option<Self> {
        let mut iter = rects.iter().copied();
        let first = iter.next()?;
        let bounds = iter.fold(first, LinneaMiningRect::union);
        Some(Self {
            center: bounds.center(),
            area: bounds.area(),
            bounds,
            rects,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningTarget {
    pub rect: LinneaMiningRect,
    pub aim_point: LinneaMiningPoint,
    pub cluster: LinneaMiningCluster,
    pub confidence: f32,
}

impl LinneaMiningTarget {
    fn from_cluster(cluster: LinneaMiningCluster, rect: LinneaMiningRect, confidence: f32) -> Self {
        Self {
            aim_point: rect.center(),
            rect,
            cluster,
            confidence,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningObservation {
    pub scan_round: i32,
    pub screen_size: LinneaMiningScreenSize,
    pub detections: Vec<LinneaMiningDetection>,
    pub selected_target: Option<LinneaMiningTarget>,
}

pub trait LinneaMiningRuntime {
    fn switch_linnea_avatar(&mut self, avatar_rule: &LinneaMiningAvatarRule) -> Result<()>;

    fn enter_aiming_mode(&mut self, aiming_rule: &LinneaMiningAimingRule) -> Result<()>;

    fn leave_aiming_mode(&mut self, cleanup_rule: &LinneaMiningCleanupRule) -> Result<()>;

    fn hold_elemental_sight(&mut self, scan_rule: &LinneaMiningScanRule) -> Result<()>;

    fn release_elemental_sight(&mut self, scan_rule: &LinneaMiningScanRule) -> Result<()>;

    fn wait_linnea_mining(&mut self, duration_ms: u32) -> Result<()>;

    fn capture_screen_size(&mut self) -> Result<LinneaMiningScreenSize>;

    fn detect_minerals(
        &mut self,
        detection_rule: &LinneaMiningDetectionRule,
        screen_size: LinneaMiningScreenSize,
    ) -> Result<Vec<LinneaMiningDetection>>;

    fn move_aim(&mut self, delta_x: i32, delta_y: i32) -> Result<()>;

    fn attack_mine(&mut self, mine_rule: &LinneaMiningMineRule) -> Result<()>;

    fn turn_left_for_next_scan_round(&mut self, pixels_1080p: i32, wait_ms: u32) -> Result<()>;

    fn clear_mining_overlay(&mut self) -> Result<()>;

    fn draw_mining_target(
        &mut self,
        _observation: &LinneaMiningObservation,
        _target: Option<&LinneaMiningTarget>,
    ) -> Result<()> {
        Ok(())
    }
}

pub fn plan_linnea_mining(config: LinneaMiningExecutionConfig) -> LinneaMiningExecutionPlan {
    let effective_action_params = config.effective_action_params();
    let core = plan_linnea_mining_action(effective_action_params.as_deref());

    let avatar_rule = LinneaMiningAvatarRule {
        avatar_name: core.avatar_name.clone(),
        switch_avatar_before_mining: core.switch_avatar_before_mining,
        switch_wait_ms: core.switch_wait_ms,
    };
    let aiming_rule = LinneaMiningAimingRule {
        aiming_mode_action: core.aiming_mode_action,
        enter_aim_wait_ms: core.enter_aim_wait_ms,
    };
    let detection_rule = LinneaMiningDetectionRule {
        model_name: core.detection_rule.model_name.clone(),
        model_relative_path: core.detection_rule.model_relative_path.clone(),
        accepted_label: core.detection_rule.accepted_label.clone(),
        confidence_threshold: core.detection_rule.confidence_threshold,
        source: LinneaMiningDetectionSource::FullCapture,
    };
    let cluster_rule = LinneaMiningClusterRule {
        base_cluster_distance_1080p: core.cluster_rule.base_cluster_distance_1080p,
        base_cluster_area_1080p: core.cluster_rule.base_cluster_area_1080p,
        base_alignment_expansion_1080p: core.cluster_rule.base_alignment_expansion_1080p,
        base_edge_ignore_1080p: core.cluster_rule.base_edge_ignore_1080p,
        area_ratio_threshold: core.cluster_rule.area_ratio_threshold,
        prefer_right_when_scan_rounds_gt_one: core
            .cluster_rule
            .prefer_right_when_scan_rounds_gt_one,
        target_selection: core.cluster_rule.target_selection.clone(),
    };
    let alignment_rule = LinneaMiningAlignmentRule {
        max_inner_retry: core.alignment_rule.max_inner_retry,
        element_sight_refresh_ms: core.alignment_rule.element_sight_refresh_ms,
        refresh_release_ms: core.alignment_rule.refresh_release_ms,
        refresh_hold_ms: core.alignment_rule.refresh_hold_ms,
        aim_sensitivity_factor_x: core.alignment_rule.aim_sensitivity_factor_x,
        aim_sensitivity_factor_y: core.alignment_rule.aim_sensitivity_factor_y,
        aim_move_delay_ms: core.alignment_rule.aim_move_delay_ms,
        fallback_shot_on_last_successful_detection: core
            .alignment_rule
            .fallback_shot_on_last_successful_detection,
    };
    let scan_rule = LinneaMiningScanRule {
        middle_button_hold_ms: core.scan_rule.middle_button_hold_ms,
        middle_button_release_ms: core.scan_rule.middle_button_release_ms,
        compensate_detection_hold_ms: core.scan_rule.compensate_detection_hold_ms,
        compensate_move_wait_ms: core.scan_rule.compensate_move_wait_ms,
        left_turn_step_1080p: core.scan_rule.left_turn_step_1080p,
        left_turn_wait_ms: core.scan_rule.left_turn_wait_ms,
    };
    let mine_rule = LinneaMiningMineRule {
        compensate_up_pixels: core.mine_rule.compensate_up_pixels,
        compensate_up_wait_ms: core.mine_rule.compensate_up_wait_ms,
        attack_button: core.mine_rule.attack_button.clone(),
        after_attack_wait_ms: core.mine_rule.after_attack_wait_ms,
    };
    let cleanup_rule = LinneaMiningCleanupRule {
        leave_aiming_mode_action: core.cleanup_rule.leave_aiming_mode_action,
        middle_button_up: core.cleanup_rule.middle_button_up,
        clear_vision_drawings: core.cleanup_rule.clear_vision_drawings,
    };

    LinneaMiningExecutionPlan {
        task_key: LINNEA_MINING_TASK_KEY.to_string(),
        display_name: "莉奈娅挖矿".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        action_code: core.action_code,
        raw_params: core.raw_params,
        mine_count: core.mine_count,
        scan_rounds: core.scan_rounds,
        prefer_right: core.prefer_right,
        avatar_rule: avatar_rule.clone(),
        aiming_rule,
        detection_rule,
        cluster_rule,
        alignment_rule: alignment_rule.clone(),
        scan_rule: scan_rule.clone(),
        mine_rule: mine_rule.clone(),
        cleanup_rule,
        steps: linnea_mining_steps(&avatar_rule, aiming_rule, &alignment_rule, &scan_rule, &mine_rule),
        notes:
            "LinneaMining preserves the legacy avatar requirement, BgiMine detection, clustering, aiming, scan, mine, compensation, and cleanup rules with an injectable Rust executor boundary; desktop adapters still provide live capture, ONNX inference, input, and overlay implementations."
                .to_string(),
    }
}

pub fn execute_linnea_mining_plan<R>(
    plan: &LinneaMiningExecutionPlan,
    runtime: &mut R,
) -> Result<LinneaMiningExecutionReport>
where
    R: LinneaMiningRuntime,
{
    let mut state = LinneaMiningExecutorState::default();
    let mut actions = Vec::new();
    let mut cleanup_actions = Vec::new();
    let mut decisions = Vec::new();
    let execution_result =
        execute_linnea_mining_body(plan, runtime, &mut state, &mut actions, &mut decisions);
    let cleanup_result = cleanup_linnea_mining(plan, runtime, &mut state, &mut cleanup_actions);

    match (execution_result, cleanup_result) {
        (Ok(()), Ok(())) => {
            let completed = state.mined_count >= effective_linnea_count(plan.mine_count);
            Ok(LinneaMiningExecutionReport {
                task_key: plan.task_key.clone(),
                status: if completed {
                    LinneaMiningExecutionStatus::Completed
                } else {
                    LinneaMiningExecutionStatus::Partial
                },
                completed,
                state,
                decisions,
                actions,
                cleanup_actions,
            })
        }
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(execution_error), Err(cleanup_error)) => Err(TaskError::CommonJobExecution(format!(
            "{execution_error}; cleanup also failed: {cleanup_error}"
        ))),
    }
}

fn execute_linnea_mining_body<R>(
    plan: &LinneaMiningExecutionPlan,
    runtime: &mut R,
    state: &mut LinneaMiningExecutorState,
    actions: &mut Vec<LinneaMiningRuntimeActionReport>,
    decisions: &mut Vec<LinneaMiningDecision>,
) -> Result<()>
where
    R: LinneaMiningRuntime,
{
    if plan.avatar_rule.switch_avatar_before_mining {
        runtime.switch_linnea_avatar(&plan.avatar_rule)?;
        state.switched_avatar = true;
        actions.push(LinneaMiningRuntimeActionReport::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningRuntimeActionKind::SwitchAvatar,
            "switch to Linnea before mining",
            LinneaMiningRuntimeOutcome::Completed,
        ));
        if plan.avatar_rule.switch_wait_ms > 0 {
            runtime.wait_linnea_mining(plan.avatar_rule.switch_wait_ms)?;
            actions.push(LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Setup,
                LinneaMiningRuntimeActionKind::Delay,
                "wait after switching Linnea",
                LinneaMiningRuntimeOutcome::Completed,
            ));
        }
    }

    runtime.enter_aiming_mode(&plan.aiming_rule)?;
    state.aiming_mode_entered = true;
    actions.push(LinneaMiningRuntimeActionReport::new(
        LinneaMiningStepPhase::Setup,
        LinneaMiningRuntimeActionKind::AimingMode,
        "enter aiming mode",
        LinneaMiningRuntimeOutcome::Completed,
    ));
    if plan.aiming_rule.enter_aim_wait_ms > 0 {
        runtime.wait_linnea_mining(plan.aiming_rule.enter_aim_wait_ms)?;
        actions.push(LinneaMiningRuntimeActionReport::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningRuntimeActionKind::Delay,
            "wait after entering aiming mode",
            LinneaMiningRuntimeOutcome::Completed,
        ));
    }

    let mine_goal = effective_linnea_count(plan.mine_count);
    let scan_rounds = effective_linnea_count(plan.scan_rounds);

    for scan_round in 1..=scan_rounds {
        state.scan_rounds_completed = scan_round;
        if state.mined_count >= mine_goal {
            decisions.push(LinneaMiningDecision {
                kind: LinneaMiningDecisionKind::Stop,
                scan_round,
                mine_index: state.mined_count,
                target: None,
                reason: "mine goal reached".to_string(),
            });
            break;
        }

        runtime.hold_elemental_sight(&plan.scan_rule)?;
        state.middle_button_held = true;
        actions.push(
            LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::ScanLoop,
                LinneaMiningRuntimeActionKind::ElementalSight,
                "hold elemental sight before cluster detection",
                LinneaMiningRuntimeOutcome::Completed,
            )
            .scan_round(scan_round),
        );
        if plan.scan_rule.middle_button_hold_ms > 0 {
            runtime.wait_linnea_mining(plan.scan_rule.middle_button_hold_ms)?;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::ScanLoop,
                    LinneaMiningRuntimeActionKind::Delay,
                    "wait while elemental sight is held",
                    LinneaMiningRuntimeOutcome::Completed,
                )
                .scan_round(scan_round),
            );
        }

        let observation = observe_linnea_mining(plan, runtime, scan_round)?;
        let Some(mut target) = observation.selected_target.clone() else {
            state.missing_detections += 1;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::Detection,
                    LinneaMiningRuntimeActionKind::DetectMineralCluster,
                    "detect nearest mineral cluster",
                    LinneaMiningRuntimeOutcome::Missing,
                )
                .scan_round(scan_round)
                .message(format!("{} raw detection(s)", observation.detections.len())),
            );
            decisions.push(LinneaMiningDecision {
                kind: if scan_round < scan_rounds {
                    LinneaMiningDecisionKind::TurnLeftAndContinue
                } else {
                    LinneaMiningDecisionKind::Stop
                },
                scan_round,
                mine_index: state.mined_count,
                target: None,
                reason: "no accepted mineral cluster detected".to_string(),
            });

            if scan_round < scan_rounds {
                runtime.turn_left_for_next_scan_round(
                    plan.scan_rule.left_turn_step_1080p,
                    plan.scan_rule.left_turn_wait_ms,
                )?;
                state.left_turns += 1;
                actions.push(
                    LinneaMiningRuntimeActionReport::new(
                        LinneaMiningStepPhase::ScanLoop,
                        LinneaMiningRuntimeActionKind::TurnForNextScanRound,
                        "turn left before next scan round",
                        LinneaMiningRuntimeOutcome::Completed,
                    )
                    .scan_round(scan_round),
                );
            }
            continue;
        };

        state.detections += 1;
        runtime.draw_mining_target(&observation, Some(&target))?;
        actions.push(
            LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Detection,
                LinneaMiningRuntimeActionKind::DetectMineralCluster,
                "detect nearest mineral cluster",
                LinneaMiningRuntimeOutcome::Detected,
            )
            .scan_round(scan_round)
            .target(target.clone())
            .message(format!("{} raw detection(s)", observation.detections.len())),
        );
        decisions.push(LinneaMiningDecision {
            kind: LinneaMiningDecisionKind::RetryAlignment,
            scan_round,
            mine_index: state.mined_count,
            target: Some(target.clone()),
            reason: "cluster detected; begin alignment".to_string(),
        });

        let mut aligned = linnea_target_is_aligned(linnea_alignment_delta(
            plan,
            target.aim_point,
            observation.screen_size,
        ));
        if aligned {
            state.alignment_successes += 1;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::Alignment,
                    LinneaMiningRuntimeActionKind::AlignTarget,
                    "initial detection already aligned",
                    LinneaMiningRuntimeOutcome::Aligned,
                )
                .scan_round(scan_round)
                .target(target.clone()),
            );
            decisions.push(LinneaMiningDecision {
                kind: LinneaMiningDecisionKind::Mine,
                scan_round,
                mine_index: state.mined_count,
                target: Some(target.clone()),
                reason: "initial target already aligned".to_string(),
            });
        }
        let max_retry = plan.alignment_rule.max_inner_retry.max(1);
        for retry_index in 1..=max_retry {
            if aligned {
                break;
            }
            state.alignment_attempts += 1;
            let delta = linnea_alignment_delta(plan, target.aim_point, observation.screen_size);
            runtime.move_aim(delta.x, delta.y)?;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::Alignment,
                    LinneaMiningRuntimeActionKind::AlignTarget,
                    "align target with retry and elemental-sight refresh",
                    LinneaMiningRuntimeOutcome::Completed,
                )
                .scan_round(scan_round)
                .retry_index(retry_index)
                .target(target.clone())
                .message(format!("delta=({}, {})", delta.x, delta.y)),
            );
            if plan.alignment_rule.aim_move_delay_ms > 0 {
                runtime.wait_linnea_mining(plan.alignment_rule.aim_move_delay_ms)?;
            }

            let refreshed = refresh_linnea_detection(plan, runtime, scan_round)?;
            if let Some(refreshed_target) = refreshed.selected_target.clone() {
                let refreshed_delta =
                    linnea_alignment_delta(plan, refreshed_target.aim_point, refreshed.screen_size);
                target = refreshed_target;
                if linnea_target_is_aligned(refreshed_delta) {
                    aligned = true;
                    state.alignment_successes += 1;
                    actions.push(
                        LinneaMiningRuntimeActionReport::new(
                            LinneaMiningStepPhase::Alignment,
                            LinneaMiningRuntimeActionKind::AlignTarget,
                            "alignment confirmed by refreshed detection",
                            LinneaMiningRuntimeOutcome::Aligned,
                        )
                        .scan_round(scan_round)
                        .retry_index(retry_index)
                        .target(target.clone()),
                    );
                    decisions.push(LinneaMiningDecision {
                        kind: LinneaMiningDecisionKind::Mine,
                        scan_round,
                        mine_index: state.mined_count,
                        target: Some(target.clone()),
                        reason: "target aligned".to_string(),
                    });
                    break;
                }
            } else if plan
                .alignment_rule
                .fallback_shot_on_last_successful_detection
            {
                actions.push(
                    LinneaMiningRuntimeActionReport::new(
                        LinneaMiningStepPhase::Alignment,
                        LinneaMiningRuntimeActionKind::DetectMineralCluster,
                        "refreshed detection lost target",
                        LinneaMiningRuntimeOutcome::Missing,
                    )
                    .scan_round(scan_round)
                    .retry_index(retry_index)
                    .target(target.clone()),
                );
                break;
            }
        }

        if !aligned {
            if !plan
                .alignment_rule
                .fallback_shot_on_last_successful_detection
            {
                actions.push(
                    LinneaMiningRuntimeActionReport::new(
                        LinneaMiningStepPhase::Alignment,
                        LinneaMiningRuntimeActionKind::AlignTarget,
                        "alignment failed without fallback",
                        LinneaMiningRuntimeOutcome::Failed,
                    )
                    .scan_round(scan_round)
                    .target(target.clone()),
                );
                decisions.push(LinneaMiningDecision {
                    kind: LinneaMiningDecisionKind::Stop,
                    scan_round,
                    mine_index: state.mined_count,
                    target: Some(target),
                    reason: "alignment failed and fallback is disabled".to_string(),
                });
                break;
            }

            state.fallback_attacks += 1;
            runtime.move_aim(0, plan.mine_rule.compensate_up_pixels)?;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::Alignment,
                    LinneaMiningRuntimeActionKind::AlignTarget,
                    "fallback compensate camera after lost detection",
                    LinneaMiningRuntimeOutcome::Fallback,
                )
                .scan_round(scan_round)
                .target(target.clone()),
            );
            if plan.mine_rule.compensate_up_wait_ms > 0 {
                runtime.wait_linnea_mining(plan.mine_rule.compensate_up_wait_ms)?;
            }
            decisions.push(LinneaMiningDecision {
                kind: LinneaMiningDecisionKind::FallbackMine,
                scan_round,
                mine_index: state.mined_count,
                target: Some(target.clone()),
                reason: "alignment did not converge; use last successful detection".to_string(),
            });
        }

        runtime.attack_mine(&plan.mine_rule)?;
        state.mined_count += 1;
        actions.push(
            LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Mine,
                LinneaMiningRuntimeActionKind::MineAttack,
                "shoot mineral target",
                if aligned {
                    LinneaMiningRuntimeOutcome::Aligned
                } else {
                    LinneaMiningRuntimeOutcome::Fallback
                },
            )
            .scan_round(scan_round)
            .mine_index(state.mined_count)
            .target(target),
        );
        if plan.mine_rule.after_attack_wait_ms > 0 {
            runtime.wait_linnea_mining(plan.mine_rule.after_attack_wait_ms)?;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::Mine,
                    LinneaMiningRuntimeActionKind::Delay,
                    "wait after mining attack",
                    LinneaMiningRuntimeOutcome::Completed,
                )
                .scan_round(scan_round)
                .mine_index(state.mined_count),
            );
        }

        if state.mined_count < mine_goal && scan_round < scan_rounds {
            runtime.turn_left_for_next_scan_round(
                plan.scan_rule.left_turn_step_1080p,
                plan.scan_rule.left_turn_wait_ms,
            )?;
            state.left_turns += 1;
            actions.push(
                LinneaMiningRuntimeActionReport::new(
                    LinneaMiningStepPhase::ScanLoop,
                    LinneaMiningRuntimeActionKind::TurnForNextScanRound,
                    "turn left before next scan round",
                    LinneaMiningRuntimeOutcome::Completed,
                )
                .scan_round(scan_round),
            );
        }
    }

    Ok(())
}

fn cleanup_linnea_mining<R>(
    plan: &LinneaMiningExecutionPlan,
    runtime: &mut R,
    state: &mut LinneaMiningExecutorState,
    cleanup_actions: &mut Vec<LinneaMiningRuntimeActionReport>,
) -> Result<()>
where
    R: LinneaMiningRuntime,
{
    let mut first_error: Option<TaskError> = None;

    if let Err(error) = runtime.leave_aiming_mode(&plan.cleanup_rule) {
        first_error.get_or_insert(error);
        cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningRuntimeActionKind::Cleanup,
            "leave aiming mode",
            LinneaMiningRuntimeOutcome::Failed,
        ));
    } else {
        state.cleanup_leave_aiming_attempted = true;
        cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningRuntimeActionKind::Cleanup,
            "leave aiming mode",
            LinneaMiningRuntimeOutcome::Completed,
        ));
    }

    if plan.cleanup_rule.middle_button_up {
        if let Err(error) = runtime.release_elemental_sight(&plan.scan_rule) {
            first_error.get_or_insert(error);
            cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Cleanup,
                LinneaMiningRuntimeActionKind::ElementalSight,
                "release elemental sight",
                LinneaMiningRuntimeOutcome::Failed,
            ));
        } else {
            state.middle_button_held = false;
            state.cleanup_middle_button_released = true;
            cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Cleanup,
                LinneaMiningRuntimeActionKind::ElementalSight,
                "release elemental sight",
                LinneaMiningRuntimeOutcome::Completed,
            ));
        }
    }

    if plan.cleanup_rule.clear_vision_drawings {
        if let Err(error) = runtime.clear_mining_overlay() {
            first_error.get_or_insert(error);
            cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Cleanup,
                LinneaMiningRuntimeActionKind::Overlay,
                "clear mining overlay drawings",
                LinneaMiningRuntimeOutcome::Failed,
            ));
        } else {
            state.cleanup_drawings_cleared = true;
            cleanup_actions.push(LinneaMiningRuntimeActionReport::new(
                LinneaMiningStepPhase::Cleanup,
                LinneaMiningRuntimeActionKind::Overlay,
                "clear mining overlay drawings",
                LinneaMiningRuntimeOutcome::Completed,
            ));
        }
    }

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn observe_linnea_mining<R>(
    plan: &LinneaMiningExecutionPlan,
    runtime: &mut R,
    scan_round: i32,
) -> Result<LinneaMiningObservation>
where
    R: LinneaMiningRuntime,
{
    let screen_size = runtime.capture_screen_size()?;
    let detections = runtime.detect_minerals(&plan.detection_rule, screen_size)?;
    let selected_target = select_linnea_mining_target(
        &detections,
        screen_size,
        &plan.detection_rule,
        &plan.cluster_rule,
        plan.prefer_right,
    );
    Ok(LinneaMiningObservation {
        scan_round,
        screen_size,
        detections,
        selected_target,
    })
}

fn refresh_linnea_detection<R>(
    plan: &LinneaMiningExecutionPlan,
    runtime: &mut R,
    scan_round: i32,
) -> Result<LinneaMiningObservation>
where
    R: LinneaMiningRuntime,
{
    runtime.release_elemental_sight(&plan.scan_rule)?;
    if plan.alignment_rule.refresh_release_ms > 0 {
        runtime.wait_linnea_mining(plan.alignment_rule.refresh_release_ms)?;
    }
    runtime.hold_elemental_sight(&plan.scan_rule)?;
    if plan.alignment_rule.refresh_hold_ms > 0 {
        runtime.wait_linnea_mining(plan.alignment_rule.refresh_hold_ms)?;
    }
    observe_linnea_mining(plan, runtime, scan_round)
}

pub fn select_linnea_mining_target(
    detections: &[LinneaMiningDetection],
    screen_size: LinneaMiningScreenSize,
    detection_rule: &LinneaMiningDetectionRule,
    cluster_rule: &LinneaMiningClusterRule,
    prefer_right: bool,
) -> Option<LinneaMiningTarget> {
    let accepted: Vec<_> = detections
        .iter()
        .filter(|detection| {
            detection.label == detection_rule.accepted_label
                && detection.confidence >= detection_rule.confidence_threshold
                && detection.rect.is_inside_horizontal_margin(
                    screen_size,
                    scaled_1080p(cluster_rule.base_edge_ignore_1080p, screen_size),
                )
        })
        .cloned()
        .collect();
    if accepted.is_empty() {
        return None;
    }

    let clusters = linnea_mining_clusters(&accepted, screen_size, cluster_rule);
    let center = LinneaMiningPoint {
        x: (screen_size.width / 2) as i32,
        y: (screen_size.height / 2) as i32,
    };
    let selected_cluster = clusters.into_iter().min_by(|a, b| {
        distance_sq(a.center, center)
            .partial_cmp(&distance_sq(b.center, center))
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    let mut candidate_rects = selected_cluster.rects.clone();
    candidate_rects.sort_by(|a, b| {
        distance_sq(a.center(), center)
            .partial_cmp(&distance_sq(b.center(), center))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if prefer_right
        && cluster_rule.prefer_right_when_scan_rounds_gt_one
        && candidate_rects.len() >= 2
    {
        candidate_rects.truncate(2);
        candidate_rects.sort_by_key(|rect| rect.center().x);
        candidate_rects.last().copied()
    } else {
        candidate_rects.first().copied()
    }
    .map(|rect| {
        let confidence = accepted
            .iter()
            .filter(|detection| detection.rect == rect)
            .map(|detection| detection.confidence)
            .fold(0.0_f32, f32::max);
        LinneaMiningTarget::from_cluster(selected_cluster, rect, confidence)
    })
}

fn linnea_mining_clusters(
    detections: &[LinneaMiningDetection],
    screen_size: LinneaMiningScreenSize,
    cluster_rule: &LinneaMiningClusterRule,
) -> Vec<LinneaMiningCluster> {
    let mut clusters: Vec<Vec<LinneaMiningRect>> = Vec::new();
    let expansion = scaled_1080p(cluster_rule.base_alignment_expansion_1080p, screen_size);
    let cluster_distance = scaled_1080p(cluster_rule.base_cluster_distance_1080p, screen_size);
    let min_area = scaled_area_1080p(cluster_rule.base_cluster_area_1080p, screen_size);

    for detection in detections {
        let rect = detection.rect;
        let expanded = rect.expanded(expansion, screen_size);
        let mut target_cluster = None;
        for (index, cluster) in clusters.iter().enumerate() {
            if cluster.iter().any(|existing| {
                expanded.intersects(existing.expanded(expansion, screen_size))
                    || distance_sq(rect.center(), existing.center())
                        <= cluster_distance * cluster_distance
            }) {
                target_cluster = Some(index);
                break;
            }
        }

        if let Some(index) = target_cluster {
            clusters[index].push(rect);
        } else {
            clusters.push(vec![rect]);
        }
    }

    clusters
        .into_iter()
        .filter_map(LinneaMiningCluster::new)
        .filter(|cluster| {
            cluster.area >= min_area
                || cluster.rects.iter().map(|rect| rect.area()).sum::<f64>()
                    >= min_area / cluster_rule.area_ratio_threshold.max(1.0)
        })
        .collect()
}

fn linnea_alignment_delta(
    plan: &LinneaMiningExecutionPlan,
    aim_point: LinneaMiningPoint,
    screen_size: LinneaMiningScreenSize,
) -> LinneaMiningPoint {
    let center_x = screen_size.width as f64 / 2.0;
    let center_y = screen_size.height as f64 / 2.0;
    LinneaMiningPoint {
        x: ((aim_point.x as f64 - center_x) * plan.alignment_rule.aim_sensitivity_factor_x).round()
            as i32,
        y: ((aim_point.y as f64 - center_y) * plan.alignment_rule.aim_sensitivity_factor_y).round()
            as i32,
    }
}

fn linnea_target_is_aligned(delta: LinneaMiningPoint) -> bool {
    delta.x.abs() <= 2 && delta.y.abs() <= 2
}

fn effective_linnea_count(value: i32) -> i32 {
    value.max(1)
}

fn scaled_1080p(value: f64, screen_size: LinneaMiningScreenSize) -> f64 {
    let scale_x = screen_size.width as f64 / 1920.0;
    let scale_y = screen_size.height as f64 / 1080.0;
    value * scale_x.min(scale_y)
}

fn scaled_area_1080p(value: f64, screen_size: LinneaMiningScreenSize) -> f64 {
    let scale_x = screen_size.width as f64 / 1920.0;
    let scale_y = screen_size.height as f64 / 1080.0;
    value * scale_x * scale_y
}

fn distance_sq(a: LinneaMiningPoint, b: LinneaMiningPoint) -> f64 {
    let x = (a.x - b.x) as f64;
    let y = (a.y - b.y) as f64;
    x * x + y * y
}

fn linnea_mining_steps(
    avatar_rule: &LinneaMiningAvatarRule,
    aiming_rule: LinneaMiningAimingRule,
    alignment_rule: &LinneaMiningAlignmentRule,
    scan_rule: &LinneaMiningScanRule,
    mine_rule: &LinneaMiningMineRule,
) -> Vec<LinneaMiningStep> {
    vec![
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningStepCondition::Always,
            "switch to Linnea before mining",
            LinneaMiningStepAction::SwitchAvatar {
                avatar_name: avatar_rule.avatar_name.clone(),
                wait_ms: avatar_rule.switch_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningStepCondition::Always,
            "enter aiming mode",
            LinneaMiningStepAction::ToggleAimingMode {
                action: aiming_rule.aiming_mode_action,
                wait_ms: aiming_rule.enter_aim_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::ScanLoop,
            LinneaMiningStepCondition::PerScanRound,
            "hold elemental sight before cluster detection",
            LinneaMiningStepAction::HoldElementalSight {
                hold_ms: scan_rule.middle_button_hold_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Detection,
            LinneaMiningStepCondition::PerScanRound,
            "detect nearest mineral cluster",
            LinneaMiningStepAction::DetectMineralCluster,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Alignment,
            LinneaMiningStepCondition::WhenClusterDetected,
            "align target with retry and elemental-sight refresh",
            LinneaMiningStepAction::AlignTarget {
                max_retries: alignment_rule.max_inner_retry,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Mine,
            LinneaMiningStepCondition::WhenAlignedOrFallback,
            "shoot mineral target",
            LinneaMiningStepAction::MineTarget {
                attack_button: mine_rule.attack_button.clone(),
                after_attack_wait_ms: mine_rule.after_attack_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Alignment,
            LinneaMiningStepCondition::WhenDetectionLost,
            "compensate camera after lost detection",
            LinneaMiningStepAction::CompensateLostDetection,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::ScanLoop,
            LinneaMiningStepCondition::BetweenScanRounds,
            "turn left before next scan round",
            LinneaMiningStepAction::TurnForNextScanRound {
                left_turn_step_1080p: scan_rule.left_turn_step_1080p,
                wait_ms: scan_rule.left_turn_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "leave aiming mode",
            LinneaMiningStepAction::ToggleAimingMode {
                action: aiming_rule.aiming_mode_action,
                wait_ms: 0,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "release elemental sight",
            LinneaMiningStepAction::ReleaseMiddleButton,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "clear mining overlay drawings",
            LinneaMiningStepAction::ClearVisionDrawings,
        ),
    ]
}

fn string_member(value: &Value, keys: impl IntoIterator<Item = &'static str>) -> Option<String> {
    keys.into_iter().find_map(|key| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn i32_member(value: &Value, keys: impl IntoIterator<Item = &'static str>) -> Option<i32> {
    keys.into_iter().find_map(|key| {
        value.get(key).and_then(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .or_else(|| value.as_str().and_then(|value| value.parse::<i32>().ok()))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Default)]
    struct FakeLinneaMiningRuntime {
        events: Vec<String>,
        detections: VecDeque<Vec<LinneaMiningDetection>>,
        screen_size: LinneaMiningScreenSize,
        fail_move_aim: bool,
        fail_attack: bool,
    }

    impl FakeLinneaMiningRuntime {
        fn new(detections: impl IntoIterator<Item = Vec<LinneaMiningDetection>>) -> Self {
            Self {
                events: Vec::new(),
                detections: detections.into_iter().collect(),
                screen_size: LinneaMiningScreenSize::default(),
                fail_move_aim: false,
                fail_attack: false,
            }
        }

        fn fail_move_aim(mut self) -> Self {
            self.fail_move_aim = true;
            self
        }

        fn fail_attack(mut self) -> Self {
            self.fail_attack = true;
            self
        }

        fn event_count(&self, prefix: &str) -> usize {
            self.events
                .iter()
                .filter(|event| event.starts_with(prefix))
                .count()
        }
    }

    impl LinneaMiningRuntime for FakeLinneaMiningRuntime {
        fn switch_linnea_avatar(&mut self, avatar_rule: &LinneaMiningAvatarRule) -> Result<()> {
            self.events
                .push(format!("switch_avatar:{}", avatar_rule.avatar_name));
            Ok(())
        }

        fn enter_aiming_mode(&mut self, _: &LinneaMiningAimingRule) -> Result<()> {
            self.events.push("enter_aiming".to_string());
            Ok(())
        }

        fn leave_aiming_mode(&mut self, _: &LinneaMiningCleanupRule) -> Result<()> {
            self.events.push("leave_aiming".to_string());
            Ok(())
        }

        fn hold_elemental_sight(&mut self, _: &LinneaMiningScanRule) -> Result<()> {
            self.events.push("middle_down".to_string());
            Ok(())
        }

        fn release_elemental_sight(&mut self, _: &LinneaMiningScanRule) -> Result<()> {
            self.events.push("middle_up".to_string());
            Ok(())
        }

        fn wait_linnea_mining(&mut self, duration_ms: u32) -> Result<()> {
            self.events.push(format!("wait:{duration_ms}"));
            Ok(())
        }

        fn capture_screen_size(&mut self) -> Result<LinneaMiningScreenSize> {
            self.events.push("capture_size".to_string());
            Ok(self.screen_size)
        }

        fn detect_minerals(
            &mut self,
            _: &LinneaMiningDetectionRule,
            _: LinneaMiningScreenSize,
        ) -> Result<Vec<LinneaMiningDetection>> {
            self.events.push("detect".to_string());
            Ok(self.detections.pop_front().unwrap_or_default())
        }

        fn move_aim(&mut self, delta_x: i32, delta_y: i32) -> Result<()> {
            self.events.push(format!("move_aim:{delta_x},{delta_y}"));
            if self.fail_move_aim {
                Err(TaskError::CommonJobExecution(
                    "injected aim failure".to_string(),
                ))
            } else {
                Ok(())
            }
        }

        fn attack_mine(&mut self, mine_rule: &LinneaMiningMineRule) -> Result<()> {
            self.events
                .push(format!("attack:{}", mine_rule.attack_button));
            if self.fail_attack {
                Err(TaskError::CommonJobExecution(
                    "injected attack failure".to_string(),
                ))
            } else {
                Ok(())
            }
        }

        fn turn_left_for_next_scan_round(&mut self, pixels_1080p: i32, wait_ms: u32) -> Result<()> {
            self.events
                .push(format!("turn_left:{pixels_1080p}:{wait_ms}"));
            Ok(())
        }

        fn clear_mining_overlay(&mut self) -> Result<()> {
            self.events.push("clear_overlay".to_string());
            Ok(())
        }

        fn draw_mining_target(
            &mut self,
            _: &LinneaMiningObservation,
            target: Option<&LinneaMiningTarget>,
        ) -> Result<()> {
            self.events
                .push(format!("draw_target:{}", target.is_some()));
            Ok(())
        }
    }

    fn ore_rect(x: i32, y: i32, width: i32, height: i32) -> LinneaMiningDetection {
        LinneaMiningDetection {
            rect: LinneaMiningRect::new(x, y, width, height),
            label: "ore".to_string(),
            confidence: 0.91,
        }
    }

    fn test_plan(mine_count: i32, scan_rounds: i32) -> LinneaMiningExecutionPlan {
        let mut plan = plan_linnea_mining(LinneaMiningExecutionConfig {
            mine_count: Some(mine_count),
            scan_rounds: Some(scan_rounds),
            ..LinneaMiningExecutionConfig::default()
        });
        plan.avatar_rule.switch_wait_ms = 0;
        plan.aiming_rule.enter_aim_wait_ms = 0;
        plan.scan_rule.middle_button_hold_ms = 0;
        plan.scan_rule.left_turn_wait_ms = 0;
        plan.alignment_rule.refresh_release_ms = 0;
        plan.alignment_rule.refresh_hold_ms = 0;
        plan.alignment_rule.aim_move_delay_ms = 0;
        plan.mine_rule.compensate_up_wait_ms = 0;
        plan.mine_rule.after_attack_wait_ms = 0;
        plan
    }

    #[test]
    fn linnea_mining_plan_is_executor_ready_without_native_pending_step() {
        let plan = plan_linnea_mining(LinneaMiningExecutionConfig::default());

        assert!(plan.executor_ready);
        assert!(!plan.steps.iter().any(|step| matches!(
            step.action,
            LinneaMiningStepAction::MarkNativePending { .. }
        )));
        assert!(plan.notes.contains("injectable Rust executor boundary"));
    }

    #[test]
    fn linnea_mining_detects_aligns_and_mines_target() {
        let plan = test_plan(1, 1);
        let mut runtime = FakeLinneaMiningRuntime::new([
            vec![ore_rect(1_080, 520, 80, 80)],
            vec![ore_rect(920, 500, 80, 80)],
        ]);

        let report = execute_linnea_mining_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(report.status, LinneaMiningExecutionStatus::Completed);
        assert_eq!(report.state.mined_count, 1);
        assert_eq!(report.state.detections, 1);
        assert_eq!(report.state.alignment_successes, 1);
        assert_eq!(runtime.event_count("attack:"), 1);
        assert!(runtime
            .events
            .iter()
            .any(|event| event == "draw_target:true"));
        assert!(report.actions.iter().any(|action| {
            action.action_kind == LinneaMiningRuntimeActionKind::MineAttack
                && action.outcome == LinneaMiningRuntimeOutcome::Aligned
        }));
        assert_cleanup_completed(&report, &runtime);
    }

    #[test]
    fn linnea_mining_turns_left_for_each_missing_detection_before_final_round() {
        let plan = test_plan(3, 3);
        let mut runtime = FakeLinneaMiningRuntime::new([vec![], vec![], vec![]]);

        let report = execute_linnea_mining_plan(&plan, &mut runtime).unwrap();

        assert!(!report.completed);
        assert_eq!(report.status, LinneaMiningExecutionStatus::Partial);
        assert_eq!(report.state.mined_count, 0);
        assert_eq!(report.state.missing_detections, 3);
        assert_eq!(report.state.left_turns, 2);
        assert_eq!(runtime.event_count("turn_left:"), 2);
        assert_eq!(runtime.event_count("attack:"), 0);
        assert_eq!(
            report
                .decisions
                .iter()
                .filter(|decision| {
                    decision.kind == LinneaMiningDecisionKind::TurnLeftAndContinue
                })
                .count(),
            2
        );
        assert_cleanup_completed(&report, &runtime);
    }

    #[test]
    fn linnea_mining_alignment_failure_falls_back_and_cleans_up() {
        let plan = test_plan(1, 1);
        let mut runtime =
            FakeLinneaMiningRuntime::new([vec![ore_rect(1_080, 520, 80, 80)], vec![]]);

        let report = execute_linnea_mining_plan(&plan, &mut runtime).unwrap();

        assert!(report.completed);
        assert_eq!(report.state.mined_count, 1);
        assert_eq!(report.state.fallback_attacks, 1);
        assert!(runtime.events.iter().any(|event| event == "move_aim:0,-25"));
        assert!(report.actions.iter().any(|action| {
            action.action_kind == LinneaMiningRuntimeActionKind::MineAttack
                && action.outcome == LinneaMiningRuntimeOutcome::Fallback
        }));
        assert!(report
            .decisions
            .iter()
            .any(|decision| decision.kind == LinneaMiningDecisionKind::FallbackMine));
        assert_cleanup_completed(&report, &runtime);
    }

    #[test]
    fn linnea_mining_cleanup_runs_when_execution_errors() {
        let plan = test_plan(1, 1);
        let mut runtime =
            FakeLinneaMiningRuntime::new([vec![ore_rect(1_080, 520, 80, 80)]]).fail_move_aim();

        let error = execute_linnea_mining_plan(&plan, &mut runtime).unwrap_err();

        assert!(matches!(error, TaskError::CommonJobExecution(_)));
        assert_eq!(runtime.event_count("attack:"), 0);
        assert!(runtime.events.iter().any(|event| event == "leave_aiming"));
        assert!(runtime.events.iter().any(|event| event == "middle_up"));
        assert!(runtime.events.iter().any(|event| event == "clear_overlay"));
    }

    #[test]
    fn linnea_mining_cleanup_runs_when_attack_errors() {
        let plan = test_plan(1, 1);
        let mut runtime =
            FakeLinneaMiningRuntime::new([vec![ore_rect(920, 500, 80, 80)]]).fail_attack();

        let error = execute_linnea_mining_plan(&plan, &mut runtime).unwrap_err();

        assert!(matches!(error, TaskError::CommonJobExecution(_)));
        assert!(runtime
            .events
            .iter()
            .any(|event| event == "attack:LeftMouse"));
        assert!(runtime.events.iter().any(|event| event == "leave_aiming"));
        assert!(runtime.events.iter().any(|event| event == "middle_up"));
        assert!(runtime.events.iter().any(|event| event == "clear_overlay"));
    }

    fn assert_cleanup_completed(
        report: &LinneaMiningExecutionReport,
        runtime: &FakeLinneaMiningRuntime,
    ) {
        assert!(report.state.cleanup_leave_aiming_attempted);
        assert!(report.state.cleanup_middle_button_released);
        assert!(report.state.cleanup_drawings_cleared);
        assert_eq!(report.cleanup_actions.len(), 3);
        assert!(runtime.events.iter().any(|event| event == "leave_aiming"));
        assert!(runtime.events.iter().any(|event| event == "middle_up"));
        assert!(runtime.events.iter().any(|event| event == "clear_overlay"));
    }
}
