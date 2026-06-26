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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoTrackPhase {
    Startup,
    MainUi,
    MissionText,
    Teleport,
    Tracking,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
        executor_ready: false,
        pending_native: vec![
            "TaskSemaphore, CancellationContext, live capture, SystemControl window activation, and DrawContent cleanup".to_string(),
            "Paimon/main-ui template matching, Paddle OCR for mission text and saved distance rect, QuickTeleport template matching, and Bv big-map/main-ui detection".to_string(),
            "Simulation input dispatch for quest menu, tracking toggle clicks, V navigation, mouse steering, W key state, and cancellation-aware tracking loop".to_string(),
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
