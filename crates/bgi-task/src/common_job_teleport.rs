use crate::map_recognition::{teyvat_big_map_sift_recognition_rule, BigMapSiftRecognitionRule};
use crate::{Result, TaskError, TaskPortState};
use bgi_vision::Size;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TELEPORT_TASK_KEY: &str = "Teleport";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub capture_size: Size,
    pub kind: TeleportPlanKind,
    pub target: Option<TeleportTargetPlan>,
    pub force: bool,
    pub force_country: Option<String>,
    pub preflight: TeleportPreflightPlan,
    pub retry_rule: TeleportRetryRule,
    pub map_rule: TeleportMapRule,
    pub move_map_rule: TeleportMoveMapRule,
    pub quick_teleport_rule: TeleportQuickTeleportRule,
    pub pending_native: Vec<TeleportNativeDependency>,
    pub steps: Vec<TeleportStep>,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportPlanKind {
    CoordinateTeleport,
    MoveMapTo,
    StatueOfTheSeven,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportTargetPlan {
    pub x: f64,
    pub y: f64,
    pub map_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TeleportMapPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportPreflightPlan {
    pub open_big_map_ui: bool,
    pub verify_big_map_ui: bool,
    pub normalize_underground_map: bool,
    pub return_main_ui_before_open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportRetryRule {
    pub max_attempts: u8,
    pub point_not_activated_policy: TeleportFailurePolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportMapRule {
    pub tp_json_asset: String,
    pub feature_keypoints_asset: String,
    pub feature_mat_asset: String,
    pub layer_256_to_2048_scale: u64,
    pub sift_recognition_rule: BigMapSiftRecognitionRule,
    pub uses_map_matching: bool,
    pub uses_coordinate_conversion: bool,
    pub adjusts_zoom_level: bool,
    pub drags_big_map: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportMoveMapRule {
    pub map_zoom_enabled: bool,
    pub display_tp_point_zoom_level: f64,
    pub min_zoom_level: f64,
    pub max_zoom_level: f64,
    pub default_final_zoom_level: f64,
    pub zoom_precision_threshold: f64,
    pub zoom_recovery_threshold: f64,
    pub map_zoom_out_distance: f64,
    pub map_zoom_in_distance: f64,
    pub map_scale_factor: f64,
    pub move_tolerance: f64,
    pub max_iterations: u64,
    pub max_mouse_move: f64,
    pub step_interval_ms: u64,
    pub max_prediction_failures: u64,
    pub false_positive_min_jump: f64,
    pub false_positive_expected_move_factor: f64,
    pub target_window_max_retries: u64,
    pub post_force_jump_delay_ms: u64,
    pub target_near_current_center_skip_distance: f64,
    pub country_positions: Vec<TeleportCountryPositionRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportCountryPositionRule {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TeleportMoveMapCenterRejectReason {
    RecognitionFailed,
    FalsePositiveJump {
        jump_distance: f64,
        expected_move_len: f64,
        threshold: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TeleportMoveMapPostDragObservation {
    Accepted {
        center: TeleportMapPoint,
        jump_distance: f64,
        expected_move_len: f64,
        threshold: f64,
    },
    Rejected {
        reason: TeleportMoveMapCenterRejectReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TeleportMoveMapCenterDecision {
    UseRecognized {
        center: TeleportMapPoint,
        exception_times: u64,
    },
    BlindWalk {
        center: TeleportMapPoint,
        exception_times: u64,
        reason: TeleportMoveMapCenterRejectReason,
    },
    AbortReTeleport {
        exception_times: u64,
        reason: TeleportMoveMapCenterRejectReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportQuickTeleportRule {
    pub asset_root: String,
    pub detects_teleport_button: bool,
    pub handles_candidate_panel: bool,
    pub waits_for_main_ui_after_click: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportNativeDependency {
    BigMapUiRecognition,
    TpJsonMapAssets,
    MapMatching,
    CoordinateConversion,
    MouseDragAndClickDispatch,
    QuickTeleportPanelRecognition,
    TeleportCompletionDetection,
    PathingNavigationSeed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportStep {
    pub phase: TeleportStepPhase,
    pub label: String,
    pub action: TeleportStepAction,
}

impl TeleportStep {
    fn new(phase: TeleportStepPhase, label: impl Into<String>, action: TeleportStepAction) -> Self {
        Self {
            phase,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportStepPhase {
    Setup,
    ResolveTarget,
    MapNavigation,
    TeleportExecution,
    Completion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum TeleportStepAction {
    OpenBigMapUi,
    VerifyBigMapUi,
    ResolveCoordinateTarget {
        target: TeleportTargetPlan,
        force: bool,
    },
    ResolveNearestTeleportPoint {
        target: TeleportTargetPlan,
        force: bool,
    },
    SwitchCountryOrMap {
        target: TeleportTargetPlan,
        map_name: Option<String>,
        force_country: Option<String>,
    },
    NormalizeUndergroundMap,
    ReadBigMapZoomLevel,
    AdjustMapZoomLevel,
    RecognizeBigMapCenter,
    RecognizeBigMapRect,
    DragBigMapToTarget {
        target: TeleportTargetPlan,
    },
    VerifyTargetPointInBigMapWindow {
        target: TeleportTargetPlan,
    },
    ConvertMapCoordinateToScreenPoint {
        target: TeleportTargetPlan,
    },
    ClickMapTeleportPoint,
    ClickTeleportPanelOrCandidate {
        allow_candidate_fallback: bool,
    },
    MoveMapTo {
        target: TeleportTargetPlan,
        force_country: Option<String>,
        final_zoom_level: f64,
    },
    SelectStatueOfTheSeven,
    HandlePointNotActivated {
        failure_policy: TeleportFailurePolicy,
    },
    WaitForTeleportCompletion {
        max_attempts: u16,
        delay_ms: u32,
        failure_policy: TeleportFailurePolicy,
    },
    SeedNavigationPreviousPositionAfterTeleport {
        target: Option<TeleportTargetPlan>,
    },
    ReturnResult {
        result: TeleportStepResult,
    },
    Log {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportFailurePolicy {
    WarningOnly,
    ContinueAfterPointNotActivated,
    HardError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeleportStepResult {
    Planned,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TeleportExecutionConfig {
    pub kind: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    #[serde(alias = "mapName")]
    #[serde(alias = "MapName")]
    pub map_name: Option<String>,
    pub force: bool,
    #[serde(alias = "forceCountry")]
    #[serde(alias = "ForceCountry")]
    pub force_country: Option<String>,
    #[serde(alias = "finalZoomLevel")]
    #[serde(alias = "FinalZoomLevel")]
    pub final_zoom_level: Option<f64>,
    pub capture_size: Size,
}

impl Default for TeleportExecutionConfig {
    fn default() -> Self {
        Self {
            kind: None,
            x: None,
            y: None,
            map_name: None,
            force: false,
            force_country: None,
            final_zoom_level: None,
            capture_size: Size::new(1920, 1080),
        }
    }
}

impl TeleportExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

pub fn default_teleport_move_map_rule() -> TeleportMoveMapRule {
    TeleportMoveMapRule {
        map_zoom_enabled: true,
        display_tp_point_zoom_level: 4.4,
        min_zoom_level: 2.0,
        max_zoom_level: 5.0,
        default_final_zoom_level: 2.0,
        zoom_precision_threshold: 0.05,
        zoom_recovery_threshold: 0.3,
        map_zoom_out_distance: 1000.0,
        map_zoom_in_distance: 400.0,
        map_scale_factor: 2.361,
        move_tolerance: 200.0,
        max_iterations: 30,
        max_mouse_move: 300.0,
        step_interval_ms: 20,
        max_prediction_failures: 5,
        false_positive_min_jump: 200.0,
        false_positive_expected_move_factor: 2.0,
        target_window_max_retries: 5,
        post_force_jump_delay_ms: 300,
        target_near_current_center_skip_distance: 50.0,
        country_positions: vec![
            TeleportCountryPositionRule {
                name: "蒙德".to_string(),
                x: -876.0,
                y: 2278.0,
            },
            TeleportCountryPositionRule {
                name: "璃月".to_string(),
                x: 270.0,
                y: -666.0,
            },
            TeleportCountryPositionRule {
                name: "稻妻".to_string(),
                x: -4400.0,
                y: -3050.0,
            },
            TeleportCountryPositionRule {
                name: "须弥".to_string(),
                x: 2877.0,
                y: -374.0,
            },
            TeleportCountryPositionRule {
                name: "枫丹".to_string(),
                x: 4515.0,
                y: 3631.0,
            },
            TeleportCountryPositionRule {
                name: "纳塔".to_string(),
                x: 8973.5,
                y: -1879.1,
            },
            TeleportCountryPositionRule {
                name: "挪德卡莱".to_string(),
                x: 9542.25,
                y: 1661.84,
            },
        ],
    }
}

pub fn teleport_move_map_expected_move_len(
    mouse_move_x: i32,
    mouse_move_y: i32,
    zoom_level: f64,
    map_scale_factor: f64,
) -> Option<f64> {
    if !zoom_level.is_finite()
        || !map_scale_factor.is_finite()
        || zoom_level <= 0.0
        || map_scale_factor <= 0.0
    {
        return None;
    }
    let dx = f64::from(mouse_move_x);
    let dy = f64::from(mouse_move_y);
    Some(dx.hypot(dy) * zoom_level / map_scale_factor)
}

pub fn teleport_move_map_false_positive_threshold(
    expected_move_len: f64,
    rule: &TeleportMoveMapRule,
) -> f64 {
    rule.false_positive_min_jump
        .max(expected_move_len * rule.false_positive_expected_move_factor)
}

pub fn teleport_move_map_jump_distance(
    predicted: TeleportMapPoint,
    recognized: TeleportMapPoint,
) -> f64 {
    (recognized.x - predicted.x).hypot(recognized.y - predicted.y)
}

pub fn classify_teleport_move_map_post_drag_center(
    predicted: TeleportMapPoint,
    recognized: Option<TeleportMapPoint>,
    mouse_move_x: i32,
    mouse_move_y: i32,
    zoom_level: f64,
    rule: &TeleportMoveMapRule,
) -> TeleportMoveMapPostDragObservation {
    let expected_move_len = teleport_move_map_expected_move_len(
        mouse_move_x,
        mouse_move_y,
        zoom_level,
        rule.map_scale_factor,
    )
    .unwrap_or(f64::INFINITY);
    let threshold = teleport_move_map_false_positive_threshold(expected_move_len, rule);

    let Some(center) = recognized else {
        return TeleportMoveMapPostDragObservation::Rejected {
            reason: TeleportMoveMapCenterRejectReason::RecognitionFailed,
        };
    };

    let jump_distance = teleport_move_map_jump_distance(predicted, center);
    if jump_distance > threshold {
        TeleportMoveMapPostDragObservation::Rejected {
            reason: TeleportMoveMapCenterRejectReason::FalsePositiveJump {
                jump_distance,
                expected_move_len,
                threshold,
            },
        }
    } else {
        TeleportMoveMapPostDragObservation::Accepted {
            center,
            jump_distance,
            expected_move_len,
            threshold,
        }
    }
}

pub fn apply_teleport_move_map_center_observation(
    previous_exception_times: u64,
    predicted: TeleportMapPoint,
    observation: TeleportMoveMapPostDragObservation,
    max_prediction_failures: u64,
) -> TeleportMoveMapCenterDecision {
    match observation {
        TeleportMoveMapPostDragObservation::Accepted { center, .. } => {
            TeleportMoveMapCenterDecision::UseRecognized {
                center,
                exception_times: 0,
            }
        }
        TeleportMoveMapPostDragObservation::Rejected { reason } => {
            let exception_times = previous_exception_times.saturating_add(1);
            if exception_times > max_prediction_failures {
                TeleportMoveMapCenterDecision::AbortReTeleport {
                    exception_times,
                    reason,
                }
            } else {
                TeleportMoveMapCenterDecision::BlindWalk {
                    center: predicted,
                    exception_times,
                    reason,
                }
            }
        }
    }
}

pub fn decide_teleport_move_map_center_after_drag(
    predicted: TeleportMapPoint,
    recognized: Option<TeleportMapPoint>,
    mouse_move_x: i32,
    mouse_move_y: i32,
    zoom_level: f64,
    previous_exception_times: u64,
    rule: &TeleportMoveMapRule,
) -> TeleportMoveMapCenterDecision {
    let observation = classify_teleport_move_map_post_drag_center(
        predicted,
        recognized,
        mouse_move_x,
        mouse_move_y,
        zoom_level,
        rule,
    );
    apply_teleport_move_map_center_observation(
        previous_exception_times,
        predicted,
        observation,
        rule.max_prediction_failures,
    )
}

pub fn plan_teleport(config: TeleportExecutionConfig) -> Result<TeleportExecutionPlan> {
    let kind = teleport_kind(config.kind.as_deref())?;
    let target = match kind {
        TeleportPlanKind::CoordinateTeleport | TeleportPlanKind::MoveMapTo => {
            Some(TeleportTargetPlan {
                x: config.x.ok_or_else(|| missing_coordinate("x"))?,
                y: config.y.ok_or_else(|| missing_coordinate("y"))?,
                map_name: config.map_name,
            })
        }
        TeleportPlanKind::StatueOfTheSeven => None,
    };
    let mut steps = Vec::new();
    let preflight = TeleportPreflightPlan {
        open_big_map_ui: true,
        verify_big_map_ui: true,
        normalize_underground_map: true,
        return_main_ui_before_open: true,
    };
    let retry_rule = TeleportRetryRule {
        max_attempts: 3,
        point_not_activated_policy: TeleportFailurePolicy::ContinueAfterPointNotActivated,
    };
    let sift_recognition_rule = teyvat_big_map_sift_recognition_rule();
    let map_rule = TeleportMapRule {
        tp_json_asset: "GameTask/AutoTrackPath/Assets/tp.json".to_string(),
        feature_keypoints_asset: sift_recognition_rule.feature_keypoints_asset.clone(),
        feature_mat_asset: sift_recognition_rule.feature_mat_asset.clone(),
        layer_256_to_2048_scale: sift_recognition_rule.feature_layer.image_to_2048_scale,
        sift_recognition_rule,
        uses_map_matching: true,
        uses_coordinate_conversion: true,
        adjusts_zoom_level: true,
        drags_big_map: true,
    };
    let move_map_rule = default_teleport_move_map_rule();
    let quick_teleport_rule = TeleportQuickTeleportRule {
        asset_root: "GameTask/QuickTeleport/Assets".to_string(),
        detects_teleport_button: true,
        handles_candidate_panel: true,
        waits_for_main_ui_after_click: true,
    };
    let pending_native = vec![
        TeleportNativeDependency::BigMapUiRecognition,
        TeleportNativeDependency::TpJsonMapAssets,
        TeleportNativeDependency::MapMatching,
        TeleportNativeDependency::CoordinateConversion,
        TeleportNativeDependency::MouseDragAndClickDispatch,
        TeleportNativeDependency::QuickTeleportPanelRecognition,
        TeleportNativeDependency::TeleportCompletionDetection,
        TeleportNativeDependency::PathingNavigationSeed,
    ];

    steps.push(TeleportStep::new(
        TeleportStepPhase::Setup,
        "log teleport start",
        TeleportStepAction::Log {
            message: format!("start Teleport common job plan: {kind:?}"),
        },
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::Setup,
        "open big map UI",
        TeleportStepAction::OpenBigMapUi,
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::Setup,
        "verify big map UI",
        TeleportStepAction::VerifyBigMapUi,
    ));
    match kind {
        TeleportPlanKind::CoordinateTeleport => {
            let target = target
                .clone()
                .expect("coordinate teleport target is validated");
            steps.push(TeleportStep::new(
                TeleportStepPhase::ResolveTarget,
                "resolve coordinate teleport target",
                TeleportStepAction::ResolveCoordinateTarget {
                    target: target.clone(),
                    force: config.force,
                },
            ));
            steps.push(TeleportStep::new(
                TeleportStepPhase::ResolveTarget,
                "resolve nearest teleport point",
                TeleportStepAction::ResolveNearestTeleportPoint {
                    target: target.clone(),
                    force: config.force,
                },
            ));
            push_map_navigation_steps(&mut steps, &target, config.force_country.clone(), false);
            steps.push(TeleportStep::new(
                TeleportStepPhase::TeleportExecution,
                "click map teleport point",
                TeleportStepAction::ClickMapTeleportPoint,
            ));
            steps.push(TeleportStep::new(
                TeleportStepPhase::TeleportExecution,
                "click teleport panel or candidate entry",
                TeleportStepAction::ClickTeleportPanelOrCandidate {
                    allow_candidate_fallback: true,
                },
            ));
            steps.push(TeleportStep::new(
                TeleportStepPhase::TeleportExecution,
                "handle point not activated",
                TeleportStepAction::HandlePointNotActivated {
                    failure_policy: retry_rule.point_not_activated_policy.clone(),
                },
            ));
        }
        TeleportPlanKind::MoveMapTo => {
            let target = target.clone().expect("move-map target is validated");
            push_map_navigation_steps(&mut steps, &target, config.force_country.clone(), true);
            steps.push(TeleportStep::new(
                TeleportStepPhase::MapNavigation,
                "move big map to requested target",
                TeleportStepAction::MoveMapTo {
                    target,
                    force_country: config.force_country.clone(),
                    final_zoom_level: config
                        .final_zoom_level
                        .unwrap_or(move_map_rule.default_final_zoom_level),
                },
            ));
        }
        TeleportPlanKind::StatueOfTheSeven => {
            steps.push(TeleportStep::new(
                TeleportStepPhase::ResolveTarget,
                "select configured Statue of the Seven target",
                TeleportStepAction::SelectStatueOfTheSeven,
            ));
            steps.push(TeleportStep::new(
                TeleportStepPhase::TeleportExecution,
                "click teleport panel or candidate entry",
                TeleportStepAction::ClickTeleportPanelOrCandidate {
                    allow_candidate_fallback: false,
                },
            ));
        }
    }
    if matches!(
        kind,
        TeleportPlanKind::CoordinateTeleport | TeleportPlanKind::StatueOfTheSeven
    ) {
        steps.push(TeleportStep::new(
            TeleportStepPhase::Completion,
            "wait for teleport completion",
            TeleportStepAction::WaitForTeleportCompletion {
                max_attempts: 50,
                delay_ms: 1_200,
                failure_policy: TeleportFailurePolicy::WarningOnly,
            },
        ));
        steps.push(TeleportStep::new(
            TeleportStepPhase::Completion,
            "seed pathing navigation previous position after teleport",
            TeleportStepAction::SeedNavigationPreviousPositionAfterTeleport {
                target: target.clone(),
            },
        ));
    }
    steps.push(TeleportStep::new(
        TeleportStepPhase::Completion,
        "return planned result",
        TeleportStepAction::ReturnResult {
            result: TeleportStepResult::Planned,
        },
    ));

    Ok(TeleportExecutionPlan {
        task_key: TELEPORT_TASK_KEY.to_string(),
        display_name: "Teleport".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: true,
        capture_size: config.capture_size,
        kind,
        target,
        force: config.force,
        force_country: config.force_country,
        preflight,
        retry_rule,
        map_rule,
        move_map_rule,
        quick_teleport_rule,
        pending_native,
        steps,
        notes: "Legacy TpTask calls are represented and executable through an injectable big-map/map-matching/click/completion state machine; desktop live bridge now dispatches OpenMap, verifies BigMap with PureRust templates, normalizes underground map state with QuickTeleport templates, opens the area menu and clicks the target country/independent map from WinRT OCR, loads tp.json, carries the legacy MoveMapTo TpConfig defaults and Teyvat country-center anchors in the plan, resolves the nearest teleport point with the legacy position[2]/position[0] coordinate mapping, reads and adjusts the BigMap zoom slider with legacy 2.0/4.4 thresholds, preserves Teyvat coordinate conversion and target-window guard geometry, plans BigMap drags with the legacy mapScaleFactor/max-move/cosine-step/predicted-center math from the plan rule, runs MoveMapTo through a bounded prediction loop with legacy zoom-out/zoom-in target formulas, target-window rechecks, post-drag center update decisions, false-positive jump rejection, and C#-compatible >5 prediction-failure termination, removes duplicate MoveMapTo pre-drag actions, fails MoveMapTo when the runtime reports non-convergence, clicks the direct teleport-panel button, detects the QuickTeleport map-close/not-activated boundary, falls back to QuickTeleport candidate-panel template/OCR clicks with the legacy 500ms minimum delay plus 6x300ms teleport-button appear/disappear probing, applies point-not-activated ESC recovery with up to three coordinate-teleport attempts, polls main-UI completion, records the effective pathing navigation seed in the execution report, and can feed that seed into the AutoPathing action-boundary previous-position report when called from HandleTeleport, while non-CHS localization parity, native SIFT/map matching/BigMap center+rect recognition, MoveMapTo native recognized-center feed/rect correction/force-jump recovery, track-coordinate conversion into live navigation state, and full PathExecutor movement consumption remain pending.".to_string(),
    })
}

fn push_map_navigation_steps(
    steps: &mut Vec<TeleportStep>,
    target: &TeleportTargetPlan,
    force_country: Option<String>,
    move_map_only: bool,
) {
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "switch country or independent map",
        TeleportStepAction::SwitchCountryOrMap {
            target: target.clone(),
            map_name: target.map_name.clone(),
            force_country,
        },
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "normalize underground map state",
        TeleportStepAction::NormalizeUndergroundMap,
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "read big-map zoom level",
        TeleportStepAction::ReadBigMapZoomLevel,
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "adjust big-map zoom level",
        TeleportStepAction::AdjustMapZoomLevel,
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "recognize big-map center",
        TeleportStepAction::RecognizeBigMapCenter,
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "recognize big-map rect",
        TeleportStepAction::RecognizeBigMapRect,
    ));
    if move_map_only {
        return;
    }
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "drag big map to teleport target",
        TeleportStepAction::DragBigMapToTarget {
            target: target.clone(),
        },
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "verify target point in big-map window",
        TeleportStepAction::VerifyTargetPointInBigMapWindow {
            target: target.clone(),
        },
    ));
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        "convert map coordinate to screen point",
        TeleportStepAction::ConvertMapCoordinateToScreenPoint {
            target: target.clone(),
        },
    ));
}

fn teleport_kind(kind: Option<&str>) -> Result<TeleportPlanKind> {
    match kind.unwrap_or("").trim() {
        "" | "coordinateTeleport" | "CoordinateTeleport" | "teleport" | "Teleport" => {
            Ok(TeleportPlanKind::CoordinateTeleport)
        }
        "moveMapTo" | "MoveMapTo" => Ok(TeleportPlanKind::MoveMapTo),
        "statueOfTheSeven" | "StatueOfTheSeven" => Ok(TeleportPlanKind::StatueOfTheSeven),
        other => Err(TaskError::InvalidTaskConfig {
            key: TELEPORT_TASK_KEY.to_string(),
            message: format!("unknown teleport kind: {other}"),
        }),
    }
}

fn missing_coordinate(name: &str) -> TaskError {
    TaskError::InvalidTaskConfig {
        key: TELEPORT_TASK_KEY.to_string(),
        message: format!("{name} is required for teleport target"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> TeleportMapPoint {
        TeleportMapPoint { x, y }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn teleport_move_map_expected_move_len_uses_integer_drag_zoom_and_scale() {
        let actual = teleport_move_map_expected_move_len(3, 4, 2.0, 2.0);

        assert_eq!(actual, Some(5.0));
        assert_eq!(teleport_move_map_expected_move_len(3, 4, 0.0, 2.0), None);
        assert_eq!(teleport_move_map_expected_move_len(3, 4, 2.0, 0.0), None);
    }

    #[test]
    fn teleport_move_map_false_positive_threshold_uses_floor_and_strict_greater_than() {
        let mut rule = default_teleport_move_map_rule();
        rule.map_scale_factor = 1.0;
        let predicted = point(0.0, 0.0);

        let accepted = classify_teleport_move_map_post_drag_center(
            predicted,
            Some(point(200.0, 0.0)),
            25,
            0,
            2.0,
            &rule,
        );
        assert!(matches!(
            accepted,
            TeleportMoveMapPostDragObservation::Accepted {
                jump_distance: 200.0,
                expected_move_len: 50.0,
                threshold: 200.0,
                ..
            }
        ));

        let rejected = classify_teleport_move_map_post_drag_center(
            predicted,
            Some(point(200.001, 0.0)),
            25,
            0,
            2.0,
            &rule,
        );
        match rejected {
            TeleportMoveMapPostDragObservation::Rejected {
                reason:
                    TeleportMoveMapCenterRejectReason::FalsePositiveJump {
                        jump_distance,
                        expected_move_len,
                        threshold,
                    },
            } => {
                assert_close(jump_distance, 200.001);
                assert_eq!(expected_move_len, 50.0);
                assert_eq!(threshold, 200.0);
            }
            other => panic!("unexpected observation: {other:?}"),
        }
    }

    #[test]
    fn teleport_move_map_false_positive_threshold_uses_twice_expected_for_large_moves() {
        let mut rule = default_teleport_move_map_rule();
        rule.map_scale_factor = 1.0;
        let predicted = point(0.0, 0.0);

        let accepted = classify_teleport_move_map_post_drag_center(
            predicted,
            Some(point(300.0, 0.0)),
            75,
            0,
            2.0,
            &rule,
        );
        assert!(matches!(
            accepted,
            TeleportMoveMapPostDragObservation::Accepted {
                jump_distance: 300.0,
                expected_move_len: 150.0,
                threshold: 300.0,
                ..
            }
        ));

        let rejected = classify_teleport_move_map_post_drag_center(
            predicted,
            Some(point(300.001, 0.0)),
            75,
            0,
            2.0,
            &rule,
        );
        assert!(matches!(
            rejected,
            TeleportMoveMapPostDragObservation::Rejected {
                reason: TeleportMoveMapCenterRejectReason::FalsePositiveJump {
                    expected_move_len: 150.0,
                    threshold: 300.0,
                    ..
                }
            }
        ));
    }

    #[test]
    fn teleport_move_map_success_accepts_center_and_resets_exception_times() {
        let mut rule = default_teleport_move_map_rule();
        rule.map_scale_factor = 1.0;
        let recognized = point(150.0, 0.0);

        let decision = decide_teleport_move_map_center_after_drag(
            point(100.0, 0.0),
            Some(recognized),
            50,
            0,
            2.0,
            3,
            &rule,
        );

        assert_eq!(
            decision,
            TeleportMoveMapCenterDecision::UseRecognized {
                center: recognized,
                exception_times: 0
            }
        );
    }

    #[test]
    fn teleport_move_map_recognition_failure_blind_walks_with_predicted_center() {
        let mut rule = default_teleport_move_map_rule();
        rule.max_prediction_failures = 5;
        let predicted = point(100.0, 0.0);

        let decision =
            decide_teleport_move_map_center_after_drag(predicted, None, 50, 0, 2.0, 2, &rule);

        assert_eq!(
            decision,
            TeleportMoveMapCenterDecision::BlindWalk {
                center: predicted,
                exception_times: 3,
                reason: TeleportMoveMapCenterRejectReason::RecognitionFailed
            }
        );
    }

    #[test]
    fn teleport_move_map_false_positive_blind_walks_like_recognition_failure() {
        let mut rule = default_teleport_move_map_rule();
        rule.map_scale_factor = 1.0;
        let predicted = point(100.0, 0.0);

        let decision = decide_teleport_move_map_center_after_drag(
            predicted,
            Some(point(301.0, 0.0)),
            50,
            0,
            2.0,
            2,
            &rule,
        );

        match decision {
            TeleportMoveMapCenterDecision::BlindWalk {
                center,
                exception_times,
                reason:
                    TeleportMoveMapCenterRejectReason::FalsePositiveJump {
                        jump_distance,
                        expected_move_len,
                        threshold,
                    },
            } => {
                assert_eq!(center, predicted);
                assert_eq!(exception_times, 3);
                assert_eq!(jump_distance, 201.0);
                assert_eq!(expected_move_len, 100.0);
                assert_eq!(threshold, 200.0);
            }
            other => panic!("unexpected decision: {other:?}"),
        }
    }

    #[test]
    fn teleport_move_map_allows_five_failures_and_aborts_on_sixth() {
        let mut rule = default_teleport_move_map_rule();
        rule.max_prediction_failures = 5;
        let predicted = point(100.0, 0.0);

        let allowed =
            decide_teleport_move_map_center_after_drag(predicted, None, 50, 0, 2.0, 4, &rule);
        assert_eq!(
            allowed,
            TeleportMoveMapCenterDecision::BlindWalk {
                center: predicted,
                exception_times: 5,
                reason: TeleportMoveMapCenterRejectReason::RecognitionFailed
            }
        );

        let aborted =
            decide_teleport_move_map_center_after_drag(predicted, None, 50, 0, 2.0, 5, &rule);
        assert_eq!(
            aborted,
            TeleportMoveMapCenterDecision::AbortReTeleport {
                exception_times: 6,
                reason: TeleportMoveMapCenterRejectReason::RecognitionFailed
            }
        );
    }
}
