use super::dispatcher::{DispatcherCommand, GenshinCommand, GenshinHost, ScriptDispatcherHost};
use super::file::{
    ImageMatReadExecution, ImageMatReadPlan, ImageMatWriteExecution, ImageMatWritePlan,
    LimitedFileHost, StrategyFileHost,
};
use super::global_input::{
    AvatarRecognitionPlan, CaptureGameRegionExecution, CaptureGameRegionPlan, GameCaptureArea,
    GameCaptureFrameSource, GameMetrics, GlobalInputDispatchMode, GlobalInputExecution,
    GlobalInputHost,
};
use super::html_mask::{HtmlMaskCommand, HtmlMaskHost, HtmlMaskInitialState, HtmlMaskSnapshot};
use super::http::{HttpDispatchMode, HttpExecution, HttpHost, HttpRequestPlan};
use super::key_mouse_hook::{
    KeyMouseHookCommand, KeyMouseHookDispatch, KeyMouseHookHost, KeyMouseHookSnapshot,
};
use super::key_mouse_script::{
    KeyMouseScriptDispatchMode, KeyMouseScriptExecution, KeyMouseScriptHost, KeyMouseScriptRunPlan,
};
use super::notifications::{
    NotificationDispatchMode, NotificationExecution, ScriptLogHost, ScriptLogRecord,
    ScriptNotificationHost, ScriptNotificationRecord,
};
use super::pathing::{PathingScriptExecution, PathingScriptHost, PathingScriptRunPlan};
use super::server_time::ServerTimeHost;
use super::vision::{VisionHost, VisionImageMatExecution, VisionRecognitionExecution};
use crate::policy::{ScriptHostPolicyError, ScriptHttpPolicy, ScriptNotificationPolicy};
use crate::r#macro::{KeyMouseMacroError, MacroPlaybackContext};
use bgi_core::BgiError;
use bgi_input::{InputCancellationToken, InputEvent, PostMessageEvent};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
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

#[allow(clippy::large_enum_variant)]
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
    pub(super) global_input: GlobalInputHost,
    pub(super) global_input_dispatch_mode: GlobalInputDispatchMode,
    pub(super) input_window_handle: Option<isize>,
    pub(super) genshin: GenshinHost,
    pub(super) pathing_script: PathingScriptHost,
    pub(super) key_mouse_script: KeyMouseScriptHost,
    pub(super) key_mouse_dispatch_mode: KeyMouseScriptDispatchMode,
    pub(super) cancellation: Option<Arc<InputCancellationToken>>,
    pub(super) file: LimitedFileHost,
    pub(super) vision: VisionHost,
    pub(super) log: ScriptLogHost,
    pub(super) http: HttpHost,
    pub(super) http_dispatch_mode: HttpDispatchMode,
    pub(super) dispatcher: ScriptDispatcherHost,
    pub(super) notification: ScriptNotificationHost,
    pub(super) notification_dispatch_mode: NotificationDispatchMode,
    pub(super) strategy_file: StrategyFileHost,
    pub(super) server_time: ServerTimeHost,
    pub(super) html_mask: HtmlMaskHost,
    pub(super) key_mouse_hook: KeyMouseHookHost,
    pub(super) logical_time_ms: u64,
}
