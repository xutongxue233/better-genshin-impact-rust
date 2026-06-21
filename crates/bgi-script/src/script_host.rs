use crate::policy::{
    NotificationRateLimiter, ScriptFilePolicy, ScriptHostPolicyError, ScriptHttpPolicy,
    ScriptNotificationPolicy,
};
use crate::r#macro::{KeyMouseMacroError, KeyMouseScript, MacroPlaybackContext};
use bgi_capture::{CaptureFrame, PixelFormat};
use bgi_core::{BgiError, PathingExecutionPlan, PathingSummary, PathingTask};
use bgi_input::{
    send_events, send_events_to_window, send_events_to_window_with_cancellation,
    send_events_with_cancellation, InputCancellationToken, InputEvent, InputSequence, MouseButton,
    PostMessageEvent, PostMessageSequence,
};
use bgi_vision::{
    resize_bgr_nearest, BgrImage, ColorConversion, ColorMatchConfig, ImageRegion, ImageRegionModel,
    PureRustVisionBackend, RecognitionObject, RecognitionType, Rect, Region, Scalar4,
    Size as VisionSize, TemplateMatchMode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum ScriptHostRuntimeError {
    #[error("script host file policy rejected the path: {0}")]
    Policy(#[from] ScriptHostPolicyError),
    #[error("key/mouse macro error: {0}")]
    KeyMouseMacro(#[from] KeyMouseMacroError),
    #[error("pathing task error: {0}")]
    Pathing(#[from] BgiError),
    #[error("failed to read script host file at {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("input dispatch failed: {0}")]
    Input(#[from] bgi_input::InputError),
    #[error("vision image IO failed: {0}")]
    Vision(#[from] bgi_vision::VisionError),
    #[error("virtual key name is not supported: {0}")]
    UnsupportedVirtualKey(String),
    #[error("game resolution must be 16:9, got {width}x{height}")]
    InvalidGameMetrics { width: u32, height: u32 },
    #[error("mouse coordinate ({x}, {y}) is outside the game metrics {width}x{height}")]
    MouseCoordinateOutOfBounds {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    #[error("capture area is invalid: {area:?}")]
    InvalidCaptureArea { area: GameCaptureArea },
    #[error("capture frame source failed: {0}")]
    Capture(#[from] bgi_capture::CaptureError),
    #[error("capture frame does not contain a BGR24 image")]
    UnsupportedCaptureFrame,
    #[error("rename source does not exist: {0:?}")]
    RenameSourceMissing(PathBuf),
    #[error("cannot rename mixed file/directory paths from {from:?} to {to:?}")]
    RenameKindMismatch { from: PathBuf, to: PathBuf },
    #[error("script host method {target}.{method} is not implemented")]
    UnknownHostMethod {
        target: &'static str,
        method: String,
    },
    #[error("script host method {method} argument {index} must be {expected}")]
    InvalidArgument {
        method: String,
        index: usize,
        expected: &'static str,
    },
    #[error("invalid server timezone offset: {0}")]
    InvalidServerTimeZoneOffset(String),
    #[error("headers JSON must be an object of string values")]
    InvalidHttpHeaders,
    #[error("invalid HTTP method: {0}")]
    InvalidHttpMethod(String),
    #[error("HTTP request dispatch failed: {0}")]
    HttpDispatch(#[from] reqwest::Error),
    #[error("task invocation planning failed: {0}")]
    Task(#[from] bgi_task::TaskError),
}

pub type Result<T> = std::result::Result<T, ScriptHostRuntimeError>;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseScriptRunPlan {
    pub source: KeyMouseScriptSource,
    pub normalized_path: Option<PathBuf>,
    pub summary: crate::KeyMouseMacroSummary,
    pub input_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum KeyMouseScriptDispatchMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseScriptExecution {
    pub mode: KeyMouseScriptDispatchMode,
    pub plan: KeyMouseScriptRunPlan,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub processed_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseScriptSource {
    InlineJson,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PathingScriptSource {
    InlineJson,
    ScriptFile,
    UserAutoPathingFile,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingScriptRunPlan {
    pub source: PathingScriptSource,
    pub normalized_path: Option<PathBuf>,
    pub summary: PathingSummary,
    pub task: PathingTask,
    pub party_config: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathingScriptExecution {
    pub plan: PathingScriptRunPlan,
    pub execution_plan: PathingExecutionPlan,
    pub dispatched: bool,
    pub completed: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatResizePlan {
    pub width: f64,
    pub height: f64,
    pub interpolation: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatReadPlan {
    pub normalized_path: PathBuf,
    pub color_mode: &'static str,
    pub resize: Option<ImageMatResizePlan>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatWritePlan {
    pub normalized_path: PathBuf,
    pub source: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatReadExecution {
    pub normalized_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub pixels: Vec<u8>,
    pub color_mode: &'static str,
    pub resize: Option<ImageMatResizePlan>,
}

impl ImageMatReadExecution {
    fn from_image(
        normalized_path: PathBuf,
        image: BgrImage,
        resize: Option<ImageMatResizePlan>,
    ) -> Self {
        Self {
            normalized_path,
            width: image.size.width,
            height: image.size.height,
            pixel_format: "BGR24",
            pixels: image.pixels,
            color_mode: "color",
            resize,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImageMatWriteExecution {
    pub normalized_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VisionRecognitionExecution {
    pub recognition_type: RecognitionType,
    pub image_region: ImageRegionModel,
    pub first: Region,
    pub matches: Vec<Region>,
    pub matched_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VisionImageMatExecution {
    pub image_region: ImageRegionModel,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub pixels: Vec<u8>,
    pub color_mode: &'static str,
}

impl VisionImageMatExecution {
    fn from_image_region(region: ImageRegion) -> Self {
        Self {
            width: region.image.size.width,
            height: region.image.size.height,
            pixel_format: "BGR24",
            pixels: region.image.pixels,
            color_mode: "color",
            image_region: region.model,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CustomHostFunctionCommand {
    NewArrayVariable {
        element_type: String,
        dimensions: u32,
        legacy_jagged_type: String,
    },
    NewObject {
        type_name: String,
        args: Vec<Value>,
    },
    DeleteObject {
        target: Option<Value>,
    },
    TypeLookup {
        type_name: String,
    },
    ToIterator {
        source: Value,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptLogRecord {
    pub level: ScriptLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptNotificationKind {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptNotificationRecord {
    pub kind: ScriptNotificationKind,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptNotificationDelivery {
    pub event_code: &'static str,
    pub result: &'static str,
    pub message: String,
    pub timestamp_ms: u64,
}

impl ScriptNotificationDelivery {
    pub fn from_record(record: &ScriptNotificationRecord) -> Self {
        let (event_code, result) = match record.kind {
            ScriptNotificationKind::Success => ("js.custom", "success"),
            ScriptNotificationKind::Error => ("js.error", "fail"),
        };
        Self {
            event_code,
            result,
            message: record.message.clone(),
            timestamp_ms: record.timestamp_ms,
        }
    }
}

pub trait ScriptNotificationSink {
    fn deliver(&mut self, delivery: ScriptNotificationDelivery) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationDispatchMode {
    RecordOnly,
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationExecution {
    pub mode: NotificationDispatchMode,
    pub record: ScriptNotificationRecord,
    pub delivery: Option<ScriptNotificationDelivery>,
    pub dispatched: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RecordingNotificationSink {
    deliveries: Vec<ScriptNotificationDelivery>,
}

impl RecordingNotificationSink {
    pub fn deliveries(&self) -> &[ScriptNotificationDelivery] {
        &self.deliveries
    }
}

impl ScriptNotificationSink for RecordingNotificationSink {
    fn deliver(&mut self, delivery: ScriptNotificationDelivery) -> Result<()> {
        self.deliveries.push(delivery);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ServerTimeHost {
    server_time_zone_offset_milliseconds: i32,
}

impl ServerTimeHost {
    pub fn from_offset_milliseconds(server_time_zone_offset_milliseconds: i32) -> Self {
        Self {
            server_time_zone_offset_milliseconds,
        }
    }

    pub fn from_offset_string(offset: &str) -> Result<Self> {
        Ok(Self::from_offset_milliseconds(
            parse_server_time_zone_offset_milliseconds(offset)?,
        ))
    }

    pub fn server_time_zone_offset_milliseconds(&self) -> i32 {
        self.server_time_zone_offset_milliseconds
    }
}

impl Default for ServerTimeHost {
    fn default() -> Self {
        Self::from_offset_milliseconds(8 * 60 * 60 * 1_000)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpRequestPlan {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpResponseRecord {
    pub status_code: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HttpDispatchMode {
    PlanOnly,
    Reqwest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpExecution {
    pub mode: HttpDispatchMode,
    pub request: HttpRequestPlan,
    pub response: Option<HttpResponseRecord>,
    pub dispatched: bool,
}

pub trait ScriptHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord>;
}

#[derive(Debug, Clone)]
pub struct RecordingHttpClient {
    response: HttpResponseRecord,
    requests: Vec<HttpRequestPlan>,
}

impl RecordingHttpClient {
    pub fn new(response: HttpResponseRecord) -> Self {
        Self {
            response,
            requests: Vec::new(),
        }
    }

    pub fn ok_json(body: impl Into<String>) -> Self {
        Self::new(HttpResponseRecord {
            status_code: 200,
            headers: BTreeMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: body.into(),
        })
    }

    pub fn requests(&self) -> &[HttpRequestPlan] {
        &self.requests
    }
}

impl ScriptHttpClient for RecordingHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord> {
        self.requests.push(request);
        Ok(self.response.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestScriptHttpClient {
    client: reqwest::blocking::Client,
}

impl ReqwestScriptHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestScriptHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptHttpClient for ReqwestScriptHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord> {
        let method = request
            .method
            .parse::<reqwest::Method>()
            .map_err(|_| ScriptHostRuntimeError::InvalidHttpMethod(request.method.clone()))?;
        let mut builder = self.client.request(method, &request.url);
        for (key, value) in request.headers {
            builder = builder.header(key, value);
        }
        if let Some(body) = request.body {
            builder = builder
                .header(reqwest::header::CONTENT_TYPE, request.content_type)
                .body(body);
        }

        let response = builder.send()?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(key, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (key.as_str().to_string(), value.to_string()))
            })
            .collect::<BTreeMap<_, _>>();
        let body = response.text()?;

        Ok(HttpResponseRecord {
            status_code,
            headers,
            body,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpHost {
    policy: ScriptHttpPolicy,
}

impl HttpHost {
    pub fn new(policy: ScriptHttpPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &ScriptHttpPolicy {
        &self.policy
    }

    pub fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers_json: Option<&str>,
    ) -> Result<HttpRequestPlan> {
        self.policy.check_url(url)?;
        let (headers, content_type) = normalize_http_headers(headers_json)?;
        Ok(HttpRequestPlan {
            method: method.to_ascii_uppercase(),
            url: url.to_string(),
            body: body.map(ToOwned::to_owned),
            headers,
            content_type,
        })
    }

    pub fn execute_request<C: ScriptHttpClient>(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers_json: Option<&str>,
        client: &mut C,
    ) -> Result<HttpResponseRecord> {
        let plan = self.request(method, url, body, headers_json)?;
        client.send(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlMaskMessage {
    pub url: String,
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HtmlMaskWindowPlan {
    pub window_id: String,
    pub final_url: String,
    pub requested_url: String,
    pub normalized_path: Option<PathBuf>,
    pub click_through: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HtmlMaskCommand {
    Show(HtmlMaskWindowPlan),
    Close {
        window_id: String,
    },
    CloseAll {
        window_ids: Vec<String>,
    },
    SetClickThrough {
        window_id: String,
        enabled: bool,
    },
    Send {
        window_id: String,
        message: HtmlMaskMessage,
    },
    Request {
        window_id: String,
        message: HtmlMaskMessage,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HtmlMaskSnapshot {
    pub windows: Vec<HtmlMaskWindowPlan>,
    pub commands: Vec<HtmlMaskCommand>,
    pub to_html_queue_count: usize,
    pub from_html_queue_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HtmlMaskInitialState {
    pub windows: Vec<HtmlMaskWindowPlan>,
    pub from_html: Vec<(String, HtmlMaskMessage)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum KeyMouseHookEventKind {
    KeyDown,
    KeyUp,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseWheel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseHookListener {
    pub id: String,
    pub event: KeyMouseHookEventKind,
    pub use_code_only: bool,
    pub interval_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseHookCommand {
    AddListener(KeyMouseHookListener),
    RemoveAllListeners,
    Dispose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum KeyMouseHookEvent {
    Key {
        event: KeyMouseHookEventKind,
        key_data: String,
        key_code: String,
    },
    MouseButton {
        event: KeyMouseHookEventKind,
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseMove {
        x: i32,
        y: i32,
        timestamp_ms: u64,
    },
    MouseWheel {
        delta: i32,
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeyMouseHookDispatch {
    pub listener_id: String,
    pub event: KeyMouseHookEventKind,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyMouseHookSnapshot {
    pub listeners: Vec<KeyMouseHookListener>,
    pub commands: Vec<KeyMouseHookCommand>,
    pub disposed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RealtimeTimerHostPlan {
    pub name: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub clears_existing_triggers: bool,
}

impl RealtimeTimerHostPlan {
    pub fn new(name: impl Into<String>, config: Option<Value>) -> Self {
        Self {
            name: name.into(),
            interval_ms: 50,
            config,
            clears_existing_triggers: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SoloTaskHostPlan {
    pub name: String,
    pub config: Option<Value>,
    pub uses_linked_cancellation: bool,
}

impl SoloTaskHostPlan {
    pub fn new(name: impl Into<String>, config: Option<Value>) -> Self {
        Self {
            name: name.into(),
            config,
            uses_linked_cancellation: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoPickExternalConfig {
    pub text_list: Vec<String>,
    pub force_interaction: bool,
}

impl AutoPickExternalConfig {
    pub fn from_value(value: Option<&Value>) -> Result<Self> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let Value::Object(map) = value else {
            return Err(invalid_arg_for_method(
                "AutoPickExternalConfig",
                0,
                "object",
            ));
        };
        let text_list = map
            .get("textList")
            .or_else(|| map.get("TextList"))
            .or_else(|| map.get("text_list"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let force_interaction = map
            .get("forceInteraction")
            .or_else(|| map.get("ForceInteraction"))
            .or_else(|| map.get("force_interaction"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        Ok(Self {
            text_list,
            force_interaction,
        })
    }

    pub fn to_legacy_config_value(&self) -> Value {
        serde_json::json!({
            "TextList": self.text_list,
            "ForceInteraction": self.force_interaction
        })
    }
}

impl Default for AutoPickExternalConfig {
    fn default() -> Self {
        Self {
            text_list: Vec::new(),
            force_interaction: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DispatcherCommand {
    ClearAllTriggers,
    AddRealtimeTimer(RealtimeTimerHostPlan),
    RunCurrentTask,
    RunSoloTask(SoloTaskHostPlan),
    LinkedCancellationTokenSource,
    LinkedCancellationToken,
    RunBuiltinTask {
        name: String,
        config: Value,
        uses_linked_cancellation: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GenshinCommand {
    Uid,
    Teleport {
        x: f64,
        y: f64,
        map_name: Option<String>,
        force: bool,
    },
    MoveMapTo {
        x: f64,
        y: f64,
        map_name: Option<String>,
        force_country: Option<String>,
    },
    GetBigMapZoomLevel,
    SetBigMapZoomLevel {
        zoom_level: f64,
    },
    TpToStatueOfTheSeven,
    GetPositionFromBigMap {
        map_name: Option<String>,
    },
    GetPositionFromMap {
        map_name: Option<String>,
        cache_time_ms: Option<u64>,
        matching_method: Option<String>,
        nearby: Option<(f64, f64)>,
    },
    GetCameraOrientation,
    SwitchParty {
        party_name: String,
    },
    ClearPartyCache,
    BlessingOfTheWelkinMoon,
    ChooseTalkOption {
        option: String,
        skip_times: u32,
        is_orange: bool,
    },
    ClaimBattlePassRewards,
    ClaimEncounterPointsRewards,
    GoToAdventurersGuild {
        country: String,
    },
    GoToCraftingBench {
        country: String,
    },
    ReturnMainUi,
    AutoFishing {
        fishing_time_policy: i32,
    },
    Relogin,
    WonderlandCycle,
    SetTime {
        hour: u32,
        minute: u32,
        skip: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct GenshinHost {
    commands: Vec<GenshinCommand>,
}

impl GenshinHost {
    pub fn commands(&self) -> &[GenshinCommand] {
        &self.commands
    }

    pub fn task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.commands
            .iter()
            .filter_map(genshin_command_to_task_input)
            .map(|command| {
                bgi_task::TaskInvocationPlan::from_script_dispatcher_command(command)
                    .map_err(Into::into)
            })
            .collect()
    }

    pub fn push(&mut self, command: GenshinCommand) -> GenshinCommand {
        self.commands.push(command.clone());
        command
    }
}

pub fn genshin_command_to_task_input(
    command: &GenshinCommand,
) -> Option<bgi_task::ScriptDispatcherCommandInput> {
    let (name, config) = match command {
        GenshinCommand::Teleport {
            x,
            y,
            map_name,
            force,
        } => (
            "Teleport",
            serde_json::json!({ "x": x, "y": y, "mapName": map_name, "force": force }),
        ),
        GenshinCommand::MoveMapTo {
            x,
            y,
            map_name,
            force_country,
        } => (
            "Teleport",
            serde_json::json!({
                "kind": "moveMapTo",
                "x": x,
                "y": y,
                "mapName": map_name,
                "forceCountry": force_country
            }),
        ),
        GenshinCommand::TpToStatueOfTheSeven => (
            "Teleport",
            serde_json::json!({ "kind": "statueOfTheSeven" }),
        ),
        GenshinCommand::SwitchParty { party_name } => (
            "SwitchParty",
            serde_json::json!({ "partyName": party_name }),
        ),
        GenshinCommand::BlessingOfTheWelkinMoon => {
            ("BlessingOfTheWelkinMoon", serde_json::json!({}))
        }
        GenshinCommand::ChooseTalkOption {
            option,
            skip_times,
            is_orange,
        } => (
            "ChooseTalkOption",
            serde_json::json!({
                "option": option,
                "skipTimes": skip_times,
                "isOrange": is_orange
            }),
        ),
        GenshinCommand::ClaimBattlePassRewards => ("ClaimBattlePassRewards", serde_json::json!({})),
        GenshinCommand::ClaimEncounterPointsRewards => {
            ("ClaimEncounterPointsRewards", serde_json::json!({}))
        }
        GenshinCommand::ReturnMainUi => ("ReturnMainUi", serde_json::json!({})),
        GenshinCommand::SetTime { hour, minute, skip } => (
            "SetTime",
            serde_json::json!({ "hour": hour, "minute": minute, "skip": skip }),
        ),
        GenshinCommand::AutoFishing {
            fishing_time_policy,
        } => (
            "AutoFishing",
            serde_json::json!({ "fishingTimePolicy": fishing_time_policy }),
        ),
        GenshinCommand::GoToAdventurersGuild { country } => (
            "GoToAdventurersGuild",
            serde_json::json!({ "country": country }),
        ),
        GenshinCommand::GoToCraftingBench { country } => (
            "GoToCraftingBench",
            serde_json::json!({ "country": country }),
        ),
        GenshinCommand::Relogin => ("Relogin", serde_json::json!({})),
        GenshinCommand::WonderlandCycle => ("WonderlandCycle", serde_json::json!({})),
        _ => return None,
    };

    Some(bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
        name: name.to_string(),
        config,
        uses_linked_cancellation: true,
    })
}

#[derive(Debug, Clone, Default)]
pub struct ScriptDispatcherHost {
    commands: Vec<DispatcherCommand>,
}

impl ScriptDispatcherHost {
    pub fn commands(&self) -> &[DispatcherCommand] {
        &self.commands
    }

    pub fn task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.commands
            .iter()
            .cloned()
            .map(|command| {
                bgi_task::TaskInvocationPlan::from_script_dispatcher_command(command.into())
                    .map_err(Into::into)
            })
            .collect()
    }

    pub fn add_timer(&mut self, mut timer: RealtimeTimerHostPlan) -> DispatcherCommand {
        timer.clears_existing_triggers = true;
        self.commands.push(DispatcherCommand::ClearAllTriggers);
        let command = DispatcherCommand::AddRealtimeTimer(timer);
        self.commands.push(command.clone());
        command
    }

    pub fn add_trigger(&mut self, mut timer: RealtimeTimerHostPlan) -> DispatcherCommand {
        timer.clears_existing_triggers = false;
        let command = DispatcherCommand::AddRealtimeTimer(timer);
        self.commands.push(command.clone());
        command
    }

    pub fn clear_all_triggers(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::ClearAllTriggers;
        self.commands.push(command.clone());
        command
    }

    pub fn run_current_task(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::RunCurrentTask;
        self.commands.push(command.clone());
        command
    }

    pub fn run_solo_task(&mut self, task: SoloTaskHostPlan) -> DispatcherCommand {
        let command = DispatcherCommand::RunSoloTask(task);
        self.commands.push(command.clone());
        command
    }

    pub fn get_linked_cancellation_token_source(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::LinkedCancellationTokenSource;
        self.commands.push(command.clone());
        command
    }

    pub fn get_linked_cancellation_token(&mut self) -> DispatcherCommand {
        let command = DispatcherCommand::LinkedCancellationToken;
        self.commands.push(command.clone());
        command
    }

    pub fn run_builtin_task(&mut self, name: &str, config: Value) -> DispatcherCommand {
        let command = DispatcherCommand::RunBuiltinTask {
            name: name.to_string(),
            config,
            uses_linked_cancellation: true,
        };
        self.commands.push(command.clone());
        command
    }
}

impl From<RealtimeTimerHostPlan> for bgi_task::DispatcherTimerInput {
    fn from(value: RealtimeTimerHostPlan) -> Self {
        Self {
            name: value.name,
            interval_ms: value.interval_ms,
            config: value.config,
            clears_existing_triggers: value.clears_existing_triggers,
        }
    }
}

impl From<SoloTaskHostPlan> for bgi_task::DispatcherSoloTaskInput {
    fn from(value: SoloTaskHostPlan) -> Self {
        Self {
            name: value.name,
            config: value.config,
            uses_linked_cancellation: value.uses_linked_cancellation,
        }
    }
}

impl From<DispatcherCommand> for bgi_task::ScriptDispatcherCommandInput {
    fn from(value: DispatcherCommand) -> Self {
        match value {
            DispatcherCommand::ClearAllTriggers => Self::ClearAllTriggers,
            DispatcherCommand::AddRealtimeTimer(timer) => Self::AddRealtimeTimer(timer.into()),
            DispatcherCommand::RunCurrentTask => Self::RunCurrentTask,
            DispatcherCommand::RunSoloTask(task) => Self::RunSoloTask(task.into()),
            DispatcherCommand::LinkedCancellationTokenSource => Self::LinkedCancellationTokenSource,
            DispatcherCommand::LinkedCancellationToken => Self::LinkedCancellationToken,
            DispatcherCommand::RunBuiltinTask {
                name,
                config,
                uses_linked_cancellation,
            } => Self::RunBuiltinTask {
                name,
                config,
                uses_linked_cancellation,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptHostTarget {
    Global,
    Genshin,
    PathingScript,
    KeyMouseScript,
    File,
    Vision,
    Log,
    Http,
    Dispatcher,
    Notification,
    PostMessage,
    StrategyFile,
    ServerTime,
    HtmlMask,
    KeyMouseHook,
    CustomHostFunctions,
}

impl ScriptHostTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Genshin => "genshin",
            Self::PathingScript => "pathingScript",
            Self::KeyMouseScript => "keyMouseScript",
            Self::File => "file",
            Self::Vision => "vision",
            Self::Log => "log",
            Self::Http => "http",
            Self::Dispatcher => "dispatcher",
            Self::Notification => "notification",
            Self::PostMessage => "PostMessage",
            Self::StrategyFile => "strategyFile",
            Self::ServerTime => "ServerTime",
            Self::HtmlMask => "htmlMask",
            Self::KeyMouseHook => "KeyMouseHook",
            Self::CustomHostFunctions => "host",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptHostCall {
    pub target: ScriptHostTarget,
    pub method: String,
    pub args: Vec<Value>,
}

impl ScriptHostCall {
    pub fn new(target: ScriptHostTarget, method: impl Into<String>, args: Vec<Value>) -> Self {
        Self {
            target,
            method: method.into(),
            args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ScriptHostCallResult {
    None,
    Bool(bool),
    Integer(i64),
    String(String),
    StringList(Vec<String>),
    GameMetrics(GameMetrics),
    CaptureGameRegionPlan(CaptureGameRegionPlan),
    CaptureGameRegionExecution(CaptureGameRegionExecution),
    AvatarRecognitionPlan(AvatarRecognitionPlan),
    ImageMatReadPlan(ImageMatReadPlan),
    ImageMatWritePlan(ImageMatWritePlan),
    ImageMatReadExecution(ImageMatReadExecution),
    ImageMatWriteExecution(ImageMatWriteExecution),
    VisionRecognitionExecution(VisionRecognitionExecution),
    VisionImageMatExecution(VisionImageMatExecution),
    CustomHostFunctionCommand(CustomHostFunctionCommand),
    InputEvents(Vec<InputEvent>),
    InputExecution(GlobalInputExecution),
    PostMessageEvents(Vec<PostMessageEvent>),
    HttpRequestPlan(HttpRequestPlan),
    HttpExecution(HttpExecution),
    DispatcherCommand(DispatcherCommand),
    DispatcherCommands(Vec<DispatcherCommand>),
    GenshinCommand(GenshinCommand),
    GenshinCommands(Vec<GenshinCommand>),
    PathingPlan(PathingScriptRunPlan),
    PathingExecution(PathingScriptExecution),
    KeyMousePlan(KeyMouseScriptRunPlan),
    KeyMouseExecution(KeyMouseScriptExecution),
    HtmlMaskCommand(HtmlMaskCommand),
    HtmlMaskSnapshot(HtmlMaskSnapshot),
    KeyMouseHookCommand(KeyMouseHookCommand),
    KeyMouseHookDispatches(Vec<KeyMouseHookDispatch>),
    KeyMouseHookSnapshot(KeyMouseHookSnapshot),
    LogRecords(Vec<ScriptLogRecord>),
    NotificationExecution(NotificationExecution),
    NotificationRecords(Vec<ScriptNotificationRecord>),
}

#[derive(Clone, Serialize)]
pub struct ScriptHostRuntimeConfig {
    pub script_root: PathBuf,
    pub strategy_root: PathBuf,
    pub user_auto_pathing_root: PathBuf,
    pub pathing_party_config: Option<Value>,
    pub capture_area: GameCaptureArea,
    pub initial_game_metrics: Option<GameMetrics>,
    #[serde(skip)]
    pub capture_frame_source: Option<Arc<dyn GameCaptureFrameSource>>,
    pub runtime_dpi: f64,
    pub input_window_handle: Option<isize>,
    pub global_input_dispatch_mode: GlobalInputDispatchMode,
    pub key_mouse_dispatch_mode: KeyMouseScriptDispatchMode,
    pub macro_playback_context: MacroPlaybackContext,
    #[serde(skip)]
    pub cancellation: Option<Arc<InputCancellationToken>>,
    pub http_dispatch_mode: HttpDispatchMode,
    pub http_policy: ScriptHttpPolicy,
    pub notification_policy: ScriptNotificationPolicy,
    pub notification_dispatch_mode: NotificationDispatchMode,
    pub server_time_zone_offset_milliseconds: i32,
    pub html_mask_initial_state: HtmlMaskInitialState,
}

impl std::fmt::Debug for ScriptHostRuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptHostRuntimeConfig")
            .field("script_root", &self.script_root)
            .field("strategy_root", &self.strategy_root)
            .field("user_auto_pathing_root", &self.user_auto_pathing_root)
            .field("pathing_party_config", &self.pathing_party_config)
            .field("capture_area", &self.capture_area)
            .field("initial_game_metrics", &self.initial_game_metrics)
            .field(
                "capture_frame_source",
                &self
                    .capture_frame_source
                    .as_ref()
                    .map(|_| "<capture-source>"),
            )
            .field("runtime_dpi", &self.runtime_dpi)
            .field("input_window_handle", &self.input_window_handle)
            .field(
                "global_input_dispatch_mode",
                &self.global_input_dispatch_mode,
            )
            .field("key_mouse_dispatch_mode", &self.key_mouse_dispatch_mode)
            .field("macro_playback_context", &self.macro_playback_context)
            .field(
                "cancellation",
                &self.cancellation.as_ref().map(|token| token.is_cancelled()),
            )
            .field("http_dispatch_mode", &self.http_dispatch_mode)
            .field("http_policy", &self.http_policy)
            .field("notification_policy", &self.notification_policy)
            .field(
                "notification_dispatch_mode",
                &self.notification_dispatch_mode,
            )
            .field(
                "server_time_zone_offset_milliseconds",
                &self.server_time_zone_offset_milliseconds,
            )
            .field("html_mask_initial_state", &self.html_mask_initial_state)
            .finish()
    }
}

impl PartialEq for ScriptHostRuntimeConfig {
    fn eq(&self, other: &Self) -> bool {
        self.script_root == other.script_root
            && self.strategy_root == other.strategy_root
            && self.user_auto_pathing_root == other.user_auto_pathing_root
            && self.pathing_party_config == other.pathing_party_config
            && self.capture_area == other.capture_area
            && self.initial_game_metrics == other.initial_game_metrics
            && self.runtime_dpi == other.runtime_dpi
            && self.input_window_handle == other.input_window_handle
            && self.global_input_dispatch_mode == other.global_input_dispatch_mode
            && self.key_mouse_dispatch_mode == other.key_mouse_dispatch_mode
            && self.macro_playback_context == other.macro_playback_context
            && self.http_dispatch_mode == other.http_dispatch_mode
            && self.http_policy == other.http_policy
            && self.notification_policy == other.notification_policy
            && self.notification_dispatch_mode == other.notification_dispatch_mode
            && self.server_time_zone_offset_milliseconds
                == other.server_time_zone_offset_milliseconds
            && self.html_mask_initial_state == other.html_mask_initial_state
    }
}

impl ScriptHostRuntimeConfig {
    pub fn new(script_root: impl Into<PathBuf>, strategy_root: impl Into<PathBuf>) -> Self {
        Self {
            script_root: script_root.into(),
            strategy_root: strategy_root.into(),
            user_auto_pathing_root: PathBuf::from("User").join("AutoPathing"),
            pathing_party_config: None,
            capture_area: GameCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            initial_game_metrics: None,
            capture_frame_source: None,
            runtime_dpi: 1.0,
            input_window_handle: None,
            global_input_dispatch_mode: GlobalInputDispatchMode::PlanOnly,
            key_mouse_dispatch_mode: KeyMouseScriptDispatchMode::PlanOnly,
            macro_playback_context: MacroPlaybackContext::default(),
            cancellation: None,
            http_dispatch_mode: HttpDispatchMode::PlanOnly,
            http_policy: ScriptHttpPolicy::new(false, Vec::<String>::new()),
            notification_policy: ScriptNotificationPolicy::new(true, true),
            notification_dispatch_mode: NotificationDispatchMode::RecordOnly,
            server_time_zone_offset_milliseconds: ServerTimeHost::default()
                .server_time_zone_offset_milliseconds(),
            html_mask_initial_state: HtmlMaskInitialState::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptHostRuntime {
    global_input: GlobalInputHost,
    global_input_dispatch_mode: GlobalInputDispatchMode,
    input_window_handle: Option<isize>,
    genshin: GenshinHost,
    pathing_script: PathingScriptHost,
    key_mouse_script: KeyMouseScriptHost,
    key_mouse_dispatch_mode: KeyMouseScriptDispatchMode,
    cancellation: Option<Arc<InputCancellationToken>>,
    file: LimitedFileHost,
    vision: VisionHost,
    log: ScriptLogHost,
    http: HttpHost,
    http_dispatch_mode: HttpDispatchMode,
    dispatcher: ScriptDispatcherHost,
    notification: ScriptNotificationHost,
    notification_dispatch_mode: NotificationDispatchMode,
    strategy_file: StrategyFileHost,
    server_time: ServerTimeHost,
    html_mask: HtmlMaskHost,
    key_mouse_hook: KeyMouseHookHost,
    logical_time_ms: u64,
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

impl KeyMouseScriptRunPlan {
    pub fn sequence(&self) -> InputSequence {
        self.input_events
            .iter()
            .copied()
            .fold(InputSequence::new(), append_input_event)
    }

    pub fn send(&self) -> Result<()> {
        Ok(send_events(&self.input_events)?)
    }

    pub fn send_to_window(&self, hwnd: isize) -> Result<()> {
        Ok(send_events_to_window(hwnd, &self.input_events)?)
    }

    pub fn send_with_cancellation(
        &self,
        cancellation: &InputCancellationToken,
    ) -> Result<(usize, bool)> {
        match send_events_with_cancellation(&self.input_events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(bgi_input::InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error.into()),
        }
    }

    pub fn send_to_window_with_cancellation(
        &self,
        hwnd: isize,
        cancellation: &InputCancellationToken,
    ) -> Result<(usize, bool)> {
        match send_events_to_window_with_cancellation(hwnd, &self.input_events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(bgi_input::InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error.into()),
        }
    }

    pub fn execute(
        &self,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.execute_with_cancellation(mode, window_handle, None)
    }

    pub fn execute_with_cancellation(
        &self,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        let mut dispatched_events = 0;
        let mut cancelled = false;
        if mode == KeyMouseScriptDispatchMode::SendInput {
            let result = match (window_handle, cancellation) {
                (Some(hwnd), Some(cancellation)) => {
                    self.send_to_window_with_cancellation(hwnd, cancellation)?
                }
                (None, Some(cancellation)) => self.send_with_cancellation(cancellation)?,
                (Some(hwnd), None) => {
                    self.send_to_window(hwnd)?;
                    (self.input_events.len(), false)
                }
                (None, None) => {
                    self.send()?;
                    (self.input_events.len(), false)
                }
            };
            dispatched_events = result.0;
            cancelled = result.1;
        }

        Ok(KeyMouseScriptExecution {
            mode,
            plan: self.clone(),
            dispatched: mode == KeyMouseScriptDispatchMode::SendInput,
            dispatched_events,
            processed_events: if mode == KeyMouseScriptDispatchMode::SendInput {
                dispatched_events
            } else {
                0
            },
            cancelled,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PathingScriptHost {
    script_file_host: LimitedFileHost,
    user_auto_pathing_file_host: LimitedFileHost,
    party_config: Option<Value>,
}

impl PathingScriptHost {
    pub fn new(
        script_root: impl Into<PathBuf>,
        user_auto_pathing_root: impl Into<PathBuf>,
        party_config: Option<Value>,
    ) -> Self {
        Self {
            script_file_host: LimitedFileHost::new(script_root),
            user_auto_pathing_file_host: LimitedFileHost::new(user_auto_pathing_root),
            party_config,
        }
    }

    pub fn run(&self, json: &str) -> Result<PathingScriptRunPlan> {
        self.plan_from_json(json, PathingScriptSource::InlineJson, None)
    }

    pub fn execute(&self, json: &str) -> Result<PathingScriptExecution> {
        self.run(json)?.execute()
    }

    pub fn run_file(&self, path: &str) -> Result<PathingScriptRunPlan> {
        let json = self.script_file_host.read_text_sync(path)?;
        let normalized_path = self.script_file_host.normalize_path(path)?;
        self.plan_from_json(
            &json,
            PathingScriptSource::ScriptFile,
            Some(normalized_path),
        )
    }

    pub fn execute_file(&self, path: &str) -> Result<PathingScriptExecution> {
        self.run_file(path)?.execute()
    }

    pub fn run_file_from_user(&self, path: &str) -> Result<PathingScriptRunPlan> {
        let json = self.user_auto_pathing_file_host.read_text_sync(path)?;
        let normalized_path = self.user_auto_pathing_file_host.normalize_path(path)?;
        self.plan_from_json(
            &json,
            PathingScriptSource::UserAutoPathingFile,
            Some(normalized_path),
        )
    }

    pub fn execute_file_from_user(&self, path: &str) -> Result<PathingScriptExecution> {
        self.run_file_from_user(path)?.execute()
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_exists(path)
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_file(path)
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        self.user_auto_pathing_file_host.is_folder(path)
    }

    pub fn read_path_sync(&self, path: &str) -> Result<Vec<String>> {
        self.user_auto_pathing_file_host.read_path_sync(path)
    }

    fn plan_from_json(
        &self,
        json: &str,
        source: PathingScriptSource,
        normalized_path: Option<PathBuf>,
    ) -> Result<PathingScriptRunPlan> {
        let mut task = PathingTask::from_json(json)?;
        if let Some(path) = &normalized_path {
            task.file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned);
            task.full_path = Some(path.clone());
        }
        let summary = task.summary();
        Ok(PathingScriptRunPlan {
            source,
            normalized_path,
            summary,
            task,
            party_config: self.party_config.clone(),
        })
    }
}

impl PathingScriptRunPlan {
    pub fn execute(self) -> Result<PathingScriptExecution> {
        Ok(PathingScriptExecution {
            execution_plan: self.task.execution_plan(),
            plan: self,
            dispatched: false,
            completed: false,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HtmlMaskHost {
    file_host: LimitedFileHost,
    windows: HashMap<String, HtmlMaskWindowPlan>,
    opened_windows: Vec<String>,
    to_html: HashMap<String, VecDeque<HtmlMaskMessage>>,
    from_html: HashMap<String, VecDeque<HtmlMaskMessage>>,
    commands: Vec<HtmlMaskCommand>,
    next_window_id: u64,
    next_request_id: u64,
}

impl HtmlMaskHost {
    pub fn new(work_dir: impl Into<PathBuf>) -> Self {
        Self {
            file_host: LimitedFileHost::new(work_dir),
            windows: HashMap::new(),
            opened_windows: Vec::new(),
            to_html: HashMap::new(),
            from_html: HashMap::new(),
            commands: Vec::new(),
            next_window_id: 1,
            next_request_id: 1,
        }
    }

    pub fn with_initial_state(
        work_dir: impl Into<PathBuf>,
        initial_state: HtmlMaskInitialState,
    ) -> Self {
        let mut host = Self::new(work_dir);
        for window in initial_state.windows {
            let window_id = window.window_id.clone();
            host.windows.insert(window_id.clone(), window);
            if !host.opened_windows.iter().any(|id| id == &window_id) {
                host.opened_windows.push(window_id.clone());
            }
            host.to_html.entry(window_id.clone()).or_default();
            host.from_html.entry(window_id).or_default();
        }
        for (window_id, message) in initial_state.from_html {
            if !host.windows.contains_key(&window_id) {
                continue;
            }
            host.from_html
                .entry(window_id)
                .or_default()
                .push_back(message);
        }
        host
    }

    pub fn commands(&self) -> &[HtmlMaskCommand] {
        &self.commands
    }

    pub fn remaining_from_html_messages(&self) -> Vec<(String, HtmlMaskMessage)> {
        let mut messages = self
            .from_html
            .iter()
            .flat_map(|(window_id, queue)| {
                queue
                    .iter()
                    .cloned()
                    .map(|message| (window_id.clone(), message))
            })
            .collect::<Vec<_>>();
        messages.sort_by(|left, right| left.0.cmp(&right.0));
        messages
    }

    pub fn snapshot(&self) -> HtmlMaskSnapshot {
        let mut windows = self.windows.values().cloned().collect::<Vec<_>>();
        windows.sort_by(|left, right| left.window_id.cmp(&right.window_id));
        HtmlMaskSnapshot {
            windows,
            commands: self.commands.clone(),
            to_html_queue_count: self.to_html.values().map(VecDeque::len).sum(),
            from_html_queue_count: self.from_html.values().map(VecDeque::len).sum(),
        }
    }

    pub fn show(&mut self, url: &str, id: Option<&str>) -> Result<HtmlMaskCommand> {
        if url.trim().is_empty() {
            return Err(invalid_arg_for_method("htmlMask.show", 0, "non-empty URL"));
        }

        let (final_url, normalized_path) = if is_http_url(url) {
            (url.to_string(), None)
        } else {
            let normalized = self.file_host.normalize_path(url)?;
            (path_to_file_url(&normalized), Some(normalized))
        };
        let window_id = id
            .filter(|id| !id.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.next_window_id());
        let plan = HtmlMaskWindowPlan {
            window_id: window_id.clone(),
            final_url,
            requested_url: url.to_string(),
            normalized_path,
            click_through: false,
        };

        self.windows.insert(window_id.clone(), plan.clone());
        if !self.opened_windows.iter().any(|id| id == &window_id) {
            self.opened_windows.push(window_id.clone());
        }
        self.to_html.entry(window_id.clone()).or_default();
        self.from_html.entry(window_id).or_default();
        Ok(self.push_html_mask_command(HtmlMaskCommand::Show(plan)))
    }

    pub fn close(&mut self, window_id: &str) -> HtmlMaskCommand {
        self.opened_windows.retain(|id| id != window_id);
        self.windows.remove(window_id);
        self.to_html.remove(window_id);
        self.from_html.remove(window_id);
        self.push_html_mask_command(HtmlMaskCommand::Close {
            window_id: window_id.to_string(),
        })
    }

    pub fn close_all(&mut self) -> HtmlMaskCommand {
        let window_ids = self.opened_windows.clone();
        self.opened_windows.clear();
        self.windows.clear();
        self.to_html.clear();
        self.from_html.clear();
        self.push_html_mask_command(HtmlMaskCommand::CloseAll { window_ids })
    }

    pub fn window_ids(&self) -> Vec<String> {
        let mut ids = self.windows.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn exists(&self, window_id: &str) -> bool {
        self.windows.contains_key(window_id)
    }

    pub fn set_click_through(&mut self, window_id: &str, enabled: bool) -> Result<HtmlMaskCommand> {
        let Some(window) = self.windows.get_mut(window_id) else {
            return Err(invalid_arg_for_method(
                "htmlMask.setClickThrough",
                0,
                "existing window id",
            ));
        };
        window.click_through = enabled;
        Ok(
            self.push_html_mask_command(HtmlMaskCommand::SetClickThrough {
                window_id: window_id.to_string(),
                enabled,
            }),
        )
    }

    pub fn get_click_through(&self, window_id: &str) -> Result<bool> {
        self.windows
            .get(window_id)
            .map(|window| window.click_through)
            .ok_or_else(|| {
                invalid_arg_for_method("htmlMask.getClickThrough", 0, "existing window id")
            })
    }

    pub fn toggle_click_through(&mut self, window_id: &str) -> Result<HtmlMaskCommand> {
        let enabled = !self.get_click_through(window_id)?;
        self.set_click_through(window_id, enabled)
    }

    pub fn send(&mut self, window_id: &str, url: &str, json_data: &str) -> Result<HtmlMaskCommand> {
        self.ensure_window(window_id, "htmlMask.send")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: None,
        };
        self.to_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message.clone());
        Ok(self.push_html_mask_command(HtmlMaskCommand::Send {
            window_id: window_id.to_string(),
            message,
        }))
    }

    pub fn request(
        &mut self,
        window_id: &str,
        url: &str,
        json_data: &str,
        timeout_ms: u64,
    ) -> Result<HtmlMaskCommand> {
        self.ensure_window(window_id, "htmlMask.request")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: Some(self.next_request_id()),
        };
        self.to_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message.clone());
        Ok(self.push_html_mask_command(HtmlMaskCommand::Request {
            window_id: window_id.to_string(),
            message,
            timeout_ms,
        }))
    }

    pub fn receive(&mut self, window_id: &str, _timeout_ms: u64) -> Result<Option<String>> {
        self.ensure_window(window_id, "htmlMask.receive")?;
        self.poll(window_id)
    }

    pub fn poll(&mut self, window_id: &str) -> Result<Option<String>> {
        self.ensure_window(window_id, "htmlMask.poll")?;
        let Some(queue) = self.from_html.get_mut(window_id) else {
            return Ok(None);
        };
        queue
            .pop_front()
            .map(|message| serialize_html_mask_message(&message))
            .transpose()
    }

    pub fn poll_all(&mut self, window_id: &str) -> Result<String> {
        self.ensure_window(window_id, "htmlMask.pollAll")?;
        let Some(queue) = self.from_html.get_mut(window_id) else {
            return Ok("[]".to_string());
        };
        let messages = queue.drain(..).collect::<Vec<_>>();
        serialize_html_mask_messages(&messages)
    }

    pub fn flush_pending_messages(&mut self, window_id: &str) -> Result<Vec<String>> {
        self.ensure_window(window_id, "htmlMask.flushPendingMessages")?;
        let Some(queue) = self.to_html.get_mut(window_id) else {
            return Ok(Vec::new());
        };
        queue
            .drain(..)
            .map(|message| serialize_html_mask_message(&message))
            .collect()
    }

    pub fn send_from_html(
        &mut self,
        window_id: &str,
        url: &str,
        json_data: &str,
        request_id: Option<&str>,
    ) -> Result<()> {
        self.ensure_window(window_id, "htmlMask.sendFromHtml")?;
        let message = HtmlMaskMessage {
            url: url.to_string(),
            data: parse_html_mask_data(json_data)?,
            request_id: request_id.map(ToOwned::to_owned),
        };
        self.from_html
            .entry(window_id.to_string())
            .or_default()
            .push_back(message);
        Ok(())
    }

    fn ensure_window(&self, window_id: &str, method: &'static str) -> Result<()> {
        if self.windows.contains_key(window_id) {
            Ok(())
        } else {
            Err(invalid_arg_for_method(method, 0, "existing window id"))
        }
    }

    fn next_window_id(&mut self) -> String {
        let id = format!("html-mask-{}", self.next_window_id);
        self.next_window_id = self.next_window_id.saturating_add(1);
        id
    }

    fn next_request_id(&mut self) -> String {
        let id = format!("request-{}", self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        id
    }

    fn push_html_mask_command(&mut self, command: HtmlMaskCommand) -> HtmlMaskCommand {
        self.commands.push(command.clone());
        command
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeyMouseHookHost {
    listeners: Vec<KeyMouseHookListener>,
    commands: Vec<KeyMouseHookCommand>,
    last_global_mouse_move_ms: Option<u64>,
    last_listener_mouse_move_ms: HashMap<String, u64>,
    next_listener_id: u64,
    disposed: bool,
}

impl KeyMouseHookHost {
    pub fn listeners(&self) -> &[KeyMouseHookListener] {
        &self.listeners
    }

    pub fn commands(&self) -> &[KeyMouseHookCommand] {
        &self.commands
    }

    pub fn snapshot(&self) -> KeyMouseHookSnapshot {
        KeyMouseHookSnapshot {
            listeners: self.listeners.clone(),
            commands: self.commands.clone(),
            disposed: self.disposed,
        }
    }

    pub fn on_key_down(
        &mut self,
        callback_id: Option<&str>,
        use_code_only: bool,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::KeyDown,
            callback_id,
            use_code_only,
            None,
        )
    }

    pub fn on_key_up(
        &mut self,
        callback_id: Option<&str>,
        use_code_only: bool,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::KeyUp,
            callback_id,
            use_code_only,
            None,
        )
    }

    pub fn on_mouse_down(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseDown, callback_id, true, None)
    }

    pub fn on_mouse_up(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseUp, callback_id, true, None)
    }

    pub fn on_mouse_move(
        &mut self,
        callback_id: Option<&str>,
        interval_ms: u64,
    ) -> KeyMouseHookCommand {
        self.add_listener(
            KeyMouseHookEventKind::MouseMove,
            callback_id,
            true,
            Some(interval_ms),
        )
    }

    pub fn on_mouse_wheel(&mut self, callback_id: Option<&str>) -> KeyMouseHookCommand {
        self.add_listener(KeyMouseHookEventKind::MouseWheel, callback_id, true, None)
    }

    pub fn remove_all_listeners(&mut self) -> KeyMouseHookCommand {
        self.listeners.clear();
        self.last_listener_mouse_move_ms.clear();
        let command = KeyMouseHookCommand::RemoveAllListeners;
        self.commands.push(command.clone());
        command
    }

    pub fn dispose(&mut self) -> KeyMouseHookCommand {
        self.remove_all_listeners();
        self.disposed = true;
        let command = KeyMouseHookCommand::Dispose;
        self.commands.push(command.clone());
        command
    }

    pub fn dispatch_event(&mut self, event: KeyMouseHookEvent) -> Vec<KeyMouseHookDispatch> {
        if self.disposed {
            return Vec::new();
        }

        let event_kind = event.kind();
        if let KeyMouseHookEvent::MouseMove { timestamp_ms, .. } = event {
            if let Some(last) = self.last_global_mouse_move_ms {
                if timestamp_ms.saturating_sub(last) < 10 {
                    return Vec::new();
                }
            }
            self.last_global_mouse_move_ms = Some(timestamp_ms);
        }

        let listeners = self
            .listeners
            .iter()
            .filter(|listener| listener.event == event_kind)
            .cloned()
            .collect::<Vec<_>>();
        listeners
            .into_iter()
            .filter_map(|listener| self.dispatch_to_listener(&listener, &event))
            .collect()
    }

    fn add_listener(
        &mut self,
        event: KeyMouseHookEventKind,
        callback_id: Option<&str>,
        use_code_only: bool,
        interval_ms: Option<u64>,
    ) -> KeyMouseHookCommand {
        self.disposed = false;
        let listener = KeyMouseHookListener {
            id: callback_id
                .filter(|id| !id.trim().is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| self.next_listener_id()),
            event,
            use_code_only,
            interval_ms,
        };
        let command = KeyMouseHookCommand::AddListener(listener.clone());
        self.listeners.push(listener);
        self.commands.push(command.clone());
        command
    }

    fn dispatch_to_listener(
        &mut self,
        listener: &KeyMouseHookListener,
        event: &KeyMouseHookEvent,
    ) -> Option<KeyMouseHookDispatch> {
        let args = match event {
            KeyMouseHookEvent::Key {
                key_data, key_code, ..
            } => vec![Value::String(if listener.use_code_only {
                key_code.clone()
            } else {
                key_data.clone()
            })],
            KeyMouseHookEvent::MouseButton { button, x, y, .. } => vec![
                Value::String(mouse_button_name(*button).to_string()),
                serde_json::json!(x),
                serde_json::json!(y),
            ],
            KeyMouseHookEvent::MouseMove { x, y, timestamp_ms } => {
                let interval = listener.interval_ms.unwrap_or(200);
                let last = self.last_listener_mouse_move_ms.get(&listener.id).copied();
                if last
                    .map(|last| timestamp_ms.saturating_sub(last) < interval)
                    .unwrap_or(false)
                {
                    return None;
                }
                self.last_listener_mouse_move_ms
                    .insert(listener.id.clone(), *timestamp_ms);
                vec![serde_json::json!(x), serde_json::json!(y)]
            }
            KeyMouseHookEvent::MouseWheel { delta, x, y } => {
                vec![
                    serde_json::json!(delta),
                    serde_json::json!(x),
                    serde_json::json!(y),
                ]
            }
        };
        Some(KeyMouseHookDispatch {
            listener_id: listener.id.clone(),
            event: listener.event,
            args,
        })
    }

    fn next_listener_id(&mut self) -> String {
        let id = format!("listener-{}", self.next_listener_id);
        self.next_listener_id = self.next_listener_id.saturating_add(1);
        id
    }
}

impl KeyMouseHookEvent {
    fn kind(&self) -> KeyMouseHookEventKind {
        match self {
            Self::Key { event, .. } => *event,
            Self::MouseButton { event, .. } => *event,
            Self::MouseMove { .. } => KeyMouseHookEventKind::MouseMove,
            Self::MouseWheel { .. } => KeyMouseHookEventKind::MouseWheel,
        }
    }
}

impl ScriptHostRuntime {
    pub fn new(config: ScriptHostRuntimeConfig) -> Result<Self> {
        let mut global_input = GlobalInputHost::new_with_frame_source(
            config.capture_area,
            config.runtime_dpi,
            config.capture_frame_source,
        )?;
        if let Some(metrics) = config.initial_game_metrics {
            global_input.set_game_metrics(metrics.width, metrics.height, metrics.dpi)?;
        }

        Ok(Self {
            global_input,
            global_input_dispatch_mode: config.global_input_dispatch_mode,
            input_window_handle: config.input_window_handle,
            genshin: GenshinHost::default(),
            pathing_script: PathingScriptHost::new(
                config.script_root.clone(),
                config.user_auto_pathing_root,
                config.pathing_party_config,
            ),
            key_mouse_script: KeyMouseScriptHost::new(
                config.script_root.clone(),
                config.macro_playback_context,
            ),
            key_mouse_dispatch_mode: config.key_mouse_dispatch_mode,
            cancellation: config.cancellation,
            file: LimitedFileHost::new(config.script_root.clone()),
            vision: VisionHost::default(),
            log: ScriptLogHost::default(),
            http: HttpHost::new(config.http_policy),
            http_dispatch_mode: config.http_dispatch_mode,
            dispatcher: ScriptDispatcherHost::default(),
            notification: ScriptNotificationHost::new(config.notification_policy),
            notification_dispatch_mode: config.notification_dispatch_mode,
            strategy_file: StrategyFileHost::new(config.strategy_root),
            server_time: ServerTimeHost::from_offset_milliseconds(
                config.server_time_zone_offset_milliseconds,
            ),
            html_mask: HtmlMaskHost::with_initial_state(
                config.script_root,
                config.html_mask_initial_state,
            ),
            key_mouse_hook: KeyMouseHookHost::default(),
            logical_time_ms: 0,
        })
    }

    pub fn call(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        self.logical_time_ms = self.logical_time_ms.saturating_add(1);
        self.call_at(call, self.logical_time_ms)
    }

    pub fn call_at(&mut self, call: ScriptHostCall, now_ms: u64) -> Result<ScriptHostCallResult> {
        match call.target {
            ScriptHostTarget::Global => self.call_global(call),
            ScriptHostTarget::Genshin => self.call_genshin(call),
            ScriptHostTarget::PathingScript => self.call_pathing_script(call),
            ScriptHostTarget::KeyMouseScript => self.call_key_mouse_script(call),
            ScriptHostTarget::File => self.call_file(call),
            ScriptHostTarget::Vision => self.call_vision(call),
            ScriptHostTarget::Log => self.call_log(call),
            ScriptHostTarget::Http => self.call_http(call),
            ScriptHostTarget::Dispatcher => self.call_dispatcher(call),
            ScriptHostTarget::Notification => self.call_notification(call, now_ms),
            ScriptHostTarget::PostMessage => self.call_post_message(call),
            ScriptHostTarget::StrategyFile => self.call_strategy_file(call),
            ScriptHostTarget::ServerTime => self.call_server_time(call),
            ScriptHostTarget::HtmlMask => self.call_html_mask(call),
            ScriptHostTarget::KeyMouseHook => self.call_key_mouse_hook(call),
            ScriptHostTarget::CustomHostFunctions => self.call_custom_host_functions(call),
        }
    }

    pub fn log_records(&self) -> &[ScriptLogRecord] {
        self.log.records()
    }

    pub fn notification_records(&self) -> &[ScriptNotificationRecord] {
        self.notification.records()
    }

    pub fn dispatcher_commands(&self) -> &[DispatcherCommand] {
        self.dispatcher.commands()
    }

    pub fn dispatcher_task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.dispatcher.task_invocation_plans()
    }

    pub fn genshin_commands(&self) -> &[GenshinCommand] {
        self.genshin.commands()
    }

    pub fn genshin_task_invocation_plans(&self) -> Result<Vec<bgi_task::TaskInvocationPlan>> {
        self.genshin.task_invocation_plans()
    }

    pub fn game_metrics(&self) -> GameMetrics {
        self.global_input.game_metrics()
    }

    pub fn html_mask_remaining_from_html_messages(&self) -> Vec<(String, HtmlMaskMessage)> {
        self.html_mask.remaining_from_html_messages()
    }

    fn call_global(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "sleep" | "Sleep" => {
                self.global_input_result(InputSequence::new().delay(arg_u64(&call, 0)?))
            }
            "getVersion" | "GetVersion" => Ok(ScriptHostCallResult::String(
                env!("CARGO_PKG_VERSION").to_string(),
            )),
            "keyDown" | "KeyDown" => {
                self.global_input_result(self.global_input.key_down(arg_str(&call, 0)?)?)
            }
            "keyUp" | "KeyUp" => {
                self.global_input_result(self.global_input.key_up(arg_str(&call, 0)?)?)
            }
            "keyPress" | "KeyPress" => {
                self.global_input_result(self.global_input.key_press(arg_str(&call, 0)?)?)
            }
            "setGameMetrics" | "SetGameMetrics" => {
                let dpi = optional_f64(&call, 2)?.unwrap_or(1.0);
                self.global_input
                    .set_game_metrics(arg_u32(&call, 0)?, arg_u32(&call, 1)?, dpi)?;
                Ok(ScriptHostCallResult::None)
            }
            "getGameMetrics" | "GetGameMetrics" => Ok(ScriptHostCallResult::GameMetrics(
                self.global_input.game_metrics(),
            )),
            "moveMouseBy" | "MoveMouseBy" => self.global_input_result(
                self.global_input
                    .move_mouse_by(arg_i32(&call, 0)?, arg_i32(&call, 1)?),
            ),
            "moveMouseTo" | "MoveMouseTo" => self.global_input_result(
                self.global_input
                    .move_mouse_to(arg_i32(&call, 0)?, arg_i32(&call, 1)?)?,
            ),
            "click" | "Click" => self.global_input_result(
                self.global_input
                    .click(arg_i32(&call, 0)?, arg_i32(&call, 1)?)?,
            ),
            "leftButtonClick" | "LeftButtonClick" => {
                self.global_input_result(self.global_input.left_button_click())
            }
            "leftButtonDown" | "LeftButtonDown" => {
                self.global_input_result(self.global_input.left_button_down())
            }
            "leftButtonUp" | "LeftButtonUp" => {
                self.global_input_result(self.global_input.left_button_up())
            }
            "rightButtonClick" | "RightButtonClick" => {
                self.global_input_result(self.global_input.right_button_click())
            }
            "rightButtonDown" | "RightButtonDown" => {
                self.global_input_result(self.global_input.right_button_down())
            }
            "rightButtonUp" | "RightButtonUp" => {
                self.global_input_result(self.global_input.right_button_up())
            }
            "middleButtonClick" | "MiddleButtonClick" => {
                self.global_input_result(self.global_input.middle_button_click())
            }
            "middleButtonDown" | "MiddleButtonDown" => {
                self.global_input_result(self.global_input.middle_button_down())
            }
            "middleButtonUp" | "MiddleButtonUp" => {
                self.global_input_result(self.global_input.middle_button_up())
            }
            "verticalScroll" | "VerticalScroll" => {
                self.global_input_result(self.global_input.vertical_scroll(arg_i32(&call, 0)?))
            }
            "captureGameRegion" | "CaptureGameRegion" => {
                if let Some(execution) = self.global_input.capture_game_region_execution()? {
                    Ok(ScriptHostCallResult::CaptureGameRegionExecution(execution))
                } else {
                    Ok(ScriptHostCallResult::CaptureGameRegionPlan(
                        self.global_input.capture_game_region(),
                    ))
                }
            }
            "getAvatars" | "GetAvatars" => Ok(ScriptHostCallResult::AvatarRecognitionPlan(
                self.global_input.get_avatars(),
            )),
            "inputText" | "InputText" => {
                self.global_input_result(self.global_input.input_text(arg_str(&call, 0)?))
            }
            _ => Err(unknown_method(&call)),
        }
    }

    fn global_input_result(&self, sequence: InputSequence) -> Result<ScriptHostCallResult> {
        Ok(ScriptHostCallResult::InputExecution(
            GlobalInputExecution::execute(
                sequence,
                self.global_input_dispatch_mode,
                self.input_window_handle,
            )?,
        ))
    }

    fn call_genshin(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "uid" | "Uid" => GenshinCommand::Uid,
            "tp" | "Tp" => {
                let (map_name, force) = match call.args.get(2) {
                    None | Some(Value::Null) => (None, false),
                    Some(Value::String(map_name)) => (
                        Some(map_name.clone()),
                        optional_bool(&call, 3)?.unwrap_or(false),
                    ),
                    Some(Value::Bool(force)) => (None, *force),
                    Some(_) => return Err(invalid_arg(&call, 2, "string or bool")),
                };
                GenshinCommand::Teleport {
                    x: arg_f64_like(&call, 0)?,
                    y: arg_f64_like(&call, 1)?,
                    map_name,
                    force,
                }
            }
            "moveMapTo" | "MoveMapTo" => GenshinCommand::MoveMapTo {
                x: arg_f64_like(&call, 0)?,
                y: arg_f64_like(&call, 1)?,
                map_name: None,
                force_country: optional_str(&call, 2)?.map(ToOwned::to_owned),
            },
            "moveIndependentMapTo" | "MoveIndependentMapTo" => GenshinCommand::MoveMapTo {
                x: arg_f64_like(&call, 0)?,
                y: arg_f64_like(&call, 1)?,
                map_name: Some(arg_str(&call, 2)?.to_string()),
                force_country: optional_str(&call, 3)?.map(ToOwned::to_owned),
            },
            "getBigMapZoomLevel" | "GetBigMapZoomLevel" => GenshinCommand::GetBigMapZoomLevel,
            "setBigMapZoomLevel" | "SetBigMapZoomLevel" => GenshinCommand::SetBigMapZoomLevel {
                zoom_level: arg_f64_like(&call, 0)?,
            },
            "tpToStatueOfTheSeven" | "TpToStatueOfTheSeven" => GenshinCommand::TpToStatueOfTheSeven,
            "getPositionFromBigMap" | "GetPositionFromBigMap" => {
                GenshinCommand::GetPositionFromBigMap {
                    map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                }
            }
            "getPositionFromMap" | "GetPositionFromMap" => {
                let nearby = if call.args.len() >= 3 {
                    Some((arg_f64_like(&call, 1)?, arg_f64_like(&call, 2)?))
                } else {
                    None
                };
                GenshinCommand::GetPositionFromMap {
                    map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                    cache_time_ms: if nearby.is_some() {
                        None
                    } else {
                        optional_u64(&call, 1)?
                    },
                    matching_method: None,
                    nearby,
                }
            }
            "getPositionFromMapWithMatchingMethod" | "GetPositionFromMapWithMatchingMethod" => {
                if call.args.len() == 1 {
                    GenshinCommand::GetPositionFromMap {
                        map_name: None,
                        matching_method: Some(arg_str(&call, 0)?.to_string()),
                        cache_time_ms: None,
                        nearby: None,
                    }
                } else {
                    GenshinCommand::GetPositionFromMap {
                        map_name: optional_str(&call, 0)?.map(ToOwned::to_owned),
                        matching_method: Some(arg_str(&call, 1)?.to_string()),
                        cache_time_ms: optional_u64(&call, 2)?,
                        nearby: None,
                    }
                }
            }
            "getCameraOrientation" | "GetCameraOrientation" => GenshinCommand::GetCameraOrientation,
            "switchParty" | "SwitchParty" => GenshinCommand::SwitchParty {
                party_name: arg_str(&call, 0)?.to_string(),
            },
            "clearPartyCache" | "ClearPartyCache" => GenshinCommand::ClearPartyCache,
            "blessingOfTheWelkinMoon" | "BlessingOfTheWelkinMoon" => {
                GenshinCommand::BlessingOfTheWelkinMoon
            }
            "chooseTalkOption" | "ChooseTalkOption" => GenshinCommand::ChooseTalkOption {
                option: arg_str(&call, 0)?.to_string(),
                skip_times: optional_u32_like(&call, 1)?.unwrap_or(10),
                is_orange: optional_bool(&call, 2)?.unwrap_or(false),
            },
            "claimBattlePassRewards" | "ClaimBattlePassRewards" => {
                GenshinCommand::ClaimBattlePassRewards
            }
            "claimEncounterPointsRewards" | "ClaimEncounterPointsRewards" => {
                GenshinCommand::ClaimEncounterPointsRewards
            }
            "goToAdventurersGuild" | "GoToAdventurersGuild" => {
                GenshinCommand::GoToAdventurersGuild {
                    country: arg_str(&call, 0)?.to_string(),
                }
            }
            "goToCraftingBench" | "GoToCraftingBench" => GenshinCommand::GoToCraftingBench {
                country: arg_str(&call, 0)?.to_string(),
            },
            "returnMainUi" | "ReturnMainUi" => GenshinCommand::ReturnMainUi,
            "autoFishing" | "AutoFishing" => GenshinCommand::AutoFishing {
                fishing_time_policy: optional_i32(&call, 0)?.unwrap_or(0),
            },
            "relogin" | "Relogin" => GenshinCommand::Relogin,
            "wonderlandCycle" | "WonderlandCycle" => GenshinCommand::WonderlandCycle,
            "setTime" | "SetTime" => {
                let hour = arg_u32_like(&call, 0)?;
                let minute = arg_u32_like(&call, 1)?;
                if hour > 23 {
                    return Err(invalid_arg(&call, 0, "hour 0..=23"));
                }
                if minute > 59 {
                    return Err(invalid_arg(&call, 1, "minute 0..=59"));
                }
                GenshinCommand::SetTime {
                    hour,
                    minute,
                    skip: optional_bool(&call, 2)?.unwrap_or(false),
                }
            }
            "commands" | "Commands" => {
                return Ok(ScriptHostCallResult::GenshinCommands(
                    self.genshin.commands().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::GenshinCommand(
            self.genshin.push(command),
        ))
    }

    fn call_pathing_script(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "run" | "Run" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script.execute(arg_str(&call, 0)?)?,
            )),
            "runFile" | "RunFile" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script.execute_file(arg_str(&call, 0)?)?,
            )),
            "runFileFromUser" | "RunFileFromUser" => Ok(ScriptHostCallResult::PathingExecution(
                self.pathing_script
                    .execute_file_from_user(arg_str(&call, 0)?)?,
            )),
            "plan" | "Plan" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run(arg_str(&call, 0)?)?,
            )),
            "planFile" | "PlanFile" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run_file(arg_str(&call, 0)?)?,
            )),
            "planFileFromUser" | "PlanFileFromUser" => Ok(ScriptHostCallResult::PathingPlan(
                self.pathing_script.run_file_from_user(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_exists(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_file(arg_str(&call, 0)?)?,
            )),
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.pathing_script.is_folder(arg_str(&call, 0)?)?,
            )),
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.pathing_script
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_key_mouse_script(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let cancellation = self.cancellation.as_deref();
        match call.method.as_str() {
            "run" | "Run" => Ok(ScriptHostCallResult::KeyMouseExecution(
                self.key_mouse_script.execute_with_cancellation(
                    arg_str(&call, 0)?,
                    self.key_mouse_dispatch_mode,
                    self.input_window_handle,
                    cancellation,
                )?,
            )),
            "runFile" | "RunFile" => Ok(ScriptHostCallResult::KeyMouseExecution(
                self.key_mouse_script.execute_file_with_cancellation(
                    arg_str(&call, 0)?,
                    self.key_mouse_dispatch_mode,
                    self.input_window_handle,
                    cancellation,
                )?,
            )),
            "plan" | "Plan" => Ok(ScriptHostCallResult::KeyMousePlan(
                self.key_mouse_script.run(arg_str(&call, 0)?)?,
            )),
            "planFile" | "PlanFile" => Ok(ScriptHostCallResult::KeyMousePlan(
                self.key_mouse_script.run_file(arg_str(&call, 0)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_file(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.file
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            "createDirectory" | "CreateDirectory" => Ok(ScriptHostCallResult::Bool(
                self.file.create_directory(arg_str(&call, 0)?)?,
            )),
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.file.is_folder(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.file.is_file(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.file.is_exists(arg_str(&call, 0)?)?,
            )),
            "readTextSync" | "ReadTextSync" | "readText" | "ReadText" => Ok(
                ScriptHostCallResult::String(self.file.read_text_sync(arg_str(&call, 0)?)?),
            ),
            "writeTextSync" | "WriteTextSync" | "writeText" | "WriteText" => {
                Ok(ScriptHostCallResult::Bool(self.file.write_text_sync(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    optional_bool(&call, 2)?.unwrap_or(false),
                )?))
            }
            "readImageMatSync" | "ReadImageMatSync" => {
                Ok(ScriptHostCallResult::ImageMatReadExecution(
                    self.file.read_image_mat_sync(arg_str(&call, 0)?)?,
                ))
            }
            "readImageMatWithResizeSync" | "ReadImageMatWithResizeSync" => {
                Ok(ScriptHostCallResult::ImageMatReadExecution(
                    self.file.read_image_mat_with_resize_sync(
                        arg_str(&call, 0)?,
                        arg_f64_like(&call, 1)?,
                        arg_f64_like(&call, 2)?,
                        optional_i32(&call, 3)?.unwrap_or(1),
                    )?,
                ))
            }
            "writeImageSync" | "WriteImageSync" => {
                Ok(ScriptHostCallResult::ImageMatWriteExecution(
                    self.file
                        .write_image_sync(arg_str(&call, 0)?, arg_owned_value(&call, 1)?)?,
                ))
            }
            "renamePathSync" | "RenamePathSync" => Ok(ScriptHostCallResult::Bool(
                self.file
                    .rename_path_sync(arg_str(&call, 0)?, arg_str(&call, 1)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_vision(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "findTemplate" | "FindTemplate" => Ok(
                ScriptHostCallResult::VisionRecognitionExecution(self.vision.find_template(
                    arg_owned_value(&call, 0)?,
                    arg_owned_value(&call, 1)?,
                    optional_owned_value(&call, 2),
                )?),
            ),
            "findColor" | "FindColor" => Ok(ScriptHostCallResult::VisionRecognitionExecution(
                self.vision
                    .find_color(arg_owned_value(&call, 0)?, optional_owned_value(&call, 1))?,
            )),
            "crop" | "Crop" => Ok(ScriptHostCallResult::VisionImageMatExecution(
                self.vision
                    .crop(arg_owned_value(&call, 0)?, arg_owned_value(&call, 1)?)?,
            )),
            "to1080p" | "To1080p" => Ok(ScriptHostCallResult::VisionImageMatExecution(
                self.vision.to_1080p(arg_owned_value(&call, 0)?)?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_log(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "debug" | "Debug" => self.log.debug(arg_str(&call, 0)?),
            "info" | "Info" => self.log.info(arg_str(&call, 0)?),
            "warn" | "Warn" => self.log.warn(arg_str(&call, 0)?),
            "error" | "Error" => self.log.error(arg_str(&call, 0)?),
            "records" | "Records" => {
                return Ok(ScriptHostCallResult::LogRecords(
                    self.log.records().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        }
        Ok(ScriptHostCallResult::None)
    }

    fn call_http(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let mut client = ReqwestScriptHttpClient::new();
        self.call_http_with_client(call, &mut client)
    }

    fn call_http_with_client<C: ScriptHttpClient>(
        &self,
        call: ScriptHostCall,
        client: &mut C,
    ) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "request" | "Request" => {
                let request = self.http.request(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    optional_str(&call, 2)?,
                    optional_str(&call, 3)?,
                )?;
                match self.http_dispatch_mode {
                    HttpDispatchMode::PlanOnly => {
                        Ok(ScriptHostCallResult::HttpRequestPlan(request))
                    }
                    HttpDispatchMode::Reqwest => {
                        let response = client.send(request.clone())?;
                        Ok(ScriptHostCallResult::HttpExecution(HttpExecution {
                            mode: self.http_dispatch_mode,
                            request,
                            response: Some(response),
                            dispatched: true,
                        }))
                    }
                }
            }
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_dispatcher(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "addTimer" | "AddTimer" => self
                .dispatcher
                .add_timer(timer_plan_from_arg(&call, 0, true)?),
            "addTrigger" | "AddTrigger" => self
                .dispatcher
                .add_trigger(timer_plan_from_arg(&call, 0, false)?),
            "clearAllTriggers" | "ClearAllTriggers" => self.dispatcher.clear_all_triggers(),
            "runTask" | "RunTask" if call.args.is_empty() => self.dispatcher.run_current_task(),
            "runTask" | "RunTask" => self
                .dispatcher
                .run_solo_task(solo_task_plan_from_arg(&call, 0)?),
            "getLinkedCancellationTokenSource" | "GetLinkedCancellationTokenSource" => {
                self.dispatcher.get_linked_cancellation_token_source()
            }
            "getLinkedCancellationToken" | "GetLinkedCancellationToken" => {
                self.dispatcher.get_linked_cancellation_token()
            }
            "runAutoDomainTask" | "RunAutoDomainTask" => self
                .dispatcher
                .run_builtin_task("AutoDomain", arg_owned_value(&call, 0)?),
            "runAutoBossTask" | "RunAutoBossTask" => self
                .dispatcher
                .run_builtin_task("AutoBoss", arg_owned_value(&call, 0)?),
            "runAutoFightTask" | "RunAutoFightTask" => self
                .dispatcher
                .run_builtin_task("AutoFight", arg_owned_value(&call, 0)?),
            "runAutoLeyLineOutcropTask" | "RunAutoLeyLineOutcropTask" => self
                .dispatcher
                .run_builtin_task("AutoLeyLineOutcrop", arg_owned_value(&call, 0)?),
            "runAutoStygianOnslaughtTask" | "RunAutoStygianOnslaughtTask" => self
                .dispatcher
                .run_builtin_task("AutoStygianOnslaught", arg_owned_value(&call, 0)?),
            "commands" | "Commands" => {
                return Ok(ScriptHostCallResult::DispatcherCommands(
                    self.dispatcher.commands().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::DispatcherCommand(command))
    }

    fn call_notification(
        &mut self,
        call: ScriptHostCall,
        now_ms: u64,
    ) -> Result<ScriptHostCallResult> {
        let mut sink = RecordingNotificationSink::default();
        self.call_notification_with_sink(call, now_ms, &mut sink)
    }

    fn call_notification_with_sink<S: ScriptNotificationSink>(
        &mut self,
        call: ScriptHostCall,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "send" | "Send" | "success" | "Success" => {
                return self.notification_result(
                    ScriptNotificationKind::Success,
                    arg_str(&call, 0)?,
                    now_ms,
                    sink,
                );
            }
            "error" | "Error" => {
                return self.notification_result(
                    ScriptNotificationKind::Error,
                    arg_str(&call, 0)?,
                    now_ms,
                    sink,
                );
            }
            "records" | "Records" => {
                return Ok(ScriptHostCallResult::NotificationRecords(
                    self.notification.records().to_vec(),
                ));
            }
            _ => return Err(unknown_method(&call)),
        }
    }

    fn notification_result<S: ScriptNotificationSink>(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptHostCallResult> {
        let (record, delivery, dispatched) = match self.notification_dispatch_mode {
            NotificationDispatchMode::RecordOnly => {
                let record = match kind {
                    ScriptNotificationKind::Success => {
                        self.notification.send_at(message, now_ms)?
                    }
                    ScriptNotificationKind::Error => self.notification.error_at(message, now_ms)?,
                };
                (record, None, false)
            }
            NotificationDispatchMode::Sink => {
                let delivery = match kind {
                    ScriptNotificationKind::Success => {
                        self.notification.send_to(message, now_ms, sink)?
                    }
                    ScriptNotificationKind::Error => {
                        self.notification.error_to(message, now_ms, sink)?
                    }
                };
                let record = self
                    .notification
                    .records()
                    .last()
                    .cloned()
                    .expect("notification delivery records before sink dispatch");
                (record, Some(delivery), true)
            }
        };
        Ok(ScriptHostCallResult::NotificationExecution(
            NotificationExecution {
                mode: self.notification_dispatch_mode,
                record,
                delivery,
                dispatched,
            },
        ))
    }

    fn call_post_message(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "keyDown" | "KeyDown" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_down_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "keyUp" | "KeyUp" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_up_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "keyPress" | "KeyPress" => Ok(post_message_result(
                PostMessageSequence::new()
                    .key_press_background(virtual_key_code_for_script(arg_str(&call, 0)?)?),
            )),
            "click" | "Click" => {
                let sequence = if call.args.len() >= 2 {
                    PostMessageSequence::new()
                        .left_button_click_at(arg_i32(&call, 0)?, arg_i32(&call, 1)?)
                } else {
                    PostMessageSequence::new().left_button_click()
                };
                Ok(post_message_result(sequence))
            }
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_strategy_file(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "isFolder" | "IsFolder" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_folder(arg_str(&call, 0)?)?,
            )),
            "isFile" | "IsFile" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_file(arg_str(&call, 0)?)?,
            )),
            "isExists" | "IsExists" => Ok(ScriptHostCallResult::Bool(
                self.strategy_file.is_exists(arg_str(&call, 0)?)?,
            )),
            "readPathSync" | "ReadPathSync" => Ok(ScriptHostCallResult::StringList(
                self.strategy_file
                    .read_path_sync(optional_str(&call, 0)?.unwrap_or("."))?,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_server_time(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "getServerTimeZoneOffset"
            | "GetServerTimeZoneOffset"
            | "serverTimeZoneOffsetMilliseconds"
            | "ServerTimeZoneOffsetMilliseconds" => Ok(ScriptHostCallResult::Integer(
                self.server_time.server_time_zone_offset_milliseconds() as i64,
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_html_mask(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "show" | "Show" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask
                    .show(arg_str(&call, 0)?, optional_str(&call, 1)?)?,
            )),
            "close" | "Close" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.close(arg_str(&call, 0)?),
            )),
            "closeAll" | "CloseAll" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.close_all(),
            )),
            "getWindowIds" | "GetWindowIds" => Ok(ScriptHostCallResult::StringList(
                self.html_mask.window_ids(),
            )),
            "exists" | "Exists" => Ok(ScriptHostCallResult::Bool(
                self.html_mask.exists(arg_str(&call, 0)?),
            )),
            "setClickThrough" | "SetClickThrough" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.set_click_through(
                    arg_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                )?,
            )),
            "getClickThrough" | "GetClickThrough" => Ok(ScriptHostCallResult::Bool(
                self.html_mask.get_click_through(arg_str(&call, 0)?)?,
            )),
            "toggleClickThrough" | "ToggleClickThrough" => {
                Ok(ScriptHostCallResult::HtmlMaskCommand(
                    self.html_mask.toggle_click_through(arg_str(&call, 0)?)?,
                ))
            }
            "send" | "Send" => Ok(ScriptHostCallResult::HtmlMaskCommand(self.html_mask.send(
                arg_str(&call, 0)?,
                arg_str(&call, 1)?,
                arg_str(&call, 2)?,
            )?)),
            "request" | "Request" => Ok(ScriptHostCallResult::HtmlMaskCommand(
                self.html_mask.request(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    arg_str(&call, 2)?,
                    optional_u64(&call, 3)?.unwrap_or(0),
                )?,
            )),
            "receive" | "Receive" => Ok(
                match self
                    .html_mask
                    .receive(arg_str(&call, 0)?, optional_u64(&call, 1)?.unwrap_or(0))?
                {
                    Some(message) => ScriptHostCallResult::String(message),
                    None => ScriptHostCallResult::None,
                },
            ),
            "poll" | "Poll" => Ok(match self.html_mask.poll(arg_str(&call, 0)?)? {
                Some(message) => ScriptHostCallResult::String(message),
                None => ScriptHostCallResult::None,
            }),
            "pollAll" | "PollAll" => Ok(ScriptHostCallResult::String(
                self.html_mask.poll_all(arg_str(&call, 0)?)?,
            )),
            "flushPendingMessages" | "FlushPendingMessages" => {
                Ok(ScriptHostCallResult::StringList(
                    self.html_mask.flush_pending_messages(arg_str(&call, 0)?)?,
                ))
            }
            "sendFromHtml" | "SendFromHtml" => {
                self.html_mask.send_from_html(
                    arg_str(&call, 0)?,
                    arg_str(&call, 1)?,
                    arg_str(&call, 2)?,
                    optional_str(&call, 3)?,
                )?;
                Ok(ScriptHostCallResult::None)
            }
            "snapshot" | "Snapshot" => Ok(ScriptHostCallResult::HtmlMaskSnapshot(
                self.html_mask.snapshot(),
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_key_mouse_hook(&mut self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        match call.method.as_str() {
            "onKeyDown" | "OnKeyDown" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_key_down(
                    optional_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                ),
            )),
            "onKeyUp" | "OnKeyUp" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_key_up(
                    optional_str(&call, 0)?,
                    optional_bool(&call, 1)?.unwrap_or(true),
                ),
            )),
            "onMouseDown" | "OnMouseDown" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_down(optional_str(&call, 0)?),
            )),
            "onMouseUp" | "OnMouseUp" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_up(optional_str(&call, 0)?),
            )),
            "onMouseMove" | "OnMouseMove" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_move(
                    optional_str(&call, 0)?,
                    optional_u64(&call, 1)?.unwrap_or(200),
                ),
            )),
            "onMouseWheel" | "OnMouseWheel" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.on_mouse_wheel(optional_str(&call, 0)?),
            )),
            "removeAllListeners" | "RemoveAllListeners" => {
                Ok(ScriptHostCallResult::KeyMouseHookCommand(
                    self.key_mouse_hook.remove_all_listeners(),
                ))
            }
            "dispose" | "Dispose" => Ok(ScriptHostCallResult::KeyMouseHookCommand(
                self.key_mouse_hook.dispose(),
            )),
            "dispatchEvent" | "DispatchEvent" => {
                let event = key_mouse_hook_event_from_arg(&call, 0)?;
                Ok(ScriptHostCallResult::KeyMouseHookDispatches(
                    self.key_mouse_hook.dispatch_event(event),
                ))
            }
            "snapshot" | "Snapshot" => Ok(ScriptHostCallResult::KeyMouseHookSnapshot(
                self.key_mouse_hook.snapshot(),
            )),
            _ => Err(unknown_method(&call)),
        }
    }

    fn call_custom_host_functions(&self, call: ScriptHostCall) -> Result<ScriptHostCallResult> {
        let command = match call.method.as_str() {
            "newVarOfArr" | "NewVarOfArr" => {
                let element_type = arg_str(&call, 0)?.to_string();
                let dimensions = arg_u32_like(&call, 1)?;
                if dimensions == 0 {
                    return Err(invalid_arg(&call, 1, "array dimensions greater than zero"));
                }
                CustomHostFunctionCommand::NewArrayVariable {
                    legacy_jagged_type: legacy_jagged_array_type(&element_type, dimensions),
                    element_type,
                    dimensions,
                }
            }
            "newObj" | "NewObj" => CustomHostFunctionCommand::NewObject {
                type_name: arg_str(&call, 0)?.to_string(),
                args: call.args.iter().skip(1).cloned().collect(),
            },
            "delObj" | "DelObj" => CustomHostFunctionCommand::DeleteObject {
                target: call.args.first().cloned(),
            },
            "type" | "Type" => CustomHostFunctionCommand::TypeLookup {
                type_name: arg_str(&call, 0)?.to_string(),
            },
            "toIterator" | "ToIterator" => CustomHostFunctionCommand::ToIterator {
                source: arg_owned_value(&call, 0)?,
            },
            _ => return Err(unknown_method(&call)),
        };
        Ok(ScriptHostCallResult::CustomHostFunctionCommand(command))
    }
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

#[derive(Debug, Clone)]
pub struct LimitedFileHost {
    policy: ScriptFilePolicy,
}

impl LimitedFileHost {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            policy: ScriptFilePolicy::new(root),
        }
    }

    pub fn with_policy(policy: ScriptFilePolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &ScriptFilePolicy {
        &self.policy
    }

    pub fn normalize_path(&self, path: &str) -> Result<PathBuf> {
        Ok(self.policy.normalize_path(path)?)
    }

    pub fn read_path_sync(&self, folder_path: &str) -> Result<Vec<String>> {
        let normalized = self.policy.normalize_path(folder_path)?;
        if !normalized.is_dir() {
            return Ok(Vec::new());
        }
        let root = self.policy.normalize_path(".")?;

        let mut entries = Vec::new();
        for entry in fs::read_dir(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| ScriptHostRuntimeError::Io {
                path: normalized.clone(),
                source,
            })?;
            entries.push(relative_to_root(&root, &entry.path()));
        }
        entries.sort();
        Ok(entries)
    }

    pub fn create_directory(&self, folder_path: &str) -> Result<bool> {
        let normalized = self.policy.normalize_path(folder_path)?;
        fs::create_dir_all(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized,
            source,
        })?;
        Ok(true)
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        Ok(self.policy.normalize_path(path)?.is_dir())
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        Ok(self.policy.normalize_path(path)?.is_file())
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        let normalized = self.policy.normalize_path(path)?;
        Ok(normalized.exists())
    }

    pub fn read_text_sync(&self, path: &str) -> Result<String> {
        let normalized = self.policy.normalize_path(path)?;
        fs::read_to_string(&normalized).map_err(|source| ScriptHostRuntimeError::Io {
            path: normalized,
            source,
        })
    }

    pub fn write_text_sync(&self, path: &str, content: &str, append: bool) -> Result<bool> {
        let normalized = self.policy.validate_text_write(path, content)?;
        if let Some(parent) = normalized.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        if append && normalized.exists() {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&normalized)
                .map_err(|source| ScriptHostRuntimeError::Io {
                    path: normalized.clone(),
                    source,
                })?;
            file.write_all(content.as_bytes())
                .map_err(|source| ScriptHostRuntimeError::Io {
                    path: normalized,
                    source,
                })?;
        } else {
            fs::write(&normalized, content).map_err(|source| ScriptHostRuntimeError::Io {
                path: normalized,
                source,
            })?;
        }
        Ok(true)
    }

    pub fn read_image_mat_plan_sync(&self, path: &str) -> Result<ImageMatReadPlan> {
        let normalized = self.normalize_image_read_path(path)?;
        Ok(ImageMatReadPlan {
            normalized_path: normalized,
            color_mode: "color",
            resize: None,
        })
    }

    pub fn read_image_mat_sync(&self, path: &str) -> Result<ImageMatReadExecution> {
        let normalized = self.normalize_image_read_path(path)?;
        let image = BgrImage::read(&normalized)?;
        Ok(ImageMatReadExecution::from_image(normalized, image, None))
    }

    pub fn read_image_mat_with_resize_plan_sync(
        &self,
        path: &str,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatReadPlan> {
        let resize = self.validate_image_resize_args(width, height, interpolation)?;
        let normalized = self.normalize_image_read_path(path)?;
        Ok(ImageMatReadPlan {
            normalized_path: normalized,
            color_mode: "color",
            resize: Some(resize),
        })
    }

    pub fn read_image_mat_with_resize_sync(
        &self,
        path: &str,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatReadExecution> {
        let resize = self.validate_image_resize_args(width, height, interpolation)?;
        let normalized = self.normalize_image_read_path(path)?;
        let image = resize_bgr_nearest(
            &BgrImage::read(&normalized)?,
            VisionSize::new(resize.width.round() as u32, resize.height.round() as u32),
        )?;
        Ok(ImageMatReadExecution::from_image(
            normalized,
            image,
            Some(resize),
        ))
    }

    fn validate_image_resize_args(
        &self,
        width: f64,
        height: f64,
        interpolation: i32,
    ) -> Result<ImageMatResizePlan> {
        if width <= 0.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                1,
                "positive width",
            ));
        }
        if height <= 0.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                2,
                "positive height",
            ));
        }
        if width.round() < 1.0 || height.round() < 1.0 {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                1,
                "positive rounded image size",
            ));
        }
        if !(0..=5).contains(&interpolation) {
            return Err(invalid_arg_for_method(
                "file.ReadImageMatWithResizeSync",
                3,
                "OpenCV interpolation value 0..=5",
            ));
        }
        Ok(ImageMatResizePlan {
            width,
            height,
            interpolation,
        })
    }

    fn normalize_image_read_path(&self, path: &str) -> Result<PathBuf> {
        let normalized = self.policy.normalize_path(path)?;
        self.policy.validate_image_extension(&normalized)?;
        Ok(normalized)
    }

    pub fn write_image_plan_sync(&self, path: &str, source: Value) -> Result<ImageMatWritePlan> {
        let normalized = self.policy.normalize_image_write_target(path)?;
        Ok(ImageMatWritePlan {
            normalized_path: normalized,
            source,
        })
    }

    pub fn write_image_sync(&self, path: &str, source: Value) -> Result<ImageMatWriteExecution> {
        let plan = self.write_image_plan_sync(path, source)?;
        let image = image_from_mat_value(&plan.source)?;
        if let Some(parent) = plan.normalized_path.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        image.write_png(&plan.normalized_path)?;
        let bytes_written = fs::metadata(&plan.normalized_path)
            .map_err(|source| ScriptHostRuntimeError::Io {
                path: plan.normalized_path.clone(),
                source,
            })?
            .len();
        Ok(ImageMatWriteExecution {
            normalized_path: plan.normalized_path,
            width: image.size.width,
            height: image.size.height,
            pixel_format: "BGR24",
            bytes_written,
        })
    }

    pub fn rename_path_sync(&self, old_path: &str, new_path: &str) -> Result<bool> {
        let old_normalized = self.policy.normalize_path(old_path)?;
        let new_normalized = self.policy.normalize_path(new_path)?;
        if !old_normalized.exists() {
            return Err(ScriptHostRuntimeError::RenameSourceMissing(old_normalized));
        }
        if old_normalized.is_file() {
            self.policy.validate_write_extension(&new_normalized)?;
        }
        if new_normalized.exists() && old_normalized.is_file() != new_normalized.is_file() {
            return Err(ScriptHostRuntimeError::RenameKindMismatch {
                from: old_normalized,
                to: new_normalized,
            });
        }
        if let Some(parent) = new_normalized.parent() {
            fs::create_dir_all(parent).map_err(|source| ScriptHostRuntimeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::rename(&old_normalized, &new_normalized).map_err(|source| {
            ScriptHostRuntimeError::Io {
                path: old_normalized,
                source,
            }
        })?;
        Ok(true)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VisionHost;

impl VisionHost {
    const INLINE_TEMPLATE_ASSET: &'static str = "__script_inline_template__";

    pub fn crop(&self, image: Value, rect: Value) -> Result<VisionImageMatExecution> {
        const METHOD: &str = "vision.crop";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let rect = rect_from_value(&rect, METHOD, 1)?;
        let region = ImageRegion::from_mat_handle("script-mat", image, 0, 0).derive_crop(rect)?;
        Ok(VisionImageMatExecution::from_image_region(region))
    }

    pub fn to_1080p(&self, image: Value) -> Result<VisionImageMatExecution> {
        const METHOD: &str = "vision.to1080p";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let region = ImageRegion::from_mat_handle("script-mat", image, 0, 0).derive_to_1080p()?;
        Ok(VisionImageMatExecution::from_image_region(region))
    }

    pub fn find_template(
        &self,
        image: Value,
        template: Value,
        options: Option<Value>,
    ) -> Result<VisionRecognitionExecution> {
        const METHOD: &str = "vision.findTemplate";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let template = image_from_mat_value_for(&template, METHOD, 1)?;
        let options = object_options(options.as_ref(), METHOD, 2)?;

        let mut object = RecognitionObject::template_match(Self::INLINE_TEMPLATE_ASSET);
        object.name = optional_string_field(options, &["name", "Name"]);
        object.region_of_interest = optional_rect_field(
            options,
            &[
                "roi",
                "ROI",
                "regionOfInterest",
                "region_of_interest",
                "RegionOfInterest",
            ],
            METHOD,
            2,
        )?;
        object.template.template_size = Some(template.size);
        if let Some(threshold) =
            optional_f64_field(options, &["threshold", "Threshold"], METHOD, 2)?
        {
            object.template.threshold = threshold;
        }
        if let Some(use_3_channels) = optional_bool_field(
            options,
            &["use3Channels", "use_3_channels", "Use3Channels"],
            METHOD,
            2,
        )? {
            object.template.use_3_channels = use_3_channels;
        }
        if let Some(mode) =
            optional_template_match_mode_field(options, &["mode", "Mode"], METHOD, 2)?
        {
            object.template.mode = mode;
        }
        if let Some(max_match_count) = optional_i32_field(
            options,
            &["maxMatchCount", "max_match_count", "MaxMatchCount"],
            METHOD,
            2,
        )? {
            object.template.max_match_count = max_match_count;
        }
        object.validate()?;

        let mut backend = PureRustVisionBackend::new();
        backend.register_template(Self::INLINE_TEMPLATE_ASSET, template);
        self.execute(image, object, &backend)
    }

    pub fn find_color(
        &self,
        image: Value,
        options: Option<Value>,
    ) -> Result<VisionRecognitionExecution> {
        const METHOD: &str = "vision.findColor";

        let image = image_from_mat_value_for(&image, METHOD, 0)?;
        let options = object_options(options.as_ref(), METHOD, 1)?
            .ok_or_else(|| invalid_arg_for_method(METHOD, 1, "color match options object"))?;

        let mut object = RecognitionObject {
            recognition_type: RecognitionType::ColorMatch,
            ..RecognitionObject::default()
        };
        object.name = optional_string_field(Some(options), &["name", "Name"]);
        object.region_of_interest = optional_rect_field(
            Some(options),
            &[
                "roi",
                "ROI",
                "regionOfInterest",
                "region_of_interest",
                "RegionOfInterest",
            ],
            METHOD,
            1,
        )?;
        object.color = ColorMatchConfig {
            conversion: optional_color_conversion_field(
                Some(options),
                &[
                    "conversion",
                    "Conversion",
                    "colorConversion",
                    "ColorConversion",
                ],
                METHOD,
                1,
            )?
            .unwrap_or_default(),
            lower_color: required_scalar_field(
                options,
                &["lowerColor", "lower_color", "LowerColor", "lower"],
                METHOD,
                1,
            )?,
            upper_color: required_scalar_field(
                options,
                &["upperColor", "upper_color", "UpperColor", "upper"],
                METHOD,
                1,
            )?,
            match_count: optional_u32_field(
                Some(options),
                &["matchCount", "match_count", "MatchCount"],
                METHOD,
                1,
            )?
            .unwrap_or(1),
        };
        object.validate()?;

        let backend = PureRustVisionBackend::new();
        self.execute(image, object, &backend)
    }

    fn execute(
        &self,
        image: BgrImage,
        object: RecognitionObject,
        backend: &PureRustVisionBackend,
    ) -> Result<VisionRecognitionExecution> {
        let recognition_type = object.recognition_type;
        let image_region = ImageRegion::from_mat_handle("script-mat", image, 0, 0);
        let matches = image_region.find_multi(backend, &object)?;
        let first = matches.first().cloned().unwrap_or_else(Region::empty);
        Ok(VisionRecognitionExecution {
            recognition_type,
            image_region: image_region.model,
            matched_count: matches.len(),
            first,
            matches,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptLogHost {
    records: Vec<ScriptLogRecord>,
}

impl ScriptLogHost {
    pub fn debug(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Debug, message);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Info, message);
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Warn, message);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Error, message);
    }

    pub fn records(&self) -> &[ScriptLogRecord] {
        &self.records
    }

    fn push(&mut self, level: ScriptLogLevel, message: impl Into<String>) {
        self.records.push(ScriptLogRecord {
            level,
            message: message.into(),
        });
    }
}

#[derive(Debug, Clone)]
pub struct ScriptNotificationHost {
    limiter: NotificationRateLimiter,
    records: Vec<ScriptNotificationRecord>,
}

impl ScriptNotificationHost {
    pub fn new(policy: ScriptNotificationPolicy) -> Self {
        Self {
            limiter: NotificationRateLimiter::new(policy),
            records: Vec::new(),
        }
    }

    pub fn send_at(&mut self, message: &str, now_ms: u64) -> Result<ScriptNotificationRecord> {
        self.push(ScriptNotificationKind::Success, message, now_ms)
    }

    pub fn send_to<S: ScriptNotificationSink>(
        &mut self,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        self.push_to(ScriptNotificationKind::Success, message, now_ms, sink)
    }

    pub fn error_at(&mut self, message: &str, now_ms: u64) -> Result<ScriptNotificationRecord> {
        self.push(ScriptNotificationKind::Error, message, now_ms)
    }

    pub fn error_to<S: ScriptNotificationSink>(
        &mut self,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        self.push_to(ScriptNotificationKind::Error, message, now_ms, sink)
    }

    pub fn records(&self) -> &[ScriptNotificationRecord] {
        &self.records
    }

    fn push(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
    ) -> Result<ScriptNotificationRecord> {
        self.validated_record(kind, message, now_ms)
    }

    fn push_to<S: ScriptNotificationSink>(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        let record = self.validated_record(kind, message, now_ms)?;
        let delivery = ScriptNotificationDelivery::from_record(&record);
        sink.deliver(delivery.clone())?;
        Ok(delivery)
    }

    fn validated_record(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
    ) -> Result<ScriptNotificationRecord> {
        self.limiter.check_send_at(message, now_ms)?;
        let record = ScriptNotificationRecord {
            kind,
            message: message.to_string(),
            timestamp_ms: now_ms,
        };
        self.records.push(record.clone());
        Ok(record)
    }
}

#[derive(Debug, Clone)]
pub struct StrategyFileHost {
    file_host: LimitedFileHost,
}

impl StrategyFileHost {
    pub fn new(auto_fight_root: impl Into<PathBuf>) -> Self {
        Self {
            file_host: LimitedFileHost::new(auto_fight_root),
        }
    }

    pub fn is_folder(&self, path: &str) -> Result<bool> {
        self.file_host.is_folder(path)
    }

    pub fn is_file(&self, path: &str) -> Result<bool> {
        self.file_host.is_file(path)
    }

    pub fn is_exists(&self, path: &str) -> Result<bool> {
        self.file_host.is_exists(path)
    }

    pub fn read_path_sync(&self, path: &str) -> Result<Vec<String>> {
        self.file_host.read_path_sync(path)
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
        let key = parse_virtual_key(key)?;
        Ok(key_input_sequence(key, KeyInputAction::Down))
    }

    pub fn key_up(&self, key: &str) -> Result<InputSequence> {
        let key = parse_virtual_key(key)?;
        Ok(key_input_sequence(key, KeyInputAction::Up))
    }

    pub fn key_press(&self, key: &str) -> Result<InputSequence> {
        let key = parse_virtual_key(key)?;
        Ok(key_input_sequence(key, KeyInputAction::Press))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyInputAction {
    Down,
    Up,
    Press,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedVirtualKey {
    Keyboard(u16),
    Mouse(MouseButton),
}

fn key_input_sequence(key: ParsedVirtualKey, action: KeyInputAction) -> InputSequence {
    match (key, action) {
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Down) => InputSequence::new().key_down(vk),
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Up) => InputSequence::new().key_up(vk),
        (ParsedVirtualKey::Keyboard(vk), KeyInputAction::Press) => {
            InputSequence::new().key_press(vk)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Down) => {
            InputSequence::new().mouse_down(button)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Up) => {
            InputSequence::new().mouse_up(button)
        }
        (ParsedVirtualKey::Mouse(button), KeyInputAction::Press) => {
            InputSequence::new().mouse_click(button)
        }
    }
}

fn parse_virtual_key(key: &str) -> Result<ParsedVirtualKey> {
    match key {
        "VK_LBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Left)),
        "VK_RBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Right)),
        "VK_MBUTTON" => Ok(ParsedVirtualKey::Mouse(MouseButton::Middle)),
        "VK_XBUTTON1" => Ok(ParsedVirtualKey::Mouse(MouseButton::X(1))),
        "VK_XBUTTON2" => Ok(ParsedVirtualKey::Mouse(MouseButton::X(2))),
        _ => virtual_key_code(key)
            .map(ParsedVirtualKey::Keyboard)
            .ok_or_else(|| ScriptHostRuntimeError::UnsupportedVirtualKey(key.to_string())),
    }
}

pub fn virtual_key_code_for_script(key: &str) -> Result<u16> {
    match parse_virtual_key(key)? {
        ParsedVirtualKey::Keyboard(vk) => Ok(vk),
        ParsedVirtualKey::Mouse(_) => Err(ScriptHostRuntimeError::UnsupportedVirtualKey(
            key.to_string(),
        )),
    }
}

fn virtual_key_code(key: &str) -> Option<u16> {
    let key = key.strip_prefix("VK_").unwrap_or(key);
    match key {
        "BACK" | "BACKSPACE" => Some(0x08),
        "TAB" => Some(0x09),
        "RETURN" | "ENTER" => Some(0x0D),
        "SHIFT" => Some(0x10),
        "CONTROL" | "CTRL" => Some(0x11),
        "MENU" | "ALT" => Some(0x12),
        "ESCAPE" | "ESC" => Some(0x1B),
        "SPACE" => Some(0x20),
        "PRIOR" | "PAGE_UP" => Some(0x21),
        "NEXT" | "PAGE_DOWN" => Some(0x22),
        "END" => Some(0x23),
        "HOME" => Some(0x24),
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),
        "INSERT" => Some(0x2D),
        "DELETE" => Some(0x2E),
        "LWIN" => Some(0x5B),
        "RWIN" => Some(0x5C),
        "NUMPAD0" => Some(0x60),
        "NUMPAD1" => Some(0x61),
        "NUMPAD2" => Some(0x62),
        "NUMPAD3" => Some(0x63),
        "NUMPAD4" => Some(0x64),
        "NUMPAD5" => Some(0x65),
        "NUMPAD6" => Some(0x66),
        "NUMPAD7" => Some(0x67),
        "NUMPAD8" => Some(0x68),
        "NUMPAD9" => Some(0x69),
        "MULTIPLY" => Some(0x6A),
        "ADD" => Some(0x6B),
        "SUBTRACT" => Some(0x6D),
        "DECIMAL" => Some(0x6E),
        "DIVIDE" => Some(0x6F),
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        "LSHIFT" => Some(0xA0),
        "RSHIFT" => Some(0xA1),
        "LCONTROL" | "LCTRL" => Some(0xA2),
        "RCONTROL" | "RCTRL" => Some(0xA3),
        "LMENU" | "LALT" => Some(0xA4),
        "RMENU" | "RALT" => Some(0xA5),
        _ if key.len() == 1 => {
            let ch = key.as_bytes()[0];
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_uppercase() as u16)
            } else {
                None
            }
        }
        _ => key.parse::<u16>().ok(),
    }
}

fn post_message_result(sequence: PostMessageSequence) -> ScriptHostCallResult {
    ScriptHostCallResult::PostMessageEvents(sequence.events().to_vec())
}

fn image_from_mat_value(source: &Value) -> Result<BgrImage> {
    image_from_mat_value_for(source, "file.WriteImageSync", 1)
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

fn image_from_mat_value_for(source: &Value, method: &str, index: usize) -> Result<BgrImage> {
    let width = value_u32_field(source, &["width", "Width"], method, index)?;
    let height = value_u32_field(source, &["height", "Height"], method, index)?;
    let pixel_format =
        value_str_field(source, &["pixel_format", "pixelFormat", "PixelFormat"]).unwrap_or("BGR24");
    if !pixel_format.eq_ignore_ascii_case("BGR24") {
        return Err(invalid_arg_for_method(method, index, "BGR24 image payload"));
    }
    let pixels = value_u8_vec_field(source, &["pixels", "Pixels"], method, index)?;
    BgrImage::new(VisionSize::new(width, height), pixels).map_err(Into::into)
}

fn object_options<'a>(
    options: Option<&'a Value>,
    method: &str,
    index: usize,
) -> Result<Option<&'a serde_json::Map<String, Value>>> {
    match options {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map)),
        Some(_) => Err(invalid_arg_for_method(method, index, "options object")),
    }
}

fn value_field<'a>(map: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| map.get(*key))
}

fn optional_string_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
) -> Option<String> {
    value_field(options?, keys)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn optional_bool_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<bool>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    bool_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "bool field"))
}

fn optional_u32_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<u32>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    u32_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "u32 field"))
}

fn optional_i32_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<i32>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    i32_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "i32 field"))
}

fn optional_f64_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<f64>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    f64_like(value)
        .map(Some)
        .ok_or_else(|| invalid_arg_for_method(method, index, "number field"))
}

fn optional_rect_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<Rect>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    rect_from_value(value, method, index).map(Some)
}

fn rect_from_value(value: &Value, method: &str, index: usize) -> Result<Rect> {
    match value {
        Value::Array(values) if values.len() == 4 => Rect::new(
            value_i32_component(&values[0], method, index)?,
            value_i32_component(&values[1], method, index)?,
            value_i32_component(&values[2], method, index)?,
            value_i32_component(&values[3], method, index)?,
        )
        .map_err(Into::into),
        Value::Object(map) => Rect::new(
            required_i32_component(map, &["x", "X"], method, index)?,
            required_i32_component(map, &["y", "Y"], method, index)?,
            required_i32_component(map, &["width", "Width"], method, index)?,
            required_i32_component(map, &["height", "Height"], method, index)?,
        )
        .map_err(Into::into),
        _ => Err(invalid_arg_for_method(
            method,
            index,
            "rect object or [x,y,width,height]",
        )),
    }
}

fn required_scalar_field(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Scalar4> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "scalar color field"))?;
    scalar_from_value(value, method, index)
}

fn scalar_from_value(value: &Value, method: &str, index: usize) -> Result<Scalar4> {
    match value {
        Value::Array(values) if values.len() == 3 || values.len() == 4 => Ok(Scalar4 {
            v0: value_f64_component(&values[0], method, index)?,
            v1: value_f64_component(&values[1], method, index)?,
            v2: value_f64_component(&values[2], method, index)?,
            v3: values
                .get(3)
                .map(|value| value_f64_component(value, method, index))
                .transpose()?
                .unwrap_or(0.0),
        }),
        Value::Object(map) => Ok(Scalar4 {
            v0: required_f64_component(map, &["v0", "V0"], method, index)?,
            v1: required_f64_component(map, &["v1", "V1"], method, index)?,
            v2: required_f64_component(map, &["v2", "V2"], method, index)?,
            v3: optional_f64_component(map, &["v3", "V3"], method, index)?.unwrap_or(0.0),
        }),
        _ => Err(invalid_arg_for_method(
            method,
            index,
            "scalar color array or object",
        )),
    }
}

fn optional_template_match_mode_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<TemplateMatchMode>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    template_match_mode_from_value(value, method, index).map(Some)
}

fn template_match_mode_from_value(
    value: &Value,
    method: &str,
    index: usize,
) -> Result<TemplateMatchMode> {
    if let Some(number) = i32_like(value) {
        return match number {
            0 => Ok(TemplateMatchMode::SqDiff),
            1 => Ok(TemplateMatchMode::SqDiffNormed),
            2 => Ok(TemplateMatchMode::CCorr),
            3 => Ok(TemplateMatchMode::CCorrNormed),
            4 => Ok(TemplateMatchMode::CCoeff),
            5 => Ok(TemplateMatchMode::CCoeffNormed),
            _ => Err(invalid_arg_for_method(
                method,
                index,
                "template match mode 0..=5",
            )),
        };
    }
    let Some(value) = value.as_str() else {
        return Err(invalid_arg_for_method(method, index, "template match mode"));
    };
    match normalize_enum_token(value).as_str() {
        "sqdiff" | "templatemodessqdiff" => Ok(TemplateMatchMode::SqDiff),
        "sqdiffnormed" | "templatemodessqdiffnormed" => Ok(TemplateMatchMode::SqDiffNormed),
        "ccorr" | "templatemodesccorr" => Ok(TemplateMatchMode::CCorr),
        "ccorrnormed" | "templatemodesccorrnormed" => Ok(TemplateMatchMode::CCorrNormed),
        "ccoeff" | "templatemodesccoeff" => Ok(TemplateMatchMode::CCoeff),
        "ccoeffnormed" | "templatemodesccoeffnormed" => Ok(TemplateMatchMode::CCoeffNormed),
        _ => Err(invalid_arg_for_method(method, index, "template match mode")),
    }
}

fn optional_color_conversion_field(
    options: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<ColorConversion>> {
    let Some(value) = options.and_then(|map| value_field(map, keys)) else {
        return Ok(None);
    };
    color_conversion_from_value(value, method, index).map(Some)
}

fn color_conversion_from_value(
    value: &Value,
    method: &str,
    index: usize,
) -> Result<ColorConversion> {
    if let Some(number) = i32_like(value) {
        return match number {
            1 => Ok(ColorConversion::BgraToBgr),
            4 => Ok(ColorConversion::BgrToRgb),
            6 => Ok(ColorConversion::BgrToGray),
            40 => Ok(ColorConversion::BgrToHsv),
            _ => Err(invalid_arg_for_method(
                method,
                index,
                "OpenCV color conversion code 1, 4, 6, or 40",
            )),
        };
    }
    let Some(value) = value.as_str() else {
        return Err(invalid_arg_for_method(method, index, "color conversion"));
    };
    match normalize_enum_token(value).as_str() {
        "bgrtorgb" | "bgr2rgb" | "rgb" => Ok(ColorConversion::BgrToRgb),
        "bgrtohsv" | "bgr2hsv" | "hsv" => Ok(ColorConversion::BgrToHsv),
        "bgrtogray" | "bgr2gray" | "gray" | "grey" => Ok(ColorConversion::BgrToGray),
        "bgratobgr" | "bgra2bgr" => Ok(ColorConversion::BgraToBgr),
        _ => Err(invalid_arg_for_method(method, index, "color conversion")),
    }
}

fn normalize_enum_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn required_i32_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<i32> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "i32 component"))?;
    value_i32_component(value, method, index)
}

fn required_f64_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<f64> {
    let value = value_field(map, keys)
        .ok_or_else(|| invalid_arg_for_method(method, index, "number component"))?;
    value_f64_component(value, method, index)
}

fn optional_f64_component(
    map: &serde_json::Map<String, Value>,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Option<f64>> {
    value_field(map, keys)
        .map(|value| value_f64_component(value, method, index))
        .transpose()
}

fn value_i32_component(value: &Value, method: &str, index: usize) -> Result<i32> {
    i32_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "i32 component"))
}

fn value_f64_component(value: &Value, method: &str, index: usize) -> Result<f64> {
    f64_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "number component"))
}

fn value_u32_field(source: &Value, keys: &[&str], method: &str, index: usize) -> Result<u32> {
    let value = keys
        .iter()
        .find_map(|key| source.get(*key))
        .ok_or_else(|| invalid_arg_for_method(method, index, "image width/height fields"))?;
    u32_like(value).ok_or_else(|| invalid_arg_for_method(method, index, "u32 field"))
}

fn value_str_field<'a>(source: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| source.get(*key))
        .and_then(Value::as_str)
}

fn value_u8_vec_field(
    source: &Value,
    keys: &[&str],
    method: &str,
    index: usize,
) -> Result<Vec<u8>> {
    let value = keys
        .iter()
        .find_map(|key| source.get(*key))
        .ok_or_else(|| invalid_arg_for_method(method, index, "image pixels field"))?;
    let pixels = value
        .as_array()
        .ok_or_else(|| invalid_arg_for_method(method, index, "u8 pixel array"))?;
    pixels
        .iter()
        .map(|value| {
            let byte = value
                .as_u64()
                .ok_or_else(|| invalid_arg_for_method(method, index, "u8 pixel array"))?;
            u8::try_from(byte).map_err(|_| invalid_arg_for_method(method, index, "u8 pixel array"))
        })
        .collect()
}

fn unknown_method(call: &ScriptHostCall) -> ScriptHostRuntimeError {
    ScriptHostRuntimeError::UnknownHostMethod {
        target: call.target.as_str(),
        method: call.method.clone(),
    }
}

fn arg_value<'a>(
    call: &'a ScriptHostCall,
    index: usize,
    expected: &'static str,
) -> Result<&'a Value> {
    call.args
        .get(index)
        .filter(|value| !value.is_null())
        .ok_or_else(|| invalid_arg(call, index, expected))
}

fn arg_str<'a>(call: &'a ScriptHostCall, index: usize) -> Result<&'a str> {
    arg_value(call, index, "string")?
        .as_str()
        .ok_or_else(|| invalid_arg(call, index, "string"))
}

fn optional_str<'a>(call: &'a ScriptHostCall, index: usize) -> Result<Option<&'a str>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "string")),
    }
}

fn arg_i32(call: &ScriptHostCall, index: usize) -> Result<i32> {
    let value = arg_value(call, index, "i32")?;
    let number = value
        .as_i64()
        .ok_or_else(|| invalid_arg(call, index, "i32"))?;
    i32::try_from(number).map_err(|_| invalid_arg(call, index, "i32"))
}

fn arg_u32(call: &ScriptHostCall, index: usize) -> Result<u32> {
    let value = arg_value(call, index, "u32")?;
    let number = value
        .as_u64()
        .ok_or_else(|| invalid_arg(call, index, "u32"))?;
    u32::try_from(number).map_err(|_| invalid_arg(call, index, "u32"))
}

fn arg_u64(call: &ScriptHostCall, index: usize) -> Result<u64> {
    arg_value(call, index, "u64")?
        .as_u64()
        .ok_or_else(|| invalid_arg(call, index, "u64"))
}

fn arg_f64_like(call: &ScriptHostCall, index: usize) -> Result<f64> {
    f64_like(arg_value(call, index, "number or numeric string")?)
        .ok_or_else(|| invalid_arg(call, index, "number or numeric string"))
}

fn arg_u32_like(call: &ScriptHostCall, index: usize) -> Result<u32> {
    u32_like(arg_value(call, index, "u32 or numeric string")?)
        .ok_or_else(|| invalid_arg(call, index, "u32 or numeric string"))
}

fn optional_u32_like(call: &ScriptHostCall, index: usize) -> Result<Option<u32>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => u32_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "u32 or numeric string")),
    }
}

fn optional_u64(call: &ScriptHostCall, index: usize) -> Result<Option<u64>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => u64_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "u64 or numeric string")),
    }
}

fn optional_i32(call: &ScriptHostCall, index: usize) -> Result<Option<i32>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => i32_like(value)
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "i32 or numeric string")),
    }
}

fn optional_f64(call: &ScriptHostCall, index: usize) -> Result<Option<f64>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "f64")),
    }
}

fn optional_bool(call: &ScriptHostCall, index: usize) -> Result<Option<bool>> {
    match call.args.get(index) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| invalid_arg(call, index, "bool")),
    }
}

fn arg_owned_value(call: &ScriptHostCall, index: usize) -> Result<Value> {
    Ok(arg_value(call, index, "object")?.clone())
}

fn optional_owned_value(call: &ScriptHostCall, index: usize) -> Option<Value> {
    match call.args.get(index) {
        None | Some(Value::Null) => None,
        Some(value) => Some(value.clone()),
    }
}

fn f64_like(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse::<f64>().ok())
}

fn bool_like(value: &Value) -> Option<bool> {
    if let Some(value) = value.as_bool() {
        return Some(value);
    }
    match value.as_str()?.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

fn u64_like(value: &Value) -> Option<u64> {
    if let Some(value) = value.as_u64() {
        return Some(value);
    }
    let value = value.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    value.parse::<u64>().ok()
}

fn u32_like(value: &Value) -> Option<u32> {
    u64_like(value).and_then(|value| u32::try_from(value).ok())
}

fn i32_like(value: &Value) -> Option<i32> {
    if let Some(value) = value.as_i64() {
        return i32::try_from(value).ok();
    }
    let value = value.as_str()?.trim();
    if value.is_empty() {
        return None;
    }
    value
        .parse::<i64>()
        .ok()
        .and_then(|value| i32::try_from(value).ok())
}

fn legacy_jagged_array_type(element_type: &str, dimensions: u32) -> String {
    let mut type_name = element_type.trim().to_string();
    for _ in 0..dimensions {
        type_name.push_str("[]");
    }
    type_name
}

fn timer_plan_from_arg(
    call: &ScriptHostCall,
    index: usize,
    clears_existing_triggers: bool,
) -> Result<RealtimeTimerHostPlan> {
    let value = arg_value(call, index, "timer object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "timer object"));
    };
    let name = map
        .get("Name")
        .or_else(|| map.get("name"))
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| invalid_arg(call, index, "timer.name"))?;
    let interval_ms = map
        .get("Interval")
        .or_else(|| map.get("interval"))
        .and_then(Value::as_u64)
        .unwrap_or(50);
    let config_value = map.get("Config").or_else(|| map.get("config"));
    let config = if name.eq_ignore_ascii_case("AutoPick") {
        Some(AutoPickExternalConfig::from_value(config_value)?.to_legacy_config_value())
    } else {
        config_value.cloned()
    };
    Ok(RealtimeTimerHostPlan {
        name: name.to_string(),
        interval_ms,
        config,
        clears_existing_triggers,
    })
}

fn solo_task_plan_from_arg(call: &ScriptHostCall, index: usize) -> Result<SoloTaskHostPlan> {
    let value = arg_value(call, index, "solo task object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "solo task object"));
    };
    let name = map
        .get("Name")
        .or_else(|| map.get("name"))
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| invalid_arg(call, index, "soloTask.name"))?;
    let config = map.get("Config").or_else(|| map.get("config")).cloned();
    Ok(SoloTaskHostPlan::new(name, config))
}

fn key_mouse_hook_event_from_arg(call: &ScriptHostCall, index: usize) -> Result<KeyMouseHookEvent> {
    let value = arg_value(call, index, "key/mouse hook event object")?;
    let Value::Object(map) = value else {
        return Err(invalid_arg(call, index, "key/mouse hook event object"));
    };
    let event_type = map
        .get("type")
        .or_else(|| map.get("Type"))
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_arg(call, index, "event.type"))?;

    match event_type {
        "keyDown" | "KeyDown" => Ok(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyDown,
            key_data: hook_event_string(map, "keyData", "KeyData", "key_data", "Unknown"),
            key_code: hook_event_string(map, "keyCode", "KeyCode", "key_code", "Unknown"),
        }),
        "keyUp" | "KeyUp" => Ok(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyUp,
            key_data: hook_event_string(map, "keyData", "KeyData", "key_data", "Unknown"),
            key_code: hook_event_string(map, "keyCode", "KeyCode", "key_code", "Unknown"),
        }),
        "mouseDown" | "MouseDown" => Ok(KeyMouseHookEvent::MouseButton {
            event: KeyMouseHookEventKind::MouseDown,
            button: hook_event_mouse_button(map),
            x: hook_event_i32(map, "x", "X", "localX", 0),
            y: hook_event_i32(map, "y", "Y", "localY", 0),
        }),
        "mouseUp" | "MouseUp" => Ok(KeyMouseHookEvent::MouseButton {
            event: KeyMouseHookEventKind::MouseUp,
            button: hook_event_mouse_button(map),
            x: hook_event_i32(map, "x", "X", "localX", 0),
            y: hook_event_i32(map, "y", "Y", "localY", 0),
        }),
        "mouseMove" | "MouseMove" => Ok(KeyMouseHookEvent::MouseMove {
            x: hook_event_i32(map, "x", "X", "localX", 0),
            y: hook_event_i32(map, "y", "Y", "localY", 0),
            timestamp_ms: hook_event_u64(map, "timestampMs", "TimestampMs", "timestamp_ms", 0),
        }),
        "mouseWheel" | "MouseWheel" => Ok(KeyMouseHookEvent::MouseWheel {
            delta: hook_event_i32(map, "delta", "Delta", "wheelDelta", 0),
            x: hook_event_i32(map, "x", "X", "localX", 0),
            y: hook_event_i32(map, "y", "Y", "localY", 0),
        }),
        _ => Err(invalid_arg(call, index, "known key/mouse hook event type")),
    }
}

fn hook_event_string(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: &str,
) -> String {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn hook_event_i32(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: i32,
) -> i32 {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(i32_like)
        .unwrap_or(default)
}

fn hook_event_u64(
    map: &serde_json::Map<String, Value>,
    primary: &str,
    secondary: &str,
    tertiary: &str,
    default: u64,
) -> u64 {
    map.get(primary)
        .or_else(|| map.get(secondary))
        .or_else(|| map.get(tertiary))
        .and_then(u64_like)
        .unwrap_or(default)
}

fn hook_event_mouse_button(map: &serde_json::Map<String, Value>) -> MouseButton {
    let button = map
        .get("button")
        .or_else(|| map.get("Button"))
        .and_then(Value::as_str)
        .unwrap_or("Left");
    parse_mouse_button_name(button)
}

fn invalid_arg(
    call: &ScriptHostCall,
    index: usize,
    expected: &'static str,
) -> ScriptHostRuntimeError {
    invalid_arg_for_method(
        &format!("{}.{}", call.target.as_str(), call.method),
        index,
        expected,
    )
}

fn invalid_arg_for_method(
    method: &str,
    index: usize,
    expected: &'static str,
) -> ScriptHostRuntimeError {
    ScriptHostRuntimeError::InvalidArgument {
        method: method.to_string(),
        index,
        expected,
    }
}

fn parse_server_time_zone_offset_milliseconds(offset: &str) -> Result<i32> {
    let trimmed = offset.trim();
    if trimmed.is_empty() {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let (sign, value) = match trimmed.as_bytes()[0] {
        b'-' => (-1_i64, &trimmed[1..]),
        b'+' => (1_i64, &trimmed[1..]),
        _ => (1_i64, trimmed),
    };

    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let hours = parse_offset_component(parts[0], offset)?;
    let minutes = parse_offset_component(parts[1], offset)?;
    let seconds = parse_offset_component(parts[2], offset)?;
    if minutes >= 60 || seconds >= 60 {
        return Err(ScriptHostRuntimeError::InvalidServerTimeZoneOffset(
            offset.to_string(),
        ));
    }

    let milliseconds =
        sign * ((hours * 60 * 60 * 1_000) + (minutes * 60 * 1_000) + (seconds * 1_000));
    i32::try_from(milliseconds)
        .map_err(|_| ScriptHostRuntimeError::InvalidServerTimeZoneOffset(offset.to_string()))
}

fn parse_offset_component(component: &str, original: &str) -> Result<i64> {
    component
        .parse::<i64>()
        .map_err(|_| ScriptHostRuntimeError::InvalidServerTimeZoneOffset(original.to_string()))
}

fn normalize_http_headers(headers_json: Option<&str>) -> Result<(Vec<(String, String)>, String)> {
    let mut headers = Vec::new();
    let mut content_type = "application/json".to_string();
    let Some(headers_json) = headers_json else {
        return Ok((headers, content_type));
    };
    if headers_json.trim().is_empty() {
        return Ok((headers, content_type));
    }

    let value: Value = serde_json::from_str(headers_json)
        .map_err(|_| ScriptHostRuntimeError::InvalidHttpHeaders)?;
    let Value::Object(map) = value else {
        return Err(ScriptHostRuntimeError::InvalidHttpHeaders);
    };

    for (key, value) in map {
        let Some(value) = value.as_str() else {
            return Err(ScriptHostRuntimeError::InvalidHttpHeaders);
        };
        let key = key.to_ascii_lowercase();
        if key == "content-type" {
            content_type = value.to_string();
        } else {
            headers.push((key, value.to_string()));
        }
    }
    headers.sort();
    Ok((headers, content_type))
}

fn parse_html_mask_data(json_data: &str) -> Result<Option<Value>> {
    if json_data.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(
        serde_json::from_str(json_data).unwrap_or_else(|_| Value::String(json_data.to_string())),
    ))
}

fn serialize_html_mask_message(message: &HtmlMaskMessage) -> Result<String> {
    serde_json::to_string(message)
        .map_err(|_| invalid_arg_for_method("htmlMask.message", 0, "serializable message"))
}

fn serialize_html_mask_messages(messages: &[HtmlMaskMessage]) -> Result<String> {
    serde_json::to_string(messages)
        .map_err(|_| invalid_arg_for_method("htmlMask.messages", 0, "serializable messages"))
}

fn is_http_url(url: &str) -> bool {
    url.get(..4)
        .map(|prefix| prefix.eq_ignore_ascii_case("http"))
        .unwrap_or(false)
}

fn path_to_file_url(path: &Path) -> String {
    let mut path = path.to_string_lossy().replace('\\', "/");
    if !path.starts_with('/') {
        path = format!("/{path}");
    }
    format!("file://{path}")
}

fn mouse_button_name(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "Left",
        MouseButton::Middle => "Middle",
        MouseButton::Right => "Right",
        MouseButton::X(1) => "XButton1",
        MouseButton::X(2) => "XButton2",
        MouseButton::X(_) => "XButton",
    }
}

fn parse_mouse_button_name(button: &str) -> MouseButton {
    match button {
        "Middle" | "middle" => MouseButton::Middle,
        "Right" | "right" => MouseButton::Right,
        "XButton1" | "xButton1" | "X1" | "x1" => MouseButton::X(1),
        "XButton2" | "xButton2" | "X2" | "x2" => MouseButton::X(2),
        _ => MouseButton::Left,
    }
}

#[derive(Debug, Clone)]
pub struct KeyMouseScriptHost {
    file_policy: ScriptFilePolicy,
    playback_context: MacroPlaybackContext,
}

impl KeyMouseScriptHost {
    pub fn new(script_root: impl Into<PathBuf>, playback_context: MacroPlaybackContext) -> Self {
        Self {
            file_policy: ScriptFilePolicy::new(script_root),
            playback_context,
        }
    }

    pub fn with_policy(
        file_policy: ScriptFilePolicy,
        playback_context: MacroPlaybackContext,
    ) -> Self {
        Self {
            file_policy,
            playback_context,
        }
    }

    pub fn file_policy(&self) -> &ScriptFilePolicy {
        &self.file_policy
    }

    pub fn playback_context(&self) -> MacroPlaybackContext {
        self.playback_context
    }

    pub fn run(&self, json: &str) -> Result<KeyMouseScriptRunPlan> {
        let script = KeyMouseScript::from_json(json)?;
        self.plan(script, KeyMouseScriptSource::InlineJson, None)
    }

    pub fn execute(
        &self,
        json: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run(json)?.execute(mode, window_handle)
    }

    pub fn execute_with_cancellation(
        &self,
        json: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run(json)?
            .execute_with_cancellation(mode, window_handle, cancellation)
    }

    pub fn run_file(&self, path: &str) -> Result<KeyMouseScriptRunPlan> {
        let normalized = self.file_policy.normalize_path(path)?;
        self.file_policy.validate_write_extension(&normalized)?;
        let json = read_text(&normalized)?;
        let script = KeyMouseScript::from_json(&json)?;
        self.plan(script, KeyMouseScriptSource::File, Some(normalized))
    }

    pub fn execute_file(
        &self,
        path: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run_file(path)?.execute(mode, window_handle)
    }

    pub fn execute_file_with_cancellation(
        &self,
        path: &str,
        mode: KeyMouseScriptDispatchMode,
        window_handle: Option<isize>,
        cancellation: Option<&InputCancellationToken>,
    ) -> Result<KeyMouseScriptExecution> {
        self.run_file(path)?
            .execute_with_cancellation(mode, window_handle, cancellation)
    }

    pub fn run_path(&self, path: impl AsRef<Path>) -> Result<KeyMouseScriptRunPlan> {
        let path_string = path.as_ref().to_string_lossy();
        self.run_file(&path_string)
    }

    fn plan(
        &self,
        script: KeyMouseScript,
        source: KeyMouseScriptSource,
        normalized_path: Option<PathBuf>,
    ) -> Result<KeyMouseScriptRunPlan> {
        let input_events = script.to_input_events(self.playback_context)?;
        Ok(KeyMouseScriptRunPlan {
            source,
            normalized_path,
            summary: script.summary(),
            input_events,
        })
    }
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| ScriptHostRuntimeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn append_input_event(sequence: InputSequence, event: InputEvent) -> InputSequence {
    match event {
        InputEvent::KeyDown { vk, .. } => sequence.key_down(vk),
        InputEvent::KeyUp { vk, .. } => sequence.key_up(vk),
        InputEvent::UnicodeChar { ch } => sequence.text(&ch.to_string()),
        InputEvent::MouseMoveRelative { dx, dy } => sequence.move_mouse_by(dx, dy),
        InputEvent::MouseMoveAbsolute {
            x,
            y,
            virtual_desktop,
        } => {
            if virtual_desktop {
                sequence.move_mouse_to_virtual_desktop(x, y)
            } else {
                sequence.move_mouse_to(x, y)
            }
        }
        InputEvent::MouseButtonDown { button } => sequence.mouse_down(button),
        InputEvent::MouseButtonUp { button } => sequence.mouse_up(button),
        InputEvent::MouseWheel { amount, horizontal } => {
            let clicks = amount / 120;
            if horizontal {
                sequence.horizontal_scroll(clicks)
            } else {
                sequence.vertical_scroll(clicks)
            }
        }
        InputEvent::Delay { milliseconds } => sequence.delay(milliseconds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#macro::MacroPlaybackContext;
    use bgi_input::{InputEvent, MouseButton};
    use std::sync::Arc;

    struct StaticFrameSource {
        frame: CaptureFrame,
        area: Option<GameCaptureArea>,
    }

    impl GameCaptureFrameSource for StaticFrameSource {
        fn capture_frame(&self) -> Result<CaptureFrame> {
            Ok(self.frame.clone())
        }

        fn capture_frame_area(&self, frame: &CaptureFrame) -> GameCaptureArea {
            self.area.unwrap_or(GameCaptureArea {
                x: 0,
                y: 0,
                width: frame.size.width,
                height: frame.size.height,
            })
        }
    }

    #[test]
    fn inline_key_mouse_script_builds_input_plan() {
        let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
        let plan = host
            .run(
                r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
            )
            .unwrap();

        assert_eq!(plan.source, KeyMouseScriptSource::InlineJson);
        assert_eq!(plan.summary.event_count, 2);
        assert_eq!(
            plan.input_events,
            vec![
                InputEvent::Delay { milliseconds: 10 },
                InputEvent::KeyDown {
                    vk: 87,
                    extended: None
                },
                InputEvent::Delay { milliseconds: 20 },
                InputEvent::KeyUp {
                    vk: 87,
                    extended: None
                }
            ]
        );
    }

    #[test]
    fn key_mouse_script_plan_only_execution_reports_events_without_dispatch() {
        let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
        let execution = host
            .execute(
                r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
                KeyMouseScriptDispatchMode::PlanOnly,
                None,
            )
            .unwrap();

        assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
        assert!(!execution.dispatched);
        assert_eq!(execution.dispatched_events, 0);
        assert_eq!(execution.plan.summary.event_count, 2);
        assert_eq!(execution.plan.input_events.len(), 4);
    }

    #[test]
    fn key_mouse_script_send_input_honors_pre_cancelled_token() {
        let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
        let cancellation = InputCancellationToken::new();
        cancellation.cancel();

        let execution = host
            .execute_with_cancellation(
                r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 87, "time": 10 },
                    { "type": 1, "keyCode": 87, "time": 30 }
                  ]
                }"#,
                KeyMouseScriptDispatchMode::SendInput,
                None,
                Some(&cancellation),
            )
            .unwrap();

        assert_eq!(execution.mode, KeyMouseScriptDispatchMode::SendInput);
        assert!(execution.dispatched);
        assert!(execution.cancelled);
        assert_eq!(execution.dispatched_events, 0);
        assert_eq!(execution.processed_events, 0);
        assert_eq!(execution.plan.input_events.len(), 4);
    }

    #[test]
    fn run_file_uses_script_file_policy_root() {
        let root = std::env::temp_dir().join(format!(
            "bgi-keymouse-host-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let script_path = root.join("macro.json");
        fs::write(
            &script_path,
            r#"{
              "macroEvents": [
                { "type": 4, "mouseButton": "Left", "mouseX": 16, "mouseY": 16, "time": 0 }
              ]
            }"#,
        )
        .unwrap();

        let host = KeyMouseScriptHost::new(&root, MacroPlaybackContext::default());
        let plan = host.run_file("macro.json").unwrap();

        assert_eq!(plan.source, KeyMouseScriptSource::File);
        assert_eq!(plan.normalized_path, Some(script_path));
        assert_eq!(
            plan.input_events,
            vec![
                InputEvent::MouseMoveAbsolute {
                    x: 546,
                    y: 970,
                    virtual_desktop: false
                },
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left
                }
            ]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn run_file_rejects_path_traversal() {
        let host = KeyMouseScriptHost::new(".", MacroPlaybackContext::default());
        let error = host.run_file("../macro.json").unwrap_err();

        assert!(matches!(
            error,
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
        ));
    }

    #[test]
    fn global_input_host_maps_keyboard_and_mouse_keys() {
        let host = GlobalInputHost::new(
            GameCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
        )
        .unwrap();

        assert_eq!(
            host.key_press("VK_F").unwrap().events(),
            &[
                InputEvent::KeyDown {
                    vk: 0x46,
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: 0x46,
                    extended: None
                }
            ]
        );
        assert_eq!(
            host.key_down("VK_LBUTTON").unwrap().events(),
            &[InputEvent::MouseButtonDown {
                button: MouseButton::Left
            }]
        );
    }

    #[test]
    fn global_input_host_tracks_game_metrics_and_coordinates() {
        let mut host = GlobalInputHost::new(
            GameCaptureArea {
                x: 100,
                y: 50,
                width: 1280,
                height: 720,
            },
            2.0,
        )
        .unwrap();

        host.set_game_metrics(1920, 1080, 1.0).unwrap();

        assert_eq!(
            host.move_mouse_to(960, 540).unwrap().events(),
            &[InputEvent::MouseMoveAbsolute {
                x: 740,
                y: 410,
                virtual_desktop: false
            }]
        );
        assert_eq!(
            host.move_mouse_by(10, -5).events(),
            &[InputEvent::MouseMoveRelative { dx: 20, dy: -10 }]
        );
    }

    #[test]
    fn global_input_host_rejects_non_16_by_9_metrics() {
        let mut host = GlobalInputHost::new(
            GameCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
        )
        .unwrap();

        let error = host.set_game_metrics(1024, 768, 1.0).unwrap_err();
        assert!(matches!(
            error,
            ScriptHostRuntimeError::InvalidGameMetrics {
                width: 1024,
                height: 768
            }
        ));
    }

    #[test]
    fn global_input_text_uses_unicode_events() {
        let host = GlobalInputHost::new(
            GameCaptureArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            1.0,
        )
        .unwrap();

        assert_eq!(
            host.input_text("GI").events(),
            &[
                InputEvent::UnicodeChar { ch: 'G' },
                InputEvent::UnicodeChar { ch: 'I' }
            ]
        );
    }

    #[test]
    fn key_mouse_hook_host_dispatches_registered_listeners() {
        let mut host = KeyMouseHookHost::default();
        host.on_key_down(Some("key-code"), true);
        host.on_key_down(Some("key-data"), false);
        host.on_mouse_down(Some("mouse-down"));
        host.on_mouse_move(Some("move-fast"), 50);
        host.on_mouse_move(Some("move-slow"), 200);
        host.on_mouse_wheel(Some("wheel"));

        let key_dispatches = host.dispatch_event(KeyMouseHookEvent::Key {
            event: KeyMouseHookEventKind::KeyDown,
            key_data: "Control, F".to_string(),
            key_code: "F".to_string(),
        });
        assert_eq!(key_dispatches.len(), 2);
        assert_eq!(key_dispatches[0].listener_id, "key-code");
        assert_eq!(key_dispatches[0].args, vec![serde_json::json!("F")]);
        assert_eq!(key_dispatches[1].listener_id, "key-data");
        assert_eq!(
            key_dispatches[1].args,
            vec![serde_json::json!("Control, F")]
        );

        let mouse_down = host.dispatch_event(KeyMouseHookEvent::MouseButton {
            event: KeyMouseHookEventKind::MouseDown,
            button: MouseButton::Right,
            x: 12,
            y: 34,
        });
        assert_eq!(
            mouse_down[0].args,
            vec![
                serde_json::json!("Right"),
                serde_json::json!(12),
                serde_json::json!(34)
            ]
        );

        let first_move = host.dispatch_event(KeyMouseHookEvent::MouseMove {
            x: 10,
            y: 20,
            timestamp_ms: 100,
        });
        assert_eq!(first_move.len(), 2);
        let throttled_global = host.dispatch_event(KeyMouseHookEvent::MouseMove {
            x: 11,
            y: 21,
            timestamp_ms: 105,
        });
        assert!(throttled_global.is_empty());
        let fast_only = host.dispatch_event(KeyMouseHookEvent::MouseMove {
            x: 12,
            y: 22,
            timestamp_ms: 160,
        });
        assert_eq!(fast_only.len(), 1);
        assert_eq!(fast_only[0].listener_id, "move-fast");

        let wheel = host.dispatch_event(KeyMouseHookEvent::MouseWheel {
            delta: -120,
            x: 7,
            y: 8,
        });
        assert_eq!(
            wheel[0].args,
            vec![
                serde_json::json!(-120),
                serde_json::json!(7),
                serde_json::json!(8)
            ]
        );

        host.remove_all_listeners();
        assert!(host.listeners().is_empty());
        host.on_key_up(Some("key-up"), true);
        host.dispose();
        assert!(host.snapshot().disposed);
        assert!(host
            .dispatch_event(KeyMouseHookEvent::Key {
                event: KeyMouseHookEventKind::KeyUp,
                key_data: "F".to_string(),
                key_code: "F".to_string(),
            })
            .is_empty());
    }

    #[test]
    fn realtime_timer_and_solo_task_models_preserve_legacy_defaults() {
        let timer = RealtimeTimerHostPlan::new("AutoPick", None);
        assert_eq!(timer.name, "AutoPick");
        assert_eq!(timer.interval_ms, 50);
        assert_eq!(timer.config, None);

        let auto_pick = AutoPickExternalConfig::from_value(Some(&serde_json::json!({
            "textList": ["Open", "Pick"],
            "forceInteraction": true
        })))
        .unwrap();
        assert_eq!(
            auto_pick.to_legacy_config_value(),
            serde_json::json!({
                "TextList": ["Open", "Pick"],
                "ForceInteraction": true
            })
        );

        let call = ScriptHostCall::new(
            ScriptHostTarget::Dispatcher,
            "AddTrigger",
            vec![serde_json::json!({
                "name": "AutoPick",
                "config": {
                    "textList": ["Open"],
                    "forceInteraction": true
                }
            })],
        );
        let timer = timer_plan_from_arg(&call, 0, false).unwrap();
        assert_eq!(
            timer.config,
            Some(serde_json::json!({
                "TextList": ["Open"],
                "ForceInteraction": true
            }))
        );

        let solo = SoloTaskHostPlan::new("AutoFight", Some(serde_json::json!({"team": "daily"})));
        assert_eq!(solo.name, "AutoFight");
        assert!(solo.uses_linked_cancellation);
    }

    #[test]
    fn limited_file_host_reads_writes_lists_and_renames() {
        let root = test_root("bgi-limited-file-host");
        let host = LimitedFileHost::new(&root);

        assert!(host.create_directory("nested").unwrap());
        assert!(host
            .write_text_sync("nested/a.txt", "hello", false)
            .unwrap());
        assert!(host
            .write_text_sync("nested/a.txt", " world", true)
            .unwrap());
        assert_eq!(host.read_text_sync("nested/a.txt").unwrap(), "hello world");
        assert!(host.is_file("nested/a.txt").unwrap());
        assert!(host.is_folder("nested").unwrap());
        assert!(host.is_exists("nested/a.txt").unwrap());

        let entries = host.read_path_sync("nested").unwrap();
        assert_eq!(entries, vec!["nested/a.txt"]);

        assert!(host
            .rename_path_sync("nested/a.txt", "nested/b.txt")
            .unwrap());
        assert!(!host.is_exists("nested/a.txt").unwrap());
        assert!(host.is_file("nested/b.txt").unwrap());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn limited_file_host_rejects_disallowed_write_extension() {
        let root = test_root("bgi-limited-file-policy");
        let host = LimitedFileHost::new(&root);

        let error = host.write_text_sync("bad.exe", "x", false).unwrap_err();

        assert!(matches!(
            error,
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::ExtensionNotAllowed(ext))
                if ext == ".exe"
        ));

        fs::remove_dir_all(root).unwrap_or(());
    }

    #[test]
    fn limited_file_host_plans_image_mat_io() {
        let root = test_root("bgi-limited-file-image-policy");
        let host = LimitedFileHost::new(&root);

        let read = host.read_image_mat_plan_sync("assets/icon.png").unwrap();
        assert_eq!(read.normalized_path, root.join("assets/icon.png"));
        assert_eq!(read.color_mode, "color");
        assert_eq!(read.resize, None);

        let resized = host
            .read_image_mat_with_resize_plan_sync("assets/icon.webp", 64.0, 32.0, 4)
            .unwrap();
        assert_eq!(
            resized.resize,
            Some(ImageMatResizePlan {
                width: 64.0,
                height: 32.0,
                interpolation: 4
            })
        );

        let write = host
            .write_image_plan_sync("output/avatar", serde_json::json!({"matHandle": "m1"}))
            .unwrap();
        assert_eq!(write.normalized_path, root.join("output/avatar.png"));
        assert_eq!(write.source, serde_json::json!({"matHandle": "m1"}));

        let error = host
            .read_image_mat_with_resize_sync("assets/icon.png", 0.0, 32.0, 1)
            .unwrap_err();
        assert!(matches!(
            error,
            ScriptHostRuntimeError::InvalidArgument { index: 1, .. }
        ));

        fs::remove_dir_all(root).unwrap_or(());
    }

    #[test]
    fn limited_file_host_executes_image_mat_io() {
        let root = test_root("bgi-limited-file-image-exec");
        let host = LimitedFileHost::new(&root);
        fs::create_dir_all(root.join("assets")).unwrap();
        let source = BgrImage::new(
            VisionSize::new(2, 2),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        )
        .unwrap();
        source.write_png(root.join("assets/icon.png")).unwrap();

        let read = host.read_image_mat_sync("assets/icon.png").unwrap();
        let resized = host
            .read_image_mat_with_resize_sync("assets/icon.png", 1.0, 1.0, 1)
            .unwrap();
        let write = host
            .write_image_sync(
                "output/copy",
                serde_json::json!({
                    "width": read.width,
                    "height": read.height,
                    "pixelFormat": read.pixel_format,
                    "pixels": read.pixels,
                }),
            )
            .unwrap();
        let copied = BgrImage::read(root.join("output/copy.png")).unwrap();

        assert_eq!(read.normalized_path, root.join("assets/icon.png"));
        assert_eq!(read.width, 2);
        assert_eq!(read.height, 2);
        assert_eq!(read.pixel_format, "BGR24");
        assert_eq!(read.pixels, source.pixels);
        assert_eq!(resized.width, 1);
        assert_eq!(resized.height, 1);
        assert_eq!(resized.pixels, vec![1, 2, 3]);
        assert_eq!(write.normalized_path, root.join("output/copy.png"));
        assert_eq!(write.width, 2);
        assert_eq!(write.height, 2);
        assert!(write.bytes_written > 0);
        assert_eq!(copied, source);

        fs::remove_dir_all(root).unwrap_or(());
    }

    #[test]
    fn strategy_file_host_is_limited_to_its_root() {
        let root = test_root("bgi-strategy-file-host");
        fs::create_dir_all(root.join("teams")).unwrap();
        fs::write(root.join("teams/default.json"), "{}").unwrap();

        let host = StrategyFileHost::new(&root);

        assert!(host.is_folder("teams").unwrap());
        assert!(host.is_file("teams/default.json").unwrap());
        assert_eq!(
            host.read_path_sync("teams").unwrap(),
            vec!["teams/default.json"]
        );
        assert!(matches!(
            host.is_exists("../outside.json").unwrap_err(),
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn log_host_records_script_log_levels() {
        let mut host = ScriptLogHost::default();
        host.debug("a");
        host.info("b");
        host.warn("c");
        host.error("d");

        assert_eq!(host.records().len(), 4);
        assert_eq!(host.records()[2].level, ScriptLogLevel::Warn);
        assert_eq!(host.records()[3].message, "d");
    }

    #[test]
    fn notification_host_applies_policy_and_rate_limit() {
        let policy = ScriptNotificationPolicy::new(true, true);
        let mut host = ScriptNotificationHost::new(policy);

        for i in 0..5 {
            host.send_at(&format!("msg{i}"), i).unwrap();
        }
        let error = host.error_at("too fast", 10).unwrap_err();
        assert!(matches!(
            error,
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::NotificationRateLimited)
        ));

        assert_eq!(host.records().len(), 5);
    }

    #[test]
    fn notification_host_delivers_to_sink_with_legacy_event_codes() {
        let policy = ScriptNotificationPolicy::new(true, true);
        let mut host = ScriptNotificationHost::new(policy);
        let mut sink = RecordingNotificationSink::default();

        let delivery = host.send_to("ready", 100, &mut sink).unwrap();
        assert_eq!(delivery.event_code, "js.custom");
        assert_eq!(delivery.result, "success");
        assert_eq!(delivery.message, "ready");

        let delivery = host.error_to("failed", 101, &mut sink).unwrap();
        assert_eq!(delivery.event_code, "js.error");
        assert_eq!(delivery.result, "fail");
        assert_eq!(sink.deliveries().len(), 2);
        assert_eq!(host.records().len(), 2);
    }

    #[test]
    fn http_host_builds_request_plan_and_normalizes_headers() {
        let host = HttpHost::new(ScriptHttpPolicy::new(
            true,
            ["https://example.com/*".to_string()],
        ));

        let plan = host
            .request(
                "post",
                "https://example.com/api",
                Some("{\"ok\":true}"),
                Some(r#"{"Content-Type":"text/plain","X-Test":"1"}"#),
            )
            .unwrap();

        assert_eq!(plan.method, "POST");
        assert_eq!(plan.body, Some("{\"ok\":true}".to_string()));
        assert_eq!(plan.content_type, "text/plain");
        assert_eq!(plan.headers, vec![("x-test".to_string(), "1".to_string())]);
        assert!(matches!(
            host.request("GET", "https://blocked.example/api", None, None)
                .unwrap_err(),
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::HttpUrlDenied(_))
        ));
        assert!(matches!(
            host.request("GET", "https://example.com/api", None, Some("[]"))
                .unwrap_err(),
            ScriptHostRuntimeError::InvalidHttpHeaders
        ));
    }

    #[test]
    fn http_host_executes_request_through_pluggable_client() {
        let host = HttpHost::new(ScriptHttpPolicy::new(
            true,
            ["https://example.com/*".to_string()],
        ));
        let mut client = RecordingHttpClient::ok_json("{\"ok\":true}");

        let response = host
            .execute_request(
                "post",
                "https://example.com/api",
                Some("{\"ok\":true}"),
                Some(r#"{"Content-Type":"text/plain","X-Test":"1"}"#),
                &mut client,
            )
            .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "{\"ok\":true}");
        assert_eq!(
            response.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(client.requests().len(), 1);
        assert_eq!(client.requests()[0].method, "POST");
        assert_eq!(client.requests()[0].content_type, "text/plain");
        assert_eq!(
            client.requests()[0].headers,
            vec![("x-test".to_string(), "1".to_string())]
        );
    }

    #[test]
    fn html_mask_host_plans_windows_and_message_queues() {
        let root = test_root("bgi-html-mask-host");
        fs::write(root.join("overlay.html"), "<html></html>").unwrap();
        let mut host = HtmlMaskHost::new(&root);

        let command = host.show("overlay.html", Some("mask")).unwrap();
        let HtmlMaskCommand::Show(plan) = command else {
            panic!("expected show command");
        };
        assert_eq!(plan.window_id, "mask");
        assert!(plan.final_url.starts_with("file:///"));
        assert!(plan.normalized_path.unwrap().ends_with("overlay.html"));
        assert!(host.exists("mask"));
        assert_eq!(host.window_ids(), vec!["mask".to_string()]);

        host.show("https://example.com/widget", Some("remote"))
            .unwrap();
        assert_eq!(
            host.snapshot()
                .windows
                .iter()
                .find(|window| window.window_id == "remote")
                .unwrap()
                .final_url,
            "https://example.com/widget"
        );

        let send = host.send("mask", "/status", r#"{"ready":true}"#).unwrap();
        assert!(matches!(
            send,
            HtmlMaskCommand::Send {
                ref window_id,
                ref message,
            } if window_id == "mask" && message.data.as_ref().unwrap()["ready"] == true
        ));
        let send_json = serde_json::to_string(&send).unwrap();
        assert!(send_json.contains("\"window_id\":\"mask\""));
        assert!(!send_json.contains("request_id"));
        assert!(!send_json.contains("requestId"));
        let pending = host.flush_pending_messages("mask").unwrap();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].contains("\"ready\":true"));

        let request = host
            .request("mask", "/status", r#"{"wait":true}"#, 250)
            .unwrap();
        let request_json = serde_json::to_string(&request).unwrap();
        assert!(request_json.contains("\"requestId\":\"request-1\""));
        assert!(request_json.contains("\"timeout_ms\":250"));
        assert_eq!(host.flush_pending_messages("mask").unwrap().len(), 1);

        host.send_from_html("mask", "/event", r#"{"ok":true}"#, None)
            .unwrap();
        assert_eq!(
            host.poll("mask").unwrap(),
            Some(r#"{"url":"/event","data":{"ok":true}}"#.to_string())
        );
        assert_eq!(host.poll("mask").unwrap(), None);

        host.send_from_html("mask", "/a", "plain text", Some("response-1"))
            .unwrap();
        let all = host.poll_all("mask").unwrap();
        assert!(all.contains("\"data\":\"plain text\""));
        assert!(all.contains("\"requestId\":\"response-1\""));

        host.toggle_click_through("mask").unwrap();
        assert!(host.get_click_through("mask").unwrap());
        host.close("mask");
        assert!(!host.exists("mask"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn html_mask_host_can_resume_from_desktop_bridge_state() {
        let root = test_root("bgi-html-mask-resume");
        fs::write(root.join("overlay.html"), "<html></html>").unwrap();
        let plan = HtmlMaskWindowPlan {
            window_id: "mask".to_string(),
            final_url: path_to_file_url(&root.join("overlay.html")),
            requested_url: "overlay.html".to_string(),
            normalized_path: Some(root.join("overlay.html")),
            click_through: true,
        };
        let message = HtmlMaskMessage {
            url: "/event".to_string(),
            data: Some(serde_json::json!({ "ok": true })),
            request_id: Some("req-1".to_string()),
        };
        let mut host = HtmlMaskHost::with_initial_state(
            &root,
            HtmlMaskInitialState {
                windows: vec![plan],
                from_html: vec![("mask".to_string(), message)],
            },
        );

        assert!(host.exists("mask"));
        assert!(host.get_click_through("mask").unwrap());
        let polled = host.poll("mask").unwrap().unwrap();
        assert!(polled.contains("\"url\":\"/event\""));
        assert!(polled.contains("\"requestId\":\"req-1\""));
        assert!(host.remaining_from_html_messages().is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn server_time_host_parses_legacy_timespan_offsets() {
        assert_eq!(
            ServerTimeHost::from_offset_string("08:00:00")
                .unwrap()
                .server_time_zone_offset_milliseconds(),
            28_800_000
        );
        assert_eq!(
            ServerTimeHost::from_offset_string("-05:30:00")
                .unwrap()
                .server_time_zone_offset_milliseconds(),
            -19_800_000
        );
        assert!(matches!(
            ServerTimeHost::from_offset_string("08:99:00").unwrap_err(),
            ScriptHostRuntimeError::InvalidServerTimeZoneOffset(_)
        ));
    }

    #[test]
    fn script_host_runtime_routes_global_and_post_message_calls() {
        let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

        let version = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "GetVersion",
                Vec::new(),
            ))
            .unwrap();
        assert_eq!(
            version,
            ScriptHostCallResult::String(env!("CARGO_PKG_VERSION").to_string())
        );

        let global = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "KeyPress",
                vec![serde_json::json!("VK_F")],
            ))
            .unwrap();
        let ScriptHostCallResult::InputExecution(global) = global else {
            panic!("expected input execution");
        };
        assert_eq!(global.mode, GlobalInputDispatchMode::PlanOnly);
        assert!(!global.dispatched);
        assert_eq!(global.dispatched_events, 0);
        assert_eq!(
            global.events,
            vec![
                InputEvent::KeyDown {
                    vk: 0x46,
                    extended: None
                },
                InputEvent::KeyUp {
                    vk: 0x46,
                    extended: None
                }
            ]
        );

        let post_message = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PostMessage,
                "KeyPress",
                vec![serde_json::json!("VK_F")],
            ))
            .unwrap();
        let ScriptHostCallResult::PostMessageEvents(events) = post_message else {
            panic!("expected post message events");
        };
        assert!(matches!(
            events.first(),
            Some(PostMessageEvent::Message {
                message: bgi_input::WM_ACTIVATE,
                ..
            })
        ));
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn script_host_runtime_routes_global_vision_and_host_helper_plans() {
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.capture_area = GameCaptureArea {
            x: 100,
            y: 50,
            width: 1280,
            height: 720,
        };
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let capture = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "CaptureGameRegion",
                Vec::new(),
            ))
            .unwrap();
        assert!(matches!(
            capture,
            ScriptHostCallResult::CaptureGameRegionPlan(CaptureGameRegionPlan {
                area: GameCaptureArea {
                    x: 100,
                    y: 50,
                    width: 1280,
                    height: 720
                },
                pixel_format: "BGR24",
                source: "game_capture_region"
            })
        ));

        assert!(runtime.global_input.capture_frame_source.is_none());

        let avatars = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "GetAvatars",
                Vec::new(),
            ))
            .unwrap();
        assert!(matches!(
            avatars,
            ScriptHostCallResult::AvatarRecognitionPlan(AvatarRecognitionPlan {
                model_name: "BgiAvatarSide",
                model_relative_path: "Assets/Model/Common/avatar_side_classify_sim.onnx",
                output: "avatar_names",
                ..
            })
        ));

        let array_plan = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::CustomHostFunctions,
                "NewVarOfArr",
                vec![
                    serde_json::json!("OpenCvSharp.Point2f"),
                    serde_json::json!(2),
                ],
            ))
            .unwrap();
        assert_eq!(
            array_plan,
            ScriptHostCallResult::CustomHostFunctionCommand(
                CustomHostFunctionCommand::NewArrayVariable {
                    element_type: "OpenCvSharp.Point2f".to_string(),
                    dimensions: 2,
                    legacy_jagged_type: "OpenCvSharp.Point2f[][]".to_string()
                }
            )
        );
    }

    #[test]
    fn script_host_runtime_executes_injected_capture_game_region_source() {
        let source_pixels = vec![
            1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, //
            5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, //
            9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12,
        ];
        let frame = CaptureFrame::packed_bgr(4, 3, source_pixels).unwrap();
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.capture_area = GameCaptureArea {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        config.capture_frame_source = Some(Arc::new(StaticFrameSource {
            frame,
            area: Some(GameCaptureArea {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            }),
        }));
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let result = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "captureGameRegion",
                Vec::new(),
            ))
            .unwrap();
        let ScriptHostCallResult::CaptureGameRegionExecution(capture) = result else {
            panic!("expected capture execution");
        };

        assert_eq!(capture.width, 2);
        assert_eq!(capture.height, 2);
        assert_eq!(capture.pixel_format, "BGR24");
        assert_eq!(capture.source_width, 4);
        assert_eq!(capture.source_height, 3);
        assert_eq!(
            capture.pixels,
            vec![6, 6, 6, 7, 7, 7, 10, 10, 10, 11, 11, 11]
        );
        assert_eq!(capture.plan.area.x, 1);
        assert_eq!(
            capture.image_region.source,
            bgi_vision::ImageRegionSource::DerivedCrop
        );
        assert_eq!(capture.image_region.rect, Rect::new(1, 1, 2, 2).unwrap());
    }

    #[test]
    fn script_host_runtime_uses_initial_game_metrics_for_global_input() {
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.capture_area = GameCaptureArea {
            x: 120,
            y: 108,
            width: 1600,
            height: 900,
        };
        config.initial_game_metrics = Some(GameMetrics {
            width: 1600,
            height: 900,
            dpi: 1.0,
        });
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let metrics = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "getGameMetrics",
                Vec::new(),
            ))
            .unwrap();
        assert_eq!(
            metrics,
            ScriptHostCallResult::GameMetrics(GameMetrics {
                width: 1600,
                height: 900,
                dpi: 1.0
            })
        );

        let click = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "click",
                vec![serde_json::json!(800), serde_json::json!(450)],
            ))
            .unwrap();
        let ScriptHostCallResult::InputExecution(execution) = click else {
            panic!("expected input execution");
        };
        assert!(matches!(
            execution.events.first(),
            Some(InputEvent::MouseMoveAbsolute {
                x: 920,
                y: 558,
                virtual_desktop: false
            })
        ));
    }

    #[test]
    fn injected_capture_frame_source_defaults_to_full_frame_region() {
        let frame = CaptureFrame::packed_bgr(
            2,
            2,
            vec![
                1, 1, 1, 2, 2, 2, //
                3, 3, 3, 4, 4, 4,
            ],
        )
        .unwrap();
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.capture_area = GameCaptureArea {
            x: 200,
            y: 100,
            width: 1280,
            height: 720,
        };
        config.capture_frame_source = Some(Arc::new(StaticFrameSource { frame, area: None }));
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let result = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Global,
                "captureGameRegion",
                Vec::new(),
            ))
            .unwrap();
        let ScriptHostCallResult::CaptureGameRegionExecution(capture) = result else {
            panic!("expected capture execution");
        };

        assert_eq!(capture.width, 2);
        assert_eq!(capture.height, 2);
        assert_eq!(capture.pixels, vec![1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4]);
        assert_eq!(
            capture.plan.area,
            GameCaptureArea {
                x: 0,
                y: 0,
                width: 2,
                height: 2
            }
        );
    }

    #[test]
    fn script_host_runtime_routes_server_time_calls() {
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.server_time_zone_offset_milliseconds = -18_000_000;
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::ServerTime,
                    "GetServerTimeZoneOffset",
                    Vec::new(),
                ))
                .unwrap(),
            ScriptHostCallResult::Integer(-18_000_000)
        );
    }

    #[test]
    fn script_host_runtime_routes_http_calls() {
        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let result = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Http,
                "Request",
                vec![
                    serde_json::json!("GET"),
                    serde_json::json!("https://example.com/status"),
                    Value::Null,
                    serde_json::json!(r#"{"Accept":"application/json"}"#),
                ],
            ))
            .unwrap();

        let ScriptHostCallResult::HttpRequestPlan(plan) = result else {
            panic!("expected HTTP request plan");
        };
        assert_eq!(plan.method, "GET");
        assert_eq!(plan.url, "https://example.com/status");
        assert_eq!(
            plan.headers,
            vec![("accept".to_string(), "application/json".to_string())]
        );

        let mut config = ScriptHostRuntimeConfig::new(".", ".");
        config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
        config.http_dispatch_mode = HttpDispatchMode::Reqwest;
        let runtime = ScriptHostRuntime::new(config).unwrap();
        let mut client = RecordingHttpClient::ok_json(r#"{"status":"ok"}"#);
        let result = runtime
            .call_http_with_client(
                ScriptHostCall::new(
                    ScriptHostTarget::Http,
                    "Request",
                    vec![
                        serde_json::json!("POST"),
                        serde_json::json!("https://example.com/status"),
                        serde_json::json!(r#"{"ping":true}"#),
                        serde_json::json!(r#"{"Content-Type":"application/json"}"#),
                    ],
                ),
                &mut client,
            )
            .unwrap();
        let ScriptHostCallResult::HttpExecution(execution) = result else {
            panic!("expected HTTP execution");
        };
        assert_eq!(execution.mode, HttpDispatchMode::Reqwest);
        assert!(execution.dispatched);
        assert_eq!(execution.request.method, "POST");
        assert_eq!(execution.response.as_ref().unwrap().status_code, 200);
        assert_eq!(
            execution.response.as_ref().unwrap().body,
            r#"{"status":"ok"}"#
        );
        assert_eq!(client.requests().len(), 1);
    }

    #[test]
    fn script_host_runtime_routes_html_mask_calls() {
        let root = test_root("bgi-script-host-runtime-html-mask");
        fs::write(root.join("overlay.html"), "<html></html>").unwrap();
        let mut runtime =
            ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&root, &root)).unwrap();

        let show = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "Show",
                vec![serde_json::json!("overlay.html"), serde_json::json!("mask")],
            ))
            .unwrap();
        assert!(matches!(
            show,
            ScriptHostCallResult::HtmlMaskCommand(HtmlMaskCommand::Show(ref plan))
                if plan.window_id == "mask" && plan.final_url.starts_with("file:///")
        ));

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "Send",
                vec![
                    serde_json::json!("mask"),
                    serde_json::json!("/status"),
                    serde_json::json!("{\"ready\":true}"),
                ],
            ))
            .unwrap();
        let flushed = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "FlushPendingMessages",
                vec![serde_json::json!("mask")],
            ))
            .unwrap();
        let ScriptHostCallResult::StringList(flushed) = flushed else {
            panic!("expected flushed messages");
        };
        assert_eq!(flushed.len(), 1);

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "SendFromHtml",
                vec![
                    serde_json::json!("mask"),
                    serde_json::json!("/event"),
                    serde_json::json!("{\"ok\":true}"),
                ],
            ))
            .unwrap();
        let polled = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "Poll",
                vec![serde_json::json!("mask")],
            ))
            .unwrap();
        assert!(matches!(
            polled,
            ScriptHostCallResult::String(ref message) if message.contains("\"ok\":true")
        ));

        let snapshot = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::HtmlMask,
                "Snapshot",
                Vec::new(),
            ))
            .unwrap();
        assert!(matches!(
            snapshot,
            ScriptHostCallResult::HtmlMaskSnapshot(HtmlMaskSnapshot { ref windows, .. })
                if windows.len() == 1
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_host_runtime_routes_key_mouse_hook_calls() {
        let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

        let registration = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "OnKeyDown",
                vec![serde_json::json!("key"), serde_json::json!(true)],
            ))
            .unwrap();
        assert!(matches!(
            registration,
            ScriptHostCallResult::KeyMouseHookCommand(KeyMouseHookCommand::AddListener(ref listener))
                if listener.id == "key" && listener.event == KeyMouseHookEventKind::KeyDown
        ));
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "OnMouseMove",
                vec![serde_json::json!("move"), serde_json::json!(25)],
            ))
            .unwrap();

        let key_dispatch = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "DispatchEvent",
                vec![serde_json::json!({
                    "type": "keyDown",
                    "keyData": "Control, F",
                    "keyCode": "F"
                })],
            ))
            .unwrap();
        let ScriptHostCallResult::KeyMouseHookDispatches(key_dispatches) = key_dispatch else {
            panic!("expected key hook dispatches");
        };
        assert_eq!(key_dispatches.len(), 1);
        assert_eq!(key_dispatches[0].args, vec![serde_json::json!("F")]);

        let move_dispatch = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "DispatchEvent",
                vec![serde_json::json!({
                    "type": "mouseMove",
                    "x": 10,
                    "y": 20,
                    "timestampMs": 100
                })],
            ))
            .unwrap();
        assert!(matches!(
            move_dispatch,
            ScriptHostCallResult::KeyMouseHookDispatches(ref dispatches)
                if dispatches.len() == 1 && dispatches[0].listener_id == "move"
        ));

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "RemoveAllListeners",
                Vec::new(),
            ))
            .unwrap();
        let snapshot = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseHook,
                "Snapshot",
                Vec::new(),
            ))
            .unwrap();
        assert!(matches!(
            snapshot,
            ScriptHostCallResult::KeyMouseHookSnapshot(KeyMouseHookSnapshot { ref listeners, .. })
                if listeners.is_empty()
        ));
    }

    #[test]
    fn dispatcher_host_records_timer_and_task_plans() {
        let mut host = ScriptDispatcherHost::default();

        let timer = RealtimeTimerHostPlan {
            name: "AutoPick".to_string(),
            interval_ms: 50,
            config: Some(serde_json::json!({"enabled": true})),
            clears_existing_triggers: false,
        };
        let add = host.add_timer(timer);
        assert!(matches!(
            host.commands()[0],
            DispatcherCommand::ClearAllTriggers
        ));
        assert!(matches!(add, DispatcherCommand::AddRealtimeTimer(_)));
        assert_eq!(host.commands().len(), 2);

        let task = SoloTaskHostPlan {
            name: "AutoFight".to_string(),
            config: None,
            uses_linked_cancellation: true,
        };
        let run = host.run_solo_task(task);
        assert!(matches!(run, DispatcherCommand::RunSoloTask(_)));
        assert_eq!(host.commands().len(), 3);
    }

    #[test]
    fn script_host_runtime_routes_dispatcher_calls() {
        let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

        let add = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Dispatcher,
                "AddTrigger",
                vec![serde_json::json!({
                    "Name": "AutoSkip",
                    "Interval": 100,
                    "Config": { "quickTeleportEnabled": true }
                })],
            ))
            .unwrap();
        let ScriptHostCallResult::DispatcherCommand(DispatcherCommand::AddRealtimeTimer(timer)) =
            add
        else {
            panic!("expected add realtime timer command");
        };
        assert_eq!(timer.name, "AutoSkip");
        assert_eq!(timer.interval_ms, 100);
        assert!(!timer.clears_existing_triggers);

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Dispatcher,
                "RunTask",
                vec![serde_json::json!({
                    "name": "AutoFight",
                    "config": { "strategy": "default" }
                })],
            ))
            .unwrap();
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Dispatcher,
                "RunAutoBossTask",
                vec![serde_json::json!({"boss": "sample"})],
            ))
            .unwrap();

        assert_eq!(runtime.dispatcher_commands().len(), 3);
        assert!(matches!(
            runtime.dispatcher_commands()[2],
            DispatcherCommand::RunBuiltinTask { ref name, .. } if name == "AutoBoss"
        ));

        let plans = runtime.dispatcher_task_invocation_plans().unwrap();
        assert_eq!(plans[0].task_key.as_deref(), Some("AutoSkip"));
        assert_eq!(
            plans[1].kind,
            bgi_task::TaskInvocationKind::RunIndependentTask
        );
        assert_eq!(plans[2].task_key.as_deref(), Some("AutoBoss"));
    }

    #[test]
    fn genshin_host_maps_action_commands_to_task_invocations() {
        let mut host = GenshinHost::default();
        host.push(GenshinCommand::Uid);
        host.push(GenshinCommand::Teleport {
            x: 100.5,
            y: 200.25,
            map_name: None,
            force: true,
        });
        host.push(GenshinCommand::SwitchParty {
            party_name: "default".to_string(),
        });
        host.push(GenshinCommand::ChooseTalkOption {
            option: "Katheryne".to_string(),
            skip_times: 2,
            is_orange: true,
        });
        host.push(GenshinCommand::SetTime {
            hour: 8,
            minute: 30,
            skip: true,
        });

        let plans = host.task_invocation_plans().unwrap();

        assert_eq!(plans.len(), 4);
        assert!(plans
            .iter()
            .all(|plan| plan.kind == bgi_task::TaskInvocationKind::RunCommonJob));
        assert_eq!(plans[0].task_key.as_deref(), Some("Teleport"));
        assert_eq!(plans[0].config.as_ref().unwrap()["force"], true);
        assert_eq!(plans[1].task_key.as_deref(), Some("SwitchParty"));
        assert_eq!(
            plans[1].config.as_ref().unwrap()["partyName"],
            serde_json::json!("default")
        );
        assert_eq!(plans[2].task_key.as_deref(), Some("ChooseTalkOption"));
        assert_eq!(
            plans[2].config.as_ref().unwrap()["skipTimes"],
            serde_json::json!(2)
        );
        assert_eq!(plans[3].task_key.as_deref(), Some("SetTime"));
    }

    #[test]
    fn script_host_runtime_routes_genshin_calls() {
        let mut runtime = ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(".", ".")).unwrap();

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Genshin,
                "Tp",
                vec![
                    serde_json::json!("100.5"),
                    serde_json::json!(200.25),
                    serde_json::json!(true),
                ],
            ))
            .unwrap();
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Genshin,
                "GetPositionFromMapWithMatchingMethod",
                vec![serde_json::json!("featureMatch")],
            ))
            .unwrap();
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Genshin,
                "SwitchParty",
                vec![serde_json::json!("daily")],
            ))
            .unwrap();
        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Genshin,
                "SetTime",
                vec![
                    serde_json::json!(8),
                    serde_json::json!("30"),
                    serde_json::json!(true),
                ],
            ))
            .unwrap();

        assert_eq!(runtime.genshin_commands().len(), 4);
        assert!(matches!(
            runtime.genshin_commands()[0],
            GenshinCommand::Teleport { force: true, .. }
        ));
        assert!(matches!(
            runtime.genshin_commands()[1],
            GenshinCommand::GetPositionFromMap {
                ref matching_method,
                ..
            } if matching_method.as_deref() == Some("featureMatch")
        ));

        let plans = runtime.genshin_task_invocation_plans().unwrap();
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].task_key.as_deref(), Some("Teleport"));
        assert_eq!(plans[1].task_key.as_deref(), Some("SwitchParty"));
        assert_eq!(plans[2].task_key.as_deref(), Some("SetTime"));
    }

    #[test]
    fn script_host_runtime_routes_file_and_strategy_file_calls() {
        let script_root = test_root("bgi-script-host-runtime-file");
        let strategy_root = test_root("bgi-script-host-runtime-strategy");
        fs::create_dir_all(strategy_root.join("teams")).unwrap();
        fs::write(strategy_root.join("teams/default.json"), "{}").unwrap();

        let mut runtime =
            ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&script_root, &strategy_root))
                .unwrap();

        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "CreateDirectory",
                    vec![serde_json::json!("nested")],
                ))
                .unwrap(),
            ScriptHostCallResult::Bool(true)
        );
        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "WriteTextSync",
                    vec![
                        serde_json::json!("nested/a.txt"),
                        serde_json::json!("hello"),
                    ],
                ))
                .unwrap(),
            ScriptHostCallResult::Bool(true)
        );
        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "ReadTextSync",
                    vec![serde_json::json!("nested/a.txt")],
                ))
                .unwrap(),
            ScriptHostCallResult::String("hello".to_string())
        );
        let source_image = BgrImage::new(
            VisionSize::new(2, 2),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        )
        .unwrap();
        source_image
            .write_png(script_root.join("nested/source.png"))
            .unwrap();
        let image_read = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "ReadImageMatWithResizeSync",
                vec![
                    serde_json::json!("nested/source.png"),
                    serde_json::json!("1"),
                    serde_json::json!(1.0),
                    serde_json::json!(1),
                ],
            ))
            .unwrap();
        let ScriptHostCallResult::ImageMatReadExecution(read) = image_read else {
            panic!("expected image mat read execution");
        };
        assert_eq!(read.normalized_path, script_root.join("nested/source.png"));
        assert_eq!(read.width, 1);
        assert_eq!(read.height, 1);
        assert_eq!(read.pixel_format, "BGR24");
        assert_eq!(read.pixels, vec![1, 2, 3]);
        let image_write = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::File,
                "WriteImageSync",
                vec![
                    serde_json::json!("nested/output"),
                    serde_json::json!({
                        "width": source_image.size.width,
                        "height": source_image.size.height,
                        "pixelFormat": "BGR24",
                        "pixels": source_image.pixels,
                    }),
                ],
            ))
            .unwrap();
        let ScriptHostCallResult::ImageMatWriteExecution(write) = image_write else {
            panic!("expected image mat write execution");
        };
        assert_eq!(write.normalized_path, script_root.join("nested/output.png"));
        assert_eq!(write.width, 2);
        assert_eq!(write.height, 2);
        assert!(write.bytes_written > 0);
        assert!(script_root.join("nested/output.png").is_file());
        let template_match = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Vision,
                "FindTemplate",
                vec![
                    serde_json::json!({
                        "width": 3,
                        "height": 3,
                        "pixelFormat": "BGR24",
                        "pixels": [
                            1, 1, 1,  2, 2, 2,  3, 3, 3,
                            4, 4, 4,  40, 40, 40,  50, 50, 50,
                            6, 6, 6,  70, 70, 70,  80, 80, 80
                        ]
                    }),
                    serde_json::json!({
                        "width": 2,
                        "height": 2,
                        "pixelFormat": "BGR24",
                        "pixels": [
                            40, 40, 40,  50, 50, 50,
                            70, 70, 70,  80, 80, 80
                        ]
                    }),
                    serde_json::json!({
                        "threshold": 0.99,
                        "use3Channels": true,
                        "mode": "CCorrNormed",
                        "maxMatchCount": 1,
                        "name": "patch"
                    }),
                ],
            ))
            .unwrap();
        let ScriptHostCallResult::VisionRecognitionExecution(template_match) = template_match
        else {
            panic!("expected vision recognition execution");
        };
        assert_eq!(
            template_match.recognition_type,
            RecognitionType::TemplateMatch
        );
        assert_eq!(template_match.first.rect, Rect::new(1, 1, 2, 2).unwrap());
        assert_eq!(template_match.first.text, "patch");
        assert_eq!(template_match.matched_count, 1);

        let color_match = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Vision,
                "FindColor",
                vec![
                    serde_json::json!({
                        "width": 3,
                        "height": 2,
                        "pixelFormat": "BGR24",
                        "pixels": [
                            1, 2, 3,  40, 40, 40,  5, 6, 7,
                            8, 9, 10,  40, 40, 40,  11, 12, 13
                        ]
                    }),
                    serde_json::json!({
                        "conversion": "BgrToRgb",
                        "lowerColor": [40, 40, 40],
                        "upperColor": [40, 40, 40],
                        "matchCount": 2,
                        "name": "gray-column"
                    }),
                ],
            ))
            .unwrap();
        let ScriptHostCallResult::VisionRecognitionExecution(color_match) = color_match else {
            panic!("expected color recognition execution");
        };
        assert_eq!(color_match.recognition_type, RecognitionType::ColorMatch);
        assert_eq!(color_match.first.rect, Rect::new(1, 0, 1, 2).unwrap());
        assert_eq!(color_match.first.text, "gray-column");
        assert_eq!(color_match.first.score, Some(2.0));
        let cropped = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Vision,
                "Crop",
                vec![
                    serde_json::json!({
                        "width": 3,
                        "height": 2,
                        "pixelFormat": "BGR24",
                        "pixels": [
                            1, 2, 3,  40, 40, 40,  5, 6, 7,
                            8, 9, 10,  50, 50, 50,  11, 12, 13
                        ]
                    }),
                    serde_json::json!({"x": 1, "y": 0, "width": 1, "height": 2}),
                ],
            ))
            .unwrap();
        let ScriptHostCallResult::VisionImageMatExecution(cropped) = cropped else {
            panic!("expected vision image mat execution");
        };
        assert_eq!(cropped.width, 1);
        assert_eq!(cropped.height, 2);
        assert_eq!(cropped.pixel_format, "BGR24");
        assert_eq!(cropped.pixels, vec![40, 40, 40, 50, 50, 50]);
        assert!(matches!(
            cropped.image_region.source,
            bgi_vision::ImageRegionSource::DerivedCrop
        ));

        let scaled = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Vision,
                "To1080p",
                vec![serde_json::json!({
                    "width": 1921,
                    "height": 1081,
                    "pixelFormat": "BGR24",
                    "pixels": vec![7; 1921 * 1081 * 3]
                })],
            ))
            .unwrap();
        let ScriptHostCallResult::VisionImageMatExecution(scaled) = scaled else {
            panic!("expected vision image mat execution");
        };
        assert_eq!(scaled.width, 1920);
        assert_eq!(scaled.pixel_format, "BGR24");
        assert!(matches!(
            scaled.image_region.source,
            bgi_vision::ImageRegionSource::DerivedScale
        ));
        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::StrategyFile,
                    "ReadPathSync",
                    vec![serde_json::json!("teams")],
                ))
                .unwrap(),
            ScriptHostCallResult::StringList(vec!["teams/default.json".to_string()])
        );
        assert!(matches!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "IsExists",
                    vec![serde_json::json!("../outside.txt")],
                ))
                .unwrap_err(),
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
        ));

        fs::remove_dir_all(script_root).unwrap();
        fs::remove_dir_all(strategy_root).unwrap();
    }

    #[test]
    fn script_host_runtime_routes_pathing_script_calls() {
        let script_root = test_root("bgi-script-host-runtime-pathing-script");
        let strategy_root = test_root("bgi-script-host-runtime-pathing-strategy");
        let user_auto_pathing_root = test_root("bgi-script-host-runtime-user-pathing");
        let route_json = r#"{
          "info": {
            "name": "sample route",
            "type": "collect",
            "map_name": "Teyvat"
          },
          "positions": [
            { "x": 100.0, "y": 200.0, "type": "path", "move_mode": "dash", "action": "pick_around" }
          ]
        }"#;
        fs::write(script_root.join("route.json"), route_json).unwrap();
        fs::write(user_auto_pathing_root.join("user-route.json"), route_json).unwrap();

        let mut config = ScriptHostRuntimeConfig::new(&script_root, &strategy_root);
        config.user_auto_pathing_root = user_auto_pathing_root.clone();
        config.pathing_party_config = Some(serde_json::json!({ "partyName": "daily" }));
        let mut runtime = ScriptHostRuntime::new(config).unwrap();

        let inline = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "Run",
                vec![serde_json::json!(route_json)],
            ))
            .unwrap();
        let ScriptHostCallResult::PathingExecution(inline_execution) = inline else {
            panic!("expected inline pathing execution");
        };
        let inline_plan = &inline_execution.plan;
        assert!(!inline_execution.dispatched);
        assert!(!inline_execution.completed);
        assert_eq!(inline_execution.execution_plan.segment_count, 1);
        assert_eq!(inline_execution.execution_plan.waypoint_count, 1);
        assert_eq!(inline_plan.source, PathingScriptSource::InlineJson);
        assert_eq!(inline_plan.summary.waypoint_count, 1);
        assert_eq!(inline_plan.summary.actions, vec!["pick_around".to_string()]);
        assert_eq!(
            inline_plan.party_config,
            Some(serde_json::json!({ "partyName": "daily" }))
        );

        let from_script = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "RunFile",
                vec![serde_json::json!("route.json")],
            ))
            .unwrap();
        let ScriptHostCallResult::PathingExecution(from_script) = from_script else {
            panic!("expected script file pathing execution");
        };
        assert_eq!(from_script.plan.source, PathingScriptSource::ScriptFile);
        assert_eq!(
            from_script.plan.task.file_name.as_deref(),
            Some("route.json")
        );

        let from_user = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "RunFileFromUser",
                vec![serde_json::json!("user-route.json")],
            ))
            .unwrap();
        let ScriptHostCallResult::PathingExecution(from_user) = from_user else {
            panic!("expected user pathing execution");
        };
        assert_eq!(
            from_user.plan.source,
            PathingScriptSource::UserAutoPathingFile
        );
        assert_eq!(
            from_user.plan.task.file_name.as_deref(),
            Some("user-route.json")
        );
        assert!(!from_user.dispatched);
        assert!(!from_user.completed);
        assert_eq!(from_user.execution_plan.segment_count, 1);
        assert_eq!(from_user.execution_plan.waypoint_count, 1);
        assert_eq!(
            from_user.execution_plan.segments[0].seed_previous_position,
            Some(bgi_core::PathingPoint { x: 100.0, y: 200.0 })
        );

        let plan = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::PathingScript,
                "PlanFileFromUser",
                vec![serde_json::json!("user-route.json")],
            ))
            .unwrap();
        let ScriptHostCallResult::PathingPlan(plan) = plan else {
            panic!("expected user pathing plan");
        };
        assert_eq!(plan.source, PathingScriptSource::UserAutoPathingFile);
        assert_eq!(plan.task.file_name.as_deref(), Some("user-route.json"));

        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::PathingScript,
                    "IsFile",
                    vec![serde_json::json!("user-route.json")],
                ))
                .unwrap(),
            ScriptHostCallResult::Bool(true)
        );
        assert_eq!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::PathingScript,
                    "ReadPathSync",
                    Vec::new(),
                ))
                .unwrap(),
            ScriptHostCallResult::StringList(vec!["user-route.json".to_string()])
        );
        assert!(matches!(
            runtime
                .call(ScriptHostCall::new(
                    ScriptHostTarget::PathingScript,
                    "RunFileFromUser",
                    vec![serde_json::json!("../outside.json")],
                ))
                .unwrap_err(),
            ScriptHostRuntimeError::Policy(ScriptHostPolicyError::PathTraversal { .. })
        ));

        fs::remove_dir_all(script_root).unwrap();
        fs::remove_dir_all(strategy_root).unwrap();
        fs::remove_dir_all(user_auto_pathing_root).unwrap();
    }

    #[test]
    fn script_host_runtime_routes_key_mouse_log_and_notification_calls() {
        let root = test_root("bgi-script-host-runtime-keymouse");
        let mut runtime =
            ScriptHostRuntime::new(ScriptHostRuntimeConfig::new(&root, &root)).unwrap();

        let plan = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseScript,
                "Run",
                vec![serde_json::json!(
                    r#"{
                      "macroEvents": [
                        { "type": 0, "keyCode": 87, "time": 0 },
                        { "type": 1, "keyCode": 87, "time": 50 }
                      ]
                    }"#
                )],
            ))
            .unwrap();
        let ScriptHostCallResult::KeyMouseExecution(execution) = plan else {
            panic!("expected key/mouse execution");
        };
        assert_eq!(execution.mode, KeyMouseScriptDispatchMode::PlanOnly);
        assert!(!execution.dispatched);
        assert_eq!(execution.plan.summary.event_count, 2);
        assert_eq!(execution.plan.input_events.len(), 3);

        let plan = runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::KeyMouseScript,
                "Plan",
                vec![serde_json::json!(
                    r#"{
                      "macroEvents": [
                        { "type": 0, "keyCode": 87, "time": 0 },
                        { "type": 1, "keyCode": 87, "time": 50 }
                      ]
                    }"#
                )],
            ))
            .unwrap();
        assert!(matches!(
            plan,
            ScriptHostCallResult::KeyMousePlan(KeyMouseScriptRunPlan { .. })
        ));

        runtime
            .call(ScriptHostCall::new(
                ScriptHostTarget::Log,
                "Info",
                vec![serde_json::json!("ready")],
            ))
            .unwrap();
        let notification = runtime
            .call_at(
                ScriptHostCall::new(
                    ScriptHostTarget::Notification,
                    "Send",
                    vec![serde_json::json!("ready")],
                ),
                10,
            )
            .unwrap();
        let ScriptHostCallResult::NotificationExecution(notification) = notification else {
            panic!("expected notification execution");
        };
        assert_eq!(notification.mode, NotificationDispatchMode::RecordOnly);
        assert!(!notification.dispatched);
        assert_eq!(notification.record.message, "ready");
        assert!(notification.delivery.is_none());

        assert_eq!(runtime.log_records()[0].level, ScriptLogLevel::Info);
        assert_eq!(runtime.notification_records()[0].timestamp_ms, 10);

        let mut config = ScriptHostRuntimeConfig::new(&root, &root);
        config.notification_dispatch_mode = NotificationDispatchMode::Sink;
        let mut runtime = ScriptHostRuntime::new(config).unwrap();
        let mut sink = RecordingNotificationSink::default();
        let notification = runtime
            .call_notification_with_sink(
                ScriptHostCall::new(
                    ScriptHostTarget::Notification,
                    "Error",
                    vec![serde_json::json!("failed")],
                ),
                11,
                &mut sink,
            )
            .unwrap();
        let ScriptHostCallResult::NotificationExecution(notification) = notification else {
            panic!("expected notification execution");
        };
        assert_eq!(notification.mode, NotificationDispatchMode::Sink);
        assert!(notification.dispatched);
        assert_eq!(
            notification.delivery.as_ref().unwrap().event_code,
            "js.error"
        );
        assert_eq!(notification.delivery.as_ref().unwrap().result, "fail");
        assert_eq!(sink.deliveries().len(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    fn test_root(prefix: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
