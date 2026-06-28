use crate::Result;
use bgi_core::AutoSkipConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUTO_TRACK_TASK_KEY: &str = "AutoTrack";
pub const AUTO_TRACK_DISPLAY_NAME: &str = "自动追踪";
pub const AUTO_TRACK_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_TRACK_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_TRACK_LONG_DISTANCE_THRESHOLD_METERS: i32 = 150;
pub const AUTO_TRACK_ARRIVAL_DISTANCE_METERS: i32 = 3;
pub const AUTO_TRACK_MAX_STEERING_ITERATIONS: u64 = 2_400;
pub const AUTO_TRACK_PAIMON_MENU_ASSET: &str = "Common/Element:paimon_menu.png";
pub const AUTO_TRACK_BLUE_TRACK_POINT_ASSET: &str = "Common/Element:blue_track_point_28x.png";
pub const AUTO_TRACK_TELEPORT_ASSETS: &[&str] = &[
    "QuickTeleport:TeleportWaypoint.png",
    "QuickTeleport:StatueOfTheSeven.png",
    "QuickTeleport:Domain.png",
    "QuickTeleport:Domain2.png",
    "QuickTeleport:ObsidianTotemPole.png",
    "QuickTeleport:PortableWaypoint.png",
    "QuickTeleport:Mansion.png",
    "QuickTeleport:SubSpaceWaypoint.png",
    "QuickTeleport:NodKraiMeetingPoint.png",
    "QuickTeleport:TabletOfTona.png",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub config_rule: AutoTrackConfigRule,
    pub startup_rule: AutoTrackStartupRule,
    pub main_ui_rule: AutoTrackMainUiRule,
    pub mission_text_rule: AutoTrackMissionTextRule,
    pub teleport_rule: AutoTrackTeleportRule,
    pub tracking_rule: AutoTrackBluePointTrackingRule,
    pub locators: AutoTrackLocators,
    pub steps: Vec<AutoTrackStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub auto_skip_config: AutoSkipConfig,
}

impl Default for AutoTrackExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_TRACK_DEFAULT_CAPTURE_WIDTH,
                AUTO_TRACK_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            auto_skip_config: AutoSkipConfig::default(),
        }
    }
}

impl AutoTrackExecutionConfig {
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
        }

        let auto_skip_value = value
            .get("autoSkipConfig")
            .or_else(|| value.get("AutoSkipConfig"))
            .or_else(|| value.get("auto_skip_config"))
            .unwrap_or(value);
        config.auto_skip_config =
            serde_json::from_value(auto_skip_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackConfigRule {
    pub coupled_to_auto_skip_config_section: bool,
    pub auto_skip_enabled_value_is_not_checked_by_legacy_task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackStartupRule {
    pub uses_task_semaphore_non_blocking: bool,
    pub activates_game_window: bool,
    pub uses_global_cancellation_context: bool,
    pub normal_end_exception_is_logged: bool,
    pub clears_draw_content_on_finish: bool,
    pub releases_task_semaphore_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackMainUiRule {
    pub requires_paimon_menu_template: bool,
    pub not_in_main_ui_sleep_ms: u64,
    pub moves_mouse_down_before_ocr: bool,
    pub mouse_move_y_before_ocr: i32,
    pub waits_for_mission_text_animation_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackMissionTextRule {
    pub ocr_roi_from_paimon_menu: AutoTrackRelativeRectRule,
    pub distance_text_max_len: usize,
    pub distance_text_contains: String,
    pub missing_text_sleep_ms: u64,
    pub distance_not_found_sentinel: i32,
    pub long_distance_threshold_meters: i32,
    pub mission_distance_rect_saved_for_arrival_ocr: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackRelativeRectRule {
    pub x_from_source_left: i32,
    pub y_from_source_top: i32,
    pub width_1080p: i32,
    pub height_1080p: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackTeleportRule {
    pub opens_quest_menu_action: String,
    pub open_quest_menu_sleep_ms: u64,
    pub track_toggle_button_rule: String,
    pub track_toggle_first_second_sleep_ms: u64,
    pub track_toggle_after_second_sleep_ms: u64,
    pub map_choose_icon_assets: Vec<AutoTrackTemplateLocator>,
    pub matches_map_choose_icons_on_full_grayscale_capture: bool,
    pub deduplicates_matches_by_painting_matched_area: bool,
    pub chooses_nearest_teleport_to_screen_center: bool,
    pub nearest_teleport_distance_uses_match_top_left: bool,
    pub post_teleport_click_sleep_ms: u64,
    pub big_map_still_open_is_warning: bool,
    pub post_leave_big_map_sleep_ms: u64,
    pub wait_main_ui_retry_attempts: u64,
    pub wait_main_ui_retry_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackBluePointTrackingRule {
    pub quest_navigation_action: String,
    pub post_quest_navigation_sleep_ms: u64,
    pub blue_point_missing_first_time_is_warning: bool,
    pub keep_top_down_mouse_move_y: i32,
    pub loop_sleep_ms: u64,
    pub force_above_center_move_x: i32,
    pub releases_forward_when_forcing_above: bool,
    pub rotation_divisor: i32,
    pub minimum_abs_rotation: i32,
    pub start_forward_when_rotation_zero_or_direction_crosses: bool,
    pub arrival_abs_rotation_less_than: i32,
    pub arrival_abs_y_from_center_less_than: i32,
    pub arrival_distance_meters: i32,
    pub arrival_ocr_uses_saved_mission_distance_rect: bool,
    pub arrival_distance_ocr_only_affects_log: bool,
    pub releases_forward_on_arrival: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackLocators {
    pub paimon_menu: AutoTrackTemplateLocator,
    pub blue_track_point: AutoTrackTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoTrackTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub roi_rule: Option<String>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoTrackStep {
    pub phase: AutoTrackPhase,
    pub action: AutoTrackAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackPhase {
    Startup,
    MainUi,
    MissionText,
    Teleport,
    Tracking,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackAction {
    AcquireSemaphore,
    ActivateWindow,
    CheckPaimonMenu,
    MoveMouseAndWaitForMissionText,
    OcrMissionText,
    ParseMissionDistance,
    OpenQuestMenuAndToggleTrack,
    SelectNearestTeleport,
    WaitForMainUi,
    PressQuestNavigation,
    SteerBlueTrackPoint,
    ReleaseForwardAndFinish,
    ClearOverlayAndReleaseSemaphore,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackTemplateMatch {
    pub name: String,
    pub asset: String,
    pub rect: Rect,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackMainUiObservation {
    pub paimon_menu: Option<AutoTrackTemplateMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoTrackMissionTextObservation {
    pub roi: Rect,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackTeleportCandidate {
    pub asset: String,
    pub rect: Rect,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackTeleportObservation {
    pub candidates: Vec<AutoTrackTeleportCandidate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackTrackingObservation {
    pub blue_track_point: Option<AutoTrackTemplateMatch>,
    pub mission_distance_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoTrackObservation {
    MainUi(AutoTrackMainUiObservation),
    MissionText(AutoTrackMissionTextObservation),
    TeleportCandidates(AutoTrackTeleportObservation),
    BigMapOpen { detected: bool },
    Tracking(AutoTrackTrackingObservation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackActionPress {
    KeyDown,
    KeyUp,
    KeyPress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoTrackRuntimeAction {
    AcquireSemaphore,
    ActivateWindow,
    MouseMove {
        x: i32,
        y: i32,
    },
    Delay {
        duration_ms: u64,
    },
    OpenQuestMenu {
        action: String,
    },
    ClickTrackToggle {
        x: i32,
        y: i32,
        ordinal: u8,
    },
    ClickTeleportCandidate {
        candidate: AutoTrackTeleportCandidate,
    },
    PressQuestNavigation {
        action: String,
    },
    ForwardKey {
        press: AutoTrackActionPress,
    },
    ClearOverlay,
    ReleaseSemaphore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackActionStatus {
    Executed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackRuntimeActionReport {
    pub phase: AutoTrackPhase,
    pub action: AutoTrackRuntimeAction,
    pub status: AutoTrackActionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackForwardCommand {
    None,
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoTrackSteeringDecision {
    pub iteration: u64,
    pub blue_point_rect: Option<Rect>,
    pub x_from_center: i32,
    pub y_from_center: i32,
    pub rotation: i32,
    pub distance_meters: Option<i32>,
    pub action: AutoTrackSteeringAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoTrackSteeringAction {
    MissingBluePoint,
    ForceAboveCenter {
        mouse_move_x: i32,
        mouse_move_y: i32,
        forward: AutoTrackForwardCommand,
    },
    Steer {
        mouse_move_x: i32,
        mouse_move_y: i32,
        forward: AutoTrackForwardCommand,
    },
    Arrived {
        release_forward: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoTrackDecision {
    Semaphore {
        acquired: bool,
    },
    MainUi {
        detected: bool,
    },
    MissionDistance {
        text: Option<String>,
        distance_meters: Option<i32>,
        is_long_distance: bool,
    },
    TeleportBranch {
        required: bool,
        distance_meters: Option<i32>,
    },
    SelectTeleport {
        selected: Option<AutoTrackTeleportCandidate>,
        candidates_considered: usize,
    },
    WaitMainUi {
        detected: bool,
        attempts: u64,
    },
    Tracking(AutoTrackSteeringDecision),
    Abort {
        status: AutoTrackExecutionStatus,
    },
    Cleanup {
        released_forward: bool,
        cleared_overlay: bool,
        released_semaphore: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackDecisionReport {
    pub phase: AutoTrackPhase,
    pub decision: AutoTrackDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrackExecutionStatus {
    Completed,
    SemaphoreUnavailable,
    MainUiMissing,
    MissionTextMissing,
    NoTeleportCandidate,
    WaitMainUiTimedOut,
    BluePointMissing,
    TrackingObservationEnded,
    MaxSteeringIterationsReached,
    Cancelled,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackExecutionState {
    pub semaphore_acquired: bool,
    pub semaphore_released: bool,
    pub window_activated: bool,
    pub main_ui_detected: bool,
    pub mission_distance_rect: Option<Rect>,
    pub mission_text: Option<String>,
    pub mission_distance_meters: Option<i32>,
    pub long_distance: bool,
    pub teleport_branch_entered: bool,
    pub selected_teleport: Option<AutoTrackTeleportCandidate>,
    pub waited_main_ui: bool,
    pub quest_navigation_pressed: bool,
    pub tracking_iterations: u64,
    pub forward_held: bool,
    pub previous_rotation: Option<i32>,
    pub arrived: bool,
    pub cancelled: bool,
    pub cleanup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoTrackExecutionReport {
    pub task_key: String,
    pub status: AutoTrackExecutionStatus,
    pub state: AutoTrackExecutionState,
    pub observations: Vec<AutoTrackObservation>,
    pub decisions: Vec<AutoTrackDecisionReport>,
    pub executed_actions: Vec<AutoTrackRuntimeActionReport>,
    pub warnings: Vec<String>,
}

pub trait AutoTrackRuntime {
    fn is_auto_track_cancelled(&mut self) -> Result<bool> {
        Ok(false)
    }

    fn try_acquire_auto_track_semaphore(&mut self) -> Result<bool>;

    fn release_auto_track_semaphore(&mut self) -> Result<()>;

    fn activate_auto_track_window(&mut self) -> Result<()>;

    fn observe_auto_track_main_ui(
        &mut self,
        plan: &AutoTrackExecutionPlan,
    ) -> Result<AutoTrackMainUiObservation>;

    fn ocr_auto_track_mission_text(
        &mut self,
        plan: &AutoTrackExecutionPlan,
        roi: Rect,
    ) -> Result<AutoTrackMissionTextObservation>;

    fn observe_auto_track_teleport_candidates(
        &mut self,
        plan: &AutoTrackExecutionPlan,
    ) -> Result<AutoTrackTeleportObservation>;

    fn is_auto_track_big_map_open(&mut self, plan: &AutoTrackExecutionPlan) -> Result<bool>;

    fn observe_auto_track_tracking(
        &mut self,
        plan: &AutoTrackExecutionPlan,
        state: &AutoTrackExecutionState,
    ) -> Result<Option<AutoTrackTrackingObservation>>;

    fn dispatch_auto_track_action(&mut self, action: &AutoTrackRuntimeAction) -> Result<()>;

    fn delay_auto_track(&mut self, duration_ms: u64) -> Result<()>;

    fn clear_auto_track_overlay(&mut self) -> Result<()>;
}

pub fn execute_auto_track_plan<R>(
    plan: &AutoTrackExecutionPlan,
    runtime: &mut R,
) -> Result<AutoTrackExecutionReport>
where
    R: AutoTrackRuntime,
{
    let mut context = AutoTrackExecutionContext::new(plan);
    let status = match execute_auto_track_plan_inner(plan, runtime, &mut context) {
        Ok(status) => status,
        Err(error) => {
            cleanup_auto_track(plan, runtime, &mut context)?;
            return Err(error);
        }
    };
    cleanup_auto_track(plan, runtime, &mut context)?;
    Ok(context.into_report(status))
}

struct AutoTrackExecutionContext<'a> {
    plan: &'a AutoTrackExecutionPlan,
    state: AutoTrackExecutionState,
    observations: Vec<AutoTrackObservation>,
    decisions: Vec<AutoTrackDecisionReport>,
    executed_actions: Vec<AutoTrackRuntimeActionReport>,
    warnings: Vec<String>,
}

impl<'a> AutoTrackExecutionContext<'a> {
    fn new(plan: &'a AutoTrackExecutionPlan) -> Self {
        Self {
            plan,
            state: AutoTrackExecutionState::default(),
            observations: Vec::new(),
            decisions: Vec::new(),
            executed_actions: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn decision(&mut self, phase: AutoTrackPhase, decision: AutoTrackDecision) {
        self.decisions
            .push(AutoTrackDecisionReport { phase, decision });
    }

    fn action(
        &mut self,
        phase: AutoTrackPhase,
        action: AutoTrackRuntimeAction,
        status: AutoTrackActionStatus,
    ) {
        self.executed_actions.push(AutoTrackRuntimeActionReport {
            phase,
            action,
            status,
        });
    }

    fn into_report(self, status: AutoTrackExecutionStatus) -> AutoTrackExecutionReport {
        AutoTrackExecutionReport {
            task_key: self.plan.task_key.clone(),
            status,
            state: self.state,
            observations: self.observations,
            decisions: self.decisions,
            executed_actions: self.executed_actions,
            warnings: self.warnings,
        }
    }
}

fn execute_auto_track_plan_inner<R>(
    plan: &AutoTrackExecutionPlan,
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
) -> Result<AutoTrackExecutionStatus>
where
    R: AutoTrackRuntime,
{
    if check_auto_track_cancelled(runtime, context, AutoTrackPhase::Startup)? {
        return Ok(AutoTrackExecutionStatus::Cancelled);
    }

    let acquired = runtime.try_acquire_auto_track_semaphore()?;
    context.state.semaphore_acquired = acquired;
    context.decision(
        AutoTrackPhase::Startup,
        AutoTrackDecision::Semaphore { acquired },
    );
    if acquired {
        context.action(
            AutoTrackPhase::Startup,
            AutoTrackRuntimeAction::AcquireSemaphore,
            AutoTrackActionStatus::Executed,
        );
    } else {
        context.action(
            AutoTrackPhase::Startup,
            AutoTrackRuntimeAction::AcquireSemaphore,
            AutoTrackActionStatus::Skipped,
        );
        context.decision(
            AutoTrackPhase::Startup,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::SemaphoreUnavailable,
            },
        );
        return Ok(AutoTrackExecutionStatus::SemaphoreUnavailable);
    }

    runtime.activate_auto_track_window()?;
    context.state.window_activated = true;
    context.action(
        AutoTrackPhase::Startup,
        AutoTrackRuntimeAction::ActivateWindow,
        AutoTrackActionStatus::Executed,
    );

    if check_auto_track_cancelled(runtime, context, AutoTrackPhase::MainUi)? {
        return Ok(AutoTrackExecutionStatus::Cancelled);
    }

    let main_ui = runtime.observe_auto_track_main_ui(plan)?;
    let main_ui_detected = main_ui.paimon_menu.is_some();
    context.state.main_ui_detected = main_ui_detected;
    context
        .observations
        .push(AutoTrackObservation::MainUi(main_ui.clone()));
    context.decision(
        AutoTrackPhase::MainUi,
        AutoTrackDecision::MainUi {
            detected: main_ui_detected,
        },
    );
    if !main_ui_detected {
        context.decision(
            AutoTrackPhase::MainUi,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::MainUiMissing,
            },
        );
        runtime.delay_auto_track(plan.main_ui_rule.not_in_main_ui_sleep_ms)?;
        context.action(
            AutoTrackPhase::MainUi,
            AutoTrackRuntimeAction::Delay {
                duration_ms: plan.main_ui_rule.not_in_main_ui_sleep_ms,
            },
            AutoTrackActionStatus::Executed,
        );
        return Ok(AutoTrackExecutionStatus::MainUiMissing);
    }

    let mission_distance_rect = mission_distance_rect(plan, main_ui.paimon_menu.as_ref());
    context.state.mission_distance_rect = Some(mission_distance_rect);
    runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::MouseMove {
        x: 0,
        y: plan.main_ui_rule.mouse_move_y_before_ocr,
    })?;
    context.action(
        AutoTrackPhase::MissionText,
        AutoTrackRuntimeAction::MouseMove {
            x: 0,
            y: plan.main_ui_rule.mouse_move_y_before_ocr,
        },
        AutoTrackActionStatus::Executed,
    );
    runtime.delay_auto_track(plan.main_ui_rule.waits_for_mission_text_animation_ms)?;
    context.action(
        AutoTrackPhase::MissionText,
        AutoTrackRuntimeAction::Delay {
            duration_ms: plan.main_ui_rule.waits_for_mission_text_animation_ms,
        },
        AutoTrackActionStatus::Executed,
    );

    if check_auto_track_cancelled(runtime, context, AutoTrackPhase::MissionText)? {
        return Ok(AutoTrackExecutionStatus::Cancelled);
    }

    let mission_text = runtime.ocr_auto_track_mission_text(plan, mission_distance_rect)?;
    context.state.mission_text = mission_text.text.clone();
    context
        .observations
        .push(AutoTrackObservation::MissionText(mission_text.clone()));
    let distance_meters = mission_text
        .text
        .as_deref()
        .and_then(parse_auto_track_distance_meters);
    context.state.mission_distance_meters = distance_meters;
    let long_distance = distance_meters
        .map(|distance| distance > plan.mission_text_rule.long_distance_threshold_meters)
        .unwrap_or(false);
    context.state.long_distance = long_distance;
    context.decision(
        AutoTrackPhase::MissionText,
        AutoTrackDecision::MissionDistance {
            text: mission_text.text.clone(),
            distance_meters,
            is_long_distance: long_distance,
        },
    );

    if mission_text
        .text
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        runtime.delay_auto_track(plan.mission_text_rule.missing_text_sleep_ms)?;
        context.action(
            AutoTrackPhase::MissionText,
            AutoTrackRuntimeAction::Delay {
                duration_ms: plan.mission_text_rule.missing_text_sleep_ms,
            },
            AutoTrackActionStatus::Executed,
        );
        context.decision(
            AutoTrackPhase::MissionText,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::MissionTextMissing,
            },
        );
        return Ok(AutoTrackExecutionStatus::MissionTextMissing);
    }

    context.decision(
        AutoTrackPhase::Teleport,
        AutoTrackDecision::TeleportBranch {
            required: long_distance,
            distance_meters,
        },
    );
    if long_distance {
        let status = execute_auto_track_teleport_branch(plan, runtime, context)?;
        if status != AutoTrackExecutionStatus::Completed {
            return Ok(status);
        }
    }

    if check_auto_track_cancelled(runtime, context, AutoTrackPhase::Tracking)? {
        return Ok(AutoTrackExecutionStatus::Cancelled);
    }

    runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::PressQuestNavigation {
        action: plan.tracking_rule.quest_navigation_action.clone(),
    })?;
    context.state.quest_navigation_pressed = true;
    context.action(
        AutoTrackPhase::Tracking,
        AutoTrackRuntimeAction::PressQuestNavigation {
            action: plan.tracking_rule.quest_navigation_action.clone(),
        },
        AutoTrackActionStatus::Executed,
    );
    runtime.delay_auto_track(plan.tracking_rule.post_quest_navigation_sleep_ms)?;
    context.action(
        AutoTrackPhase::Tracking,
        AutoTrackRuntimeAction::Delay {
            duration_ms: plan.tracking_rule.post_quest_navigation_sleep_ms,
        },
        AutoTrackActionStatus::Executed,
    );

    execute_auto_track_steering_loop(plan, runtime, context)
}

fn execute_auto_track_teleport_branch<R>(
    plan: &AutoTrackExecutionPlan,
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
) -> Result<AutoTrackExecutionStatus>
where
    R: AutoTrackRuntime,
{
    context.state.teleport_branch_entered = true;
    runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::OpenQuestMenu {
        action: plan.teleport_rule.opens_quest_menu_action.clone(),
    })?;
    context.action(
        AutoTrackPhase::Teleport,
        AutoTrackRuntimeAction::OpenQuestMenu {
            action: plan.teleport_rule.opens_quest_menu_action.clone(),
        },
        AutoTrackActionStatus::Executed,
    );
    runtime.delay_auto_track(plan.teleport_rule.open_quest_menu_sleep_ms)?;
    context.action(
        AutoTrackPhase::Teleport,
        AutoTrackRuntimeAction::Delay {
            duration_ms: plan.teleport_rule.open_quest_menu_sleep_ms,
        },
        AutoTrackActionStatus::Executed,
    );

    for ordinal in 1..=2 {
        let (x, y) = track_toggle_point(plan);
        runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::ClickTrackToggle {
            x,
            y,
            ordinal,
        })?;
        context.action(
            AutoTrackPhase::Teleport,
            AutoTrackRuntimeAction::ClickTrackToggle { x, y, ordinal },
            AutoTrackActionStatus::Executed,
        );
        let duration_ms = if ordinal == 1 {
            plan.teleport_rule.track_toggle_first_second_sleep_ms
        } else {
            plan.teleport_rule.track_toggle_after_second_sleep_ms
        };
        runtime.delay_auto_track(duration_ms)?;
        context.action(
            AutoTrackPhase::Teleport,
            AutoTrackRuntimeAction::Delay { duration_ms },
            AutoTrackActionStatus::Executed,
        );
    }

    if check_auto_track_cancelled(runtime, context, AutoTrackPhase::Teleport)? {
        return Ok(AutoTrackExecutionStatus::Cancelled);
    }

    let teleport_observation = runtime.observe_auto_track_teleport_candidates(plan)?;
    context
        .observations
        .push(AutoTrackObservation::TeleportCandidates(
            teleport_observation.clone(),
        ));
    let selected = nearest_auto_track_teleport_candidate(plan, &teleport_observation.candidates);
    context.state.selected_teleport = selected.cloned();
    context.decision(
        AutoTrackPhase::Teleport,
        AutoTrackDecision::SelectTeleport {
            selected: selected.cloned(),
            candidates_considered: teleport_observation.candidates.len(),
        },
    );
    let Some(selected) = selected else {
        context.decision(
            AutoTrackPhase::Teleport,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::NoTeleportCandidate,
            },
        );
        return Ok(AutoTrackExecutionStatus::NoTeleportCandidate);
    };

    runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::ClickTeleportCandidate {
        candidate: selected.clone(),
    })?;
    context.action(
        AutoTrackPhase::Teleport,
        AutoTrackRuntimeAction::ClickTeleportCandidate {
            candidate: selected.clone(),
        },
        AutoTrackActionStatus::Executed,
    );
    runtime.delay_auto_track(plan.teleport_rule.post_teleport_click_sleep_ms)?;
    context.action(
        AutoTrackPhase::Teleport,
        AutoTrackRuntimeAction::Delay {
            duration_ms: plan.teleport_rule.post_teleport_click_sleep_ms,
        },
        AutoTrackActionStatus::Executed,
    );

    let big_map_open = runtime.is_auto_track_big_map_open(plan)?;
    context.observations.push(AutoTrackObservation::BigMapOpen {
        detected: big_map_open,
    });
    if big_map_open && plan.teleport_rule.big_map_still_open_is_warning {
        context
            .warnings
            .push("big map is still open after teleport candidate click".to_string());
    }
    runtime.delay_auto_track(plan.teleport_rule.post_leave_big_map_sleep_ms)?;
    context.action(
        AutoTrackPhase::Teleport,
        AutoTrackRuntimeAction::Delay {
            duration_ms: plan.teleport_rule.post_leave_big_map_sleep_ms,
        },
        AutoTrackActionStatus::Executed,
    );

    let mut detected = false;
    let mut attempts = 0;
    for attempt in 1..=plan.teleport_rule.wait_main_ui_retry_attempts {
        if check_auto_track_cancelled(runtime, context, AutoTrackPhase::Teleport)? {
            return Ok(AutoTrackExecutionStatus::Cancelled);
        }
        attempts = attempt;
        let observation = runtime.observe_auto_track_main_ui(plan)?;
        detected = observation.paimon_menu.is_some();
        context
            .observations
            .push(AutoTrackObservation::MainUi(observation));
        if detected {
            break;
        }
        runtime.delay_auto_track(plan.teleport_rule.wait_main_ui_retry_interval_ms)?;
        context.action(
            AutoTrackPhase::Teleport,
            AutoTrackRuntimeAction::Delay {
                duration_ms: plan.teleport_rule.wait_main_ui_retry_interval_ms,
            },
            AutoTrackActionStatus::Executed,
        );
    }
    context.state.waited_main_ui = true;
    context.decision(
        AutoTrackPhase::Teleport,
        AutoTrackDecision::WaitMainUi { detected, attempts },
    );

    if detected {
        Ok(AutoTrackExecutionStatus::Completed)
    } else {
        context.decision(
            AutoTrackPhase::Teleport,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::WaitMainUiTimedOut,
            },
        );
        Ok(AutoTrackExecutionStatus::WaitMainUiTimedOut)
    }
}

fn execute_auto_track_steering_loop<R>(
    plan: &AutoTrackExecutionPlan,
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
) -> Result<AutoTrackExecutionStatus>
where
    R: AutoTrackRuntime,
{
    loop {
        if context.state.tracking_iterations >= AUTO_TRACK_MAX_STEERING_ITERATIONS {
            context.decision(
                AutoTrackPhase::Tracking,
                AutoTrackDecision::Abort {
                    status: AutoTrackExecutionStatus::MaxSteeringIterationsReached,
                },
            );
            return Ok(AutoTrackExecutionStatus::MaxSteeringIterationsReached);
        }
        if check_auto_track_cancelled(runtime, context, AutoTrackPhase::Tracking)? {
            return Ok(AutoTrackExecutionStatus::Cancelled);
        }

        let Some(observation) = runtime.observe_auto_track_tracking(plan, &context.state)? else {
            context.decision(
                AutoTrackPhase::Tracking,
                AutoTrackDecision::Abort {
                    status: AutoTrackExecutionStatus::TrackingObservationEnded,
                },
            );
            return Ok(AutoTrackExecutionStatus::TrackingObservationEnded);
        };
        context
            .observations
            .push(AutoTrackObservation::Tracking(observation.clone()));
        let decision = decide_auto_track_steering(plan, &context.state, &observation);
        context.state.tracking_iterations += 1;

        match &decision.action {
            AutoTrackSteeringAction::MissingBluePoint => {
                if plan.tracking_rule.blue_point_missing_first_time_is_warning {
                    context.warnings.push(
                        "blue track point was missing on first tracking observation".to_string(),
                    );
                }
                context.decision(
                    AutoTrackPhase::Tracking,
                    AutoTrackDecision::Tracking(decision),
                );
                return Ok(AutoTrackExecutionStatus::BluePointMissing);
            }
            AutoTrackSteeringAction::ForceAboveCenter {
                mouse_move_x,
                mouse_move_y,
                forward,
            }
            | AutoTrackSteeringAction::Steer {
                mouse_move_x,
                mouse_move_y,
                forward,
            } => {
                dispatch_auto_track_forward_command(runtime, context, *forward)?;
                runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::MouseMove {
                    x: *mouse_move_x,
                    y: *mouse_move_y,
                })?;
                context.action(
                    AutoTrackPhase::Tracking,
                    AutoTrackRuntimeAction::MouseMove {
                        x: *mouse_move_x,
                        y: *mouse_move_y,
                    },
                    AutoTrackActionStatus::Executed,
                );
                if decision.rotation != 0 {
                    context.state.previous_rotation = Some(decision.rotation);
                }
                context.decision(
                    AutoTrackPhase::Tracking,
                    AutoTrackDecision::Tracking(decision),
                );
                runtime.delay_auto_track(plan.tracking_rule.loop_sleep_ms)?;
                context.action(
                    AutoTrackPhase::Tracking,
                    AutoTrackRuntimeAction::Delay {
                        duration_ms: plan.tracking_rule.loop_sleep_ms,
                    },
                    AutoTrackActionStatus::Executed,
                );
            }
            AutoTrackSteeringAction::Arrived { release_forward } => {
                if *release_forward {
                    dispatch_auto_track_forward_command(
                        runtime,
                        context,
                        AutoTrackForwardCommand::KeyUp,
                    )?;
                }
                context.state.arrived = true;
                context.decision(
                    AutoTrackPhase::Tracking,
                    AutoTrackDecision::Tracking(decision),
                );
                return Ok(AutoTrackExecutionStatus::Completed);
            }
        }
    }
}

fn cleanup_auto_track<R>(
    _plan: &AutoTrackExecutionPlan,
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
) -> Result<()>
where
    R: AutoTrackRuntime,
{
    let mut released_forward = false;
    if context.state.forward_held {
        runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::ForwardKey {
            press: AutoTrackActionPress::KeyUp,
        })?;
        context.state.forward_held = false;
        released_forward = true;
        context.action(
            AutoTrackPhase::Cleanup,
            AutoTrackRuntimeAction::ForwardKey {
                press: AutoTrackActionPress::KeyUp,
            },
            AutoTrackActionStatus::Executed,
        );
    }

    runtime.clear_auto_track_overlay()?;
    context.action(
        AutoTrackPhase::Cleanup,
        AutoTrackRuntimeAction::ClearOverlay,
        AutoTrackActionStatus::Executed,
    );

    let mut released_semaphore = false;
    if context.state.semaphore_acquired && !context.state.semaphore_released {
        runtime.release_auto_track_semaphore()?;
        context.state.semaphore_released = true;
        released_semaphore = true;
        context.action(
            AutoTrackPhase::Cleanup,
            AutoTrackRuntimeAction::ReleaseSemaphore,
            AutoTrackActionStatus::Executed,
        );
    }
    context.state.cleanup_completed = true;
    context.decision(
        AutoTrackPhase::Cleanup,
        AutoTrackDecision::Cleanup {
            released_forward,
            cleared_overlay: true,
            released_semaphore,
        },
    );
    Ok(())
}

fn check_auto_track_cancelled<R>(
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
    phase: AutoTrackPhase,
) -> Result<bool>
where
    R: AutoTrackRuntime,
{
    if runtime.is_auto_track_cancelled()? {
        context.state.cancelled = true;
        context.decision(
            phase,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::Cancelled,
            },
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

fn dispatch_auto_track_forward_command<R>(
    runtime: &mut R,
    context: &mut AutoTrackExecutionContext<'_>,
    command: AutoTrackForwardCommand,
) -> Result<()>
where
    R: AutoTrackRuntime,
{
    match command {
        AutoTrackForwardCommand::None => Ok(()),
        AutoTrackForwardCommand::KeyDown if context.state.forward_held => Ok(()),
        AutoTrackForwardCommand::KeyUp if !context.state.forward_held => Ok(()),
        AutoTrackForwardCommand::KeyDown => {
            runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::ForwardKey {
                press: AutoTrackActionPress::KeyDown,
            })?;
            context.state.forward_held = true;
            context.action(
                AutoTrackPhase::Tracking,
                AutoTrackRuntimeAction::ForwardKey {
                    press: AutoTrackActionPress::KeyDown,
                },
                AutoTrackActionStatus::Executed,
            );
            Ok(())
        }
        AutoTrackForwardCommand::KeyUp => {
            runtime.dispatch_auto_track_action(&AutoTrackRuntimeAction::ForwardKey {
                press: AutoTrackActionPress::KeyUp,
            })?;
            context.state.forward_held = false;
            context.action(
                AutoTrackPhase::Tracking,
                AutoTrackRuntimeAction::ForwardKey {
                    press: AutoTrackActionPress::KeyUp,
                },
                AutoTrackActionStatus::Executed,
            );
            Ok(())
        }
    }
}

pub fn parse_auto_track_distance_meters(text: &str) -> Option<i32> {
    let compact = text
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != ',')
        .collect::<String>();
    let m_index = compact.find(['m', 'M'])?;
    let number_end = m_index;
    let number_start = compact[..number_end]
        .char_indices()
        .rev()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(index, _)| index)?;
    compact[number_start..number_end].parse().ok()
}

pub fn decide_auto_track_steering(
    plan: &AutoTrackExecutionPlan,
    state: &AutoTrackExecutionState,
    observation: &AutoTrackTrackingObservation,
) -> AutoTrackSteeringDecision {
    let iteration = state.tracking_iterations.saturating_add(1);
    let distance_meters = observation
        .mission_distance_text
        .as_deref()
        .and_then(parse_auto_track_distance_meters);
    let Some(blue_point) = &observation.blue_track_point else {
        return AutoTrackSteeringDecision {
            iteration,
            blue_point_rect: None,
            x_from_center: 0,
            y_from_center: 0,
            rotation: 0,
            distance_meters,
            action: AutoTrackSteeringAction::MissingBluePoint,
        };
    };

    let center = blue_point.rect.center();
    let screen_center_x = plan.capture_size.width as i32 / 2;
    let screen_center_y = plan.capture_size.height as i32 / 2;
    let x_from_center = center.x - screen_center_x;
    let y_from_center = center.y - screen_center_y;
    let rotation = auto_track_rotation(x_from_center, &plan.tracking_rule);
    let position_arrived = rotation.abs() < plan.tracking_rule.arrival_abs_rotation_less_than
        && y_from_center.abs() < plan.tracking_rule.arrival_abs_y_from_center_less_than;
    let distance_arrived = !plan.tracking_rule.arrival_distance_ocr_only_affects_log
        && distance_meters
            .map(|distance| distance <= plan.tracking_rule.arrival_distance_meters)
            .unwrap_or(false);

    if position_arrived || distance_arrived {
        return AutoTrackSteeringDecision {
            iteration,
            blue_point_rect: Some(blue_point.rect),
            x_from_center,
            y_from_center,
            rotation,
            distance_meters,
            action: AutoTrackSteeringAction::Arrived {
                release_forward: plan.tracking_rule.releases_forward_on_arrival,
            },
        };
    }

    if y_from_center < 0 {
        return AutoTrackSteeringDecision {
            iteration,
            blue_point_rect: Some(blue_point.rect),
            x_from_center,
            y_from_center,
            rotation,
            distance_meters,
            action: AutoTrackSteeringAction::ForceAboveCenter {
                mouse_move_x: plan.tracking_rule.force_above_center_move_x,
                mouse_move_y: plan.tracking_rule.keep_top_down_mouse_move_y,
                forward: if plan.tracking_rule.releases_forward_when_forcing_above {
                    AutoTrackForwardCommand::KeyUp
                } else {
                    AutoTrackForwardCommand::None
                },
            },
        };
    }

    AutoTrackSteeringDecision {
        iteration,
        blue_point_rect: Some(blue_point.rect),
        x_from_center,
        y_from_center,
        rotation,
        distance_meters,
        action: AutoTrackSteeringAction::Steer {
            mouse_move_x: rotation,
            mouse_move_y: plan.tracking_rule.keep_top_down_mouse_move_y,
            forward: if should_auto_track_start_forward(
                rotation,
                state.previous_rotation,
                &plan.tracking_rule,
            ) {
                AutoTrackForwardCommand::KeyDown
            } else {
                AutoTrackForwardCommand::None
            },
        },
    }
}

fn mission_distance_rect(
    plan: &AutoTrackExecutionPlan,
    paimon_menu: Option<&AutoTrackTemplateMatch>,
) -> Rect {
    let source = paimon_menu.map(|matched| matched.rect).unwrap_or_default();
    let scale = plan.capture_size.height as f64 / AUTO_TRACK_DEFAULT_CAPTURE_HEIGHT as f64;
    let rule = &plan.mission_text_rule.ocr_roi_from_paimon_menu;
    Rect {
        x: source.x + scaled_auto_track_i32(rule.x_from_source_left, scale),
        y: source.y + scaled_auto_track_i32(rule.y_from_source_top, scale),
        width: scaled_auto_track_i32(rule.width_1080p, scale),
        height: scaled_auto_track_i32(rule.height_1080p, scale),
    }
}

fn track_toggle_point(plan: &AutoTrackExecutionPlan) -> (i32, i32) {
    (
        plan.capture_size.width as i32 - 250,
        plan.capture_size.height as i32 - 60,
    )
}

fn nearest_auto_track_teleport_candidate<'a>(
    plan: &AutoTrackExecutionPlan,
    candidates: &'a [AutoTrackTeleportCandidate],
) -> Option<&'a AutoTrackTeleportCandidate> {
    let center_x = plan.capture_size.width as i32 / 2;
    let center_y = plan.capture_size.height as i32 / 2;
    candidates.iter().min_by_key(|candidate| {
        let dx = candidate.rect.x - center_x;
        let dy = candidate.rect.y - center_y;
        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
    })
}

fn auto_track_rotation(x_from_center: i32, rule: &AutoTrackBluePointTrackingRule) -> i32 {
    let divisor = rule.rotation_divisor.max(1);
    let raw = x_from_center / divisor;
    if raw == 0 {
        return 0;
    }
    if raw.abs() < rule.minimum_abs_rotation {
        raw.signum() * rule.minimum_abs_rotation
    } else {
        raw
    }
}

fn should_auto_track_start_forward(
    rotation: i32,
    previous_rotation: Option<i32>,
    rule: &AutoTrackBluePointTrackingRule,
) -> bool {
    if !rule.start_forward_when_rotation_zero_or_direction_crosses {
        return false;
    }
    if rotation == 0 {
        return true;
    }
    previous_rotation
        .filter(|previous| *previous != 0)
        .map(|previous| previous.signum() != rotation.signum())
        .unwrap_or(false)
}

fn scaled_auto_track_i32(value: i32, scale: f64) -> i32 {
    (value as f64 * scale).round() as i32
}

pub fn plan_auto_track(config: AutoTrackExecutionConfig) -> AutoTrackExecutionPlan {
    AutoTrackExecutionPlan {
        task_key: AUTO_TRACK_TASK_KEY.to_string(),
        display_name: AUTO_TRACK_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        config_rule: AutoTrackConfigRule {
            coupled_to_auto_skip_config_section: true,
            auto_skip_enabled_value_is_not_checked_by_legacy_task: true,
        },
        startup_rule: AutoTrackStartupRule {
            uses_task_semaphore_non_blocking: true,
            activates_game_window: true,
            uses_global_cancellation_context: true,
            normal_end_exception_is_logged: true,
            clears_draw_content_on_finish: true,
            releases_task_semaphore_on_finish: true,
        },
        main_ui_rule: AutoTrackMainUiRule {
            requires_paimon_menu_template: true,
            not_in_main_ui_sleep_ms: 5_000,
            moves_mouse_down_before_ocr: true,
            mouse_move_y_before_ocr: 7_000,
            waits_for_mission_text_animation_ms: 2_000,
        },
        mission_text_rule: AutoTrackMissionTextRule {
            ocr_roi_from_paimon_menu: AutoTrackRelativeRectRule {
                x_from_source_left: 0,
                y_from_source_top: 195,
                width_1080p: 300,
                height_1080p: 100,
            },
            distance_text_max_len: 7,
            distance_text_contains: "m".to_string(),
            missing_text_sleep_ms: 5_000,
            distance_not_found_sentinel: -1,
            long_distance_threshold_meters: AUTO_TRACK_LONG_DISTANCE_THRESHOLD_METERS,
            mission_distance_rect_saved_for_arrival_ocr: true,
        },
        teleport_rule: AutoTrackTeleportRule {
            opens_quest_menu_action: "GIActions.OpenQuestMenu".to_string(),
            open_quest_menu_sleep_ms: 800,
            track_toggle_button_rule: "derive from bottom-right at capture_width - 250, capture_height - 60; click twice".to_string(),
            track_toggle_first_second_sleep_ms: 200,
            track_toggle_after_second_sleep_ms: 1_500,
            map_choose_icon_assets: map_choose_icon_locators(),
            matches_map_choose_icons_on_full_grayscale_capture: true,
            deduplicates_matches_by_painting_matched_area: true,
            chooses_nearest_teleport_to_screen_center: true,
            nearest_teleport_distance_uses_match_top_left: true,
            post_teleport_click_sleep_ms: 2_000,
            big_map_still_open_is_warning: true,
            post_leave_big_map_sleep_ms: 500,
            wait_main_ui_retry_attempts: 100,
            wait_main_ui_retry_interval_ms: 1_000,
        },
        tracking_rule: AutoTrackBluePointTrackingRule {
            quest_navigation_action: "GIActions.QuestNavigation".to_string(),
            post_quest_navigation_sleep_ms: 3_000,
            blue_point_missing_first_time_is_warning: true,
            keep_top_down_mouse_move_y: 500,
            loop_sleep_ms: 100,
            force_above_center_move_x: -50,
            releases_forward_when_forcing_above: true,
            rotation_divisor: 8,
            minimum_abs_rotation: 10,
            start_forward_when_rotation_zero_or_direction_crosses: true,
            arrival_abs_rotation_less_than: 50,
            arrival_abs_y_from_center_less_than: 200,
            arrival_distance_meters: AUTO_TRACK_ARRIVAL_DISTANCE_METERS,
            arrival_ocr_uses_saved_mission_distance_rect: true,
            arrival_distance_ocr_only_affects_log: true,
            releases_forward_on_arrival: true,
        },
        locators: AutoTrackLocators {
            paimon_menu: AutoTrackTemplateLocator {
                name: "PaimonMenu".to_string(),
                asset: AUTO_TRACK_PAIMON_MENU_ASSET.to_string(),
                roi: Some(Rect {
                    x: 0,
                    y: 0,
                    width: (config.capture_size.width / 4) as i32,
                    height: (config.capture_size.height / 4) as i32,
                }),
                roi_rule: None,
                threshold: 0.8,
                match_mode: TemplateMatchMode::CCoeffNormed,
                draw_on_window: false,
            },
            blue_track_point: AutoTrackTemplateLocator {
                name: "BlueTrackPoint".to_string(),
                asset: AUTO_TRACK_BLUE_TRACK_POINT_ASSET.to_string(),
                roi: Some(Rect {
                    x: (300.0 * config.asset_scale).round() as i32,
                    y: 0,
                    width: config.capture_size.width as i32
                        - (600.0 * config.asset_scale).round() as i32,
                    height: config.capture_size.height as i32,
                }),
                roi_rule: None,
                threshold: 0.6,
                match_mode: TemplateMatchMode::CCoeffNormed,
                draw_on_window: true,
            },
        },
        steps: auto_track_steps(),
        executor_ready: true,
        pending_native: vec![
            "injectable Rust AutoTrack executor boundary is ready for semaphore, cancellation, activation, main-ui detection, mission-distance OCR, teleport selection, quest navigation, steering, forward-release, and cleanup".to_string(),
            "desktop live adapters still need to connect real TaskSemaphore/CancellationContext/SystemControl, capture, template matching, OCR, BV map/main-ui detection, Simulation input, and overlay cleanup".to_string(),
            "real-game regression notes and final desktop task-runner routing remain pending after live adapters are wired".to_string(),
        ],
    }
}

fn map_choose_icon_locators() -> Vec<AutoTrackTemplateLocator> {
    AUTO_TRACK_TELEPORT_ASSETS
        .iter()
        .map(|asset| {
            let name = asset
                .split(':')
                .next_back()
                .unwrap_or(asset)
                .trim_end_matches(".png");
            AutoTrackTemplateLocator {
                name: format!("{name}MapChooseIcon"),
                asset: (*asset).to_string(),
                roi: None,
                roi_rule: Some("QuickTeleportAssets.MapChooseIconRoi: x=1270, y=100, width=50, height=capture_height-200".to_string()),
                threshold: 0.8,
                match_mode: TemplateMatchMode::CCoeffNormed,
                draw_on_window: false,
            }
        })
        .collect()
}

fn auto_track_steps() -> Vec<AutoTrackStep> {
    use AutoTrackAction::*;
    use AutoTrackPhase::*;
    vec![
        AutoTrackStep {
            phase: Startup,
            action: AcquireSemaphore,
        },
        AutoTrackStep {
            phase: Startup,
            action: ActivateWindow,
        },
        AutoTrackStep {
            phase: MainUi,
            action: CheckPaimonMenu,
        },
        AutoTrackStep {
            phase: MissionText,
            action: MoveMouseAndWaitForMissionText,
        },
        AutoTrackStep {
            phase: MissionText,
            action: OcrMissionText,
        },
        AutoTrackStep {
            phase: MissionText,
            action: ParseMissionDistance,
        },
        AutoTrackStep {
            phase: Teleport,
            action: OpenQuestMenuAndToggleTrack,
        },
        AutoTrackStep {
            phase: Teleport,
            action: SelectNearestTeleport,
        },
        AutoTrackStep {
            phase: Teleport,
            action: WaitForMainUi,
        },
        AutoTrackStep {
            phase: Tracking,
            action: PressQuestNavigation,
        },
        AutoTrackStep {
            phase: Tracking,
            action: SteerBlueTrackPoint,
        },
        AutoTrackStep {
            phase: Tracking,
            action: ReleaseForwardAndFinish,
        },
        AutoTrackStep {
            phase: Cleanup,
            action: ClearOverlayAndReleaseSemaphore,
        },
    ]
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Default)]
    struct FakeAutoTrackRuntime {
        semaphore_available: bool,
        main_ui_observations: VecDeque<AutoTrackMainUiObservation>,
        mission_texts: VecDeque<Option<String>>,
        teleport_observations: VecDeque<AutoTrackTeleportObservation>,
        big_map_open_results: VecDeque<bool>,
        tracking_observations: VecDeque<Option<AutoTrackTrackingObservation>>,
        dispatched_actions: Vec<AutoTrackRuntimeAction>,
        delays: Vec<u64>,
        acquire_calls: u64,
        release_calls: u64,
        activate_calls: u64,
        clear_overlay_calls: u64,
    }

    impl FakeAutoTrackRuntime {
        fn new() -> Self {
            Self {
                semaphore_available: true,
                ..Self::default()
            }
        }

        fn with_main_ui(mut self, observations: impl IntoIterator<Item = bool>) -> Self {
            self.main_ui_observations = observations
                .into_iter()
                .map(|detected| AutoTrackMainUiObservation {
                    paimon_menu: detected.then(paimon_match),
                })
                .collect();
            self
        }

        fn with_mission_texts(mut self, texts: impl IntoIterator<Item = &'static str>) -> Self {
            self.mission_texts = texts
                .into_iter()
                .map(|text| Some(text.to_string()))
                .collect();
            self
        }

        fn with_teleport_candidates(
            mut self,
            candidates: impl IntoIterator<Item = AutoTrackTeleportCandidate>,
        ) -> Self {
            self.teleport_observations
                .push_back(AutoTrackTeleportObservation {
                    candidates: candidates.into_iter().collect(),
                });
            self
        }

        fn with_big_map_open(mut self, results: impl IntoIterator<Item = bool>) -> Self {
            self.big_map_open_results = results.into_iter().collect();
            self
        }

        fn with_tracking(
            mut self,
            observations: impl IntoIterator<Item = AutoTrackTrackingObservation>,
        ) -> Self {
            self.tracking_observations = observations.into_iter().map(Some).collect();
            self
        }
    }

    impl AutoTrackRuntime for FakeAutoTrackRuntime {
        fn try_acquire_auto_track_semaphore(&mut self) -> Result<bool> {
            self.acquire_calls += 1;
            Ok(self.semaphore_available)
        }

        fn release_auto_track_semaphore(&mut self) -> Result<()> {
            self.release_calls += 1;
            Ok(())
        }

        fn activate_auto_track_window(&mut self) -> Result<()> {
            self.activate_calls += 1;
            Ok(())
        }

        fn observe_auto_track_main_ui(
            &mut self,
            _plan: &AutoTrackExecutionPlan,
        ) -> Result<AutoTrackMainUiObservation> {
            Ok(self
                .main_ui_observations
                .pop_front()
                .unwrap_or(AutoTrackMainUiObservation { paimon_menu: None }))
        }

        fn ocr_auto_track_mission_text(
            &mut self,
            _plan: &AutoTrackExecutionPlan,
            roi: Rect,
        ) -> Result<AutoTrackMissionTextObservation> {
            Ok(AutoTrackMissionTextObservation {
                roi,
                text: self.mission_texts.pop_front().unwrap_or(None),
            })
        }

        fn observe_auto_track_teleport_candidates(
            &mut self,
            _plan: &AutoTrackExecutionPlan,
        ) -> Result<AutoTrackTeleportObservation> {
            Ok(self
                .teleport_observations
                .pop_front()
                .unwrap_or(AutoTrackTeleportObservation {
                    candidates: Vec::new(),
                }))
        }

        fn is_auto_track_big_map_open(&mut self, _plan: &AutoTrackExecutionPlan) -> Result<bool> {
            Ok(self.big_map_open_results.pop_front().unwrap_or(false))
        }

        fn observe_auto_track_tracking(
            &mut self,
            _plan: &AutoTrackExecutionPlan,
            _state: &AutoTrackExecutionState,
        ) -> Result<Option<AutoTrackTrackingObservation>> {
            Ok(self.tracking_observations.pop_front().unwrap_or(None))
        }

        fn dispatch_auto_track_action(&mut self, action: &AutoTrackRuntimeAction) -> Result<()> {
            self.dispatched_actions.push(action.clone());
            Ok(())
        }

        fn delay_auto_track(&mut self, duration_ms: u64) -> Result<()> {
            self.delays.push(duration_ms);
            Ok(())
        }

        fn clear_auto_track_overlay(&mut self) -> Result<()> {
            self.clear_overlay_calls += 1;
            Ok(())
        }
    }

    #[test]
    fn auto_track_execute_near_distance_tracks_without_teleport() {
        let plan = plan_auto_track(AutoTrackExecutionConfig::default());
        let mut runtime = FakeAutoTrackRuntime::new()
            .with_main_ui([true])
            .with_mission_texts(["80m"])
            .with_tracking([tracking_observation_at(960, 600, Some("2m"))]);

        let report = execute_auto_track_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoTrackExecutionStatus::Completed);
        assert!(report.state.semaphore_acquired);
        assert!(report.state.semaphore_released);
        assert!(!report.state.long_distance);
        assert!(!report.state.teleport_branch_entered);
        assert!(report.state.quest_navigation_pressed);
        assert!(report.state.arrived);
        assert!(!runtime
            .dispatched_actions
            .iter()
            .any(|action| matches!(action, AutoTrackRuntimeAction::OpenQuestMenu { .. })));
        assert!(!runtime.dispatched_actions.iter().any(|action| {
            matches!(
                action,
                AutoTrackRuntimeAction::ClickTeleportCandidate { .. }
            )
        }));
        assert!(runtime.dispatched_actions.iter().any(|action| {
            matches!(action, AutoTrackRuntimeAction::PressQuestNavigation { .. })
        }));
        assert_eq!(runtime.release_calls, 1);
        assert_eq!(runtime.clear_overlay_calls, 1);
    }

    #[test]
    fn auto_track_execute_far_distance_uses_teleport_branch() {
        let plan = plan_auto_track(AutoTrackExecutionConfig::default());
        let far_candidate = teleport_candidate(AUTO_TRACK_TELEPORT_ASSETS[0], 200, 200);
        let nearest_candidate = teleport_candidate(AUTO_TRACK_TELEPORT_ASSETS[1], 940, 520);
        let mut runtime = FakeAutoTrackRuntime::new()
            .with_main_ui([true, true])
            .with_mission_texts(["260m"])
            .with_teleport_candidates([far_candidate.clone(), nearest_candidate.clone()])
            .with_big_map_open([false])
            .with_tracking([tracking_observation_at(960, 600, Some("3m"))]);

        let report = execute_auto_track_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoTrackExecutionStatus::Completed);
        assert!(report.state.long_distance);
        assert!(report.state.teleport_branch_entered);
        assert_eq!(
            report.state.selected_teleport,
            Some(nearest_candidate.clone())
        );
        assert!(report.state.waited_main_ui);
        assert!(runtime
            .dispatched_actions
            .iter()
            .any(|action| { matches!(action, AutoTrackRuntimeAction::OpenQuestMenu { .. }) }));
        assert_eq!(
            runtime
                .dispatched_actions
                .iter()
                .filter(|action| matches!(action, AutoTrackRuntimeAction::ClickTrackToggle { .. }))
                .count(),
            2
        );
        assert!(runtime.dispatched_actions.iter().any(|action| {
            matches!(
                action,
                AutoTrackRuntimeAction::ClickTeleportCandidate { candidate }
                    if candidate == &nearest_candidate
            )
        }));
        assert!(report.decisions.iter().any(|decision| matches!(
            &decision.decision,
            AutoTrackDecision::WaitMainUi {
                detected: true,
                attempts: 1
            }
        )));
        assert_eq!(runtime.release_calls, 1);
        assert_eq!(runtime.clear_overlay_calls, 1);
    }

    #[test]
    fn auto_track_execute_missing_main_ui_skips_and_cleans_up() {
        let plan = plan_auto_track(AutoTrackExecutionConfig::default());
        let mut runtime = FakeAutoTrackRuntime::new().with_main_ui([false]);

        let report = execute_auto_track_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoTrackExecutionStatus::MainUiMissing);
        assert!(report.state.semaphore_acquired);
        assert!(report.state.semaphore_released);
        assert!(report.state.cleanup_completed);
        assert!(!report.state.quest_navigation_pressed);
        assert!(!runtime.dispatched_actions.iter().any(|action| {
            matches!(action, AutoTrackRuntimeAction::PressQuestNavigation { .. })
        }));
        assert_eq!(runtime.release_calls, 1);
        assert_eq!(runtime.clear_overlay_calls, 1);
        assert!(report.decisions.iter().any(|decision| matches!(
            &decision.decision,
            AutoTrackDecision::Abort {
                status: AutoTrackExecutionStatus::MainUiMissing
            }
        )));
    }

    #[test]
    fn auto_track_execute_arrival_releases_forward() {
        let plan = plan_auto_track(AutoTrackExecutionConfig::default());
        let mut runtime = FakeAutoTrackRuntime::new()
            .with_main_ui([true])
            .with_mission_texts(["20m"])
            .with_tracking([
                tracking_observation_at(960, 840, Some("12m")),
                tracking_observation_at(960, 600, Some("2m")),
            ]);

        let report = execute_auto_track_plan(&plan, &mut runtime).unwrap();

        assert_eq!(report.status, AutoTrackExecutionStatus::Completed);
        assert!(report.state.arrived);
        assert!(!report.state.forward_held);
        assert_eq!(
            runtime
                .dispatched_actions
                .iter()
                .filter(|action| matches!(
                    action,
                    AutoTrackRuntimeAction::ForwardKey {
                        press: AutoTrackActionPress::KeyDown
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            runtime
                .dispatched_actions
                .iter()
                .filter(|action| matches!(
                    action,
                    AutoTrackRuntimeAction::ForwardKey {
                        press: AutoTrackActionPress::KeyUp
                    }
                ))
                .count(),
            1
        );
        assert!(report.decisions.iter().any(|decision| matches!(
            &decision.decision,
            AutoTrackDecision::Tracking(AutoTrackSteeringDecision {
                action: AutoTrackSteeringAction::Arrived {
                    release_forward: true
                },
                ..
            })
        )));
    }

    fn paimon_match() -> AutoTrackTemplateMatch {
        AutoTrackTemplateMatch {
            name: "PaimonMenu".to_string(),
            asset: AUTO_TRACK_PAIMON_MENU_ASSET.to_string(),
            rect: Rect {
                x: 10,
                y: 20,
                width: 80,
                height: 80,
            },
            score: 0.95,
        }
    }

    fn teleport_candidate(asset: &str, x: i32, y: i32) -> AutoTrackTeleportCandidate {
        AutoTrackTeleportCandidate {
            asset: asset.to_string(),
            rect: Rect {
                x,
                y,
                width: 30,
                height: 30,
            },
            score: 0.91,
        }
    }

    fn tracking_observation_at(
        center_x: i32,
        center_y: i32,
        distance_text: Option<&str>,
    ) -> AutoTrackTrackingObservation {
        AutoTrackTrackingObservation {
            blue_track_point: Some(AutoTrackTemplateMatch {
                name: "BlueTrackPoint".to_string(),
                asset: AUTO_TRACK_BLUE_TRACK_POINT_ASSET.to_string(),
                rect: Rect {
                    x: center_x - 10,
                    y: center_y - 10,
                    width: 20,
                    height: 20,
                },
                score: 0.9,
            }),
            mission_distance_text: distance_text.map(ToString::to_string),
        }
    }
}
