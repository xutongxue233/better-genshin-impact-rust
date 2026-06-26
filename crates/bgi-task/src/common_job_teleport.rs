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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeleportMapRule {
    pub tp_json_asset: String,
    pub uses_map_matching: bool,
    pub uses_coordinate_conversion: bool,
    pub adjusts_zoom_level: bool,
    pub drags_big_map: bool,
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
    let map_rule = TeleportMapRule {
        tp_json_asset: "GameTask/AutoTrackPath/Assets/tp.json".to_string(),
        uses_map_matching: true,
        uses_coordinate_conversion: true,
        adjusts_zoom_level: true,
        drags_big_map: true,
    };
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
        quick_teleport_rule,
        pending_native,
        steps,
        notes: "Legacy TpTask calls are represented and executable through an injectable big-map/map-matching/click/completion state machine; desktop live big-map recognition, tp.json map assets, coordinate conversion, click execution, completion detection, and pathing HandleTeleport integration remain pending.".to_string(),
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
    steps.push(TeleportStep::new(
        TeleportStepPhase::MapNavigation,
        if move_map_only {
            "drag big map to requested target"
        } else {
            "drag big map to teleport target"
        },
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
