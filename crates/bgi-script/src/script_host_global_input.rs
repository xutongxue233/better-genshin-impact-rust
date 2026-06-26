use super::{Result, ScriptHostRuntimeError};
use bgi_capture::{
    find_bilibili_login_window, find_process_image_path_by_name, BilibiliLoginWindowKind,
    BilibiliLoginWindowSearchConfig, CaptureFrame, PixelFormat,
};
use bgi_input::{
    activate_window, send_events, send_events_to_window, InputEvent, InputSequence, MouseButton,
};
use bgi_task::{
    CommonJobFrameSource, CommonJobInputDriver, CommonJobRuntimeOutcome, ReloginPlatformDriver,
    ReloginThirdPartyRule, TaskError,
};
use bgi_vision::{BgrImage, ImageRegion, ImageRegionModel, Rect, Size as VisionSize};
use serde::Serialize;
use std::sync::Arc;

#[path = "script_host_global_input_keys.rs"]
mod keys;

pub use self::keys::virtual_key_code_for_script;
use self::keys::{key_down_sequence, key_press_sequence, key_up_sequence};

pub trait GameCaptureFrameSource: Send + Sync {
    fn capture_frame(&self) -> Result<CaptureFrame>;

    fn capture_frame_area(&self, frame: &CaptureFrame) -> GameCaptureArea {
        GameCaptureArea {
            x: 0,
            y: 0,
            width: frame.size.width,
            height: frame.size.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureGameRegionPlan {
    pub area: GameCaptureArea,
    pub pixel_format: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureGameRegionExecution {
    pub plan: CaptureGameRegionPlan,
    pub image_region: ImageRegionModel,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub pixels: Vec<u8>,
    pub source_width: u32,
    pub source_height: u32,
}

impl CaptureGameRegionExecution {
    fn from_capture(plan: CaptureGameRegionPlan, source: BgrImage) -> Result<Self> {
        let source_width = source.size.width;
        let source_height = source.size.height;
        let capture_region = ImageRegion::capture(source).derive_crop(plan.area.as_rect()?)?;
        Ok(Self {
            width: capture_region.image.size.width,
            height: capture_region.image.size.height,
            pixel_format: "BGR24",
            pixels: capture_region.image.pixels,
            image_region: capture_region.model,
            source_width,
            source_height,
            plan,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AvatarRecognitionPlan {
    pub capture: CaptureGameRegionPlan,
    pub model_name: &'static str,
    pub model_relative_path: &'static str,
    pub output: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GameMetrics {
    pub width: u32,
    pub height: u32,
    pub dpi: f64,
}

impl GameMetrics {
    pub fn new(width: u32, height: u32, dpi: f64) -> Result<Self> {
        if width.saturating_mul(9) != height.saturating_mul(16) {
            return Err(ScriptHostRuntimeError::InvalidGameMetrics { width, height });
        }
        Ok(Self { width, height, dpi })
    }
}

impl Default for GameMetrics {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            dpi: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GameCaptureArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl GameCaptureArea {
    fn validate(self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(ScriptHostRuntimeError::InvalidCaptureArea { area: self });
        }
        Ok(())
    }

    fn as_rect(self) -> Result<Rect> {
        Rect::new(self.x, self.y, self.width as i32, self.height as i32).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GlobalInputDispatchMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GlobalInputExecution {
    pub mode: GlobalInputDispatchMode,
    pub events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
}

impl GlobalInputExecution {
    pub fn execute(
        sequence: InputSequence,
        mode: GlobalInputDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<Self> {
        let events = sequence.events().to_vec();
        Self::execute_events(events, mode, window_handle)
    }

    pub fn execute_events(
        events: Vec<InputEvent>,
        mode: GlobalInputDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<Self> {
        if mode == GlobalInputDispatchMode::SendInput {
            if let Some(hwnd) = window_handle {
                send_events_to_window(hwnd, &events)?;
            } else {
                send_events(&events)?;
            }
        }

        Ok(Self {
            mode,
            dispatched: mode == GlobalInputDispatchMode::SendInput,
            dispatched_events: if mode == GlobalInputDispatchMode::SendInput {
                events.len()
            } else {
                0
            },
            events,
        })
    }
}

#[derive(Clone)]
pub struct ScriptCommonJobFrameSource {
    capture_frame_source: Arc<dyn GameCaptureFrameSource>,
}

impl ScriptCommonJobFrameSource {
    pub fn new(capture_frame_source: Arc<dyn GameCaptureFrameSource>) -> Self {
        Self {
            capture_frame_source,
        }
    }
}

impl std::fmt::Debug for ScriptCommonJobFrameSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptCommonJobFrameSource")
            .field("capture_frame_source", &"<capture-source>")
            .finish()
    }
}

impl CommonJobFrameSource for ScriptCommonJobFrameSource {
    fn capture_frame(&mut self) -> bgi_task::Result<BgrImage> {
        let frame = self
            .capture_frame_source
            .capture_frame()
            .map_err(common_job_adapter_error)?;
        bgr_image_from_capture_frame(frame).map_err(common_job_adapter_error)
    }
}

#[derive(Debug, Clone)]
pub struct ScriptCommonJobInputDriver {
    global_input: GlobalInputHost,
    mode: GlobalInputDispatchMode,
    window_handle: Option<isize>,
    executions: Vec<GlobalInputExecution>,
}

impl ScriptCommonJobInputDriver {
    pub fn new(
        global_input: GlobalInputHost,
        mode: GlobalInputDispatchMode,
        window_handle: Option<isize>,
    ) -> Self {
        Self {
            global_input,
            mode,
            window_handle,
            executions: Vec::new(),
        }
    }

    pub fn executions(&self) -> &[GlobalInputExecution] {
        &self.executions
    }

    pub fn into_executions(self) -> Vec<GlobalInputExecution> {
        self.executions
    }

    fn execute_sequence(&mut self, sequence: InputSequence) -> bgi_task::Result<()> {
        let execution = GlobalInputExecution::execute(sequence, self.mode, self.window_handle)
            .map_err(common_job_adapter_error)?;
        self.executions.push(execution);
        Ok(())
    }

    fn execute_desktop_sequence(&mut self, sequence: InputSequence) -> bgi_task::Result<()> {
        let execution = GlobalInputExecution::execute(sequence, self.mode, None)
            .map_err(common_job_adapter_error)?;
        self.executions.push(execution);
        Ok(())
    }

    fn capture_events_to_screen_events(
        &self,
        events: &[InputEvent],
    ) -> bgi_task::Result<Vec<InputEvent>> {
        let mut mapped = Vec::with_capacity(events.len());
        for event in events {
            match event {
                InputEvent::MouseMoveAbsolute { x, y, .. } => {
                    mapped.extend_from_slice(
                        self.global_input
                            .move_mouse_to(*x, *y)
                            .map_err(common_job_adapter_error)?
                            .events(),
                    );
                }
                event => mapped.push(*event),
            }
        }
        Ok(mapped)
    }
}

impl CommonJobInputDriver for ScriptCommonJobInputDriver {
    fn dispatch_input(&mut self, events: &[InputEvent]) -> bgi_task::Result<()> {
        let execution =
            GlobalInputExecution::execute_events(events.to_vec(), self.mode, self.window_handle)
                .map_err(common_job_adapter_error)?;
        self.executions.push(execution);
        Ok(())
    }

    fn dispatch_capture_input(&mut self, events: &[InputEvent]) -> bgi_task::Result<()> {
        let mapped = self.capture_events_to_screen_events(events)?;
        let execution = GlobalInputExecution::execute_events(mapped, self.mode, self.window_handle)
            .map_err(common_job_adapter_error)?;
        self.executions.push(execution);
        Ok(())
    }

    fn click_capture_point(&mut self, x: i32, y: i32) -> bgi_task::Result<()> {
        let sequence = self
            .global_input
            .click(x, y)
            .map_err(common_job_adapter_error)?;
        self.execute_sequence(sequence)
    }
}

impl ReloginPlatformDriver for ScriptCommonJobInputDriver {
    fn focus_game_window(&mut self) -> bgi_task::Result<()> {
        let Some(hwnd) = self.window_handle else {
            return Err(TaskError::CommonJobExecution(
                "Relogin live focus requires a game window handle".to_string(),
            ));
        };
        activate_window(hwnd).map_err(common_job_adapter_error)
    }

    fn execute_third_party_login_probe(
        &mut self,
        rule: &ReloginThirdPartyRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if !rule.bilibili_only {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if rule.refresh_available_before_login && !is_bilibili_login_available()? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }

        sleep_milliseconds(rule.pre_login_sleep_ms);
        let config = BilibiliLoginWindowSearchConfig::default();
        let max_probes = rule.max_login_probes.max(1);
        for probe_index in 0..max_probes {
            if self.execute_bilibili_login_probe_once(rule, &config)? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
            if probe_index + 1 < max_probes {
                sleep_milliseconds(rule.probe_interval_ms);
            }
        }

        Ok(CommonJobRuntimeOutcome::Matched(false))
    }
}

impl ScriptCommonJobInputDriver {
    fn execute_bilibili_login_probe_once(
        &mut self,
        rule: &ReloginThirdPartyRule,
        config: &BilibiliLoginWindowSearchConfig,
    ) -> bgi_task::Result<bool> {
        let Some(window) = find_bilibili_login_window(config).map_err(common_job_adapter_error)?
        else {
            return Ok(false);
        };

        match window.kind {
            BilibiliLoginWindowKind::Agreement => {
                self.click_relogin_dpi_aware_point(&rule.agreement_click)?;
                let _ = find_bilibili_login_window(config).map_err(common_job_adapter_error)?;
                Ok(false)
            }
            BilibiliLoginWindowKind::Login => {
                sleep_milliseconds(rule.login_window_sleep_ms);
                self.click_relogin_dpi_aware_point(&rule.login_click)?;
                sleep_milliseconds(rule.login_window_sleep_ms);
                Ok(find_bilibili_login_window(config)
                    .map_err(common_job_adapter_error)?
                    .is_none())
            }
        }
    }

    fn click_relogin_dpi_aware_point(
        &mut self,
        point: &bgi_task::ReloginDpiAwarePoint,
    ) -> bgi_task::Result<()> {
        let x = relogin_dpi_aware_coordinate(
            point.x_1080p,
            point.x_dpi_offset,
            self.global_input.runtime_dpi,
        );
        let y = relogin_dpi_aware_coordinate(
            point.y_1080p,
            point.y_dpi_offset,
            self.global_input.runtime_dpi,
        );
        let sequence = self
            .global_input
            .click(x, y)
            .map_err(common_job_adapter_error)?;
        self.execute_desktop_sequence(sequence)
    }
}

pub(super) fn relogin_dpi_aware_coordinate(
    base_1080p: f64,
    dpi_offset: f64,
    runtime_dpi: f64,
) -> i32 {
    (base_1080p + dpi_offset * runtime_dpi).round() as i32
}

fn is_bilibili_login_available() -> bgi_task::Result<bool> {
    let Some(path) =
        find_process_image_path_by_name("YuanShen").map_err(common_job_adapter_error)?
    else {
        return Ok(false);
    };
    let Some(directory) = path.parent() else {
        return Ok(false);
    };
    let config_path = directory.join("config.ini");
    Ok(std::fs::read_to_string(config_path)
        .ok()
        .is_some_and(|text| bilibili_config_text_has_channel_14(&text)))
}

pub(super) fn bilibili_config_text_has_channel_14(text: &str) -> bool {
    text.lines().any(|line| {
        let kv = line.trim();
        kv.starts_with("channel=") && kv.ends_with("14")
    })
}

fn sleep_milliseconds(milliseconds: u32) {
    if milliseconds > 0 {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds as u64));
    }
}

fn common_job_adapter_error(error: impl std::fmt::Display) -> TaskError {
    TaskError::CommonJobExecution(error.to_string())
}

#[derive(Clone)]
pub struct GlobalInputHost {
    game_metrics: GameMetrics,
    capture_area: GameCaptureArea,
    capture_frame_source: Option<Arc<dyn GameCaptureFrameSource>>,
    runtime_dpi: f64,
}

impl std::fmt::Debug for GlobalInputHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalInputHost")
            .field("game_metrics", &self.game_metrics)
            .field("capture_area", &self.capture_area)
            .field(
                "capture_frame_source",
                &self
                    .capture_frame_source
                    .as_ref()
                    .map(|_| "<capture-source>"),
            )
            .field("runtime_dpi", &self.runtime_dpi)
            .finish()
    }
}

impl GlobalInputHost {
    pub fn new(capture_area: GameCaptureArea, runtime_dpi: f64) -> Result<Self> {
        Self::new_with_frame_source(capture_area, runtime_dpi, None)
    }

    pub fn new_with_frame_source(
        capture_area: GameCaptureArea,
        runtime_dpi: f64,
        capture_frame_source: Option<Arc<dyn GameCaptureFrameSource>>,
    ) -> Result<Self> {
        capture_area.validate()?;
        Ok(Self {
            game_metrics: GameMetrics::default(),
            capture_area,
            capture_frame_source,
            runtime_dpi,
        })
    }

    pub fn game_metrics(&self) -> GameMetrics {
        self.game_metrics
    }

    pub fn has_capture_frame_source(&self) -> bool {
        self.capture_frame_source.is_some()
    }

    pub fn common_job_frame_source(&self) -> Option<ScriptCommonJobFrameSource> {
        self.capture_frame_source
            .as_ref()
            .map(|source| ScriptCommonJobFrameSource::new(source.clone()))
    }

    pub fn common_job_input_driver(
        &self,
        mode: GlobalInputDispatchMode,
        window_handle: Option<isize>,
    ) -> ScriptCommonJobInputDriver {
        ScriptCommonJobInputDriver::new(self.clone(), mode, window_handle)
    }

    pub fn capture_game_region(&self) -> CaptureGameRegionPlan {
        CaptureGameRegionPlan {
            area: self.capture_area,
            pixel_format: "BGR24",
            source: "game_capture_region",
        }
    }

    pub fn capture_game_region_execution(&self) -> Result<Option<CaptureGameRegionExecution>> {
        let Some(source) = &self.capture_frame_source else {
            return Ok(None);
        };
        let frame = source.capture_frame()?;
        let plan = CaptureGameRegionPlan {
            area: source.capture_frame_area(&frame),
            pixel_format: "BGR24",
            source: "game_capture_region",
        };
        let image = bgr_image_from_capture_frame(frame)?;
        CaptureGameRegionExecution::from_capture(plan, image).map(Some)
    }

    pub fn get_avatars(&self) -> AvatarRecognitionPlan {
        AvatarRecognitionPlan {
            capture: self.capture_game_region(),
            model_name: "BgiAvatarSide",
            model_relative_path: "Assets/Model/Common/avatar_side_classify_sim.onnx",
            output: "avatar_names",
        }
    }

    pub fn set_game_metrics(&mut self, width: u32, height: u32, dpi: f64) -> Result<()> {
        self.game_metrics = GameMetrics::new(width, height, dpi)?;
        Ok(())
    }

    pub fn key_down(&self, key: &str) -> Result<InputSequence> {
        key_down_sequence(key)
    }

    pub fn key_up(&self, key: &str) -> Result<InputSequence> {
        key_up_sequence(key)
    }

    pub fn key_press(&self, key: &str) -> Result<InputSequence> {
        key_press_sequence(key)
    }

    pub fn move_mouse_by(&self, x: i32, y: i32) -> InputSequence {
        let dpi = if self.game_metrics.dpi <= 0.0 {
            1.0
        } else {
            self.game_metrics.dpi
        };
        InputSequence::new().move_mouse_by(
            (x as f64 * self.runtime_dpi / dpi).trunc() as i32,
            (y as f64 * self.runtime_dpi / dpi).trunc() as i32,
        )
    }

    pub fn move_mouse_to(&self, x: i32, y: i32) -> Result<InputSequence> {
        Ok(InputSequence::new().move_mouse_to(self.to_screen_x(x)?, self.to_screen_y(y)?))
    }

    pub fn click(&self, x: i32, y: i32) -> Result<InputSequence> {
        Ok(self.move_mouse_to(x, y)?.mouse_click(MouseButton::Left))
    }

    pub fn left_button_click(&self) -> InputSequence {
        InputSequence::new()
            .mouse_down(MouseButton::Left)
            .delay(60)
            .mouse_up(MouseButton::Left)
    }

    pub fn left_button_down(&self) -> InputSequence {
        InputSequence::new().mouse_down(MouseButton::Left)
    }

    pub fn left_button_up(&self) -> InputSequence {
        InputSequence::new().mouse_up(MouseButton::Left)
    }

    pub fn right_button_click(&self) -> InputSequence {
        InputSequence::new()
            .mouse_down(MouseButton::Right)
            .delay(60)
            .mouse_up(MouseButton::Right)
    }

    pub fn right_button_down(&self) -> InputSequence {
        InputSequence::new().mouse_down(MouseButton::Right)
    }

    pub fn right_button_up(&self) -> InputSequence {
        InputSequence::new().mouse_up(MouseButton::Right)
    }

    pub fn middle_button_click(&self) -> InputSequence {
        InputSequence::new().mouse_click(MouseButton::Middle)
    }

    pub fn middle_button_down(&self) -> InputSequence {
        InputSequence::new().mouse_down(MouseButton::Middle)
    }

    pub fn middle_button_up(&self) -> InputSequence {
        InputSequence::new().mouse_up(MouseButton::Middle)
    }

    pub fn vertical_scroll(&self, scroll_amount_in_clicks: i32) -> InputSequence {
        InputSequence::new().vertical_scroll(scroll_amount_in_clicks)
    }

    pub fn input_text(&self, text: &str) -> InputSequence {
        InputSequence::new().text(text)
    }

    fn to_screen_x(&self, x: i32) -> Result<i32> {
        self.validate_mouse_coordinate(x, 0)?;
        Ok(self.capture_area.x
            + (x as f64 * self.capture_area.width as f64 / self.game_metrics.width as f64).trunc()
                as i32)
    }

    fn to_screen_y(&self, y: i32) -> Result<i32> {
        self.validate_mouse_coordinate(0, y)?;
        Ok(self.capture_area.y
            + (y as f64 * self.capture_area.height as f64 / self.game_metrics.height as f64).trunc()
                as i32)
    }

    fn validate_mouse_coordinate(&self, x: i32, y: i32) -> Result<()> {
        if x < 0
            || y < 0
            || x > self.game_metrics.width as i32
            || y > self.game_metrics.height as i32
        {
            return Err(ScriptHostRuntimeError::MouseCoordinateOutOfBounds {
                x,
                y,
                width: self.game_metrics.width,
                height: self.game_metrics.height,
            });
        }
        Ok(())
    }
}

fn bgr_image_from_capture_frame(frame: CaptureFrame) -> Result<BgrImage> {
    if frame.pixel_format != PixelFormat::Bgr8 {
        return Err(ScriptHostRuntimeError::UnsupportedCaptureFrame);
    }

    let row_bytes = frame.row_bytes();
    let pixels = if frame.is_packed() {
        frame.pixels
    } else {
        let mut pixels = Vec::with_capacity(row_bytes * frame.size.height as usize);
        for row in 0..frame.size.height as usize {
            let start = row * frame.stride;
            let end = start + row_bytes;
            pixels.extend_from_slice(&frame.pixels[start..end]);
        }
        pixels
    };

    BgrImage::new(VisionSize::new(frame.size.width, frame.size.height), pixels).map_err(Into::into)
}
