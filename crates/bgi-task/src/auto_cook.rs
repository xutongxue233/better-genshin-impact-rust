use bgi_core::AutoCookConfig;
use bgi_vision::{BgrImage, Rect, RgbPixel, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, TaskError};

pub const AUTO_COOK_TASK_KEY: &str = "AutoCook";
pub const AUTO_COOK_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_COOK_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_COOK_UI_CHECK_INTERVAL_MS: u64 = 400;
pub const AUTO_COOK_PEAK_MIN_COUNT_1080P: u32 = 600;
pub const AUTO_COOK_PEAK_TOLERANCE: u32 = 20;
pub const AUTO_COOK_PEAK_STABLE_FRAME_COUNT: u32 = 3;
pub const AUTO_COOK_TRIGGER_DROP_COUNT_1080P: u32 = 300;
pub const AUTO_COOK_VK_SPACE: u16 = 0x20;
pub const AUTO_COOK_WHITE_CONFIRM_PRE_CLICK_DELAY_MS: u64 = 500;
pub const AUTO_COOK_UI_COOK_ICON_ASSET: &str = "Common/Element:ui_left_top_cook_icon.png";
pub const AUTO_COOK_BTN_WHITE_RECOVER_ASSET: &str = "Common/Element:btn_white_recover.png";
pub const AUTO_COOK_BTN_WHITE_CONFIRM_ASSET: &str = "Common/Element:btn_white_confirm.png";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoCookExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub config_rule: AutoCookConfigRule,
    pub ui_rule: AutoCookUiRule,
    pub locators: AutoCookLocators,
    pub cook_bar_rule: AutoCookBarRule,
    pub peak_rule: AutoCookPeakRule,
    pub input_rule: AutoCookInputRule,
    pub steps: Vec<AutoCookTaskStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCookExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub auto_cook_config: AutoCookConfig,
}

impl Default for AutoCookExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_COOK_DEFAULT_CAPTURE_WIDTH,
                AUTO_COOK_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            auto_cook_config: AutoCookConfig::default(),
        }
    }
}

impl AutoCookExecutionConfig {
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

        let auto_cook_value = value
            .get("autoCookConfig")
            .or_else(|| value.get("AutoCookConfig"))
            .or_else(|| value.get("auto_cook_config"))
            .unwrap_or(value);
        config.auto_cook_config =
            serde_json::from_value(auto_cook_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookConfigRule {
    pub configured_check_interval_ms: u64,
    pub effective_check_interval_ms: u64,
    pub minimum_check_interval_ms: u64,
    pub stop_task_when_recover_button_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookUiRule {
    pub ui_check_interval_ms: u64,
    pub reset_peak_when_ui_state_changes: bool,
    pub reset_peak_when_not_in_cook_ui: bool,
    pub click_white_confirm_when_present: bool,
    pub white_confirm_pre_click_delay_ms: u64,
    pub reset_peak_after_white_confirm: bool,
    pub stop_when_recover_button_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoCookLocators {
    pub cook_icon: AutoCookTemplateLocator,
    pub white_recover_button: AutoCookTemplateLocator,
    pub white_confirm_button: AutoCookTemplateLocator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoCookTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub use_3_channels: bool,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookBarRule {
    pub cook_color_rect_1080p: Rect,
    pub scaled_cook_color_rect: Rect,
    pub target_rgb: RgbPixel,
    pub converts_capture_bgr_to_rgb_before_match: bool,
    pub exact_color_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookPeakRule {
    pub peak_min_count_1080p: u32,
    pub scaled_peak_min_count: u32,
    pub peak_tolerance: u32,
    pub peak_stable_frame_count: u32,
    pub trigger_drop_count_1080p: u32,
    pub scaled_trigger_drop_count: u32,
    pub reset_after_space_press: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookInputRule {
    pub trigger_key_vk: u16,
    pub trigger_key_name: String,
    pub action: AutoCookInputAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoCookInputAction {
    KeyPress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoCookTaskStep {
    pub phase: AutoCookTaskPhase,
    pub action: AutoCookTaskAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoCookTaskPhase {
    CaptureLoop,
    CookUiDetection,
    RecoverButtonGate,
    ConfirmButton,
    CookBarSampling,
    PeakTracking,
    Input,
    Delay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoCookTaskAction {
    CaptureFrame,
    DetectCookUiIcon,
    ResetPeakOnUiTransition,
    StopWhenRecoverButtonDetected,
    ClickWhiteConfirmButton,
    CountTargetCookColor,
    BuildStablePeak,
    PressSpaceWhenPeakDrops,
    DelayConfiguredInterval,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoCookPeakState {
    pub peak_color_count: Option<u32>,
    pub peak_candidate: Option<u32>,
    pub peak_candidate_stable_frames: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoCookPeakAction {
    None,
    BuiltPeak { peak: u32 },
    PressSpace { peak: u32, current: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoCookRuntimeFrame {
    pub now_ms: u64,
    pub in_cook_ui: bool,
    pub recover_button_detected: bool,
    pub white_confirm_button_detected: bool,
    pub target_color_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoCookPeakResetReason {
    CookUiTransition,
    NotInCookUi,
    WhiteConfirmClicked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoCookExecutionStatus {
    RuntimeEnded,
    IterationLimitReached,
    RecoverButtonDetected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoCookExecutionEvent {
    FrameCaptured {
        iteration: u32,
        now_ms: u64,
    },
    CookUiDetected {
        iteration: u32,
        in_cook_ui: bool,
    },
    PeakReset {
        iteration: u32,
        reason: AutoCookPeakResetReason,
    },
    RecoverButtonDetected {
        iteration: u32,
    },
    WhiteConfirmPreClickDelay {
        iteration: u32,
        duration_ms: u64,
    },
    WhiteConfirmClicked {
        iteration: u32,
    },
    ColorCounted {
        iteration: u32,
        count: u32,
    },
    PeakBuilt {
        iteration: u32,
        peak: u32,
    },
    SpacePressed {
        iteration: u32,
        peak: u32,
        current: u32,
        vk: u16,
    },
    Delay {
        iteration: u32,
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoCookExecutorState {
    pub in_cook_ui: bool,
    pub last_ui_check_ms: Option<u64>,
    pub peak_state: AutoCookPeakState,
    pub frames_processed: u32,
    pub space_press_count: u32,
    pub white_confirm_click_count: u32,
    pub delay_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoCookExecutionReport {
    pub task_key: String,
    pub status: AutoCookExecutionStatus,
    pub state: AutoCookExecutorState,
    pub events: Vec<AutoCookExecutionEvent>,
}

pub trait AutoCookRuntime {
    fn next_auto_cook_frame(&mut self) -> Result<Option<AutoCookRuntimeFrame>>;

    fn delay_auto_cook_white_confirm_pre_click(&mut self, duration_ms: u64) -> Result<()>;

    fn click_auto_cook_white_confirm(&mut self) -> Result<()>;

    fn press_auto_cook_key(&mut self, vk: u16) -> Result<()>;

    fn delay_auto_cook_loop(&mut self, duration_ms: u64) -> Result<()>;
}

pub fn plan_auto_cook(config: AutoCookExecutionConfig) -> AutoCookExecutionPlan {
    let check_interval_ms = config.auto_cook_config.check_interval_ms.max(1);
    let stop_task_when_recover_button_detected = config
        .auto_cook_config
        .stop_task_when_recover_button_detected;
    let cook_color_rect_1080p = Rect {
        x: 600,
        y: 660,
        width: 730,
        height: 190,
    };

    AutoCookExecutionPlan {
        task_key: AUTO_COOK_TASK_KEY.to_string(),
        display_name: "自动烹饪".to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        config_rule: AutoCookConfigRule {
            configured_check_interval_ms: config.auto_cook_config.check_interval_ms,
            effective_check_interval_ms: check_interval_ms,
            minimum_check_interval_ms: 1,
            stop_task_when_recover_button_detected,
        },
        ui_rule: AutoCookUiRule {
            ui_check_interval_ms: AUTO_COOK_UI_CHECK_INTERVAL_MS,
            reset_peak_when_ui_state_changes: true,
            reset_peak_when_not_in_cook_ui: true,
            click_white_confirm_when_present: true,
            white_confirm_pre_click_delay_ms: AUTO_COOK_WHITE_CONFIRM_PRE_CLICK_DELAY_MS,
            reset_peak_after_white_confirm: true,
            stop_when_recover_button_detected: stop_task_when_recover_button_detected,
        },
        locators: AutoCookLocators {
            cook_icon: AutoCookTemplateLocator {
                name: "UiLeftTopCookIcon".to_string(),
                asset: AUTO_COOK_UI_COOK_ICON_ASSET.to_string(),
                roi: Some(scale_rect(
                    Rect {
                        x: 0,
                        y: 0,
                        width: 150,
                        height: 120,
                    },
                    config.asset_scale,
                )),
                threshold: 0.8,
                match_mode: TemplateMatchMode::CCoeffNormed,
                use_3_channels: false,
                draw_on_window: false,
            },
            white_recover_button: AutoCookTemplateLocator {
                name: "BtnWhiteRecover".to_string(),
                asset: AUTO_COOK_BTN_WHITE_RECOVER_ASSET.to_string(),
                roi: Some(scale_rect(
                    Rect {
                        x: 580,
                        y: 950,
                        width: 90,
                        height: 95,
                    },
                    config.asset_scale,
                )),
                threshold: 0.8,
                match_mode: TemplateMatchMode::CCoeffNormed,
                use_3_channels: true,
                draw_on_window: false,
            },
            white_confirm_button: AutoCookTemplateLocator {
                name: "BtnWhiteConfirm".to_string(),
                asset: AUTO_COOK_BTN_WHITE_CONFIRM_ASSET.to_string(),
                roi: None,
                threshold: 0.8,
                match_mode: TemplateMatchMode::CCoeffNormed,
                use_3_channels: true,
                draw_on_window: false,
            },
        },
        cook_bar_rule: AutoCookBarRule {
            cook_color_rect_1080p,
            scaled_cook_color_rect: scale_rect(cook_color_rect_1080p, config.asset_scale),
            target_rgb: RgbPixel {
                r: 255,
                g: 192,
                b: 64,
            },
            converts_capture_bgr_to_rgb_before_match: true,
            exact_color_match: true,
        },
        peak_rule: AutoCookPeakRule {
            peak_min_count_1080p: AUTO_COOK_PEAK_MIN_COUNT_1080P,
            scaled_peak_min_count: scale_count(AUTO_COOK_PEAK_MIN_COUNT_1080P, config.asset_scale),
            peak_tolerance: AUTO_COOK_PEAK_TOLERANCE,
            peak_stable_frame_count: AUTO_COOK_PEAK_STABLE_FRAME_COUNT,
            trigger_drop_count_1080p: AUTO_COOK_TRIGGER_DROP_COUNT_1080P,
            scaled_trigger_drop_count: scale_count(
                AUTO_COOK_TRIGGER_DROP_COUNT_1080P,
                config.asset_scale,
            ),
            reset_after_space_press: true,
        },
        input_rule: AutoCookInputRule {
            trigger_key_vk: AUTO_COOK_VK_SPACE,
            trigger_key_name: "Space".to_string(),
            action: AutoCookInputAction::KeyPress,
        },
        steps: auto_cook_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop manual command and generic script-dispatcher live route cover BitBlt capture, ElementAssets template matching, white-confirm click, SendInput Space, exact BGR/RGB color counting, and cancellation-aware delays; legacy desktop hotkey/dispatcher entrypoints remain to be retired".to_string(),
        ],
    }
}

pub fn count_auto_cook_target_color(
    image: &BgrImage,
    cook_color_rect: Rect,
    target_rgb: RgbPixel,
) -> Result<u32> {
    let region = cook_color_rect
        .clamp_to(image.size)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let mut count = 0;
    for y in region.y..region.bottom() {
        for x in region.x..region.right() {
            if image.rgb_pixel_at(x as u32, y as u32) == Some(target_rgb) {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn update_auto_cook_peak_state(
    state: &mut AutoCookPeakState,
    current_color_count: u32,
    peak_rule: &AutoCookPeakRule,
) -> AutoCookPeakAction {
    if let Some(peak) = state.peak_color_count {
        if current_color_count <= peak.saturating_sub(peak_rule.scaled_trigger_drop_count) {
            *state = AutoCookPeakState::default();
            return AutoCookPeakAction::PressSpace {
                peak,
                current: current_color_count,
            };
        }
        return AutoCookPeakAction::None;
    }

    if current_color_count <= peak_rule.scaled_peak_min_count {
        state.peak_candidate = None;
        state.peak_candidate_stable_frames = 0;
        return AutoCookPeakAction::None;
    }

    let Some(candidate) = state.peak_candidate else {
        state.peak_candidate = Some(current_color_count);
        state.peak_candidate_stable_frames = 1;
        return AutoCookPeakAction::None;
    };

    if candidate.abs_diff(current_color_count) <= peak_rule.peak_tolerance {
        let peak = candidate.max(current_color_count);
        state.peak_candidate = Some(peak);
        state.peak_candidate_stable_frames += 1;
        if state.peak_candidate_stable_frames >= peak_rule.peak_stable_frame_count
            && peak > peak_rule.scaled_peak_min_count
        {
            state.peak_color_count = Some(peak);
            state.peak_candidate = None;
            state.peak_candidate_stable_frames = 0;
            return AutoCookPeakAction::BuiltPeak { peak };
        }
        return AutoCookPeakAction::None;
    }

    state.peak_candidate = Some(current_color_count);
    state.peak_candidate_stable_frames = 1;
    AutoCookPeakAction::None
}

pub fn execute_auto_cook_plan<R>(
    plan: &AutoCookExecutionPlan,
    runtime: &mut R,
    max_iterations: u32,
) -> Result<AutoCookExecutionReport>
where
    R: AutoCookRuntime,
{
    let mut state = AutoCookExecutorState::default();
    let mut events = Vec::new();

    loop {
        if max_iterations > 0 && state.frames_processed >= max_iterations {
            return Ok(AutoCookExecutionReport {
                task_key: plan.task_key.clone(),
                status: AutoCookExecutionStatus::IterationLimitReached,
                state,
                events,
            });
        }

        let Some(frame) = runtime.next_auto_cook_frame()? else {
            return Ok(AutoCookExecutionReport {
                task_key: plan.task_key.clone(),
                status: AutoCookExecutionStatus::RuntimeEnded,
                state,
                events,
            });
        };

        let iteration = state.frames_processed;
        state.frames_processed += 1;
        events.push(AutoCookExecutionEvent::FrameCaptured {
            iteration,
            now_ms: frame.now_ms,
        });

        let ui_check_due = !state.in_cook_ui
            || state
                .last_ui_check_ms
                .map(|last| frame.now_ms.saturating_sub(last) >= plan.ui_rule.ui_check_interval_ms)
                .unwrap_or(true);

        if ui_check_due {
            if frame.in_cook_ui != state.in_cook_ui && plan.ui_rule.reset_peak_when_ui_state_changes
            {
                reset_auto_cook_peak_state(
                    &mut state,
                    &mut events,
                    iteration,
                    AutoCookPeakResetReason::CookUiTransition,
                );
            }

            state.in_cook_ui = frame.in_cook_ui;
            state.last_ui_check_ms = Some(frame.now_ms);
            events.push(AutoCookExecutionEvent::CookUiDetected {
                iteration,
                in_cook_ui: state.in_cook_ui,
            });

            if !state.in_cook_ui {
                if plan.ui_rule.reset_peak_when_not_in_cook_ui {
                    reset_auto_cook_peak_state(
                        &mut state,
                        &mut events,
                        iteration,
                        AutoCookPeakResetReason::NotInCookUi,
                    );
                }
            } else {
                if plan.ui_rule.stop_when_recover_button_detected && frame.recover_button_detected {
                    events.push(AutoCookExecutionEvent::RecoverButtonDetected { iteration });
                    return Ok(AutoCookExecutionReport {
                        task_key: plan.task_key.clone(),
                        status: AutoCookExecutionStatus::RecoverButtonDetected,
                        state,
                        events,
                    });
                }

                if plan.ui_rule.click_white_confirm_when_present
                    && frame.white_confirm_button_detected
                {
                    if plan.ui_rule.white_confirm_pre_click_delay_ms > 0 {
                        runtime.delay_auto_cook_white_confirm_pre_click(
                            plan.ui_rule.white_confirm_pre_click_delay_ms,
                        )?;
                        events.push(AutoCookExecutionEvent::WhiteConfirmPreClickDelay {
                            iteration,
                            duration_ms: plan.ui_rule.white_confirm_pre_click_delay_ms,
                        });
                    }
                    runtime.click_auto_cook_white_confirm()?;
                    state.white_confirm_click_count += 1;
                    events.push(AutoCookExecutionEvent::WhiteConfirmClicked { iteration });
                    if plan.ui_rule.reset_peak_after_white_confirm {
                        reset_auto_cook_peak_state(
                            &mut state,
                            &mut events,
                            iteration,
                            AutoCookPeakResetReason::WhiteConfirmClicked,
                        );
                    }
                }
            }
        }

        if state.in_cook_ui {
            events.push(AutoCookExecutionEvent::ColorCounted {
                iteration,
                count: frame.target_color_count,
            });
            match update_auto_cook_peak_state(
                &mut state.peak_state,
                frame.target_color_count,
                &plan.peak_rule,
            ) {
                AutoCookPeakAction::None => {}
                AutoCookPeakAction::BuiltPeak { peak } => {
                    events.push(AutoCookExecutionEvent::PeakBuilt { iteration, peak });
                }
                AutoCookPeakAction::PressSpace { peak, current } => {
                    runtime.press_auto_cook_key(plan.input_rule.trigger_key_vk)?;
                    state.space_press_count += 1;
                    events.push(AutoCookExecutionEvent::SpacePressed {
                        iteration,
                        peak,
                        current,
                        vk: plan.input_rule.trigger_key_vk,
                    });
                }
            }
        }

        runtime.delay_auto_cook_loop(plan.config_rule.effective_check_interval_ms)?;
        state.delay_count += 1;
        events.push(AutoCookExecutionEvent::Delay {
            iteration,
            duration_ms: plan.config_rule.effective_check_interval_ms,
        });
    }
}

fn reset_auto_cook_peak_state(
    state: &mut AutoCookExecutorState,
    events: &mut Vec<AutoCookExecutionEvent>,
    iteration: u32,
    reason: AutoCookPeakResetReason,
) {
    state.peak_state = AutoCookPeakState::default();
    events.push(AutoCookExecutionEvent::PeakReset { iteration, reason });
}

fn auto_cook_steps() -> Vec<AutoCookTaskStep> {
    vec![
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::CaptureLoop,
            action: AutoCookTaskAction::CaptureFrame,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::CookUiDetection,
            action: AutoCookTaskAction::DetectCookUiIcon,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::CookUiDetection,
            action: AutoCookTaskAction::ResetPeakOnUiTransition,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::RecoverButtonGate,
            action: AutoCookTaskAction::StopWhenRecoverButtonDetected,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::ConfirmButton,
            action: AutoCookTaskAction::ClickWhiteConfirmButton,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::CookBarSampling,
            action: AutoCookTaskAction::CountTargetCookColor,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::PeakTracking,
            action: AutoCookTaskAction::BuildStablePeak,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::Input,
            action: AutoCookTaskAction::PressSpaceWhenPeakDrops,
        },
        AutoCookTaskStep {
            phase: AutoCookTaskPhase::Delay,
            action: AutoCookTaskAction::DelayConfiguredInterval,
        },
    ]
}

fn scale_rect(rect: Rect, asset_scale: f64) -> Rect {
    Rect {
        x: scale_i32(rect.x, asset_scale),
        y: scale_i32(rect.y, asset_scale),
        width: scale_i32(rect.width, asset_scale),
        height: scale_i32(rect.height, asset_scale),
    }
}

fn scale_i32(value: i32, asset_scale: f64) -> i32 {
    (value as f64 * asset_scale) as i32
}

fn scale_count(value: u32, asset_scale: f64) -> u32 {
    (value as f64 * asset_scale) as u32
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

fn f64_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<f64> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(Value::as_f64)
}
