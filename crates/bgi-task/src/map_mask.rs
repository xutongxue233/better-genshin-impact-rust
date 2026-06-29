use crate::map_recognition::{teyvat_big_map_sift_recognition_rule, BigMapSiftRecognitionRule};
pub use crate::map_recognition::{
    TEYVAT_256_SIFT_KEYPOINTS as MAP_MASK_TEYVAT_256_SIFT_KEYPOINTS,
    TEYVAT_256_SIFT_MAT as MAP_MASK_TEYVAT_256_SIFT_MAT,
};
use bgi_core::MapMaskConfig;
use bgi_vision::{Point, Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAP_MASK_TASK_KEY: &str = "MapMask";
pub const MAP_MASK_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const MAP_MASK_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const MAP_MASK_BIG_MAP_SCALE_BUTTON: &str = "QuickTeleport:MapScaleButton.png";
pub const MAP_MASK_BIG_MAP_SETTINGS_BUTTON: &str = "QuickTeleport:MapSettingsButton.png";
pub const MAP_MASK_PAIMON_MENU: &str = "Common/Element:paimon_menu.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: MapMaskConfigRule,
    pub throttle_rule: MapMaskThrottleRule,
    pub ui_detection_rule: MapMaskUiDetectionRule,
    pub stability_rule: MapMaskStabilityRule,
    pub big_map_rule: MapMaskBigMapRule,
    pub mini_map_rule: MapMaskMiniMapRule,
    pub teyvat_map_rule: MapMaskTeyvatMapRule,
    pub point_provider_rule: MapMaskPointProviderRule,
    pub overlay_rule: MapMaskOverlayRule,
    pub steps: Vec<MapMaskTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapMaskExecutionConfig {
    pub capture_size: Size,
    pub map_mask_config: MapMaskConfig,
}

impl Default for MapMaskExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                MAP_MASK_DEFAULT_CAPTURE_WIDTH,
                MAP_MASK_DEFAULT_CAPTURE_HEIGHT,
            ),
            map_mask_config: MapMaskConfig::default(),
        }
    }
}

impl MapMaskExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let map_mask_value = value
            .get("mapMaskConfig")
            .or_else(|| value.get("MapMaskConfig"))
            .or_else(|| value.get("map_mask_config"))
            .unwrap_or(value);
        config.map_mask_config = serde_json::from_value(map_mask_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskConfigRule {
    pub enabled: bool,
    pub mini_map_mask_enabled: bool,
    pub path_auto_record_enabled: bool,
    pub map_point_api_provider: MapMaskPointProvider,
    pub map_point_api_provider_raw: String,
    pub ho_yo_lab_language: String,
    pub ho_yo_lab_supported_languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MapMaskPointProvider {
    MihoyoMap,
    KongyingTavern,
    HoYoLab,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskThrottleRule {
    pub tick_interval_ms: u64,
    pub skip_when_elapsed_less_or_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskUiDetectionRule {
    pub supported_trigger_category: String,
    pub big_map_when_current_ui_big_map: bool,
    pub big_map_when_bv_detects_big_map: bool,
    pub mini_map_requires_main_ui: bool,
    pub big_map_templates: Vec<MapMaskTemplateLocator>,
    pub main_ui_template: MapMaskTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskStabilityRule {
    pub similarity_threshold: f64,
    pub stable_frame_count: u64,
    pub downscale_size: Size,
    pub match_mode: TemplateMatchMode,
    pub reset_after_stable_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskBigMapRule {
    pub feature_keypoints_asset: String,
    pub feature_mat_asset: String,
    pub layer_256_to_2048_scale: u64,
    pub sift_recognition_rule: BigMapSiftRecognitionRule,
    pub reject_when_width_lt_and_height_lt: MapMaskSizeRejectRule,
    pub reject_when_width_gt_and_height_gt: MapMaskSizeRejectRule,
    pub update_points_canvas_when_rect_found: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskSizeRejectRule {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskMiniMapRule {
    pub enabled: bool,
    pub requires_main_ui: bool,
    pub mini_map_roi: Rect,
    pub viewport_size: f64,
    pub center_source: String,
    pub position_jump_reset_threshold: f64,
    pub match_config: MapMaskMiniMapMatchRule,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapMaskMiniMapMatchRule {
    pub original_size: u32,
    pub rough_size: u32,
    pub exact_size: u32,
    pub rough_zoom: u32,
    pub exact_zoom: u32,
    pub rough_search_radius: u32,
    pub exact_search_radius: u32,
    pub confidence_thresholds: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskTeyvatMapRule {
    pub rows: u32,
    pub cols: u32,
    pub up_rows: u32,
    pub left_cols: u32,
    pub block_size: u32,
    pub origin_x: u32,
    pub origin_y: u32,
    pub split_rows: u32,
    pub split_cols: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskPointProviderRule {
    pub provider: MapMaskPointProvider,
    pub provider_raw: String,
    pub ho_yo_lab_language: String,
    pub supported_ho_yo_lab_languages: Vec<String>,
    pub fetches_points_for_big_map: bool,
    pub fetches_points_for_mini_map: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskOverlayRule {
    pub big_map_canvas: String,
    pub mini_map_canvas: String,
    pub window_source: String,
    pub clears_big_map_points_outside_big_map: bool,
    pub clears_mini_map_points_outside_main_ui: bool,
    pub path_auto_record_status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MapMaskRuntimeState {
    pub stable_count: u64,
    pub previous_big_map_rect_256: Option<Rect>,
    pub is_in_big_map_ui: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskTickObservation {
    pub elapsed_since_previous_tick_ms: u64,
    pub current_ui_is_big_map: bool,
    pub bv_detected_big_map_ui: bool,
    pub bv_detected_main_ui: bool,
    pub frame_is_stable: bool,
}

impl Default for MapMaskTickObservation {
    fn default() -> Self {
        Self {
            elapsed_since_previous_tick_ms: 51,
            current_ui_is_big_map: false,
            bv_detected_big_map_ui: false,
            bv_detected_main_ui: false,
            frame_is_stable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskTickReduction {
    pub state: MapMaskRuntimeState,
    pub effects: Vec<MapMaskTickEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskTickExecutionReport {
    pub observation: MapMaskTickObservation,
    pub effects: Vec<MapMaskTickEffect>,
    pub executed_actions: Vec<MapMaskExecutedAction>,
    pub state: MapMaskRuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MapMaskExecutedAction {
    ClearAllOverlayCanvases,
    SetBigMapUi(bool),
    EnqueueBigMapCompute { previous_rect_256: Option<Rect> },
    ResetBigMapPreviousRect,
    EnqueueMiniMapCompute,
    ClearMiniMapViewport,
}

pub trait MapMaskRuntime {
    fn observe_map_mask_tick(
        &mut self,
        plan: &MapMaskExecutionPlan,
        state: &MapMaskRuntimeState,
    ) -> MapMaskTickObservation;

    fn clear_map_mask_overlay_canvases(&mut self, plan: &MapMaskExecutionPlan);

    fn set_map_mask_big_map_ui(&mut self, plan: &MapMaskExecutionPlan, is_in_big_map_ui: bool);

    fn enqueue_map_mask_big_map_compute(
        &mut self,
        plan: &MapMaskExecutionPlan,
        previous_rect_256: Option<Rect>,
    );

    fn reset_map_mask_previous_big_map_rect(&mut self, plan: &MapMaskExecutionPlan);

    fn enqueue_map_mask_mini_map_compute(&mut self, plan: &MapMaskExecutionPlan);

    fn clear_map_mask_mini_map_viewport(&mut self, plan: &MapMaskExecutionPlan);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MapMaskTickEffect {
    SkipTick(MapMaskSkipReason),
    ClearAllOverlayCanvases,
    SetBigMapUi(bool),
    EnqueueBigMapCompute { previous_rect_256: Option<Rect> },
    ResetBigMapPreviousRect,
    EnqueueMiniMapCompute,
    ClearMiniMapViewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MapMaskSkipReason {
    Disabled,
    Throttled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MapMaskBigMapViewportDecision {
    Update {
        viewport_2048: Rect,
        accepted_rect_256: Option<Rect>,
    },
    Reject {
        reason: MapMaskBigMapRectRejectReason,
        reset_previous_rect: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MapMaskBigMapRectRejectReason {
    TooSmall,
    TooLarge,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MapMaskMiniMapViewportDecision {
    Update { viewport: MapMaskViewport },
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MapMaskViewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MapMaskTickStep {
    pub phase: MapMaskTickPhase,
    pub condition: MapMaskTickCondition,
    pub action: MapMaskTickAction,
}

impl MapMaskTickStep {
    fn new(
        phase: MapMaskTickPhase,
        condition: MapMaskTickCondition,
        action: MapMaskTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MapMaskTickPhase {
    Throttle,
    BigMapGate,
    BigMapStability,
    BigMapMatch,
    MiniMapGate,
    MiniMapMatch,
    PointProvider,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MapMaskTickCondition {
    WhenTickElapsedLessOrEqual50Ms,
    WhenBigMapUiDetected,
    WhenFrameStable,
    WhenStableCountReachesResetThreshold,
    WhenBigMapRectAccepted,
    WhenMiniMapEnabledAndMainUi,
    WhenMiniMapPositionStable,
    WhenPointsAreAvailable,
    WhenLeavingRelevantUi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MapMaskTickAction {
    SkipTick,
    RunBigMapStabilityDetector,
    ResetBigMapStableCount,
    MatchTeyvat256LayerRect,
    UpdateBigMapPointsCanvas,
    MatchMiniMapPosition,
    FetchOrReadPointCache,
    UpdateMiniMapPointsCanvas,
    ClearOverlayCanvases,
}

pub fn plan_map_mask(config: MapMaskExecutionConfig) -> MapMaskExecutionPlan {
    let capture_size = config.capture_size;
    let map_mask_config = config.map_mask_config;
    let provider = point_provider(&map_mask_config.map_point_api_provider);
    let supported_languages = vec![
        "en-us".to_string(),
        "pt-pt".to_string(),
        "es-es".to_string(),
    ];
    let sift_recognition_rule = teyvat_big_map_sift_recognition_rule();
    let config_rule = MapMaskConfigRule {
        enabled: map_mask_config.enabled,
        mini_map_mask_enabled: map_mask_config.mini_map_mask_enabled,
        path_auto_record_enabled: map_mask_config.path_auto_record_enabled,
        map_point_api_provider: provider.clone(),
        map_point_api_provider_raw: map_mask_config.map_point_api_provider.clone(),
        ho_yo_lab_language: map_mask_config.ho_yo_lab_language.clone(),
        ho_yo_lab_supported_languages: supported_languages.clone(),
    };

    MapMaskExecutionPlan {
        task_key: MAP_MASK_TASK_KEY.to_string(),
        display_name: "Map Mask".to_string(),
        capture_size,
        throttle_rule: MapMaskThrottleRule {
            tick_interval_ms: 50,
            skip_when_elapsed_less_or_equal: true,
        },
        ui_detection_rule: MapMaskUiDetectionRule {
            supported_trigger_category: "Unknown".to_string(),
            big_map_when_current_ui_big_map: true,
            big_map_when_bv_detects_big_map: true,
            mini_map_requires_main_ui: true,
            big_map_templates: vec![
                template(
                    "MapScaleButton",
                    MAP_MASK_BIG_MAP_SCALE_BUTTON,
                    Rect {
                        x: scaled(30, capture_size),
                        y: scaled(440, capture_size),
                        width: scaled(40, capture_size),
                        height: scaled(200, capture_size),
                    },
                ),
                template(
                    "MapSettingsButton",
                    MAP_MASK_BIG_MAP_SETTINGS_BUTTON,
                    Rect {
                        x: scaled(25, capture_size),
                        y: scaled(990, capture_size),
                        width: scaled(58, capture_size),
                        height: scaled(62, capture_size),
                    },
                ),
            ],
            main_ui_template: template(
                "PaimonMenu",
                MAP_MASK_PAIMON_MENU,
                Rect {
                    x: 0,
                    y: 0,
                    width: capture_size.width as i32 / 2,
                    height: capture_size.height as i32 / 2,
                },
            ),
        },
        stability_rule: MapMaskStabilityRule {
            similarity_threshold: 0.98,
            stable_frame_count: 2,
            downscale_size: Size::new(320, 180),
            match_mode: TemplateMatchMode::CCoeffNormed,
            reset_after_stable_count: 20,
        },
        big_map_rule: MapMaskBigMapRule {
            feature_keypoints_asset: sift_recognition_rule.feature_keypoints_asset.clone(),
            feature_mat_asset: sift_recognition_rule.feature_mat_asset.clone(),
            layer_256_to_2048_scale: sift_recognition_rule.feature_layer.image_to_2048_scale,
            sift_recognition_rule,
            reject_when_width_lt_and_height_lt: MapMaskSizeRejectRule {
                width: 50,
                height: 40,
            },
            reject_when_width_gt_and_height_gt: MapMaskSizeRejectRule {
                width: 3000,
                height: 1800,
            },
            update_points_canvas_when_rect_found: true,
        },
        mini_map_rule: MapMaskMiniMapRule {
            enabled: config_rule.mini_map_mask_enabled,
            requires_main_ui: true,
            mini_map_roi: Rect {
                x: scaled(62, capture_size),
                y: scaled(19, capture_size),
                width: scaled(212, capture_size),
                height: scaled(212, capture_size),
            },
            viewport_size: 212.0 / 3.0 * 10.0,
            center_source: "NavigationInstance.GetPositionStable".to_string(),
            position_jump_reset_threshold: 150.0,
            match_config: MapMaskMiniMapMatchRule {
                original_size: 156,
                rough_size: 52,
                exact_size: 260,
                rough_zoom: 5,
                exact_zoom: 1,
                rough_search_radius: 50,
                exact_search_radius: 20,
                confidence_thresholds: vec![0.99, 0.97, 0.95],
            },
        },
        teyvat_map_rule: MapMaskTeyvatMapRule {
            rows: 15,
            cols: 22,
            up_rows: 7,
            left_cols: 15,
            block_size: 2048,
            origin_x: 32768,
            origin_y: 16384,
            split_rows: 30,
            split_cols: 44,
        },
        point_provider_rule: MapMaskPointProviderRule {
            provider,
            provider_raw: config_rule.map_point_api_provider_raw.clone(),
            ho_yo_lab_language: config_rule.ho_yo_lab_language.clone(),
            supported_ho_yo_lab_languages: supported_languages,
            fetches_points_for_big_map: true,
            fetches_points_for_mini_map: true,
        },
        overlay_rule: MapMaskOverlayRule {
            big_map_canvas: "PointsCanvasControl".to_string(),
            mini_map_canvas: "MiniMapPointsCanvasControl".to_string(),
            window_source: "MaskWindow".to_string(),
            clears_big_map_points_outside_big_map: true,
            clears_mini_map_points_outside_main_ui: true,
            path_auto_record_status: if config_rule.path_auto_record_enabled {
                "configured but native route-recording TODO remains".to_string()
            } else {
                "disabled".to_string()
            },
        },
        config_rule,
        steps: map_mask_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop adapters for OpenCV/SIFT feature loading and BigMapTeyvat256Layer.KnnMatchRect"
                .to_string(),
            "desktop adapters for MiniMapPreprocessor, FastSqDiffMatcher, TemplateMatchSubPix, live capture Mat/grayscale cache, and worker queue coordination".to_string(),
            "desktop adapters for WPF MaskWindow/points canvas rendering and MihoyoMap/KongyingTavern/HoYoLab point API cache/icon loading".to_string(),
            "PathAutoRecordEnabled route recording integration, which still needs a native Rust adapter"
                .to_string(),
        ],
    }
}

fn map_mask_steps() -> Vec<MapMaskTickStep> {
    vec![
        MapMaskTickStep::new(
            MapMaskTickPhase::Throttle,
            MapMaskTickCondition::WhenTickElapsedLessOrEqual50Ms,
            MapMaskTickAction::SkipTick,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::BigMapGate,
            MapMaskTickCondition::WhenBigMapUiDetected,
            MapMaskTickAction::RunBigMapStabilityDetector,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::BigMapStability,
            MapMaskTickCondition::WhenFrameStable,
            MapMaskTickAction::MatchTeyvat256LayerRect,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::BigMapStability,
            MapMaskTickCondition::WhenStableCountReachesResetThreshold,
            MapMaskTickAction::ResetBigMapStableCount,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::BigMapMatch,
            MapMaskTickCondition::WhenBigMapRectAccepted,
            MapMaskTickAction::UpdateBigMapPointsCanvas,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::MiniMapGate,
            MapMaskTickCondition::WhenMiniMapEnabledAndMainUi,
            MapMaskTickAction::MatchMiniMapPosition,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::MiniMapMatch,
            MapMaskTickCondition::WhenMiniMapPositionStable,
            MapMaskTickAction::UpdateMiniMapPointsCanvas,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::PointProvider,
            MapMaskTickCondition::WhenPointsAreAvailable,
            MapMaskTickAction::FetchOrReadPointCache,
        ),
        MapMaskTickStep::new(
            MapMaskTickPhase::Overlay,
            MapMaskTickCondition::WhenLeavingRelevantUi,
            MapMaskTickAction::ClearOverlayCanvases,
        ),
    ]
}

pub fn reduce_map_mask_tick(
    plan: &MapMaskExecutionPlan,
    state: &MapMaskRuntimeState,
    observation: MapMaskTickObservation,
) -> MapMaskTickReduction {
    let mut next = state.clone();
    let mut effects = Vec::new();

    if !plan.config_rule.enabled {
        effects.push(MapMaskTickEffect::ClearAllOverlayCanvases);
        effects.push(MapMaskTickEffect::SkipTick(MapMaskSkipReason::Disabled));
        return MapMaskTickReduction {
            state: next,
            effects,
        };
    }

    if plan.throttle_rule.skip_when_elapsed_less_or_equal
        && observation.elapsed_since_previous_tick_ms <= plan.throttle_rule.tick_interval_ms
    {
        effects.push(MapMaskTickEffect::SkipTick(MapMaskSkipReason::Throttled));
        return MapMaskTickReduction {
            state: next,
            effects,
        };
    }

    let in_big_map_ui = observation.current_ui_is_big_map || observation.bv_detected_big_map_ui;
    next.is_in_big_map_ui = in_big_map_ui;

    if in_big_map_ui {
        if observation.frame_is_stable {
            next.stable_count = next.stable_count.saturating_add(1);
            if next.stable_count >= plan.stability_rule.reset_after_stable_count {
                next.stable_count = 0;
            }
        } else {
            next.stable_count = 0;
        }

        if next.stable_count == 0 {
            effects.push(MapMaskTickEffect::EnqueueBigMapCompute {
                previous_rect_256: next.previous_big_map_rect_256,
            });
        }
    } else {
        if plan.config_rule.mini_map_mask_enabled {
            if observation.bv_detected_main_ui {
                effects.push(MapMaskTickEffect::EnqueueMiniMapCompute);
            } else {
                effects.push(MapMaskTickEffect::ClearMiniMapViewport);
            }
        }

        if next.previous_big_map_rect_256.is_some() {
            next.previous_big_map_rect_256 = None;
            effects.push(MapMaskTickEffect::ResetBigMapPreviousRect);
        }
    }

    effects.push(MapMaskTickEffect::SetBigMapUi(in_big_map_ui));

    MapMaskTickReduction {
        state: next,
        effects,
    }
}

pub fn execute_map_mask_tick_plan<R: MapMaskRuntime>(
    plan: &MapMaskExecutionPlan,
    state: &MapMaskRuntimeState,
    runtime: &mut R,
) -> MapMaskTickExecutionReport {
    let observation = runtime.observe_map_mask_tick(plan, state);
    let reduction = reduce_map_mask_tick(plan, state, observation.clone());
    let mut executed_actions = Vec::new();

    for effect in &reduction.effects {
        match effect {
            MapMaskTickEffect::ClearAllOverlayCanvases => {
                runtime.clear_map_mask_overlay_canvases(plan);
                executed_actions.push(MapMaskExecutedAction::ClearAllOverlayCanvases);
            }
            MapMaskTickEffect::SetBigMapUi(is_in_big_map_ui) => {
                runtime.set_map_mask_big_map_ui(plan, *is_in_big_map_ui);
                executed_actions.push(MapMaskExecutedAction::SetBigMapUi(*is_in_big_map_ui));
            }
            MapMaskTickEffect::EnqueueBigMapCompute { previous_rect_256 } => {
                runtime.enqueue_map_mask_big_map_compute(plan, *previous_rect_256);
                executed_actions.push(MapMaskExecutedAction::EnqueueBigMapCompute {
                    previous_rect_256: *previous_rect_256,
                });
            }
            MapMaskTickEffect::ResetBigMapPreviousRect => {
                runtime.reset_map_mask_previous_big_map_rect(plan);
                executed_actions.push(MapMaskExecutedAction::ResetBigMapPreviousRect);
            }
            MapMaskTickEffect::EnqueueMiniMapCompute => {
                runtime.enqueue_map_mask_mini_map_compute(plan);
                executed_actions.push(MapMaskExecutedAction::EnqueueMiniMapCompute);
            }
            MapMaskTickEffect::ClearMiniMapViewport => {
                runtime.clear_map_mask_mini_map_viewport(plan);
                executed_actions.push(MapMaskExecutedAction::ClearMiniMapViewport);
            }
            MapMaskTickEffect::SkipTick(_) => {}
        }
    }

    MapMaskTickExecutionReport {
        observation,
        effects: reduction.effects,
        executed_actions,
        state: reduction.state,
    }
}

pub fn resolve_map_mask_big_map_viewport(
    rect_256: Rect,
    rule: &MapMaskBigMapRule,
) -> MapMaskBigMapViewportDecision {
    if rect_256.is_empty() {
        return MapMaskBigMapViewportDecision::Update {
            viewport_2048: Rect::empty(),
            accepted_rect_256: None,
        };
    }

    if rect_256.width < rule.reject_when_width_lt_and_height_lt.width as i32
        && rect_256.height < rule.reject_when_width_lt_and_height_lt.height as i32
    {
        return MapMaskBigMapViewportDecision::Reject {
            reason: MapMaskBigMapRectRejectReason::TooSmall,
            reset_previous_rect: true,
        };
    }

    if rect_256.width > rule.reject_when_width_gt_and_height_gt.width as i32
        && rect_256.height > rule.reject_when_width_gt_and_height_gt.height as i32
    {
        return MapMaskBigMapViewportDecision::Reject {
            reason: MapMaskBigMapRectRejectReason::TooLarge,
            reset_previous_rect: true,
        };
    }

    let scale = rule.layer_256_to_2048_scale as i32;
    MapMaskBigMapViewportDecision::Update {
        viewport_2048: Rect {
            x: rect_256.x * scale,
            y: rect_256.y * scale,
            width: rect_256.width * scale,
            height: rect_256.height * scale,
        },
        accepted_rect_256: Some(rect_256),
    }
}

pub fn resolve_map_mask_mini_map_viewport(
    mini_map_position: Option<Point>,
    rule: &MapMaskMiniMapRule,
) -> MapMaskMiniMapViewportDecision {
    let Some(point) = mini_map_position else {
        return MapMaskMiniMapViewportDecision::Clear;
    };
    let size = rule.viewport_size;
    MapMaskMiniMapViewportDecision::Update {
        viewport: MapMaskViewport {
            x: point.x as f64 - size / 2.0,
            y: point.y as f64 - size / 2.0,
            width: size,
            height: size,
        },
    }
}

fn point_provider(value: &str) -> MapMaskPointProvider {
    match value {
        "MihoyoMap" => MapMaskPointProvider::MihoyoMap,
        "KongyingTavern" => MapMaskPointProvider::KongyingTavern,
        "HoYoLab" => MapMaskPointProvider::HoYoLab,
        other => MapMaskPointProvider::Unknown(other.to_string()),
    }
}

fn template(name: &str, asset: &str, roi: Rect) -> MapMaskTemplateLocator {
    MapMaskTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold: 0.8,
        match_mode: TemplateMatchMode::CCoeffNormed,
        use_3_channels: false,
        draw_on_window: false,
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / MAP_MASK_DEFAULT_CAPTURE_WIDTH as i64) as i32
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    let capture = value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .unwrap_or(value);
    let width = u32_member(capture, ["width", "Width", "captureWidth", "CaptureWidth"])?;
    let height = u32_member(
        capture,
        ["height", "Height", "captureHeight", "CaptureHeight"],
    )?;
    Some(Size::new(width, height))
}

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}
