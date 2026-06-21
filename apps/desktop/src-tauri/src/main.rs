use bgi_capture::{
    capture_mode_infos, find_game_window, CaptureBackend, CaptureFrame,
    CaptureMode as NativeCaptureMode, CaptureSettings, GameCapture, GameWindowMatch,
    GameWindowSearchConfig, PixelFormat, WindowHandle,
};
use bgi_core::{
    config_path, default_navigation, execute_notification_dispatch_plan,
    execute_notification_dispatch_with_transports, initial_triggers, latest_version_from_notice,
    migration_capabilities, mirror_chyan_latest_outcome, notification_dispatch_plan_for_provider,
    notification_provider_plans, overlay_metric_item_from_key, parse_redeem_code_feed_items,
    read_config, redeem_code_feed_update_decision, redeem_code_live_act_id_from_bbs_response,
    redeem_code_live_codes_from_response, redeem_code_live_index_from_response, ui_shell_decision,
    update_decision, update_download_page_url, update_request_plan, updater_launch_options,
    updater_launch_plan, write_config, AppConfig, GenshinAction, MaskWindowConfig, MaskWindowState,
    MirrorChyanLatestOutcome, MirrorChyanLatestResponse, NavigationItem, Notice,
    NotificationDispatchError, NotificationDispatchExecution, NotificationEmailClient,
    NotificationEmailRequest, NotificationEmailSecurity, NotificationEventResult,
    NotificationHttpClient, NotificationHttpRequest, NotificationHttpResponse, NotificationImage,
    NotificationPayload, NotificationProviderKind, NotificationWebSocketClient,
    NotificationWindowsToastClient, NotificationWindowsToastRequest, RedeemCodeFeedItem,
    RedeemCodeFeedUpdateDecision, RedeemCodeLiveData, UiShellDecision, UpdateChannel,
    UpdateDecision, UpdateDecisionAction, UpdateOption, UpdateRequestPlan, UpdateTrigger,
    UpdaterLaunchPlan, ALPHA_RELEASES_URL, DOWNLOAD_PAGE_URL, REDEEM_CODE_BBS_ACT_ID_1_URL,
    REDEEM_CODE_BBS_ACT_ID_2_URL, REDEEM_CODE_CODES_URL, REDEEM_CODE_LIVE_INDEX_URL,
    REDEEM_CODE_LIVE_REFRESH_CODE_URL, REDEEM_CODE_UPDATE_TIME_URL,
};
use bgi_hotkey::Hotkey;
use bgi_input::{
    post_message_events_for_action, release_pressed_keys_sequence, InputSequence, KeyActionType,
    MouseButton, PostMessageMode,
};
use bgi_script::{
    add_key_mouse_script_project, add_pathing_script_project, add_script_group_project,
    add_shell_script_project, available_js_script_projects, available_key_mouse_scripts,
    available_pathing_scripts, available_pathing_tree, clear_repo_bridge_update,
    create_script_group, delete_script_group, execute_file_repo_import, execute_git_repo_update,
    execute_repo_import_with_git, execute_zip_repo_import, git_update_plan, host_bindings,
    mark_repo_bridge_path_updated, move_script_group_project, parse_import_uri,
    read_repo_bridge_file, read_repo_bridge_repo_json, read_script_group_file, read_script_groups,
    read_script_settings_document, read_subscription_file, remove_script_group_project,
    rename_script_group, repo_bridge_index_nodes, repo_bridge_subscribed_paths_json,
    save_script_group_project_settings, script_group_file_path, script_host_security_summary,
    script_import_plan, script_repo_bridge_paths, script_runtime_summary,
    update_script_group_project, zip_import_plan, GameCaptureFrameSource, HtmlMaskCommand,
    HtmlMaskInitialState, HtmlMaskMessage, HtmlMaskWindowPlan, InputCancellationToken,
    KeyMouseScript, NotificationDispatchMode, ScriptGroup, ScriptGroupFile, ScriptGroupProject,
    ScriptGroupProjectPatch, ScriptGroupResumePointer, ScriptHostRuntimeConfig,
    ScriptHostRuntimeError, ScriptHostTarget, ScriptProjectStatus, ScriptProjectType,
    ScriptRepoBridgeIndexNode, SystemGitRunner,
};
use bgi_script_engine::{
    JavaScriptExecutionOutcome, ScriptGroupExecutionOutcome, ScriptGroupExecutionRoots,
};
use bgi_task::{
    detect_active_combat_avatar_index_from_default_rects_with_arrow,
    execute_auto_fight_finish_detection_live_probe, execute_independent_task_with_cancel,
    execute_team_context_combat_script_inputs, extract_redeem_codes_from_text, independent_tasks,
    redeem_code_entries_from_strings, runtime_triggers, select_triggers_for_tick, task_catalog,
    AutoFightExecutionConfig, AutoFightExecutionPlan, AutoFightFinishDetectionExecutionMode,
    AutoFightFinishDetectionLiveExecution, AutoFightParam, AutoPathingExecutionPlan,
    CombatActiveAvatarDetectionResult, CombatCommandPlaybackMode, CombatTeamPlaybackExecution,
    DispatcherRuntime, IndependentTaskExecution, IndependentTaskExecutionRequest, RedeemCodeEntry,
    RunnerRuntime, ShellConfig, ShellExecutionResult, UseRedeemCodeExecutionPlan,
};
use bgi_vision::{
    recognition_type_infos, registered_onnx_models, BgrImage, OnnxModelLoadPlan,
    OnnxProviderSelection, Size as VisionSize,
};
use chrono::{Local, NaiveDate};
use image::ImageEncoder;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton as TrayMouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{
    Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

const DESKTOP_LOG_RETAINED_FILE_COUNT: usize = 31;
const DESKTOP_LOG_RETAINED_DAYS: i64 = 21;
const REDEEM_CLIPBOARD_MAX_TEXT_LEN: usize = 1_000;
const REDEEM_CLIPBOARD_POLL_INTERVAL: Duration = Duration::from_secs(2);
const REDEEM_CLIPBOARD_EVENT: &str = "redeem-code://clipboard-detected";
const SCRIPT_IMPORT_CLIPBOARD_EVENT: &str = "script-repo://clipboard-import-detected";
const BACKGROUND_UPDATE_EVENT: &str = "desktop-update://background-check";
const HTML_MASK_WINDOW_PREFIX: &str = "html-mask-";
const HTML_MASK_POSITION_SYNC_INTERVAL: Duration = Duration::from_millis(250);
const HTML_MASK_BRIDGE_SCRIPT: &str = r#"
(function () {
  if (window.htmlMask && window.htmlMask.__betterGiRustBridge) return;

  function invoke(command, args) {
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function") {
      return window.__TAURI_INTERNALS__.invoke(command, args || {});
    }
    return Promise.reject(new Error("Tauri invoke bridge is unavailable"));
  }

  const chromeMessageListeners = [];
  let dispatchingBridgeMessage = false;

  function postChromeWebMessage(payload) {
    let msg = payload;
    if (typeof payload === "string") {
      try {
        msg = JSON.parse(payload);
      } catch (_) {
        msg = { url: "", data: payload, requestId: null };
      }
    }
    if (!msg || typeof msg !== "object") {
      msg = { url: "", data: msg, requestId: null };
    }
    return bridge.send(msg.url || "", msg.data === undefined ? null : msg.data, msg.requestId || null);
  }

  function dispatchChromeWebMessage(raw) {
    const event = { data: raw };
    chromeMessageListeners.slice().forEach(function (listener) {
      try {
        listener(event);
      } catch (_) {}
    });
  }

  const bridge = {
    __betterGiRustBridge: true,
    _callbacks: {},
    _seq: 0,
    request: function (url, data) {
      const id = "__req_" + (++bridge._seq);
      return new Promise(function (resolve, reject) {
        bridge._callbacks[id] = { resolve: resolve, reject: reject };
        invoke("html_mask_receive_from_webview", {
          payload: {
            url: url || "",
            data: data === undefined ? null : data,
            requestId: id,
          },
        }).catch(function (error) {
          delete bridge._callbacks[id];
          reject(error);
        });
      });
    },
    send: function (url, data, requestId) {
      return invoke("html_mask_receive_from_webview", {
        payload: {
          url: url || "",
          data: data === undefined ? null : data,
          requestId: requestId || null,
        },
      });
    },
    onMessage: null,
    _dispatch: function (raw) {
      if (dispatchingBridgeMessage) return;
      dispatchingBridgeMessage = true;
      try {
        let msg = raw;
        try {
          if (typeof raw === "string") msg = JSON.parse(raw);
        } catch (_) {}

        if (msg && msg.requestId && bridge._callbacks[msg.requestId]) {
          bridge._callbacks[msg.requestId].resolve(msg);
          delete bridge._callbacks[msg.requestId];
        } else if (typeof bridge.onMessage === "function") {
          const result = bridge.onMessage(msg);
          if (msg && msg.requestId && result !== undefined) {
            Promise.resolve(result).then(function (data) {
              return bridge.send("/__response__", data, msg.requestId);
            });
          }
        }

        window.dispatchEvent(new CustomEvent("bettergi:html-mask-message", { detail: msg }));
        if (window.BetterGIHtmlMask && typeof window.BetterGIHtmlMask.receive === "function") {
          window.BetterGIHtmlMask.receive(msg);
        }
        if (typeof window.onBetterGIHtmlMaskMessage === "function") {
          window.onBetterGIHtmlMaskMessage(msg);
        }
        dispatchChromeWebMessage(raw);
      } finally {
        dispatchingBridgeMessage = false;
      }
    },
  };

  window.htmlMask = bridge;
  window.chrome = window.chrome || {};
  window.chrome.webview = window.chrome.webview || {};
  window.chrome.webview.postMessage = postChromeWebMessage;
  window.chrome.webview.addEventListener = function (name, listener) {
    if (name === "message" && typeof listener === "function") chromeMessageListeners.push(listener);
  };
  window.chrome.webview.removeEventListener = function (name, listener) {
    if (name !== "message") return;
    const index = chromeMessageListeners.indexOf(listener);
    if (index >= 0) chromeMessageListeners.splice(index, 1);
  };
  window.BetterGIHtmlMask = window.BetterGIHtmlMask || {
    send: bridge.send,
    request: bridge.request,
    receive: null,
  };
})();
"#;

#[derive(Default)]
struct RedeemCodeClipboardState {
    ignored_hashes: Mutex<BTreeSet<String>>,
    last_hash: Mutex<Option<String>>,
}

#[derive(Debug, Serialize)]
struct ConfigSummary {
    capture_mode: String,
    trigger_interval: u64,
    auto_pick_enabled: bool,
    auto_skip_enabled: bool,
    bgi_enabled_hotkey: String,
    modeled_config_sections: usize,
    strongly_typed_config_sections: usize,
    compatibility_config_sections: usize,
    unknown_top_level_fields: usize,
}

impl From<AppConfig> for ConfigSummary {
    fn from(config: AppConfig) -> Self {
        let coverage = config.coverage();
        Self {
            capture_mode: format!("{:?}", config.capture_mode),
            trigger_interval: config.trigger_interval,
            auto_pick_enabled: config.auto_pick_config.enabled,
            auto_skip_enabled: config.auto_skip_config.enabled,
            bgi_enabled_hotkey: config.hot_key_config.bgi_enabled_hotkey,
            modeled_config_sections: coverage.modeled_config_sections,
            strongly_typed_config_sections: coverage.strongly_typed_sections,
            compatibility_config_sections: coverage.compatibility_sections,
            unknown_top_level_fields: coverage.preserved_unknown_top_level_fields,
        }
    }
}

#[derive(Debug, Serialize)]
struct DashboardState {
    navigation: Vec<NavigationItem>,
    capabilities: Vec<bgi_core::Capability>,
    triggers: Vec<bgi_core::TriggerDescriptor>,
    config: ConfigSummary,
    ui_shell: UiShellDecision,
    native_backend: NativeBackendSummary,
    task_runtime: TaskRuntimeSummary,
    script_runtime: ScriptRuntimePanel,
}

#[derive(Debug, Serialize)]
struct NativeBackendSummary {
    input_events_in_demo_sequence: usize,
    post_message_events_in_demo_sequence: usize,
    sample_hotkey: String,
    capture_modes: Vec<bgi_capture::CaptureModeInfo>,
    recognition_types: Vec<bgi_vision::RecognitionTypeInfo>,
    registered_onnx_models: usize,
    avatar_side_model: Option<OnnxModelLoadPlan>,
    windows_only_backends: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct TaskRuntimeSummary {
    dispatcher: DispatcherRuntime,
    runner: RunnerRuntime,
    enabled_triggers: usize,
    selected_triggers: Vec<bgi_task::RunnableTrigger>,
    selection_reason: bgi_task::TaskSelectionReason,
    independent_tasks: Vec<bgi_task::IndependentTaskDescriptor>,
    catalog_entries: usize,
    config_bound_catalog_entries: usize,
    native_pending_catalog_entries: usize,
}

#[derive(Debug, Serialize)]
struct ScriptRuntimePanel {
    summary: bgi_script::ScriptRuntimeSummary,
    hosts: Vec<bgi_script::HostBindingDescriptor>,
    security: bgi_script::ScriptHostSecuritySummary,
    sample_macro: bgi_script::KeyMouseMacroSummary,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationTestPayload {
    message: Option<String>,
    provider: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct DesktopNotificationServiceState {
    initialized: bool,
    config_path: String,
    enabled_providers: Vec<String>,
    provider_count: usize,
    refreshed_at_ms: Option<u64>,
}

#[derive(Default)]
struct NotificationServiceState {
    state: Mutex<DesktopNotificationServiceState>,
}

struct DesktopTaskRuntimeState {
    dispatcher: Mutex<DispatcherRuntime>,
    runner: Mutex<RunnerRuntime>,
    script_cancellation: Arc<InputCancellationToken>,
}

impl Default for DesktopTaskRuntimeState {
    fn default() -> Self {
        Self {
            dispatcher: Mutex::new(DispatcherRuntime::default()),
            runner: Mutex::new(RunnerRuntime::default()),
            script_cancellation: Arc::new(InputCancellationToken::new()),
        }
    }
}

#[derive(Debug, Serialize)]
struct ScriptStopResult {
    requested: bool,
    runner_state: RunnerRuntime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckPayload {
    channel: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateLaunchPayload {
    source: Option<String>,
    exit_after_launch: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverlayPatchPayload {
    mask_enabled: Option<bool>,
    show_log_box: Option<bool>,
    show_status: Option<bool>,
    display_recognition_results_on_mask: Option<bool>,
    show_overlay_metrics: Option<bool>,
    overlay_layout_edit_enabled: Option<bool>,
    metric_key: Option<String>,
    metric_enabled: Option<bool>,
    reset_metrics_layout: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHtmlMaskDispatch {
    action: String,
    window_id: Option<String>,
    window_label: Option<String>,
    dispatched: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHtmlMaskBridgeMessage {
    window_id: String,
    message: HtmlMaskMessage,
    timestamp_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHtmlMaskBridgeSnapshot {
    from_html: BTreeMap<String, Vec<DesktopHtmlMaskBridgeMessage>>,
    pending_requests: BTreeMap<String, String>,
    window_labels: BTreeMap<String, String>,
    windows: BTreeMap<String, HtmlMaskWindowPlan>,
}

#[derive(Default)]
struct HtmlMaskBridgeState {
    from_html: Mutex<BTreeMap<String, Vec<DesktopHtmlMaskBridgeMessage>>>,
    pending_requests: Mutex<BTreeMap<String, String>>,
    window_labels: Mutex<BTreeMap<String, String>>,
    windows: Mutex<BTreeMap<String, HtmlMaskWindowPlan>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HtmlMaskWebviewPayload {
    url: Option<String>,
    data: Option<serde_json::Value>,
    request_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct DesktopUpdateCheckResult {
    trigger: UpdateTrigger,
    app_version: String,
    channel: UpdateChannel,
    request: UpdateRequestPlan,
    latest_version: Option<String>,
    decision: UpdateDecision,
    mirror_outcome: Option<MirrorChyanLatestOutcome>,
    release_notes: Option<DesktopReleaseNotes>,
    download_page_url: String,
    updater_path: String,
    updater_exists: bool,
    updater_options: Vec<UpdaterLaunchPlan>,
    ignored_version: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct DesktopBackgroundUpdateState {
    running: bool,
    last_result: Option<DesktopUpdateCheckResult>,
    last_error: Option<String>,
}

#[derive(Default)]
struct BackgroundUpdateState {
    state: Mutex<DesktopBackgroundUpdateState>,
}

#[derive(Clone, Debug, Serialize)]
struct DesktopReleaseNotes {
    name: Option<String>,
    body: Option<String>,
    html_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct DesktopUpdateActionResult {
    ok: bool,
    action: String,
    detail: String,
    exit_scheduled: bool,
}

#[derive(Debug, Serialize)]
struct DesktopRedeemCodeFeedResult {
    request_url: String,
    local_version: String,
    remote_text: Option<String>,
    decision: RedeemCodeFeedUpdateDecision,
}

#[derive(Debug, Serialize)]
struct DesktopRedeemCodeFeedItemsResult {
    request_url: String,
    items: Vec<RedeemCodeFeedItem>,
    raw_bytes: usize,
}

#[derive(Debug, Serialize)]
struct DesktopRedeemCodeLiveResult {
    act_id_sources: Vec<String>,
    index_url: String,
    refresh_code_url: String,
    data: RedeemCodeLiveData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedeemCodePlanPayload {
    text: Option<String>,
    codes: Option<Vec<String>>,
    feed_items: Option<Vec<RedeemCodeFeedItem>>,
    live_codes: Option<Vec<bgi_core::RedeemCodeLiveCode>>,
}

#[derive(Debug, Serialize)]
struct DesktopRedeemCodePlanResult {
    extracted_codes: Vec<String>,
    plan: UseRedeemCodeExecutionPlan,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedeemCodeClipboardCheckPayload {
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct DesktopRedeemCodeClipboardState {
    clipboard_listener_enabled: bool,
    ignored_hash_count: usize,
}

#[derive(Clone, Debug, Serialize)]
struct DesktopRedeemCodeClipboardCheckResult {
    clipboard_listener_enabled: bool,
    ignored: bool,
    hash: String,
    text: String,
    extracted_codes: Vec<String>,
    plan: Option<UseRedeemCodeExecutionPlan>,
    source: String,
}

#[derive(Debug, Serialize)]
struct DesktopLogState {
    path: String,
    exists: bool,
    bytes: u64,
    tail: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DesktopShellState {
    exit_to_tray: bool,
    tray_enabled: bool,
    config_path: String,
    log_path: String,
}

#[derive(Debug, Serialize)]
struct DesktopShellActionResult {
    ok: bool,
    action: String,
    detail: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopShellTaskPayload {
    command: String,
    timeout_seconds: Option<i32>,
    no_window: Option<bool>,
    output: Option<bool>,
    disable: Option<bool>,
    working_directory: Option<String>,
}

#[derive(Debug, Serialize)]
struct DesktopShellTaskExecution {
    task: String,
    result: ShellExecutionResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAutoPathingTaskPayload {
    route: String,
}

#[derive(Debug, Serialize)]
struct DesktopAutoPathingTaskExecution {
    task: String,
    result: AutoPathingExecutionPlan,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAutoFightTaskPayload {
    strategy_name: Option<String>,
    team_names: Option<String>,
}

#[derive(Debug, Serialize)]
struct DesktopAutoFightTaskExecution {
    task: String,
    result: AutoFightExecutionPlan,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAutoFightTeamPlaybackPayload {
    strategy_name: Option<String>,
    team_names: Option<String>,
    send_input: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DesktopAutoFightTeamPlaybackExecution {
    task: String,
    result: CombatTeamPlaybackExecution,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAutoFightFinishProbePayload {
    strategy_name: Option<String>,
    team_names: Option<String>,
    send_input: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DesktopAutoFightFinishProbeExecution {
    task: String,
    result: AutoFightFinishDetectionLiveExecution,
}

#[derive(Debug, Serialize)]
struct DesktopActiveAvatarDetectionExecution {
    task: String,
    result: CombatActiveAvatarDetectionResult,
}

#[derive(Debug, Serialize)]
struct ScriptRepoDesktopState {
    repo_path: String,
    repo_exists: bool,
    repo_json_path: Option<String>,
    subscription_file_path: String,
    repo_json_bytes: usize,
    subscribed_paths: Vec<String>,
    index_nodes: Vec<ScriptRepoBridgeIndexNode>,
}

#[derive(Debug, Serialize)]
struct ScriptRepoDesktopImportResult {
    imported_targets: usize,
    skipped_unknown_paths: Vec<String>,
    subscriptions: Vec<String>,
    dependency_files_copied: usize,
    git_checkouts: usize,
}

#[derive(Debug, Serialize)]
struct ScriptRepoDesktopUriImportResult {
    paths: Vec<String>,
    clear_clipboard_after_import: bool,
    result: ScriptRepoDesktopImportResult,
}

#[derive(Clone, Debug, Serialize)]
struct ScriptRepoDesktopClipboardImportResult {
    uri: String,
    hash: String,
    path_json: String,
    paths: Vec<String>,
    source: String,
}

#[derive(Debug, Serialize)]
struct ScriptRepoDesktopUpdateResult {
    repo_url: String,
    branch: String,
    repo_folder_name: String,
    repo_path: String,
    repo_updated_json_path: String,
    updated: bool,
    cloned: bool,
    remote_changed: bool,
    created_new_folder: bool,
    fallback_reclone: bool,
    marker_generated: bool,
    old_repo_overlap_ratio: Option<f64>,
    current_commit: Option<String>,
    remote_commit: Option<String>,
}

#[derive(Debug, Serialize)]
struct ScriptRepoDesktopZipImportResult {
    zip_path: String,
    repo_json_path: String,
    target_folder_name: String,
    target_path: String,
    repo_updated_json_path: String,
    best_overlap_ratio: Option<f64>,
    matched_existing_folder: Option<String>,
    old_repo_overlap_ratio: Option<f64>,
    marker_generated: bool,
}

#[derive(Debug, Serialize)]
struct ScriptGroupDesktopSummary {
    name: String,
    index: i32,
    path: String,
    project_count: usize,
    projects: Vec<ScriptGroupProjectDesktopSummary>,
}

#[derive(Debug, Serialize)]
struct ScriptGroupProjectDesktopSummary {
    index: usize,
    project_index: i32,
    name: String,
    folder_name: String,
    project_type: String,
    status: String,
    schedule: String,
    run_num: i32,
    allow_js_notification: Option<bool>,
    allow_js_http_hash: Option<String>,
    has_settings: bool,
}

#[derive(Debug, Serialize)]
struct AvailableJsProjectDesktopSummary {
    folder_name: String,
    name: String,
    version: String,
    description: String,
    settings_ui: String,
    has_settings_ui: bool,
}

#[derive(Debug, Serialize)]
struct AvailableKeyMouseDesktopSummary {
    name: String,
    relative_path: String,
}

#[derive(Debug, Serialize)]
struct AvailablePathingDesktopSummary {
    name: String,
    folder_name: String,
    relative_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptGroupProjectPatchPayload {
    name: Option<String>,
    folder_name: Option<String>,
    project_type: Option<String>,
    status: Option<String>,
    schedule: Option<String>,
    run_num: Option<i32>,
    allow_js_notification: Option<bool>,
    allow_js_http_hash: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
struct ScriptSettingsDesktopDocument {
    group_name: String,
    project_index: usize,
    project_folder_name: String,
    project_path: String,
    manifest_name: String,
    manifest_version: String,
    settings_ui_path: Option<String>,
    items: Vec<bgi_script::ScriptSettingItem>,
    values: serde_json::Value,
    defaults_applied: bool,
    cleaned_invalid_values: usize,
}

#[derive(Debug, Serialize)]
struct ScriptSettingsDesktopSaveResult {
    group_path: String,
    group_name: String,
    project_index: usize,
    project_folder_name: String,
    settings: serde_json::Value,
    cleaned_invalid_values: usize,
}

#[derive(Debug)]
struct DesktopGameCaptureFrameSource {
    hwnd: WindowHandle,
    settings: CaptureSettings,
}

impl DesktopGameCaptureFrameSource {
    fn new(
        hwnd: WindowHandle,
        settings: CaptureSettings,
    ) -> Result<Self, bgi_capture::CaptureError> {
        CaptureBackend::new(settings.mode)?;
        Ok(Self { hwnd, settings })
    }
}

impl bgi_script::GameCaptureFrameSource for DesktopGameCaptureFrameSource {
    fn capture_frame(
        &self,
    ) -> std::result::Result<CaptureFrame, bgi_script::ScriptHostRuntimeError> {
        let mut capture = CaptureBackend::new(self.settings.mode)?;
        capture.start(self.hwnd, self.settings.clone())?;
        capture.capture()?.ok_or_else(|| {
            ScriptHostRuntimeError::Capture(bgi_capture::CaptureError::Win32(
                "desktop capture backend returned no frame".to_string(),
            ))
        })
    }
}

struct DesktopNotificationHttpClient {
    client: reqwest::blocking::Client,
}

impl DesktopNotificationHttpClient {
    fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
        })
    }
}

impl NotificationHttpClient for DesktopNotificationHttpClient {
    fn send(
        &mut self,
        request: &NotificationHttpRequest,
    ) -> std::result::Result<NotificationHttpResponse, NotificationDispatchError> {
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        let mut builder = self
            .client
            .request(method, &request.url)
            .body(request.body.clone());
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }
        let response = builder
            .send()
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        Ok(NotificationHttpResponse { status, body })
    }
}

struct DesktopNotificationWebSocketClient;

impl NotificationWebSocketClient for DesktopNotificationWebSocketClient {
    fn send_text(
        &mut self,
        endpoint: &str,
        text: &str,
    ) -> std::result::Result<(), NotificationDispatchError> {
        use tungstenite::{connect, Message};

        let (mut socket, _) = connect(endpoint)
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        socket
            .send(Message::Text(text.to_string().into()))
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        let _ = socket.close(None);
        Ok(())
    }
}

struct DesktopNotificationEmailClient;

impl NotificationEmailClient for DesktopNotificationEmailClient {
    fn send_email(
        &mut self,
        request: &NotificationEmailRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        use lettre::message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart};
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::transport::smtp::client::{Tls, TlsParameters};
        use lettre::{Address, Message, SmtpTransport, Transport};

        let from_address: Address = request
            .from_email
            .parse::<Address>()
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        let to_address: Address = request
            .to_email
            .parse::<Address>()
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
        let from = Mailbox::new(non_empty_string(&request.from_name), from_address);
        let to = Mailbox::new(None, to_address);
        let mut body = MultiPart::mixed().singlepart(SinglePart::html(request.html_body.clone()));
        for attachment in &request.attachments {
            let content_type = ContentType::parse(&attachment.content_type)
                .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
            let part = match attachment
                .content_id
                .as_ref()
                .and_then(|id| non_empty_string(id))
            {
                Some(content_id) => {
                    Attachment::new_inline_with_name(content_id, attachment.file_name.clone())
                        .body(attachment.bytes.clone(), content_type)
                }
                None => Attachment::new(attachment.file_name.clone())
                    .body(attachment.bytes.clone(), content_type),
            };
            body = body.singlepart(part);
        }
        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(request.subject.clone())
            .multipart(body)
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;

        let tls_parameters = || {
            TlsParameters::new(request.smtp_server.clone())
                .map_err(|error| NotificationDispatchError::Transport(error.to_string()))
        };
        let mut builder =
            SmtpTransport::builder_dangerous(request.smtp_server.clone()).port(request.smtp_port);
        builder = match request.security {
            NotificationEmailSecurity::None => builder.tls(Tls::None),
            NotificationEmailSecurity::StartTls => builder.tls(Tls::Required(tls_parameters()?)),
            NotificationEmailSecurity::SslOnConnect => builder.tls(Tls::Wrapper(tls_parameters()?)),
            NotificationEmailSecurity::Auto => builder.tls(Tls::Opportunistic(tls_parameters()?)),
        };
        if let Some(username) = non_empty_string(request.smtp_username.as_deref().unwrap_or("")) {
            builder = builder.credentials(Credentials::new(
                username,
                request.smtp_password.clone().unwrap_or_default(),
            ));
        }
        let mailer = builder.build();
        mailer.send(&message).map_err(|error| {
            NotificationDispatchError::Transport(format!("发送邮件失败: {error}"))
        })?;
        Ok(())
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

struct DesktopNotificationWindowsToastClient {
    temp_dir: PathBuf,
}

impl DesktopNotificationWindowsToastClient {
    fn new(app_root: &Path) -> Self {
        Self {
            temp_dir: app_root.join("Temp").join("Notification"),
        }
    }
}

impl NotificationWindowsToastClient for DesktopNotificationWindowsToastClient {
    fn show_toast(
        &mut self,
        request: &NotificationWindowsToastRequest,
    ) -> std::result::Result<(), NotificationDispatchError> {
        show_windows_toast_notification(&self.temp_dir, request)
    }
}

#[cfg(windows)]
fn show_windows_toast_notification(
    temp_dir: &Path,
    request: &NotificationWindowsToastRequest,
) -> std::result::Result<(), NotificationDispatchError> {
    use windows::core::{Interface, HSTRING};
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::{DateTime, IReference, PropertyValue};
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

    const APP_USER_MODEL_ID: &str = "BetterGI.Rust.Desktop";
    const WINDOWS_TICKS_PER_SECOND: i64 = 10_000_000;
    const WINDOWS_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

    let hero_image_path = request
        .screenshot
        .as_ref()
        .map(|image| write_toast_screenshot(temp_dir, image))
        .transpose()?;
    let xml = windows_toast_xml(request.message.as_deref(), hero_image_path.as_deref());
    let document = XmlDocument::new()
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    document
        .LoadXml(&HSTRING::from(xml))
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;

    let toast = ToastNotification::CreateToastNotification(&document)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    toast
        .SetGroup(&HSTRING::from(request.event.clone()))
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;

    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?
        .as_secs() as i64;
    let expiration_seconds = now_seconds + i64::from(request.expiration_hours) * 60 * 60;
    let expiration = DateTime {
        UniversalTime: WINDOWS_UNIX_EPOCH_TICKS + expiration_seconds * WINDOWS_TICKS_PER_SECOND,
    };
    let expiration_ref: IReference<DateTime> = PropertyValue::CreateDateTime(expiration)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?
        .cast()
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    toast
        .SetExpirationTime(&expiration_ref)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;

    unsafe {
        SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(APP_USER_MODEL_ID))
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    }
    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_USER_MODEL_ID))
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    notifier
        .Show(&toast)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    Ok(())
}

#[cfg(not(windows))]
fn show_windows_toast_notification(
    _temp_dir: &Path,
    _request: &NotificationWindowsToastRequest,
) -> std::result::Result<(), NotificationDispatchError> {
    Err(NotificationDispatchError::UnsupportedProvider(
        "Windows UWP",
    ))
}

#[cfg(windows)]
fn write_toast_screenshot(
    temp_dir: &Path,
    image: &NotificationImage,
) -> std::result::Result<PathBuf, NotificationDispatchError> {
    fs::create_dir_all(temp_dir)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    let file_name = format!(
        "notification_image_{}.png",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?
            .as_nanos()
    );
    let path = temp_dir.join(file_name);
    fs::write(&path, &image.bytes)
        .map_err(|error| NotificationDispatchError::Transport(error.to_string()))?;
    Ok(path)
}

#[cfg(windows)]
fn windows_toast_xml(message: Option<&str>, hero_image_path: Option<&Path>) -> String {
    let mut visual = String::from("<binding template=\"ToastGeneric\">");
    if let Some(path) = hero_image_path {
        visual.push_str("<image placement=\"hero\" src=\"");
        visual.push_str(&xml_escape(&path_to_file_uri(path)));
        visual.push_str("\"/>");
    }
    if let Some(message) = message.and_then(non_empty_string) {
        visual.push_str("<text>");
        visual.push_str(&xml_escape(&message));
        visual.push_str("</text>");
    }
    visual.push_str("</binding>");
    format!("<toast><visual>{visual}</visual></toast>")
}

#[cfg(windows)]
fn path_to_file_uri(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    format!("file:///{}", normalized)
}

#[cfg(windows)]
fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[tauri::command]
fn dashboard_state(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> DashboardState {
    let workspace_root = app_root(&app).unwrap_or_else(|_| PathBuf::from("."));
    let config_path = app
        .path()
        .app_config_dir()
        .map(|dir| dir.join("config.json"))
        .ok()
        .filter(|path| path.exists());

    let config = config_path
        .and_then(|path| read_config(path).ok())
        .unwrap_or_default();

    let mut task_triggers = runtime_triggers(false);
    let dispatcher = task_state
        .dispatcher
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    apply_registered_realtime_triggers(&mut task_triggers, &dispatcher);
    let runner = task_state
        .runner
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    let selection = select_triggers_for_tick(
        &task_triggers,
        &dispatcher,
        std::time::Duration::from_secs(60),
    );

    DashboardState {
        navigation: default_navigation(),
        capabilities: migration_capabilities(),
        triggers: initial_triggers(),
        config: ConfigSummary::from(config),
        ui_shell: ui_shell_decision(),
        native_backend: NativeBackendSummary {
            input_events_in_demo_sequence: InputSequence::new()
                .modified_key_stroke([0x11], 0x41)
                .mouse_click(MouseButton::Left)
                .genshin_action(
                    &AppConfig::default().key_bindings_config,
                    GenshinAction::QuickUseGadget,
                    KeyActionType::KeyPress,
                )
                .unwrap_or_default()
                .events()
                .len()
                + release_pressed_keys_sequence([0x57, 0xA0]).events().len(),
            post_message_events_in_demo_sequence: post_message_events_for_action(
                &AppConfig::default().key_bindings_config,
                GenshinAction::QuickUseGadget,
                KeyActionType::KeyPress,
                PostMessageMode::Background,
            )
            .len(),
            sample_hotkey: "Ctrl + F11"
                .parse::<Hotkey>()
                .map(|hotkey| hotkey.to_string())
                .unwrap_or_else(|_| "Ctrl + F11".to_string()),
            capture_modes: capture_mode_infos(),
            recognition_types: recognition_type_infos(),
            registered_onnx_models: registered_onnx_models().len(),
            avatar_side_model: registered_onnx_models()
                .into_iter()
                .find(|model| model.rust_name == "BgiAvatarSide")
                .map(|model| {
                    model.load_plan(
                        &workspace_root,
                        env!("CARGO_PKG_VERSION"),
                        OnnxProviderSelection::CPU,
                    )
                }),
            windows_only_backends: vec!["BitBlt", "SendInput", "RegisterHotKey"],
        },
        task_runtime: TaskRuntimeSummary {
            dispatcher,
            runner,
            enabled_triggers: task_triggers
                .iter()
                .filter(|trigger| trigger.enabled)
                .count(),
            selected_triggers: selection.triggers,
            selection_reason: selection.reason,
            independent_tasks: independent_tasks(),
            catalog_entries: task_catalog().len(),
            config_bound_catalog_entries: task_catalog()
                .iter()
                .filter(|task| task.config_bound())
                .count(),
            native_pending_catalog_entries: task_catalog()
                .iter()
                .filter(|task| {
                    matches!(
                        task.port_state,
                        bgi_task::TaskPortState::MetadataOnly
                            | bgi_task::TaskPortState::ConfigBound
                            | bgi_task::TaskPortState::RuntimeScaffolded
                            | bgi_task::TaskPortState::NativePending
                    )
                })
                .count(),
        },
        script_runtime: ScriptRuntimePanel {
            summary: script_runtime_summary(),
            hosts: host_bindings(),
            security: script_host_security_summary(),
            sample_macro: KeyMouseScript::from_json(
                r#"{
                  "macroEvents": [
                    { "type": 0, "keyCode": 90, "time": 100 },
                    { "type": 1, "keyCode": 90, "time": 160 }
                  ],
                  "info": { "x": 0, "y": 0, "width": 1920, "height": 1080, "recordDpi": 1 }
                }"#,
            )
            .unwrap_or_default()
            .summary(),
        },
    }
}

fn apply_registered_realtime_triggers(
    triggers: &mut [bgi_task::RunnableTrigger],
    dispatcher: &DispatcherRuntime,
) {
    for registered in &dispatcher.registered_realtime_triggers {
        if let Some(trigger) = triggers.iter_mut().find(|trigger| {
            trigger
                .descriptor
                .key
                .eq_ignore_ascii_case(&registered.task_key)
        }) {
            trigger.enabled = true;
        }
    }
}

#[tauri::command]
fn notification_send_test(
    app: tauri::AppHandle,
    payload: Option<NotificationTestPayload>,
) -> Result<NotificationDispatchExecution, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "notification test dispatch requested");
    let config = read_desktop_config(&app, &app_root);
    let payload_request = payload.unwrap_or_default();
    let provider_kind = payload_request
        .provider
        .as_deref()
        .map(notification_provider_kind_from_str)
        .transpose()?;
    let message = payload_request
        .message
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| "BetterGI Rust test notification".to_string());
    let timestamp_ms = current_time_ms()?;
    let mut payload = NotificationPayload {
        event: "notify.test".to_string(),
        result: NotificationEventResult::Success,
        message: Some(message),
        data: None,
        timestamp_ms: Some(timestamp_ms),
        has_screenshot: false,
        screenshot: None,
    };
    if config.notification_config.include_screen_shot {
        if let Some(screenshot) = capture_notification_screenshot(&config) {
            payload = payload.with_screenshot(screenshot);
        }
    }
    let mut client = DesktopNotificationHttpClient::new().map_err(|error| error.to_string())?;
    let mut web_socket_client = DesktopNotificationWebSocketClient;
    let mut email_client = DesktopNotificationEmailClient;
    let mut windows_toast_client = DesktopNotificationWindowsToastClient::new(&app_root);
    if let Some(provider_kind) = provider_kind {
        let plan = notification_dispatch_plan_for_provider(
            &config.notification_config,
            payload,
            provider_kind,
        );
        return Ok(execute_notification_dispatch_plan(
            &config.notification_config,
            &plan,
            &mut client,
            &mut web_socket_client,
            &mut email_client,
            &mut windows_toast_client,
        ));
    }
    Ok(execute_notification_dispatch_with_transports(
        &config.notification_config,
        payload,
        &mut client,
        &mut web_socket_client,
        &mut email_client,
        &mut windows_toast_client,
    ))
}

#[tauri::command]
fn notification_service_state(
    app: tauri::AppHandle,
    state: tauri::State<'_, NotificationServiceState>,
) -> Result<DesktopNotificationServiceState, String> {
    let current = state
        .state
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    if current.initialized {
        return Ok(current);
    }
    refresh_notification_service_state(&app, &state)
}

#[tauri::command]
fn notification_service_refresh(
    app: tauri::AppHandle,
    state: tauri::State<'_, NotificationServiceState>,
) -> Result<DesktopNotificationServiceState, String> {
    refresh_notification_service_state(&app, &state)
}

#[tauri::command]
fn update_check(
    app: tauri::AppHandle,
    payload: Option<UpdateCheckPayload>,
) -> Result<DesktopUpdateCheckResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "update check requested");
    let config = read_desktop_config(&app, &app_root);
    let channel = payload
        .and_then(|payload| payload.channel)
        .as_deref()
        .map(update_channel_from_str)
        .transpose()?
        .unwrap_or(UpdateChannel::Stable);
    desktop_update_check(&app, UpdateTrigger::Manual, channel, Some(config))
}

#[tauri::command]
fn update_background_state(
    state: tauri::State<'_, BackgroundUpdateState>,
) -> Result<DesktopBackgroundUpdateState, String> {
    state
        .state
        .lock()
        .map(|state| state.clone())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_open_download_page(channel: String) -> Result<DesktopUpdateActionResult, String> {
    let channel = update_channel_from_str(&channel)?;
    let url = update_download_page_url(channel);
    open_url(url)?;
    Ok(DesktopUpdateActionResult {
        ok: true,
        action: "open_download_page".to_string(),
        detail: url.to_string(),
        exit_scheduled: false,
    })
}

#[tauri::command]
fn update_launch_updater(
    app: tauri::AppHandle,
    payload: Option<UpdateLaunchPayload>,
) -> Result<DesktopUpdateActionResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "updater launch requested");
    let updater_path = updater_exe_path(&app_root);
    if !updater_path.exists() {
        return Err(format!(
            "updater executable does not exist: {}",
            updater_path.display()
        ));
    }
    let payload = payload.unwrap_or(UpdateLaunchPayload {
        source: None,
        exit_after_launch: None,
    });
    let source = payload.source;
    let exit_after_launch = payload.exit_after_launch.unwrap_or(true);
    let plan = updater_launch_plan(source.as_deref())?;
    Command::new(&updater_path)
        .args(&plan.args)
        .spawn()
        .map_err(|error| format!("failed to start {}: {error}", updater_path.display()))?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "updater launched: {} {}, exit_after_launch={exit_after_launch}",
            updater_path.display(),
            plan.args.join(" ")
        ),
    );

    if exit_after_launch {
        let handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(800));
            handle.exit(0);
        });
    }

    Ok(DesktopUpdateActionResult {
        ok: true,
        action: "launch_updater".to_string(),
        detail: format!("{} {}", updater_path.display(), plan.args.join(" ")),
        exit_scheduled: exit_after_launch,
    })
}

#[tauri::command]
fn update_ignore_version(
    app: tauri::AppHandle,
    version: String,
) -> Result<DesktopUpdateActionResult, String> {
    let app_root = app_root(&app)?;
    let config_path = config_path(&app_root);
    let mut config = read_desktop_config(&app, &app_root);
    config.not_show_new_version_notice_end_version = version.trim().to_string();
    write_config(&config_path, &config).map_err(|error| error.to_string())?;
    Ok(DesktopUpdateActionResult {
        ok: true,
        action: "ignore_version".to_string(),
        detail: config.not_show_new_version_notice_end_version,
        exit_scheduled: false,
    })
}

#[tauri::command]
fn redeem_code_feed_check(app: tauri::AppHandle) -> Result<DesktopRedeemCodeFeedResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "redeem code feed check requested");
    let config = read_desktop_config(&app, &app_root);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("BetterGI Rust")
        .build()
        .map_err(|error| error.to_string())?;
    let remote_text = client
        .get(REDEEM_CODE_UPDATE_TIME_URL)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())?;
    let decision = redeem_code_feed_update_decision(
        &config.common_config.redeem_code_feeds_update_version,
        Some(&remote_text),
    );

    Ok(DesktopRedeemCodeFeedResult {
        request_url: REDEEM_CODE_UPDATE_TIME_URL.to_string(),
        local_version: config.common_config.redeem_code_feeds_update_version,
        remote_text: Some(remote_text),
        decision,
    })
}

#[tauri::command]
fn redeem_code_feed_mark_read(
    app: tauri::AppHandle,
    version: String,
) -> Result<DesktopRedeemCodeFeedResult, String> {
    let app_root = app_root(&app)?;
    let config_path = config_path(&app_root);
    let mut config = read_desktop_config(&app, &app_root);
    let version = version.trim().to_string();
    if version.is_empty() {
        return Err("redeem code feed version is empty".to_string());
    }
    config.common_config.redeem_code_feeds_update_version = version.clone();
    write_config(&config_path, &config).map_err(|error| error.to_string())?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("redeem code feed marked read at {version}"),
    );

    let decision = redeem_code_feed_update_decision(&version, Some(&version));
    Ok(DesktopRedeemCodeFeedResult {
        request_url: REDEEM_CODE_UPDATE_TIME_URL.to_string(),
        local_version: version.clone(),
        remote_text: Some(version),
        decision,
    })
}

#[tauri::command]
fn redeem_code_feed_items(
    app: tauri::AppHandle,
) -> Result<DesktopRedeemCodeFeedItemsResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "redeem code feed items requested");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("BetterGI Rust")
        .build()
        .map_err(|error| error.to_string())?;
    let json = client
        .get(REDEEM_CODE_CODES_URL)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())?;
    let items = parse_redeem_code_feed_items(&json).map_err(|error| error.to_string())?;

    Ok(DesktopRedeemCodeFeedItemsResult {
        request_url: REDEEM_CODE_CODES_URL.to_string(),
        raw_bytes: json.len(),
        items,
    })
}

#[tauri::command]
fn redeem_code_live_codes(app: tauri::AppHandle) -> Result<DesktopRedeemCodeLiveResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "redeem code live codes requested");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("BetterGI Rust")
        .build()
        .map_err(|error| error.to_string())?;
    let act_id_sources = vec![
        REDEEM_CODE_BBS_ACT_ID_1_URL.to_string(),
        REDEEM_CODE_BBS_ACT_ID_2_URL.to_string(),
    ];
    let mut act_id = None;
    for url in &act_id_sources {
        let text = client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(|error| error.to_string())?
            .text()
            .map_err(|error| error.to_string())?;
        act_id = redeem_code_live_act_id_from_bbs_response(&text);
        if act_id.is_some() {
            break;
        }
    }
    let act_id = act_id.ok_or_else(|| "no live preview act_id found".to_string())?;
    let index_text = client
        .get(REDEEM_CODE_LIVE_INDEX_URL)
        .header("x-rpc-act_id", &act_id)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())?;
    let (code_version, title) = redeem_code_live_index_from_response(&index_text)
        .ok_or_else(|| "live preview index did not return a code version".to_string())?;
    let refresh_code_url = live_refresh_code_url(&code_version);
    let code_text = client
        .get(&refresh_code_url)
        .header("x-rpc-act_id", &act_id)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())?;
    let codes = redeem_code_live_codes_from_response(&code_text);

    Ok(DesktopRedeemCodeLiveResult {
        act_id_sources,
        index_url: REDEEM_CODE_LIVE_INDEX_URL.to_string(),
        refresh_code_url,
        data: RedeemCodeLiveData {
            act_id,
            code_version,
            title,
            codes,
        },
    })
}

#[tauri::command]
fn redeem_code_auto_redeem_plan(
    app: tauri::AppHandle,
    payload: RedeemCodePlanPayload,
) -> Result<DesktopRedeemCodePlanResult, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(&app_root, "INFO", "redeem code auto-redeem plan requested");
    let entries = redeem_code_entries_from_payload(payload);
    let extracted_codes = entries.iter().map(|entry| entry.code.clone()).collect();
    let plan = plan_use_redeem_codes_through_task_boundary(entries, &app_root)?;
    Ok(DesktopRedeemCodePlanResult {
        extracted_codes,
        plan,
    })
}

#[tauri::command]
fn redeem_code_clipboard_state(
    app: tauri::AppHandle,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<DesktopRedeemCodeClipboardState, String> {
    let app_root = app_root(&app)?;
    let config = read_desktop_config(&app, &app_root);
    let ignored_hash_count = state
        .ignored_hashes
        .lock()
        .map_err(|error| error.to_string())?
        .len();
    Ok(DesktopRedeemCodeClipboardState {
        clipboard_listener_enabled: config.auto_redeem_code_config.clipboard_listener_enabled,
        ignored_hash_count,
    })
}

#[tauri::command]
fn redeem_code_clipboard_set_enabled(
    app: tauri::AppHandle,
    enabled: bool,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<DesktopRedeemCodeClipboardState, String> {
    let app_root = app_root(&app)?;
    let config_path = config_path(&app_root);
    let mut config = read_desktop_config(&app, &app_root);
    config.auto_redeem_code_config.clipboard_listener_enabled = enabled;
    write_config(&config_path, &config).map_err(|error| error.to_string())?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("redeem code clipboard listener set to {enabled}"),
    );
    let ignored_hash_count = state
        .ignored_hashes
        .lock()
        .map_err(|error| error.to_string())?
        .len();
    Ok(DesktopRedeemCodeClipboardState {
        clipboard_listener_enabled: enabled,
        ignored_hash_count,
    })
}

#[tauri::command]
fn redeem_code_clipboard_check(
    app: tauri::AppHandle,
    payload: RedeemCodeClipboardCheckPayload,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<DesktopRedeemCodeClipboardCheckResult, String> {
    let app_root = app_root(&app)?;
    let text = payload.text.unwrap_or_default();
    let result = redeem_code_clipboard_check_text(&app, &state, text, "manual")?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "redeem code clipboard check found {} code(s), ignored={ignored}",
            result.extracted_codes.len(),
            ignored = result.ignored
        ),
    );
    Ok(result)
}

#[tauri::command]
fn redeem_code_clipboard_ignore(
    app: tauri::AppHandle,
    payload: RedeemCodeClipboardCheckPayload,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<DesktopRedeemCodeClipboardState, String> {
    let app_root = app_root(&app)?;
    let config = read_desktop_config(&app, &app_root);
    let text = payload.text.unwrap_or_default();
    let ignored_hash_count = remember_ignored_clipboard_hash(&state, &text)?;
    append_desktop_log(&app_root, "INFO", "redeem code clipboard text ignored");
    Ok(DesktopRedeemCodeClipboardState {
        clipboard_listener_enabled: config.auto_redeem_code_config.clipboard_listener_enabled,
        ignored_hash_count,
    })
}

fn redeem_code_entries_from_payload(payload: RedeemCodePlanPayload) -> Vec<RedeemCodeEntry> {
    let mut entries = Vec::new();

    if let Some(text) = payload.text {
        let extracted = extract_redeem_codes_from_text(&text);
        entries.extend(redeem_code_entries_from_strings(
            extracted.iter().map(String::as_str),
        ));
    }

    if let Some(codes) = payload.codes {
        entries
            .extend(redeem_code_entries_from_strings(codes.iter().map(String::as_str)).into_iter());
    }

    if let Some(feed_items) = payload.feed_items {
        for item in feed_items {
            for code in item.codes {
                let items = if item.content.trim().is_empty() {
                    None
                } else {
                    Some(item.content.clone())
                };
                if let Some(entry) = RedeemCodeEntry::new(code, items) {
                    entries.push(entry);
                }
            }
        }
    }

    if let Some(live_codes) = payload.live_codes {
        for code in live_codes {
            if let Some(entry) = RedeemCodeEntry::new(code.code, Some(code.items)) {
                entries.push(entry);
            }
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    entries
        .into_iter()
        .filter(|entry| seen.insert(entry.code.clone()))
        .collect()
}

fn redeem_code_clipboard_check_text(
    app: &tauri::AppHandle,
    state: &RedeemCodeClipboardState,
    text: String,
    source: &str,
) -> Result<DesktopRedeemCodeClipboardCheckResult, String> {
    let app_root = app_root(app)?;
    let config = read_desktop_config(app, &app_root);
    let hash = redeem_clipboard_hash(&text);
    let ignored = state
        .ignored_hashes
        .lock()
        .map_err(|error| error.to_string())?
        .contains(&hash);
    let extracted_codes = if config.auto_redeem_code_config.clipboard_listener_enabled && !ignored {
        extract_redeem_codes_from_text(&text)
    } else {
        Vec::new()
    };
    let plan = if extracted_codes.is_empty() {
        None
    } else {
        let entries = redeem_code_entries_from_strings(extracted_codes.iter().map(String::as_str));
        Some(plan_use_redeem_codes_through_task_boundary(
            entries, &app_root,
        )?)
    };

    Ok(DesktopRedeemCodeClipboardCheckResult {
        clipboard_listener_enabled: config.auto_redeem_code_config.clipboard_listener_enabled,
        ignored,
        hash,
        text,
        extracted_codes,
        plan,
        source: source.to_string(),
    })
}

fn plan_use_redeem_codes_through_task_boundary(
    entries: Vec<RedeemCodeEntry>,
    app_root: &Path,
) -> Result<UseRedeemCodeExecutionPlan, String> {
    let request = IndependentTaskExecutionRequest::use_redeem_code(
        entries,
        bgi_vision::Size::new(1920, 1080),
        app_root,
    );
    let execution = execute_independent_task_with_cancel(&request, || false)
        .map_err(|error| error.to_string())?;
    let IndependentTaskExecution::UseRedeemCodePlan(plan) = execution.execution else {
        return Err("UseRedeemCode returned a non-redeem execution".to_string());
    };
    Ok(plan)
}

fn remember_ignored_clipboard_hash(
    state: &RedeemCodeClipboardState,
    text: &str,
) -> Result<usize, String> {
    let text = text.trim();
    let mut hashes = state
        .ignored_hashes
        .lock()
        .map_err(|error| error.to_string())?;
    if text.is_empty() {
        return Ok(hashes.len());
    }
    if hashes.len() > 10 {
        hashes.clear();
    }
    hashes.insert(redeem_clipboard_hash(text));
    Ok(hashes.len())
}

fn script_repo_clipboard_detect_text(
    state: &RedeemCodeClipboardState,
    text: &str,
    source: &str,
) -> Result<Option<ScriptRepoDesktopClipboardImportResult>, String> {
    let Some(plan) = parse_import_uri(text, true).map_err(|error| error.to_string())? else {
        return Ok(None);
    };
    let hash = redeem_clipboard_hash(&plan.uri);
    let ignored = state
        .ignored_hashes
        .lock()
        .map_err(|error| error.to_string())?
        .contains(&hash);
    if ignored {
        return Ok(None);
    }

    Ok(Some(ScriptRepoDesktopClipboardImportResult {
        uri: plan.uri,
        hash,
        path_json: plan.path_json,
        paths: plan.paths,
        source: source.to_string(),
    }))
}

fn start_redeem_clipboard_monitor(app: &tauri::App) {
    let handle = app.handle().clone();
    append_desktop_app_log(&handle, "INFO", "redeem code clipboard monitor started");
    std::thread::spawn(move || loop {
        std::thread::sleep(REDEEM_CLIPBOARD_POLL_INTERVAL);
        let Some(text) = read_system_clipboard_text() else {
            continue;
        };
        let text = text.trim().to_string();
        if text.is_empty() || text.len() > REDEEM_CLIPBOARD_MAX_TEXT_LEN {
            continue;
        }

        let hash = redeem_clipboard_hash(&text);
        let Some(state) = handle.try_state::<RedeemCodeClipboardState>() else {
            continue;
        };
        let already_seen = match state.last_hash.lock() {
            Ok(mut last_hash) => {
                if last_hash.as_deref() == Some(&hash) {
                    true
                } else {
                    *last_hash = Some(hash);
                    false
                }
            }
            Err(_) => true,
        };
        if already_seen {
            continue;
        }

        match script_repo_clipboard_detect_text(&state, &text, "system") {
            Ok(Some(result)) => {
                append_desktop_app_log(
                    &handle,
                    "INFO",
                    &format!(
                        "script repository clipboard monitor detected {} path(s)",
                        result.paths.len()
                    ),
                );
                let _ = handle.emit(SCRIPT_IMPORT_CLIPBOARD_EVENT, result);
                continue;
            }
            Ok(None) => {}
            Err(error) => append_desktop_app_log(
                &handle,
                "WARN",
                &format!("script repository clipboard monitor failed: {error}"),
            ),
        }

        match redeem_code_clipboard_check_text(&handle, &state, text, "system") {
            Ok(result) if !result.extracted_codes.is_empty() => {
                append_desktop_app_log(
                    &handle,
                    "INFO",
                    &format!(
                        "redeem code clipboard monitor detected {} code(s)",
                        result.extracted_codes.len()
                    ),
                );
                let _ = handle.emit(REDEEM_CLIPBOARD_EVENT, result);
            }
            Ok(_) => {}
            Err(error) => append_desktop_app_log(
                &handle,
                "WARN",
                &format!("redeem code clipboard monitor failed: {error}"),
            ),
        }
    });
}

fn redeem_clipboard_hash(text: &str) -> String {
    hex_lower(Md5::digest(text.as_bytes()).as_slice())
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(windows)]
fn read_system_clipboard_text() -> Option<String> {
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    };
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    use windows::Win32::System::Ole::CF_UNICODETEXT;

    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() {
            return None;
        }
        if OpenClipboard(None).is_err() {
            return None;
        }

        let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
            Ok(handle) => handle,
            Err(_) => {
                let _ = CloseClipboard();
                return None;
            }
        };
        let global = HGLOBAL(handle.0);
        let ptr = GlobalLock(global) as *const u16;
        if ptr.is_null() {
            let _ = CloseClipboard();
            return None;
        }

        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
            if len > REDEEM_CLIPBOARD_MAX_TEXT_LEN + 1 {
                break;
            }
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        let _ = GlobalUnlock(global);
        let _ = CloseClipboard();
        Some(text)
    }
}

#[cfg(not(windows))]
fn read_system_clipboard_text() -> Option<String> {
    None
}

#[cfg(windows)]
fn clear_system_clipboard() -> bool {
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard};

    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let cleared = EmptyClipboard().is_ok();
        let _ = CloseClipboard();
        cleared
    }
}

#[cfg(not(windows))]
fn clear_system_clipboard() -> bool {
    false
}

#[tauri::command]
fn desktop_shell_state(app: tauri::AppHandle) -> Result<DesktopShellState, String> {
    let app_root = app_root(&app)?;
    let config = read_desktop_config(&app, &app_root);
    Ok(desktop_shell_state_from(&app_root, &config))
}

#[tauri::command]
fn desktop_shell_set_exit_to_tray(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<DesktopShellState, String> {
    let app_root = app_root(&app)?;
    let config_path = config_path(&app_root);
    let mut config = read_desktop_config(&app, &app_root);
    config.common_config.exit_to_tray = enabled;
    write_config(&config_path, &config).map_err(|error| error.to_string())?;
    append_desktop_log(&app_root, "INFO", &format!("exit_to_tray set to {enabled}"));
    Ok(desktop_shell_state_from(&app_root, &config))
}

#[tauri::command]
fn desktop_overlay_state(app: tauri::AppHandle) -> Result<MaskWindowState, String> {
    let app_root = app_root(&app)?;
    let config = read_desktop_config(&app, &app_root);
    Ok(config.mask_window_config.overlay_state())
}

#[tauri::command]
fn desktop_overlay_update(
    app: tauri::AppHandle,
    payload: OverlayPatchPayload,
) -> Result<MaskWindowState, String> {
    let app_root = app_root(&app)?;
    let config_path = config_path(&app_root);
    let mut config = read_desktop_config(&app, &app_root);
    let overlay = &mut config.mask_window_config;

    if let Some(value) = payload.mask_enabled {
        overlay.mask_enabled = value;
    }
    if let Some(value) = payload.show_log_box {
        overlay.show_log_box = value;
    }
    if let Some(value) = payload.show_status {
        overlay.show_status = value;
    }
    if let Some(value) = payload.display_recognition_results_on_mask {
        overlay.display_recognition_results_on_mask = value;
    }
    if let Some(value) = payload.show_overlay_metrics {
        overlay.show_overlay_metrics = value;
    }
    if let Some(value) = payload.overlay_layout_edit_enabled {
        overlay.overlay_layout_edit_enabled = value;
    }
    if payload.reset_metrics_layout.unwrap_or(false) {
        let default_state = MaskWindowConfig::default().overlay_state();
        overlay.metrics_left_ratio = default_state.metrics_layout.left_ratio;
        overlay.metrics_top_ratio = default_state.metrics_layout.top_ratio;
        overlay.metrics_width_ratio = default_state.metrics_layout.width_ratio;
        overlay.metrics_height_ratio = default_state.metrics_layout.height_ratio;
    }
    if let Some(metric_key) = payload.metric_key.as_deref() {
        if overlay_metric_item_from_key(metric_key).is_none() {
            return Err(format!("unsupported overlay metric: {metric_key}"));
        }
        let Some(enabled) = payload.metric_enabled else {
            return Err("metricEnabled is required when metricKey is set".to_string());
        };
        overlay.ensure_overlay_metric_items();
        overlay
            .overlay_metric_items
            .insert(metric_key.to_string(), enabled);
    }

    overlay.ensure_overlay_metric_items();
    write_config(&config_path, &config).map_err(|error| error.to_string())?;
    append_desktop_log(&app_root, "INFO", "desktop overlay config updated");
    Ok(config.mask_window_config.overlay_state())
}

#[tauri::command]
fn desktop_shell_show_main_window(
    app: tauri::AppHandle,
) -> Result<DesktopShellActionResult, String> {
    show_main_window(&app)?;
    Ok(DesktopShellActionResult {
        ok: true,
        action: "show_main_window".to_string(),
        detail: "main window shown".to_string(),
    })
}

#[tauri::command]
fn desktop_shell_hide_main_window(
    app: tauri::AppHandle,
) -> Result<DesktopShellActionResult, String> {
    hide_main_window(&app)?;
    Ok(DesktopShellActionResult {
        ok: true,
        action: "hide_main_window".to_string(),
        detail: "main window hidden".to_string(),
    })
}

#[tauri::command]
fn desktop_shell_toggle_main_window(
    app: tauri::AppHandle,
) -> Result<DesktopShellActionResult, String> {
    let visible = toggle_main_window(&app)?;
    Ok(DesktopShellActionResult {
        ok: true,
        action: "toggle_main_window".to_string(),
        detail: if visible {
            "main window shown".to_string()
        } else {
            "main window hidden".to_string()
        },
    })
}

#[tauri::command]
fn desktop_log_state(
    app: tauri::AppHandle,
    tail: Option<usize>,
) -> Result<DesktopLogState, String> {
    let app_root = app_root(&app)?;
    let path = current_desktop_log_path(&app_root);
    let exists = path.exists();
    let bytes = if exists {
        fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .len()
    } else {
        0
    };
    let tail = if exists {
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let count = tail.unwrap_or(80).min(500);
        let mut lines = text
            .lines()
            .rev()
            .take(count)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        lines.reverse();
        lines
    } else {
        Vec::new()
    };

    Ok(DesktopLogState {
        path: path.display().to_string(),
        exists,
        bytes,
        tail,
    })
}

#[tauri::command]
fn script_repo_state(
    app: tauri::AppHandle,
    repo_path: Option<String>,
) -> Result<ScriptRepoDesktopState, String> {
    let app_root = app_root(&app)?;
    let repo_path = repo_path.map(PathBuf::from).unwrap_or_else(|| {
        app_root
            .join("Repos")
            .join(bgi_script::DEFAULT_REPO_FOLDER_NAME)
    });
    let repo_exists = repo_path.exists();
    if !repo_exists {
        return Ok(ScriptRepoDesktopState {
            repo_path: repo_path.display().to_string(),
            repo_exists,
            repo_json_path: None,
            subscription_file_path: subscription_file_for_repo(&app_root, &repo_path)
                .display()
                .to_string(),
            repo_json_bytes: 0,
            subscribed_paths: Vec::new(),
            index_nodes: Vec::new(),
        });
    }

    let paths =
        script_repo_bridge_paths(&app_root, &repo_path, None).map_err(|error| error.to_string())?;
    let repo_json = if repo_exists {
        read_repo_bridge_repo_json(&repo_path).unwrap_or_default()
    } else {
        String::new()
    };
    let index_nodes = if repo_exists {
        repo_bridge_index_nodes(&repo_path).unwrap_or_default()
    } else {
        Vec::new()
    };
    let subscribed_paths = repo_bridge_subscribed_paths_json(&paths.subscription_file_path)
        .ok()
        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
        .unwrap_or_default();

    Ok(ScriptRepoDesktopState {
        repo_path: repo_path.display().to_string(),
        repo_exists,
        repo_json_path: repo_exists.then(|| paths.repo_json_path.display().to_string()),
        subscription_file_path: paths.subscription_file_path.display().to_string(),
        repo_json_bytes: repo_json.len(),
        subscribed_paths,
        index_nodes,
    })
}

#[tauri::command]
fn script_repo_json(_app: tauri::AppHandle, repo_path: String) -> Result<String, String> {
    read_repo_bridge_repo_json(repo_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn script_repo_subscriptions(
    app: tauri::AppHandle,
    repo_path: String,
) -> Result<Vec<String>, String> {
    let app_root = app_root(&app)?;
    let paths =
        script_repo_bridge_paths(&app_root, repo_path, None).map_err(|error| error.to_string())?;
    repo_bridge_subscribed_paths_json(paths.subscription_file_path)
        .and_then(|json| {
            serde_json::from_str::<Vec<String>>(&json).map_err(|source| {
                bgi_script::ScriptRepoError::Json {
                    path: PathBuf::from(bgi_script::SUBSCRIPTIONS_DIR),
                    source,
                }
            })
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_repo_file(
    repo_path: String,
    rel_path: String,
) -> Result<Option<bgi_script::ScriptRepoBridgeFileResponse>, String> {
    read_repo_bridge_file(repo_path, &rel_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn script_repo_mark_updated(
    app: tauri::AppHandle,
    repo_path: String,
    path: String,
) -> Result<bool, String> {
    let app_root = app_root(&app)?;
    let paths =
        script_repo_bridge_paths(&app_root, repo_path, None).map_err(|error| error.to_string())?;
    mark_repo_bridge_path_updated(paths.repo_json_path, &path).map_err(|error| error.to_string())
}

#[tauri::command]
fn script_repo_clear_update(repo_path: String) -> Result<String, String> {
    clear_repo_bridge_update(repo_path)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_repo_import_paths(
    app: tauri::AppHandle,
    repo_path: String,
    paths: Vec<String>,
    git_repo: bool,
) -> Result<ScriptRepoDesktopImportResult, String> {
    let app_root = app_root(&app)?;
    import_paths_for_repo(&app_root, &repo_path, paths, git_repo)
}

#[tauri::command]
fn script_repo_import_uri(
    app: tauri::AppHandle,
    repo_path: String,
    uri: String,
    git_repo: bool,
) -> Result<ScriptRepoDesktopUriImportResult, String> {
    let app_root = app_root(&app)?;
    let plan = parse_import_uri(&uri, false)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "not a bettergi://script import URI".to_string())?;
    let result = import_paths_for_repo(&app_root, &repo_path, plan.paths.clone(), git_repo)?;
    Ok(ScriptRepoDesktopUriImportResult {
        paths: plan.paths,
        clear_clipboard_after_import: plan.clear_clipboard_after_import,
        result,
    })
}

#[tauri::command]
fn script_repo_import_clipboard_uri(
    app: tauri::AppHandle,
    repo_path: String,
    uri: String,
    git_repo: bool,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<ScriptRepoDesktopUriImportResult, String> {
    let app_root = app_root(&app)?;
    let plan = parse_import_uri(&uri, true)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "not a bettergi://script import URI".to_string())?;
    let result = import_paths_for_repo(&app_root, &repo_path, plan.paths.clone(), git_repo)?;
    let ignored_hash_count = remember_ignored_clipboard_hash(&state, &plan.uri)?;
    let clipboard_cleared = if plan.clear_clipboard_after_import {
        clear_system_clipboard()
    } else {
        false
    };
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "script repository clipboard import completed for {} path(s), clipboard_cleared={clipboard_cleared}, ignored_hashes={ignored_hash_count}",
            plan.paths.len()
        ),
    );
    Ok(ScriptRepoDesktopUriImportResult {
        paths: plan.paths,
        clear_clipboard_after_import: plan.clear_clipboard_after_import,
        result,
    })
}

#[tauri::command]
fn script_repo_clipboard_ignore(
    app: tauri::AppHandle,
    payload: RedeemCodeClipboardCheckPayload,
    state: tauri::State<'_, RedeemCodeClipboardState>,
) -> Result<DesktopRedeemCodeClipboardState, String> {
    let app_root = app_root(&app)?;
    let config = read_desktop_config(&app, &app_root);
    let text = payload.text.unwrap_or_default();
    let plan = parse_import_uri(&text, true)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "not a bettergi://script import URI".to_string())?;
    let ignored_hash_count = remember_ignored_clipboard_hash(&state, &plan.uri)?;
    let clipboard_cleared = if plan.clear_clipboard_after_import {
        clear_system_clipboard()
    } else {
        false
    };
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "script repository clipboard import ignored, clipboard_cleared={clipboard_cleared}, ignored_hashes={ignored_hash_count}"
        ),
    );
    Ok(DesktopRedeemCodeClipboardState {
        clipboard_listener_enabled: config.auto_redeem_code_config.clipboard_listener_enabled,
        ignored_hash_count,
    })
}

#[tauri::command]
fn script_repo_update_subscribed(
    app: tauri::AppHandle,
    repo_path: String,
    git_repo: bool,
) -> Result<ScriptRepoDesktopImportResult, String> {
    let app_root = app_root(&app)?;
    let subscription_file_path = subscription_file_for_repo(&app_root, Path::new(&repo_path));
    let paths =
        read_subscription_file(&subscription_file_path).map_err(|error| error.to_string())?;
    import_paths_for_repo(&app_root, &repo_path, paths, git_repo)
}

#[tauri::command]
fn script_repo_update_from_git(
    app: tauri::AppHandle,
    repo_url: String,
) -> Result<ScriptRepoDesktopUpdateResult, String> {
    let app_root = app_root(&app)?;
    let plan = git_update_plan(&app_root, repo_url, &BTreeMap::new());
    let mut runner = SystemGitRunner::default();
    let result = execute_git_repo_update(&plan, &mut runner).map_err(|error| error.to_string())?;

    Ok(ScriptRepoDesktopUpdateResult {
        repo_url: result.repo_url,
        branch: result.branch,
        repo_folder_name: result.repo_folder_name,
        repo_path: result.repo_path.display().to_string(),
        repo_updated_json_path: result.repo_updated_json_path.display().to_string(),
        updated: result.updated,
        cloned: result.cloned,
        remote_changed: result.remote_changed,
        created_new_folder: result.created_new_folder,
        fallback_reclone: result.fallback_reclone,
        marker_generated: result.marker_generated,
        old_repo_overlap_ratio: result.old_repo_overlap_ratio,
        current_commit: result.current_commit,
        remote_commit: result.remote_commit,
    })
}

#[tauri::command]
fn script_repo_import_zip(
    app: tauri::AppHandle,
    zip_path: String,
    folder: Option<String>,
) -> Result<ScriptRepoDesktopZipImportResult, String> {
    let app_root = app_root(&app)?;
    let plan = zip_import_plan(&app_root, zip_path, folder.as_deref());
    let result = execute_zip_repo_import(&plan).map_err(|error| error.to_string())?;

    Ok(ScriptRepoDesktopZipImportResult {
        zip_path: result.zip_path.display().to_string(),
        repo_json_path: result.repo_json_path.display().to_string(),
        target_folder_name: result.target_folder_name,
        target_path: result.target_path.display().to_string(),
        repo_updated_json_path: result.repo_updated_json_path.display().to_string(),
        best_overlap_ratio: result.best_overlap_ratio,
        matched_existing_folder: result.matched_existing_folder,
        old_repo_overlap_ratio: result.old_repo_overlap_ratio,
        marker_generated: result.marker_generated,
    })
}

#[tauri::command]
fn script_groups(app: tauri::AppHandle) -> Result<Vec<ScriptGroupDesktopSummary>, String> {
    let app_root = app_root(&app)?;
    read_script_groups(script_group_root(&app_root))
        .map(|groups| groups.into_iter().map(script_group_file_summary).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_available_js_projects(
    app: tauri::AppHandle,
) -> Result<Vec<AvailableJsProjectDesktopSummary>, String> {
    let app_root = app_root(&app)?;
    available_js_script_projects(js_script_root(&app_root))
        .map(|projects| {
            projects
                .into_iter()
                .map(|project| AvailableJsProjectDesktopSummary {
                    folder_name: project.folder_name,
                    name: project.name,
                    version: project.version,
                    description: project.description,
                    settings_ui: project.settings_ui,
                    has_settings_ui: project.has_settings_ui,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_available_key_mouse_scripts(
    app: tauri::AppHandle,
) -> Result<Vec<AvailableKeyMouseDesktopSummary>, String> {
    let app_root = app_root(&app)?;
    available_key_mouse_scripts(key_mouse_script_root(&app_root))
        .map(|scripts| {
            scripts
                .into_iter()
                .map(|script| AvailableKeyMouseDesktopSummary {
                    name: script.name,
                    relative_path: script.relative_path,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_available_pathing_scripts(
    app: tauri::AppHandle,
) -> Result<Vec<AvailablePathingDesktopSummary>, String> {
    let app_root = app_root(&app)?;
    available_pathing_scripts(pathing_script_root(&app_root))
        .map(|scripts| {
            scripts
                .into_iter()
                .map(|script| AvailablePathingDesktopSummary {
                    name: script.name,
                    folder_name: script.folder_name,
                    relative_path: script.relative_path,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_available_pathing_tree(
    app: tauri::AppHandle,
) -> Result<bgi_script::AvailablePathingTreeNode, String> {
    let app_root = app_root(&app)?;
    available_pathing_tree(pathing_script_root(&app_root)).map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_create(
    app: tauri::AppHandle,
    name: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    create_script_group(script_group_root(&app_root), &name)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_rename(
    app: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    rename_script_group(script_group_root(&app_root), &old_name, &new_name)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_delete(app: tauri::AppHandle, name: String) -> Result<bool, String> {
    let app_root = app_root(&app)?;
    delete_script_group(script_group_root(&app_root), &name).map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_add_js(
    app: tauri::AppHandle,
    group_name: String,
    folder_name: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    let project = read_script_project_summary(&app_root, &folder_name)?;
    add_script_group_project(
        script_group_root(&app_root),
        &group_name,
        ScriptGroupProject::javascript(project.name, folder_name),
    )
    .map(script_group_file_summary)
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_add_key_mouse(
    app: tauri::AppHandle,
    group_name: String,
    name: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    add_key_mouse_script_project(script_group_root(&app_root), &group_name, name)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_add_pathing(
    app: tauri::AppHandle,
    group_name: String,
    name: String,
    folder_name: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    add_pathing_script_project(script_group_root(&app_root), &group_name, name, folder_name)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_add_shell(
    app: tauri::AppHandle,
    group_name: String,
    command: String,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    add_shell_script_project(script_group_root(&app_root), &group_name, command)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_update(
    app: tauri::AppHandle,
    group_name: String,
    project_index: usize,
    patch: ScriptGroupProjectPatchPayload,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    update_script_group_project(
        script_group_root(&app_root),
        &group_name,
        project_index,
        patch.into_patch()?,
    )
    .map(script_group_file_summary)
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_remove(
    app: tauri::AppHandle,
    group_name: String,
    project_index: usize,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    remove_script_group_project(script_group_root(&app_root), &group_name, project_index)
        .map(script_group_file_summary)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_group_project_move(
    app: tauri::AppHandle,
    group_name: String,
    project_index: usize,
    target_index: usize,
) -> Result<ScriptGroupDesktopSummary, String> {
    let app_root = app_root(&app)?;
    move_script_group_project(
        script_group_root(&app_root),
        &group_name,
        project_index,
        target_index,
    )
    .map(script_group_file_summary)
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn script_settings_document(
    app: tauri::AppHandle,
    group_name: String,
    project_index: usize,
) -> Result<ScriptSettingsDesktopDocument, String> {
    let app_root = app_root(&app)?;
    let group_path = script_group_file_path(script_group_root(&app_root), &group_name);
    let group = read_script_group_file(&group_path).map_err(|error| error.to_string())?;
    let project = group
        .projects
        .get(project_index)
        .ok_or_else(|| format!("project index {project_index} was not found"))?;
    let document = read_script_settings_document(
        js_script_root(&app_root),
        project.folder_name.clone(),
        project.js_script_settings_object.clone(),
    )
    .map_err(|error| error.to_string())?;

    Ok(ScriptSettingsDesktopDocument {
        group_name,
        project_index,
        project_folder_name: document.project_folder_name,
        project_path: document.project_path.display().to_string(),
        manifest_name: document.manifest.name,
        manifest_version: document.manifest.version,
        settings_ui_path: document
            .settings_ui_path
            .map(|path| path.display().to_string()),
        items: document
            .schema
            .map(|schema| schema.items)
            .unwrap_or_default(),
        values: document.values,
        defaults_applied: document.defaults_applied,
        cleaned_invalid_values: document.cleaned_invalid_values,
    })
}

#[tauri::command]
fn script_settings_save(
    app: tauri::AppHandle,
    group_name: String,
    project_index: usize,
    values: serde_json::Value,
) -> Result<ScriptSettingsDesktopSaveResult, String> {
    let app_root = app_root(&app)?;
    let result = save_script_group_project_settings(
        script_group_root(&app_root),
        &group_name,
        project_index,
        js_script_root(&app_root),
        values,
    )
    .map_err(|error| error.to_string())?;

    Ok(ScriptSettingsDesktopSaveResult {
        group_path: result.group_path.display().to_string(),
        group_name: result.group_name,
        project_index: result.project_index,
        project_folder_name: result.project_folder_name,
        settings: result.settings,
        cleaned_invalid_values: result.cleaned_invalid_values,
    })
}

#[tauri::command]
fn script_execute_js(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    folder_name: String,
    settings: Option<serde_json::Value>,
) -> Result<JavaScriptExecutionOutcome, String> {
    let app_root = app_root(&app)?;
    let script_cancellation =
        start_desktop_script_run(&task_state, format!("JavaScript:{folder_name}"))?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("javascript project execution requested: {folder_name}"),
    );
    let config = read_desktop_config(&app, &app_root);
    let game_window = find_desktop_game_window(&config);
    let html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let mut outcome =
            bgi_script_engine::execute_javascript_project_with_host_task_dispatcher_and_cancellation(
                js_script_root(&app_root),
                folder_name,
                settings,
                |host| {
                    host.notification_dispatch_mode = NotificationDispatchMode::Sink;
                    host.html_mask_initial_state = html_mask_initial_state.clone();
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                &mut dispatcher,
                Some(script_cancellation.as_ref()),
            )
            .map_err(|error| error.to_string())?;
        restore_html_mask_from_script_outcome(&app, &outcome);
        dispatch_script_notifications_in_javascript_outcome(&config, &mut outcome);
        dispatch_script_html_mask_in_javascript_outcome(&app, &mut outcome);
        Ok(outcome)
    })();
    finish_desktop_script_run(&task_state);
    result
}

fn start_desktop_script_run(
    task_state: &DesktopTaskRuntimeState,
    task_name: impl Into<String>,
) -> Result<Arc<InputCancellationToken>, String> {
    task_state.script_cancellation.reset();
    let mut runner = task_state
        .runner
        .lock()
        .map_err(|_| "task runner state lock poisoned".to_string())?;
    runner
        .start_task(task_name)
        .map_err(|error| error.to_string())?;
    Ok(Arc::clone(&task_state.script_cancellation))
}

fn finish_desktop_script_run(task_state: &DesktopTaskRuntimeState) {
    if let Ok(mut runner) = task_state.runner.lock() {
        runner.stop_task();
    }
}

#[tauri::command]
fn script_stop(
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<ScriptStopResult, String> {
    task_state.script_cancellation.cancel();
    let mut runner = task_state
        .runner
        .lock()
        .map_err(|_| "task runner state lock poisoned".to_string())?;
    let requested = matches!(
        runner.state,
        bgi_task::TaskRuntimeState::Running
            | bgi_task::TaskRuntimeState::Starting
            | bgi_task::TaskRuntimeState::Suspended
    );
    if requested {
        runner.state = bgi_task::TaskRuntimeState::Stopping;
        runner.suspended = false;
    }
    Ok(ScriptStopResult {
        requested,
        runner_state: runner.clone(),
    })
}

#[tauri::command]
fn task_execute_shell(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    payload: DesktopShellTaskPayload,
) -> Result<DesktopShellTaskExecution, String> {
    let app_root = app_root(&app)?;
    let shell_command = payload.command.trim().to_string();
    let script_cancellation =
        start_desktop_script_run(&task_state, "IndependentTask:Shell".to_string())?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("independent shell task requested: {shell_command}"),
    );
    let working_directory = payload
        .working_directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| app_root.clone());
    let config = ShellConfig {
        disable: payload.disable.unwrap_or(false),
        timeout: payload.timeout_seconds.unwrap_or(60),
        no_window: payload.no_window.unwrap_or(true),
        output: payload.output.unwrap_or(true),
    };
    let request =
        IndependentTaskExecutionRequest::shell(shell_command.clone(), config, working_directory);
    let execution =
        execute_independent_task_with_cancel(&request, || script_cancellation.is_cancelled())
            .map_err(|error| error.to_string());
    finish_desktop_script_run(&task_state);
    let execution = execution?;
    let IndependentTaskExecution::Shell(result) = execution.execution else {
        return Err("independent Shell task returned a non-shell execution".to_string());
    };
    Ok(DesktopShellTaskExecution {
        task: execution.task_key,
        result,
    })
}

#[tauri::command]
fn task_plan_auto_pathing(
    app: tauri::AppHandle,
    payload: DesktopAutoPathingTaskPayload,
) -> Result<DesktopAutoPathingTaskExecution, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("independent auto-pathing plan requested: {}", payload.route),
    );
    let request = IndependentTaskExecutionRequest::auto_pathing(payload.route.clone(), &app_root);
    let execution = execute_independent_task_with_cancel(&request, || false)
        .map_err(|error| error.to_string())?;
    let IndependentTaskExecution::AutoPathingPlan(result) = execution.execution else {
        return Err("AutoPathing returned a non-pathing execution".to_string());
    };
    Ok(DesktopAutoPathingTaskExecution {
        task: execution.task_key,
        result,
    })
}

fn desktop_auto_fight_request(
    app_root: &Path,
    strategy_name: Option<&str>,
    team_names: Option<&str>,
) -> IndependentTaskExecutionRequest {
    let strategy_name = strategy_name
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let team_names = team_names.map(str::trim).filter(|value| !value.is_empty());
    if let Some(team_names) = team_names {
        let mut param = AutoFightParam::new(strategy_name);
        param.team_names = team_names.to_string();
        IndependentTaskExecutionRequest {
            task_key: "AutoFight".to_string(),
            command: None,
            config: serde_json::to_value(AutoFightExecutionConfig { param }).ok(),
            working_directory: app_root.to_path_buf(),
        }
    } else {
        IndependentTaskExecutionRequest::auto_fight(strategy_name, app_root)
    }
}

#[tauri::command]
fn task_plan_auto_fight(
    app: tauri::AppHandle,
    payload: DesktopAutoFightTaskPayload,
) -> Result<DesktopAutoFightTaskExecution, String> {
    let app_root = app_root(&app)?;
    let strategy_name = payload
        .strategy_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "independent auto-fight plan requested: {}",
            strategy_name.unwrap_or("根据队伍自动选择")
        ),
    );
    let request =
        desktop_auto_fight_request(&app_root, strategy_name, payload.team_names.as_deref());
    let execution = execute_independent_task_with_cancel(&request, || false)
        .map_err(|error| error.to_string())?;
    let IndependentTaskExecution::AutoFightPlan(result) = execution.execution else {
        return Err("AutoFight returned a non-fight execution".to_string());
    };
    Ok(DesktopAutoFightTaskExecution {
        task: execution.task_key,
        result,
    })
}

#[tauri::command]
fn task_execute_auto_fight_team_playback(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    payload: DesktopAutoFightTeamPlaybackPayload,
) -> Result<DesktopAutoFightTeamPlaybackExecution, String> {
    let app_root = app_root(&app)?;
    let strategy_name = payload
        .strategy_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "independent auto-fight team playback requested: {}",
            strategy_name.unwrap_or("根据队伍自动选择")
        ),
    );
    let request =
        desktop_auto_fight_request(&app_root, strategy_name, payload.team_names.as_deref());
    let execution = execute_independent_task_with_cancel(&request, || false)
        .map_err(|error| error.to_string())?;
    let IndependentTaskExecution::AutoFightPlan(plan) = execution.execution else {
        return Err("AutoFight returned a non-fight execution".to_string());
    };
    let script_name = plan
        .team_selection
        .script_name
        .as_deref()
        .ok_or_else(|| plan.team_selection.message.clone())?;
    let script = plan
        .script_execution_plans
        .iter()
        .find(|script| script.name == script_name)
        .ok_or_else(|| {
            format!("selected combat script execution plan was not found: {script_name}")
        })?;
    let team_plan = plan.team_plan.as_ref().ok_or_else(|| {
        "auto-fight team playback requires configured or recognized team context".to_string()
    })?;
    let mode = if payload.send_input.unwrap_or(false) {
        CombatCommandPlaybackMode::SendInput
    } else {
        CombatCommandPlaybackMode::PlanOnly
    };
    let cancellation = if matches!(mode, CombatCommandPlaybackMode::SendInput) {
        task_state.script_cancellation.reset();
        Some(task_state.script_cancellation.clone())
    } else {
        None
    };
    let result = execute_team_context_combat_script_inputs(
        script,
        team_plan,
        &plan.team_selection.executable_commands,
        mode,
        cancellation.as_deref(),
    )
    .map_err(|error| error.to_string())?;

    Ok(DesktopAutoFightTeamPlaybackExecution {
        task: execution.task_key,
        result,
    })
}

#[tauri::command]
fn task_probe_auto_fight_finish(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    payload: DesktopAutoFightFinishProbePayload,
) -> Result<DesktopAutoFightFinishProbeExecution, String> {
    let app_root = app_root(&app)?;
    let strategy_name = payload
        .strategy_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "independent auto-fight finish probe requested: {}",
            strategy_name.unwrap_or("根据队伍自动选择")
        ),
    );
    let request =
        desktop_auto_fight_request(&app_root, strategy_name, payload.team_names.as_deref());
    let execution = execute_independent_task_with_cancel(&request, || false)
        .map_err(|error| error.to_string())?;
    let IndependentTaskExecution::AutoFightPlan(plan) = execution.execution else {
        return Err("AutoFight returned a non-fight execution".to_string());
    };

    let config = read_desktop_config(&app, &app_root);
    let mode = if payload.send_input.unwrap_or(false) {
        AutoFightFinishDetectionExecutionMode::SendInput
    } else {
        AutoFightFinishDetectionExecutionMode::PlanOnly
    };
    let cancellation = if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        task_state.script_cancellation.reset();
        Some(task_state.script_cancellation.clone())
    } else {
        None
    };
    let result = execute_auto_fight_finish_detection_live_probe(
        &plan.finish_detection_plan,
        mode,
        cancellation.as_deref(),
        || {
            capture_desktop_game_bgr_image(&config).map_err(|error| {
                bgi_task::TaskError::VisionPlan(format!("desktop capture failed: {error}"))
            })
        },
    )
    .map_err(|error| error.to_string())?;

    Ok(DesktopAutoFightFinishProbeExecution {
        task: execution.task_key,
        result,
    })
}

#[tauri::command]
fn task_detect_auto_fight_active_avatar(
    app: tauri::AppHandle,
) -> Result<DesktopActiveAvatarDetectionExecution, String> {
    let app_root = app_root(&app)?;
    append_desktop_log(
        &app_root,
        "INFO",
        "auto-fight active avatar detection requested",
    );
    let config = read_desktop_config(&app, &app_root);
    let image = capture_desktop_game_bgr_image(&config)?;
    let result = detect_active_combat_avatar_index_from_default_rects_with_arrow(&app_root, &image)
        .map_err(|error| error.to_string())?;
    Ok(DesktopActiveAvatarDetectionExecution {
        task: "AutoFight".to_string(),
        result,
    })
}

#[tauri::command]
fn script_execute_group_project(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    group_name: String,
    project_index: usize,
    settings: Option<serde_json::Value>,
    honor_run_count: Option<bool>,
) -> Result<ScriptGroupExecutionOutcome, String> {
    let app_root = app_root(&app)?;
    let script_cancellation = start_desktop_script_run(
        &task_state,
        format!("ScriptGroupProject:{group_name}[{project_index}]"),
    )?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("script group project execution requested: {group_name}[{project_index}]"),
    );
    let group_path = script_group_file_path(script_group_root(&app_root), &group_name);
    let mut group = read_script_group_file(&group_path).map_err(|error| error.to_string())?;
    if let Some(settings) = settings {
        let project = group.projects.get_mut(project_index).ok_or_else(|| {
            format!("script project index {project_index} not found in {group_name}")
        })?;
        project.js_script_settings_object = Some(settings);
    }
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    let config = read_desktop_config(&app, &app_root);
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let outcome = if honor_run_count.unwrap_or(false) {
            bgi_script_engine::execute_script_group_project_repeated_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                project_index,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                },
            )
        } else {
            bgi_script_engine::execute_script_group_project_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                project_index,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                },
            )
        };
        let mut outcome = outcome.map_err(|error| error.to_string())?;
        dispatch_script_notifications_in_group_outcome(&config, &mut outcome);
        Ok(outcome)
    })();
    finish_desktop_script_run(&task_state);
    result
}

#[tauri::command]
fn script_execute_group(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    group_name: String,
) -> Result<ScriptGroupExecutionOutcome, String> {
    let app_root = app_root(&app)?;
    let script_cancellation =
        start_desktop_script_run(&task_state, format!("ScriptGroup:{group_name}"))?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("script group execution requested: {group_name}"),
    );
    let group_path = script_group_file_path(script_group_root(&app_root), &group_name);
    let group = read_script_group_file(&group_path).map_err(|error| error.to_string())?;
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    let config = read_desktop_config(&app, &app_root);
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let mut outcome =
            bgi_script_engine::execute_script_group_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                },
            );
        dispatch_script_notifications_in_group_outcome(&config, &mut outcome);
        Ok(outcome)
    })();
    finish_desktop_script_run(&task_state);
    result
}

#[tauri::command]
fn script_execute_group_from_project(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    group_name: String,
    project_index: usize,
) -> Result<ScriptGroupExecutionOutcome, String> {
    let app_root = app_root(&app)?;
    let script_cancellation = start_desktop_script_run(
        &task_state,
        format!("ScriptGroupResume:{group_name}[{project_index}]"),
    )?;
    append_desktop_log(
        &app_root,
        "INFO",
        &format!("script group resume execution requested: {group_name}[{project_index}]"),
    );
    let group_path = script_group_file_path(script_group_root(&app_root), &group_name);
    let group = read_script_group_file(&group_path).map_err(|error| error.to_string())?;
    let project = group
        .projects
        .get(project_index)
        .ok_or_else(|| format!("script project index {project_index} not found in {group_name}"))?;
    let resume_pointer = ScriptGroupResumePointer {
        group_name: group.name.clone(),
        project_index: project.index,
        folder_name: project.folder_name.clone(),
        project_name: project.name.clone(),
    };
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    let config = read_desktop_config(&app, &app_root);
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let mut outcome =
            bgi_script_engine::execute_script_group_from_resume_with_task_dispatcher_hooks_and_cancellation(
                &roots,
                &group,
                &resume_pointer,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                },
            );
        dispatch_script_notifications_in_group_outcome(&config, &mut outcome);
        Ok(outcome)
    })();
    finish_desktop_script_run(&task_state);
    result
}

fn script_group_file_summary(file: ScriptGroupFile) -> ScriptGroupDesktopSummary {
    script_group_summary(file.path, file.group)
}

fn script_group_summary(path: PathBuf, group: ScriptGroup) -> ScriptGroupDesktopSummary {
    let projects = group
        .projects
        .iter()
        .enumerate()
        .map(|(index, project)| ScriptGroupProjectDesktopSummary {
            index,
            project_index: project.index,
            name: project.name.clone(),
            folder_name: project.folder_name.clone(),
            project_type: script_project_type_label(&project.project_type).to_string(),
            status: script_project_status_label(&project.status).to_string(),
            schedule: project.schedule.clone(),
            run_num: project.run_num,
            allow_js_notification: project.allow_js_notification,
            allow_js_http_hash: project.allow_js_http_hash.clone(),
            has_settings: project.js_script_settings_object.is_some(),
        })
        .collect::<Vec<_>>();

    ScriptGroupDesktopSummary {
        name: group.name,
        index: group.index,
        path: path.display().to_string(),
        project_count: projects.len(),
        projects,
    }
}

fn script_project_type_label(project_type: &ScriptProjectType) -> &'static str {
    match project_type {
        ScriptProjectType::Javascript => "Javascript",
        ScriptProjectType::KeyMouse => "KeyMouse",
        ScriptProjectType::Pathing => "Pathing",
        ScriptProjectType::Shell => "Shell",
    }
}

fn script_project_status_label(status: &ScriptProjectStatus) -> &'static str {
    match status {
        ScriptProjectStatus::Enabled => "Enabled",
        ScriptProjectStatus::Disabled => "Disabled",
    }
}

impl ScriptGroupProjectPatchPayload {
    fn into_patch(self) -> Result<ScriptGroupProjectPatch, String> {
        Ok(ScriptGroupProjectPatch {
            name: self.name,
            folder_name: self.folder_name,
            project_type: parse_project_type(self.project_type)?,
            status: parse_project_status(self.status)?,
            schedule: self.schedule,
            run_num: self.run_num,
            allow_js_notification: self.allow_js_notification,
            allow_js_http_hash: self.allow_js_http_hash,
        })
    }
}

fn parse_project_type(value: Option<String>) -> Result<Option<ScriptProjectType>, String> {
    value
        .map(|value| match value.trim() {
            "" => Ok(None),
            "Javascript" => Ok(Some(ScriptProjectType::Javascript)),
            "KeyMouse" => Ok(Some(ScriptProjectType::KeyMouse)),
            "Pathing" => Ok(Some(ScriptProjectType::Pathing)),
            "Shell" => Ok(Some(ScriptProjectType::Shell)),
            other => Err(format!("unsupported project type: {other}")),
        })
        .transpose()
        .map(Option::flatten)
}

fn parse_project_status(value: Option<String>) -> Result<Option<ScriptProjectStatus>, String> {
    value
        .map(|value| match value.trim() {
            "" => Ok(None),
            "Enabled" => Ok(Some(ScriptProjectStatus::Enabled)),
            "Disabled" => Ok(Some(ScriptProjectStatus::Disabled)),
            other => Err(format!("unsupported project status: {other}")),
        })
        .transpose()
        .map(Option::flatten)
}

fn read_script_project_summary(
    app_root: &Path,
    folder_name: &str,
) -> Result<AvailableJsProjectDesktopSummary, String> {
    available_js_script_projects(js_script_root(app_root))
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|project| project.folder_name == folder_name)
        .map(|project| AvailableJsProjectDesktopSummary {
            folder_name: project.folder_name,
            name: project.name,
            version: project.version,
            description: project.description,
            settings_ui: project.settings_ui,
            has_settings_ui: project.has_settings_ui,
        })
        .ok_or_else(|| format!("JavaScript project folder {folder_name:?} was not found"))
}

fn import_paths_for_repo(
    app_root: &Path,
    repo_path: &str,
    paths: Vec<String>,
    git_repo: bool,
) -> Result<ScriptRepoDesktopImportResult, String> {
    let center_repo_path = PathBuf::from(repo_path);
    let source_repo_path = import_source_repo_path(&center_repo_path, git_repo);
    let bridge_paths = script_repo_bridge_paths(app_root, &center_repo_path, None)
        .map_err(|error| error.to_string())?;
    let existing = read_subscription_file(&bridge_paths.subscription_file_path)
        .map_err(|error| error.to_string())?;
    let mut plan = script_import_plan(
        app_root,
        source_repo_path,
        &AppConfig::default().script_config,
        &BTreeMap::new(),
        paths,
        existing,
        &BTreeMap::new(),
    );
    plan.subscription_file_path = bridge_paths.subscription_file_path;
    let result = if git_repo {
        let mut runner = SystemGitRunner::default();
        execute_repo_import_with_git(&plan, Some(&mut runner)).map_err(|error| error.to_string())?
    } else {
        execute_file_repo_import(&plan).map_err(|error| error.to_string())?
    };

    Ok(ScriptRepoDesktopImportResult {
        imported_targets: result.imported_targets.len(),
        skipped_unknown_paths: result.skipped_unknown_paths,
        subscriptions: result.subscriptions,
        dependency_files_copied: result.dependency_files_copied.len(),
        git_checkouts: result.git_checkouts.len(),
    })
}

fn script_group_root(app_root: &Path) -> PathBuf {
    app_root.join("User").join("ScriptGroup")
}

fn js_script_root(app_root: &Path) -> PathBuf {
    app_root.join("User").join("JsScript")
}

fn key_mouse_script_root(app_root: &Path) -> PathBuf {
    app_root.join("User").join("KeyMouseScript")
}

fn pathing_script_root(app_root: &Path) -> PathBuf {
    app_root.join("User").join("AutoPathing")
}

fn read_desktop_config(app: &tauri::AppHandle, app_root: &Path) -> AppConfig {
    let root_config = config_path(app_root);
    if let Ok(config) = read_config(&root_config) {
        return config;
    }

    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("config.json"))
        .and_then(|path| read_config(path).ok())
        .unwrap_or_default()
}

fn find_desktop_game_window(config: &AppConfig) -> Option<GameWindowMatch> {
    let mut search_config = GameWindowSearchConfig::default();
    if !config.genshin_start_config.install_path.trim().is_empty() {
        search_config = search_config.with_install_path(&config.genshin_start_config.install_path);
    }

    find_game_window(&search_config).ok().flatten()
}

fn configure_desktop_script_roots(
    game_window: Option<&GameWindowMatch>,
    roots: &mut ScriptGroupExecutionRoots,
) {
    if let Some(window) = game_window {
        roots.input_window_handle = Some(window.handle.0);
    }
}

fn configure_desktop_script_host(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    host: &mut ScriptHostRuntimeConfig,
) {
    let Some(window) = game_window else {
        return;
    };

    host.input_window_handle = Some(window.handle.0);
    apply_game_window_metrics(window, host);

    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };

    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return;
    }

    let Ok(source) = DesktopGameCaptureFrameSource::new(window.handle, settings) else {
        return;
    };
    host.capture_frame_source = Some(Arc::new(source));
}

fn capture_notification_screenshot(config: &AppConfig) -> Option<NotificationImage> {
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return None;
    }
    let window = find_desktop_game_window(config)?;
    let source = DesktopGameCaptureFrameSource::new(window.handle, settings).ok()?;
    let frame = source.capture_frame().ok()?;
    let png = encode_capture_frame_png(frame).ok()?;
    Some(NotificationImage::png(png))
}

fn capture_desktop_game_bgr_image(config: &AppConfig) -> Result<BgrImage, String> {
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("desktop game capture requires the BitBlt capture backend".to_string());
    }
    let window = find_desktop_game_window(config)
        .ok_or_else(|| "desktop game capture could not find the game window".to_string())?;
    let source = DesktopGameCaptureFrameSource::new(window.handle, settings)
        .map_err(|error| error.to_string())?;
    let frame = source.capture_frame().map_err(|error| error.to_string())?;
    bgr_image_from_desktop_capture_frame(frame).map_err(|error| error.to_string())
}

fn encode_capture_frame_png(
    frame: CaptureFrame,
) -> Result<Vec<u8>, bgi_script::ScriptHostRuntimeError> {
    if frame.pixel_format != PixelFormat::Bgr8 {
        return Err(ScriptHostRuntimeError::UnsupportedCaptureFrame);
    }
    let row_bytes = frame.row_bytes();
    let mut rgb = Vec::with_capacity(row_bytes * frame.size.height as usize);
    for row in 0..frame.size.height as usize {
        let start = row * frame.stride;
        let end = start + row_bytes;
        for pixel in frame.pixels[start..end].chunks_exact(3) {
            rgb.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
    }
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &rgb,
            frame.size.width,
            frame.size.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|source| {
            ScriptHostRuntimeError::Vision(bgi_vision::VisionError::ImageDecode {
                path: None,
                source,
            })
        })?;
    Ok(png)
}

fn bgr_image_from_desktop_capture_frame(
    frame: CaptureFrame,
) -> Result<BgrImage, bgi_script::ScriptHostRuntimeError> {
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

#[derive(Debug, Clone)]
struct ScriptNotificationProviderPayload {
    call_index: usize,
    event: String,
    result: NotificationEventResult,
    message: String,
    timestamp_ms: u64,
}

fn dispatch_script_notifications_in_group_outcome(
    config: &AppConfig,
    outcome: &mut ScriptGroupExecutionOutcome,
) {
    for step in &mut outcome.steps {
        if let Some(javascript) = &mut step.javascript {
            dispatch_script_notifications_in_javascript_outcome(config, javascript);
        }
    }
}

fn dispatch_script_notifications_in_javascript_outcome(
    config: &AppConfig,
    outcome: &mut JavaScriptExecutionOutcome,
) {
    let notifications = outcome
        .host_calls
        .iter()
        .enumerate()
        .filter_map(|(call_index, call)| {
            script_notification_payload_from_host_call(call_index, call)
        })
        .collect::<Vec<_>>();

    if notifications.is_empty() {
        return;
    }

    let screenshot = if config.notification_config.include_screen_shot {
        capture_notification_screenshot(config)
    } else {
        None
    };
    let mut client = match DesktopNotificationHttpClient::new() {
        Ok(client) => client,
        Err(error) => {
            for notification in notifications {
                annotate_script_notification_dispatch_error(
                    outcome,
                    notification.call_index,
                    format!("notification HTTP client setup failed: {error}"),
                );
            }
            return;
        }
    };
    let mut web_socket_client = DesktopNotificationWebSocketClient;
    let mut email_client = DesktopNotificationEmailClient;
    let mut windows_toast_client =
        DesktopNotificationWindowsToastClient::new(&std::env::temp_dir().join("BetterGI"));

    for notification in notifications {
        let mut payload = NotificationPayload {
            event: notification.event,
            result: notification.result,
            message: Some(notification.message),
            data: None,
            timestamp_ms: Some(notification.timestamp_ms),
            has_screenshot: false,
            screenshot: None,
        };
        if let Some(screenshot) = screenshot.clone() {
            payload = payload.with_screenshot(screenshot);
        }
        let dispatch = execute_notification_dispatch_with_transports(
            &config.notification_config,
            payload,
            &mut client,
            &mut web_socket_client,
            &mut email_client,
            &mut windows_toast_client,
        );
        annotate_script_notification_dispatch(outcome, notification.call_index, dispatch);
    }
}

fn dispatch_script_html_mask_in_javascript_outcome(
    app: &tauri::AppHandle,
    outcome: &mut JavaScriptExecutionOutcome,
) {
    let commands = outcome
        .host_calls
        .iter()
        .enumerate()
        .filter_map(|(call_index, call)| script_html_mask_command_from_host_call(call_index, call))
        .collect::<Vec<_>>();

    for (call_index, command) in commands {
        let dispatch = dispatch_desktop_html_mask_command(app, &command);
        annotate_script_host_call_value(outcome, call_index, "desktop_html_mask", dispatch);
    }
}

fn script_html_mask_command_from_host_call(
    call_index: usize,
    call: &bgi_script_engine::ExecutedHostCall,
) -> Option<(usize, HtmlMaskCommand)> {
    if call.target != ScriptHostTarget::HtmlMask {
        return None;
    }
    let command = serde_json::from_value::<HtmlMaskCommand>(call.result.clone()).ok()?;
    Some((call_index, command))
}

#[tauri::command]
fn html_mask_receive_from_webview(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, HtmlMaskBridgeState>,
    payload: HtmlMaskWebviewPayload,
) -> Result<(), String> {
    let label = window.label().to_string();
    let window_id = html_mask_window_id_for_label(&state, &label);
    let request_id = payload.request_id.filter(|value| !value.is_empty());
    let message = HtmlMaskMessage {
        url: payload.url.unwrap_or_default(),
        data: payload.data,
        request_id,
    };
    record_html_mask_from_webview_message(&state, &window_id, message);
    Ok(())
}

#[tauri::command]
fn html_mask_bridge_snapshot(
    state: tauri::State<'_, HtmlMaskBridgeState>,
) -> DesktopHtmlMaskBridgeSnapshot {
    DesktopHtmlMaskBridgeSnapshot {
        from_html: state.from_html.lock().unwrap().clone(),
        pending_requests: state.pending_requests.lock().unwrap().clone(),
        window_labels: state.window_labels.lock().unwrap().clone(),
        windows: state.windows.lock().unwrap().clone(),
    }
}

fn dispatch_desktop_html_mask_command(
    app: &tauri::AppHandle,
    command: &HtmlMaskCommand,
) -> DesktopHtmlMaskDispatch {
    match command {
        HtmlMaskCommand::Show(plan) => match show_desktop_html_mask_window(app, plan) {
            Ok(label) => {
                register_html_mask_window(app, &label, plan);
                DesktopHtmlMaskDispatch {
                    action: "show".to_string(),
                    window_id: Some(plan.window_id.clone()),
                    window_label: Some(label),
                    dispatched: true,
                    message: "window shown".to_string(),
                }
            }
            Err(error) => DesktopHtmlMaskDispatch {
                action: "show".to_string(),
                window_id: Some(plan.window_id.clone()),
                window_label: Some(html_mask_window_label(&plan.window_id)),
                dispatched: false,
                message: error,
            },
        },
        HtmlMaskCommand::Close { window_id } => {
            let label = html_mask_window_label(window_id);
            let dispatched = app
                .get_webview_window(&label)
                .map(|window| window.close().is_ok())
                .unwrap_or(false);
            if dispatched {
                unregister_html_mask_window(app, &label, window_id);
            }
            DesktopHtmlMaskDispatch {
                action: "close".to_string(),
                window_id: Some(window_id.clone()),
                window_label: Some(label),
                dispatched,
                message: if dispatched {
                    "window closed".to_string()
                } else {
                    "window not found".to_string()
                },
            }
        }
        HtmlMaskCommand::CloseAll { window_ids } => {
            let mut closed = 0usize;
            for window_id in window_ids {
                let label = html_mask_window_label(window_id);
                if let Some(window) = app.get_webview_window(&label) {
                    if window.close().is_ok() {
                        unregister_html_mask_window(app, &label, window_id);
                        closed += 1;
                    }
                }
            }
            DesktopHtmlMaskDispatch {
                action: "closeAll".to_string(),
                window_id: None,
                window_label: None,
                dispatched: closed > 0 || window_ids.is_empty(),
                message: format!("{closed} window(s) closed"),
            }
        }
        HtmlMaskCommand::SetClickThrough { window_id, enabled } => {
            let label = html_mask_window_label(window_id);
            let result = app
                .get_webview_window(&label)
                .ok_or_else(|| "window not found".to_string())
                .and_then(|window| {
                    window
                        .set_ignore_cursor_events(*enabled)
                        .map_err(|error| error.to_string())
                });
            DesktopHtmlMaskDispatch {
                action: "setClickThrough".to_string(),
                window_id: Some(window_id.clone()),
                window_label: Some(label),
                dispatched: result.is_ok(),
                message: result
                    .map(|_| format!("click-through set to {enabled}"))
                    .unwrap_or_else(|error| error),
            }
        }
        HtmlMaskCommand::Send { window_id, message } => {
            dispatch_html_mask_message_to_window(app, window_id, message, "send")
        }
        HtmlMaskCommand::Request {
            window_id,
            message,
            timeout_ms,
        } => {
            register_html_mask_pending_request(app, window_id, message);
            let mut dispatch =
                dispatch_html_mask_message_to_window(app, window_id, message, "request");
            if dispatch.dispatched {
                dispatch.message = format!(
                    "request message dispatched; response queued for future script bridge (timeout {timeout_ms} ms)"
                );
            }
            dispatch
        }
    }
}

fn show_desktop_html_mask_window(
    app: &tauri::AppHandle,
    plan: &HtmlMaskWindowPlan,
) -> Result<String, String> {
    let label = html_mask_window_label(&plan.window_id);
    let game_window = html_mask_current_game_window(app);
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|error| error.to_string())?;
        if let Some(game_window) = game_window.as_ref() {
            let _ = apply_html_mask_window_bounds(&window, game_window);
        }
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(label);
    }

    let url = html_mask_webview_url(&plan.final_url)?;
    let builder = WebviewWindowBuilder::new(app, &label, url)
        .title(format!("BetterGI HtmlMask {}", plan.window_id))
        .initialization_script(HTML_MASK_BRIDGE_SCRIPT)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .inner_size(960.0, 540.0);
    let builder = if let Some(game_window) = game_window.as_ref() {
        html_mask_window_builder_with_game_bounds(builder, game_window)
    } else {
        builder
    };
    let window = builder.build().map_err(|error| error.to_string())?;
    let cleanup_app = app.clone();
    let cleanup_label = label.clone();
    let cleanup_window_id = plan.window_id.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            unregister_html_mask_window(&cleanup_app, &cleanup_label, &cleanup_window_id);
        }
    });
    if plan.click_through {
        let _ = window.set_ignore_cursor_events(true);
    }
    Ok(label)
}

fn dispatch_html_mask_message_to_window(
    app: &tauri::AppHandle,
    window_id: &str,
    message: &HtmlMaskMessage,
    action: &str,
) -> DesktopHtmlMaskDispatch {
    let label = html_mask_window_label(window_id);
    let result = app
        .get_webview_window(&label)
        .ok_or_else(|| "window not found".to_string())
        .and_then(|window| {
            let script = html_mask_message_dispatch_script(message)?;
            window.eval(script).map_err(|error| error.to_string())
        });
    DesktopHtmlMaskDispatch {
        action: action.to_string(),
        window_id: Some(window_id.to_string()),
        window_label: Some(label),
        dispatched: result.is_ok(),
        message: result
            .map(|_| "message dispatched to HtmlMask WebView".to_string())
            .unwrap_or_else(|error| error),
    }
}

fn html_mask_window_builder_with_game_bounds<'a>(
    builder: WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>>,
    game_window: &GameWindowMatch,
) -> WebviewWindowBuilder<'a, tauri::Wry, tauri::AppHandle<tauri::Wry>> {
    let Some(bounds) = html_mask_game_window_bounds(game_window) else {
        return builder;
    };
    builder
        .position(bounds.left as f64, bounds.top as f64)
        .inner_size(bounds.width() as f64, bounds.height() as f64)
}

fn apply_html_mask_window_bounds(
    window: &tauri::WebviewWindow,
    game_window: &GameWindowMatch,
) -> Result<(), String> {
    let Some(bounds) = html_mask_game_window_bounds(game_window) else {
        return Ok(());
    };
    window
        .set_position(PhysicalPosition::new(bounds.left, bounds.top))
        .map_err(|error| error.to_string())?;
    window
        .set_size(PhysicalSize::new(
            bounds.width() as u32,
            bounds.height() as u32,
        ))
        .map_err(|error| error.to_string())
}

fn html_mask_game_window_bounds(game_window: &GameWindowMatch) -> Option<bgi_capture::WindowRect> {
    let bounds = game_window.metrics?.capture_area;
    if bounds.is_empty() {
        None
    } else {
        Some(bounds)
    }
}

fn html_mask_message_dispatch_script(message: &HtmlMaskMessage) -> Result<String, String> {
    let raw = serde_json::to_string(message)
        .map_err(|error| format!("failed to serialize HtmlMask message: {error}"))?;
    let raw_json = serde_json::to_string(&raw)
        .map_err(|error| format!("failed to quote HtmlMask message: {error}"))?;
    Ok(format!(
        r#"(function () {{
  if (window.htmlMask && typeof window.htmlMask._dispatch === "function") {{
    window.htmlMask._dispatch({raw_json});
  }} else {{
    const raw = {raw_json};
    let msg = raw;
    try {{ msg = JSON.parse(raw); }} catch (_) {{}}
    window.dispatchEvent(new CustomEvent("bettergi:html-mask-message", {{ detail: msg }}));
    if (typeof window.onBetterGIHtmlMaskMessage === "function") {{
      window.onBetterGIHtmlMaskMessage(msg);
    }}
  }}
}})();"#
    ))
}

fn register_html_mask_window(app: &tauri::AppHandle, label: &str, plan: &HtmlMaskWindowPlan) {
    let Some(state) = app.try_state::<HtmlMaskBridgeState>() else {
        return;
    };
    let window_id = &plan.window_id;
    state
        .window_labels
        .lock()
        .unwrap()
        .insert(label.to_string(), window_id.to_string());
    state
        .windows
        .lock()
        .unwrap()
        .insert(window_id.to_string(), plan.clone());
    state
        .from_html
        .lock()
        .unwrap()
        .entry(window_id.to_string())
        .or_default();
}

fn unregister_html_mask_window(app: &tauri::AppHandle, label: &str, window_id: &str) {
    let Some(state) = app.try_state::<HtmlMaskBridgeState>() else {
        return;
    };
    state.window_labels.lock().unwrap().remove(label);
    state.windows.lock().unwrap().remove(window_id);
    state.from_html.lock().unwrap().remove(window_id);
    state
        .pending_requests
        .lock()
        .unwrap()
        .retain(|_, pending_window_id| pending_window_id != window_id);
}

fn register_html_mask_pending_request(
    app: &tauri::AppHandle,
    window_id: &str,
    message: &HtmlMaskMessage,
) {
    let Some(request_id) = message.request_id.as_ref() else {
        return;
    };
    let Some(state) = app.try_state::<HtmlMaskBridgeState>() else {
        return;
    };
    state
        .pending_requests
        .lock()
        .unwrap()
        .insert(request_id.clone(), window_id.to_string());
}

fn record_html_mask_from_webview_message(
    state: &HtmlMaskBridgeState,
    window_id: &str,
    message: HtmlMaskMessage,
) {
    if let Some(request_id) = message.request_id.as_ref() {
        state.pending_requests.lock().unwrap().remove(request_id);
    }
    let record = DesktopHtmlMaskBridgeMessage {
        window_id: window_id.to_string(),
        message,
        timestamp_ms: Local::now().timestamp_millis(),
    };
    state
        .from_html
        .lock()
        .unwrap()
        .entry(window_id.to_string())
        .or_default()
        .push(record);
}

fn take_html_mask_initial_state_for_script(app: &tauri::AppHandle) -> HtmlMaskInitialState {
    let Some(state) = app.try_state::<HtmlMaskBridgeState>() else {
        return HtmlMaskInitialState::default();
    };
    let windows = state
        .windows
        .lock()
        .map(|windows| windows.values().cloned().collect())
        .unwrap_or_default();
    let from_html = state
        .from_html
        .lock()
        .map(|mut queues| {
            queues
                .iter_mut()
                .flat_map(|(window_id, records)| {
                    records
                        .drain(..)
                        .map(|record| (window_id.clone(), record.message))
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();
    HtmlMaskInitialState { windows, from_html }
}

fn restore_html_mask_from_script_outcome(
    app: &tauri::AppHandle,
    outcome: &JavaScriptExecutionOutcome,
) {
    let Some(state) = app.try_state::<HtmlMaskBridgeState>() else {
        return;
    };
    for (window_id, message) in &outcome.html_mask_from_html {
        record_html_mask_from_webview_message(&state, window_id, message.clone());
    }
}

fn html_mask_window_id_for_label(state: &HtmlMaskBridgeState, label: &str) -> String {
    state
        .window_labels
        .lock()
        .unwrap()
        .get(label)
        .cloned()
        .unwrap_or_else(|| {
            label
                .strip_prefix(HTML_MASK_WINDOW_PREFIX)
                .unwrap_or(label)
                .to_string()
        })
}

fn start_html_mask_position_sync(app: &tauri::App) {
    let handle = app.handle().clone();
    append_desktop_app_log(&handle, "INFO", "HtmlMask position sync started");
    std::thread::spawn(move || loop {
        std::thread::sleep(HTML_MASK_POSITION_SYNC_INTERVAL);
        let Some(state) = handle.try_state::<HtmlMaskBridgeState>() else {
            continue;
        };
        let windows = html_mask_bridge_window_labels(&state);
        if windows.is_empty() {
            continue;
        }

        let Some(game_window) = html_mask_current_game_window(&handle) else {
            continue;
        };
        for (label, window_id) in windows {
            match handle.get_webview_window(&label) {
                Some(window) => {
                    let _ = apply_html_mask_window_bounds(&window, &game_window);
                }
                None => unregister_html_mask_window(&handle, &label, &window_id),
            }
        }
    });
}

fn html_mask_bridge_window_labels(state: &HtmlMaskBridgeState) -> Vec<(String, String)> {
    state
        .window_labels
        .lock()
        .map(|labels| {
            labels
                .iter()
                .map(|(label, window_id)| (label.clone(), window_id.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn html_mask_current_game_window(app: &tauri::AppHandle) -> Option<GameWindowMatch> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|app_root| read_desktop_config(app, &app_root))
        .and_then(|config| find_desktop_game_window(&config))
}

fn html_mask_webview_url(value: &str) -> Result<WebviewUrl, String> {
    let url = tauri::Url::parse(value).map_err(|error| format!("invalid HtmlMask URL: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(WebviewUrl::External(url)),
        "file" => Ok(WebviewUrl::CustomProtocol(url)),
        scheme => Err(format!("unsupported HtmlMask URL scheme: {scheme}")),
    }
}

fn html_mask_window_label(window_id: &str) -> String {
    let mut label = String::from(HTML_MASK_WINDOW_PREFIX);
    for ch in window_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            label.push(ch);
        } else {
            label.push('-');
        }
    }
    label
}

fn script_notification_payload_from_host_call(
    call_index: usize,
    call: &bgi_script_engine::ExecutedHostCall,
) -> Option<ScriptNotificationProviderPayload> {
    if call.target != ScriptHostTarget::Notification {
        return None;
    }

    let root = call.result.as_object()?;
    if !root.get("dispatched")?.as_bool()? {
        return None;
    }
    let delivery = root.get("delivery")?.as_object()?;
    let event = delivery.get("event_code")?.as_str()?.trim();
    if event.is_empty() {
        return None;
    }
    let message = delivery
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let timestamp_ms = delivery
        .get("timestamp_ms")
        .and_then(Value::as_u64)
        .or_else(|| root.get("record")?.get("timestamp_ms")?.as_u64())?;

    Some(ScriptNotificationProviderPayload {
        call_index,
        event: event.to_string(),
        result: notification_event_result_from_script(
            delivery
                .get("result")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        message,
        timestamp_ms,
    })
}

fn notification_event_result_from_script(result: &str) -> NotificationEventResult {
    match result.trim().to_ascii_lowercase().as_str() {
        "fail" | "failed" | "error" => NotificationEventResult::Fail,
        "partial" | "partial_success" | "partialsuccess" => NotificationEventResult::PartialSuccess,
        _ => NotificationEventResult::Success,
    }
}

fn annotate_script_notification_dispatch(
    outcome: &mut JavaScriptExecutionOutcome,
    call_index: usize,
    dispatch: NotificationDispatchExecution,
) {
    match serde_json::to_value(dispatch) {
        Ok(value) => {
            annotate_script_host_call_json_value(outcome, call_index, "app_dispatch", value)
        }
        Err(error) => annotate_script_notification_dispatch_error(
            outcome,
            call_index,
            format!("notification dispatch serialization failed: {error}"),
        ),
    }
}

fn annotate_script_notification_dispatch_error(
    outcome: &mut JavaScriptExecutionOutcome,
    call_index: usize,
    error: String,
) {
    annotate_script_host_call_json_value(
        outcome,
        call_index,
        "app_dispatch_error",
        Value::String(error),
    );
}

fn annotate_script_host_call_value<T: Serialize>(
    outcome: &mut JavaScriptExecutionOutcome,
    call_index: usize,
    key: &str,
    value: T,
) {
    if let Ok(value) = serde_json::to_value(value) {
        annotate_script_host_call_json_value(outcome, call_index, key, value);
    }
}

fn annotate_script_host_call_json_value(
    outcome: &mut JavaScriptExecutionOutcome,
    call_index: usize,
    key: &str,
    value: Value,
) {
    let Some(call) = outcome.host_calls.get_mut(call_index) else {
        return;
    };
    let Some(result) = call.result.as_object_mut() else {
        return;
    };
    result.insert(key.to_string(), value);
}

fn apply_game_window_metrics(window: &GameWindowMatch, host: &mut ScriptHostRuntimeConfig) {
    let Some(metrics) = window.metrics else {
        return;
    };
    let capture_area = metrics.capture_area;
    host.capture_area = bgi_script::GameCaptureArea {
        x: capture_area.left,
        y: capture_area.top,
        width: metrics.client_width,
        height: metrics.client_height,
    };

    if metrics.client_width.saturating_mul(9) == metrics.client_height.saturating_mul(16) {
        host.initial_game_metrics = Some(bgi_script::GameMetrics {
            width: metrics.client_width,
            height: metrics.client_height,
            dpi: 1.0,
        });
    }
}

fn native_capture_mode(mode: &bgi_core::CaptureMode) -> NativeCaptureMode {
    match mode {
        bgi_core::CaptureMode::BitBlt => NativeCaptureMode::BitBlt,
        bgi_core::CaptureMode::DwmGetDxSharedSurface => NativeCaptureMode::DwmGetDxSharedSurface,
        bgi_core::CaptureMode::WindowsGraphicsCapture => NativeCaptureMode::WindowsGraphicsCapture,
        bgi_core::CaptureMode::WindowsGraphicsCaptureHdr => {
            NativeCaptureMode::WindowsGraphicsCaptureHdr
        }
    }
}

fn fetch_latest_update_version(
    client: &reqwest::blocking::Client,
    request: &UpdateRequestPlan,
) -> Result<Option<String>, String> {
    match request.channel {
        UpdateChannel::Stable => {
            let notice = client
                .get(&request.url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|error| error.to_string())?
                .text()
                .map_err(|error| error.to_string())?;
            let notice = serde_json::from_str::<Notice>(&notice)
                .map_err(|error| format!("failed to parse update notice: {error}"))?;
            Ok(latest_version_from_notice(
                &notice,
                desktop_update_gray_bucket(),
            ))
        }
        UpdateChannel::Alpha => {
            let response = client
                .get(update_request_url(request))
                .send()
                .map_err(|error| error.to_string())?;
            let text = response.text().map_err(|error| error.to_string())?;
            let latest_response = serde_json::from_str::<MirrorChyanLatestResponse>(&text)
                .map_err(|error| format!("failed to parse MirrorChyan latest response: {error}"))?;
            match mirror_chyan_latest_outcome(Some(&latest_response)) {
                MirrorChyanLatestOutcome::Version(version) => Ok(Some(version)),
                MirrorChyanLatestOutcome::Empty => Ok(None),
                MirrorChyanLatestOutcome::Warning { message, .. }
                | MirrorChyanLatestOutcome::Severe { message, .. } => Err(message),
            }
        }
    }
}

fn desktop_update_check(
    app: &tauri::AppHandle,
    trigger: UpdateTrigger,
    channel: UpdateChannel,
    config: Option<AppConfig>,
) -> Result<DesktopUpdateCheckResult, String> {
    let app_root = app_root(app)?;
    let config = config.unwrap_or_else(|| read_desktop_config(app, &app_root));
    let option = UpdateOption { trigger, channel };
    let request = update_request_plan(option);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("BetterGI Rust")
        .build()
        .map_err(|error| error.to_string())?;
    let latest_version = fetch_latest_update_version(&client, &request)?;
    let mirror_outcome = (channel == UpdateChannel::Alpha).then(|| {
        latest_version
            .as_ref()
            .map(|version| MirrorChyanLatestOutcome::Version(version.clone()))
            .unwrap_or(MirrorChyanLatestOutcome::Empty)
    });
    let app_version = app.package_info().version.to_string();
    let decision = update_decision(
        option,
        &app_version,
        Some(&config.not_show_new_version_notice_end_version),
        latest_version.as_deref(),
    );
    let release_notes = if matches!(decision.action, UpdateDecisionAction::OpenUpdateWindow)
        && channel == UpdateChannel::Stable
    {
        fetch_stable_release_notes(&client).ok()
    } else {
        None
    };
    let updater_path = updater_exe_path(&app_root);

    Ok(DesktopUpdateCheckResult {
        trigger,
        app_version,
        channel,
        request,
        latest_version,
        decision,
        mirror_outcome,
        release_notes,
        download_page_url: update_download_page_url(channel).to_string(),
        updater_exists: updater_path.exists(),
        updater_path: updater_path.display().to_string(),
        updater_options: updater_launch_options(channel),
        ignored_version: config.not_show_new_version_notice_end_version,
    })
}

fn start_background_update_check(app: &tauri::App) {
    let handle = app.handle().clone();
    append_desktop_app_log(&handle, "INFO", "background update check scheduled");
    std::thread::spawn(move || {
        let Some(state) = handle.try_state::<BackgroundUpdateState>() else {
            return;
        };
        if let Ok(mut guard) = state.state.lock() {
            guard.running = true;
            guard.last_error = None;
            let _ = handle.emit(BACKGROUND_UPDATE_EVENT, guard.clone());
        }

        let result =
            desktop_update_check(&handle, UpdateTrigger::Auto, UpdateChannel::Stable, None);
        let final_state = match state.state.lock() {
            Ok(mut guard) => {
                guard.running = false;
                match result {
                    Ok(result) => {
                        append_desktop_app_log(
                            &handle,
                            "INFO",
                            &format!(
                                "background update check completed: {:?}",
                                result.decision.action
                            ),
                        );
                        guard.last_result = Some(result);
                        guard.last_error = None;
                    }
                    Err(error) => {
                        append_desktop_app_log(
                            &handle,
                            "WARN",
                            &format!("background update check failed: {error}"),
                        );
                        guard.last_error = Some(error);
                    }
                }
                guard.clone()
            }
            Err(_) => return,
        };
        let _ = handle.emit(BACKGROUND_UPDATE_EVENT, final_state);
    });
}

fn fetch_stable_release_notes(
    client: &reqwest::blocking::Client,
) -> Result<DesktopReleaseNotes, String> {
    let request = bgi_core::stable_release_notes_request_plan();
    let mut builder = client.get(&request.url);
    if let Some(user_agent) = request.user_agent {
        builder = builder.header(reqwest::header::USER_AGENT, user_agent);
    }
    let value = builder
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())?;
    let value = serde_json::from_str::<Value>(&value)
        .map_err(|error| format!("failed to parse GitHub release response: {error}"))?;

    Ok(DesktopReleaseNotes {
        name: value
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        body: value
            .get("body")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        html_url: value
            .get("html_url")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn live_refresh_code_url(code_version: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!(
        "{REDEEM_CODE_LIVE_REFRESH_CODE_URL}?version={}&time={timestamp}",
        code_version.trim()
    )
}

fn update_request_url(request: &UpdateRequestPlan) -> String {
    if request.query.is_empty() {
        return request.url.clone();
    }
    let query = request
        .query
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}?{}", request.url, query)
}

fn desktop_update_gray_bucket() -> u32 {
    std::env::var("COMPUTERNAME")
        .ok()
        .or_else(|| std::env::var("USERNAME").ok())
        .map(|value| stable_hash_mod10(&value))
        .unwrap_or(0)
}

fn stable_hash_mod10(value: &str) -> u32 {
    value.bytes().fold(0u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(u32::from(byte))
    }) % 10
}

fn update_channel_from_str(value: &str) -> Result<UpdateChannel, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "stable" => Ok(UpdateChannel::Stable),
        "alpha" => Ok(UpdateChannel::Alpha),
        other => Err(format!("unknown update channel: {other}")),
    }
}

fn notification_provider_kind_from_str(value: &str) -> Result<NotificationProviderKind, String> {
    match value.trim() {
        "Webhook" => Ok(NotificationProviderKind::Webhook),
        "WindowsUwp" => Ok(NotificationProviderKind::WindowsUwp),
        "Feishu" => Ok(NotificationProviderKind::Feishu),
        "OneBot" => Ok(NotificationProviderKind::OneBot),
        "WorkWeixin" => Ok(NotificationProviderKind::WorkWeixin),
        "WebSocket" => Ok(NotificationProviderKind::WebSocket),
        "Bark" => Ok(NotificationProviderKind::Bark),
        "Email" => Ok(NotificationProviderKind::Email),
        "DingDingWebhook" => Ok(NotificationProviderKind::DingDingWebhook),
        "Telegram" => Ok(NotificationProviderKind::Telegram),
        "Xxtui" => Ok(NotificationProviderKind::Xxtui),
        "DiscordWebhook" => Ok(NotificationProviderKind::DiscordWebhook),
        "ServerChan" => Ok(NotificationProviderKind::ServerChan),
        "Meow" => Ok(NotificationProviderKind::Meow),
        other => Err(format!("unsupported notification provider: {other}")),
    }
}

fn refresh_notification_service_state(
    app: &tauri::AppHandle,
    state: &NotificationServiceState,
) -> Result<DesktopNotificationServiceState, String> {
    let app_root = app_root(app)?;
    let config = read_desktop_config(app, &app_root);
    let enabled_providers = notification_provider_plans(&config.notification_config)
        .into_iter()
        .map(|provider| provider.name.to_string())
        .collect::<Vec<_>>();
    let service_state = DesktopNotificationServiceState {
        initialized: true,
        config_path: config_path(&app_root).display().to_string(),
        provider_count: enabled_providers.len(),
        enabled_providers,
        refreshed_at_ms: Some(current_time_ms()?),
    };
    *state.state.lock().map_err(|error| error.to_string())? = service_state.clone();
    append_desktop_log(
        &app_root,
        "INFO",
        &format!(
            "notification service providers refreshed: {} enabled",
            service_state.provider_count
        ),
    );
    Ok(service_state)
}

fn current_time_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())
        .map(|duration| duration.as_millis() as u64)
}

fn updater_exe_path(app_root: &Path) -> PathBuf {
    app_root.join("BetterGI.update.exe")
}

fn open_url(url: &str) -> Result<(), String> {
    match url {
        DOWNLOAD_PAGE_URL | ALPHA_RELEASES_URL => {}
        _ => return Err(format!("refusing to open untrusted URL: {url}")),
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|error| format!("failed to open {url}: {error}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|error| format!("failed to open {url}: {error}"))?;
        Ok(())
    }
}

fn desktop_log_path(app_root: &Path) -> PathBuf {
    current_desktop_log_path(app_root)
}

fn desktop_log_dir(app_root: &Path) -> PathBuf {
    app_root.join("log")
}

fn current_desktop_log_path(app_root: &Path) -> PathBuf {
    desktop_log_path_for_date(app_root, Local::now().date_naive())
}

fn desktop_log_path_for_date(app_root: &Path, date: NaiveDate) -> PathBuf {
    desktop_log_dir(app_root).join(format!(
        "better-genshin-impact{}.log",
        date.format("%Y%m%d")
    ))
}

fn parse_desktop_log_date(path: &Path) -> Option<NaiveDate> {
    let stem = path.file_stem()?.to_str()?;
    let date = stem.strip_prefix("better-genshin-impact")?;
    if date.len() != 8 || !date.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y%m%d").ok()
}

fn desktop_shell_state_from(app_root: &Path, config: &AppConfig) -> DesktopShellState {
    DesktopShellState {
        exit_to_tray: config.common_config.exit_to_tray,
        tray_enabled: true,
        config_path: config_path(app_root).display().to_string(),
        log_path: desktop_log_path(app_root).display().to_string(),
    }
}

fn append_desktop_log(app_root: &Path, level: &str, message: &str) {
    let path = current_desktop_log_path(app_root);
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let timestamp = Local::now().format("%H:%M:%S%.3f");
    let level = serilog_level(level);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "[{timestamp}] [{level}] RustDesktop\n{message}\n");
    prune_desktop_logs(app_root);
}

fn serilog_level(level: &str) -> &str {
    match level {
        "TRACE" | "DEBUG" => "DBG",
        "WARN" | "WARNING" => "WRN",
        "ERROR" => "ERR",
        "FATAL" => "FTL",
        _ => "INF",
    }
}

fn prune_desktop_logs(app_root: &Path) {
    let dir = desktop_log_dir(app_root);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let cutoff = Local::now().date_naive() - chrono::Duration::days(DESKTOP_LOG_RETAINED_DAYS);
    let mut retained = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(date) = parse_desktop_log_date(&path) else {
            continue;
        };
        if date < cutoff {
            let _ = fs::remove_file(path);
        } else {
            retained.push((date, path));
        }
    }

    retained.sort_by(|left, right| right.0.cmp(&left.0));
    for (_, path) in retained.into_iter().skip(DESKTOP_LOG_RETAINED_FILE_COUNT) {
        let _ = fs::remove_file(path);
    }
}

fn append_desktop_app_log(app: &tauri::AppHandle, level: &str, message: &str) {
    if let Ok(root) = app_root(app) {
        append_desktop_log(&root, level, message);
    }
}

fn desktop_exit_to_tray_enabled(app: &tauri::AppHandle) -> bool {
    app_root(app)
        .ok()
        .map(|root| read_desktop_config(app, &root).common_config.exit_to_tray)
        .unwrap_or(false)
}

fn setup_desktop_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_root = app_root(&app.handle()).unwrap_or_else(|_| PathBuf::from("."));
    append_desktop_log(&app_root, "INFO", "desktop app setup started");

    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("show_hide", "Show / Hide").build(app)?)
        .item(&MenuItemBuilder::with_id("check_update", "Check Update").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("exit", "Exit").build(app)?)
        .build()?;

    let mut tray = TrayIconBuilder::with_id("bettergi-main")
        .tooltip("BetterGI Rust")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_hide" => {
                let _ = toggle_main_window(app);
            }
            "check_update" => trigger_tray_update_check(app),
            "exit" => {
                append_desktop_app_log(app, "INFO", "tray exit requested");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: TrayMouseButton::Left,
                ..
            } = event
            {
                let _ = toggle_main_window(&tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }
    tray.build(app)?;
    append_desktop_log(&app_root, "INFO", "desktop tray initialized");
    Ok(())
}

fn main_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())
}

fn hide_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window.hide().map_err(|error| error.to_string())?;
    append_desktop_app_log(app, "INFO", "main window hidden");
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = main_window(app)?;
    window.show().map_err(|error| error.to_string())?;
    let _ = window.unminimize();
    let _ = window.set_focus();
    append_desktop_app_log(app, "INFO", "main window shown");
    Ok(())
}

fn toggle_main_window(app: &tauri::AppHandle) -> Result<bool, String> {
    let window = main_window(app)?;
    match window.is_visible() {
        Ok(true) => {
            window.hide().map_err(|error| error.to_string())?;
            append_desktop_app_log(app, "INFO", "main window hidden from tray");
            Ok(false)
        }
        _ => {
            window.show().map_err(|error| error.to_string())?;
            let _ = window.unminimize();
            let _ = window.set_focus();
            append_desktop_app_log(app, "INFO", "main window shown from tray");
            Ok(true)
        }
    }
}

fn trigger_tray_update_check(app: &tauri::AppHandle) {
    let _ = show_main_window(app);
    let _ = app.emit("desktop-shell://check-update", "stable");
    append_desktop_app_log(app, "INFO", "tray check-update action requested");
}

fn subscription_file_for_repo(app_root: &Path, repo_path: &Path) -> PathBuf {
    let folder_name = repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(bgi_script::DEFAULT_REPO_FOLDER_NAME);
    app_root
        .join(bgi_script::SUBSCRIPTIONS_DIR)
        .join(format!("{folder_name}.json"))
}

fn import_source_repo_path(center_repo_path: &Path, git_repo: bool) -> PathBuf {
    if git_repo {
        return center_repo_path.to_path_buf();
    }

    let file_repo_root = center_repo_path.join("repo");
    if file_repo_root.is_dir() {
        file_repo_root
    } else {
        center_repo_path.to_path_buf()
    }
}

fn app_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    std::env::current_dir()
        .map_err(|error| error.to_string())
        .or_else(|_| {
            app.path()
                .app_config_dir()
                .map_err(|error| error.to_string())
        })
}

fn main() {
    tauri::Builder::default()
        .manage(RedeemCodeClipboardState::default())
        .manage(BackgroundUpdateState::default())
        .manage(NotificationServiceState::default())
        .manage(DesktopTaskRuntimeState::default())
        .manage(HtmlMaskBridgeState::default())
        .setup(|app| {
            setup_desktop_tray(app)?;
            if let Some(state) = app.handle().try_state::<NotificationServiceState>() {
                let _ = refresh_notification_service_state(app.handle(), &state);
            }
            start_background_update_check(app);
            start_redeem_clipboard_monitor(app);
            start_html_mask_position_sync(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            let WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };
            if window.label() != "main" {
                return;
            }

            let app = window.app_handle();
            if !desktop_exit_to_tray_enabled(app) {
                append_desktop_app_log(app, "INFO", "main window close requested");
                return;
            }

            api.prevent_close();
            match window.hide() {
                Ok(()) => append_desktop_app_log(
                    app,
                    "INFO",
                    "main window close requested; hidden to tray",
                ),
                Err(error) => append_desktop_app_log(
                    app,
                    "WARN",
                    &format!("failed to hide main window on close request: {error}"),
                ),
            }
        })
        .invoke_handler(tauri::generate_handler![
            dashboard_state,
            desktop_shell_state,
            desktop_shell_set_exit_to_tray,
            desktop_shell_show_main_window,
            desktop_shell_hide_main_window,
            desktop_shell_toggle_main_window,
            task_execute_shell,
            task_plan_auto_pathing,
            task_plan_auto_fight,
            task_execute_auto_fight_team_playback,
            task_probe_auto_fight_finish,
            task_detect_auto_fight_active_avatar,
            desktop_overlay_state,
            desktop_overlay_update,
            desktop_log_state,
            notification_send_test,
            notification_service_state,
            notification_service_refresh,
            update_check,
            update_background_state,
            update_open_download_page,
            update_launch_updater,
            update_ignore_version,
            redeem_code_feed_check,
            redeem_code_feed_mark_read,
            redeem_code_feed_items,
            redeem_code_live_codes,
            redeem_code_auto_redeem_plan,
            redeem_code_clipboard_state,
            redeem_code_clipboard_set_enabled,
            redeem_code_clipboard_check,
            redeem_code_clipboard_ignore,
            script_repo_state,
            script_repo_json,
            script_repo_subscriptions,
            script_repo_file,
            script_repo_mark_updated,
            script_repo_clear_update,
            script_repo_import_paths,
            script_repo_import_uri,
            script_repo_import_clipboard_uri,
            script_repo_clipboard_ignore,
            script_repo_update_subscribed,
            script_repo_update_from_git,
            script_repo_import_zip,
            script_groups,
            script_available_js_projects,
            script_available_key_mouse_scripts,
            script_available_pathing_scripts,
            script_available_pathing_tree,
            script_group_create,
            script_group_rename,
            script_group_delete,
            script_group_project_add_js,
            script_group_project_add_key_mouse,
            script_group_project_add_pathing,
            script_group_project_add_shell,
            script_group_project_update,
            script_group_project_remove,
            script_group_project_move,
            script_settings_document,
            script_settings_save,
            script_execute_js,
            script_execute_group_project,
            script_execute_group,
            script_execute_group_from_project,
            script_stop,
            html_mask_receive_from_webview,
            html_mask_bridge_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run BetterGI Rust desktop app");
}
