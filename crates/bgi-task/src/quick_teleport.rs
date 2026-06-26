use bgi_core::QuickTeleportConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::Result;

pub const QUICK_TELEPORT_TASK_KEY: &str = "QuickTeleport";
pub const QUICK_TELEPORT_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const QUICK_TELEPORT_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const QUICK_TELEPORT_GO_TELEPORT: &str = "QuickTeleport:GoTeleport.png";
pub const QUICK_TELEPORT_MAP_SCALE_BUTTON: &str = "QuickTeleport:MapScaleButton.png";
pub const QUICK_TELEPORT_MAP_CLOSE_BUTTON: &str = "QuickTeleport:MapCloseButton.png";
pub const QUICK_TELEPORT_MAP_SETTINGS_BUTTON: &str = "QuickTeleport:MapSettingsButton.png";
pub const QUICK_TELEPORT_MAP_CHOOSE: &str = "QuickTeleport:MapChoose.png";
pub const QUICK_TELEPORT_UNDERGROUND_SWITCH: &str = "QuickTeleport:MapUndergroundSwitchButton.png";
pub const QUICK_TELEPORT_UNDERGROUND_TO_GROUND: &str =
    "QuickTeleport:MapUndergroundToGroundButton.png";
pub const QUICK_TELEPORT_TRANSPARENT_BACKGROUND: &str =
    "QuickTeleport:TeleportTransparentBackground.png";
pub const QUICK_TELEPORT_ICON_TELEPORT_WAYPOINT: &str = "QuickTeleport:TeleportWaypoint.png";
pub const QUICK_TELEPORT_ICON_STATUE_OF_THE_SEVEN: &str = "QuickTeleport:StatueOfTheSeven.png";
pub const QUICK_TELEPORT_ICON_DOMAIN: &str = "QuickTeleport:Domain.png";
pub const QUICK_TELEPORT_ICON_DOMAIN2: &str = "QuickTeleport:Domain2.png";
pub const QUICK_TELEPORT_ICON_OBSIDIAN_TOTEM_POLE: &str = "QuickTeleport:ObsidianTotemPole.png";
pub const QUICK_TELEPORT_ICON_PORTABLE_WAYPOINT: &str = "QuickTeleport:PortableWaypoint.png";
pub const QUICK_TELEPORT_ICON_MANSION: &str = "QuickTeleport:Mansion.png";
pub const QUICK_TELEPORT_ICON_SUB_SPACE_WAYPOINT: &str = "QuickTeleport:SubSpaceWaypoint.png";
pub const QUICK_TELEPORT_ICON_NOD_KRAI_MEETING_POINT: &str =
    "QuickTeleport:NodKraiMeetingPoint.png";
pub const QUICK_TELEPORT_ICON_TABLET_OF_TONA: &str = "QuickTeleport:TabletOfTona.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickTeleportExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: QuickTeleportConfigRule,
    pub throttle_rule: QuickTeleportThrottleRule,
    pub hotkey_rule: QuickTeleportHotkeyRule,
    pub locators: QuickTeleportLocators,
    pub multi_match_rule: QuickTeleportMultiMatchRule,
    pub text_ocr_rule: QuickTeleportTextOcrRule,
    pub steps: Vec<QuickTeleportTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickTeleportExecutionConfig {
    pub capture_size: Size,
    pub quick_teleport_config: QuickTeleportConfig,
    pub quick_teleport_tick_hotkey: Option<String>,
}

impl Default for QuickTeleportExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                QUICK_TELEPORT_DEFAULT_CAPTURE_WIDTH,
                QUICK_TELEPORT_DEFAULT_CAPTURE_HEIGHT,
            ),
            quick_teleport_config: QuickTeleportConfig::default(),
            quick_teleport_tick_hotkey: None,
        }
    }
}

impl QuickTeleportExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }

        let quick_teleport_value = value
            .get("quickTeleportConfig")
            .or_else(|| value.get("QuickTeleportConfig"))
            .or_else(|| value.get("quick_teleport_config"))
            .unwrap_or(value);
        config.quick_teleport_config =
            serde_json::from_value(quick_teleport_value.clone()).unwrap_or_default();

        config.quick_teleport_tick_hotkey = string_member(
            value,
            [
                "quickTeleportTickHotkey",
                "QuickTeleportTickHotkey",
                "quick_teleport_tick_hotkey",
            ],
        )
        .or_else(|| {
            value
                .get("hotKeyConfig")
                .or_else(|| value.get("HotKeyConfig"))
                .or_else(|| value.get("hot_key_config"))
                .and_then(|hotkey_config| {
                    string_member(
                        hotkey_config,
                        [
                            "quickTeleportTickHotkey",
                            "QuickTeleportTickHotkey",
                            "quick_teleport_tick_hotkey",
                        ],
                    )
                })
        });

        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportConfigRule {
    pub enabled: bool,
    pub teleport_list_click_delay_ms: u64,
    pub wait_teleport_panel_delay_ms: u64,
    pub hotkey_tp_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportThrottleRule {
    pub tick_interval_ms: u64,
    pub log_click_option_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportHotkeyRule {
    pub hotkey_tp_enabled: bool,
    pub configured_tick_hotkey: Option<String>,
    pub requires_pressed: bool,
    pub supports_keyboard_hook: bool,
    pub supports_mouse_hook: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickTeleportLocators {
    pub map_choose_icon_roi: Rect,
    pub map_choose_icon_templates: Vec<QuickTeleportTemplateLocator>,
    pub teleport_button: QuickTeleportTemplateLocator,
    pub map_scale_button: QuickTeleportTemplateLocator,
    pub map_close_button: QuickTeleportTemplateLocator,
    pub map_settings_button: QuickTeleportTemplateLocator,
    pub map_choose: QuickTeleportTemplateLocator,
    pub map_underground_switch_button: QuickTeleportTemplateLocator,
    pub map_underground_to_ground_button: QuickTeleportTemplateLocator,
    pub transparent_background_asset: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickTeleportTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub use_grey_template_for_multi_match: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuickTeleportMultiMatchRule {
    pub hdr_threshold: f64,
    pub standard_threshold: f64,
    pub sort_candidates_by_y_ascending: bool,
    pub click_first_valid_candidate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportTextOcrRule {
    pub text_region_width: i32,
    pub text_region_y_offset: i32,
    pub text_region_height_padding: i32,
    pub standard_capture_lower_bgr: QuickTeleportBgrColor,
    pub standard_capture_upper_bgr: QuickTeleportBgrColor,
    pub hdr_uses_plain_ocr: bool,
    pub standard_uses_color_range_and_ocr: bool,
    pub skip_empty_text: bool,
    pub skip_single_character_text: bool,
    pub strip_log_marker: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct QuickTeleportBgrColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportTickStep {
    pub phase: QuickTeleportTickPhase,
    pub condition: QuickTeleportTickCondition,
    pub action: QuickTeleportTickAction,
}

impl QuickTeleportTickStep {
    fn new(
        phase: QuickTeleportTickPhase,
        condition: QuickTeleportTickCondition,
        action: QuickTeleportTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickTeleportTickPhase {
    Throttle,
    HotkeyGate,
    BigMapGate,
    TeleportButton,
    SelectionGuard,
    MapChooseList,
    TeleportPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickTeleportTickCondition {
    WhenTickIntervalNotElapsed,
    WhenHotkeyGateActiveAndNotPressed,
    WhenNotBigMapUi,
    WhenTeleportButtonDetected,
    WhenMapCloseButtonDetected,
    WhenMapChooseButtonDetected,
    WhenTeleportButtonMissingAndMapChooseIconFound,
    WhenCandidateTextValid,
    WhenCandidateClicked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickTeleportTickAction {
    SkipTick,
    DetectBigMapUi,
    ClickTeleportButton,
    ReturnWithoutClick,
    MultiMatchMapChooseIcons,
    OcrCandidateTextRegion,
    SleepTeleportListClickDelay,
    ClickCandidateTextRegion,
    SleepWaitTeleportPanelDelay,
    RecheckTeleportButtonWithFreshCapture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportMapChooseCandidate {
    pub icon_rect: Rect,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportDecisionInput {
    pub elapsed_since_previous_tick_ms: u64,
    pub hotkey_pressed: bool,
    pub is_big_map_ui: bool,
    pub teleport_button_detected: bool,
    pub map_close_button_detected: bool,
    pub map_choose_button_detected: bool,
    pub map_choose_candidates: Vec<QuickTeleportMapChooseCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportDecisionReport {
    pub sorted_candidates: Vec<QuickTeleportMapChooseCandidate>,
    pub action: QuickTeleportDecisionAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickTeleportDecisionAction {
    Skip {
        reason: QuickTeleportSkipReason,
    },
    ClickTeleportButton,
    ClickCandidate {
        source_index: usize,
        text: String,
        log_text: String,
        icon_rect: Rect,
        text_rect: Rect,
        teleport_list_click_delay_ms: u64,
        wait_teleport_panel_delay_ms: u64,
        recheck_teleport_button_after_click: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuickTeleportSkipReason {
    Disabled,
    TickIntervalNotElapsed,
    HotkeyNotPressed,
    NotBigMapUi,
    MapCloseButtonDetected,
    MapChooseButtonDetected,
    NoValidCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuickTeleportTickExecutionReport {
    pub task_key: String,
    pub decision: QuickTeleportDecisionReport,
    pub executed_actions: Vec<QuickTeleportExecutedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum QuickTeleportExecutedAction {
    ClickTeleportButton {
        asset: String,
    },
    Delay {
        duration_ms: u64,
    },
    ClickCandidateTextRegion {
        source_index: usize,
        text: String,
        log_text: String,
        text_rect: Rect,
    },
    RecheckTeleportButton {
        detected: bool,
    },
    ClickRecheckedTeleportButton {
        asset: String,
    },
}

pub trait QuickTeleportRuntime {
    fn observe_quick_teleport_tick(
        &mut self,
        plan: &QuickTeleportExecutionPlan,
    ) -> Result<QuickTeleportDecisionInput>;

    fn click_quick_teleport_button(&mut self, locator: &QuickTeleportTemplateLocator)
        -> Result<()>;

    fn delay_quick_teleport(&mut self, duration_ms: u64) -> Result<()>;

    fn click_quick_teleport_candidate_text_region(
        &mut self,
        text_rect: Rect,
        text: &str,
    ) -> Result<()>;

    fn recheck_quick_teleport_button(
        &mut self,
        locator: &QuickTeleportTemplateLocator,
    ) -> Result<bool>;
}

pub fn plan_quick_teleport(config: QuickTeleportExecutionConfig) -> QuickTeleportExecutionPlan {
    let capture_size = config.capture_size;
    let quick_config = config.quick_teleport_config;
    let locators = quick_teleport_locators(capture_size);
    let hotkey_tp_enabled = quick_config.hotkey_tp_enabled;
    let configured_tick_hotkey = config
        .quick_teleport_tick_hotkey
        .filter(|value| !value.trim().is_empty());

    QuickTeleportExecutionPlan {
        task_key: QUICK_TELEPORT_TASK_KEY.to_string(),
        display_name: "Quick Teleport".to_string(),
        capture_size,
        config_rule: QuickTeleportConfigRule {
            enabled: quick_config.enabled,
            teleport_list_click_delay_ms: quick_config.teleport_list_click_delay,
            wait_teleport_panel_delay_ms: quick_config.wait_teleport_panel_delay,
            hotkey_tp_enabled,
        },
        throttle_rule: QuickTeleportThrottleRule {
            tick_interval_ms: 300,
            log_click_option_interval_ms: 500,
        },
        hotkey_rule: QuickTeleportHotkeyRule {
            hotkey_tp_enabled,
            requires_pressed: hotkey_tp_enabled && configured_tick_hotkey.is_some(),
            configured_tick_hotkey,
            supports_keyboard_hook: true,
            supports_mouse_hook: true,
        },
        locators,
        multi_match_rule: QuickTeleportMultiMatchRule {
            hdr_threshold: 0.7,
            standard_threshold: 0.8,
            sort_candidates_by_y_ascending: true,
            click_first_valid_candidate: true,
        },
        text_ocr_rule: QuickTeleportTextOcrRule {
            text_region_width: 200,
            text_region_y_offset: -8,
            text_region_height_padding: 16,
            standard_capture_lower_bgr: QuickTeleportBgrColor {
                b: 249,
                g: 249,
                r: 249,
            },
            standard_capture_upper_bgr: QuickTeleportBgrColor {
                b: 255,
                g: 255,
                r: 255,
            },
            hdr_uses_plain_ocr: true,
            standard_uses_color_range_and_ocr: true,
            skip_empty_text: true,
            skip_single_character_text: true,
            strip_log_marker: ">".to_string(),
        },
        steps: quick_teleport_steps(),
        executor_ready: true,
        pending_native: vec![
            "live adapter for BigMap capture and BV UI recognition".to_string(),
            "live adapter for OpenCV multi-template matching over grey map icon ROI".to_string(),
            "live adapter for HDR-aware OCR and color-range OCR for candidate text".to_string(),
            "live adapter for keyboard/mouse hook state for optional quick-teleport hotkey gate"
                .to_string(),
            "live adapter for mouse click dispatch on teleport button and candidate text regions"
                .to_string(),
            "live adapter for fresh capture after waiting for the teleport panel".to_string(),
        ],
    }
}

pub fn execute_quick_teleport_tick_plan<R>(
    plan: &QuickTeleportExecutionPlan,
    runtime: &mut R,
) -> Result<QuickTeleportTickExecutionReport>
where
    R: QuickTeleportRuntime,
{
    let input = runtime.observe_quick_teleport_tick(plan)?;
    let decision = decide_quick_teleport_tick(plan, input);
    let mut executed_actions = Vec::new();

    match &decision.action {
        QuickTeleportDecisionAction::Skip { .. } => {}
        QuickTeleportDecisionAction::ClickTeleportButton => {
            runtime.click_quick_teleport_button(&plan.locators.teleport_button)?;
            executed_actions.push(QuickTeleportExecutedAction::ClickTeleportButton {
                asset: plan.locators.teleport_button.asset.clone(),
            });
        }
        QuickTeleportDecisionAction::ClickCandidate {
            source_index,
            text,
            log_text,
            text_rect,
            teleport_list_click_delay_ms,
            wait_teleport_panel_delay_ms,
            recheck_teleport_button_after_click,
            ..
        } => {
            runtime.delay_quick_teleport(*teleport_list_click_delay_ms)?;
            executed_actions.push(QuickTeleportExecutedAction::Delay {
                duration_ms: *teleport_list_click_delay_ms,
            });
            runtime.click_quick_teleport_candidate_text_region(*text_rect, text)?;
            executed_actions.push(QuickTeleportExecutedAction::ClickCandidateTextRegion {
                source_index: *source_index,
                text: text.clone(),
                log_text: log_text.clone(),
                text_rect: *text_rect,
            });
            runtime.delay_quick_teleport(*wait_teleport_panel_delay_ms)?;
            executed_actions.push(QuickTeleportExecutedAction::Delay {
                duration_ms: *wait_teleport_panel_delay_ms,
            });

            if *recheck_teleport_button_after_click {
                let detected =
                    runtime.recheck_quick_teleport_button(&plan.locators.teleport_button)?;
                executed_actions
                    .push(QuickTeleportExecutedAction::RecheckTeleportButton { detected });
                if detected {
                    runtime.click_quick_teleport_button(&plan.locators.teleport_button)?;
                    executed_actions.push(
                        QuickTeleportExecutedAction::ClickRecheckedTeleportButton {
                            asset: plan.locators.teleport_button.asset.clone(),
                        },
                    );
                }
            }
        }
    }

    Ok(QuickTeleportTickExecutionReport {
        task_key: plan.task_key.clone(),
        decision,
        executed_actions,
    })
}

pub fn decide_quick_teleport_tick(
    plan: &QuickTeleportExecutionPlan,
    input: QuickTeleportDecisionInput,
) -> QuickTeleportDecisionReport {
    let sorted_candidates =
        sorted_quick_teleport_candidates(&input.map_choose_candidates, &plan.multi_match_rule);

    if !plan.config_rule.enabled {
        return quick_teleport_skip(sorted_candidates, QuickTeleportSkipReason::Disabled);
    }
    if input.elapsed_since_previous_tick_ms <= plan.throttle_rule.tick_interval_ms {
        return quick_teleport_skip(
            sorted_candidates,
            QuickTeleportSkipReason::TickIntervalNotElapsed,
        );
    }
    if plan.hotkey_rule.requires_pressed && !input.hotkey_pressed {
        return quick_teleport_skip(sorted_candidates, QuickTeleportSkipReason::HotkeyNotPressed);
    }
    if !input.is_big_map_ui {
        return quick_teleport_skip(sorted_candidates, QuickTeleportSkipReason::NotBigMapUi);
    }
    if input.teleport_button_detected {
        return QuickTeleportDecisionReport {
            sorted_candidates,
            action: QuickTeleportDecisionAction::ClickTeleportButton,
        };
    }
    if input.map_close_button_detected {
        return quick_teleport_skip(
            sorted_candidates,
            QuickTeleportSkipReason::MapCloseButtonDetected,
        );
    }
    if input.map_choose_button_detected {
        return quick_teleport_skip(
            sorted_candidates,
            QuickTeleportSkipReason::MapChooseButtonDetected,
        );
    }

    for candidate in &sorted_candidates {
        if !quick_teleport_candidate_text_is_valid(&candidate.text, &plan.text_ocr_rule) {
            continue;
        }
        let selected_candidate = candidate.clone();
        let source_index = input
            .map_choose_candidates
            .iter()
            .position(|source| source == &selected_candidate)
            .unwrap_or(0);
        let action = QuickTeleportDecisionAction::ClickCandidate {
            source_index,
            text: selected_candidate.text.clone(),
            log_text: selected_candidate
                .text
                .replace(plan.text_ocr_rule.strip_log_marker.as_str(), ""),
            icon_rect: selected_candidate.icon_rect,
            text_rect: quick_teleport_candidate_text_rect(
                selected_candidate.icon_rect,
                &plan.locators,
                &plan.text_ocr_rule,
            ),
            teleport_list_click_delay_ms: plan.config_rule.teleport_list_click_delay_ms,
            wait_teleport_panel_delay_ms: plan.config_rule.wait_teleport_panel_delay_ms,
            recheck_teleport_button_after_click: true,
        };
        return QuickTeleportDecisionReport {
            sorted_candidates,
            action,
        };
    }

    quick_teleport_skip(sorted_candidates, QuickTeleportSkipReason::NoValidCandidate)
}

fn sorted_quick_teleport_candidates(
    candidates: &[QuickTeleportMapChooseCandidate],
    rule: &QuickTeleportMultiMatchRule,
) -> Vec<QuickTeleportMapChooseCandidate> {
    let mut sorted = candidates.to_vec();
    if rule.sort_candidates_by_y_ascending {
        sorted.sort_by_key(|candidate| candidate.icon_rect.y);
    }
    sorted
}

fn quick_teleport_candidate_text_is_valid(text: &str, rule: &QuickTeleportTextOcrRule) -> bool {
    let text = text.trim();
    if rule.skip_empty_text && text.is_empty() {
        return false;
    }
    if rule.skip_single_character_text && text.chars().count() == 1 {
        return false;
    }
    true
}

fn quick_teleport_candidate_text_rect(
    icon_rect: Rect,
    locators: &QuickTeleportLocators,
    rule: &QuickTeleportTextOcrRule,
) -> Rect {
    Rect {
        x: locators.map_choose_icon_roi.x + icon_rect.x + icon_rect.width,
        y: locators.map_choose_icon_roi.y + icon_rect.y + rule.text_region_y_offset,
        width: rule.text_region_width,
        height: icon_rect.height + rule.text_region_height_padding,
    }
}

fn quick_teleport_skip(
    sorted_candidates: Vec<QuickTeleportMapChooseCandidate>,
    reason: QuickTeleportSkipReason,
) -> QuickTeleportDecisionReport {
    QuickTeleportDecisionReport {
        sorted_candidates,
        action: QuickTeleportDecisionAction::Skip { reason },
    }
}

fn quick_teleport_locators(capture_size: Size) -> QuickTeleportLocators {
    let map_choose_icon_roi = map_choose_icon_roi(capture_size);
    QuickTeleportLocators {
        map_choose_icon_roi,
        map_choose_icon_templates: map_choose_icon_templates(map_choose_icon_roi),
        teleport_button: template_locator(
            "GoTeleport",
            QUICK_TELEPORT_GO_TELEPORT,
            Rect {
                x: scaled(1440, capture_size),
                y: capture_size.height as i32 - scaled(120, capture_size),
                width: scaled(100, capture_size),
                height: scaled(120, capture_size),
            },
            0.8,
            false,
            false,
        ),
        map_scale_button: template_locator(
            "MapScaleButton",
            QUICK_TELEPORT_MAP_SCALE_BUTTON,
            Rect {
                x: scaled(30, capture_size),
                y: scaled(440, capture_size),
                width: scaled(40, capture_size),
                height: scaled(200, capture_size),
            },
            0.8,
            false,
            false,
        ),
        map_close_button: template_locator(
            "MapCloseButton",
            QUICK_TELEPORT_MAP_CLOSE_BUTTON,
            Rect {
                x: capture_size.width as i32 - scaled(107, capture_size),
                y: scaled(19, capture_size),
                width: scaled(58, capture_size),
                height: scaled(58, capture_size),
            },
            0.8,
            false,
            false,
        ),
        map_settings_button: template_locator(
            "MapSettingsButton",
            QUICK_TELEPORT_MAP_SETTINGS_BUTTON,
            Rect {
                x: scaled(25, capture_size),
                y: scaled(990, capture_size),
                width: scaled(58, capture_size),
                height: scaled(62, capture_size),
            },
            0.8,
            false,
            false,
        ),
        map_choose: template_locator(
            "MapChoose",
            QUICK_TELEPORT_MAP_CHOOSE,
            Rect {
                x: capture_size.width as i32 - scaled(480, capture_size),
                y: 0,
                width: scaled(100, capture_size),
                height: scaled(70, capture_size),
            },
            0.8,
            false,
            false,
        ),
        map_underground_switch_button: template_locator(
            "MapUndergroundSwitchButton",
            QUICK_TELEPORT_UNDERGROUND_SWITCH,
            underground_button_roi(capture_size),
            0.8,
            true,
            false,
        ),
        map_underground_to_ground_button: template_locator(
            "MapUndergroundToGroundButton",
            QUICK_TELEPORT_UNDERGROUND_TO_GROUND,
            underground_button_roi(capture_size),
            0.8,
            false,
            false,
        ),
        transparent_background_asset: QUICK_TELEPORT_TRANSPARENT_BACKGROUND.to_string(),
    }
}

fn map_choose_icon_templates(roi: Rect) -> Vec<QuickTeleportTemplateLocator> {
    [
        (
            "TeleportWaypoint.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_TELEPORT_WAYPOINT,
            0.7,
        ),
        (
            "StatueOfTheSeven.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_STATUE_OF_THE_SEVEN,
            0.8,
        ),
        ("Domain.pngMapChooseIcon", QUICK_TELEPORT_ICON_DOMAIN, 0.8),
        ("Domain2.pngMapChooseIcon", QUICK_TELEPORT_ICON_DOMAIN2, 0.8),
        (
            "ObsidianTotemPole.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_OBSIDIAN_TOTEM_POLE,
            0.8,
        ),
        (
            "PortableWaypoint.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_PORTABLE_WAYPOINT,
            0.8,
        ),
        ("Mansion.pngMapChooseIcon", QUICK_TELEPORT_ICON_MANSION, 0.8),
        (
            "SubSpaceWaypoint.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_SUB_SPACE_WAYPOINT,
            0.8,
        ),
        (
            "NodKraiMeetingPoint.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_NOD_KRAI_MEETING_POINT,
            0.8,
        ),
        (
            "TabletOfTona.pngMapChooseIcon",
            QUICK_TELEPORT_ICON_TABLET_OF_TONA,
            0.8,
        ),
    ]
    .into_iter()
    .map(|(name, asset, threshold)| template_locator(name, asset, roi, threshold, false, true))
    .collect()
}

fn quick_teleport_steps() -> Vec<QuickTeleportTickStep> {
    vec![
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::Throttle,
            QuickTeleportTickCondition::WhenTickIntervalNotElapsed,
            QuickTeleportTickAction::SkipTick,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::HotkeyGate,
            QuickTeleportTickCondition::WhenHotkeyGateActiveAndNotPressed,
            QuickTeleportTickAction::SkipTick,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::BigMapGate,
            QuickTeleportTickCondition::WhenNotBigMapUi,
            QuickTeleportTickAction::DetectBigMapUi,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::TeleportButton,
            QuickTeleportTickCondition::WhenTeleportButtonDetected,
            QuickTeleportTickAction::ClickTeleportButton,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::SelectionGuard,
            QuickTeleportTickCondition::WhenMapCloseButtonDetected,
            QuickTeleportTickAction::ReturnWithoutClick,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::SelectionGuard,
            QuickTeleportTickCondition::WhenMapChooseButtonDetected,
            QuickTeleportTickAction::ReturnWithoutClick,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::MapChooseList,
            QuickTeleportTickCondition::WhenTeleportButtonMissingAndMapChooseIconFound,
            QuickTeleportTickAction::MultiMatchMapChooseIcons,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::MapChooseList,
            QuickTeleportTickCondition::WhenTeleportButtonMissingAndMapChooseIconFound,
            QuickTeleportTickAction::OcrCandidateTextRegion,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::MapChooseList,
            QuickTeleportTickCondition::WhenCandidateTextValid,
            QuickTeleportTickAction::SleepTeleportListClickDelay,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::MapChooseList,
            QuickTeleportTickCondition::WhenCandidateTextValid,
            QuickTeleportTickAction::ClickCandidateTextRegion,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::TeleportPanel,
            QuickTeleportTickCondition::WhenCandidateClicked,
            QuickTeleportTickAction::SleepWaitTeleportPanelDelay,
        ),
        QuickTeleportTickStep::new(
            QuickTeleportTickPhase::TeleportPanel,
            QuickTeleportTickCondition::WhenCandidateClicked,
            QuickTeleportTickAction::RecheckTeleportButtonWithFreshCapture,
        ),
    ]
}

fn template_locator(
    name: &str,
    asset: &str,
    roi: Rect,
    threshold: f64,
    use_3_channels: bool,
    use_grey_template_for_multi_match: bool,
) -> QuickTeleportTemplateLocator {
    QuickTeleportTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode: TemplateMatchMode::CCoeffNormed,
        use_3_channels,
        use_grey_template_for_multi_match,
        draw_on_window: false,
    }
}

fn map_choose_icon_roi(size: Size) -> Rect {
    Rect {
        x: scaled(1270, size),
        y: scaled(100, size),
        width: scaled(50, size),
        height: size.height as i32 - scaled(200, size),
    }
}

fn underground_button_roi(size: Size) -> Rect {
    Rect {
        x: size.width as i32 - scaled(120, size),
        y: scaled(250, size),
        width: scaled(90, size),
        height: scaled(570, size),
    }
}

fn scaled(value_1080p: i32, size: Size) -> i32 {
    ((value_1080p as i64 * size.width as i64) / QUICK_TELEPORT_DEFAULT_CAPTURE_WIDTH as i64) as i32
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

fn string_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<String> {
    let Value::Object(map) = value else {
        return None;
    };
    string_from_map(map, keys)
}

fn string_from_map<const N: usize>(map: &Map<String, Value>, keys: [&str; N]) -> Option<String> {
    keys.into_iter()
        .filter_map(|key| map.get(key))
        .find_map(Value::as_str)
        .map(ToOwned::to_owned)
}
