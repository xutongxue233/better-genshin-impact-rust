use bgi_capture::{
    capture_mode_infos, find_game_window, find_process_image_path_by_name, CaptureBackend,
    CaptureFrame, CaptureMode as NativeCaptureMode, CaptureSettings, GameCapture, GameWindowMatch,
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
    updater_launch_plan, write_config, AppConfig, GenshinAction, KeyBindingsConfig, KeyId,
    MaskWindowConfig, MaskWindowState, MirrorChyanLatestOutcome, MirrorChyanLatestResponse,
    NavigationItem, Notice, NotificationDispatchError, NotificationDispatchExecution,
    NotificationEmailClient, NotificationEmailRequest, NotificationEmailSecurity,
    NotificationEventResult, NotificationHttpClient, NotificationHttpRequest,
    NotificationHttpResponse, NotificationImage, NotificationPayload, NotificationProviderKind,
    NotificationWebSocketClient, NotificationWindowsToastClient, NotificationWindowsToastRequest,
    RedeemCodeFeedItem, RedeemCodeFeedUpdateDecision, RedeemCodeLiveData, UiShellDecision,
    UpdateChannel, UpdateDecision, UpdateDecisionAction, UpdateOption, UpdateRequestPlan,
    UpdateTrigger, UpdaterLaunchPlan, ALPHA_RELEASES_URL, DOWNLOAD_PAGE_URL,
    REDEEM_CODE_BBS_ACT_ID_1_URL, REDEEM_CODE_BBS_ACT_ID_2_URL, REDEEM_CODE_CODES_URL,
    REDEEM_CODE_LIVE_INDEX_URL, REDEEM_CODE_LIVE_REFRESH_CODE_URL, REDEEM_CODE_UPDATE_TIME_URL,
};
use bgi_hotkey::Hotkey;
use bgi_input::{
    currently_pressed_keys, input_events_for_action, input_events_for_key,
    post_message_events_for_action, release_pressed_keys_sequence,
    send_events_to_window_with_cancellation, InputEvent, InputSequence, KeyActionType, MouseButton,
    PostMessageMode,
};
use bgi_script::{
    add_key_mouse_script_project, add_pathing_script_project, add_script_group_project,
    add_shell_script_project, analyze_log_groups, available_js_script_projects,
    available_key_mouse_scripts, available_pathing_scripts, available_pathing_tree,
    clear_repo_bridge_update, create_script_group, delete_script_group, discover_log_files,
    execute_file_repo_import, execute_git_repo_update, execute_repo_import_with_git,
    execute_zip_repo_import, git_update_plan, host_bindings, load_log_parse_config,
    mark_repo_bridge_path_updated, move_script_group_project, parse_import_uri,
    parse_log_file_entries, read_repo_bridge_file, read_repo_bridge_repo_json,
    read_script_group_file, read_script_groups, read_script_settings_document,
    read_subscription_file, remove_script_group_project, rename_script_group,
    repo_bridge_index_nodes, repo_bridge_subscribed_paths_json, save_script_group_project_settings,
    script_group_file_path, script_host_security_summary, script_import_plan,
    script_repo_bridge_paths, script_runtime_summary, update_script_group_project,
    write_log_parse_config, zip_import_plan, GameCaptureFrameSource, GlobalInputDispatchMode,
    HtmlMaskCommand, HtmlMaskInitialState, HtmlMaskMessage, HtmlMaskWindowPlan,
    InputCancellationToken, KeyMouseScript, LogActionItem, LogAnalysisOptions, LogAnalysisReport,
    LogFileEntry, LogParseConfig, NotificationDispatchMode, ScriptGroup, ScriptGroupFile,
    ScriptGroupProject, ScriptGroupProjectPatch, ScriptGroupResumePointer, ScriptHostRuntimeConfig,
    ScriptHostRuntimeError, ScriptHostTarget, ScriptProjectStatus, ScriptProjectType,
    ScriptRepoBridgeIndexNode, SystemGitRunner,
};
use bgi_script_engine::{
    JavaScriptExecutionOutcome, ScriptGroupExecutionOutcome, ScriptGroupExecutionRoots,
};
use bgi_task::{
    battle_pass_reward_text_candidates_from_ocr_regions,
    check_rewards_text_candidates_from_ocr_regions, choose_talk_option_candidates_from_ocr_regions,
    choose_talk_option_ocr_rect_from_lowest_icon,
    claim_encounter_points_text_candidates_from_ocr_regions, count_auto_cook_target_color,
    decide_quick_teleport_tick, decide_teleport_move_map_center_after_drag,
    detect_active_combat_avatar_index_from_default_rects_with_arrow,
    evaluate_auto_pathing_resolution_preflight, execute_auto_cook_plan, execute_auto_eat_food_plan,
    execute_auto_eat_tick_plan, execute_auto_fight_finish_detection_live_probe,
    execute_auto_fish_tick_plan, execute_auto_music_album_plan,
    execute_auto_music_performance_plan, execute_auto_open_chest_plan,
    execute_auto_pathing_action_boundary_with_live_executor, execute_auto_pick_tick_plan,
    execute_auto_track_plan, execute_auto_wood_plan, execute_blessing_of_the_welkin_moon_live,
    execute_check_rewards_plan, execute_choose_talk_option_plan,
    execute_claim_battle_pass_rewards_plan, execute_claim_encounter_points_rewards_plan,
    execute_claim_mail_rewards_live, execute_go_to_crafting_bench_plan,
    execute_independent_task_live_if_available, execute_independent_task_with_cancel,
    execute_lower_head_then_walk_to_plan, execute_macro_hotkey_plan,
    execute_one_key_expedition_live, execute_quick_buy_plan, execute_quick_serenitea_pot_plan,
    execute_quick_teleport_tick_plan, execute_realtime_trigger_live_if_available,
    execute_relogin_live, execute_return_main_ui_live, execute_return_main_ui_plan,
    execute_set_time_live, execute_switch_party_plan, execute_team_context_combat_script_inputs,
    execute_teleport_plan, execute_use_redeem_code_plan, execute_walk_to_f_live,
    execute_wonderland_cycle_live, execute_wonderland_cycle_plan, extract_redeem_codes_from_text,
    independent_tasks, parse_auto_pick_text_list, plan_auto_cook, plan_auto_eat, plan_auto_fight,
    plan_auto_fish, plan_auto_music_game, plan_auto_open_chest, plan_auto_pathing, plan_auto_pick,
    plan_auto_wood, plan_quick_buy, plan_quick_enhance_artifact_macro, plan_quick_serenitea_pot,
    plan_quick_teleport, plan_return_main_ui, plan_turn_around_macro, plan_wonderland_cycle,
    redeem_code_entries_from_strings, reduce_lower_head_then_walk_to_tracking_frame,
    runtime_triggers, select_triggers_for_tick, switch_party_find_matching_text_candidate,
    switch_party_text_candidates_from_ocr_regions, task_catalog, AutoCookExecutionConfig,
    AutoCookExecutionPlan, AutoCookExecutionReport, AutoCookExecutionStatus, AutoCookRuntime,
    AutoCookRuntimeFrame, AutoCookTemplateLocator, AutoEatExecutionConfig, AutoEatExecutionPlan,
    AutoEatFoodExecutionPlan, AutoEatFoodExecutionReport, AutoEatFoodPlanMode, AutoEatFoodRuntime,
    AutoEatFoodStepAction, AutoEatFoodStepCondition, AutoEatRuntime, AutoEatTemplateLocator,
    AutoEatTickExecutionReport, AutoEatTickObservation, AutoEatTriggerState,
    AutoEatTriggeredAction, AutoFightExecutionConfig, AutoFightExecutionPlan,
    AutoFightFinishDetectionExecutionMode, AutoFightFinishDetectionLiveExecution, AutoFightParam,
    AutoFishBiteRule, AutoFishExecutionConfig, AutoFishExecutionPlan, AutoFishInputAction,
    AutoFishOverlayAction, AutoFishRuntime, AutoFishTemplateLocator, AutoFishTickExecutionReport,
    AutoFishTriggerState, AutoMusicAlbumExecutionReport, AutoMusicAlbumPageStatus,
    AutoMusicAlbumRuntime, AutoMusicDifficultyRule, AutoMusicGameExecutionConfig,
    AutoMusicGameExecutionPlan, AutoMusicGameKeyLane, AutoMusicLaneBlueSample,
    AutoMusicPerformanceFrame, AutoMusicPerformanceReport, AutoMusicPerformanceRuntime,
    AutoMusicTemplateLocator, AutoOpenChestAction, AutoOpenChestActionPress,
    AutoOpenChestExecutionConfig, AutoOpenChestExecutionPlan, AutoOpenChestExecutionReport,
    AutoOpenChestObservation, AutoOpenChestRuntime, AutoPathingActionBoundaryReport,
    AutoPathingExecutionConfig, AutoPathingExecutionPlan, AutoPathingResolutionPreflightStatus,
    AutoPickExecutionConfig, AutoPickExecutionPlan, AutoPickRelativeTemplateLocator,
    AutoPickRuntime, AutoPickRuntimeLists, AutoPickTemplateLocator, AutoPickTickExecutionReport,
    AutoPickTickObservation, AutoTrackActionPress, AutoTrackExecutionConfig,
    AutoTrackExecutionPlan, AutoTrackExecutionReport, AutoTrackExecutionState,
    AutoTrackMainUiObservation, AutoTrackMissionTextObservation, AutoTrackRuntime,
    AutoTrackRuntimeAction, AutoTrackTeleportCandidate, AutoTrackTeleportObservation,
    AutoTrackTemplateLocator, AutoTrackTemplateMatch, AutoTrackTrackingObservation,
    AutoWoodCleanupOutcome, AutoWoodDelayOutcome, AutoWoodDelayReason, AutoWoodExecutionConfig,
    AutoWoodExecutionPlan, AutoWoodExecutionReport, AutoWoodGadgetOutcome,
    AutoWoodGarbageCollectionOutcome, AutoWoodInputAction, AutoWoodInputOutcome,
    AutoWoodOcrOutcome, AutoWoodRefreshOutcome, AutoWoodRefreshStrategy, AutoWoodRuntime,
    AutoWoodRuntimeRoundContext, AutoWoodStartupOutcome, AutoWoodTemplateLocator,
    AutoWoodThirdPartyLoginMode, AutoWoodThirdPartyLoginOutcome, AutoWoodWindowOutcome,
    BattlePassClaimAllRule, BattlePassClaimScope, BattlePassRewardTextCandidate,
    BlessingOfTheWelkinMoonExecutionPlan, BlessingOfTheWelkinMoonExecutionReport,
    CancellableCommonJobClock, CheckRewardsExecutionPlan, CheckRewardsExecutionReport,
    CheckRewardsRuntime, CheckRewardsTextCandidate, ChooseTalkOptionCandidate,
    ChooseTalkOptionExecutionPlan, ChooseTalkOptionExecutionReport, ChooseTalkOptionOcrRule,
    ChooseTalkOptionOrangeRule, ChooseTalkOptionRuntime, ClaimBattlePassRewardsExecutionPlan,
    ClaimBattlePassRewardsExecutionReport, ClaimBattlePassRewardsRuntime,
    ClaimEncounterPointsRewardsExecutionPlan, ClaimEncounterPointsRewardsExecutionReport,
    ClaimEncounterPointsRewardsOcrRule, ClaimEncounterPointsRewardsRuntime,
    ClaimEncounterPointsRewardsTextCandidate, ClaimMailRewardsExecutionPlan,
    ClaimMailRewardsExecutionReport, CombatActiveAvatarDetectionResult, CombatCommandPlaybackMode,
    CombatTeamPlaybackExecution, CommonJobClock, CommonJobExecutionPlan, CommonJobFrameSource,
    CommonJobInputDriver, CommonJobLiveExecutionReport, CommonJobRuntime, CommonJobRuntimeOutcome,
    CommonJobStepAction, CountInventoryGridIconMatch, CountInventoryGridItemFrame,
    CountInventoryItemExecutionPlan, CountInventoryItemExecutionReport,
    CountInventoryItemStepAction, CountInventoryItemStepCondition,
    CountInventoryOpenInventoryOutcome, CountInventoryOpenInventoryRule, DispatcherRuntime,
    GoToAdventurersGuildExecutionPlan, GoToAdventurersGuildStepAction,
    GoToAdventurersGuildStepCondition, GoToCraftingBenchExecutionPlan,
    GoToCraftingBenchExecutionReport, GoToCraftingBenchInteractionRule,
    GoToCraftingBenchPathingRule, GoToCraftingBenchResinCounts, GoToCraftingBenchResinCraftRule,
    GoToCraftingBenchResinRecognitionRule, GoToCraftingBenchRuntime, GoToCraftingBenchStepAction,
    GoToSereniteaPotEntryMode, GoToSereniteaPotExecutionPlan, GoToSereniteaPotExecutionReport,
    GoToSereniteaPotStepAction, GoToSereniteaPotStepCondition, GridIconClassifierRule,
    GridIconCropRule, GridItemCountOcrRule, GridItemDetectionRule, GridScrollRule, GridTemplate,
    IndependentTaskExecution, IndependentTaskExecutionRequest, IndependentTaskLiveExecutionReport,
    LowerHeadThenWalkToExecutionPlan, LowerHeadThenWalkToExecutionReport,
    LowerHeadThenWalkToFKeyRule, LowerHeadThenWalkToMovementRule, LowerHeadThenWalkToRuntime,
    LowerHeadThenWalkToStepResult, LowerHeadThenWalkToTrackingObservation,
    MacroHotkeyExecutionConfig, MacroHotkeyExecutionPlan, MacroHotkeyExecutionReport,
    MacroHotkeyPreflightRule, MacroHotkeyRuntime, MacroHotkeyScreenPoint,
    OneKeyExpeditionExecutionPlan, OneKeyExpeditionExecutionReport, PartyTextClickYAnchor,
    PureTemplateCommonJobRuntime, QuickBuyClickTarget, QuickBuyExecutionConfig,
    QuickBuyExecutionPlan, QuickBuyExecutionReport, QuickBuyPreflightRule, QuickBuyRuntime,
    QuickBuyScreenPoint, QuickSereniteaPotExecutionConfig, QuickSereniteaPotExecutionPlan,
    QuickSereniteaPotExecutionReport, QuickSereniteaPotInteractionOutcome,
    QuickSereniteaPotInteractionRule, QuickSereniteaPotPlacementOutcome,
    QuickSereniteaPotPlacementRule, QuickSereniteaPotPreflightRule, QuickSereniteaPotRuntime,
    QuickSereniteaPotScreenPoint, QuickTeleportDecisionAction, QuickTeleportDecisionInput,
    QuickTeleportExecutionConfig, QuickTeleportExecutionPlan, QuickTeleportMapChooseCandidate,
    QuickTeleportRuntime, QuickTeleportTemplateLocator, QuickTeleportTickExecutionReport,
    RealtimeTriggerExecutionPlan, RealtimeTriggerLiveExecutionReport, RedeemCodeEntry,
    ReloginDpiAwarePoint, ReloginExecutionPlan, ReloginExecutionReport, ReloginPlatformDriver,
    ReloginThirdPartyRule, ReturnMainUiExecutionPlan, ReturnMainUiExecutionReport, RunnerRuntime,
    ScanPickDropsExecutionPlan, ScanPickDropsExecutionReport, ScriptDispatcherExecutionPlan,
    ScriptDispatcherLiveExecutionReport, SetTimeExecutionPlan, SetTimeExecutionReport, ShellConfig,
    ShellExecutionResult, SwitchPartyChooseMenuRule, SwitchPartyConfirmRule,
    SwitchPartyExecutionPlan, SwitchPartyExecutionReport, SwitchPartyListScanOutcome,
    SwitchPartyListScanRule, SwitchPartyRuntime, SwitchPartyTextCandidate, TaskError,
    TaskInvocationExecutionMode, TaskInvocationExecutionResult, TaskInvocationExecutionStatus,
    TeleportExecutionPlan, TeleportExecutionReport, TeleportFailurePolicy, TeleportMapPoint,
    TeleportMoveMapCenterDecision, TeleportMoveMapRule, TeleportRuntime, TeleportStepAction,
    TeleportTargetPlan, UseRedeemCodeExecutionConfig, UseRedeemCodeExecutionPlan,
    UseRedeemCodeExecutionReport, UseRedeemCodeRuntime, WalkToFExecutionPlan,
    WalkToFExecutionReport, WonderlandCycleExecutionPlan, WonderlandCycleExecutionReport,
    AUTO_BOSS_TASK_KEY, AUTO_DOMAIN_TASK_KEY, AUTO_GENIUS_INVOKATION_TASK_KEY,
    AUTO_LEY_LINE_OUTCROP_TASK_KEY, AUTO_MUSIC_GAME_TASK_KEY,
    AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH, AUTO_OPEN_CHEST_TASK_KEY, AUTO_PICK_PICK_KEY_ASSET,
    AUTO_STYGIAN_ONSLAUGHT_TASK_KEY, AUTO_TRACK_DEFAULT_CAPTURE_WIDTH, AUTO_TRACK_PATH_TASK_KEY,
    AUTO_TRACK_TASK_KEY, AUTO_WOOD_DEFAULT_CAPTURE_WIDTH, AUTO_WOOD_TASK_KEY,
    COMMON_BTN_WHITE_CONFIRM, QUICK_BUY_TASK_KEY, QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY,
    QUICK_SERENITEA_POT_TASK_KEY, QUICK_TELEPORT_MAP_SCALE_BUTTON,
    QUICK_TELEPORT_MAP_SETTINGS_BUTTON, RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS,
    RETURN_MAIN_UI_TASK_KEY, TURN_AROUND_MACRO_TASK_KEY, USE_REDEEM_CODE_TASK_KEY,
};
use bgi_vision::{
    convert_bgr_image, crop_bgr_image, in_range_mask, recognition_type_infos,
    registered_onnx_models, resize_bgr_nearest, BgrImage, BvImage, BvLocatorOperation,
    BvLocatorPlan, BvPageCommand, ColorConversion, ImageRegion, OcrMatchConfig, OcrResult,
    OcrResultRegion, OnnxModelLoadPlan, OnnxProviderSelection, PureRustVisionBackend,
    RecognitionType, Rect, Region, Size as VisionSize, VisionBackend,
};
use chrono::{FixedOffset, Local, NaiveDate, Offset};
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
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    rust_invocation_plan_ready_catalog_entries: usize,
    rust_execution_plan_ready_catalog_entries: usize,
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

#[derive(Default)]
struct DesktopQuickTeleportTriggerState {
    previous_tick: Option<Instant>,
    chat_hotkey_guard: DesktopChatUiHotkeyGuardState,
    hotkey_latch: DesktopQuickTeleportHotkeyLatchState,
}

#[derive(Debug, Clone, Default)]
struct DesktopQuickTeleportHotkeyLatchState {
    configured_vk: Option<u16>,
    physical_was_pressed: bool,
    pressed: bool,
}

impl DesktopQuickTeleportHotkeyLatchState {
    fn update(
        &mut self,
        configured_vk: Option<u16>,
        physical_pressed: bool,
        game_window_foreground: bool,
        chat_hotkey_blocked: bool,
    ) -> bool {
        if self.configured_vk != configured_vk {
            self.configured_vk = configured_vk;
            self.physical_was_pressed = physical_pressed;
            self.pressed = false;
        }
        if configured_vk.is_none() {
            return false;
        }
        if !physical_pressed {
            self.physical_was_pressed = false;
            self.pressed = false;
            return false;
        }

        if !self.physical_was_pressed {
            self.pressed = game_window_foreground && !chat_hotkey_blocked;
        }
        self.physical_was_pressed = true;
        self.pressed && game_window_foreground
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum DesktopChatUiState {
    #[default]
    Closed,
    PanelOpen,
    InputOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DesktopChatUiDetection {
    state: DesktopChatUiState,
    has_back_button: bool,
    has_more_button: bool,
    has_add_conversation_button: bool,
    bottom_circle_count: usize,
    has_send_button: bool,
}

#[derive(Debug, Clone)]
struct DesktopChatUiHotkeyGuardState {
    chat_ui_state: DesktopChatUiState,
    enter_frame_count: u8,
    exit_frame_count: u8,
    chat_key_prime_until: Option<Instant>,
}

impl Default for DesktopChatUiHotkeyGuardState {
    fn default() -> Self {
        Self {
            chat_ui_state: DesktopChatUiState::Closed,
            enter_frame_count: 0,
            exit_frame_count: 0,
            chat_key_prime_until: None,
        }
    }
}

impl DesktopChatUiHotkeyGuardState {
    const STABLE_FRAME_THRESHOLD: u8 = 2;
    const CHAT_KEY_PRIME_DURATION_MS: u64 = 280;

    fn update_visual_state(&mut self, detection: DesktopChatUiDetection, now: Instant) {
        let visual_state = detection.state;
        if visual_state == DesktopChatUiState::Closed {
            self.enter_frame_count = 0;
            if self.chat_ui_state == DesktopChatUiState::Closed {
                self.exit_frame_count = 0;
            } else {
                self.exit_frame_count = self.exit_frame_count.saturating_add(1);
                if self.exit_frame_count >= Self::STABLE_FRAME_THRESHOLD {
                    self.chat_ui_state = DesktopChatUiState::Closed;
                    self.exit_frame_count = 0;
                }
            }
        } else {
            self.exit_frame_count = 0;
            self.chat_key_prime_until = None;
            if self.chat_ui_state == DesktopChatUiState::Closed {
                self.enter_frame_count = self.enter_frame_count.saturating_add(1);
                if self.enter_frame_count >= Self::STABLE_FRAME_THRESHOLD {
                    self.chat_ui_state = visual_state;
                    self.enter_frame_count = 0;
                }
            } else {
                self.enter_frame_count = 0;
                self.chat_ui_state = visual_state;
            }
        }

        if self
            .chat_key_prime_until
            .is_some_and(|deadline| deadline <= now)
        {
            self.chat_key_prime_until = None;
        }
    }

    fn prime_from_open_chat_key(&mut self, pressed_keys: &[u16], open_chat_vk: u16, now: Instant) {
        if open_chat_vk == KeyId::NONE.vk() || !pressed_keys.contains(&open_chat_vk) {
            return;
        }
        if self.chat_ui_state != DesktopChatUiState::Closed {
            return;
        }
        self.chat_key_prime_until =
            Some(now + Duration::from_millis(Self::CHAT_KEY_PRIME_DURATION_MS));
    }

    fn should_block_hotkey(&self, config_property_name: &str, now: Instant) -> bool {
        if config_property_name == "BgiEnabledHotkey" {
            return false;
        }
        self.chat_ui_state != DesktopChatUiState::Closed
            || self
                .chat_key_prime_until
                .is_some_and(|deadline| deadline > now)
    }
}

struct DesktopTaskRuntimeState {
    dispatcher: Mutex<DispatcherRuntime>,
    runner: Mutex<RunnerRuntime>,
    auto_eat_state: Mutex<AutoEatTriggerState>,
    auto_fish_state: Mutex<AutoFishTriggerState>,
    quick_teleport_state: Mutex<DesktopQuickTeleportTriggerState>,
    script_cancellation: Arc<InputCancellationToken>,
}

impl Default for DesktopTaskRuntimeState {
    fn default() -> Self {
        Self {
            dispatcher: Mutex::new(DispatcherRuntime::default()),
            runner: Mutex::new(RunnerRuntime::default()),
            auto_eat_state: Mutex::new(AutoEatTriggerState::default()),
            auto_fish_state: Mutex::new(AutoFishTriggerState::default()),
            quick_teleport_state: Mutex::new(DesktopQuickTeleportTriggerState::default()),
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

#[derive(Debug, Serialize)]
struct DesktopRedeemCodeExecutionResult {
    extracted_codes: Vec<String>,
    plan: UseRedeemCodeExecutionPlan,
    report: UseRedeemCodeExecutionReport,
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

#[derive(Debug, Serialize)]
struct DesktopQuickBuyTaskExecution {
    task: String,
    result: QuickBuyExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopQuickSereniteaPotTaskExecution {
    task: String,
    result: QuickSereniteaPotExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoWoodTaskExecution {
    task: String,
    result: AutoWoodExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoTrackTaskExecution {
    task: String,
    result: AutoTrackExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoOpenChestTaskExecution {
    task: String,
    result: AutoOpenChestExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoCookTaskExecution {
    task: String,
    result: AutoCookExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoEatTickExecution {
    task: String,
    result: AutoEatTickExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoFishTickExecution {
    task: String,
    result: AutoFishTickExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoPickTickExecution {
    task: String,
    result: AutoPickTickExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopQuickTeleportTickExecution {
    task: String,
    result: QuickTeleportTickExecutionReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoMusicGamePerformanceExecution {
    task: String,
    result: AutoMusicPerformanceReport,
}

#[derive(Debug, Serialize)]
struct DesktopAutoMusicGameAlbumExecution {
    task: String,
    result: AutoMusicAlbumExecutionReport,
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

#[derive(Debug, Serialize)]
struct DesktopAutoPathingActionBoundaryExecution {
    task: String,
    plan: AutoPathingExecutionPlan,
    boundary: AutoPathingActionBoundaryReport,
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct DesktopLogParseAnalyzeRequest {
    script_group_name: Option<String>,
    range_value: Option<String>,
    day_range_value: Option<String>,
    merge_stats: Option<bool>,
    hoeing_delay_seconds: Option<i64>,
    action_items: Vec<LogActionItem>,
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

#[derive(Debug, Clone)]
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
    let task_catalog_entries = task_catalog();

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
            catalog_entries: task_catalog_entries.len(),
            config_bound_catalog_entries: task_catalog_entries
                .iter()
                .filter(|task| task.config_bound())
                .count(),
            rust_invocation_plan_ready_catalog_entries: task_catalog_entries
                .iter()
                .filter(|task| {
                    task.rust_execution_surface()
                        == bgi_task::TaskRustExecutionSurface::InvocationPlanOnly
                })
                .count(),
            rust_execution_plan_ready_catalog_entries: task_catalog_entries
                .iter()
                .filter(|task| {
                    task.rust_execution_surface()
                        == bgi_task::TaskRustExecutionSurface::ExecutionPlanOnly
                })
                .count(),
            native_pending_catalog_entries: task_catalog_entries
                .iter()
                .filter(|task| {
                    if task.has_rust_execution_plan() {
                        return false;
                    }
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

fn dispatch_desktop_notification_logged(
    app_root: &Path,
    config: &AppConfig,
    event: &str,
    result: NotificationEventResult,
    message: &str,
) {
    match dispatch_desktop_notification(app_root, config, event, result, message) {
        Ok(execution) => append_desktop_log(
            app_root,
            "INFO",
            &format!(
                "notification dispatched: event={event}, attempted={}, deliveries={}",
                execution.attempted,
                execution.deliveries.len()
            ),
        ),
        Err(error) => append_desktop_log(
            app_root,
            "WARN",
            &format!("notification dispatch failed for {event}: {error}"),
        ),
    }
}

fn dispatch_desktop_notification(
    app_root: &Path,
    config: &AppConfig,
    event: &str,
    result: NotificationEventResult,
    message: &str,
) -> Result<NotificationDispatchExecution, String> {
    let mut payload = NotificationPayload {
        event: event.to_string(),
        result,
        message: Some(message.to_string()),
        data: None,
        timestamp_ms: Some(current_time_ms()?),
        has_screenshot: false,
        screenshot: None,
    };
    if config.notification_config.include_screen_shot {
        if let Some(screenshot) = capture_notification_screenshot(config) {
            payload = payload.with_screenshot(screenshot);
        }
    }

    let mut client = DesktopNotificationHttpClient::new().map_err(|error| error.to_string())?;
    let mut web_socket_client = DesktopNotificationWebSocketClient;
    let mut email_client = DesktopNotificationEmailClient;
    let mut windows_toast_client = DesktopNotificationWindowsToastClient::new(app_root);
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
    let extracted_codes: Vec<String> = entries.iter().map(|entry| entry.code.clone()).collect();
    let plan = plan_use_redeem_codes_through_task_boundary(entries, &app_root)?;
    Ok(DesktopRedeemCodePlanResult {
        extracted_codes,
        plan,
    })
}

#[tauri::command]
fn redeem_code_auto_redeem_execute(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    state: tauri::State<'_, RedeemCodeClipboardState>,
    payload: RedeemCodePlanPayload,
) -> Result<DesktopRedeemCodeExecutionResult, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "IndependentTask:UseRedeemCode".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "redeem code auto-redeem execution requested",
        );
        let entries = redeem_code_entries_from_payload(payload);
        let extracted_codes: Vec<String> = entries.iter().map(|entry| entry.code.clone()).collect();
        for code in &extracted_codes {
            let _ = remember_ignored_clipboard_hash(&state, code)?;
        }
        let (plan, report) = execute_desktop_use_redeem_code_live_plan(
            &app_root,
            &config,
            find_desktop_game_window(&config).as_ref(),
            entries,
            Arc::clone(&script_cancellation),
        )?;
        Ok(DesktopRedeemCodeExecutionResult {
            extracted_codes,
            plan,
            report,
        })
    })();
    finish_desktop_script_run(&task_state);
    execution
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
        entries.extend(redeem_code_entries_from_strings(
            codes.iter().map(String::as_str),
        ));
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
    plan_use_redeem_codes_through_task_boundary_with_capture_size(
        entries,
        bgi_vision::Size::new(1920, 1080),
        app_root,
    )
}

fn plan_use_redeem_codes_through_task_boundary_with_capture_size(
    entries: Vec<RedeemCodeEntry>,
    capture_size: VisionSize,
    app_root: &Path,
) -> Result<UseRedeemCodeExecutionPlan, String> {
    let request = IndependentTaskExecutionRequest::use_redeem_code(entries, capture_size, app_root);
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

#[cfg(windows)]
fn set_system_clipboard_text(text: &str) -> Result<(), String> {
    use std::ptr;
    use windows::Win32::Foundation::{GlobalFree, HANDLE};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_UNICODETEXT;

    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let byte_len = wide.len() * std::mem::size_of::<u16>();

    unsafe {
        let global = GlobalAlloc(GMEM_MOVEABLE, byte_len).map_err(|error| error.to_string())?;
        let ptr = GlobalLock(global) as *mut u16;
        if ptr.is_null() {
            let _ = GlobalFree(Some(global));
            return Err("failed to lock clipboard text buffer".to_string());
        }
        ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        let _ = GlobalUnlock(global);

        if let Err(error) = OpenClipboard(None) {
            let _ = GlobalFree(Some(global));
            return Err(error.to_string());
        }
        if let Err(error) = EmptyClipboard() {
            let _ = CloseClipboard();
            let _ = GlobalFree(Some(global));
            return Err(error.to_string());
        }
        if let Err(error) = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(global.0))) {
            let _ = CloseClipboard();
            let _ = GlobalFree(Some(global));
            return Err(error.to_string());
        }
        let _ = CloseClipboard();
    }

    Ok(())
}

#[cfg(not(windows))]
fn set_system_clipboard_text(_text: &str) -> Result<(), String> {
    Err("clipboard write is only supported on Windows".to_string())
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

fn desktop_log_parse_selected_files(
    app_root: &Path,
    day_range_value: Option<&str>,
) -> Result<Vec<LogFileEntry>, String> {
    let mut files = discover_log_files(app_root.join("log")).map_err(|error| error.to_string())?;
    let value = day_range_value.unwrap_or("7").trim();
    if value.eq_ignore_ascii_case("All") {
        return Ok(files);
    }

    let log_day = value
        .parse::<usize>()
        .map_err(|_| format!("invalid log day range: {value}"))?;
    if files.len() > log_day {
        files = files.split_off(files.len() - log_day);
    }
    Ok(files)
}

#[tauri::command]
fn log_parse_config_get(app: tauri::AppHandle) -> Result<LogParseConfig, String> {
    let app_root = app_root(&app)?;
    Ok(load_log_parse_config(&app_root))
}

#[tauri::command]
fn log_parse_config_save(
    app: tauri::AppHandle,
    config: LogParseConfig,
) -> Result<LogParseConfig, String> {
    let app_root = app_root(&app)?;
    write_log_parse_config(&app_root, &config).map_err(|error| error.to_string())?;
    Ok(config)
}

#[tauri::command]
fn log_parse_analyze(
    app: tauri::AppHandle,
    request: DesktopLogParseAnalyzeRequest,
) -> Result<LogAnalysisReport, String> {
    let app_root = app_root(&app)?;
    let files = desktop_log_parse_selected_files(&app_root, request.day_range_value.as_deref())?;
    let mut config_groups = parse_log_file_entries(&files);

    let range_value = request.range_value.as_deref().unwrap_or("CurrentConfig");
    if range_value.trim().eq_ignore_ascii_case("CurrentConfig") {
        if let Some(script_group_name) = request
            .script_group_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            config_groups.retain(|group| group.name == script_group_name);
        }
    }

    let options = LogAnalysisOptions {
        merge_stats: request.merge_stats.unwrap_or(false),
        hoeing_delay_seconds: request.hoeing_delay_seconds.unwrap_or(0),
    };
    Ok(analyze_log_groups(
        config_groups,
        &request.action_items,
        &options,
    ))
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
        execute_desktop_task_invocation_live_in_javascript_outcome(
            &app_root,
            &config,
            game_window.as_ref(),
            &task_state,
            Arc::clone(&script_cancellation),
            &mut outcome,
        );
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
fn task_execute_quick_buy(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopQuickBuyTaskExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "IndependentTask:QuickBuy".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "independent quick-buy execution requested",
        );
        execute_desktop_quick_buy_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_quick_serenitea_pot(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopQuickSereniteaPotTaskExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "IndependentTask:QuickSereniteaPot".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "independent quick-serenitea-pot execution requested",
        );
        execute_desktop_quick_serenitea_pot_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_open_chest(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoOpenChestTaskExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "IndependentTask:AutoOpenChest".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "independent auto-open-chest execution requested",
        );
        execute_desktop_auto_open_chest_live_plan(
            &app_root,
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_cook(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoCookTaskExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "ScriptDispatcher:AutoCook".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(&app_root, "INFO", "auto-cook live execution requested");
        execute_desktop_auto_cook_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_eat_tick(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoEatTickExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "RealtimeTrigger:AutoEat".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(&app_root, "INFO", "auto-eat realtime tick requested");
        execute_desktop_auto_eat_tick_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            &task_state.auto_eat_state,
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_fish_tick(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoFishTickExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "RealtimeTrigger:AutoFish".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(&app_root, "INFO", "auto-fish realtime tick requested");
        execute_desktop_auto_fish_tick_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            &task_state.auto_fish_state,
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_pick_tick(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoPickTickExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "RealtimeTrigger:AutoPick".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        let runner_pause_count = task_state
            .runner
            .lock()
            .map(|runner| runner.auto_pick_pause_count)
            .map_err(|_| "task runner state lock poisoned".to_string())?;
        append_desktop_log(&app_root, "INFO", "auto-pick realtime tick requested");
        execute_desktop_auto_pick_tick_live_plan(
            &app_root,
            &config,
            find_desktop_game_window(&config).as_ref(),
            runner_pause_count,
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_quick_teleport_tick(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopQuickTeleportTickExecution, String> {
    let script_cancellation =
        start_desktop_script_run(&task_state, "RealtimeTrigger:QuickTeleport".to_string())?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(&app_root, "INFO", "quick-teleport realtime tick requested");
        execute_desktop_quick_teleport_tick_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            &task_state.quick_teleport_state,
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_music_game_performance(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoMusicGamePerformanceExecution, String> {
    let script_cancellation = start_desktop_script_run(
        &task_state,
        "IndependentTask:AutoMusicGame:Performance".to_string(),
    )?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "auto-music-game performance live execution requested",
        );
        execute_desktop_auto_music_game_performance_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        )
    })();
    finish_desktop_script_run(&task_state);
    execution
}

#[tauri::command]
fn task_execute_auto_music_game_album(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
) -> Result<DesktopAutoMusicGameAlbumExecution, String> {
    let script_cancellation = start_desktop_script_run(
        &task_state,
        "IndependentTask:AutoMusicGame:Album".to_string(),
    )?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            "auto-music-game album live execution requested",
        );
        dispatch_desktop_notification_logged(
            &app_root,
            &config,
            "album.start",
            NotificationEventResult::Success,
            "自动音游专辑启动",
        );
        let execution = execute_desktop_auto_music_game_album_live_plan(
            &config,
            find_desktop_game_window(&config).as_ref(),
            Arc::clone(&script_cancellation),
        );
        match execution {
            Ok(execution) => {
                dispatch_desktop_notification_logged(
                    &app_root,
                    &config,
                    "album.end",
                    NotificationEventResult::Success,
                    "自动音游专辑结束",
                );
                Ok(execution)
            }
            Err(error) => {
                dispatch_desktop_notification_logged(
                    &app_root,
                    &config,
                    "album.error",
                    NotificationEventResult::Fail,
                    &format!("自动音游专辑异常: {error}"),
                );
                Err(error)
            }
        }
    })();
    finish_desktop_script_run(&task_state);
    execution
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

#[tauri::command]
fn task_execute_auto_pathing_action_boundary(
    app: tauri::AppHandle,
    task_state: tauri::State<DesktopTaskRuntimeState>,
    payload: DesktopAutoPathingTaskPayload,
) -> Result<DesktopAutoPathingActionBoundaryExecution, String> {
    let script_cancellation = start_desktop_script_run(
        &task_state,
        format!("IndependentTask:AutoPathing:{}", payload.route),
    )?;
    let execution = (|| {
        let app_root = app_root(&app)?;
        let config = read_desktop_config(&app, &app_root);
        append_desktop_log(
            &app_root,
            "INFO",
            &format!(
                "independent auto-pathing action boundary requested: {}",
                payload.route
            ),
        );
        let request =
            IndependentTaskExecutionRequest::auto_pathing(payload.route.clone(), &app_root);
        let execution =
            execute_independent_task_with_cancel(&request, || script_cancellation.is_cancelled())
                .map_err(|error| error.to_string())?;
        let task = execution.task_key;
        let IndependentTaskExecution::AutoPathingPlan(plan) = execution.execution else {
            return Err("AutoPathing returned a non-pathing execution".to_string());
        };

        let game_window = find_desktop_game_window(&config);
        let boundary = execute_desktop_auto_pathing_action_boundary_live_plan(
            &config,
            game_window.as_ref(),
            Arc::clone(&script_cancellation),
            &plan,
        )
        .map_err(|error| error.to_string())?;

        Ok(DesktopAutoPathingActionBoundaryExecution {
            task,
            plan,
            boundary,
        })
    })();
    finish_desktop_script_run(&task_state);
    execution
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
        task_state.script_cancellation.clone()
    } else {
        Arc::new(InputCancellationToken::new())
    };
    let game_window = find_desktop_game_window(&config);
    let result = execute_desktop_auto_fight_finish_probe_live_plan(
        &config,
        game_window.as_ref(),
        &plan,
        mode,
        cancellation,
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
    roots.app_version = Some(app.package_info().version.to_string());
    let config = read_desktop_config(&app, &app_root);
    let farming_context = bgi_script::FarmingPlanExecutionContext::from_app_root(
        &app_root,
        config.other_config.farming_plan_config.clone(),
    )
    .with_app_version(app.package_info().version.to_string());
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let storage = bgi_script::ExecutionRecordStorage::from_app_root(&app_root);
        let execution_clock = desktop_execution_record_clock(&config)?;
        let outcome =
            bgi_script_engine::execute_script_group_project_with_execution_records_and_farming_plan_hooks_and_cancellation(
                &roots,
                &group,
                project_index,
                &storage,
                execution_clock,
                &farming_context,
                honor_run_count.unwrap_or(false),
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                    execute_desktop_task_invocation_live_in_javascript_outcome(
                        &app_root,
                        &config,
                        game_window.as_ref(),
                        &task_state,
                        Arc::clone(&script_cancellation),
                        javascript,
                    );
                },
            );
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
    let group_root = script_group_root(&app_root);
    let group_path = script_group_file_path(&group_root, &group_name);
    let group = read_script_group_file(&group_path).map_err(|error| error.to_string())?;
    let mut current_group_found = false;
    let mut all_groups = read_script_groups(&group_root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|file| {
            if file.path == group_path {
                current_group_found = true;
                group.clone()
            } else {
                file.group
            }
        })
        .collect::<Vec<_>>();
    if !current_group_found {
        all_groups.push(group.clone());
    }
    let mut roots = ScriptGroupExecutionRoots::from_app_root(&app_root);
    roots.app_version = Some(app.package_info().version.to_string());
    let config = read_desktop_config(&app, &app_root);
    let farming_context = bgi_script::FarmingPlanExecutionContext::from_app_root(
        &app_root,
        config.other_config.farming_plan_config.clone(),
    )
    .with_app_version(app.package_info().version.to_string());
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let storage = bgi_script::ExecutionRecordStorage::from_app_root(&app_root);
        let execution_clock = desktop_execution_record_clock(&config)?;
        let mut outcome =
            bgi_script_engine::execute_script_group_with_pre_execution_records_and_farming_plan_hooks_and_cancellation(
                &roots,
                &group,
                &all_groups,
                &storage,
                execution_clock,
                &farming_context,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                    execute_desktop_task_invocation_live_in_javascript_outcome(
                        &app_root,
                        &config,
                        game_window.as_ref(),
                        &task_state,
                        Arc::clone(&script_cancellation),
                        javascript,
                    );
                },
            )
            .map_err(|error| error.to_string())?;
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
    roots.app_version = Some(app.package_info().version.to_string());
    let config = read_desktop_config(&app, &app_root);
    let farming_context = bgi_script::FarmingPlanExecutionContext::from_app_root(
        &app_root,
        config.other_config.farming_plan_config.clone(),
    )
    .with_app_version(app.package_info().version.to_string());
    let game_window = find_desktop_game_window(&config);
    configure_desktop_script_roots(game_window.as_ref(), &mut roots);
    let result = (|| {
        let mut dispatcher = task_state
            .dispatcher
            .lock()
            .map_err(|_| "task dispatcher state lock poisoned".to_string())?;
        let storage = bgi_script::ExecutionRecordStorage::from_app_root(&app_root);
        let execution_clock = desktop_execution_record_clock(&config)?;
        let mut outcome =
            bgi_script_engine::execute_script_group_from_resume_with_execution_records_and_farming_plan_hooks_and_cancellation(
                &roots,
                &group,
                &resume_pointer,
                &storage,
                execution_clock,
                &farming_context,
                Some(&mut dispatcher),
                Some(script_cancellation.as_ref()),
                |host| {
                    host.html_mask_initial_state = take_html_mask_initial_state_for_script(&app);
                    configure_desktop_script_host(&config, game_window.as_ref(), host);
                },
                |javascript| {
                    restore_html_mask_from_script_outcome(&app, javascript);
                    dispatch_script_html_mask_in_javascript_outcome(&app, javascript);
                    execute_desktop_task_invocation_live_in_javascript_outcome(
                        &app_root,
                        &config,
                        game_window.as_ref(),
                        &task_state,
                        Arc::clone(&script_cancellation),
                        javascript,
                    );
                },
            )
            .map_err(|error| error.to_string())?;
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

#[cfg(windows)]
fn desktop_game_window_process_is_foreground(window: &GameWindowMatch) -> bool {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_invalid() {
        return false;
    }
    if foreground == HWND(window.handle.0 as *mut std::ffi::c_void) {
        return true;
    }
    let Some(expected_process_id) = window.process_id else {
        return false;
    };
    let mut foreground_process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(foreground, Some(&mut foreground_process_id));
    }
    foreground_process_id == expected_process_id
}

#[cfg(not(windows))]
fn desktop_game_window_process_is_foreground(_window: &GameWindowMatch) -> bool {
    true
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

fn execute_desktop_task_invocation_live_in_javascript_outcome(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_state: &DesktopTaskRuntimeState,
    cancellation: Arc<InputCancellationToken>,
    outcome: &mut JavaScriptExecutionOutcome,
) {
    if outcome.task_execution.mode != TaskInvocationExecutionMode::ExecuteReady {
        return;
    }

    for result in outcome
        .task_execution
        .dispatcher
        .iter_mut()
        .chain(outcome.task_execution.genshin.iter_mut())
    {
        execute_desktop_common_job_live_result(
            config,
            game_window,
            Arc::clone(&cancellation),
            result,
        );
        execute_desktop_script_dispatcher_live_result(
            config,
            game_window,
            Arc::clone(&cancellation),
            result,
        );
        execute_desktop_realtime_trigger_live_result(
            app_root,
            config,
            game_window,
            task_state,
            Arc::clone(&cancellation),
            result,
        );
        execute_desktop_independent_task_live_result(
            app_root,
            config,
            game_window,
            Arc::clone(&cancellation),
            result,
        );
    }
}

fn execute_desktop_realtime_trigger_live_result(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_state: &DesktopTaskRuntimeState,
    cancellation: Arc<InputCancellationToken>,
    result: &mut TaskInvocationExecutionResult,
) {
    let mut live_executor = |plan: &RealtimeTriggerExecutionPlan| match plan {
        RealtimeTriggerExecutionPlan::AutoEat(plan) => {
            let window = game_window.ok_or_else(|| {
                TaskError::VisionPlan(
                    "AutoEat live tick requires a detected game window".to_string(),
                )
            })?;
            let mut state = task_state.auto_eat_state.lock().map_err(|_| {
                TaskError::VisionPlan("AutoEat trigger state lock poisoned".to_string())
            })?;
            let report = execute_desktop_auto_eat_tick_live(
                config,
                window,
                plan,
                &mut state,
                Arc::clone(&cancellation),
            )
            .map_err(TaskError::VisionPlan)?;
            Ok(Some(RealtimeTriggerLiveExecutionReport::AutoEat(report)))
        }
        RealtimeTriggerExecutionPlan::AutoFish(plan) => {
            let window = game_window.ok_or_else(|| {
                TaskError::VisionPlan(
                    "AutoFish live tick requires a detected game window".to_string(),
                )
            })?;
            let mut state = task_state.auto_fish_state.lock().map_err(|_| {
                TaskError::VisionPlan("AutoFish trigger state lock poisoned".to_string())
            })?;
            let report = execute_desktop_auto_fish_tick_live(
                config,
                window,
                plan,
                &mut state,
                Arc::clone(&cancellation),
            )
            .map_err(TaskError::VisionPlan)?;
            Ok(Some(RealtimeTriggerLiveExecutionReport::AutoFish(report)))
        }
        RealtimeTriggerExecutionPlan::AutoPick(plan) => {
            let window = game_window.ok_or_else(|| {
                TaskError::VisionPlan(
                    "AutoPick live tick requires a detected game window".to_string(),
                )
            })?;
            let runner_pause_count = task_state
                .runner
                .lock()
                .map(|runner| runner.auto_pick_pause_count)
                .map_err(|_| {
                    TaskError::VisionPlan("task runner state lock poisoned".to_string())
                })?;
            let report = execute_desktop_auto_pick_tick_live(
                app_root,
                config,
                window,
                runner_pause_count,
                plan,
                Arc::clone(&cancellation),
            )
            .map_err(TaskError::VisionPlan)?;
            Ok(Some(RealtimeTriggerLiveExecutionReport::AutoPick(report)))
        }
        RealtimeTriggerExecutionPlan::QuickTeleport(plan) => {
            let window = game_window.ok_or_else(|| {
                TaskError::VisionPlan(
                    "QuickTeleport live tick requires a detected game window".to_string(),
                )
            })?;
            let mut state = task_state.quick_teleport_state.lock().map_err(|_| {
                TaskError::VisionPlan("QuickTeleport trigger state lock poisoned".to_string())
            })?;
            let report = execute_desktop_quick_teleport_tick_live(
                config,
                window,
                plan,
                &mut state,
                Arc::clone(&cancellation),
            )
            .map_err(TaskError::VisionPlan)?;
            Ok(Some(RealtimeTriggerLiveExecutionReport::QuickTeleport(
                report,
            )))
        }
        _ => Ok(None),
    };
    execute_realtime_trigger_live_if_available(result, &mut live_executor);
}

fn execute_desktop_common_job_live_result(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    result: &mut TaskInvocationExecutionResult,
) {
    let Some(plan) = result.common_job_execution_plan.clone() else {
        return;
    };
    let task_key = plan.task_key().to_string();
    match execute_desktop_common_job_live_plan(config, game_window, cancellation, &plan) {
        Ok(Some(report)) => {
            let (task_name, completed, executed_steps, skipped_steps) =
                desktop_common_job_live_summary(&report);
            result.status = TaskInvocationExecutionStatus::Ready;
            result.executed = true;
            result.message = format!(
                "{} live execution completed: completed={}, executed_steps={}, skipped_steps={}",
                task_name, completed, executed_steps, skipped_steps
            );
            result.live_completed = Some(completed);
            result.common_job_live_execution = Some(report);
        }
        Ok(None) => {
            result.live_completed = None;
        }
        Err(error) => {
            result.status = TaskInvocationExecutionStatus::Invalid;
            result.executed = false;
            result.message = format!("{task_key} live execution failed: {error}");
            result.live_completed = None;
            result.common_job_live_execution = None;
        }
    }
}

fn execute_desktop_script_dispatcher_live_result(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    result: &mut TaskInvocationExecutionResult,
) {
    let Some(plan) = result.script_dispatcher_execution_plan.clone() else {
        return;
    };
    let task_key = plan.task_key().to_string();
    match execute_desktop_script_dispatcher_live_plan(config, game_window, cancellation, &plan) {
        Ok(Some(report)) => {
            let completed = report.completed();
            result.status = TaskInvocationExecutionStatus::Ready;
            result.executed = true;
            result.message = desktop_script_dispatcher_live_summary(&report);
            result.live_completed = Some(completed);
            result.script_dispatcher_live_execution = Some(report);
        }
        Ok(None) => {
            result.live_completed = None;
        }
        Err(error) => {
            result.status = TaskInvocationExecutionStatus::Invalid;
            result.executed = false;
            result.message = format!("{task_key} live execution failed: {error}");
            result.live_completed = None;
            result.script_dispatcher_live_execution = None;
        }
    }
}

fn desktop_script_dispatcher_live_summary(report: &ScriptDispatcherLiveExecutionReport) -> String {
    match report {
        ScriptDispatcherLiveExecutionReport::AutoCook(report) => {
            let completed = report.status != AutoCookExecutionStatus::IterationLimitReached;
            format!(
                "AutoCook live execution completed: completed={}, status={:?}, frames_processed={}, space_press_count={}, white_confirm_click_count={}",
                completed,
                report.status,
                report.state.frames_processed,
                report.state.space_press_count,
                report.state.white_confirm_click_count
            )
        }
        ScriptDispatcherLiveExecutionReport::AutoEatFood(report) => {
            let outcome = report
                .state
                .decision
                .as_ref()
                .map(|decision| format!("{:?}", decision.outcome))
                .unwrap_or_else(|| "None".to_string());
            format!(
                "AutoEatFood live execution completed: completed={}, outcome={}, return_value={:?}, actions={}",
                report.completed,
                outcome,
                report
                    .state
                    .decision
                    .as_ref()
                    .and_then(|decision| decision.return_value),
                report.executed_actions.len()
            )
        }
        ScriptDispatcherLiveExecutionReport::AutoFishing(report) => format!(
            "AutoFishing live execution completed: completed={}, status={:?}, rounds_completed={}, fishponds_found={}, pull_bar_successes={}, actions={}, skipped_steps={}",
            report.completed,
            report.status,
            report.state.current_round,
            report.state.fishponds_found,
            report.state.pull_bar_successes,
            report.executed_actions.len(),
            report.skipped_steps.len()
        ),
    }
}

fn execute_desktop_auto_eat_tick_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    state: &Mutex<AutoEatTriggerState>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoEatTickExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoEat live tick requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_auto_eat(AutoEatExecutionConfig {
        capture_size,
        auto_eat_config: config.auto_eat_config.clone(),
    });
    let task = plan.task_key.clone();
    let mut state = state
        .lock()
        .map_err(|_| "AutoEat trigger state lock poisoned".to_string())?;
    let result =
        execute_desktop_auto_eat_tick_live(config, window, &plan, &mut state, cancellation)?;
    Ok(DesktopAutoEatTickExecution { task, result })
}

fn execute_desktop_auto_eat_tick_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoEatExecutionPlan,
    state: &mut AutoEatTriggerState,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoEatTickExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoEat live tick cancelled".to_string());
    }
    if !desktop_game_window_process_is_foreground(window) {
        return Err("AutoEat live tick requires the game window to be foreground".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoEat live tick requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoEat live tick requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoEat live tick requires the BitBlt capture backend".to_string());
    }

    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings)
        .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoEatRuntime::new(
        bgi_task::task_asset_root(),
        capture_size,
        capture_source,
        window.handle.0,
        config.key_bindings_config.clone(),
        cancellation,
    );
    execute_auto_eat_tick_plan(plan, state, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoEatRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    key_bindings_config: KeyBindingsConfig,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoEatRuntime {
    fn new(
        template_root: PathBuf,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        key_bindings_config: KeyBindingsConfig,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            capture_size,
            capture_source,
            window_handle,
            key_bindings_config,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "AutoEat live tick cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_eat_template(
        &self,
        capture: &ImageRegion,
        locator: &AutoEatTemplateLocator,
    ) -> bgi_task::Result<bool> {
        let object =
            desktop_auto_eat_template_object(locator, self.capture_size).map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoEat template object failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        let region = capture
            .find(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoEat template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist())
    }

    fn execute_events(&self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }
}

impl AutoEatRuntime for DesktopAutoEatRuntime {
    fn observe_auto_eat_tick(
        &mut self,
        plan: &AutoEatExecutionPlan,
    ) -> bgi_task::Result<AutoEatTickObservation> {
        let image = self.capture_bgr_image()?;
        let current_avatar_low_hp = desktop_auto_eat_current_avatar_low_hp(&image, plan);
        let capture = ImageRegion::capture(image);
        let recovery_icon_detected =
            self.locate_auto_eat_template(&capture, &plan.locators.recovery_icon)?;
        let resurrection_icon_detected =
            self.locate_auto_eat_template(&capture, &plan.locators.resurrection_icon)?;
        let now_ms = current_time_ms().map_err(TaskError::VisionPlan)?;
        Ok(AutoEatTickObservation {
            now_ms,
            current_avatar_low_hp,
            recovery_icon_detected,
            resurrection_icon_detected,
        })
    }

    fn dispatch_auto_eat_action(
        &mut self,
        action: &AutoEatTriggeredAction,
    ) -> bgi_task::Result<()> {
        let action = match action {
            AutoEatTriggeredAction::Eat { action }
            | AutoEatTriggeredAction::Resurrect { action } => action,
        };
        let action = desktop_auto_eat_genshin_action(action).ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "AutoEat action is not supported on desktop: {action}"
            ))
        })?;
        let events =
            input_events_for_action(&self.key_bindings_config, action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        self.execute_events(events)
    }
}

fn desktop_auto_eat_template_object(
    locator: &AutoEatTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object = image.to_recognition_object_for_screen(
        Some(locator.roi),
        locator.threshold,
        capture_size,
    )?;
    object.name = Some(locator.name.clone());
    object.template.mode = locator.match_mode;
    object.template.use_3_channels = locator.use_3_channels;
    object.template.draw_on_window = locator.draw_on_window;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_eat_current_avatar_low_hp(image: &BgrImage, plan: &AutoEatExecutionPlan) -> bool {
    let probe = &plan.detection_rule.low_hp_pixel_probe;
    let Ok(x) = u32::try_from(probe.point.x) else {
        return false;
    };
    let Ok(y) = u32::try_from(probe.point.y) else {
        return false;
    };
    image
        .rgb_pixel_at(x, y)
        .is_some_and(|pixel| pixel == probe.expected_rgb)
}

fn desktop_auto_eat_genshin_action(action: &str) -> Option<GenshinAction> {
    match action.trim() {
        "GIActions.QuickUseGadget" | "QuickUseGadget" => Some(GenshinAction::QuickUseGadget),
        _ => None,
    }
}

fn execute_desktop_auto_fish_tick_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    state: &Mutex<AutoFishTriggerState>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoFishTickExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoFish live tick requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_auto_fish(AutoFishExecutionConfig {
        capture_size,
        auto_fishing_config: config.auto_fishing_config.clone(),
    });
    let task = plan.task_key.clone();
    let mut state = state
        .lock()
        .map_err(|_| "AutoFish trigger state lock poisoned".to_string())?;
    let result =
        execute_desktop_auto_fish_tick_live(config, window, &plan, &mut state, cancellation)?;
    Ok(DesktopAutoFishTickExecution { task, result })
}

fn execute_desktop_auto_fish_tick_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoFishExecutionPlan,
    state: &mut AutoFishTriggerState,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoFishTickExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoFish live tick cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoFish live tick requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoFish live tick requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoFish live tick requires the BitBlt capture backend".to_string());
    }

    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings)
        .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoFishRuntime::new(
        bgi_task::task_asset_root(),
        capture_size,
        capture_source,
        window.handle.0,
        cancellation,
    );
    execute_auto_fish_tick_plan(plan, state, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoFishRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoFishRuntime {
    fn new(
        template_root: PathBuf,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            capture_size,
            capture_source,
            window_handle,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "AutoFish live tick cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_fish_template(
        &self,
        capture: &ImageRegion,
        locator: &AutoFishTemplateLocator,
    ) -> bgi_task::Result<bool> {
        let object =
            desktop_auto_fish_template_object(locator, self.capture_size).map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoFish template object failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        let region = capture
            .find(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoFish template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist())
    }

    fn execute_events(&self, events: Vec<InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }
}

impl AutoFishRuntime for DesktopAutoFishRuntime {
    fn auto_fish_now_ms(&mut self, _plan: &AutoFishExecutionPlan) -> bgi_task::Result<u64> {
        current_time_ms().map_err(TaskError::VisionPlan)
    }

    fn detect_auto_fish_template(
        &mut self,
        locator: &AutoFishTemplateLocator,
    ) -> bgi_task::Result<bool> {
        let image = self.capture_bgr_image()?;
        let capture = ImageRegion::capture(image);
        self.locate_auto_fish_template(&capture, locator)
    }

    fn detect_auto_fish_bite_text_block(
        &mut self,
        _rule: &AutoFishBiteRule,
    ) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        Ok(false)
    }

    fn ocr_auto_fish_bite_text(
        &mut self,
        _rule: &AutoFishBiteRule,
    ) -> bgi_task::Result<Option<String>> {
        self.ensure_not_cancelled()?;
        Ok(None)
    }

    fn detect_auto_fish_fish_box_rects(
        &mut self,
        _plan: &AutoFishExecutionPlan,
    ) -> bgi_task::Result<Vec<Rect>> {
        self.ensure_not_cancelled()?;
        Ok(Vec::new())
    }

    fn detect_auto_fish_fishing_bar_rects(
        &mut self,
        _plan: &AutoFishExecutionPlan,
        _fish_box_rect: Rect,
    ) -> bgi_task::Result<Vec<Rect>> {
        self.ensure_not_cancelled()?;
        Ok(Vec::new())
    }

    fn dispatch_auto_fish_input(&mut self, action: AutoFishInputAction) -> bgi_task::Result<()> {
        self.execute_events(desktop_auto_fish_input_events(action))
    }

    fn update_auto_fish_overlay(
        &mut self,
        _action: &AutoFishOverlayAction,
    ) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()
    }
}

fn desktop_auto_fish_template_object(
    locator: &AutoFishTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object = image.to_recognition_object_for_screen(
        Some(locator.roi),
        locator.threshold,
        capture_size,
    )?;
    object.name = Some(locator.name.clone());
    object.template.mode = locator.match_mode;
    object.template.use_3_channels = locator.use_3_channels;
    object.template.draw_on_window = locator.draw_on_window;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_fish_input_events(action: AutoFishInputAction) -> Vec<InputEvent> {
    match action {
        AutoFishInputAction::LeftButtonClick => InputSequence::new()
            .mouse_down(MouseButton::Left)
            .delay(50)
            .mouse_up(MouseButton::Left)
            .delay(50)
            .events()
            .to_vec(),
        AutoFishInputAction::LeftButtonDown => {
            vec![InputEvent::MouseButtonDown {
                button: MouseButton::Left,
            }]
        }
        AutoFishInputAction::LeftButtonUp => {
            vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }]
        }
    }
}

fn execute_desktop_auto_pick_tick_live_plan(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    runner_pause_count: u32,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoPickTickExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoPick live tick requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_auto_pick(AutoPickExecutionConfig {
        capture_size,
        auto_pick_config: config.auto_pick_config.clone(),
        external_config: Default::default(),
    });
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_pick_tick_live(
        app_root,
        config,
        window,
        runner_pause_count,
        &plan,
        cancellation,
    )?;
    Ok(DesktopAutoPickTickExecution { task, result })
}

fn execute_desktop_auto_pick_tick_live(
    app_root: &Path,
    config: &AppConfig,
    window: &GameWindowMatch,
    runner_pause_count: u32,
    plan: &AutoPickExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoPickTickExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoPick live tick cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoPick live tick requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoPick live tick requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoPick live tick requires the BitBlt capture backend".to_string());
    }

    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings)
        .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoPickRuntime::new(
        app_root.to_path_buf(),
        bgi_task::task_asset_root(),
        capture_size,
        capture_source,
        window.handle.0,
        runner_pause_count,
        cancellation,
    );
    execute_auto_pick_tick_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoPickRuntime {
    app_root: PathBuf,
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    runner_pause_count: u32,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoPickRuntime {
    fn new(
        app_root: PathBuf,
        template_root: PathBuf,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        runner_pause_count: u32,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            app_root,
            template_root,
            capture_size,
            capture_source,
            window_handle,
            runner_pause_count,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "AutoPick live tick cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_pick_template_rect(
        &self,
        capture: &ImageRegion,
        locator: &AutoPickTemplateLocator,
    ) -> bgi_task::Result<Option<Rect>> {
        let object =
            desktop_auto_pick_template_object(locator, self.capture_size).map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoPick template object failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        let region = capture
            .find(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoPick template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region.rect))
    }

    fn detect_auto_pick_relative_icon(
        &self,
        capture: &ImageRegion,
        locator: &AutoPickRelativeTemplateLocator,
        found_pick_rect: Rect,
    ) -> bgi_task::Result<bool> {
        let Some(locator) = desktop_auto_pick_relative_template_locator(
            locator,
            found_pick_rect,
            self.capture_size,
        ) else {
            return Ok(false);
        };
        Ok(self
            .locate_auto_pick_template_rect(capture, &locator)?
            .is_some())
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }
}

impl AutoPickRuntime for DesktopAutoPickRuntime {
    fn observe_auto_pick_tick(
        &mut self,
        plan: &AutoPickExecutionPlan,
    ) -> bgi_task::Result<AutoPickTickObservation> {
        let image = self.capture_bgr_image()?;
        let scroll_icon_detected = desktop_auto_pick_scroll_icon_detected(&image, plan);
        let capture = ImageRegion::capture(image);
        let found_pick_rect =
            self.locate_auto_pick_template_rect(&capture, &plan.template_rule.pick_template)?;
        let l_key_detected = if found_pick_rect.is_some() {
            self.locate_auto_pick_template_rect(&capture, &plan.template_rule.l_key_template)?
                .is_some()
        } else {
            false
        };
        let excluded_icon_detected = if let Some(found_pick_rect) = found_pick_rect {
            self.detect_auto_pick_relative_icon(
                &capture,
                &plan.template_rule.chat_icon_template,
                found_pick_rect,
            )? || self.detect_auto_pick_relative_icon(
                &capture,
                &plan.template_rule.settings_icon_template,
                found_pick_rect,
            )?
        } else {
            false
        };

        Ok(AutoPickTickObservation {
            runner_pause_count: self.runner_pause_count,
            found_pick_rect,
            scroll_icon_detected,
            l_key_detected,
            excluded_icon_detected,
            average_text_gradient: None,
            raw_ocr_text: None,
        })
    }

    fn auto_pick_lists(
        &mut self,
        plan: &AutoPickExecutionPlan,
    ) -> bgi_task::Result<AutoPickRuntimeLists> {
        self.ensure_not_cancelled()?;
        Ok(AutoPickRuntimeLists {
            white_list: desktop_auto_pick_read_text_list(
                &self.app_root,
                &plan.config_rule.list_files.user_white_list_txt,
            )?
            .into_iter()
            .chain(plan.external_config.text_list.clone())
            .collect(),
            exact_black_list: desktop_auto_pick_read_json_string_list(
                &self.app_root,
                &plan.config_rule.list_files.default_black_list_json,
            )?
            .into_iter()
            .chain(desktop_auto_pick_read_text_list(
                &self.app_root,
                &plan.config_rule.list_files.user_black_list_txt,
            )?)
            .collect(),
            fuzzy_black_list: desktop_auto_pick_read_text_list(
                &self.app_root,
                &plan.config_rule.list_files.user_fuzzy_black_list_txt,
            )?,
        })
    }

    fn press_auto_pick_key(&mut self, key: &str) -> bgi_task::Result<()> {
        let vk = desktop_auto_pick_virtual_key(key).ok_or_else(|| {
            TaskError::VisionPlan(format!("AutoPick key is not supported on desktop: {key}"))
        })?;
        self.execute_sequence(InputSequence::new().key_press(vk))
    }

    fn scroll_auto_pick(&mut self, vertical_delta: i32) -> bgi_task::Result<()> {
        self.execute_sequence(InputSequence::new().vertical_scroll(vertical_delta))
    }

    fn delay_auto_pick(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < deadline {
            if self.cancellation.is_cancelled() {
                break;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            std::thread::sleep(remaining.min(Duration::from_millis(25)));
        }
        self.ensure_not_cancelled()
    }
}

fn desktop_auto_pick_template_object(
    locator: &AutoPickTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object = image.to_recognition_object_for_screen(
        Some(locator.region_of_interest),
        0.8,
        capture_size,
    )?;
    object.name = Some(locator.name.clone());
    object.template.draw_on_window = locator.draw_on_window;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_pick_relative_template_locator(
    locator: &AutoPickRelativeTemplateLocator,
    found_pick_rect: Rect,
    capture_size: VisionSize,
) -> Option<AutoPickTemplateLocator> {
    let scale = capture_size.height as f64 / 1080.0;
    let rect = Rect {
        x: found_pick_rect.x + ((locator.region.x_offset_1080p as f64) * scale).trunc() as i32,
        y: found_pick_rect.y + ((locator.region.y_offset_1080p as f64) * scale).trunc() as i32,
        width: ((locator.region.width_1080p as f64) * scale).trunc() as i32,
        height: match locator.region.height_source {
            bgi_task::AutoPickRelativeHeightSource::FoundPickKeyHeight => found_pick_rect.height,
        },
    };
    if rect.x < 0
        || rect.y < 0
        || rect.width <= 0
        || rect.height <= 0
        || rect.x + rect.width > capture_size.width as i32
        || rect.y + rect.height > capture_size.height as i32
    {
        return None;
    }
    Some(AutoPickTemplateLocator {
        name: locator.name.clone(),
        asset: locator.asset.clone(),
        region_of_interest: rect,
        draw_on_window: locator.draw_on_window,
    })
}

fn desktop_auto_pick_scroll_icon_detected(image: &BgrImage, plan: &AutoPickExecutionPlan) -> bool {
    let scale = plan.capture_size.height as f64 / 1080.0;
    plan.scroll_rule.probe_points.iter().all(|probe| {
        let x = ((probe.x_1080p as f64) * scale).trunc() as u32;
        let y = ((probe.y_1080p as f64) * scale).trunc() as u32;
        image.rgb_pixel_at(x, y).is_some_and(|pixel| {
            pixel.r == probe.rgb.r && pixel.g == probe.rgb.g && pixel.b == probe.rgb.b
        })
    })
}

fn desktop_auto_pick_read_text_list(
    app_root: &Path,
    relative_path: &str,
) -> bgi_task::Result<Vec<String>> {
    let path = app_root.join(relative_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| TaskError::VisionPlan(format!("AutoPick list read failed: {error}")))?;
    Ok(parse_auto_pick_text_list(&text))
}

fn desktop_auto_pick_read_json_string_list(
    app_root: &Path,
    relative_path: &str,
) -> bgi_task::Result<Vec<String>> {
    let path = app_root.join(relative_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).map_err(|error| {
        TaskError::VisionPlan(format!("AutoPick default list read failed: {error}"))
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        TaskError::VisionPlan(format!("AutoPick default list JSON parse failed: {error}"))
    })?;
    let mut strings = Vec::new();
    desktop_auto_pick_collect_json_strings(&value, &mut strings);
    Ok(strings)
}

fn desktop_auto_pick_collect_json_strings(value: &Value, strings: &mut Vec<String>) {
    match value {
        Value::String(value) if !value.is_empty() => strings.push(value.clone()),
        Value::Array(items) => {
            for item in items {
                desktop_auto_pick_collect_json_strings(item, strings);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                desktop_auto_pick_collect_json_strings(value, strings);
            }
        }
        _ => {}
    }
}

fn desktop_auto_pick_virtual_key(key: &str) -> Option<u16> {
    let normalized = key.trim();
    if normalized.len() == 1 {
        let ch = normalized.chars().next()?.to_ascii_uppercase();
        if ch.is_ascii_alphanumeric() {
            return Some(ch as u16);
        }
    }

    match normalized.to_ascii_lowercase().as_str() {
        "space" => Some(KeyId::SPACE.vk()),
        "enter" | "return" => Some(KeyId::ENTER.vk()),
        "tab" => Some(KeyId::TAB.vk()),
        "escape" | "esc" => Some(KeyId::ESCAPE.vk()),
        "up" => Some(KeyId::UP.vk()),
        "down" => Some(KeyId::DOWN.vk()),
        "left" => Some(KeyId::LEFT.vk()),
        "right" => Some(KeyId::RIGHT.vk()),
        _ => None,
    }
}

fn execute_desktop_quick_teleport_tick_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    state: &Mutex<DesktopQuickTeleportTriggerState>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopQuickTeleportTickExecution, String> {
    let window = game_window
        .ok_or_else(|| "QuickTeleport live tick requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let quick_teleport_tick_hotkey = if config
        .hot_key_config
        .quick_teleport_tick_hotkey
        .trim()
        .is_empty()
    {
        None
    } else {
        Some(config.hot_key_config.quick_teleport_tick_hotkey.clone())
    };
    let plan = plan_quick_teleport(QuickTeleportExecutionConfig {
        capture_size,
        quick_teleport_config: config.quick_teleport_config.clone(),
        quick_teleport_tick_hotkey,
    });
    let task = plan.task_key.clone();
    let mut state = state
        .lock()
        .map_err(|_| "QuickTeleport trigger state lock poisoned".to_string())?;
    let result =
        execute_desktop_quick_teleport_tick_live(config, window, &plan, &mut state, cancellation)?;
    Ok(DesktopQuickTeleportTickExecution { task, result })
}

fn execute_desktop_quick_teleport_tick_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &QuickTeleportExecutionPlan,
    state: &mut DesktopQuickTeleportTriggerState,
    cancellation: Arc<InputCancellationToken>,
) -> Result<QuickTeleportTickExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("QuickTeleport live tick cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "QuickTeleport live tick requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "QuickTeleport live tick requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("QuickTeleport live tick requires the BitBlt capture backend".to_string());
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime = DesktopQuickTeleportRuntime::new(
        bgi_task::task_asset_root(),
        global_input,
        capture_size,
        capture_source,
        DesktopQuickTeleportWindowContext {
            handle: window.handle.0,
            foreground: desktop_game_window_process_is_foreground(window),
            open_chat_vk: config.key_bindings_config.open_chat_screen.vk(),
        },
        state,
        cancellation,
    )?;
    execute_quick_teleport_tick_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Copy)]
struct DesktopQuickTeleportWindowContext {
    handle: isize,
    foreground: bool,
    open_chat_vk: u16,
}

struct DesktopQuickTeleportRuntime<'a> {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window: DesktopQuickTeleportWindowContext,
    trigger_state: &'a mut DesktopQuickTeleportTriggerState,
    last_teleport_button_region: Option<Region>,
    cancellation: Arc<InputCancellationToken>,
}

impl<'a> DesktopQuickTeleportRuntime<'a> {
    fn new(
        template_root: PathBuf,
        mut global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window: DesktopQuickTeleportWindowContext,
        trigger_state: &'a mut DesktopQuickTeleportTriggerState,
        cancellation: Arc<InputCancellationToken>,
    ) -> Result<Self, String> {
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_size,
            capture_source,
            window,
            trigger_state,
            last_teleport_button_region: None,
            cancellation,
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "QuickTeleport live tick cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let image = bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_quick_teleport_template(
        &self,
        capture: &ImageRegion,
        locator: &QuickTeleportTemplateLocator,
    ) -> bgi_task::Result<Option<Region>> {
        let object = desktop_quick_teleport_template_object(locator, self.capture_size).map_err(
            |error| {
                TaskError::VisionPlan(format!(
                    "QuickTeleport template object failed under {}: {error}",
                    self.template_root.display()
                ))
            },
        )?;
        let region = capture
            .find(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "QuickTeleport template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn locate_quick_teleport_template_matches(
        &self,
        capture: &ImageRegion,
        locator: &QuickTeleportTemplateLocator,
    ) -> bgi_task::Result<Vec<Region>> {
        let object = desktop_quick_teleport_template_object(locator, self.capture_size).map_err(
            |error| {
                TaskError::VisionPlan(format!(
                    "QuickTeleport template object failed under {}: {error}",
                    self.template_root.display()
                ))
            },
        )?;
        capture
            .find_multi(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "QuickTeleport template multi-lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })
    }

    fn detect_big_map_ui(
        &self,
        capture: &ImageRegion,
        plan: &QuickTeleportExecutionPlan,
    ) -> bgi_task::Result<bool> {
        Ok(self
            .locate_quick_teleport_template(capture, &plan.locators.map_scale_button)?
            .is_some()
            || self
                .locate_quick_teleport_template(capture, &plan.locators.map_settings_button)?
                .is_some())
    }

    fn detect_chat_ui(&self, capture: &ImageRegion) -> bgi_task::Result<DesktopChatUiDetection> {
        let back_button = desktop_chat_ui_back_button_object(self.capture_size)
            .and_then(|object| capture.find(&self.vision_backend, &object))
            .map_err(|error| {
                TaskError::VisionPlan(format!("QuickTeleport chat UI detection failed: {error}"))
            })?;
        Ok(desktop_detect_chat_ui(capture, back_button.is_exist()))
    }

    fn map_choose_candidates(
        &self,
        capture: &ImageRegion,
        plan: &QuickTeleportExecutionPlan,
    ) -> bgi_task::Result<Vec<QuickTeleportMapChooseCandidate>> {
        let mut candidates = Vec::new();
        for locator in &plan.locators.map_choose_icon_templates {
            for region in self.locate_quick_teleport_template_matches(capture, locator)? {
                let icon_rect =
                    desktop_quick_teleport_relative_candidate_icon_rect(region.rect, plan)?;
                candidates.push(QuickTeleportMapChooseCandidate {
                    icon_rect,
                    text: String::new(),
                });
            }
        }
        let mut candidates = desktop_quick_teleport_deduplicate_candidates(candidates);
        for candidate in &mut candidates {
            candidate.text =
                desktop_quick_teleport_ocr_candidate_text(capture, candidate.icon_rect, plan)?;
        }
        Ok(candidates)
    }

    fn click_capture_point(&self, x: i32, y: i32) -> bgi_task::Result<()> {
        let sequence = self
            .global_input
            .click(x, y)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        self.execute_sequence(sequence)
    }

    fn click_region_center(&self, region: &Region) -> bgi_task::Result<()> {
        let center = region.rect.center();
        self.click_capture_point(center.x, center.y)
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window.handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn delay_with_cancellation(&self, duration_ms: u64) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < deadline {
            if self.cancellation.is_cancelled() {
                break;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            std::thread::sleep(remaining.min(Duration::from_millis(25)));
        }
        self.ensure_not_cancelled()
    }
}

impl QuickTeleportRuntime for DesktopQuickTeleportRuntime<'_> {
    fn observe_quick_teleport_tick(
        &mut self,
        plan: &QuickTeleportExecutionPlan,
    ) -> bgi_task::Result<QuickTeleportDecisionInput> {
        let now = Instant::now();
        let pressed_keys = currently_pressed_keys().unwrap_or_default();
        self.trigger_state
            .chat_hotkey_guard
            .prime_from_open_chat_key(&pressed_keys, self.window.open_chat_vk, now);
        let capture = if plan.config_rule.enabled {
            let capture = self.capture_image_region()?;
            let chat_ui_detection = self.detect_chat_ui(&capture)?;
            self.trigger_state
                .chat_hotkey_guard
                .update_visual_state(chat_ui_detection, Instant::now());
            Some(capture)
        } else {
            None
        };
        let guard_now = Instant::now();
        let chat_hotkey_blocked = self
            .trigger_state
            .chat_hotkey_guard
            .should_block_hotkey("QuickTeleportTickHotkey", guard_now);
        let hotkey_pressed = desktop_quick_teleport_hotkey_pressed(
            plan,
            self.window.foreground,
            chat_hotkey_blocked,
            &pressed_keys,
            &mut self.trigger_state.hotkey_latch,
        );
        let elapsed_since_previous_tick_ms = self
            .trigger_state
            .elapsed_since_previous_tick_ms(plan, hotkey_pressed);
        let throttled = elapsed_since_previous_tick_ms <= plan.throttle_rule.tick_interval_ms;
        if !plan.config_rule.enabled
            || (plan.hotkey_rule.requires_pressed && !hotkey_pressed)
            || throttled
        {
            return Ok(QuickTeleportDecisionInput {
                elapsed_since_previous_tick_ms,
                hotkey_pressed,
                is_big_map_ui: false,
                teleport_button_detected: false,
                map_close_button_detected: false,
                map_choose_button_detected: false,
                map_choose_candidates: Vec::new(),
            });
        }

        let capture = capture.ok_or_else(|| {
            TaskError::VisionPlan("QuickTeleport enabled tick did not capture a frame".to_string())
        })?;
        let is_big_map_ui = self.detect_big_map_ui(&capture, plan)?;
        if !is_big_map_ui {
            return Ok(QuickTeleportDecisionInput {
                elapsed_since_previous_tick_ms,
                hotkey_pressed,
                is_big_map_ui,
                teleport_button_detected: false,
                map_close_button_detected: false,
                map_choose_button_detected: false,
                map_choose_candidates: Vec::new(),
            });
        }

        self.last_teleport_button_region =
            self.locate_quick_teleport_template(&capture, &plan.locators.teleport_button)?;
        let teleport_button_detected = self.last_teleport_button_region.is_some();
        let map_close_button_detected = !teleport_button_detected
            && self
                .locate_quick_teleport_template(&capture, &plan.locators.map_close_button)?
                .is_some();
        let map_choose_button_detected = !teleport_button_detected
            && !map_close_button_detected
            && self
                .locate_quick_teleport_template(&capture, &plan.locators.map_choose)?
                .is_some();
        let map_choose_candidates = if teleport_button_detected
            || map_close_button_detected
            || map_choose_button_detected
        {
            Vec::new()
        } else {
            self.map_choose_candidates(&capture, plan)?
        };

        Ok(QuickTeleportDecisionInput {
            elapsed_since_previous_tick_ms,
            hotkey_pressed,
            is_big_map_ui,
            teleport_button_detected,
            map_close_button_detected,
            map_choose_button_detected,
            map_choose_candidates,
        })
    }

    fn click_quick_teleport_button(
        &mut self,
        locator: &QuickTeleportTemplateLocator,
    ) -> bgi_task::Result<()> {
        let region = if let Some(region) = self.last_teleport_button_region.clone() {
            region
        } else {
            let capture = self.capture_image_region()?;
            self.locate_quick_teleport_template(&capture, locator)?
                .ok_or_else(|| {
                    TaskError::VisionPlan(
                        "QuickTeleport teleport button disappeared before click".to_string(),
                    )
                })?
        };
        self.click_region_center(&region)
    }

    fn delay_quick_teleport(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.delay_with_cancellation(duration_ms)
    }

    fn click_quick_teleport_candidate_text_region(
        &mut self,
        text_rect: Rect,
        _text: &str,
    ) -> bgi_task::Result<()> {
        let center = text_rect.center();
        self.click_capture_point(center.x, center.y)
    }

    fn recheck_quick_teleport_button(
        &mut self,
        locator: &QuickTeleportTemplateLocator,
    ) -> bgi_task::Result<bool> {
        let capture = self.capture_image_region()?;
        self.last_teleport_button_region =
            self.locate_quick_teleport_template(&capture, locator)?;
        Ok(self.last_teleport_button_region.is_some())
    }
}

impl DesktopQuickTeleportTriggerState {
    fn elapsed_since_previous_tick_ms(
        &mut self,
        plan: &QuickTeleportExecutionPlan,
        hotkey_pressed: bool,
    ) -> u64 {
        let now = Instant::now();
        let elapsed = self
            .previous_tick
            .and_then(|previous| now.checked_duration_since(previous))
            .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
            .unwrap_or_else(|| plan.throttle_rule.tick_interval_ms.saturating_add(1));
        if plan.config_rule.enabled
            && (!plan.hotkey_rule.requires_pressed || hotkey_pressed)
            && elapsed > plan.throttle_rule.tick_interval_ms
        {
            self.previous_tick = Some(now);
        }
        elapsed
    }
}

fn desktop_quick_teleport_template_object(
    locator: &QuickTeleportTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object = image.to_recognition_object_for_screen(
        Some(locator.roi),
        locator.threshold,
        capture_size,
    )?;
    object.name = Some(locator.name.clone());
    object.template.mode = locator.match_mode;
    object.template.use_3_channels = locator.use_3_channels;
    object.template.draw_on_window = locator.draw_on_window;
    if locator.use_grey_template_for_multi_match {
        object.template.max_match_count = 32;
    }
    object.validate()?;
    Ok(object)
}

fn desktop_chat_ui_back_button_object(
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(bgi_task::USE_REDEEM_CODE_ESC_RETURN_BUTTON)?;
    let roi = Rect::new(
        0,
        0,
        desktop_scaled_1080p(220, capture_size),
        desktop_scaled_1080p(160, capture_size),
    )?;
    let mut object = image.to_recognition_object(Some(roi), 0.72)?;
    object.name = Some("ChatBackButton".to_string());
    object.template.draw_on_window = false;
    object.validate()?;
    Ok(object)
}

fn desktop_detect_chat_ui(capture: &ImageRegion, has_back_button: bool) -> DesktopChatUiDetection {
    let scale = desktop_chat_ui_scale(capture.image.size);
    let has_more_button = desktop_chat_ui_has_more_button(&capture.image, scale);
    let has_add_conversation_button =
        desktop_chat_ui_has_add_conversation_button(&capture.image, scale);
    let bottom_circle_count = desktop_chat_ui_bottom_circle_button_count(&capture.image, scale);
    let has_send_button = desktop_chat_ui_has_send_button(&capture.image, scale);
    let state = desktop_chat_ui_state_from_parts(
        has_back_button,
        has_more_button,
        has_add_conversation_button,
        bottom_circle_count,
        has_send_button,
    );
    DesktopChatUiDetection {
        state,
        has_back_button,
        has_more_button,
        has_add_conversation_button,
        bottom_circle_count,
        has_send_button,
    }
}

fn desktop_chat_ui_state_from_parts(
    has_back_button: bool,
    has_more_button: bool,
    has_add_conversation_button: bool,
    bottom_circle_count: usize,
    has_send_button: bool,
) -> DesktopChatUiState {
    if !has_back_button || !has_add_conversation_button {
        return DesktopChatUiState::Closed;
    }
    if bottom_circle_count >= 2 || has_send_button {
        return DesktopChatUiState::InputOpen;
    }
    if has_more_button {
        DesktopChatUiState::PanelOpen
    } else {
        DesktopChatUiState::Closed
    }
}

fn desktop_chat_ui_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / 1920.0
}

fn desktop_chat_ui_scaled(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}

fn desktop_chat_ui_has_more_button(image: &BgrImage, scale: f64) -> bool {
    let roi = Rect::new(
        image.size.width as i32 - desktop_chat_ui_scaled(280.0, scale),
        0,
        desktop_chat_ui_scaled(250.0, scale),
        desktop_chat_ui_scaled(140.0, scale),
    )
    .unwrap_or_default();
    desktop_chat_ui_has_ellipsis_dots(image, roi, scale, true)
        || desktop_chat_ui_has_ellipsis_dots(image, roi, scale, false)
}

fn desktop_chat_ui_has_add_conversation_button(image: &BgrImage, scale: f64) -> bool {
    let roi = Rect::new(
        0,
        image.size.height as i32 - desktop_chat_ui_scaled(260.0, scale),
        desktop_chat_ui_scaled(320.0, scale),
        desktop_chat_ui_scaled(260.0, scale),
    )
    .unwrap_or_default();
    desktop_chat_ui_count_bright_rounded_buttons(
        image,
        roi,
        scale,
        DesktopChatUiButtonBlobRule {
            min_width: 28,
            max_width: 92,
            min_height: 28,
            max_height: 92,
            min_aspect: 0.72,
            max_aspect: 1.28,
        },
    ) > 0
}

fn desktop_chat_ui_bottom_circle_button_count(image: &BgrImage, scale: f64) -> usize {
    let roi = Rect::new(
        desktop_chat_ui_scaled(620.0, scale),
        image.size.height as i32 - desktop_chat_ui_scaled(220.0, scale),
        desktop_chat_ui_scaled(760.0, scale),
        desktop_chat_ui_scaled(180.0, scale),
    )
    .unwrap_or_default();
    desktop_chat_ui_count_bright_rounded_buttons(
        image,
        roi,
        scale,
        DesktopChatUiButtonBlobRule {
            min_width: 26,
            max_width: 92,
            min_height: 26,
            max_height: 92,
            min_aspect: 0.72,
            max_aspect: 1.28,
        },
    )
}

fn desktop_chat_ui_has_send_button(image: &BgrImage, scale: f64) -> bool {
    let roi = Rect::new(
        desktop_chat_ui_scaled(820.0, scale),
        image.size.height as i32 - desktop_chat_ui_scaled(220.0, scale),
        desktop_chat_ui_scaled(500.0, scale),
        desktop_chat_ui_scaled(180.0, scale),
    )
    .unwrap_or_default();
    desktop_chat_ui_count_bright_rounded_buttons(
        image,
        roi,
        scale,
        DesktopChatUiButtonBlobRule {
            min_width: 90,
            max_width: 260,
            min_height: 26,
            max_height: 92,
            min_aspect: 1.45,
            max_aspect: 5.5,
        },
    ) > 0
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopChatUiButtonBlobRule {
    min_width: i32,
    max_width: i32,
    min_height: i32,
    max_height: i32,
    min_aspect: f64,
    max_aspect: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DesktopChatUiBlob {
    rect: Rect,
    center_x: i32,
    center_y: i32,
    area: u32,
}

fn desktop_chat_ui_count_bright_rounded_buttons(
    image: &BgrImage,
    roi: Rect,
    scale: f64,
    rule: DesktopChatUiButtonBlobRule,
) -> usize {
    let scaled_min_width = 8.max(desktop_chat_ui_scaled(rule.min_width as f64, scale));
    let scaled_max_width =
        (scaled_min_width + 1).max(desktop_chat_ui_scaled(rule.max_width as f64, scale));
    let scaled_min_height = 8.max(desktop_chat_ui_scaled(rule.min_height as f64, scale));
    let scaled_max_height =
        (scaled_min_height + 1).max(desktop_chat_ui_scaled(rule.max_height as f64, scale));
    let min_area = scaled_min_width as f64 * scaled_min_height as f64 * 0.35;

    desktop_chat_ui_component_blobs(image, roi, |pixel| {
        pixel[0] >= 180 && pixel[1] >= 165 && pixel[2] >= 135
    })
    .into_iter()
    .filter(|blob| {
        let width = blob.rect.width;
        let height = blob.rect.height;
        if width < scaled_min_width
            || height < scaled_min_height
            || width > scaled_max_width
            || height > scaled_max_height
        {
            return false;
        }
        let aspect = width as f64 / height.max(1) as f64;
        if aspect < rule.min_aspect || aspect > rule.max_aspect {
            return false;
        }
        if blob.area as f64 <= min_area {
            return false;
        }
        let fill_ratio = blob.area as f64 / (width.max(1) * height.max(1)) as f64;
        fill_ratio >= 0.48
    })
    .count()
}

fn desktop_chat_ui_has_ellipsis_dots(
    image: &BgrImage,
    roi: Rect,
    scale: f64,
    detect_dark_dots: bool,
) -> bool {
    let mut dots = desktop_chat_ui_component_blobs(image, roi, |pixel| {
        let gray = desktop_chat_ui_gray(pixel);
        if detect_dark_dots {
            gray <= 115
        } else {
            gray >= 210
        }
    })
    .into_iter()
    .filter(|blob| {
        let min_dot = 3.max(desktop_chat_ui_scaled(4.0, scale));
        let max_dot = (min_dot + 1).max(desktop_chat_ui_scaled(22.0, scale));
        let width = blob.rect.width;
        let height = blob.rect.height;
        if width < min_dot || height < min_dot || width > max_dot || height > max_dot {
            return false;
        }
        let aspect = width as f64 / height.max(1) as f64;
        (0.55..=1.8).contains(&aspect)
    })
    .collect::<Vec<_>>();
    if dots.len() < 3 {
        return false;
    }

    dots.sort_by_key(|dot| dot.center_x);
    let max_y_offset = 4.max(desktop_chat_ui_scaled(10.0, scale));
    let min_gap = 2.max(desktop_chat_ui_scaled(3.0, scale));
    let max_gap = (min_gap + 1).max(desktop_chat_ui_scaled(36.0, scale));
    for window in dots.windows(3) {
        let first = window[0];
        let second = window[1];
        let third = window[2];
        if (first.center_y - second.center_y).abs() > max_y_offset
            || (second.center_y - third.center_y).abs() > max_y_offset
            || (first.center_y - third.center_y).abs() > max_y_offset
        {
            continue;
        }
        let gap1 = second.center_x - first.center_x;
        let gap2 = third.center_x - second.center_x;
        if gap1 >= min_gap && gap2 >= min_gap && gap1 <= max_gap && gap2 <= max_gap {
            return true;
        }
    }
    false
}

fn desktop_chat_ui_component_blobs<F>(
    image: &BgrImage,
    roi: Rect,
    mut matches: F,
) -> Vec<DesktopChatUiBlob>
where
    F: FnMut([u8; 3]) -> bool,
{
    let Ok(roi) = roi.clamp_to(image.size) else {
        return Vec::new();
    };
    if roi.width <= 0 || roi.height <= 0 {
        return Vec::new();
    }

    let width = roi.width as usize;
    let height = roi.height as usize;
    let mut visited = vec![false; width * height];
    let mut blobs = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if visited[index] {
                continue;
            }
            visited[index] = true;
            let global_x = roi.x + x as i32;
            let global_y = roi.y + y as i32;
            let Some(pixel) = image.bgr_pixel_at(global_x as u32, global_y as u32) else {
                continue;
            };
            if !matches([pixel.b, pixel.g, pixel.r]) {
                continue;
            }

            let mut stack = vec![(x, y)];
            let mut min_x = global_x;
            let mut min_y = global_y;
            let mut max_x = global_x;
            let mut max_y = global_y;
            let mut area = 0u32;
            while let Some((current_x, current_y)) = stack.pop() {
                let current_global_x = roi.x + current_x as i32;
                let current_global_y = roi.y + current_y as i32;
                area += 1;
                min_x = min_x.min(current_global_x);
                min_y = min_y.min(current_global_y);
                max_x = max_x.max(current_global_x);
                max_y = max_y.max(current_global_y);

                let neighbors = [
                    (current_x.wrapping_sub(1), current_y, current_x > 0),
                    (current_x + 1, current_y, current_x + 1 < width),
                    (current_x, current_y.wrapping_sub(1), current_y > 0),
                    (current_x, current_y + 1, current_y + 1 < height),
                ];
                for (next_x, next_y, valid) in neighbors {
                    if !valid {
                        continue;
                    }
                    let next_index = next_y * width + next_x;
                    if visited[next_index] {
                        continue;
                    }
                    visited[next_index] = true;
                    let next_global_x = roi.x + next_x as i32;
                    let next_global_y = roi.y + next_y as i32;
                    let Some(pixel) =
                        image.bgr_pixel_at(next_global_x as u32, next_global_y as u32)
                    else {
                        continue;
                    };
                    if matches([pixel.b, pixel.g, pixel.r]) {
                        stack.push((next_x, next_y));
                    }
                }
            }

            let rect =
                Rect::new(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).unwrap_or_default();
            blobs.push(DesktopChatUiBlob {
                rect,
                center_x: rect.center().x,
                center_y: rect.center().y,
                area,
            });
        }
    }
    blobs
}

fn desktop_chat_ui_gray(pixel: [u8; 3]) -> u8 {
    (0.114 * pixel[0] as f64 + 0.587 * pixel[1] as f64 + 0.299 * pixel[2] as f64).round() as u8
}

fn desktop_quick_teleport_relative_candidate_icon_rect(
    icon_rect: Rect,
    plan: &QuickTeleportExecutionPlan,
) -> bgi_task::Result<Rect> {
    let roi = plan.locators.map_choose_icon_roi;
    Rect::new(
        icon_rect.x - roi.x,
        icon_rect.y - roi.y,
        icon_rect.width,
        icon_rect.height,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn desktop_quick_teleport_candidate_text_rect(
    icon_rect: Rect,
    plan: &QuickTeleportExecutionPlan,
) -> Rect {
    Rect {
        x: plan.locators.map_choose_icon_roi.x + icon_rect.x + icon_rect.width,
        y: plan.locators.map_choose_icon_roi.y
            + icon_rect.y
            + plan.text_ocr_rule.text_region_y_offset,
        width: plan.text_ocr_rule.text_region_width,
        height: icon_rect.height + plan.text_ocr_rule.text_region_height_padding,
    }
}

fn desktop_quick_teleport_ocr_candidate_text(
    capture: &ImageRegion,
    icon_rect: Rect,
    plan: &QuickTeleportExecutionPlan,
) -> bgi_task::Result<String> {
    let text_rect = desktop_quick_teleport_candidate_text_rect(icon_rect, plan);
    let roi = desktop_ocr_roi_for_image(capture.image.size, text_rect)?;
    let cropped = capture.derive_crop(roi).map_err(|error| {
        TaskError::VisionPlan(format!("QuickTeleport candidate text crop failed: {error}"))
    })?;

    if plan.text_ocr_rule.standard_uses_color_range_and_ocr {
        let image = desktop_quick_teleport_color_range_ocr_image(&cropped.image, plan)?;
        let regions = desktop_winrt_ocr_bgr_image(&image).map_err(|error| {
            TaskError::VisionPlan(format!(
                "QuickTeleport candidate color-range OCR failed: {error}"
            ))
        })?;
        let text = desktop_quick_teleport_ocr_text_from_regions(&regions);
        if !text.trim().is_empty() || !plan.text_ocr_rule.hdr_uses_plain_ocr {
            return Ok(text.trim().to_string());
        }
    }

    if plan.text_ocr_rule.hdr_uses_plain_ocr {
        let regions = desktop_winrt_ocr_bgr_image(&cropped.image).map_err(|error| {
            TaskError::VisionPlan(format!("QuickTeleport candidate plain OCR failed: {error}"))
        })?;
        return Ok(desktop_quick_teleport_ocr_text_from_regions(&regions)
            .trim()
            .to_string());
    }

    Ok(String::new())
}

fn desktop_quick_teleport_color_range_ocr_image(
    image: &BgrImage,
    plan: &QuickTeleportExecutionPlan,
) -> bgi_task::Result<BgrImage> {
    let lower = plan.text_ocr_rule.standard_capture_lower_bgr;
    let upper = plan.text_ocr_rule.standard_capture_upper_bgr;
    let mut pixels = Vec::with_capacity(image.pixels.len());
    for pixel in image.pixels.chunks_exact(3) {
        let in_range = pixel[0] >= lower.b
            && pixel[0] <= upper.b
            && pixel[1] >= lower.g
            && pixel[1] <= upper.g
            && pixel[2] >= lower.r
            && pixel[2] <= upper.r;
        if in_range {
            pixels.extend_from_slice(&[255, 255, 255]);
        } else {
            pixels.extend_from_slice(&[0, 0, 0]);
        }
    }
    BgrImage::new(image.size, pixels).map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn desktop_quick_teleport_ocr_text_from_regions(regions: &[OcrResultRegion]) -> String {
    let mut regions = regions
        .iter()
        .filter(|region| !region.text.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));

    let mut lines: Vec<(i32, i32, Vec<OcrResultRegion>)> = Vec::new();
    for region in regions {
        let center_y = region.rect.center().y;
        let height = region.rect.height.max(1);
        let Some((line_y, line_height, line_regions)) =
            lines.iter_mut().find(|(line_y, line_height, _)| {
                let tolerance = ((*line_height).max(height) / 2).max(4);
                (center_y - *line_y).abs() <= tolerance
            })
        else {
            lines.push((center_y, height, vec![region]));
            continue;
        };
        let line_len = line_regions.len() as i32;
        *line_y = ((*line_y * line_len) + center_y) / (line_len + 1);
        *line_height = (*line_height).max(height);
        line_regions.push(region);
    }

    lines
        .into_iter()
        .map(|(_, _, mut line_regions)| {
            line_regions.sort_by_key(|region| region.rect.center().x);
            line_regions
                .into_iter()
                .map(|region| region.text)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn desktop_quick_teleport_hotkey_pressed(
    plan: &QuickTeleportExecutionPlan,
    game_window_foreground: bool,
    chat_hotkey_blocked: bool,
    pressed_keys: &[u16],
    latch: &mut DesktopQuickTeleportHotkeyLatchState,
) -> bool {
    if !plan.hotkey_rule.requires_pressed {
        latch.update(None, false, game_window_foreground, chat_hotkey_blocked);
        return true;
    }
    let configured_vk = desktop_quick_teleport_configured_hotkey_vk(plan);
    let physical_pressed = configured_vk.is_some_and(|vk| pressed_keys.contains(&vk));
    latch.update(
        configured_vk,
        physical_pressed,
        game_window_foreground,
        chat_hotkey_blocked,
    )
}

fn desktop_quick_teleport_configured_hotkey_vk(plan: &QuickTeleportExecutionPlan) -> Option<u16> {
    if !plan.hotkey_rule.requires_pressed {
        return None;
    }
    plan.hotkey_rule
        .configured_tick_hotkey
        .as_deref()
        .and_then(desktop_quick_teleport_legacy_hold_hotkey_vk)
}

fn desktop_quick_teleport_legacy_hold_hotkey_vk(hotkey: &str) -> Option<u16> {
    let hotkey = hotkey.trim();
    if hotkey.is_empty() || hotkey.eq_ignore_ascii_case("< none >") {
        return None;
    }
    if hotkey.contains('+') {
        return None;
    }
    desktop_quick_teleport_legacy_mouse_hotkey_vk(hotkey)
        .or_else(|| desktop_quick_teleport_legacy_keyboard_hotkey_vk(hotkey))
}

fn desktop_quick_teleport_legacy_mouse_hotkey_vk(value: &str) -> Option<u16> {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '-'], "");
    match normalized.as_str() {
        "xbutton1" | "x1" | "mousexbutton1" | "mousesidebutton1" | "sidebutton1" => {
            Some(KeyId::MOUSE_SIDE_BUTTON1.vk())
        }
        "xbutton2" | "x2" | "mousexbutton2" | "mousesidebutton2" | "sidebutton2" => {
            Some(KeyId::MOUSE_SIDE_BUTTON2.vk())
        }
        _ => None,
    }
}

fn desktop_quick_teleport_legacy_keyboard_hotkey_vk(value: &str) -> Option<u16> {
    let normalized = value.trim();
    if normalized.len() == 1 {
        let ch = normalized.chars().next()?.to_ascii_uppercase();
        if ch.is_ascii_alphanumeric() {
            return Some(ch as u16);
        }
    }

    let lower = normalized.to_ascii_lowercase();
    if let Some(number) = lower
        .strip_prefix('f')
        .and_then(|text| text.parse::<u16>().ok())
    {
        if (1..=24).contains(&number) {
            return Some(0x70 + number - 1);
        }
    }
    if let Some(digit) = lower
        .strip_prefix('d')
        .and_then(desktop_quick_teleport_single_digit)
    {
        return Some(0x30 + digit);
    }
    if let Some(digit) = lower
        .strip_prefix("numpad")
        .and_then(desktop_quick_teleport_single_digit)
    {
        return Some(KeyId::NUM_PAD0.vk() + digit);
    }

    match lower.as_str() {
        "space" => Some(KeyId::SPACE.vk()),
        "back" | "backspace" => Some(KeyId::BACKSPACE.vk()),
        "delete" | "del" => Some(KeyId::DELETE.vk()),
        "insert" | "ins" => Some(KeyId::INSERT.vk()),
        "home" => Some(KeyId::HOME.vk()),
        "end" => Some(KeyId::END.vk()),
        "pageup" | "prior" => Some(KeyId::PAGE_UP.vk()),
        "pagedown" | "next" => Some(KeyId::PAGE_DOWN.vk()),
        "left" => Some(KeyId::LEFT.vk()),
        "up" => Some(KeyId::UP.vk()),
        "right" => Some(KeyId::RIGHT.vk()),
        "down" => Some(KeyId::DOWN.vk()),
        "numlock" => Some(KeyId::NUM_LOCK.vk()),
        "decimal" => Some(KeyId::DECIMAL.vk()),
        "oemperiod" => Some(KeyId::PERIOD.vk()),
        "add" => Some(KeyId::ADD.vk()),
        "divide" => Some(KeyId::DIVIDE.vk()),
        "multiply" => Some(KeyId::MULTIPLY.vk()),
        "subtract" => Some(KeyId::SUBTRACT.vk()),
        "oemminus" | "minus" => Some(KeyId::MINUS.vk()),
        "oemplus" | "equal" => Some(KeyId::EQUAL.vk()),
        "oemcomma" | "comma" => Some(KeyId::COMMA.vk()),
        "oemquestion" | "slash" => Some(KeyId::SLASH.vk()),
        "oemquotes" | "oem7" | "apostrophe" => Some(KeyId::APOSTROPHE.vk()),
        "oem1" | "semicolon" => Some(KeyId::SEMICOLON.vk()),
        "oemopenbrackets" => Some(KeyId::LEFT_SQUARE_BRACKET.vk()),
        "oemclosebrackets" => Some(KeyId::RIGHT_SQUARE_BRACKET.vk()),
        "oem3" | "tilde" => Some(KeyId::TILDE.vk()),
        "oem5" => Some(0xDC),
        "oem102" | "backslash" => Some(KeyId::BACKSLASH.vk()),
        _ => None,
    }
}

fn desktop_quick_teleport_single_digit(value: &str) -> Option<u16> {
    if value.len() == 1 {
        let digit = value.as_bytes()[0];
        if digit.is_ascii_digit() {
            return Some((digit - b'0') as u16);
        }
    }
    None
}

fn desktop_quick_teleport_deduplicate_candidates(
    mut candidates: Vec<QuickTeleportMapChooseCandidate>,
) -> Vec<QuickTeleportMapChooseCandidate> {
    candidates.sort_by_key(|candidate| (candidate.icon_rect.y, candidate.icon_rect.x));
    let mut deduplicated: Vec<QuickTeleportMapChooseCandidate> = Vec::new();
    for candidate in candidates {
        if deduplicated.iter().any(|existing| {
            desktop_quick_teleport_rects_overlap(existing.icon_rect, candidate.icon_rect)
        }) {
            continue;
        }
        deduplicated.push(candidate);
    }
    deduplicated
}

fn desktop_quick_teleport_rects_overlap(left: Rect, right: Rect) -> bool {
    let x1 = left.x.max(right.x);
    let y1 = left.y.max(right.y);
    let x2 = (left.x + left.width).min(right.x + right.width);
    let y2 = (left.y + left.height).min(right.y + right.height);
    let intersection_width = (x2 - x1).max(0);
    let intersection_height = (y2 - y1).max(0);
    let intersection_area = intersection_width * intersection_height;
    let left_area = left.width.max(0) * left.height.max(0);
    let right_area = right.width.max(0) * right.height.max(0);
    let smaller_area = left_area.min(right_area);
    smaller_area > 0 && intersection_area * 2 >= smaller_area
}

fn execute_desktop_script_dispatcher_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    plan: &ScriptDispatcherExecutionPlan,
) -> bgi_task::Result<Option<ScriptDispatcherLiveExecutionReport>> {
    match plan {
        ScriptDispatcherExecutionPlan::AutoCook(plan) => {
            let Some(window) = game_window else {
                return Err(TaskError::CommonJobExecution(
                    "AutoCook live execution requires a detected game window".to_string(),
                ));
            };
            execute_desktop_auto_cook_live(config, window, plan, cancellation)
                .map(ScriptDispatcherLiveExecutionReport::AutoCook)
                .map(Some)
                .map_err(TaskError::CommonJobExecution)
        }
        ScriptDispatcherExecutionPlan::AutoEatFood(plan) => {
            execute_desktop_auto_eat_food_live(config, game_window, plan, cancellation)
                .map(ScriptDispatcherLiveExecutionReport::AutoEatFood)
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn execute_desktop_auto_eat_food_live(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    plan: &AutoEatFoodExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> bgi_task::Result<AutoEatFoodExecutionReport> {
    if cancellation.is_cancelled() {
        return Err(TaskError::CommonJobExecution(
            "AutoEatFood live execution cancelled".to_string(),
        ));
    }
    if matches!(plan.mode, AutoEatFoodPlanMode::InventoryFood { .. }) {
        desktop_inventory_live_preflight(
            config,
            game_window,
            "AutoEatFood inventory-food",
            plan.capture_size,
            &cancellation,
        )
        .map_err(TaskError::CommonJobExecution)?;
        desktop_auto_eat_food_inventory_live_preflight(plan)
            .map_err(TaskError::CommonJobExecution)?;
        return Err(TaskError::CommonJobExecution(
            "AutoEatFood inventory-food live execution requires desktop runtime adapter wiring after preflight"
                .to_string(),
        ));
    }

    let mut runtime = DesktopAutoEatFoodRuntime::new(cancellation);
    execute_auto_eat_food_plan(plan, &mut runtime)
}

struct DesktopAutoEatFoodRuntime {
    cancellation: Arc<InputCancellationToken>,
    logs: Vec<String>,
}

impl DesktopAutoEatFoodRuntime {
    fn new(cancellation: Arc<InputCancellationToken>) -> Self {
        Self {
            cancellation,
            logs: Vec::new(),
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoEatFood live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }
}

impl AutoEatFoodRuntime for DesktopAutoEatFoodRuntime {
    fn execute_auto_eat_food_common_job(
        &mut self,
        task_key: &str,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(format!(
            "AutoEatFood desktop runtime cannot execute common job {task_key} without inventory live adapters"
        )))
    }

    fn open_auto_eat_food_inventory(
        &mut self,
        _rule: &CountInventoryOpenInventoryRule,
    ) -> bgi_task::Result<CountInventoryOpenInventoryOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood desktop inventory opening adapter is not wired".to_string(),
        ))
    }

    fn confirm_auto_eat_food_expired_item_prompt(
        &mut self,
        _confirm_asset: &str,
        _crop_bottom_ratio: f64,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood expired-item prompt adapter is not wired".to_string(),
        ))
    }

    fn open_auto_eat_food_inventory_tab(
        &mut self,
        _rule: &CountInventoryOpenInventoryRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood inventory tab adapter is not wired".to_string(),
        ))
    }

    fn load_auto_eat_food_grid_icon_classifier(
        &mut self,
        _rule: &GridIconClassifierRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood GridIcon classifier adapter is not wired".to_string(),
        ))
    }

    fn enumerate_auto_eat_food_grid_items(
        &mut self,
        _template: &GridTemplate,
        _detection_rule: &GridItemDetectionRule,
        _scroll_rule: &GridScrollRule,
    ) -> bgi_task::Result<Vec<CountInventoryGridItemFrame>> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood grid enumeration adapter is not wired".to_string(),
        ))
    }

    fn crop_auto_eat_food_grid_icons(
        &mut self,
        _items: &[CountInventoryGridItemFrame],
        _rule: &GridIconCropRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood grid icon crop adapter is not wired".to_string(),
        ))
    }

    fn infer_auto_eat_food_grid_icons(
        &mut self,
        _items: &[CountInventoryGridItemFrame],
        _rule: &GridIconClassifierRule,
    ) -> bgi_task::Result<Vec<CountInventoryGridIconMatch>> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood GridIcon inference adapter is not wired".to_string(),
        ))
    }

    fn ocr_auto_eat_food_item_count(
        &mut self,
        _matched: &CountInventoryGridIconMatch,
        _rule: &GridItemCountOcrRule,
    ) -> bgi_task::Result<Option<String>> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood item-count OCR adapter is not wired".to_string(),
        ))
    }

    fn click_auto_eat_food_item(
        &mut self,
        _matched: &CountInventoryGridIconMatch,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood matched-item click adapter is not wired".to_string(),
        ))
    }

    fn delay_auto_eat_food_after_item_click(
        &mut self,
        duration_ms: u64,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        std::thread::sleep(Duration::from_millis(duration_ms));
        self.ensure_not_cancelled()?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn click_auto_eat_food_white_confirm_if_present(
        &mut self,
        _asset: &str,
    ) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        Err(TaskError::CommonJobExecution(
            "AutoEatFood white-confirm click adapter is not wired".to_string(),
        ))
    }

    fn clear_auto_eat_food_vision_drawings(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn log_auto_eat_food(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.ensure_not_cancelled()?;
        self.logs.push(message.to_string());
        Ok(CommonJobRuntimeOutcome::None)
    }
}

fn desktop_auto_eat_food_inventory_live_preflight(
    plan: &AutoEatFoodExecutionPlan,
) -> Result<(), String> {
    let inventory_plan = plan.inventory_plan.as_ref().ok_or_else(|| {
        "AutoEatFood inventory-food live execution requires an inventory plan".to_string()
    })?;
    desktop_inventory_count_plan_live_preflight("AutoEatFood inventory-food", inventory_plan)?;

    for step in &plan.steps {
        if !desktop_auto_eat_food_preflight_condition_applies(plan, step.condition) {
            continue;
        }

        match &step.action {
            AutoEatFoodStepAction::LogInfo { .. }
            | AutoEatFoodStepAction::LogWarning { .. }
            | AutoEatFoodStepAction::ReturnMainUi
            | AutoEatFoodStepAction::OpenFoodInventoryTab { .. }
            | AutoEatFoodStepAction::LoadGridIconClassifier { .. }
            | AutoEatFoodStepAction::EnumerateGridItems { .. }
            | AutoEatFoodStepAction::CropGridIcon { .. }
            | AutoEatFoodStepAction::InferGridIcon { .. }
            | AutoEatFoodStepAction::OcrMatchedFoodCount { .. }
            | AutoEatFoodStepAction::DelayAfterItemClick { .. }
            | AutoEatFoodStepAction::ClearVisionDrawings
            | AutoEatFoodStepAction::ReturnResult { .. } => {}
            AutoEatFoodStepAction::PortableNutritionBagLoop => {
                return Err(
                    "AutoEatFood inventory-food live execution requires desktop portable nutrition bag adapter"
                        .to_string(),
                );
            }
            AutoEatFoodStepAction::ClickMatchedFoodItem { .. } => {
                return Err(
                    "AutoEatFood inventory-food live execution requires desktop matched-food click adapter"
                        .to_string(),
                );
            }
            AutoEatFoodStepAction::ConfirmUseFoodIfVisible { .. } => {
                return Err(
                    "AutoEatFood inventory-food live execution requires desktop white-confirm click adapter"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}

fn desktop_auto_eat_food_preflight_condition_applies(
    plan: &AutoEatFoodExecutionPlan,
    condition: AutoEatFoodStepCondition,
) -> bool {
    match condition {
        AutoEatFoodStepCondition::Always | AutoEatFoodStepCondition::Finally => true,
        AutoEatFoodStepCondition::WhenInventoryFoodMode
        | AutoEatFoodStepCondition::WhenClassifierMatchesFood
        | AutoEatFoodStepCondition::WhenCountOcrFails => {
            matches!(plan.mode, AutoEatFoodPlanMode::InventoryFood { .. })
        }
        AutoEatFoodStepCondition::WhenPortableNutritionBagMode => {
            matches!(plan.mode, AutoEatFoodPlanMode::PortableNutritionBagLoop)
        }
        AutoEatFoodStepCondition::WhenDefaultFoodMissing => {
            matches!(plan.mode, AutoEatFoodPlanMode::MissingDefaultFood { .. })
        }
    }
}

fn execute_desktop_common_job_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    plan: &CommonJobExecutionPlan,
) -> bgi_task::Result<Option<CommonJobLiveExecutionReport>> {
    if !matches!(
        plan,
        CommonJobExecutionPlan::ReturnMainUi(_)
            | CommonJobExecutionPlan::CountInventoryItem(_)
            | CommonJobExecutionPlan::SetTime(_)
            | CommonJobExecutionPlan::ChooseTalkOption(_)
            | CommonJobExecutionPlan::CheckRewards(_)
            | CommonJobExecutionPlan::BlessingOfTheWelkinMoon(_)
            | CommonJobExecutionPlan::ClaimBattlePassRewards(_)
            | CommonJobExecutionPlan::ClaimEncounterPointsRewards(_)
            | CommonJobExecutionPlan::ClaimMailRewards(_)
            | CommonJobExecutionPlan::WonderlandCycle(_)
            | CommonJobExecutionPlan::Relogin(_)
            | CommonJobExecutionPlan::SwitchParty(_)
            | CommonJobExecutionPlan::Teleport(_)
            | CommonJobExecutionPlan::OneKeyExpedition(_)
            | CommonJobExecutionPlan::GoToCraftingBench(_)
            | CommonJobExecutionPlan::GoToAdventurersGuild(_)
            | CommonJobExecutionPlan::GoToSereniteaPot(_)
            | CommonJobExecutionPlan::ScanPickDrops(_)
            | CommonJobExecutionPlan::LowerHeadThenWalkTo(_)
            | CommonJobExecutionPlan::WalkToF(_)
    ) {
        return Ok(None);
    }

    let task_key = plan.task_key().to_string();
    let Some(window) = game_window else {
        return Err(TaskError::CommonJobExecution(format!(
            "{task_key} live execution requires a detected game window"
        )));
    };

    let execution = match plan {
        CommonJobExecutionPlan::ReturnMainUi(plan) => {
            execute_desktop_return_main_ui_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::ReturnMainUi)
        }
        CommonJobExecutionPlan::CountInventoryItem(plan) => {
            execute_desktop_count_inventory_item_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::CountInventoryItem)
        }
        CommonJobExecutionPlan::SetTime(plan) => {
            execute_desktop_set_time_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::SetTime)
        }
        CommonJobExecutionPlan::ChooseTalkOption(plan) => {
            execute_desktop_choose_talk_option_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::ChooseTalkOption)
        }
        CommonJobExecutionPlan::CheckRewards(plan) => {
            execute_desktop_check_rewards_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::CheckRewards)
        }
        CommonJobExecutionPlan::BlessingOfTheWelkinMoon(plan) => {
            execute_desktop_blessing_of_the_welkin_moon_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon)
        }
        CommonJobExecutionPlan::ClaimBattlePassRewards(plan) => {
            execute_desktop_claim_battle_pass_rewards_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::ClaimBattlePassRewards)
        }
        CommonJobExecutionPlan::ClaimEncounterPointsRewards(plan) => {
            execute_desktop_claim_encounter_points_rewards_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::ClaimEncounterPointsRewards)
        }
        CommonJobExecutionPlan::ClaimMailRewards(plan) => {
            execute_desktop_claim_mail_rewards_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::ClaimMailRewards)
        }
        CommonJobExecutionPlan::WonderlandCycle(plan) => {
            execute_desktop_wonderland_cycle_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::WonderlandCycle)
        }
        CommonJobExecutionPlan::Relogin(plan) => {
            execute_desktop_relogin_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::Relogin)
        }
        CommonJobExecutionPlan::SwitchParty(plan) => {
            execute_desktop_switch_party_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::SwitchParty)
        }
        CommonJobExecutionPlan::Teleport(plan) => {
            execute_desktop_teleport_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::Teleport)
        }
        CommonJobExecutionPlan::OneKeyExpedition(plan) => {
            execute_desktop_one_key_expedition_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::OneKeyExpedition)
        }
        CommonJobExecutionPlan::GoToCraftingBench(plan) => {
            execute_desktop_go_to_crafting_bench_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::GoToCraftingBench)
        }
        CommonJobExecutionPlan::GoToAdventurersGuild(plan) => {
            execute_desktop_go_to_adventurers_guild_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::GoToAdventurersGuild)
        }
        CommonJobExecutionPlan::GoToSereniteaPot(plan) => execute_desktop_go_to_serenitea_pot_live(
            config,
            window,
            plan,
            Arc::clone(&cancellation),
        )
        .map(CommonJobLiveExecutionReport::GoToSereniteaPot),
        CommonJobExecutionPlan::ScanPickDrops(plan) => {
            execute_desktop_scan_pick_drops_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::ScanPickDrops)
        }
        CommonJobExecutionPlan::LowerHeadThenWalkTo(plan) => {
            execute_desktop_lower_head_then_walk_to_live(
                config,
                window,
                plan,
                Arc::clone(&cancellation),
            )
            .map(CommonJobLiveExecutionReport::LowerHeadThenWalkTo)
        }
        CommonJobExecutionPlan::WalkToF(plan) => {
            execute_desktop_walk_to_f_live(config, window, plan, Arc::clone(&cancellation))
                .map(CommonJobLiveExecutionReport::WalkToF)
        }
        _ => return Ok(None),
    };

    execution.map(Some).map_err(TaskError::CommonJobExecution)
}

fn execute_desktop_independent_task_live_result(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    result: &mut TaskInvocationExecutionResult,
) {
    let mut live_executor = |plan: &bgi_task::TaskInvocationPlan| {
        execute_desktop_independent_task_live_plan(
            app_root,
            config,
            game_window,
            Arc::clone(&cancellation),
            plan,
        )
    };
    execute_independent_task_live_if_available(result, &mut live_executor);
}

fn execute_desktop_independent_task_live_plan(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    plan: &bgi_task::TaskInvocationPlan,
) -> bgi_task::Result<Option<IndependentTaskLiveExecutionReport>> {
    match plan.task_key.as_deref() {
        Some("AutoPathing") => {
            let auto_pathing_config = AutoPathingExecutionConfig::from_value(plan.config.as_ref());
            let auto_pathing_plan = plan_auto_pathing(app_root, &auto_pathing_config.route)?;
            let report = execute_desktop_auto_pathing_action_boundary_live_plan(
                config,
                game_window,
                cancellation,
                &auto_pathing_plan,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(
                IndependentTaskLiveExecutionReport::AutoPathingActionBoundary(report),
            ))
        }
        Some("AutoFight") => match desktop_auto_fight_live_route_mode(plan.config.as_ref())? {
            DesktopAutoFightLiveRouteMode::FinishProbe => {
                let auto_fight_config = AutoFightExecutionConfig::from_value(plan.config.as_ref());
                let auto_fight_plan = plan_auto_fight(app_root, auto_fight_config.param)?;
                let mode = desktop_auto_fight_finish_probe_execution_mode(plan.config.as_ref());
                let report = execute_desktop_auto_fight_finish_probe_live_plan(
                    config,
                    game_window,
                    &auto_fight_plan,
                    mode,
                    cancellation,
                )
                .map_err(TaskError::CommonJobExecution)?;
                Ok(Some(IndependentTaskLiveExecutionReport::AutoFightFinishProbe(
                    report,
                )))
            }
            DesktopAutoFightLiveRouteMode::FullFightLoop => Err(TaskError::CommonJobExecution(
                "AutoFight full fight-loop live execution requires native CombatScenes, team recognition, skill cooldown, burst readiness, command-loop dispatch, cleanup, and post-fight pickup adapters; use mode=finishProbe for the migrated finish-detection probe boundary".to_string(),
            )),
        },
        Some(AUTO_DOMAIN_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_DOMAIN_TASK_KEY,
            "capture, map teleport, OCR/template matching, input dispatch, CombatScenes, YOLO tree detection, reward recognition, artifact-salvage handoff, cancellation, and notification",
        )),
        Some(AUTO_GENIUS_INVOKATION_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_GENIUS_INVOKATION_TASK_KEY,
            "capture, OCR/template matching, card recognition, input dispatch, duel-state tracking, script/strategy execution, reward handling, and notification",
        )),
        Some(AUTO_TRACK_PATH_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_TRACK_PATH_TASK_KEY,
            "TpTask, mini-map capture, map matching, orientation computation, mouse/key input, overlay, and cancellation",
        )),
        Some(AUTO_BOSS_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_BOSS_TASK_KEY,
            "capture, OCR/template matching, pathing/key-mouse execution, CombatScenes, reward recognition, notification, and cancellation",
        )),
        Some(AUTO_LEY_LINE_OUTCROP_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_LEY_LINE_OUTCROP_TASK_KEY,
            "capture, OCR/template execution, TpTask/PathExecutor, key-mouse dispatch, CombatScenes, ScanPickTask, mask overlay, and notification",
        )),
        Some(AUTO_STYGIAN_ONSLAUGHT_TASK_KEY) => Err(desktop_independent_live_adapter_gap(
            AUTO_STYGIAN_ONSLAUGHT_TASK_KEY,
            "capture, OCR/template detection, combat command loop, input dispatch, reward/resin handling, artifact-salvage handoff, and notification",
        )),
        Some("AutoArtifactSalvage") => Err(TaskError::CommonJobExecution(
            "AutoArtifactSalvage desktop live adapter remains pending: capture, OCR, OpenCV, ONNX, input/click, overlay, ClearScript-compatible filtering, and destructive confirmation adapters are not wired".to_string(),
        )),
        Some("GetGridIcons") => Err(TaskError::CommonJobExecution(
            "GetGridIcons desktop live adapter remains pending: capture, OpenCV enumeration, Paddle OCR, input/click, filesystem PNG save, overlay cleanup, and optional ONNX/prototype adapters are not wired".to_string(),
        )),
        Some(USE_REDEEM_CODE_TASK_KEY) => {
            let redeem_config = UseRedeemCodeExecutionConfig::from_value(plan.config.as_ref());
            if redeem_config.codes.is_empty() {
                return Err(TaskError::CommonJobExecution(
                    "UseRedeemCode live execution requires at least one redeem code".to_string(),
                ));
            }

            let (_plan, report) = execute_desktop_use_redeem_code_live_plan(
                app_root,
                config,
                game_window,
                redeem_config.codes,
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;

            Ok(Some(IndependentTaskLiveExecutionReport::UseRedeemCode(
                report,
            )))
        }
        Some(AUTO_OPEN_CHEST_TASK_KEY) => {
            let execution = execute_desktop_auto_open_chest_live_plan(
                app_root,
                config,
                game_window,
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::AutoOpenChest(
                execution.result,
            )))
        }
        Some(QUICK_BUY_TASK_KEY) => {
            let execution = execute_desktop_quick_buy_live_plan(config, game_window, cancellation)
                .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::QuickBuy(
                execution.result,
            )))
        }
        Some(QUICK_SERENITEA_POT_TASK_KEY) => {
            let execution =
                execute_desktop_quick_serenitea_pot_live_plan(config, game_window, cancellation)
                    .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::QuickSereniteaPot(
                execution.result,
            )))
        }
        Some(AUTO_WOOD_TASK_KEY) => {
            let execution = execute_desktop_auto_wood_live_plan(
                config,
                game_window,
                plan.config.as_ref(),
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::AutoWood(
                execution.result,
            )))
        }
        Some(AUTO_TRACK_TASK_KEY) => {
            let execution = execute_desktop_auto_track_live_plan(
                config,
                game_window,
                plan.config.as_ref(),
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::AutoTrack(
                execution.result,
            )))
        }
        Some(TURN_AROUND_MACRO_TASK_KEY) => {
            let report = execute_desktop_turn_around_macro_live_plan(
                config,
                game_window,
                plan.config.as_ref(),
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(IndependentTaskLiveExecutionReport::TurnAroundMacro(
                report,
            )))
        }
        Some(QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY) => {
            let report = execute_desktop_quick_enhance_artifact_macro_live_plan(
                config,
                game_window,
                plan.config.as_ref(),
                cancellation,
            )
            .map_err(TaskError::CommonJobExecution)?;
            Ok(Some(
                IndependentTaskLiveExecutionReport::QuickEnhanceArtifactMacro(report),
            ))
        }
        Some(AUTO_MUSIC_GAME_TASK_KEY) => {
            match desktop_auto_music_game_route_mode(plan.config.as_ref())? {
                DesktopAutoMusicGameRouteMode::Performance => {
                    let execution = execute_desktop_auto_music_game_performance_live_plan(
                        config,
                        game_window,
                        cancellation,
                    )
                    .map_err(TaskError::CommonJobExecution)?;
                    Ok(Some(
                        IndependentTaskLiveExecutionReport::AutoMusicGamePerformance(
                            execution.result,
                        ),
                    ))
                }
                DesktopAutoMusicGameRouteMode::Album => {
                    let execution = execute_desktop_auto_music_game_album_live_plan(
                        config,
                        game_window,
                        cancellation,
                    )
                    .map_err(TaskError::CommonJobExecution)?;
                    Ok(Some(
                        IndependentTaskLiveExecutionReport::AutoMusicGameAlbum(execution.result),
                    ))
                }
            }
        }
        _ if plan.catalog_entry.as_ref().is_some_and(|entry| {
            entry.rust_execution_surface() == bgi_task::TaskRustExecutionSurface::ExecutionPlanOnly
        }) =>
        {
            let task_key = plan
                .task_key
                .as_deref()
                .unwrap_or("<unknown independent task>");
            Err(TaskError::CommonJobExecution(format!(
                "{task_key} live execution is not wired to the desktop adapter"
            )))
        }
        _ => Ok(None),
    }
}

fn desktop_independent_live_adapter_gap(task_key: &str, pending_adapters: &str) -> TaskError {
    TaskError::CommonJobExecution(format!(
        "{task_key} desktop live adapter remains pending: {pending_adapters} adapters are not wired"
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopAutoFightLiveRouteMode {
    FinishProbe,
    FullFightLoop,
}

fn desktop_auto_fight_live_route_mode(
    config: Option<&Value>,
) -> bgi_task::Result<DesktopAutoFightLiveRouteMode> {
    let raw_mode = config.and_then(|value| {
        value
            .get("mode")
            .or_else(|| value.get("executionMode"))
            .or_else(|| value.get("taskMode"))
            .or_else(|| value.get("liveMode"))
            .and_then(Value::as_str)
    });
    let Some(raw_mode) = raw_mode else {
        return Ok(DesktopAutoFightLiveRouteMode::FullFightLoop);
    };
    match raw_mode.trim().to_ascii_lowercase().as_str() {
        "" | "full" | "fightloop" | "fight_loop" | "autofight" => {
            Ok(DesktopAutoFightLiveRouteMode::FullFightLoop)
        }
        "finishprobe" | "finish_probe" | "finish-detection" | "finishdetection"
        | "finish_detection" => Ok(DesktopAutoFightLiveRouteMode::FinishProbe),
        other => Err(TaskError::CommonJobExecution(format!(
            "unsupported AutoFight live execution mode: {other}"
        ))),
    }
}

fn desktop_auto_fight_finish_probe_execution_mode(
    config: Option<&Value>,
) -> AutoFightFinishDetectionExecutionMode {
    let send_input = config
        .and_then(|value| {
            value
                .get("sendInput")
                .or_else(|| value.get("send_input"))
                .and_then(Value::as_bool)
        })
        .unwrap_or(false);
    if send_input {
        AutoFightFinishDetectionExecutionMode::SendInput
    } else {
        AutoFightFinishDetectionExecutionMode::PlanOnly
    }
}

fn execute_desktop_auto_fight_finish_probe_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    plan: &AutoFightExecutionPlan,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoFightFinishDetectionLiveExecution, String> {
    execute_desktop_auto_fight_finish_probe_live_plan_with_capture(
        game_window,
        plan,
        mode,
        cancellation,
        || {
            capture_desktop_game_bgr_image(config).map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoFight finish probe desktop capture failed: {error}"
                ))
            })
        },
    )
}

fn execute_desktop_auto_fight_finish_probe_live_plan_with_capture<C>(
    game_window: Option<&GameWindowMatch>,
    plan: &AutoFightExecutionPlan,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Arc<InputCancellationToken>,
    capture: C,
) -> Result<AutoFightFinishDetectionLiveExecution, String>
where
    C: FnOnce() -> bgi_task::Result<BgrImage>,
{
    if cancellation.is_cancelled() {
        return Err("AutoFight finish probe live execution cancelled".to_string());
    }
    let _window = game_window.ok_or_else(|| {
        "AutoFight finish probe live execution requires a detected game window".to_string()
    })?;
    let cancellation = if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        Some(cancellation.as_ref())
    } else {
        None
    };
    execute_auto_fight_finish_detection_live_probe(
        &plan.finish_detection_plan,
        mode,
        cancellation,
        capture,
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_auto_pathing_action_boundary_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
    plan: &AutoPathingExecutionPlan,
) -> Result<AutoPathingActionBoundaryReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoPathing action boundary live execution cancelled".to_string());
    }

    let window = game_window.ok_or_else(|| {
        "AutoPathing action boundary live execution requires a detected game window".to_string()
    })?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let preflight =
        evaluate_auto_pathing_resolution_preflight(&plan.execution_plan.preflight, capture_size);
    if !matches!(
        preflight.status,
        AutoPathingResolutionPreflightStatus::Passed
            | AutoPathingResolutionPreflightStatus::Skipped
    ) {
        return Err(preflight.message);
    }

    execute_auto_pathing_action_boundary_with_live_executor(plan, capture_size, |common_job_plan| {
        execute_desktop_common_job_live_plan(
            config,
            game_window,
            Arc::clone(&cancellation),
            common_job_plan,
        )
    })
    .map_err(|error| error.to_string())
}

fn execute_desktop_turn_around_macro_live_plan(
    app_config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    plan_config: Option<&Value>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<MacroHotkeyExecutionReport, String> {
    let window = game_window.ok_or_else(|| {
        "TurnAroundMacro live execution requires a detected game window".to_string()
    })?;
    let mut config = desktop_macro_hotkey_execution_config(app_config, plan_config);
    config.capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_turn_around_macro(config);
    execute_desktop_macro_hotkey_live(&plan, window, cancellation)
}

fn execute_desktop_quick_enhance_artifact_macro_live_plan(
    app_config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    plan_config: Option<&Value>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<MacroHotkeyExecutionReport, String> {
    let window = game_window.ok_or_else(|| {
        "QuickEnhanceArtifactMacro live execution requires a detected game window".to_string()
    })?;
    let mut config = desktop_macro_hotkey_execution_config(app_config, plan_config);
    config.capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_quick_enhance_artifact_macro(config);
    execute_desktop_macro_hotkey_live(&plan, window, cancellation)
}

fn desktop_macro_hotkey_execution_config(
    app_config: &AppConfig,
    plan_config: Option<&Value>,
) -> MacroHotkeyExecutionConfig {
    let mut config = MacroHotkeyExecutionConfig::from_value(plan_config);
    config.macro_config = app_config.macro_config.clone();

    let Some(value) = plan_config else {
        return config;
    };
    let macro_value = value
        .get("macroConfig")
        .or_else(|| value.get("MacroConfig"))
        .or_else(|| value.get("macro_config"))
        .unwrap_or(value);
    let Some(overrides) = macro_value.as_object() else {
        return config;
    };

    let mut merged = match serde_json::to_value(&app_config.macro_config) {
        Ok(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    for (key, value) in overrides {
        merged.insert(key.clone(), value.clone());
    }
    config.macro_config = serde_json::from_value(Value::Object(merged))
        .unwrap_or_else(|_| app_config.macro_config.clone());
    config
}

fn execute_desktop_macro_hotkey_live(
    plan: &MacroHotkeyExecutionPlan,
    window: &GameWindowMatch,
    cancellation: Arc<InputCancellationToken>,
) -> Result<MacroHotkeyExecutionReport, String> {
    let (report, _executions) = execute_desktop_macro_hotkey_live_with_mode(
        plan,
        window,
        GlobalInputDispatchMode::SendInput,
        cancellation,
    )?;
    Ok(report)
}

fn execute_desktop_macro_hotkey_live_with_mode(
    plan: &MacroHotkeyExecutionPlan,
    window: &GameWindowMatch,
    mode: GlobalInputDispatchMode,
    cancellation: Arc<InputCancellationToken>,
) -> Result<
    (
        MacroHotkeyExecutionReport,
        Vec<bgi_script::GlobalInputExecution>,
    ),
    String,
> {
    if cancellation.is_cancelled() {
        return Err(format!("{} live execution cancelled", plan.task_key));
    }
    let metrics = window.metrics.ok_or_else(|| {
        format!(
            "{} live execution requires game window metrics",
            plan.task_key
        )
    })?;
    let mut runtime =
        DesktopMacroHotkeyRuntime::new(metrics.capture_area, window.handle.0, mode, cancellation);
    let report =
        execute_macro_hotkey_plan(plan, &mut runtime).map_err(|error| error.to_string())?;
    Ok((report, runtime.into_executions()))
}

struct DesktopMacroHotkeyRuntime {
    capture_area: bgi_capture::WindowRect,
    window_handle: isize,
    mode: GlobalInputDispatchMode,
    cancellation: Arc<InputCancellationToken>,
    executions: Vec<bgi_script::GlobalInputExecution>,
}

impl DesktopMacroHotkeyRuntime {
    fn new(
        capture_area: bgi_capture::WindowRect,
        window_handle: isize,
        mode: GlobalInputDispatchMode,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            capture_area,
            window_handle,
            mode,
            cancellation,
            executions: Vec::new(),
        }
    }

    fn into_executions(self) -> Vec<bgi_script::GlobalInputExecution> {
        self.executions
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "MacroHotkey live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn dispatch_events(&mut self, events: Vec<InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        match self.mode {
            GlobalInputDispatchMode::PlanOnly => {
                self.executions.push(bgi_script::GlobalInputExecution {
                    mode: self.mode,
                    events,
                    dispatched: false,
                    dispatched_events: 0,
                });
                Ok(())
            }
            GlobalInputDispatchMode::SendInput => {
                let report = send_events_to_window_with_cancellation(
                    self.window_handle,
                    &events,
                    self.cancellation.as_ref(),
                )
                .map_err(|error| {
                    TaskError::CommonJobExecution(format!(
                        "MacroHotkey input dispatch failed: {error}"
                    ))
                })?;
                self.executions.push(bgi_script::GlobalInputExecution {
                    mode: self.mode,
                    events,
                    dispatched: true,
                    dispatched_events: report.dispatched_events,
                });
                Ok(())
            }
        }
    }

    fn screen_point(&self, point: &MacroHotkeyScreenPoint) -> (i32, i32) {
        (
            self.capture_area.left + point.screen_x.round() as i32,
            self.capture_area.top + point.screen_y.round() as i32,
        )
    }

    fn click_capture_point_events(&self, point: &MacroHotkeyScreenPoint) -> Vec<InputEvent> {
        let (x, y) = self.screen_point(point);
        InputSequence::new()
            .move_mouse_to(x, y)
            .mouse_down(MouseButton::Left)
            .delay(50)
            .mouse_up(MouseButton::Left)
            .delay(50)
            .events()
            .to_vec()
    }

    fn move_capture_point_events(&self, point: &MacroHotkeyScreenPoint) -> Vec<InputEvent> {
        let (x, y) = self.screen_point(point);
        InputSequence::new().move_mouse_to(x, y).events().to_vec()
    }
}

impl MacroHotkeyRuntime for DesktopMacroHotkeyRuntime {
    fn macro_hotkey_preflight(
        &mut self,
        _rule: &MacroHotkeyPreflightRule,
    ) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        Ok(true)
    }

    fn move_macro_hotkey_mouse_by(&mut self, dx: i64, dy: i64) -> bgi_task::Result<()> {
        let dx = i32::try_from(dx).map_err(|_| {
            TaskError::CommonJobExecution(format!("MacroHotkey mouse dx {dx} is out of range"))
        })?;
        let dy = i32::try_from(dy).map_err(|_| {
            TaskError::CommonJobExecution(format!("MacroHotkey mouse dy {dy} is out of range"))
        })?;
        self.dispatch_events(vec![InputEvent::MouseMoveRelative { dx, dy }])
    }

    fn click_macro_hotkey_capture_point(
        &mut self,
        point: &MacroHotkeyScreenPoint,
    ) -> bgi_task::Result<()> {
        self.dispatch_events(self.click_capture_point_events(point))
    }

    fn move_macro_hotkey_capture_point(
        &mut self,
        point: &MacroHotkeyScreenPoint,
    ) -> bgi_task::Result<()> {
        self.dispatch_events(self.move_capture_point_events(point))
    }

    fn wait_macro_hotkey(&mut self, delay_ms: u64) -> bgi_task::Result<()> {
        self.dispatch_events(vec![InputEvent::Delay {
            milliseconds: delay_ms,
        }])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopAutoMusicGameRouteMode {
    Performance,
    Album,
}

fn desktop_auto_music_game_route_mode(
    config: Option<&Value>,
) -> bgi_task::Result<DesktopAutoMusicGameRouteMode> {
    let raw_mode = config.and_then(|value| {
        value
            .get("mode")
            .or_else(|| value.get("executionMode"))
            .or_else(|| value.get("taskMode"))
            .and_then(Value::as_str)
    });
    let Some(raw_mode) = raw_mode else {
        return Ok(DesktopAutoMusicGameRouteMode::Performance);
    };
    match raw_mode.trim().to_ascii_lowercase().as_str() {
        "" | "performance" | "manual" | "manualperformance" | "manual_performance" => {
            Ok(DesktopAutoMusicGameRouteMode::Performance)
        }
        "album" | "autoalbum" | "auto_album" => Ok(DesktopAutoMusicGameRouteMode::Album),
        other => Err(TaskError::CommonJobExecution(format!(
            "unsupported AutoMusicGame live execution mode: {other}"
        ))),
    }
}

fn execute_desktop_auto_open_chest_live_plan(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoOpenChestTaskExecution, String> {
    let window = game_window.ok_or_else(|| {
        "AutoOpenChest live execution requires a detected game window".to_string()
    })?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_auto_open_chest(AutoOpenChestExecutionConfig {
        capture_size,
        asset_scale: desktop_auto_open_chest_asset_scale(capture_size),
    })
    .map_err(|error| error.to_string())?;
    let result =
        execute_desktop_auto_open_chest_live(app_root, config, window, &plan, cancellation)?;
    Ok(DesktopAutoOpenChestTaskExecution {
        task: plan.task_key,
        result,
    })
}

fn desktop_auto_open_chest_asset_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / AUTO_OPEN_CHEST_DEFAULT_CAPTURE_WIDTH as f64
}

fn execute_desktop_auto_open_chest_live(
    app_root: &Path,
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoOpenChestExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoOpenChestExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoOpenChest live execution cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoOpenChest live execution requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoOpenChest live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoOpenChest live execution requires the BitBlt capture backend".to_string());
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoOpenChestRuntime::new(
        app_root.to_path_buf(),
        bgi_task::task_asset_root(),
        global_input,
        capture_size,
        capture_source,
        window.handle.0,
        config.key_bindings_config.clone(),
        cancellation,
    )?;
    execute_auto_open_chest_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoOpenChestRuntime {
    app_root: PathBuf,
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    key_bindings_config: KeyBindingsConfig,
    cancellation: Arc<InputCancellationToken>,
    started_at: Instant,
}

impl DesktopAutoOpenChestRuntime {
    #[allow(clippy::too_many_arguments)]
    fn new(
        app_root: PathBuf,
        template_root: PathBuf,
        mut global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        key_bindings_config: KeyBindingsConfig,
        cancellation: Arc<InputCancellationToken>,
    ) -> Result<Self, String> {
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            app_root,
            template_root,
            global_input,
            capture_size,
            capture_source,
            window_handle,
            key_bindings_config,
            cancellation,
            started_at: Instant::now(),
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "AutoOpenChest live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let image = bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_region_in_capture(
        &self,
        capture: &ImageRegion,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<Option<Region>> {
        let region = capture
            .find(&self.vision_backend, &locator.recognition_object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoOpenChest template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn execute_events(&self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn sleep_with_cancellation(&self, duration_ms: u64) {
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < deadline {
            if self.cancellation.is_cancelled() {
                return;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            std::thread::sleep(remaining.min(Duration::from_millis(25)));
        }
    }
}

impl AutoOpenChestRuntime for DesktopAutoOpenChestRuntime {
    fn elapsed_auto_open_chest_ms(&mut self) -> bgi_task::Result<u64> {
        Ok(self
            .started_at
            .elapsed()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64)
    }

    fn is_auto_open_chest_cancelled(&mut self) -> bgi_task::Result<bool> {
        Ok(self.cancellation.is_cancelled())
    }

    fn observe_auto_open_chest(
        &mut self,
        plan: &AutoOpenChestExecutionPlan,
    ) -> bgi_task::Result<AutoOpenChestObservation> {
        let capture = self.capture_image_region()?;
        let chest_f_icon_exists = self
            .locate_region_in_capture(&capture, &plan.locators.chest_f_icon)?
            .is_some();
        let chest_icon = self
            .locate_region_in_capture(&capture, &plan.locators.chest_icon)?
            .map(|region| region.rect);
        let flower_f_icon_exists = self
            .locate_region_in_capture(&capture, &plan.locators.flower_f_icon)?
            .is_some();

        Ok(AutoOpenChestObservation {
            initial_chest_f_icon_exists: chest_f_icon_exists,
            chest_icon,
            chest_f_icon_exists,
            flower_f_icon_exists,
            capture_width: self.capture_size.width,
        })
    }

    fn dispatch_auto_open_chest_action(
        &mut self,
        action: &AutoOpenChestAction,
    ) -> bgi_task::Result<()> {
        match action {
            AutoOpenChestAction::GenshinAction { action, press } => {
                let action_type = match press {
                    AutoOpenChestActionPress::KeyPress => KeyActionType::KeyPress,
                    AutoOpenChestActionPress::KeyDown => KeyActionType::KeyDown,
                    AutoOpenChestActionPress::KeyUp => KeyActionType::KeyUp,
                };
                let events =
                    input_events_for_action(&self.key_bindings_config, *action, action_type)
                        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
                self.execute_events(events)
            }
            AutoOpenChestAction::MouseMoveBy { delta_x, delta_y } => {
                self.execute_sequence(self.global_input.move_mouse_by(*delta_x, *delta_y))
            }
            AutoOpenChestAction::MouseMiddleClick => {
                self.execute_sequence(self.global_input.middle_button_click())
            }
            AutoOpenChestAction::Delay { duration_ms } => {
                self.sleep_with_cancellation(*duration_ms);
                Ok(())
            }
            AutoOpenChestAction::Log { message } => {
                append_desktop_log(&self.app_root, "INFO", message);
                Ok(())
            }
        }
    }
}

fn execute_desktop_quick_buy_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopQuickBuyTaskExecution, String> {
    let window = game_window
        .ok_or_else(|| "QuickBuy live execution requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_quick_buy(QuickBuyExecutionConfig { capture_size })
        .map_err(|error| error.to_string())?;
    let result = execute_desktop_quick_buy_live(config, window, &plan, cancellation)?;
    Ok(DesktopQuickBuyTaskExecution {
        task: plan.task_key,
        result,
    })
}

fn execute_desktop_quick_buy_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &QuickBuyExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<QuickBuyExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("QuickBuy live execution cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "QuickBuy live execution requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "QuickBuy live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("QuickBuy live execution requires the BitBlt capture backend".to_string());
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime = DesktopQuickBuyRuntime::new(
        bgi_task::task_asset_root(),
        global_input,
        capture_size,
        capture_source,
        window.handle.0,
        desktop_game_window_process_is_foreground(window),
        cancellation,
    )?;
    execute_quick_buy_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopQuickBuyRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    game_window_foreground: bool,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopQuickBuyRuntime {
    fn new(
        template_root: PathBuf,
        mut global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        game_window_foreground: bool,
        cancellation: Arc<InputCancellationToken>,
    ) -> Result<Self, String> {
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_source,
            window_handle,
            game_window_foreground,
            cancellation,
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "QuickBuy live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let image = bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        Ok(ImageRegion::capture(image))
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn execute_events(&self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }
}

impl QuickBuyRuntime for DesktopQuickBuyRuntime {
    fn quick_buy_preflight(&mut self, _rule: &QuickBuyPreflightRule) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        Ok(self.game_window_foreground)
    }

    fn locate_quick_buy_template(&mut self, locator: &BvLocatorPlan) -> bgi_task::Result<bool> {
        let region = self
            .capture_image_region()?
            .find(&self.vision_backend, &locator.recognition_object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "QuickBuy template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist())
    }

    fn move_quick_buy_cursor(&mut self, point: &QuickBuyScreenPoint) -> bgi_task::Result<()> {
        self.execute_sequence(
            self.global_input
                .move_mouse_to(point.screen_x.round() as i32, point.screen_y.round() as i32)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?,
        )
    }

    fn execute_quick_buy_page_command(&mut self, command: &BvPageCommand) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        match command {
            BvPageCommand::Wait { milliseconds } => {
                std::thread::sleep(Duration::from_millis(u64::from(*milliseconds)));
                Ok(())
            }
            BvPageCommand::Click1080p {
                screen_x, screen_y, ..
            } => self.execute_sequence(
                self.global_input
                    .click(screen_x.round() as i32, screen_y.round() as i32)
                    .map_err(|error| TaskError::VisionPlan(error.to_string()))?,
            ),
            other => Err(TaskError::VisionPlan(format!(
                "QuickBuy desktop runtime does not support page command {other:?}"
            ))),
        }
    }

    fn dispatch_quick_buy_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<()> {
        let mut mapped = Vec::with_capacity(events.len());
        for event in events {
            match event {
                bgi_input::InputEvent::MouseMoveAbsolute { x, y, .. } => {
                    mapped.extend_from_slice(
                        self.global_input
                            .move_mouse_to(*x, *y)
                            .map_err(|error| TaskError::VisionPlan(error.to_string()))?
                            .events(),
                    );
                }
                event => mapped.push(*event),
            }
        }
        self.execute_events(mapped)
    }

    fn click_quick_buy_target(&mut self, target: &QuickBuyClickTarget) -> bgi_task::Result<()> {
        let (x, y) = quick_buy_target_capture_point(target);
        self.execute_sequence(
            self.global_input
                .click(x, y)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?,
        )
    }

    fn clear_quick_buy_vision_drawings(&mut self) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()
    }
}

fn quick_buy_target_capture_point(target: &QuickBuyClickTarget) -> (i32, i32) {
    match target {
        QuickBuyClickTarget::Fixed1080p(point) => {
            (point.screen_x.round() as i32, point.screen_y.round() as i32)
        }
        QuickBuyClickTarget::BottomRightOffset {
            screen_x, screen_y, ..
        } => (screen_x.round() as i32, screen_y.round() as i32),
    }
}

fn execute_desktop_quick_serenitea_pot_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopQuickSereniteaPotTaskExecution, String> {
    let window = game_window.ok_or_else(|| {
        "QuickSereniteaPot live execution requires a detected game window".to_string()
    })?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_quick_serenitea_pot(QuickSereniteaPotExecutionConfig { capture_size })
        .map_err(|error| error.to_string())?;
    let result = execute_desktop_quick_serenitea_pot_live(config, window, &plan, cancellation)?;
    Ok(DesktopQuickSereniteaPotTaskExecution {
        task: plan.task_key,
        result,
    })
}

fn execute_desktop_quick_serenitea_pot_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &QuickSereniteaPotExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<QuickSereniteaPotExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("QuickSereniteaPot live execution cancelled".to_string());
    }
    let metrics = window.metrics.ok_or_else(|| {
        "QuickSereniteaPot live execution requires game window metrics".to_string()
    })?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "QuickSereniteaPot live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err(
            "QuickSereniteaPot live execution requires the BitBlt capture backend".to_string(),
        );
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime =
        DesktopQuickSereniteaPotRuntime::new(DesktopQuickSereniteaPotRuntimeConfig {
            template_root: bgi_task::task_asset_root(),
            global_input,
            capture_size,
            capture_source,
            window_handle: window.handle.0,
            game_window_foreground: desktop_game_window_process_is_foreground(window),
            key_bindings_config: config.key_bindings_config.clone(),
            cancellation,
        })?;
    execute_quick_serenitea_pot_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopQuickSereniteaPotRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    game_window_foreground: bool,
    key_bindings_config: KeyBindingsConfig,
    main_ui_locator: BvLocatorPlan,
    big_map_scale_locator: BvLocatorPlan,
    big_map_settings_locator: BvLocatorPlan,
    pick_key_locator: BvLocatorPlan,
    cancellation: Arc<InputCancellationToken>,
}

struct DesktopQuickSereniteaPotRuntimeConfig {
    template_root: PathBuf,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    game_window_foreground: bool,
    key_bindings_config: KeyBindingsConfig,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopQuickSereniteaPotRuntime {
    fn new(config: DesktopQuickSereniteaPotRuntimeConfig) -> Result<Self, String> {
        let DesktopQuickSereniteaPotRuntimeConfig {
            template_root,
            mut global_input,
            capture_size,
            capture_source,
            window_handle,
            game_window_foreground,
            key_bindings_config,
            cancellation,
        } = config;
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        let return_main_ui_plan =
            bgi_task::plan_return_main_ui(capture_size, 1).map_err(|error| error.to_string())?;
        let main_ui_locator = return_main_ui_plan
            .steps
            .into_iter()
            .find_map(|step| match step.action {
                CommonJobStepAction::Locator { locator }
                    if step.label.contains("already in main UI") =>
                {
                    Some(locator)
                }
                _ => None,
            })
            .ok_or_else(|| {
                "QuickSereniteaPot live execution could not build main-UI locator".to_string()
            })?;
        let pick_key_locator =
            desktop_quick_serenitea_pot_pick_key_locator(capture_size).map_err(|error| {
                format!("QuickSereniteaPot live execution could not build F-key locator: {error}")
            })?;
        let big_map_scale_locator = desktop_quick_serenitea_pot_big_map_locator(
            capture_size,
            "MapScaleButton",
            QUICK_TELEPORT_MAP_SCALE_BUTTON,
            Rect::new(
                desktop_scaled_1080p(30, capture_size),
                desktop_scaled_1080p(440, capture_size),
                desktop_scaled_1080p(40, capture_size),
                desktop_scaled_1080p(200, capture_size),
            )
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| {
            format!(
                "QuickSereniteaPot live execution could not build big-map scale locator: {error}"
            )
        })?;
        let big_map_settings_locator = desktop_quick_serenitea_pot_big_map_locator(
            capture_size,
            "MapSettingsButton",
            QUICK_TELEPORT_MAP_SETTINGS_BUTTON,
            Rect::new(
                desktop_scaled_1080p(25, capture_size),
                desktop_scaled_1080p(990, capture_size),
                desktop_scaled_1080p(58, capture_size),
                desktop_scaled_1080p(62, capture_size),
            )
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| {
            format!(
                "QuickSereniteaPot live execution could not build big-map settings locator: {error}"
            )
        })?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_size,
            capture_source,
            window_handle,
            game_window_foreground,
            key_bindings_config,
            main_ui_locator,
            big_map_scale_locator,
            big_map_settings_locator,
            pick_key_locator,
            cancellation,
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "QuickSereniteaPot live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let image = bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_region_once(&self, locator: &BvLocatorPlan) -> bgi_task::Result<Option<Region>> {
        let region = self
            .capture_image_region()?
            .find(&self.vision_backend, &locator.recognition_object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "QuickSereniteaPot template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn wait_for_locator(&self, locator: &BvLocatorPlan) -> bgi_task::Result<Option<Region>> {
        for attempt in 0..locator.retry_count.max(1) {
            self.ensure_not_cancelled()?;
            if let Some(region) = self.locate_region_once(locator)? {
                return Ok(Some(region));
            }
            if attempt + 1 < locator.retry_count.max(1) {
                std::thread::sleep(Duration::from_millis(u64::from(locator.retry_interval_ms)));
            }
        }
        Ok(None)
    }

    fn wait_for_locator_to_disappear(&self, locator: &BvLocatorPlan) -> bgi_task::Result<bool> {
        for attempt in 0..locator.retry_count.max(1) {
            self.ensure_not_cancelled()?;
            if self.locate_region_once(locator)?.is_none() {
                return Ok(true);
            }
            if attempt + 1 < locator.retry_count.max(1) {
                std::thread::sleep(Duration::from_millis(u64::from(locator.retry_interval_ms)));
            }
        }
        Ok(false)
    }

    fn click_capture_point(&self, x: i32, y: i32) -> bgi_task::Result<()> {
        self.execute_sequence(
            self.global_input
                .click(x, y)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?,
        )
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn execute_events(&self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn detect_main_ui(&self) -> bgi_task::Result<bool> {
        Ok(self.wait_for_locator(&self.main_ui_locator)?.is_some())
    }

    fn detect_big_map_ui(&self) -> bgi_task::Result<bool> {
        Ok(self
            .locate_region_once(&self.big_map_scale_locator)?
            .is_some()
            || self
                .locate_region_once(&self.big_map_settings_locator)?
                .is_some())
    }

    fn quick_serenitea_pot_ocr_roi_text(&self, roi: Rect) -> bgi_task::Result<String> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let image = bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let roi = desktop_ocr_roi_for_image(image.size, roi)?;
        let cropped = crop_bgr_image(&image, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("QuickSereniteaPot WinRT OCR failed: {error}"))
        })?;
        Ok(desktop_quick_serenitea_pot_ocr_text_from_regions(&regions))
    }
}

impl QuickSereniteaPotRuntime for DesktopQuickSereniteaPotRuntime {
    fn quick_serenitea_pot_preflight(
        &mut self,
        _rule: &QuickSereniteaPotPreflightRule,
    ) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        Ok(self.game_window_foreground)
    }

    fn dispatch_quick_serenitea_pot_action(
        &mut self,
        action: GenshinAction,
    ) -> bgi_task::Result<()> {
        let events =
            input_events_for_action(&self.key_bindings_config, action, KeyActionType::KeyPress)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        self.execute_events(events)
    }

    fn locate_quick_serenitea_pot_template(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<bool> {
        match locator.operation {
            BvLocatorOperation::WaitForDisappear => self.wait_for_locator_to_disappear(locator),
            BvLocatorOperation::ClickUntilDisappears => {
                let mut disappeared = false;
                for attempt in 0..locator.retry_count.max(1) {
                    let Some(region) = self.locate_region_once(locator)? else {
                        disappeared = true;
                        break;
                    };
                    let center = region.rect.center();
                    self.click_capture_point(center.x, center.y)?;
                    if attempt + 1 < locator.retry_count.max(1) {
                        std::thread::sleep(Duration::from_millis(u64::from(
                            locator.retry_interval_ms,
                        )));
                    }
                }
                Ok(disappeared)
            }
            BvLocatorOperation::Click | BvLocatorOperation::DoubleClick => {
                let Some(region) = self.wait_for_locator(locator)? else {
                    return Ok(false);
                };
                let center = region.rect.center();
                self.click_capture_point(center.x, center.y)?;
                if locator.operation == BvLocatorOperation::DoubleClick {
                    self.click_capture_point(center.x, center.y)?;
                }
                Ok(true)
            }
            BvLocatorOperation::FindAll
            | BvLocatorOperation::IsExist
            | BvLocatorOperation::WaitFor => Ok(self.wait_for_locator(locator)?.is_some()),
        }
    }

    fn click_quick_serenitea_pot_point(
        &mut self,
        point: &QuickSereniteaPotScreenPoint,
    ) -> bgi_task::Result<()> {
        let (x, y) = quick_serenitea_pot_screen_point_capture_point(point);
        self.click_capture_point(x, y)
    }

    fn execute_quick_serenitea_pot_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        match command {
            BvPageCommand::Screenshot { size } if *size == self.capture_size => {
                self.capture_image_region()?;
                Ok(())
            }
            BvPageCommand::Screenshot { size } => Err(TaskError::VisionPlan(format!(
                "QuickSereniteaPot screenshot command expects {}x{} but runtime capture is {}x{}",
                size.width, size.height, self.capture_size.width, self.capture_size.height
            ))),
            BvPageCommand::Wait { milliseconds } => {
                std::thread::sleep(Duration::from_millis(u64::from(*milliseconds)));
                Ok(())
            }
            BvPageCommand::Click1080p {
                screen_x, screen_y, ..
            } => self.click_capture_point(screen_x.round() as i32, screen_y.round() as i32),
            BvPageCommand::Ocr { locator } => {
                let roi = desktop_ocr_roi_for_image(
                    self.capture_size,
                    locator
                        .recognition_object
                        .region_of_interest
                        .unwrap_or_else(Rect::empty),
                )?;
                let _ = self.quick_serenitea_pot_ocr_roi_text(roi)?;
                Ok(())
            }
        }
    }

    fn verify_quick_serenitea_pot_placement(
        &mut self,
        rule: &QuickSereniteaPotPlacementRule,
    ) -> bgi_task::Result<QuickSereniteaPotPlacementOutcome> {
        for _ in 0..rule.main_ui_success_checks {
            if self.detect_main_ui()? {
                return Ok(QuickSereniteaPotPlacementOutcome {
                    main_ui_reached: true,
                    big_map_detected: false,
                });
            }
        }
        for attempt in 0..rule.big_map_reopen_attempts {
            if self.detect_big_map_ui()? {
                return Ok(QuickSereniteaPotPlacementOutcome {
                    main_ui_reached: false,
                    big_map_detected: true,
                });
            }
            self.dispatch_quick_serenitea_pot_action(rule.reopen_inventory_action)?;
            if attempt + 1 < rule.big_map_reopen_attempts {
                std::thread::sleep(Duration::from_millis(u64::from(
                    rule.find_pot_retry_interval_ms,
                )));
            }
        }
        if self.detect_big_map_ui()? {
            return Ok(QuickSereniteaPotPlacementOutcome {
                main_ui_reached: false,
                big_map_detected: true,
            });
        }
        Ok(QuickSereniteaPotPlacementOutcome {
            main_ui_reached: false,
            big_map_detected: false,
        })
    }

    fn click_quick_serenitea_pot_white_confirm(
        &mut self,
        locator: &BvLocatorPlan,
        pre_click_delay_ms: u32,
        _missing_is_ok: bool,
    ) -> bgi_task::Result<bool> {
        std::thread::sleep(Duration::from_millis(u64::from(pre_click_delay_ms)));
        self.locate_quick_serenitea_pot_template(locator)
    }

    fn find_quick_serenitea_pot_interaction(
        &mut self,
        rule: &QuickSereniteaPotInteractionRule,
    ) -> bgi_task::Result<QuickSereniteaPotInteractionOutcome> {
        self.ensure_not_cancelled()?;
        let Some(region) = self.locate_region_once(&self.pick_key_locator)? else {
            return Ok(QuickSereniteaPotInteractionOutcome::Missing);
        };
        let Some(text_roi) =
            desktop_quick_serenitea_pot_interaction_text_roi(region.rect, self.capture_size)
        else {
            return Ok(QuickSereniteaPotInteractionOutcome::Missing);
        };
        let text = self.quick_serenitea_pot_ocr_roi_text(text_roi)?;
        Ok(desktop_quick_serenitea_pot_interaction_from_text(
            &text, rule,
        ))
    }

    fn clear_quick_serenitea_pot_vision_drawings(&mut self) -> bgi_task::Result<()> {
        // Rust desktop locators do not currently push drawings into the WPF VisionContext layer.
        self.ensure_not_cancelled()
    }
}

fn quick_serenitea_pot_screen_point_capture_point(
    point: &QuickSereniteaPotScreenPoint,
) -> (i32, i32) {
    (point.screen_x.round() as i32, point.screen_y.round() as i32)
}

fn desktop_scaled_1080p(value: i32, capture_size: VisionSize) -> i32 {
    ((value as f64) * capture_size.width as f64 / 1920.0).round() as i32
}

fn desktop_quick_serenitea_pot_big_map_locator(
    capture_size: VisionSize,
    name: &str,
    asset: &str,
    roi: Rect,
) -> bgi_vision::Result<BvLocatorPlan> {
    let page = bgi_vision::BvPage {
        capture_size,
        ..bgi_vision::BvPage::default()
    };
    let image = BvImage::new(asset)?;
    let mut plan = page
        .locator_for_image(&image, Some(roi), 0.8)?
        .plan(BvLocatorOperation::IsExist, Some(100));
    plan.recognition_object.name = Some(name.to_string());
    plan.recognition_object.template.draw_on_window = false;
    Ok(plan)
}

fn desktop_quick_serenitea_pot_pick_key_locator(
    capture_size: VisionSize,
) -> bgi_vision::Result<BvLocatorPlan> {
    let page = bgi_vision::BvPage {
        capture_size,
        ..bgi_vision::BvPage::default()
    };
    let scale = capture_size.width as f64 / 1920.0;
    let roi = Rect::new(
        (1090.0 * scale).round() as i32,
        (330.0 * scale).round() as i32,
        (60.0 * scale).round() as i32,
        (420.0 * scale).round() as i32,
    )?;
    let image = BvImage::new(AUTO_PICK_PICK_KEY_ASSET)?;
    Ok(page
        .locator_for_image(&image, Some(roi), 0.8)?
        .plan(BvLocatorOperation::IsExist, Some(100)))
}

fn desktop_quick_serenitea_pot_interaction_text_roi(
    pick_key_rect: Rect,
    capture_size: VisionSize,
) -> Option<Rect> {
    let scale = capture_size.width as f64 / 1920.0;
    let left_offset = (115.0 * scale).round() as i32;
    let text_width = ((400.0 - 115.0) * scale).round() as i32;
    let rect = Rect {
        x: pick_key_rect.x + left_offset,
        y: pick_key_rect.y,
        width: text_width,
        height: pick_key_rect.height,
    };
    (rect.x >= 0
        && rect.y >= 0
        && rect.width > 0
        && rect.height > 0
        && rect.x + rect.width <= capture_size.width as i32
        && rect.y + rect.height <= capture_size.height as i32)
        .then_some(rect)
}

fn desktop_quick_serenitea_pot_ocr_text_from_regions(regions: &[OcrResultRegion]) -> String {
    let mut regions = regions
        .iter()
        .filter(|region| !region.text.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));

    let mut lines: Vec<(i32, i32, Vec<OcrResultRegion>)> = Vec::new();
    for region in regions {
        let center_y = region.rect.center().y;
        let height = region.rect.height.max(1);
        let Some((line_y, line_height, line_regions)) =
            lines.iter_mut().find(|(line_y, line_height, _)| {
                let tolerance = ((*line_height).max(height) / 2).max(4);
                (center_y - *line_y).abs() <= tolerance
            })
        else {
            lines.push((center_y, height, vec![region]));
            continue;
        };
        let line_len = line_regions.len() as i32;
        *line_y = ((*line_y * line_len) + center_y) / (line_len + 1);
        *line_height = (*line_height).max(height);
        line_regions.push(region);
    }

    lines
        .into_iter()
        .map(|(_, _, mut line_regions)| {
            line_regions.sort_by_key(|region| region.rect.center().x);
            line_regions
                .into_iter()
                .map(|region| region.text)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn desktop_quick_serenitea_pot_interaction_from_text(
    text: &str,
    rule: &QuickSereniteaPotInteractionRule,
) -> QuickSereniteaPotInteractionOutcome {
    let normalized = normalize_desktop_ocr_text(text);
    let object = normalize_desktop_ocr_text(&rule.object_text);
    if normalized.contains(&object)
        && normalized.contains(&normalize_desktop_ocr_text(&rule.enter_text))
    {
        return QuickSereniteaPotInteractionOutcome::Enter;
    }
    if normalized.contains(&object)
        && normalized.contains(&normalize_desktop_ocr_text(&rule.leave_text))
    {
        return QuickSereniteaPotInteractionOutcome::Leave;
    }
    QuickSereniteaPotInteractionOutcome::Missing
}

fn normalize_desktop_ocr_text(text: &str) -> String {
    OcrMatchConfig::normalize_text(text)
}

fn execute_desktop_auto_wood_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_config: Option<&Value>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoWoodTaskExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoWood live execution requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let mut execution_config = AutoWoodExecutionConfig::from_value(task_config);
    execution_config.capture_size = capture_size;
    execution_config.asset_scale = desktop_auto_wood_asset_scale(capture_size);
    if !desktop_auto_wood_task_config_has_auto_wood_settings(task_config) {
        execution_config.auto_wood_config = config.auto_wood_config.clone();
    }
    let plan = plan_auto_wood(execution_config);
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_wood_live(config, window, &plan, cancellation)?;
    Ok(DesktopAutoWoodTaskExecution { task, result })
}

fn desktop_auto_wood_task_config_has_auto_wood_settings(task_config: Option<&Value>) -> bool {
    let Some(value) = task_config else {
        return false;
    };
    if value
        .get("autoWoodConfig")
        .or_else(|| value.get("AutoWoodConfig"))
        .or_else(|| value.get("auto_wood_config"))
        .is_some()
    {
        return true;
    }
    [
        "afterZSleepDelay",
        "AfterZSleepDelay",
        "after_z_sleep_delay",
        "woodCountOcrEnabled",
        "WoodCountOcrEnabled",
        "wood_count_ocr_enabled",
        "useWonderlandRefresh",
        "UseWonderlandRefresh",
        "use_wonderland_refresh",
    ]
    .into_iter()
    .any(|key| value.get(key).is_some())
}

fn execute_desktop_auto_wood_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoWoodExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoWoodExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoWood live execution cancelled".to_string());
    }
    let (global_input, capture_size) = desktop_common_job_global_input(config, window, "AutoWood")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoWood live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "AutoWood live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(Arc::clone(&cancellation)),
    );
    let mut runtime = DesktopAutoWoodRuntime::new(
        common_runtime,
        plan.clone(),
        bgi_task::task_asset_root(),
        capture_size,
        window.handle.0,
        config.key_bindings_config.clone(),
        cancellation,
    );
    execute_auto_wood_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoWoodRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
    plan: AutoWoodExecutionPlan,
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    capture_size: VisionSize,
    window_handle: isize,
    key_bindings_config: KeyBindingsConfig,
    last_confirm_region: Option<Region>,
    system_sleep_prevented: bool,
    cancellation: Arc<InputCancellationToken>,
}

impl<F, I, C> DesktopAutoWoodRuntime<F, I, C> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        common: PureTemplateCommonJobRuntime<F, I, C>,
        plan: AutoWoodExecutionPlan,
        template_root: PathBuf,
        capture_size: VisionSize,
        window_handle: isize,
        key_bindings_config: KeyBindingsConfig,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            common,
            plan,
            template_root,
            capture_size,
            window_handle,
            key_bindings_config,
            last_confirm_region: None,
            system_sleep_prevented: false,
            cancellation,
        }
    }
}

impl<F, I, C> DesktopAutoWoodRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoWood live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&mut self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let image = self.common.frame_source_mut().capture_frame()?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_auto_wood_template(
        &mut self,
        locator: &AutoWoodTemplateLocator,
    ) -> bgi_task::Result<Option<Region>> {
        let object = desktop_auto_wood_template_object(locator, self.capture_size)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let region = self
            .capture_image_region()?
            .find(&self.vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoWood template lookup for {} failed under {}: {error}",
                    locator.name,
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn execute_events(&mut self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        self.common.input_driver_mut().dispatch_input(&events)
    }

    fn click_capture_point(&mut self, x: i32, y: i32) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        self.common.input_driver_mut().click_capture_point(x, y)
    }

    fn click_scaled_exit_button(&mut self) -> bgi_task::Result<()> {
        let rule = &self.plan.legacy_exit_enter_rule.exit_button_click;
        let scale = self.capture_size.height as f64 / 1080.0;
        let x = self.capture_size.width as f64 - rule.x_scale_offset_1080p * scale;
        let y = self.capture_size.height as f64 - rule.y_from_bottom_scale_offset_1080p * scale;
        self.click_capture_point(x.round() as i32, y.round() as i32)
    }

    fn click_confirm(&mut self) -> bgi_task::Result<()> {
        let Some(region) = self.last_confirm_region.take() else {
            return Err(TaskError::CommonJobExecution(
                "AutoWood legacy refresh confirm template was not detected before click"
                    .to_string(),
            ));
        };
        let center = region.rect.center();
        self.click_capture_point(center.x, center.y)
    }

    fn click_enter_game(&mut self) -> bgi_task::Result<()> {
        let point = &self.plan.legacy_exit_enter_rule.enter_game_click_1080p;
        let scale = self.capture_size.height as f64 / 1080.0;
        self.click_capture_point(
            (point.x_1080p * scale).round() as i32,
            (point.y_1080p * scale).round() as i32,
        )
    }

    fn execute_auto_wood_bilibili_login(
        &mut self,
        plan: &AutoWoodExecutionPlan,
    ) -> bgi_task::Result<AutoWoodThirdPartyLoginOutcome> {
        self.ensure_not_cancelled()?;
        let rule = desktop_auto_wood_relogin_third_party_rule(plan);
        let completed = match ReloginPlatformDriver::execute_third_party_login_probe(
            self.common.input_driver_mut(),
            &rule,
        )? {
            CommonJobRuntimeOutcome::Matched(value) => value,
            CommonJobRuntimeOutcome::None => false,
        };

        Ok(AutoWoodThirdPartyLoginOutcome {
            attempted: true,
            completed,
            mode: AutoWoodThirdPartyLoginMode::Bilibili,
            message: Some(if completed {
                "AutoWood Bilibili third-party login completed through the Relogin platform driver"
                    .to_string()
            } else {
                "AutoWood Bilibili third-party login window was not completed before retry limit"
                    .to_string()
            }),
        })
    }

    fn wait_with_cancellation(&self, duration_ms: u64) {
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < deadline {
            if self.cancellation.is_cancelled() {
                return;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            std::thread::sleep(remaining.min(Duration::from_millis(25)));
        }
    }
}

impl<F, I, C> AutoWoodRuntime for DesktopAutoWoodRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver + ReloginPlatformDriver,
    C: CommonJobClock,
{
    fn start_auto_wood(
        &mut self,
        plan: &AutoWoodExecutionPlan,
    ) -> bgi_task::Result<AutoWoodStartupOutcome> {
        self.ensure_not_cancelled()?;
        let power_state = desktop_auto_wood_prevent_system_sleep();
        let third_party_login = desktop_auto_wood_third_party_login_mode(plan);
        let mut outcome = AutoWoodStartupOutcome::completed();
        self.system_sleep_prevented = power_state.is_ok();
        outcome.system_sleep_prevented = self.system_sleep_prevented;
        outcome.third_party_login_mode = third_party_login
            .as_ref()
            .map(|(mode, _)| *mode)
            .unwrap_or(AutoWoodThirdPartyLoginMode::None);
        let power_message = match power_state {
            Ok(previous) => format!(
                "AutoWood desktop startup initialized assets and prevented system/display sleep; previous execution state was {previous:#x}"
            ),
            Err(error) => {
                format!(
                    "AutoWood desktop startup initialized assets; power-state prevention failed: {error}"
                )
            }
        };
        let third_party_message = third_party_login
            .map(|(mode, message)| format!("third-party login mode={mode:?}: {message}"))
            .unwrap_or_else(|error| format!("third-party login mode=None: {error}"));
        outcome.message = Some(format!("{power_message}; {third_party_message}"));
        Ok(outcome)
    }

    fn activate_auto_wood_game_window(
        &mut self,
        _plan: &AutoWoodExecutionPlan,
    ) -> bgi_task::Result<AutoWoodWindowOutcome> {
        self.ensure_not_cancelled()?;
        match bgi_input::activate_window(self.window_handle) {
            Ok(()) => Ok(AutoWoodWindowOutcome::activated()),
            Err(error) => Ok(AutoWoodWindowOutcome {
                activated: false,
                message: Some(format!("AutoWood game window activation failed: {error}")),
            }),
        }
    }

    fn ensure_auto_wood_gadget(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        _context: &AutoWoodRuntimeRoundContext,
    ) -> bgi_task::Result<AutoWoodGadgetOutcome> {
        let found = self
            .locate_auto_wood_template(&plan.locators.the_boon_of_the_elder_tree)?
            .is_some();
        Ok(if found {
            AutoWoodGadgetOutcome::ready()
        } else {
            AutoWoodGadgetOutcome {
                ready: false,
                switched_or_equipped: false,
                message: Some(
                    "AutoWood did not detect The Boon of the Elder Tree gadget".to_string(),
                ),
            }
        })
    }

    fn dispatch_auto_wood_input(
        &mut self,
        action: AutoWoodInputAction,
        _context: &AutoWoodRuntimeRoundContext,
    ) -> bgi_task::Result<AutoWoodInputOutcome> {
        match action {
            AutoWoodInputAction::QuickUseGadget => {
                let events = input_events_for_action(
                    &self.key_bindings_config,
                    GenshinAction::QuickUseGadget,
                    KeyActionType::KeyPress,
                )
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
                self.execute_events(events)?;
            }
            AutoWoodInputAction::Escape => {
                self.execute_events(InputSequence::new().key_press(0x1B).events().to_vec())?;
            }
            AutoWoodInputAction::ClickExitButton => self.click_scaled_exit_button()?,
            AutoWoodInputAction::ClickConfirm => self.click_confirm()?,
            AutoWoodInputAction::ClickEnterGame => self.click_enter_game()?,
        }
        Ok(AutoWoodInputOutcome::dispatched(action))
    }

    fn delay_auto_wood(
        &mut self,
        duration_ms: u64,
        reason: AutoWoodDelayReason,
        _context: Option<&AutoWoodRuntimeRoundContext>,
    ) -> bgi_task::Result<AutoWoodDelayOutcome> {
        self.wait_with_cancellation(duration_ms);
        Ok(AutoWoodDelayOutcome {
            duration_ms,
            reason,
        })
    }

    fn ocr_auto_wood_count(
        &mut self,
        plan: &AutoWoodExecutionPlan,
        _context: &AutoWoodRuntimeRoundContext,
        attempt_index: u64,
    ) -> bgi_task::Result<AutoWoodOcrOutcome> {
        self.ensure_not_cancelled()?;
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, plan.ocr_rule.wood_count_rect)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("AutoWood WinRT wood-count OCR failed: {error}"))
        })?;
        let text = desktop_auto_wood_ocr_text_from_regions(&regions);
        let mut outcome = AutoWoodOcrOutcome::text(text);
        outcome.message = Some(if plan.ocr_rule.engine.eq_ignore_ascii_case("Paddle") {
            format!(
                "AutoWood wood-count OCR attempt {attempt_index} completed through the desktop WinRT OCR adapter; Paddle parity remains tracked separately"
            )
        } else {
            format!(
                "AutoWood wood-count OCR attempt {attempt_index} completed through the desktop WinRT OCR adapter for {}",
                plan.ocr_rule.engine
            )
        });
        Ok(outcome)
    }

    fn run_auto_wood_wonderland_cycle(
        &mut self,
        _plan: &AutoWoodExecutionPlan,
        _context: &AutoWoodRuntimeRoundContext,
    ) -> bgi_task::Result<AutoWoodRefreshOutcome> {
        let plan = plan_wonderland_cycle(self.capture_size)?;
        let report = execute_wonderland_cycle_plan(&plan, &mut self.common)?;
        Ok(AutoWoodRefreshOutcome {
            completed: report.completed,
            strategy: AutoWoodRefreshStrategy::WonderlandCycle,
            fallback_used: false,
            message: Some(format!(
                "WonderlandCycle completed={} with {} executed steps and {} skipped steps",
                report.completed,
                report.executed_steps.len(),
                report.skipped_steps.len()
            )),
        })
    }

    fn probe_auto_wood_legacy_template(
        &mut self,
        locator: &AutoWoodTemplateLocator,
        _context: &AutoWoodRuntimeRoundContext,
    ) -> bgi_task::Result<bool> {
        let region = self.locate_auto_wood_template(locator)?;
        if locator.name == "AutoWoodConfirm" {
            self.last_confirm_region = region.clone();
        }
        Ok(region.is_some())
    }

    fn handle_auto_wood_third_party_login(
        &mut self,
        _plan: &AutoWoodExecutionPlan,
        _context: &AutoWoodRuntimeRoundContext,
        mode: AutoWoodThirdPartyLoginMode,
    ) -> bgi_task::Result<AutoWoodThirdPartyLoginOutcome> {
        if mode == AutoWoodThirdPartyLoginMode::None {
            return Ok(AutoWoodThirdPartyLoginOutcome::skipped(mode));
        }
        match mode {
            AutoWoodThirdPartyLoginMode::Bilibili => {
                self.execute_auto_wood_bilibili_login(_plan)
            }
            AutoWoodThirdPartyLoginMode::Other => Err(TaskError::CommonJobExecution(
                "AutoWood desktop runtime does not support non-Bilibili third-party login handling yet"
                    .to_string(),
            )),
            AutoWoodThirdPartyLoginMode::None => Ok(AutoWoodThirdPartyLoginOutcome::skipped(mode)),
        }
    }

    fn collect_auto_wood_garbage(
        &mut self,
        _plan: &AutoWoodExecutionPlan,
        _context: &AutoWoodRuntimeRoundContext,
        _strategy: AutoWoodRefreshStrategy,
    ) -> bgi_task::Result<AutoWoodGarbageCollectionOutcome> {
        self.ensure_not_cancelled()?;
        Ok(AutoWoodGarbageCollectionOutcome {
            completed: true,
            message: Some("AutoWood desktop runtime skipped explicit GC boundary".to_string()),
        })
    }

    fn cleanup_auto_wood(
        &mut self,
        _plan: &AutoWoodExecutionPlan,
        _state: &bgi_task::AutoWoodExecutorState,
    ) -> bgi_task::Result<AutoWoodCleanupOutcome> {
        let power_state = if self.system_sleep_prevented {
            desktop_auto_wood_restore_system_sleep()
        } else {
            Ok(0)
        };
        self.system_sleep_prevented = false;
        let power_state_restored = power_state.is_ok();
        let message = match power_state {
            Ok(previous) if previous != 0 => format!(
                "AutoWood desktop cleanup restored power state; previous execution state was {previous:#x}; AutoWood locators do not draw overlay content, so overlay cleanup completed as a no-op"
            ),
            Ok(_) => "AutoWood desktop cleanup completed; power-state restore was not required and AutoWood overlay cleanup is a no-op because its locators do not draw overlay content".to_string(),
            Err(error) => format!(
                "AutoWood desktop cleanup failed to restore power state: {error}; AutoWood overlay cleanup completed as a no-op"
            ),
        };
        Ok(AutoWoodCleanupOutcome {
            completed: true,
            overlay_cleared: true,
            power_state_restored,
            message: Some(message),
        })
    }

    fn is_auto_wood_cancelled(&mut self) -> bool {
        self.cancellation.is_cancelled()
    }
}

fn desktop_auto_wood_template_object(
    locator: &AutoWoodTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object =
        image.to_recognition_object_for_screen(locator.roi, locator.threshold, capture_size)?;
    object.name = Some(locator.name.clone());
    object.template.mode = locator.match_mode;
    object.template.draw_on_window = locator.draw_on_window;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_wood_relogin_third_party_rule(
    plan: &AutoWoodExecutionPlan,
) -> ReloginThirdPartyRule {
    let rule = &plan.third_party_login_rule;
    let (agreement_x_offset, agreement_y_offset) = rule.agreement_click_relative_to_center;
    let (login_x_offset, login_y_offset) = rule.login_click_relative_to_center;
    ReloginThirdPartyRule {
        refresh_available_before_login: false,
        bilibili_only: true,
        pre_login_sleep_ms: 0,
        max_login_probes: rule
            .login_retry_attempts_before_give_up
            .saturating_add(1)
            .min(u64::from(u16::MAX)) as u16,
        probe_interval_ms: rule.login_retry_interval_ms.min(u64::from(u32::MAX)) as u32,
        agreement_click: ReloginDpiAwarePoint {
            x_1080p: 960.0,
            y_1080p: 540.0,
            x_dpi_offset: f64::from(agreement_x_offset),
            y_dpi_offset: f64::from(agreement_y_offset),
        },
        login_click: ReloginDpiAwarePoint {
            x_1080p: 960.0,
            y_1080p: 540.0,
            x_dpi_offset: f64::from(login_x_offset),
            y_dpi_offset: f64::from(login_y_offset),
        },
        login_window_sleep_ms: rule
            .login_click_pre_sleep_ms
            .max(rule.login_click_post_sleep_ms)
            .min(u64::from(u32::MAX)) as u32,
    }
}

fn desktop_auto_wood_ocr_text_from_regions(regions: &[OcrResultRegion]) -> String {
    let mut regions = regions
        .iter()
        .filter(|region| !region.text.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));

    let mut lines: Vec<(i32, i32, Vec<OcrResultRegion>)> = Vec::new();
    for region in regions {
        let center_y = region.rect.center().y;
        let height = region.rect.height.max(1);
        let Some((line_y, line_height, line_regions)) =
            lines.iter_mut().find(|(line_y, line_height, _)| {
                let tolerance = ((*line_height).max(height) / 2).max(4);
                (center_y - *line_y).abs() <= tolerance
            })
        else {
            lines.push((center_y, height, vec![region]));
            continue;
        };
        let line_len = line_regions.len() as i32;
        *line_y = ((*line_y * line_len) + center_y) / (line_len + 1);
        *line_height = (*line_height).max(height);
        line_regions.push(region);
    }

    lines
        .into_iter()
        .map(|(_, _, mut line_regions)| {
            line_regions.sort_by_key(|region| region.rect.center().x);
            line_regions
                .into_iter()
                .map(|region| region.text)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn desktop_auto_wood_asset_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / AUTO_WOOD_DEFAULT_CAPTURE_WIDTH as f64
}

fn desktop_auto_wood_third_party_login_mode(
    plan: &AutoWoodExecutionPlan,
) -> Result<(AutoWoodThirdPartyLoginMode, String), String> {
    if !plan
        .third_party_login_rule
        .detects_bilibili_by_yuanshen_config_channel_14
    {
        return Ok((
            AutoWoodThirdPartyLoginMode::None,
            "Bilibili config.ini detection is disabled by the execution plan".to_string(),
        ));
    }
    if desktop_auto_wood_bilibili_login_available()? {
        Ok((
            AutoWoodThirdPartyLoginMode::Bilibili,
            "YuanShen config.ini channel=14 detected".to_string(),
        ))
    } else {
        Ok((
            AutoWoodThirdPartyLoginMode::None,
            "YuanShen config.ini channel=14 was not detected".to_string(),
        ))
    }
}

fn desktop_auto_wood_bilibili_login_available() -> Result<bool, String> {
    let Some(path) =
        find_process_image_path_by_name("YuanShen").map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    let Some(directory) = path.parent() else {
        return Ok(false);
    };
    let config_path = directory.join("config.ini");
    Ok(std::fs::read_to_string(config_path)
        .ok()
        .is_some_and(|text| desktop_auto_wood_config_text_has_bilibili_channel_14(&text)))
}

fn desktop_auto_wood_config_text_has_bilibili_channel_14(text: &str) -> bool {
    text.lines().any(|line| {
        let kv = line.trim();
        kv.starts_with("channel=") && kv.ends_with("14")
    })
}

#[cfg(windows)]
fn desktop_auto_wood_prevent_system_sleep() -> Result<u32, String> {
    use windows::Win32::System::Power::{
        SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
    };

    let previous = unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)
    };
    if previous.0 == 0 {
        Err("SetThreadExecutionState returned 0 while preventing system/display sleep".to_string())
    } else {
        Ok(previous.0)
    }
}

#[cfg(not(windows))]
fn desktop_auto_wood_prevent_system_sleep() -> Result<u32, String> {
    Err("SetThreadExecutionState is only available on Windows".to_string())
}

#[cfg(windows)]
fn desktop_auto_wood_restore_system_sleep() -> Result<u32, String> {
    use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS};

    let previous = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
    if previous.0 == 0 {
        Err("SetThreadExecutionState returned 0 while restoring power state".to_string())
    } else {
        Ok(previous.0)
    }
}

#[cfg(not(windows))]
fn desktop_auto_wood_restore_system_sleep() -> Result<u32, String> {
    Err("SetThreadExecutionState is only available on Windows".to_string())
}

fn execute_desktop_auto_track_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_config: Option<&Value>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoTrackTaskExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoTrack live execution requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let mut execution_config = AutoTrackExecutionConfig::from_value(task_config);
    execution_config.capture_size = capture_size;
    execution_config.asset_scale = desktop_auto_track_asset_scale(capture_size);
    execution_config.auto_skip_config = config.auto_skip_config.clone();
    let mut plan = bgi_task::plan_auto_track(execution_config);
    plan.pending_native = desktop_auto_track_pending_native_notes();
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_track_live(config, window, &plan, cancellation)?;
    Ok(DesktopAutoTrackTaskExecution { task, result })
}

fn execute_desktop_auto_track_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoTrackExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoTrackExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoTrack live execution cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoTrack live execution requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoTrack live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoTrack live execution requires the BitBlt capture backend".to_string());
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoTrackRuntime::new(
        bgi_task::task_asset_root(),
        global_input,
        capture_size,
        capture_source,
        window.handle.0,
        config.key_bindings_config.clone(),
        cancellation,
    )?;
    execute_auto_track_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoTrackRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    key_bindings_config: KeyBindingsConfig,
    cancellation: Arc<InputCancellationToken>,
    semaphore_acquired: bool,
}

impl DesktopAutoTrackRuntime {
    #[allow(clippy::too_many_arguments)]
    fn new(
        template_root: PathBuf,
        mut global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        key_bindings_config: KeyBindingsConfig,
        cancellation: Arc<InputCancellationToken>,
    ) -> Result<Self, String> {
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_size,
            capture_source,
            window_handle,
            key_bindings_config,
            cancellation,
            semaphore_acquired: false,
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoTrack live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_track_template(
        &self,
        image: &BgrImage,
        locator: &AutoTrackTemplateLocator,
    ) -> bgi_task::Result<Option<Region>> {
        let object = desktop_auto_track_template_object(locator, self.capture_size)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let region = self
            .vision_backend
            .find(&image.pixels, image.size, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoTrack template lookup for {} failed under {}: {error}",
                    locator.name,
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn find_auto_track_template_matches(
        &self,
        image: &BgrImage,
        locator: &AutoTrackTemplateLocator,
    ) -> bgi_task::Result<Vec<Region>> {
        let object = desktop_auto_track_template_object(locator, self.capture_size)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        self.vision_backend
            .find_multi(&image.pixels, image.size, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoTrack template multi-lookup for {} failed under {}: {error}",
                    locator.name,
                    self.template_root.display()
                ))
            })
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn execute_events(&self, events: Vec<bgi_input::InputEvent>) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute_events(
            events,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn click_capture_point(&self, x: i32, y: i32) -> bgi_task::Result<()> {
        let sequence = self
            .global_input
            .click(x, y)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
        self.execute_sequence(sequence)
    }

    fn dispatch_genshin_action(
        &self,
        action: GenshinAction,
        action_type: KeyActionType,
    ) -> bgi_task::Result<()> {
        let events = input_events_for_action(&self.key_bindings_config, action, action_type)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
        self.execute_events(events)
    }

    fn dispatch_named_auto_track_action(
        &self,
        action: &str,
        action_type: KeyActionType,
    ) -> bgi_task::Result<()> {
        let action = desktop_auto_track_genshin_action(action).ok_or_else(|| {
            TaskError::CommonJobExecution(format!(
                "AutoTrack desktop runtime does not know how to dispatch action {action:?}"
            ))
        })?;
        self.dispatch_genshin_action(action, action_type)
    }

    fn sleep_loop_with_cancellation(&self, duration_ms: u64) {
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while !self.cancellation.is_cancelled() {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(25)));
        }
    }
}

impl AutoTrackRuntime for DesktopAutoTrackRuntime {
    fn is_auto_track_cancelled(&mut self) -> bgi_task::Result<bool> {
        Ok(self.cancellation.is_cancelled())
    }

    fn try_acquire_auto_track_semaphore(&mut self) -> bgi_task::Result<bool> {
        self.ensure_not_cancelled()?;
        if self.semaphore_acquired {
            return Ok(false);
        }
        self.semaphore_acquired = true;
        Ok(true)
    }

    fn release_auto_track_semaphore(&mut self) -> bgi_task::Result<()> {
        self.semaphore_acquired = false;
        Ok(())
    }

    fn activate_auto_track_window(&mut self) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_input::activate_window(self.window_handle)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn observe_auto_track_main_ui(
        &mut self,
        plan: &AutoTrackExecutionPlan,
    ) -> bgi_task::Result<AutoTrackMainUiObservation> {
        let image = self.capture_bgr_image()?;
        let paimon_menu = self
            .locate_auto_track_template(&image, &plan.locators.paimon_menu)?
            .map(|region| desktop_auto_track_template_match(&plan.locators.paimon_menu, region));
        Ok(AutoTrackMainUiObservation { paimon_menu })
    }

    fn ocr_auto_track_mission_text(
        &mut self,
        _plan: &AutoTrackExecutionPlan,
        roi: Rect,
    ) -> bgi_task::Result<AutoTrackMissionTextObservation> {
        let image = self.capture_bgr_image()?;
        let roi = desktop_ocr_roi_for_image(image.size, roi)?;
        let cropped = crop_bgr_image(&image, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "AutoTrack WinRT mission-distance OCR failed: {error}"
            ))
        })?;
        Ok(AutoTrackMissionTextObservation {
            roi,
            text: Some(desktop_auto_track_ocr_text_from_regions(&regions)),
        })
    }

    fn observe_auto_track_teleport_candidates(
        &mut self,
        plan: &AutoTrackExecutionPlan,
    ) -> bgi_task::Result<AutoTrackTeleportObservation> {
        let image = self.capture_bgr_image()?;
        let mut candidates = Vec::new();
        for locator in &plan.teleport_rule.map_choose_icon_assets {
            let regions = self.find_auto_track_template_matches(&image, locator)?;
            candidates.extend(regions.into_iter().filter(|region| region.is_exist()).map(
                |region| AutoTrackTeleportCandidate {
                    asset: locator.asset.clone(),
                    rect: region.rect,
                    score: region.score.unwrap_or_default() as f64,
                },
            ));
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.rect.y.cmp(&right.rect.y))
                .then_with(|| left.rect.x.cmp(&right.rect.x))
        });
        let candidates = desktop_auto_track_deduplicate_candidates(candidates);
        Ok(AutoTrackTeleportObservation { candidates })
    }

    fn is_auto_track_big_map_open(
        &mut self,
        plan: &AutoTrackExecutionPlan,
    ) -> bgi_task::Result<bool> {
        Ok(self.observe_auto_track_main_ui(plan)?.paimon_menu.is_none())
    }

    fn observe_auto_track_tracking(
        &mut self,
        plan: &AutoTrackExecutionPlan,
        state: &AutoTrackExecutionState,
    ) -> bgi_task::Result<Option<AutoTrackTrackingObservation>> {
        let image = self.capture_bgr_image()?;
        let blue_track_point = self
            .locate_auto_track_template(&image, &plan.locators.blue_track_point)?
            .map(|region| {
                desktop_auto_track_template_match(&plan.locators.blue_track_point, region)
            });
        let mission_distance_text = if plan
            .tracking_rule
            .arrival_ocr_uses_saved_mission_distance_rect
        {
            match state.mission_distance_rect {
                Some(roi) => self.ocr_auto_track_mission_text(plan, roi)?.text,
                None => None,
            }
        } else {
            None
        };
        Ok(Some(AutoTrackTrackingObservation {
            blue_track_point,
            mission_distance_text,
        }))
    }

    fn dispatch_auto_track_action(
        &mut self,
        action: &AutoTrackRuntimeAction,
    ) -> bgi_task::Result<()> {
        match action {
            AutoTrackRuntimeAction::AcquireSemaphore
            | AutoTrackRuntimeAction::ActivateWindow
            | AutoTrackRuntimeAction::Delay { .. }
            | AutoTrackRuntimeAction::ClearOverlay
            | AutoTrackRuntimeAction::ReleaseSemaphore => Ok(()),
            AutoTrackRuntimeAction::MouseMove { x, y } => {
                self.execute_sequence(self.global_input.move_mouse_by(*x, *y))
            }
            AutoTrackRuntimeAction::OpenQuestMenu { action }
            | AutoTrackRuntimeAction::PressQuestNavigation { action } => {
                self.dispatch_named_auto_track_action(action, KeyActionType::KeyPress)
            }
            AutoTrackRuntimeAction::ClickTrackToggle { x, y, .. } => {
                self.click_capture_point(*x, *y)
            }
            AutoTrackRuntimeAction::ClickTeleportCandidate { candidate } => {
                let center = candidate.rect.center();
                self.click_capture_point(center.x, center.y)
            }
            AutoTrackRuntimeAction::ForwardKey { press } => {
                let action_type = match press {
                    AutoTrackActionPress::KeyDown => KeyActionType::KeyDown,
                    AutoTrackActionPress::KeyUp => KeyActionType::KeyUp,
                    AutoTrackActionPress::KeyPress => KeyActionType::KeyPress,
                };
                self.dispatch_genshin_action(GenshinAction::MoveForward, action_type)
            }
        }
    }

    fn delay_auto_track(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        Ok(())
    }

    fn clear_auto_track_overlay(&mut self) -> bgi_task::Result<()> {
        self.ensure_not_cancelled().or_else(|error| {
            if self.cancellation.is_cancelled() {
                Ok(())
            } else {
                Err(error)
            }
        })
    }
}

fn desktop_auto_track_template_object(
    locator: &AutoTrackTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object =
        image.to_recognition_object_for_screen(locator.roi, locator.threshold, capture_size)?;
    object.name = Some(locator.name.clone());
    object.template.mode = locator.match_mode;
    object.template.draw_on_window = locator.draw_on_window;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_track_template_match(
    locator: &AutoTrackTemplateLocator,
    region: Region,
) -> AutoTrackTemplateMatch {
    AutoTrackTemplateMatch {
        name: locator.name.clone(),
        asset: locator.asset.clone(),
        rect: region.rect,
        score: region.score.unwrap_or_default() as f64,
    }
}

fn desktop_auto_track_ocr_text_from_regions(regions: &[OcrResultRegion]) -> String {
    let mut regions = regions
        .iter()
        .filter(|region| !region.text.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));

    let mut lines: Vec<(i32, i32, Vec<OcrResultRegion>)> = Vec::new();
    for region in regions {
        let center_y = region.rect.center().y;
        let height = region.rect.height.max(1);
        let Some((line_y, line_height, line_regions)) =
            lines.iter_mut().find(|(line_y, line_height, _)| {
                let tolerance = ((*line_height).max(height) / 2).max(4);
                (center_y - *line_y).abs() <= tolerance
            })
        else {
            lines.push((center_y, height, vec![region]));
            continue;
        };
        let line_len = line_regions.len() as i32;
        *line_y = ((*line_y * line_len) + center_y) / (line_len + 1);
        *line_height = (*line_height).max(height);
        line_regions.push(region);
    }

    lines
        .into_iter()
        .map(|(_, _, mut line_regions)| {
            line_regions.sort_by_key(|region| region.rect.center().x);
            line_regions
                .into_iter()
                .map(|region| region.text)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn desktop_auto_track_genshin_action(action: &str) -> Option<GenshinAction> {
    match action.trim() {
        "GIActions.OpenQuestMenu" | "OpenQuestMenu" => Some(GenshinAction::OpenQuestMenu),
        "GIActions.QuestNavigation" | "QuestNavigation" => Some(GenshinAction::QuestNavigation),
        "GIActions.MoveForward" | "MoveForward" => Some(GenshinAction::MoveForward),
        _ => None,
    }
}

fn desktop_auto_track_deduplicate_candidates(
    mut candidates: Vec<AutoTrackTeleportCandidate>,
) -> Vec<AutoTrackTeleportCandidate> {
    let mut deduplicated: Vec<AutoTrackTeleportCandidate> = Vec::new();
    for candidate in candidates.drain(..) {
        let overlaps_existing = deduplicated.iter().any(|existing| {
            desktop_rects_overlap(candidate.rect, existing.rect) || {
                let left = candidate.rect.center();
                let right = existing.rect.center();
                (left.x - right.x).abs() <= 4 && (left.y - right.y).abs() <= 4
            }
        });
        if !overlaps_existing {
            deduplicated.push(candidate);
        }
    }
    deduplicated
}

fn desktop_rects_overlap(left: Rect, right: Rect) -> bool {
    let left_x2 = left.x.saturating_add(left.width);
    let left_y2 = left.y.saturating_add(left.height);
    let right_x2 = right.x.saturating_add(right.width);
    let right_y2 = right.y.saturating_add(right.height);
    left.x < right_x2 && left_x2 > right.x && left.y < right_y2 && left_y2 > right.y
}

fn desktop_auto_track_asset_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / AUTO_TRACK_DEFAULT_CAPTURE_WIDTH as f64
}

fn desktop_auto_track_pending_native_notes() -> Vec<String> {
    vec![
        "desktop live phase 1 uses BitBlt capture, PureRust template matching, WinRT OCR, SendInput actions, and no-op overlay cleanup".to_string(),
        "teleport candidates are detected from QuickTeleport templates and still use the Rust plan's center-distance selection; legacy map-specific ranking/painted-area parity remains pending".to_string(),
        "native TaskSemaphore/CancellationContext and real overlay drawing cleanup are represented by a process-local runtime guard/no-op cleanup in this phase".to_string(),
    ]
}

fn execute_desktop_auto_cook_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoCookTaskExecution, String> {
    let window = game_window
        .ok_or_else(|| "AutoCook live execution requires a detected game window".to_string())?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_auto_cook(AutoCookExecutionConfig {
        capture_size,
        asset_scale: desktop_auto_cook_asset_scale(capture_size),
        auto_cook_config: config.auto_cook_config.clone(),
    });
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_cook_live(config, window, &plan, cancellation)?;
    Ok(DesktopAutoCookTaskExecution { task, result })
}

fn execute_desktop_auto_cook_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoCookExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoCookExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoCook live execution cancelled".to_string());
    }
    let metrics = window
        .metrics
        .ok_or_else(|| "AutoCook live execution requires game window metrics".to_string())?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoCook live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err("AutoCook live execution requires the BitBlt capture backend".to_string());
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    let mut runtime = DesktopAutoCookRuntime::new(
        bgi_task::task_asset_root(),
        global_input,
        capture_size,
        capture_source,
        window.handle.0,
        plan.clone(),
        cancellation,
    )?;
    execute_auto_cook_plan(plan, &mut runtime, 0).map_err(|error| error.to_string())
}

struct DesktopAutoCookRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    plan: AutoCookExecutionPlan,
    last_white_confirm_region: Option<Region>,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoCookRuntime {
    fn new(
        template_root: PathBuf,
        mut global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        plan: AutoCookExecutionPlan,
        cancellation: Arc<InputCancellationToken>,
    ) -> Result<Self, String> {
        global_input
            .set_game_metrics(capture_size.width, capture_size.height, 1.0)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_size,
            capture_source,
            window_handle,
            plan,
            last_white_confirm_region: None,
            cancellation,
        })
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::VisionPlan(
                "AutoCook live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_cook_template(
        &self,
        image: &BgrImage,
        locator: &AutoCookTemplateLocator,
    ) -> bgi_task::Result<Option<Region>> {
        let object = desktop_auto_cook_template_object(locator, self.capture_size)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let region = self
            .vision_backend
            .find(&image.pixels, image.size, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoCook template lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn sleep_loop_with_cancellation(&self, duration_ms: u64) {
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        while !self.cancellation.is_cancelled() {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(50)));
        }
    }
}

impl AutoCookRuntime for DesktopAutoCookRuntime {
    fn next_auto_cook_frame(&mut self) -> bgi_task::Result<Option<AutoCookRuntimeFrame>> {
        if self.cancellation.is_cancelled() {
            return Ok(None);
        }

        let image = self.capture_bgr_image()?;
        let in_cook_ui = self
            .locate_auto_cook_template(&image, &self.plan.locators.cook_icon)?
            .is_some();
        let recover_button_detected = if in_cook_ui {
            self.locate_auto_cook_template(&image, &self.plan.locators.white_recover_button)?
                .is_some()
        } else {
            false
        };
        let white_confirm_region = if in_cook_ui {
            self.locate_auto_cook_template(&image, &self.plan.locators.white_confirm_button)?
        } else {
            None
        };
        let white_confirm_button_detected = white_confirm_region.is_some();
        self.last_white_confirm_region = white_confirm_region;
        let target_color_count = if in_cook_ui {
            count_auto_cook_target_color(
                &image,
                self.plan.cook_bar_rule.scaled_cook_color_rect,
                self.plan.cook_bar_rule.target_rgb,
            )?
        } else {
            0
        };

        Ok(Some(AutoCookRuntimeFrame {
            now_ms: desktop_now_millis(),
            in_cook_ui,
            recover_button_detected,
            white_confirm_button_detected,
            target_color_count,
        }))
    }

    fn delay_auto_cook_white_confirm_pre_click(
        &mut self,
        duration_ms: u64,
    ) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        self.ensure_not_cancelled()
    }

    fn click_auto_cook_white_confirm(&mut self) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        let Some(region) = &self.last_white_confirm_region else {
            return Ok(());
        };
        let center = region.rect.center();
        self.execute_sequence(
            self.global_input
                .click(center.x, center.y)
                .map_err(|error| TaskError::VisionPlan(error.to_string()))?,
        )
    }

    fn press_auto_cook_key(&mut self, vk: u16) -> bgi_task::Result<()> {
        self.execute_sequence(InputSequence::new().key_press(vk))
    }

    fn delay_auto_cook_loop(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        Ok(())
    }
}

fn desktop_auto_cook_template_object(
    locator: &AutoCookTemplateLocator,
    capture_size: VisionSize,
) -> bgi_vision::Result<bgi_vision::RecognitionObject> {
    let image = BvImage::new(&locator.asset)?;
    let mut object =
        image.to_recognition_object_for_screen(locator.roi, locator.threshold, capture_size)?;
    object.template.mode = locator.match_mode;
    object.template.use_3_channels = locator.use_3_channels;
    object.validate()?;
    Ok(object)
}

fn desktop_auto_cook_asset_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / 1920.0
}

fn desktop_now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn execute_desktop_auto_music_game_performance_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoMusicGamePerformanceExecution, String> {
    let window = game_window.ok_or_else(|| {
        "AutoMusicGame performance live execution requires a detected game window".to_string()
    })?;
    let metrics = window.metrics.ok_or_else(|| {
        "AutoMusicGame performance live execution requires game window metrics".to_string()
    })?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    let plan = plan_auto_music_game(AutoMusicGameExecutionConfig {
        capture_size,
        asset_scale: desktop_auto_music_game_asset_scale(capture_size),
        auto_music_game_config: config.auto_music_game_config.clone(),
    });
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_music_game_performance_live(window, &plan, cancellation)?;
    Ok(DesktopAutoMusicGamePerformanceExecution { task, result })
}

fn execute_desktop_auto_music_game_performance_live(
    window: &GameWindowMatch,
    plan: &AutoMusicGameExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoMusicPerformanceReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoMusicGame performance live execution cancelled".to_string());
    }
    let metrics = window.metrics.ok_or_else(|| {
        "AutoMusicGame performance live execution requires game window metrics".to_string()
    })?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoMusicGame performance live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    if metrics.client_width.saturating_mul(9) != metrics.client_height.saturating_mul(16) {
        return Err(format!(
            "AutoMusicGame performance live execution requires a 16:9 game window; got {}x{}",
            metrics.client_width, metrics.client_height
        ));
    }

    let lane_points = plan
        .key_lanes
        .iter()
        .map(|lane| {
            let (x, y) = desktop_auto_music_lane_capture_point(lane, plan.asset_scale);
            (lane.key.clone(), x, y)
        })
        .collect();
    let mut runtime =
        DesktopAutoMusicPerformanceRuntime::new(window.handle.0, lane_points, cancellation);
    execute_auto_music_performance_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoMusicPerformanceRuntime {
    window_handle: isize,
    lane_points: Vec<(String, i32, i32)>,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoMusicPerformanceRuntime {
    fn new(
        window_handle: isize,
        lane_points: Vec<(String, i32, i32)>,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            window_handle,
            lane_points,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoMusicGame performance live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn key_sequence(&self, key: &str, down: bool) -> bgi_task::Result<InputSequence> {
        let vk = bgi_script::virtual_key_code_for_script(key)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
        Ok(if down {
            InputSequence::new().key_down(vk)
        } else {
            InputSequence::new().key_up(vk)
        })
    }

    fn sleep_loop_with_cancellation(&self, duration_ms: u64) {
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        while !self.cancellation.is_cancelled() {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(5)));
        }
    }
}

impl AutoMusicPerformanceRuntime for DesktopAutoMusicPerformanceRuntime {
    fn is_auto_music_performance_cancelled(&mut self) -> bgi_task::Result<bool> {
        Ok(self.cancellation.is_cancelled())
    }

    fn next_auto_music_performance_frame(
        &mut self,
    ) -> bgi_task::Result<Option<AutoMusicPerformanceFrame>> {
        if self.cancellation.is_cancelled() {
            return Ok(None);
        }

        let lane_blues = self
            .lane_points
            .iter()
            .map(|(key, x, y)| {
                let blue = desktop_auto_music_sample_blue(self.window_handle, *x, *y)
                    .map_err(TaskError::VisionPlan)?;
                Ok(AutoMusicLaneBlueSample {
                    key: key.clone(),
                    blue,
                })
            })
            .collect::<bgi_task::Result<Vec<_>>>()?;
        Ok(Some(AutoMusicPerformanceFrame { lane_blues }))
    }

    fn auto_music_key_down(&mut self, key: &str) -> bgi_task::Result<()> {
        let sequence = self.key_sequence(key, true)?;
        self.execute_sequence(sequence)
    }

    fn auto_music_key_up(&mut self, key: &str) -> bgi_task::Result<()> {
        let sequence = self.key_sequence(key, false)?;
        self.execute_sequence(sequence)
    }

    fn delay_auto_music_poll(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        Ok(())
    }

    fn release_all_auto_music_keys(
        &mut self,
        _held_keys_before_release: &[String],
    ) -> bgi_task::Result<()> {
        bgi_script::GlobalInputExecution::execute(
            bgi_input::release_all_keys_sequence(),
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }
}

fn desktop_auto_music_game_asset_scale(capture_size: VisionSize) -> f64 {
    capture_size.width as f64 / 1920.0
}

fn desktop_auto_music_lane_capture_point(
    lane: &AutoMusicGameKeyLane,
    asset_scale: f64,
) -> (i32, i32) {
    (
        (lane.x_1080p as f64 * asset_scale) as i32,
        (lane.y_1080p as f64 * asset_scale) as i32,
    )
}

#[cfg(windows)]
fn desktop_auto_music_sample_blue(window_handle: isize, x: i32, y: i32) -> Result<u8, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};

    let hwnd = HWND(window_handle as *mut std::ffi::c_void);
    unsafe {
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return Err("AutoMusicGame GetDC failed".to_string());
        }
        let color = GetPixel(hdc, x, y);
        let _ = ReleaseDC(Some(hwnd), hdc);
        if color.0 == u32::MAX {
            return Err(format!("AutoMusicGame GetPixel failed at ({x}, {y})"));
        }
        Ok(((color.0 >> 16) & 0xff) as u8)
    }
}

#[cfg(not(windows))]
fn desktop_auto_music_sample_blue(_window_handle: isize, _x: i32, _y: i32) -> Result<u8, String> {
    Err("AutoMusicGame performance live execution requires Windows GDI GetPixel".to_string())
}

const AUTO_MUSIC_ALBUM_WHITE_CONFIRM_PRE_CLICK_MS: u64 = 500;
const AUTO_MUSIC_ALBUM_PAGE_WAIT_ATTEMPTS: usize = 5;
const AUTO_MUSIC_ALBUM_PAGE_WAIT_INTERVAL_MS: u64 = 1_000;
const AUTO_MUSIC_ALBUM_ALL_SONGS_OCR_WIDTH_RATIO: f64 = 0.16;

fn execute_desktop_auto_music_game_album_live_plan(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<DesktopAutoMusicGameAlbumExecution, String> {
    let window = game_window.ok_or_else(|| {
        "AutoMusicGame album live execution requires a detected game window".to_string()
    })?;
    let metrics = window.metrics.ok_or_else(|| {
        "AutoMusicGame album live execution requires game window metrics".to_string()
    })?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    let plan = plan_auto_music_game(AutoMusicGameExecutionConfig {
        capture_size,
        asset_scale: desktop_auto_music_game_asset_scale(capture_size),
        auto_music_game_config: config.auto_music_game_config.clone(),
    });
    let task = plan.task_key.clone();
    let result = execute_desktop_auto_music_game_album_live(config, window, &plan, cancellation)?;
    Ok(DesktopAutoMusicGameAlbumExecution { task, result })
}

fn execute_desktop_auto_music_game_album_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &AutoMusicGameExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<AutoMusicAlbumExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("AutoMusicGame album live execution cancelled".to_string());
    }
    let metrics = window.metrics.ok_or_else(|| {
        "AutoMusicGame album live execution requires game window metrics".to_string()
    })?;
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan.capture_size != capture_size {
        return Err(format!(
            "AutoMusicGame album live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    if metrics.client_width.saturating_mul(9) != metrics.client_height.saturating_mul(16) {
        return Err(format!(
            "AutoMusicGame album live execution requires a 16:9 game window; got {}x{}",
            metrics.client_width, metrics.client_height
        ));
    }

    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err(
            "AutoMusicGame album live execution requires the BitBlt capture backend".to_string(),
        );
    }

    let capture_area = metrics.capture_area;
    let capture_source = DesktopGameCaptureFrameSource::new(window.handle, settings.clone())
        .map_err(|error| error.to_string())?;
    let mut global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(
            DesktopGameCaptureFrameSource::new(window.handle, settings)
                .map_err(|error| error.to_string())?,
        )),
    )
    .map_err(|error| error.to_string())?;
    global_input
        .set_game_metrics(capture_size.width, capture_size.height, 1.0)
        .map_err(|error| error.to_string())?;

    let mut runtime = DesktopAutoMusicAlbumRuntime::new(
        global_input,
        capture_size,
        capture_source,
        window.handle.0,
        plan.clone(),
        cancellation,
    );
    execute_auto_music_album_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopAutoMusicAlbumRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_size: VisionSize,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    plan: AutoMusicGameExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoMusicAlbumRuntime {
    fn new(
        global_input: bgi_script::GlobalInputHost,
        capture_size: VisionSize,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        plan: AutoMusicGameExecutionPlan,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        let template_root = bgi_task::task_asset_root();
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_size,
            capture_source,
            window_handle,
            plan,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoMusicGame album live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_auto_music_template(
        &mut self,
        image: &BgrImage,
        locator: &AutoMusicTemplateLocator,
    ) -> bgi_task::Result<Option<Region>> {
        let object = desktop_auto_music_template_object(locator, self.capture_size)
            .map_err(TaskError::VisionPlan)?;
        self.locate_recognition_object(image, &object, &locator.name)
    }

    fn locate_recognition_object(
        &mut self,
        image: &BgrImage,
        object: &bgi_vision::RecognitionObject,
        label: &str,
    ) -> bgi_task::Result<Option<Region>> {
        desktop_auto_music_register_scaled_template(&mut self.vision_backend, object, image.size)?;
        let region = self
            .vision_backend
            .find(&image.pixels, image.size, object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoMusicGame album template lookup for {label} failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn wait_for_auto_music_template(
        &mut self,
        locator: &AutoMusicTemplateLocator,
        attempts: usize,
        interval_ms: u64,
    ) -> bgi_task::Result<Option<Region>> {
        for attempt in 0..attempts.max(1) {
            self.ensure_not_cancelled()?;
            let image = self.capture_bgr_image()?;
            if let Some(region) = self.locate_auto_music_template(&image, locator)? {
                return Ok(Some(region));
            }
            if attempt + 1 < attempts.max(1) {
                self.sleep_loop_with_cancellation(interval_ms);
            }
        }
        Ok(None)
    }

    fn click_capture_point(&self, x: i32, y: i32) -> bgi_task::Result<()> {
        self.execute_sequence(
            self.global_input
                .click(x, y)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?,
        )
    }

    fn click_region_center(&self, region: &Region) -> bgi_task::Result<()> {
        let center = region.rect.center();
        self.click_capture_point(center.x, center.y)
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        self.ensure_not_cancelled()?;
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn sleep_loop_with_cancellation(&self, duration_ms: u64) {
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        while !self.cancellation.is_cancelled() {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(25)));
        }
    }
}

impl AutoMusicAlbumRuntime for DesktopAutoMusicAlbumRuntime {
    fn is_auto_music_album_cancelled(&mut self) -> bgi_task::Result<bool> {
        Ok(self.cancellation.is_cancelled())
    }

    fn check_auto_music_album_page(
        &mut self,
        icon_locator: &AutoMusicTemplateLocator,
    ) -> bgi_task::Result<AutoMusicAlbumPageStatus> {
        let image = self.capture_bgr_image()?;
        let Some(icon_region) = self.locate_auto_music_template(&image, icon_locator)? else {
            return Ok(AutoMusicAlbumPageStatus::NotAlbumPage);
        };
        let roi = desktop_auto_music_album_all_songs_ocr_roi(image.size, icon_region.rect)?;
        let cropped = crop_bgr_image(&image, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("AutoMusicGame album WinRT OCR failed: {error}"))
        })?;
        if desktop_auto_music_ocr_contains_all_songs(&regions) {
            Ok(AutoMusicAlbumPageStatus::AllSongsPage)
        } else {
            Ok(AutoMusicAlbumPageStatus::ThemeAlbum)
        }
    }

    fn is_auto_music_song_completed(
        &mut self,
        locator: &AutoMusicTemplateLocator,
    ) -> bgi_task::Result<bool> {
        let image = self.capture_bgr_image()?;
        Ok(self.locate_auto_music_template(&image, locator)?.is_some())
    }

    fn click_auto_music_next_song(&mut self, x_1080p: i32, y_1080p: i32) -> bgi_task::Result<()> {
        let (x, y) = desktop_auto_music_1080p_capture_point(x_1080p, y_1080p, self.capture_size);
        self.click_capture_point(x, y)
    }

    fn click_auto_music_white_confirm(&mut self) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(AUTO_MUSIC_ALBUM_WHITE_CONFIRM_PRE_CLICK_MS);
        self.ensure_not_cancelled()?;
        let image = self.capture_bgr_image()?;
        let object = desktop_auto_music_white_confirm_object().map_err(TaskError::VisionPlan)?;
        if let Some(region) =
            self.locate_recognition_object(&image, &object, COMMON_BTN_WHITE_CONFIRM)?
        {
            self.click_region_center(&region)?;
        }
        Ok(())
    }

    fn click_auto_music_difficulty(
        &mut self,
        difficulty: &AutoMusicDifficultyRule,
    ) -> bgi_task::Result<()> {
        let (x, y) = desktop_auto_music_1080p_capture_point(
            difficulty.click_x_1080p,
            difficulty.click_y_1080p,
            self.capture_size,
        );
        self.click_capture_point(x, y)
    }

    fn delay_auto_music_album(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        self.ensure_not_cancelled()
    }

    fn execute_auto_music_song(
        &mut self,
        _difficulty: &AutoMusicDifficultyRule,
        _song_index: u64,
    ) -> bgi_task::Result<AutoMusicPerformanceReport> {
        execute_desktop_auto_music_game_album_song_performance(
            &self.plan,
            self.global_input.clone(),
            self.capture_source.clone(),
            self.window_handle,
            Arc::clone(&self.cancellation),
        )
    }

    fn wait_auto_music_album_page(
        &mut self,
        icon_locator: &AutoMusicTemplateLocator,
    ) -> bgi_task::Result<()> {
        if self
            .wait_for_auto_music_template(
                icon_locator,
                AUTO_MUSIC_ALBUM_PAGE_WAIT_ATTEMPTS,
                AUTO_MUSIC_ALBUM_PAGE_WAIT_INTERVAL_MS,
            )?
            .is_some()
        {
            return Ok(());
        }
        Err(TaskError::CommonJobExecution(
            "AutoMusicGame album page did not return after performance".to_string(),
        ))
    }
}

fn execute_desktop_auto_music_game_album_song_performance(
    plan: &AutoMusicGameExecutionPlan,
    global_input: bgi_script::GlobalInputHost,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    cancellation: Arc<InputCancellationToken>,
) -> bgi_task::Result<AutoMusicPerformanceReport> {
    let lane_points = plan
        .key_lanes
        .iter()
        .map(|lane| {
            let (x, y) = desktop_auto_music_lane_capture_point(lane, plan.asset_scale);
            (lane.key.clone(), x, y)
        })
        .collect();
    let mut runtime = DesktopAutoMusicAlbumSongPerformanceRuntime::new(
        global_input,
        capture_source,
        window_handle,
        plan.locators.btn_list.clone(),
        plan.album_rule.album_check_interval_ms,
        lane_points,
        cancellation,
    );
    execute_auto_music_performance_plan(plan, &mut runtime)
}

struct DesktopAutoMusicAlbumSongPerformanceRuntime {
    template_root: PathBuf,
    vision_backend: PureRustVisionBackend,
    global_input: bgi_script::GlobalInputHost,
    capture_source: DesktopGameCaptureFrameSource,
    window_handle: isize,
    btn_list_locator: AutoMusicTemplateLocator,
    album_check_interval: Duration,
    next_album_check_at: std::time::Instant,
    lane_points: Vec<(String, i32, i32)>,
    cancellation: Arc<InputCancellationToken>,
}

impl DesktopAutoMusicAlbumSongPerformanceRuntime {
    fn new(
        global_input: bgi_script::GlobalInputHost,
        capture_source: DesktopGameCaptureFrameSource,
        window_handle: isize,
        btn_list_locator: AutoMusicTemplateLocator,
        album_check_interval_ms: u64,
        lane_points: Vec<(String, i32, i32)>,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        let template_root = bgi_task::task_asset_root();
        let interval = Duration::from_millis(album_check_interval_ms.max(1));
        Self {
            vision_backend: PureRustVisionBackend::new().with_template_root(&template_root),
            template_root,
            global_input,
            capture_source,
            window_handle,
            btn_list_locator,
            album_check_interval: interval,
            next_album_check_at: std::time::Instant::now() + interval,
            lane_points,
            cancellation,
        }
    }

    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "AutoMusicGame album performance live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_bgr_image(&self) -> bgi_task::Result<BgrImage> {
        self.ensure_not_cancelled()?;
        let frame = self
            .capture_source
            .capture_frame()
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        bgr_image_from_desktop_capture_frame(frame)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))
    }

    fn locate_btn_list(&mut self, image: &BgrImage) -> bgi_task::Result<Option<Region>> {
        let object = desktop_auto_music_template_object(&self.btn_list_locator, image.size)
            .map_err(TaskError::VisionPlan)?;
        desktop_auto_music_register_scaled_template(&mut self.vision_backend, &object, image.size)?;
        let region = self
            .vision_backend
            .find(&image.pixels, image.size, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "AutoMusicGame album BtnList lookup failed under {}: {error}",
                    self.template_root.display()
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn click_region_center(&self, region: &Region) -> bgi_task::Result<()> {
        let center = region.rect.center();
        self.execute_sequence(
            self.global_input
                .click(center.x, center.y)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?,
        )
    }

    fn execute_sequence(&self, sequence: InputSequence) -> bgi_task::Result<()> {
        bgi_script::GlobalInputExecution::execute(
            sequence,
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }

    fn key_sequence(&self, key: &str, down: bool) -> bgi_task::Result<InputSequence> {
        let vk = bgi_script::virtual_key_code_for_script(key)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
        Ok(if down {
            InputSequence::new().key_down(vk)
        } else {
            InputSequence::new().key_up(vk)
        })
    }

    fn sleep_loop_with_cancellation(&self, duration_ms: u64) {
        let deadline = std::time::Instant::now() + Duration::from_millis(duration_ms);
        while !self.cancellation.is_cancelled() {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(5)));
        }
    }
}

impl AutoMusicPerformanceRuntime for DesktopAutoMusicAlbumSongPerformanceRuntime {
    fn is_auto_music_performance_cancelled(&mut self) -> bgi_task::Result<bool> {
        Ok(self.cancellation.is_cancelled())
    }

    fn next_auto_music_performance_frame(
        &mut self,
    ) -> bgi_task::Result<Option<AutoMusicPerformanceFrame>> {
        if self.cancellation.is_cancelled() {
            return Ok(None);
        }

        let now = std::time::Instant::now();
        if now >= self.next_album_check_at {
            self.next_album_check_at = now + self.album_check_interval;
            let image = self.capture_bgr_image()?;
            if let Some(region) = self.locate_btn_list(&image)? {
                self.click_region_center(&region)?;
                return Ok(None);
            }
        }

        let lane_blues = self
            .lane_points
            .iter()
            .map(|(key, x, y)| {
                let blue = desktop_auto_music_sample_blue(self.window_handle, *x, *y)
                    .map_err(TaskError::VisionPlan)?;
                Ok(AutoMusicLaneBlueSample {
                    key: key.clone(),
                    blue,
                })
            })
            .collect::<bgi_task::Result<Vec<_>>>()?;
        Ok(Some(AutoMusicPerformanceFrame { lane_blues }))
    }

    fn auto_music_key_down(&mut self, key: &str) -> bgi_task::Result<()> {
        let sequence = self.key_sequence(key, true)?;
        self.execute_sequence(sequence)
    }

    fn auto_music_key_up(&mut self, key: &str) -> bgi_task::Result<()> {
        let sequence = self.key_sequence(key, false)?;
        self.execute_sequence(sequence)
    }

    fn delay_auto_music_poll(&mut self, duration_ms: u64) -> bgi_task::Result<()> {
        self.sleep_loop_with_cancellation(duration_ms);
        Ok(())
    }

    fn release_all_auto_music_keys(
        &mut self,
        _held_keys_before_release: &[String],
    ) -> bgi_task::Result<()> {
        bgi_script::GlobalInputExecution::execute(
            bgi_input::release_all_keys_sequence(),
            GlobalInputDispatchMode::SendInput,
            Some(self.window_handle),
        )
        .map(|_| ())
        .map_err(|error| TaskError::CommonJobExecution(error.to_string()))
    }
}

fn desktop_auto_music_1080p_capture_point(
    x_1080p: i32,
    y_1080p: i32,
    capture_size: VisionSize,
) -> (i32, i32) {
    let scale = desktop_auto_music_game_asset_scale(capture_size);
    (
        (x_1080p as f64 * scale) as i32,
        (y_1080p as f64 * scale) as i32,
    )
}

fn desktop_auto_music_template_object(
    locator: &AutoMusicTemplateLocator,
    capture_size: VisionSize,
) -> Result<bgi_vision::RecognitionObject, String> {
    let image = BvImage::new(&locator.asset).map_err(|error| error.to_string())?;
    let roi = desktop_auto_music_template_roi(locator, capture_size)?;
    let mut object = image
        .to_recognition_object(roi, locator.threshold.unwrap_or(0.8))
        .map_err(|error| error.to_string())?;
    object.template.mode = locator.match_mode;
    object.validate().map_err(|error| error.to_string())?;
    Ok(object)
}

fn desktop_auto_music_white_confirm_object() -> Result<bgi_vision::RecognitionObject, String> {
    let image = BvImage::new(COMMON_BTN_WHITE_CONFIRM).map_err(|error| error.to_string())?;
    image
        .to_recognition_object(None, 0.8)
        .map_err(|error| error.to_string())
}

fn desktop_auto_music_register_scaled_template(
    backend: &mut PureRustVisionBackend,
    object: &bgi_vision::RecognitionObject,
    capture_size: VisionSize,
) -> bgi_task::Result<()> {
    if capture_size == VisionSize::new(1920, 1080) {
        return Ok(());
    }
    let Some(template_asset) = object.template.template_asset.as_ref() else {
        return Ok(());
    };
    let template = BgrImage::read(bgi_task::task_asset_root().join(template_asset))
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let scale = desktop_auto_music_game_asset_scale(capture_size);
    let scaled_size = VisionSize::new(
        ((template.size.width as f64 * scale) as u32).max(1),
        ((template.size.height as f64 * scale) as u32).max(1),
    );
    let scaled = resize_bgr_nearest(&template, scaled_size)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    backend.register_template(template_asset.clone(), scaled);
    Ok(())
}

fn desktop_auto_music_template_roi(
    locator: &AutoMusicTemplateLocator,
    capture_size: VisionSize,
) -> Result<Option<Rect>, String> {
    if let Some(roi) = locator.roi {
        return desktop_auto_music_scaled_rect(roi, capture_size).map(Some);
    }
    let Some(rule) = locator.roi_rule.as_deref() else {
        return Ok(None);
    };
    match rule {
        "CaptureRect.CutRightTop(0.2, 0.2)" => {
            desktop_auto_music_cut_right_rect(capture_size, 0.2, 0.2, false).map(Some)
        }
        "CaptureRect.CutRightBottom(0.4, 0.2)" => {
            desktop_auto_music_cut_right_rect(capture_size, 0.4, 0.2, true).map(Some)
        }
        _ => Err(format!("unsupported AutoMusicGame ROI rule: {rule}")),
    }
}

fn desktop_auto_music_scaled_rect(rect: Rect, capture_size: VisionSize) -> Result<Rect, String> {
    let scale = desktop_auto_music_game_asset_scale(capture_size);
    let scaled_positive = |value: i32| ((value as f64 * scale) as i32).max(1);
    Rect::new(
        (rect.x as f64 * scale) as i32,
        (rect.y as f64 * scale) as i32,
        scaled_positive(rect.width),
        scaled_positive(rect.height),
    )
    .map_err(|error| error.to_string())
}

fn desktop_auto_music_cut_right_rect(
    capture_size: VisionSize,
    width_ratio: f64,
    height_ratio: f64,
    bottom: bool,
) -> Result<Rect, String> {
    let width = (capture_size.width as f64 * width_ratio) as i32;
    let height = (capture_size.height as f64 * height_ratio) as i32;
    let x = capture_size.width as i32 - width;
    let y = if bottom {
        capture_size.height as i32 - height
    } else {
        0
    };
    Rect::new(x, y, width.max(1), height.max(1)).map_err(|error| error.to_string())
}

fn desktop_auto_music_album_all_songs_ocr_roi(
    image_size: VisionSize,
    icon_rect: Rect,
) -> bgi_task::Result<Rect> {
    let width = (image_size.width as f64 * AUTO_MUSIC_ALBUM_ALL_SONGS_OCR_WIDTH_RATIO) as i32;
    let requested = Rect::new(
        icon_rect.x + icon_rect.width,
        icon_rect.y,
        width.max(1),
        icon_rect.height.max(1),
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    desktop_ocr_roi_for_image(image_size, requested)
}

fn desktop_auto_music_ocr_contains_all_songs(regions: &[OcrResultRegion]) -> bool {
    regions.iter().any(|region| region.text.contains("全部"))
}

fn execute_desktop_use_redeem_code_live_plan(
    app_root: &Path,
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    entries: Vec<RedeemCodeEntry>,
    cancellation: Arc<InputCancellationToken>,
) -> Result<(UseRedeemCodeExecutionPlan, UseRedeemCodeExecutionReport), String> {
    let window = game_window.ok_or_else(|| {
        "UseRedeemCode live execution requires a detected game window".to_string()
    })?;
    let capture_size = desktop_common_job_capture_size(Some(window));
    let plan = plan_use_redeem_codes_through_task_boundary_with_capture_size(
        entries,
        capture_size,
        app_root,
    )?;
    let report = execute_desktop_use_redeem_code_live(config, window, &plan, cancellation)?;
    Ok((plan, report))
}

fn execute_desktop_use_redeem_code_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &UseRedeemCodeExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<UseRedeemCodeExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("UseRedeemCode live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "UseRedeemCode")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "UseRedeemCode live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "UseRedeemCode live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(Arc::clone(&cancellation)),
    );
    let mut runtime = DesktopUseRedeemCodeRuntime::new(common_runtime, capture_size);
    match execute_use_redeem_code_plan(plan, &mut runtime) {
        Ok(report) => Ok(report),
        Err(error) => {
            let _ = runtime.clear_redeem_clipboard();
            let _ = runtime.execute_redeem_common_job(RETURN_MAIN_UI_TASK_KEY);
            Err(error.to_string())
        }
    }
}

struct DesktopUseRedeemCodeRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
    capture_size: VisionSize,
}

impl<F, I, C> DesktopUseRedeemCodeRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>, capture_size: VisionSize) -> Self {
        Self {
            common,
            capture_size,
        }
    }
}

impl<F, I, C> DesktopUseRedeemCodeRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn wait_ms(&mut self, milliseconds: u32) -> bgi_task::Result<()> {
        let command = BvPageCommand::Wait { milliseconds };
        CommonJobRuntime::execute_page_command(&mut self.common, &command).map(|_| ())
    }

    fn execute_ocr_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        match locator.operation {
            BvLocatorOperation::FindAll
            | BvLocatorOperation::IsExist
            | BvLocatorOperation::WaitFor => Ok(CommonJobRuntimeOutcome::Matched(
                self.wait_for_ocr_locator(locator)?.is_some(),
            )),
            BvLocatorOperation::WaitForDisappear => Ok(CommonJobRuntimeOutcome::Matched(
                self.wait_for_ocr_locator_to_disappear(locator)?,
            )),
            BvLocatorOperation::Click | BvLocatorOperation::DoubleClick => {
                let Some(region) = self.wait_for_ocr_locator(locator)? else {
                    return Ok(CommonJobRuntimeOutcome::Matched(false));
                };
                let center = region.rect.center();
                self.common
                    .input_driver_mut()
                    .click_capture_point(center.x, center.y)?;
                if locator.operation == BvLocatorOperation::DoubleClick {
                    self.common
                        .input_driver_mut()
                        .click_capture_point(center.x, center.y)?;
                }
                Ok(CommonJobRuntimeOutcome::Matched(true))
            }
            BvLocatorOperation::ClickUntilDisappears => {
                let mut disappeared = false;
                for attempt in 0..locator.retry_count.max(1) {
                    let Some(region) = self.locate_ocr_once(locator)? else {
                        disappeared = true;
                        break;
                    };
                    let center = region.rect.center();
                    self.common
                        .input_driver_mut()
                        .click_capture_point(center.x, center.y)?;
                    if attempt + 1 < locator.retry_count.max(1) {
                        self.wait_ms(locator.retry_interval_ms)?;
                    }
                }
                Ok(CommonJobRuntimeOutcome::Matched(disappeared))
            }
        }
    }

    fn wait_for_ocr_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<Option<Region>> {
        for attempt in 0..locator.retry_count.max(1) {
            let region = self.locate_ocr_once(locator)?;
            if region.is_some() {
                return Ok(region);
            }
            if attempt + 1 < locator.retry_count.max(1) {
                self.wait_ms(locator.retry_interval_ms)?;
            }
        }
        Ok(None)
    }

    fn wait_for_ocr_locator_to_disappear(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<bool> {
        for attempt in 0..locator.retry_count.max(1) {
            if self.locate_ocr_once(locator)?.is_none() {
                return Ok(true);
            }
            if attempt + 1 < locator.retry_count.max(1) {
                self.wait_ms(locator.retry_interval_ms)?;
            }
        }
        Ok(false)
    }

    fn locate_ocr_once(&mut self, locator: &BvLocatorPlan) -> bgi_task::Result<Option<Region>> {
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_redeem_code_ocr_roi_for_locator(frame.size, locator)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("UseRedeemCode WinRT OCR failed: {error}"))
        })?;
        desktop_redeem_code_match_ocr_regions(&locator.recognition_object, &regions, roi)
    }
}

impl<F, I, C> CommonJobRuntime for DesktopUseRedeemCodeRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if desktop_redeem_code_locator_uses_ocr(locator) {
            return self.execute_ocr_locator(locator);
        }
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> UseRedeemCodeRuntime for DesktopUseRedeemCodeRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn execute_redeem_common_job(
        &mut self,
        task_key: &str,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if task_key != RETURN_MAIN_UI_TASK_KEY {
            return Err(TaskError::CommonJobExecution(format!(
                "UseRedeemCode desktop runtime only supports nested {RETURN_MAIN_UI_TASK_KEY}; got {task_key}"
            )));
        }
        let plan = plan_return_main_ui(self.capture_size, RETURN_MAIN_UI_DEFAULT_ESCAPE_ATTEMPTS)?;
        let report = execute_return_main_ui_plan(&plan, self)?;
        Ok(CommonJobRuntimeOutcome::Matched(report.completed))
    }

    fn set_redeem_clipboard_text(
        &mut self,
        text: &str,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        set_system_clipboard_text(text).map_err(|error| {
            TaskError::CommonJobExecution(format!("set clipboard failed: {error}"))
        })?;
        Ok(CommonJobRuntimeOutcome::None)
    }

    fn clear_redeem_clipboard(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::Matched(clear_system_clipboard()))
    }
}

fn desktop_redeem_code_locator_uses_ocr(locator: &BvLocatorPlan) -> bool {
    matches!(
        locator.recognition_object.recognition_type,
        RecognitionType::Ocr | RecognitionType::OcrMatch
    )
}

fn desktop_redeem_code_ocr_roi_for_locator(
    image_size: VisionSize,
    locator: &BvLocatorPlan,
) -> bgi_task::Result<Rect> {
    let roi = locator
        .recognition_object
        .region_of_interest
        .unwrap_or_else(Rect::empty);
    let roi = if roi.is_empty() {
        Rect::new(0, 0, image_size.width as i32, image_size.height as i32)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?
    } else {
        roi
    };
    desktop_ocr_roi_for_image(image_size, roi)
}

fn desktop_redeem_code_match_ocr_regions(
    object: &bgi_vision::RecognitionObject,
    regions: &[OcrResultRegion],
    roi: Rect,
) -> bgi_task::Result<Option<Region>> {
    for region in regions {
        if desktop_redeem_code_text_matches_object(object, &region.text)? {
            return desktop_redeem_code_offset_ocr_region(region, roi).map(Some);
        }
    }

    let aggregate = OcrResult {
        regions: regions.to_vec(),
    }
    .text();
    if !aggregate.trim().is_empty() && desktop_redeem_code_text_matches_object(object, &aggregate)?
    {
        return desktop_redeem_code_bounding_ocr_region(regions, roi, aggregate).map(Some);
    }

    Ok(None)
}

fn desktop_redeem_code_text_matches_object(
    object: &bgi_vision::RecognitionObject,
    text: &str,
) -> bgi_task::Result<bool> {
    match object.recognition_type {
        RecognitionType::Ocr => {
            let needle = OcrMatchConfig::normalize_text(&object.ocr.text);
            if needle.is_empty() {
                return Ok(!OcrMatchConfig::normalize_text(text).is_empty());
            }
            Ok(OcrMatchConfig::normalize_text(text).contains(&needle))
        }
        RecognitionType::OcrMatch => object
            .ocr
            .matches_text(text)
            .map_err(|error| TaskError::VisionPlan(error.to_string())),
        other => Err(TaskError::VisionPlan(format!(
            "UseRedeemCode OCR locator cannot handle recognition type {other:?}"
        ))),
    }
}

fn desktop_redeem_code_offset_ocr_region(
    region: &OcrResultRegion,
    roi: Rect,
) -> bgi_task::Result<Region> {
    let rect = Rect::new(
        roi.x + region.rect.x,
        roi.y + region.rect.y,
        region.rect.width,
        region.rect.height,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    Ok(Region {
        rect,
        text: region.text.clone(),
        score: Some(region.score),
    })
}

fn desktop_redeem_code_bounding_ocr_region(
    regions: &[OcrResultRegion],
    roi: Rect,
    text: String,
) -> bgi_task::Result<Region> {
    let Some(first) = regions.first() else {
        return Err(TaskError::VisionPlan(
            "UseRedeemCode OCR aggregate matched without OCR regions".to_string(),
        ));
    };
    let mut left = first.rect.x;
    let mut top = first.rect.y;
    let mut right = first.rect.right();
    let mut bottom = first.rect.bottom();
    let mut score = first.score;

    for region in &regions[1..] {
        left = left.min(region.rect.x);
        top = top.min(region.rect.y);
        right = right.max(region.rect.right());
        bottom = bottom.max(region.rect.bottom());
        score = score.max(region.score);
    }

    let rect = Rect::new(roi.x + left, roi.y + top, right - left, bottom - top)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    Ok(Region {
        rect,
        text,
        score: Some(score),
    })
}

fn desktop_common_job_capture_size(game_window: Option<&GameWindowMatch>) -> VisionSize {
    game_window
        .and_then(|window| window.metrics)
        .map(|metrics| VisionSize::new(metrics.client_width, metrics.client_height))
        .unwrap_or_else(|| VisionSize::new(1920, 1080))
}

fn desktop_common_job_live_summary(
    report: &CommonJobLiveExecutionReport,
) -> (&'static str, bool, usize, usize) {
    match report {
        CommonJobLiveExecutionReport::ReturnMainUi(report) => (
            "ReturnMainUi",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::SetTime(report) => (
            "SetTime",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::ChooseTalkOption(report) => (
            "ChooseTalkOption",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::CheckRewards(report) => (
            "CheckRewards",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::BlessingOfTheWelkinMoon(report) => (
            "BlessingOfTheWelkinMoon",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::ClaimBattlePassRewards(report) => (
            "ClaimBattlePassRewards",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::ClaimEncounterPointsRewards(report) => (
            "ClaimEncounterPointsRewards",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::ClaimMailRewards(report) => (
            "ClaimMailRewards",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::CountInventoryItem(report) => (
            "CountInventoryItem",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::ScanPickDrops(report) => (
            "ScanPickDrops",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::LowerHeadThenWalkTo(report) => (
            "LowerHeadThenWalkTo",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::SwitchParty(report) => (
            "SwitchParty",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::GoToCraftingBench(report) => (
            "GoToCraftingBench",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::Teleport(report) => {
            ("Teleport", report.completed, report.executed_steps.len(), 0)
        }
        CommonJobLiveExecutionReport::GoToAdventurersGuild(report) => (
            "GoToAdventurersGuild",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::GoToSereniteaPot(report) => (
            "GoToSereniteaPot",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::OneKeyExpedition(report) => (
            "OneKeyExpedition",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::WonderlandCycle(report) => (
            "WonderlandCycle",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::Relogin(report) => (
            "Relogin",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
        CommonJobLiveExecutionReport::WalkToF(report) => (
            "WalkToF",
            report.completed,
            report.executed_steps.len(),
            report.skipped_steps.len(),
        ),
    }
}

fn desktop_common_job_global_input(
    config: &AppConfig,
    window: &GameWindowMatch,
    task_name: &str,
) -> Result<(bgi_script::GlobalInputHost, VisionSize), String> {
    let metrics = window
        .metrics
        .ok_or_else(|| format!("{task_name} live execution requires game window metrics"))?;
    let settings = CaptureSettings {
        mode: native_capture_mode(&config.capture_mode),
        auto_fix_win11_bit_blt: config.auto_fix_win11_bit_blt,
        ..CaptureSettings::default()
    };
    if !matches!(settings.mode, NativeCaptureMode::BitBlt) {
        return Err(format!(
            "{task_name} live execution requires the BitBlt capture backend"
        ));
    }

    let capture_area = metrics.capture_area;
    let source = DesktopGameCaptureFrameSource::new(window.handle, settings)
        .map_err(|error| error.to_string())?;
    let mut global_input = bgi_script::GlobalInputHost::new_with_frame_source(
        bgi_script::GameCaptureArea {
            x: capture_area.left,
            y: capture_area.top,
            width: metrics.client_width,
            height: metrics.client_height,
        },
        1.0,
        Some(Arc::new(source)),
    )
    .map_err(|error| error.to_string())?;
    global_input
        .set_game_metrics(metrics.client_width, metrics.client_height, 1.0)
        .map_err(|error| error.to_string())?;
    Ok((
        global_input,
        VisionSize::new(metrics.client_width, metrics.client_height),
    ))
}

fn desktop_common_capture_live_preflight(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_name: &str,
    plan_capture_size: VisionSize,
    cancellation: &InputCancellationToken,
) -> Result<VisionSize, String> {
    if cancellation.is_cancelled() {
        return Err(format!("{task_name} live execution cancelled"));
    }
    let Some(window) = game_window else {
        return Err(format!(
            "{task_name} live execution requires a detected game window"
        ));
    };
    let metrics = window
        .metrics
        .ok_or_else(|| format!("{task_name} live execution requires game window metrics"))?;
    let capture_mode = native_capture_mode(&config.capture_mode);
    if !matches!(capture_mode, NativeCaptureMode::BitBlt) {
        return Err(format!(
            "{task_name} live execution requires the BitBlt capture backend"
        ));
    }
    let capture_size = VisionSize::new(metrics.client_width, metrics.client_height);
    if plan_capture_size != capture_size {
        return Err(format!(
            "{task_name} live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan_capture_size.width,
            plan_capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    Ok(capture_size)
}

fn desktop_inventory_live_preflight(
    config: &AppConfig,
    game_window: Option<&GameWindowMatch>,
    task_name: &str,
    plan_capture_size: VisionSize,
    cancellation: &InputCancellationToken,
) -> Result<VisionSize, String> {
    desktop_common_capture_live_preflight(
        config,
        game_window,
        task_name,
        plan_capture_size,
        cancellation,
    )
}

fn execute_desktop_scan_pick_drops_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ScanPickDropsExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ScanPickDropsExecutionReport, String> {
    desktop_common_capture_live_preflight(
        config,
        Some(window),
        "ScanPickDrops",
        plan.capture_size,
        &cancellation,
    )?;
    Err(
        "ScanPickDrops live execution requires desktop BgiWorld ONNX YOLO inference and overlay adapters"
            .to_string(),
    )
}

fn execute_desktop_count_inventory_item_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &CountInventoryItemExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<CountInventoryItemExecutionReport, String> {
    desktop_inventory_live_preflight(
        config,
        Some(window),
        "CountInventoryItem",
        plan.capture_size,
        &cancellation,
    )?;
    desktop_count_inventory_item_live_preflight(plan)?;
    Err(
        "CountInventoryItem live execution requires desktop runtime adapter wiring after preflight"
            .to_string(),
    )
}

fn desktop_count_inventory_item_live_preflight(
    plan: &CountInventoryItemExecutionPlan,
) -> Result<(), String> {
    desktop_inventory_count_plan_live_preflight("CountInventoryItem", plan)
}

fn desktop_inventory_count_plan_live_preflight(
    task_name: &str,
    plan: &CountInventoryItemExecutionPlan,
) -> Result<(), String> {
    for step in &plan.steps {
        if !desktop_count_inventory_item_preflight_condition_applies(plan, step.condition) {
            continue;
        }

        match &step.action {
            CountInventoryItemStepAction::CommonJob { task_key }
                if task_key == RETURN_MAIN_UI_TASK_KEY => {}
            CountInventoryItemStepAction::CommonJob { task_key } => {
                return Err(format!(
                    "{task_name} live execution requires desktop nested common-job adapter for {task_key}"
                ));
            }
            CountInventoryItemStepAction::GenshinAction { action }
                if *action == GenshinAction::OpenInventory =>
            {
                return Err(format!(
                    "{task_name} live execution requires desktop inventory-opening adapter"
                ));
            }
            CountInventoryItemStepAction::GenshinAction { action } => {
                return Err(format!(
                    "{task_name} live execution requires desktop Genshin action adapter for {action:?}"
                ));
            }
            CountInventoryItemStepAction::ConfirmExpiredItemPrompt { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop expired-item prompt adapter"
                ));
            }
            CountInventoryItemStepAction::OpenInventoryTab { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop inventory tab adapter"
                ));
            }
            CountInventoryItemStepAction::LoadGridIconClassifier { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop GridIcon ONNX/prototype adapter"
                ));
            }
            CountInventoryItemStepAction::PreScrollWeaponOre { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop weapon-ore prescroll adapter"
                ));
            }
            CountInventoryItemStepAction::EnumerateGridItems { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop inventory grid enumeration adapter"
                ));
            }
            CountInventoryItemStepAction::CropGridIcon { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop grid icon crop adapter"
                ));
            }
            CountInventoryItemStepAction::InferGridIcon { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop GridIcon inference adapter"
                ));
            }
            CountInventoryItemStepAction::OcrGridItemCount { .. } => {
                return Err(format!(
                    "{task_name} live execution requires desktop item-count OCR adapter"
                ));
            }
            CountInventoryItemStepAction::ReturnResult { .. }
            | CountInventoryItemStepAction::ClearVisionDrawings
            | CountInventoryItemStepAction::Log { .. } => {}
        }
    }

    Ok(())
}

fn desktop_count_inventory_item_preflight_condition_applies(
    plan: &CountInventoryItemExecutionPlan,
    condition: CountInventoryItemStepCondition,
) -> bool {
    match condition {
        CountInventoryItemStepCondition::Always
        | CountInventoryItemStepCondition::WhenExpiredItemPromptDetected
        | CountInventoryItemStepCondition::WhenInventoryTabUnchecked
        | CountInventoryItemStepCondition::WhenStillOnMainUi
        | CountInventoryItemStepCondition::WhenClassifierMatchesTarget
        | CountInventoryItemStepCondition::WhenAllRequestedItemsFound
        | CountInventoryItemStepCondition::WhenScanComplete => true,
        CountInventoryItemStepCondition::WhenWeaponOreRequested => plan
            .search_mode
            .needs_weapon_ore_prescroll(&plan.grid_screen_name),
    }
}

fn execute_desktop_return_main_ui_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ReturnMainUiExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ReturnMainUiExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("ReturnMainUi live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "ReturnMainUi")?;
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "ReturnMainUi live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_return_main_ui_live(
        capture_size,
        plan.max_escape_attempts,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_set_time_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &SetTimeExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<SetTimeExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("SetTime live execution cancelled".to_string());
    }
    let (global_input, capture_size) = desktop_common_job_global_input(config, window, "SetTime")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "SetTime live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "SetTime live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_set_time_live(
        plan,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_one_key_expedition_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &OneKeyExpeditionExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<OneKeyExpeditionExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("OneKeyExpedition live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "OneKeyExpedition")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "OneKeyExpedition live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "OneKeyExpedition live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_one_key_expedition_live(
        plan,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_go_to_crafting_bench_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &GoToCraftingBenchExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<GoToCraftingBenchExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("GoToCraftingBench live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "GoToCraftingBench")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "GoToCraftingBench live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    desktop_go_to_crafting_bench_live_preflight(plan)?;
    let frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "GoToCraftingBench live execution has no capture frame source".to_string()
    })?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime = DesktopGoToCraftingBenchRuntime::new(common_runtime);
    execute_go_to_crafting_bench_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

fn desktop_go_to_crafting_bench_live_preflight(
    plan: &GoToCraftingBenchExecutionPlan,
) -> Result<(), String> {
    for step in &plan.steps {
        match &step.action {
            GoToCraftingBenchStepAction::Pathing { rule } => {
                desktop_common_job_pathing_live_preflight("GoToCraftingBench", &rule.pathing_json)?;
            }
            GoToCraftingBenchStepAction::InteractionRetry { .. } => {
                return Err(
                    "GoToCraftingBench live execution requires desktop crafting-bench interaction adapter"
                        .to_string(),
                );
            }
            GoToCraftingBenchStepAction::SelectLastTalkOptionUntilEnd { .. } => {
                return Err(
                    "GoToCraftingBench live execution requires desktop talk-option selection adapter"
                        .to_string(),
                );
            }
            GoToCraftingBenchStepAction::RecognizeResinCounts { .. } => {
                return Err(
                    "GoToCraftingBench live execution requires desktop resin-count OCR adapter"
                        .to_string(),
                );
            }
            GoToCraftingBenchStepAction::CraftCondensedResin { .. } => {
                return Err(
                    "GoToCraftingBench live execution requires desktop condensed-resin crafting adapter"
                        .to_string(),
                );
            }
            GoToCraftingBenchStepAction::Log { .. }
            | GoToCraftingBenchStepAction::Page { .. }
            | GoToCraftingBenchStepAction::Locator { .. }
            | GoToCraftingBenchStepAction::GenshinAction { .. }
            | GoToCraftingBenchStepAction::DetectResin { .. }
            | GoToCraftingBenchStepAction::ComputeCraftsNeeded { .. }
            | GoToCraftingBenchStepAction::Input { .. }
            | GoToCraftingBenchStepAction::CommonJob { .. }
            | GoToCraftingBenchStepAction::ReturnResult { .. } => {}
        }
    }
    Ok(())
}

fn desktop_common_job_pathing_live_preflight(
    task_name: &str,
    pathing_json: &str,
) -> Result<bgi_task::CommonJobPathingPreflightReport, String> {
    let report = bgi_task::preflight_common_job_pathing_rule(pathing_json).map_err(|error| {
        format!(
            "{task_name} live execution failed PathExecutor preflight for {pathing_json}: {error}"
        )
    })?;
    if report.native_pathing_completed {
        return Err(format!(
            "{task_name} live execution cannot consume an already-completed PathExecutor report for {pathing_json}"
        ));
    }
    Ok(report)
}

struct DesktopGoToCraftingBenchRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
}

impl<F, I, C> DesktopGoToCraftingBenchRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>) -> Self {
        Self { common }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopGoToCraftingBenchRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> GoToCraftingBenchRuntime for DesktopGoToCraftingBenchRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn execute_crafting_bench_pathing(
        &mut self,
        rule: &GoToCraftingBenchPathingRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        let report = bgi_task::preflight_common_job_pathing_rule(&rule.pathing_json)?;
        Err(TaskError::CommonJobExecution(format!(
            "GoToCraftingBench live execution requires desktop PathExecutor movement adapter for {} after validating {} waypoints",
            rule.pathing_json, report.waypoint_count
        )))
    }

    fn retry_crafting_bench_interaction(
        &mut self,
        _rule: &GoToCraftingBenchInteractionRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Err(TaskError::CommonJobExecution(
            "GoToCraftingBench live execution requires desktop crafting-bench interaction adapter"
                .to_string(),
        ))
    }

    fn select_last_crafting_bench_talk_option_until_end(
        &mut self,
        _until_locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Err(TaskError::CommonJobExecution(
            "GoToCraftingBench live execution requires desktop talk-option selection adapter"
                .to_string(),
        ))
    }

    fn recognize_crafting_bench_resin_counts(
        &mut self,
        _rule: &GoToCraftingBenchResinRecognitionRule,
    ) -> bgi_task::Result<Option<GoToCraftingBenchResinCounts>> {
        Err(TaskError::CommonJobExecution(
            "GoToCraftingBench live execution requires desktop resin-count OCR adapter".to_string(),
        ))
    }

    fn craft_condensed_resin(
        &mut self,
        _rule: &GoToCraftingBenchResinCraftRule,
        _crafts_needed: u8,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Err(TaskError::CommonJobExecution(
            "GoToCraftingBench live execution requires desktop condensed-resin crafting adapter"
                .to_string(),
        ))
    }
}

fn execute_desktop_go_to_adventurers_guild_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &GoToAdventurersGuildExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<bgi_task::GoToAdventurersGuildExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("GoToAdventurersGuild live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "GoToAdventurersGuild")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "GoToAdventurersGuild live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let _frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "GoToAdventurersGuild live execution has no capture frame source".to_string()
    })?;
    desktop_go_to_adventurers_guild_live_preflight(plan)?;
    Err(
        "GoToAdventurersGuild live execution requires desktop runtime adapter wiring after preflight"
            .to_string(),
    )
}

fn desktop_go_to_adventurers_guild_live_preflight(
    plan: &GoToAdventurersGuildExecutionPlan,
) -> Result<(), String> {
    for step in &plan.steps {
        if !desktop_go_to_adventurers_guild_preflight_condition_applies(plan, step.condition) {
            continue;
        }
        match &step.action {
            GoToAdventurersGuildStepAction::CommonJob { task_key, .. } => {
                if !desktop_go_to_adventurers_guild_nested_common_job_bridge_available(task_key) {
                    return Err(format!(
                        "GoToAdventurersGuild live execution has no desktop bridge for nested common-job {task_key} at phase {:?} ({})",
                        step.phase, step.label
                    ));
                }
            }
            GoToAdventurersGuildStepAction::Pathing { rule } => {
                desktop_common_job_pathing_live_preflight(
                    "GoToAdventurersGuild",
                    &rule.pathing_json,
                )?;
            }
            GoToAdventurersGuildStepAction::InteractionRetry { .. } => {
                return Err(format!(
                    "GoToAdventurersGuild live execution requires desktop Catherine interaction adapter at phase {:?} ({})",
                    step.phase, step.label
                ));
            }
            GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd { .. } => {
                if step.condition == GoToAdventurersGuildStepCondition::WhenTalkUiStillOpen {
                    return Err(format!(
                        "GoToAdventurersGuild live execution requires desktop talk UI probe/drain adapter at phase {:?} ({})",
                        step.phase, step.label
                    ));
                }
                return Err(format!(
                    "GoToAdventurersGuild live execution requires desktop talk-option drain adapter at phase {:?} ({})",
                    step.phase, step.label
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn desktop_go_to_adventurers_guild_preflight_condition_applies(
    plan: &GoToAdventurersGuildExecutionPlan,
    condition: GoToAdventurersGuildStepCondition,
) -> bool {
    match condition {
        GoToAdventurersGuildStepCondition::WhenDailyRewardPartyConfigured => plan
            .daily_reward_party_name
            .as_deref()
            .is_some_and(|party_name| !party_name.trim().is_empty()),
        GoToAdventurersGuildStepCondition::WhenOnlyDoOnceFalse => !plan.only_do_once,
        _ => true,
    }
}

fn desktop_go_to_adventurers_guild_nested_common_job_bridge_available(task_key: &str) -> bool {
    matches!(
        task_key,
        bgi_task::SWITCH_PARTY_TASK_KEY
            | bgi_task::CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY
            | bgi_task::CHOOSE_TALK_OPTION_TASK_KEY
            | bgi_task::RETURN_MAIN_UI_TASK_KEY
    )
}

fn execute_desktop_go_to_serenitea_pot_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &GoToSereniteaPotExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<GoToSereniteaPotExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("GoToSereniteaPot live execution cancelled".to_string());
    }
    let (_global_input, capture_size) =
        desktop_common_job_global_input(config, window, "GoToSereniteaPot")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "GoToSereniteaPot live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    desktop_go_to_serenitea_pot_live_preflight(plan)?;
    Err(
        "GoToSereniteaPot live execution requires desktop runtime adapter wiring after preflight"
            .to_string(),
    )
}

fn desktop_go_to_serenitea_pot_live_preflight(
    plan: &GoToSereniteaPotExecutionPlan,
) -> Result<(), String> {
    for step in &plan.steps {
        if !desktop_go_to_serenitea_pot_preflight_condition_applies(plan, step.condition) {
            continue;
        }
        match &step.action {
            GoToSereniteaPotStepAction::MapEntry { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop Serenitea Pot map-entry adapter"
                        .to_string(),
                );
            }
            GoToSereniteaPotStepAction::BagEntry { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop Serenitea Pot bag-entry adapter"
                        .to_string(),
                );
            }
            GoToSereniteaPotStepAction::FindAYuan { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop A Yuan interaction adapter"
                        .to_string(),
                );
            }
            GoToSereniteaPotStepAction::Reward { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop Serenitea Pot reward adapter"
                        .to_string(),
                );
            }
            GoToSereniteaPotStepAction::ShopPurchase { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop realm-depot shop adapter"
                        .to_string(),
                );
            }
            GoToSereniteaPotStepAction::Finish { .. } => {
                return Err(
                    "GoToSereniteaPot live execution requires desktop Serenitea Pot finish adapter"
                        .to_string(),
                );
            }
            _ => {}
        }
    }
    Ok(())
}

fn desktop_go_to_serenitea_pot_preflight_condition_applies(
    plan: &GoToSereniteaPotExecutionPlan,
    condition: GoToSereniteaPotStepCondition,
) -> bool {
    match condition {
        GoToSereniteaPotStepCondition::WhenMapTeleportConfigured => {
            plan.entry_mode == GoToSereniteaPotEntryMode::MapTeleport
        }
        GoToSereniteaPotStepCondition::WhenBagGadgetConfigured => {
            plan.entry_mode == GoToSereniteaPotEntryMode::BagGadget
        }
        GoToSereniteaPotStepCondition::WhenShopConfiguredAndDue => {
            !plan.secret_treasure_objects.is_empty()
        }
        _ => true,
    }
}

fn execute_desktop_choose_talk_option_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ChooseTalkOptionExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ChooseTalkOptionExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("ChooseTalkOption live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "ChooseTalkOption")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "ChooseTalkOption live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "ChooseTalkOption live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime =
        DesktopChooseTalkOptionRuntime::new(common_runtime, plan.option_icon_locator.clone());
    execute_choose_talk_option_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopChooseTalkOptionRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
    option_icon_locator: BvLocatorPlan,
}

impl<F, I, C> DesktopChooseTalkOptionRuntime<F, I, C> {
    fn new(
        common: PureTemplateCommonJobRuntime<F, I, C>,
        option_icon_locator: BvLocatorPlan,
    ) -> Self {
        Self {
            common,
            option_icon_locator,
        }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopChooseTalkOptionRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> ChooseTalkOptionRuntime for DesktopChooseTalkOptionRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn recognize_talk_options(
        &mut self,
        rule: &ChooseTalkOptionOcrRule,
    ) -> bgi_task::Result<Vec<ChooseTalkOptionCandidate>> {
        let frame = self.common.frame_source_mut().capture_frame()?;
        let mut icon_regions = self
            .common
            .vision_backend()
            .find_multi(
                &frame.pixels,
                frame.size,
                &self.option_icon_locator.recognition_object,
            )
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        icon_regions.retain(|region| region.is_exist());
        if rule.sort_icons_by_y_descending {
            icon_regions.sort_by_key(|region| std::cmp::Reverse(region.rect.y));
        } else {
            icon_regions.sort_by_key(|region| region.rect.y);
        }
        let Some(lowest_icon) = icon_regions.first() else {
            return Ok(Vec::new());
        };
        let ocr_roi =
            choose_talk_option_ocr_rect_from_lowest_icon(lowest_icon.rect, frame.size, rule)?;
        let cropped = crop_bgr_image(&frame, ocr_roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("ChooseTalkOption WinRT OCR failed: {error}"))
        })?;
        choose_talk_option_candidates_from_ocr_regions(&regions, ocr_roi, rule)
    }

    fn is_orange_talk_option(
        &mut self,
        candidate: &ChooseTalkOptionCandidate,
        rule: &ChooseTalkOptionOrangeRule,
    ) -> bgi_task::Result<bool> {
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, candidate.rect)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let hsv = convert_bgr_image(&cropped.pixels, cropped.size, ColorConversion::BgrToHsv)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let mask = in_range_mask(&hsv, rule.hsv_lower, rule.hsv_upper, None)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let pixel_count = cropped.size.width as f64 * cropped.size.height as f64;
        let orange_pixel_rate = if pixel_count <= 0.0 {
            0.0
        } else {
            mask.matched_count as f64 / pixel_count
        };
        Ok(orange_pixel_rate > rule.min_pixel_rate)
    }

    fn click_talk_option(&mut self, candidate: &ChooseTalkOptionCandidate) -> bgi_task::Result<()> {
        let center = candidate.rect.center();
        self.common
            .input_driver_mut()
            .click_capture_point(center.x, center.y)
    }
}

fn execute_desktop_check_rewards_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &CheckRewardsExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<CheckRewardsExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("CheckRewards live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "CheckRewards")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "CheckRewards live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "CheckRewards live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime = DesktopCheckRewardsRuntime::new(common_runtime);
    execute_check_rewards_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

struct DesktopCheckRewardsRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
}

impl<F, I, C> DesktopCheckRewardsRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>) -> Self {
        Self { common }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopCheckRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> CheckRewardsRuntime for DesktopCheckRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn recognize_check_rewards_text(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<Vec<CheckRewardsTextCandidate>> {
        let requested_roi = match command {
            BvPageCommand::Ocr { locator } => locator
                .recognition_object
                .region_of_interest
                .unwrap_or_else(Rect::empty),
            _ => {
                return Err(TaskError::CommonJobExecution(
                    "CheckRewards OCR requires a BvPageCommand::Ocr command".to_string(),
                ));
            }
        };
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, requested_roi)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("CheckRewards WinRT OCR failed: {error}"))
        })?;
        check_rewards_text_candidates_from_ocr_regions(&regions, roi)
    }

    fn click_check_rewards_text(
        &mut self,
        candidate: &CheckRewardsTextCandidate,
    ) -> bgi_task::Result<()> {
        let center = candidate.rect.center();
        self.common
            .input_driver_mut()
            .click_capture_point(center.x, center.y)
    }

    fn notify_check_rewards(
        &mut self,
        _payload: &NotificationPayload,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::None)
    }
}

fn execute_desktop_claim_battle_pass_rewards_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ClaimBattlePassRewardsExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ClaimBattlePassRewardsExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("ClaimBattlePassRewards live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "ClaimBattlePassRewards")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "ClaimBattlePassRewards live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "ClaimBattlePassRewards live execution has no capture frame source".to_string()
    })?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime = DesktopClaimBattlePassRewardsRuntime::new(common_runtime);
    execute_claim_battle_pass_rewards_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

struct DesktopClaimBattlePassRewardsRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
}

impl<F, I, C> DesktopClaimBattlePassRewardsRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>) -> Self {
        Self { common }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopClaimBattlePassRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> ClaimBattlePassRewardsRuntime for DesktopClaimBattlePassRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn recognize_battle_pass_reward_text(
        &mut self,
        command: &BvPageCommand,
        rule: &BattlePassClaimAllRule,
        _scope: BattlePassClaimScope,
    ) -> bgi_task::Result<Vec<BattlePassRewardTextCandidate>> {
        let requested_roi = match command {
            BvPageCommand::Ocr { locator } => locator
                .recognition_object
                .region_of_interest
                .unwrap_or(rule.ocr_roi),
            _ => {
                return Err(TaskError::CommonJobExecution(
                    "ClaimBattlePassRewards OCR requires a BvPageCommand::Ocr command".to_string(),
                ));
            }
        };
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, requested_roi)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "ClaimBattlePassRewards WinRT OCR failed: {error}"
            ))
        })?;
        battle_pass_reward_text_candidates_from_ocr_regions(&regions, roi)
    }

    fn click_battle_pass_reward_text(
        &mut self,
        candidate: &BattlePassRewardTextCandidate,
        _scope: BattlePassClaimScope,
    ) -> bgi_task::Result<()> {
        let center = candidate.rect.center();
        self.common
            .input_driver_mut()
            .click_capture_point(center.x, center.y)
    }
}

fn execute_desktop_blessing_of_the_welkin_moon_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &BlessingOfTheWelkinMoonExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<BlessingOfTheWelkinMoonExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("BlessingOfTheWelkinMoon live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "BlessingOfTheWelkinMoon")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "BlessingOfTheWelkinMoon live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let server_time_zone_offset_minutes =
        desktop_server_time_zone_offset_minutes(config, "BlessingOfTheWelkinMoon")?;
    let frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "BlessingOfTheWelkinMoon live execution has no capture frame source".to_string()
    })?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_blessing_of_the_welkin_moon_live(
        plan,
        server_time_zone_offset_minutes,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_claim_mail_rewards_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ClaimMailRewardsExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ClaimMailRewardsExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("ClaimMailRewards live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "ClaimMailRewards")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "ClaimMailRewards live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "ClaimMailRewards live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_claim_mail_rewards_live(
        plan,
        &config.key_bindings_config,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_claim_encounter_points_rewards_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ClaimEncounterPointsRewardsExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ClaimEncounterPointsRewardsExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("ClaimEncounterPointsRewards live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "ClaimEncounterPointsRewards")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "ClaimEncounterPointsRewards live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "ClaimEncounterPointsRewards live execution has no capture frame source".to_string()
    })?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime = DesktopClaimEncounterPointsRewardsRuntime::new(common_runtime);
    execute_claim_encounter_points_rewards_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

struct DesktopClaimEncounterPointsRewardsRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
}

impl<F, I, C> DesktopClaimEncounterPointsRewardsRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>) -> Self {
        Self { common }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopClaimEncounterPointsRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> ClaimEncounterPointsRewardsRuntime
    for DesktopClaimEncounterPointsRewardsRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn recognize_encounter_points_text(
        &mut self,
        command: &BvPageCommand,
        rule: &ClaimEncounterPointsRewardsOcrRule,
    ) -> bgi_task::Result<Vec<ClaimEncounterPointsRewardsTextCandidate>> {
        let requested_roi = match command {
            BvPageCommand::Ocr { locator } => locator
                .recognition_object
                .region_of_interest
                .unwrap_or(rule.left_panel_roi),
            _ => {
                return Err(TaskError::CommonJobExecution(
                    "ClaimEncounterPointsRewards OCR requires a BvPageCommand::Ocr command"
                        .to_string(),
                ));
            }
        };
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, requested_roi)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "ClaimEncounterPointsRewards WinRT OCR failed: {error}"
            ))
        })?;
        claim_encounter_points_text_candidates_from_ocr_regions(&regions, roi)
    }

    fn click_encounter_points_text(
        &mut self,
        candidate: &ClaimEncounterPointsRewardsTextCandidate,
    ) -> bgi_task::Result<()> {
        let center = candidate.rect.center();
        self.common
            .input_driver_mut()
            .click_capture_point(center.x, center.y)
    }
}

fn execute_desktop_switch_party_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &SwitchPartyExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<SwitchPartyExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("SwitchParty live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "SwitchParty")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "SwitchParty live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "SwitchParty live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let mut runtime = DesktopSwitchPartyRuntime::new(common_runtime);
    execute_switch_party_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

struct DesktopSwitchPartyRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
}

impl<F, I, C> DesktopSwitchPartyRuntime<F, I, C> {
    fn new(common: PureTemplateCommonJobRuntime<F, I, C>) -> Self {
        Self { common }
    }
}

impl<F, I, C> DesktopSwitchPartyRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn wait_ms(&mut self, milliseconds: u32) -> bgi_task::Result<()> {
        let command = BvPageCommand::Wait { milliseconds };
        CommonJobRuntime::execute_page_command(&mut self.common, &command).map(|_| ())
    }

    fn locator_matched(&mut self, locator: &BvLocatorPlan, label: &str) -> bgi_task::Result<bool> {
        let outcome = CommonJobRuntime::execute_locator(&mut self.common, locator)?;
        desktop_switch_party_outcome_as_bool(outcome, label)
    }

    fn click_locator_once(
        &mut self,
        locator: &BvLocatorPlan,
        timeout_ms: u32,
        label: &str,
    ) -> bgi_task::Result<bool> {
        let mut click_locator = locator.clone();
        click_locator.operation = BvLocatorOperation::Click;
        click_locator.timeout_ms = timeout_ms.max(1);
        click_locator.retry_interval_ms = timeout_ms.max(1);
        click_locator.retry_count = 1;
        self.locator_matched(&click_locator, label)
    }

    fn recognize_switch_party_ocr_roi(
        &mut self,
        requested_roi: Rect,
    ) -> bgi_task::Result<Vec<SwitchPartyTextCandidate>> {
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_ocr_roi_for_image(frame.size, requested_roi)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("SwitchParty WinRT OCR failed: {error}"))
        })?;
        switch_party_text_candidates_from_ocr_regions(&regions, roi)
    }

    fn click_capture_point(&mut self, x: i32, y: i32) -> bgi_task::Result<()> {
        self.common.input_driver_mut().click_capture_point(x, y)
    }
}

impl<F, I, C> CommonJobRuntime for DesktopSwitchPartyRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> SwitchPartyRuntime for DesktopSwitchPartyRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn recognize_switch_party_text(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<Vec<SwitchPartyTextCandidate>> {
        let requested_roi = match command {
            BvPageCommand::Ocr { locator } => locator
                .recognition_object
                .region_of_interest
                .ok_or_else(|| {
                    TaskError::CommonJobExecution(
                        "SwitchParty OCR command has no region of interest".to_string(),
                    )
                })?,
            _ => {
                return Err(TaskError::CommonJobExecution(
                    "SwitchParty OCR requires a BvPageCommand::Ocr command".to_string(),
                ));
            }
        };
        self.recognize_switch_party_ocr_roi(requested_roi)
    }

    fn open_switch_party_choose_menu(
        &mut self,
        rule: &SwitchPartyChooseMenuRule,
        choose_locator: &BvLocatorPlan,
        delete_locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        let attempts = rule.open_attempts.max(1);
        for attempt in 0..attempts {
            if self.locator_matched(delete_locator, "SwitchParty delete button probe")? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
            let _ = self.click_locator_once(
                choose_locator,
                rule.open_interval_ms,
                "SwitchParty choose-view button click",
            )?;
            self.wait_ms(rule.open_interval_ms)?;
            if self.locator_matched(delete_locator, "SwitchParty delete button after click")? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
            if attempt + 1 < attempts {
                self.wait_ms(rule.open_interval_ms)?;
            }
        }

        for attempt in 0..rule.delete_verify_attempts.max(1) {
            if self.locator_matched(delete_locator, "SwitchParty delete button verification")? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
            if attempt + 1 < rule.delete_verify_attempts.max(1) {
                self.wait_ms(rule.open_interval_ms)?;
            }
        }
        Ok(CommonJobRuntimeOutcome::Matched(false))
    }

    fn scan_switch_party_list(
        &mut self,
        rule: &SwitchPartyListScanRule,
        party_name: &str,
        current_page_texts: &[SwitchPartyTextCandidate],
    ) -> bgi_task::Result<SwitchPartyListScanOutcome> {
        let mut page_texts = current_page_texts.to_vec();
        let mut scanned_pages = 0;

        for page_index in 0..rule.max_pages.max(1) {
            scanned_pages = page_index + 1;
            if page_texts.is_empty() {
                return Ok(SwitchPartyListScanOutcome {
                    scanned_pages,
                    matched_party: None,
                    reached_end: true,
                });
            }

            if let Some(candidate) =
                switch_party_find_matching_text_candidate(&page_texts, party_name, true)?
            {
                let click_x = candidate.rect.x
                    + (candidate.rect.width as f64 * rule.matched_party_click_x_multiplier).round()
                        as i32;
                let click_y = match rule.matched_party_click_y_anchor {
                    PartyTextClickYAnchor::Bottom => candidate.rect.bottom(),
                };
                self.click_capture_point(click_x, click_y)?;
                self.wait_ms(rule.after_matched_party_click_delay_ms)?;
                return Ok(SwitchPartyListScanOutcome {
                    scanned_pages,
                    matched_party: Some(candidate),
                    reached_end: false,
                });
            }

            let lowest = page_texts
                .iter()
                .filter(|candidate| {
                    candidate.rect.x > rule.lowest_item_x_min
                        && candidate.rect.x < rule.lowest_item_x_max
                })
                .max_by_key(|candidate| candidate.rect.y);
            let Some(lowest) = lowest else {
                return Ok(SwitchPartyListScanOutcome {
                    scanned_pages,
                    matched_party: None,
                    reached_end: true,
                });
            };
            if lowest.rect.y < rule.last_item_threshold_y {
                return Ok(SwitchPartyListScanOutcome {
                    scanned_pages,
                    matched_party: None,
                    reached_end: true,
                });
            }

            if page_index == 0 {
                self.click_capture_point(
                    rule.first_page_preclick.screen_x.round() as i32,
                    rule.first_page_preclick.screen_y.round() as i32,
                )?;
                self.wait_ms(rule.first_page_preclick_delay_ms)?;
            }

            self.click_capture_point(
                rule.ocr_roi.x + rule.ocr_roi.width / 2,
                lowest.rect.bottom(),
            )?;
            self.wait_ms(rule.page_scroll_delay_ms)?;
            page_texts = self.recognize_switch_party_ocr_roi(rule.ocr_roi)?;
        }

        Ok(SwitchPartyListScanOutcome {
            scanned_pages,
            matched_party: None,
            reached_end: true,
        })
    }

    fn confirm_switch_party(
        &mut self,
        rule: &SwitchPartyConfirmRule,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if !self.locator_matched(&rule.left_confirm_locator, "SwitchParty left confirm click")? {
            return Ok(CommonJobRuntimeOutcome::Matched(false));
        }

        let mut choose_menu_closed = false;
        for attempt in 0..rule.close_check_attempts.max(1) {
            if !self.locator_matched(
                &rule.party_delete_locator,
                "SwitchParty choose menu close probe",
            )? {
                choose_menu_closed = true;
                break;
            }
            if attempt + 1 < rule.close_check_attempts.max(1) {
                self.wait_ms(rule.party_delete_locator.retry_interval_ms)?;
            }
        }
        if !choose_menu_closed {
            return Ok(CommonJobRuntimeOutcome::Matched(false));
        }

        self.wait_ms(rule.after_first_confirm_delay_ms)?;
        let _ = self.locator_matched(
            &rule.right_confirm_locator,
            "SwitchParty right confirm click",
        )?;
        self.wait_ms(rule.after_second_confirm_delay_ms)?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn clear_switch_party_combat_scenes(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::None)
    }
}

fn desktop_switch_party_outcome_as_bool(
    outcome: CommonJobRuntimeOutcome,
    label: &str,
) -> bgi_task::Result<bool> {
    match outcome {
        CommonJobRuntimeOutcome::Matched(value) => Ok(value),
        CommonJobRuntimeOutcome::None => Err(TaskError::CommonJobExecution(format!(
            "SwitchParty {label} did not return a match result"
        ))),
    }
}

fn desktop_ocr_roi_for_image(image_size: VisionSize, roi: Rect) -> bgi_task::Result<Rect> {
    let roi = if roi.is_empty() {
        Rect::new(0, 0, image_size.width as i32, image_size.height as i32)
    } else {
        Ok(roi)
    }
    .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = roi
        .clamp_to(image_size)
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    if roi.width <= 0 || roi.height <= 0 {
        return Err(TaskError::VisionPlan(
            "desktop OCR ROI is empty after clamping to the captured image".to_string(),
        ));
    }
    Ok(roi)
}

fn desktop_winrt_ocr_bgr_image(image: &BgrImage) -> Result<Vec<OcrResultRegion>, String> {
    let png = encode_bgr_image_png(image)?;
    desktop_winrt_ocr_png_regions(&png)
}

fn encode_bgr_image_png(image: &BgrImage) -> Result<Vec<u8>, String> {
    let rgb = image.to_rgb_bytes();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &rgb,
            image.size.width,
            image.size.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|error| error.to_string())?;
    Ok(png)
}

fn desktop_winrt_ocr_png_regions(png: &[u8]) -> Result<Vec<OcrResultRegion>, String> {
    use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat};
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

    let stream = InMemoryRandomAccessStream::new().map_err(|error| error.to_string())?;
    let output_stream = stream
        .GetOutputStreamAt(0)
        .map_err(|error| error.to_string())?;
    let writer = DataWriter::CreateDataWriter(&output_stream).map_err(|error| error.to_string())?;
    writer.WriteBytes(png).map_err(|error| error.to_string())?;
    writer
        .StoreAsync()
        .map_err(|error| error.to_string())?
        .get()
        .map_err(|error| error.to_string())?;
    writer.DetachStream().map_err(|error| error.to_string())?;
    stream.Seek(0).map_err(|error| error.to_string())?;

    let decoder = BitmapDecoder::CreateAsync(&stream)
        .map_err(|error| error.to_string())?
        .get()
        .map_err(|error| error.to_string())?;
    let bitmap = decoder
        .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Ignore)
        .map_err(|error| error.to_string())?
        .get()
        .map_err(|error| error.to_string())?;
    let engine =
        OcrEngine::TryCreateFromUserProfileLanguages().map_err(|error| error.to_string())?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(|error| error.to_string())?
        .get()
        .map_err(|error| error.to_string())?;
    let lines = result.Lines().map_err(|error| error.to_string())?;
    let mut regions = Vec::new();
    for line_index in 0..lines.Size().map_err(|error| error.to_string())? {
        let line = lines.GetAt(line_index).map_err(|error| error.to_string())?;
        let words = line.Words().map_err(|error| error.to_string())?;
        for word_index in 0..words.Size().map_err(|error| error.to_string())? {
            let word = words.GetAt(word_index).map_err(|error| error.to_string())?;
            let text = word.Text().map_err(|error| error.to_string())?.to_string();
            let bounds = word.BoundingRect().map_err(|error| error.to_string())?;
            let width = bounds.Width.round().max(0.0) as i32;
            let height = bounds.Height.round().max(0.0) as i32;
            if text.trim().is_empty() || width <= 0 || height <= 0 {
                continue;
            }
            let rect = Rect::new(
                bounds.X.round().max(0.0) as i32,
                bounds.Y.round().max(0.0) as i32,
                width,
                height,
            )
            .map_err(|error| error.to_string())?;
            regions.push(OcrResultRegion {
                rect,
                text,
                score: 1.0,
            });
        }
    }
    Ok(regions)
}

fn desktop_server_time_zone_offset_minutes(
    config: &AppConfig,
    task_name: &str,
) -> Result<i32, String> {
    let offset = bgi_script::ServerTimeHost::from_offset_string(
        &config.other_config.server_time_zone_offset,
    )
    .map_err(|error| format!("{task_name} live execution has invalid server timezone: {error}"))?;
    Ok(offset.server_time_zone_offset_milliseconds() / 60_000)
}

fn desktop_execution_record_clock(
    config: &AppConfig,
) -> Result<bgi_script::ExecutionRecordClock, String> {
    let local_now = Local::now();
    let local_offset = local_now.offset().fix();
    let offset = bgi_script::ServerTimeHost::from_offset_string(
        &config.other_config.server_time_zone_offset,
    )
    .map_err(|error| format!("script execution has invalid server timezone: {error}"))?;
    let server_offset_seconds = offset.server_time_zone_offset_milliseconds() / 1_000;
    let server_offset = FixedOffset::east_opt(server_offset_seconds).ok_or_else(|| {
        format!("script execution has invalid server timezone seconds: {server_offset_seconds}")
    })?;

    Ok(bgi_script::ExecutionRecordClock::fixed(
        local_now.naive_local(),
        local_offset,
        local_now.with_timezone(&server_offset),
    ))
}

fn execute_desktop_wonderland_cycle_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &WonderlandCycleExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<WonderlandCycleExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("WonderlandCycle live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "WonderlandCycle")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "WonderlandCycle live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "WonderlandCycle live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_wonderland_cycle_live(
        plan,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_walk_to_f_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &WalkToFExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<WalkToFExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("WalkToF live execution cancelled".to_string());
    }
    let (global_input, capture_size) = desktop_common_job_global_input(config, window, "WalkToF")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "WalkToF live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "WalkToF live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_walk_to_f_live(
        plan,
        &config.key_bindings_config,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
}

fn execute_desktop_lower_head_then_walk_to_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &LowerHeadThenWalkToExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<LowerHeadThenWalkToExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("LowerHeadThenWalkTo live execution cancelled".to_string());
    }
    let (global_input, capture_size) =
        desktop_common_job_global_input(config, window, "LowerHeadThenWalkTo")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "LowerHeadThenWalkTo live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input.common_job_frame_source().ok_or_else(|| {
        "LowerHeadThenWalkTo live execution has no capture frame source".to_string()
    })?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(Arc::clone(&cancellation)),
    );
    let mut runtime = DesktopLowerHeadThenWalkToRuntime::new(
        common_runtime,
        config.key_bindings_config.clone(),
        plan.capture_size,
        plan.timeout_ms,
        cancellation,
    );
    execute_lower_head_then_walk_to_plan(plan, &config.key_bindings_config, &mut runtime)
        .map_err(|error| error.to_string())
}

struct DesktopLowerHeadThenWalkToRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
    key_bindings: KeyBindingsConfig,
    capture_size: VisionSize,
    timeout_ms: u32,
    dpi_scale: f64,
    cancellation: Arc<InputCancellationToken>,
}

impl<F, I, C> DesktopLowerHeadThenWalkToRuntime<F, I, C> {
    fn new(
        common: PureTemplateCommonJobRuntime<F, I, C>,
        key_bindings: KeyBindingsConfig,
        capture_size: VisionSize,
        timeout_ms: u32,
        cancellation: Arc<InputCancellationToken>,
    ) -> Self {
        Self {
            common,
            key_bindings,
            capture_size,
            timeout_ms,
            dpi_scale: 1.0,
            cancellation,
        }
    }
}

impl<F, I, C> DesktopLowerHeadThenWalkToRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn ensure_not_cancelled(&self) -> bgi_task::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(TaskError::CommonJobExecution(
                "LowerHeadThenWalkTo live execution cancelled".to_string(),
            ));
        }
        Ok(())
    }

    fn capture_image_region(&mut self) -> bgi_task::Result<ImageRegion> {
        self.ensure_not_cancelled()?;
        let image = self.common.frame_source_mut().capture_frame()?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_region_on_capture(
        &self,
        capture: &ImageRegion,
        locator: &BvLocatorPlan,
        label: &str,
    ) -> bgi_task::Result<Option<Region>> {
        let region = capture
            .find(self.common.vision_backend(), &locator.recognition_object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "LowerHeadThenWalkTo {label} template lookup failed: {error}"
                ))
            })?;
        Ok(region.is_exist().then_some(region))
    }

    fn activation_text_detected(
        &self,
        capture: &ImageRegion,
        f_key_rule: &LowerHeadThenWalkToFKeyRule,
    ) -> bgi_task::Result<bool> {
        let Some(f_key_region) =
            self.locate_region_on_capture(capture, &f_key_rule.pick_key_locator, "F-key")?
        else {
            return Ok(false);
        };
        let scale = capture.image.size.width as f64 / 1920.0;
        let text_rect = Rect::new(
            f_key_region.rect.x + (f64::from(f_key_rule.text_x_offset_1080p) * scale) as i32,
            f_key_region.rect.y,
            ((f64::from(f_key_rule.text_width_1080p) * scale) as i32).max(1),
            f_key_region.rect.height.max(1),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        if text_rect.x < 0
            || text_rect.y < 0
            || text_rect.x + text_rect.width > capture.image.size.width as i32
            || text_rect.y + text_rect.height > capture.image.size.height as i32
        {
            return Ok(false);
        }
        let roi = desktop_ocr_roi_for_image(capture.image.size, text_rect)?;
        let cropped = capture.derive_crop(roi).map_err(|error| {
            TaskError::VisionPlan(format!(
                "LowerHeadThenWalkTo F-key text crop failed: {error}"
            ))
        })?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped.image).map_err(|error| {
            TaskError::VisionPlan(format!(
                "LowerHeadThenWalkTo F-key text OCR failed: {error}"
            ))
        })?;
        let text = desktop_lower_head_then_walk_to_ocr_text_from_regions(&regions);
        Ok(normalize_desktop_ocr_text(&text)
            .contains(&normalize_desktop_ocr_text(&f_key_rule.activation_text)))
    }

    fn release_move_forward_best_effort(&mut self) {
        if let Ok(events) = input_events_for_action(
            &self.key_bindings,
            GenshinAction::MoveForward,
            KeyActionType::KeyUp,
        ) {
            let _ = CommonJobRuntime::dispatch_input(&mut self.common, &events);
        }
    }

    fn execute_lower_head_tracking_loop_inner(
        &mut self,
        target_locator: &BvLocatorPlan,
        movement_rule: &LowerHeadThenWalkToMovementRule,
        f_key_rule: &LowerHeadThenWalkToFKeyRule,
    ) -> bgi_task::Result<LowerHeadThenWalkToStepResult> {
        let start = Instant::now();
        let mut previous_move_x = 0;
        loop {
            self.ensure_not_cancelled()?;
            let capture = self.capture_image_region()?;
            let target_rect = self
                .locate_region_on_capture(&capture, target_locator, "target")?
                .map(|region| region.rect);
            let activation_text_detected = if target_rect.is_some() {
                self.activation_text_detected(&capture, f_key_rule)?
            } else {
                false
            };
            let elapsed_ms = start.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
            let decision = reduce_lower_head_then_walk_to_tracking_frame(
                LowerHeadThenWalkToTrackingObservation {
                    capture_size: self.capture_size,
                    target_rect,
                    activation_text_detected,
                    elapsed_ms,
                    previous_move_x,
                    dpi_scale: self.dpi_scale,
                },
                movement_rule,
                &self.key_bindings,
                self.timeout_ms,
            )?;
            if !decision.input_events.is_empty() {
                CommonJobRuntime::dispatch_input(&mut self.common, &decision.input_events)?;
            }
            previous_move_x = decision.next_previous_move_x;
            if let Some(result) = decision.result {
                return Ok(result);
            }
        }
    }
}

impl<F, I, C> CommonJobRuntime for DesktopLowerHeadThenWalkToRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> LowerHeadThenWalkToRuntime for DesktopLowerHeadThenWalkToRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn execute_lower_head_tracking_loop(
        &mut self,
        target_locator: &BvLocatorPlan,
        movement_rule: &LowerHeadThenWalkToMovementRule,
        f_key_rule: &LowerHeadThenWalkToFKeyRule,
    ) -> bgi_task::Result<LowerHeadThenWalkToStepResult> {
        let result =
            self.execute_lower_head_tracking_loop_inner(target_locator, movement_rule, f_key_rule);
        if result.is_err() {
            self.release_move_forward_best_effort();
        }
        result
    }

    fn clear_vision_drawings(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::None)
    }
}

fn desktop_lower_head_then_walk_to_ocr_text_from_regions(regions: &[OcrResultRegion]) -> String {
    let mut regions = regions
        .iter()
        .filter(|region| !region.text.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    regions.sort_by_key(|region| (region.rect.center().y, region.rect.center().x));

    let mut lines: Vec<(i32, i32, Vec<OcrResultRegion>)> = Vec::new();
    for region in regions {
        let center_y = region.rect.center().y;
        let height = region.rect.height.max(1);
        let Some((line_y, line_height, line_regions)) =
            lines.iter_mut().find(|(line_y, line_height, _)| {
                let tolerance = ((*line_height).max(height) / 2).max(4);
                (center_y - *line_y).abs() <= tolerance
            })
        else {
            lines.push((center_y, height, vec![region]));
            continue;
        };
        let count = line_regions.len() as i32;
        *line_y = ((*line_y * count) + center_y) / (count + 1);
        *line_height = (*line_height).max(height);
        line_regions.push(region);
    }

    lines
        .into_iter()
        .map(|(_, _, mut line_regions)| {
            line_regions.sort_by_key(|region| region.rect.center().x);
            line_regions
                .into_iter()
                .map(|region| region.text)
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn execute_desktop_teleport_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &TeleportExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<TeleportExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("Teleport live execution cancelled".to_string());
    }
    let (global_input, capture_size) = desktop_common_job_global_input(config, window, "Teleport")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "Teleport live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "Teleport live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    let common_runtime = PureTemplateCommonJobRuntime::with_task_assets(
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    );
    let tp_json_asset = bgi_task::task_asset_root().join(&plan.map_rule.tp_json_asset);
    let big_map_feature_keypoints_asset =
        bgi_task::task_asset_root().join(&plan.map_rule.feature_keypoints_asset);
    let big_map_feature_mat_asset =
        bgi_task::task_asset_root().join(&plan.map_rule.feature_mat_asset);
    let quick_teleport_asset_root =
        bgi_task::task_asset_root().join(&plan.quick_teleport_rule.asset_root);
    let mut runtime = DesktopTeleportRuntime::new(
        common_runtime,
        DesktopTeleportRuntimeConfig {
            key_bindings_config: config.key_bindings_config.clone(),
            capture_size,
            tp_json_asset,
            big_map_feature_keypoints_asset,
            big_map_feature_mat_asset,
            big_map_layer_256_to_2048_scale: plan.map_rule.layer_256_to_2048_scale,
            move_map_rule: plan.move_map_rule.clone(),
            quick_teleport_asset_root,
        },
    )?;
    execute_teleport_plan(plan, &mut runtime).map_err(|error| error.to_string())
}

struct DesktopTeleportRuntimeConfig {
    key_bindings_config: KeyBindingsConfig,
    capture_size: VisionSize,
    tp_json_asset: PathBuf,
    big_map_feature_keypoints_asset: PathBuf,
    big_map_feature_mat_asset: PathBuf,
    big_map_layer_256_to_2048_scale: u64,
    move_map_rule: TeleportMoveMapRule,
    quick_teleport_asset_root: PathBuf,
}

struct DesktopTeleportRuntime<F, I, C> {
    common: PureTemplateCommonJobRuntime<F, I, C>,
    key_bindings_config: KeyBindingsConfig,
    main_ui_locator: BvLocatorPlan,
    big_map_scale_locator: BvLocatorPlan,
    big_map_settings_locator: BvLocatorPlan,
    teleport_button_locator: BvLocatorPlan,
    teleport_button_detect_locator: BvLocatorPlan,
    teleport_button_once_locator: BvLocatorPlan,
    map_close_locator: BvLocatorPlan,
    underground_switch_locator: BvLocatorPlan,
    underground_to_ground_locator: BvLocatorPlan,
    quick_teleport_plan: QuickTeleportExecutionPlan,
    quick_teleport_vision_backend: PureRustVisionBackend,
    tp_json_asset: PathBuf,
    big_map_feature_keypoints_asset: PathBuf,
    big_map_feature_mat_asset: PathBuf,
    big_map_layer_256_to_2048_scale: u64,
    move_map_rule: TeleportMoveMapRule,
    coordinate_target: Option<TeleportTargetPlan>,
    nearest_teleport_point: Option<DesktopTeleportPoint>,
    big_map_zoom_level: Option<f64>,
    big_map_center: Option<DesktopTeleportMapPoint>,
    big_map_rect: Option<DesktopTeleportMapRect>,
    target_screen_point: Option<DesktopTeleportScreenPoint>,
    teleport_panel_clicked: bool,
    point_not_activated_reason: Option<String>,
}

impl<F, I, C> DesktopTeleportRuntime<F, I, C> {
    fn new(
        common: PureTemplateCommonJobRuntime<F, I, C>,
        config: DesktopTeleportRuntimeConfig,
    ) -> Result<Self, String> {
        let DesktopTeleportRuntimeConfig {
            key_bindings_config,
            capture_size,
            tp_json_asset,
            big_map_feature_keypoints_asset,
            big_map_feature_mat_asset,
            big_map_layer_256_to_2048_scale,
            move_map_rule,
            quick_teleport_asset_root,
        } = config;
        let main_ui_locator = desktop_teleport_main_ui_locator(capture_size)?;
        let big_map_scale_locator = desktop_quick_serenitea_pot_big_map_locator(
            capture_size,
            "TeleportMapScaleButton",
            QUICK_TELEPORT_MAP_SCALE_BUTTON,
            Rect {
                x: desktop_scaled_1080p(30, capture_size),
                y: desktop_scaled_1080p(440, capture_size),
                width: desktop_scaled_1080p(40, capture_size),
                height: desktop_scaled_1080p(200, capture_size),
            },
        )
        .map_err(|error| error.to_string())?;
        let big_map_settings_locator = desktop_quick_serenitea_pot_big_map_locator(
            capture_size,
            "TeleportMapSettingsButton",
            QUICK_TELEPORT_MAP_SETTINGS_BUTTON,
            Rect {
                x: desktop_scaled_1080p(25, capture_size),
                y: desktop_scaled_1080p(990, capture_size),
                width: desktop_scaled_1080p(58, capture_size),
                height: desktop_scaled_1080p(62, capture_size),
            },
        )
        .map_err(|error| error.to_string())?;
        let quick_teleport_plan = plan_quick_teleport(QuickTeleportExecutionConfig {
            capture_size,
            ..QuickTeleportExecutionConfig::default()
        });
        let quick_teleport_vision_backend =
            PureRustVisionBackend::new().with_template_root(&quick_teleport_asset_root);
        let teleport_button_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan.locators.teleport_button,
            capture_size,
            "TeleportPanelButton",
            BvLocatorOperation::Click,
            1_000,
            100,
            10,
        )?;
        let teleport_button_detect_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan.locators.teleport_button,
            capture_size,
            "TeleportPanelButtonDetect",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )?;
        let teleport_button_once_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan.locators.teleport_button,
            capture_size,
            "TeleportPanelButtonOnce",
            BvLocatorOperation::Click,
            100,
            100,
            1,
        )?;
        let map_close_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan.locators.map_close_button,
            capture_size,
            "TeleportMapCloseButton",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )?;
        let underground_switch_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan.locators.map_underground_switch_button,
            capture_size,
            "TeleportUndergroundSwitchButton",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )?;
        let underground_to_ground_locator = desktop_teleport_quick_template_locator_plan(
            &quick_teleport_plan
                .locators
                .map_underground_to_ground_button,
            capture_size,
            "TeleportUndergroundToGroundButton",
            BvLocatorOperation::Click,
            600,
            100,
            6,
        )?;
        Ok(Self {
            common,
            key_bindings_config,
            main_ui_locator,
            big_map_scale_locator,
            big_map_settings_locator,
            teleport_button_locator,
            teleport_button_detect_locator,
            teleport_button_once_locator,
            map_close_locator,
            underground_switch_locator,
            underground_to_ground_locator,
            quick_teleport_plan,
            quick_teleport_vision_backend,
            tp_json_asset,
            big_map_feature_keypoints_asset,
            big_map_feature_mat_asset,
            big_map_layer_256_to_2048_scale,
            move_map_rule,
            coordinate_target: None,
            nearest_teleport_point: None,
            big_map_zoom_level: None,
            big_map_center: None,
            big_map_rect: None,
            target_screen_point: None,
            teleport_panel_clicked: false,
            point_not_activated_reason: None,
        })
    }

    fn locator_matched(&mut self, locator: &BvLocatorPlan) -> bgi_task::Result<bool>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        match CommonJobRuntime::execute_locator(&mut self.common, locator)? {
            CommonJobRuntimeOutcome::Matched(value) => Ok(value),
            CommonJobRuntimeOutcome::None => Ok(false),
        }
    }

    fn detect_big_map_ui(&mut self) -> bgi_task::Result<bool>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let scale = self.big_map_scale_locator.clone();
        if self.locator_matched(&scale)? {
            return Ok(true);
        }
        let settings = self.big_map_settings_locator.clone();
        self.locator_matched(&settings)
    }

    fn capture_image_region(&mut self) -> bgi_task::Result<ImageRegion>
    where
        F: CommonJobFrameSource,
    {
        let image = self.common.frame_source_mut().capture_frame()?;
        Ok(ImageRegion::capture(image))
    }

    fn locate_quick_teleport_template_matches(
        &self,
        capture: &ImageRegion,
        locator: &QuickTeleportTemplateLocator,
    ) -> bgi_task::Result<Vec<Region>> {
        let object =
            desktop_quick_teleport_template_object(locator, self.quick_teleport_plan.capture_size)
                .map_err(|error| {
                    TaskError::VisionPlan(format!(
                "Teleport quick-teleport template object failed under candidate fallback: {error}"
            ))
                })?;
        capture
            .find_multi(&self.quick_teleport_vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "Teleport quick-teleport candidate lookup failed: {error}"
                ))
            })
    }

    fn observe_quick_teleport_candidates(
        &mut self,
    ) -> bgi_task::Result<Vec<QuickTeleportMapChooseCandidate>>
    where
        F: CommonJobFrameSource,
    {
        let capture = self.capture_image_region()?;
        let mut candidates = Vec::new();
        for locator in &self.quick_teleport_plan.locators.map_choose_icon_templates {
            for region in self.locate_quick_teleport_template_matches(&capture, locator)? {
                let icon_rect = desktop_quick_teleport_relative_candidate_icon_rect(
                    region.rect,
                    &self.quick_teleport_plan,
                )?;
                candidates.push(QuickTeleportMapChooseCandidate {
                    icon_rect,
                    text: String::new(),
                });
            }
        }
        let mut candidates = desktop_quick_teleport_deduplicate_candidates(candidates);
        for candidate in &mut candidates {
            candidate.text = desktop_quick_teleport_ocr_candidate_text(
                &capture,
                candidate.icon_rect,
                &self.quick_teleport_plan,
            )?;
        }
        Ok(candidates)
    }

    fn wait_ms(&mut self, milliseconds: u32) -> bgi_task::Result<()>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        CommonJobRuntime::execute_page_command(
            &mut self.common,
            &BvPageCommand::Wait { milliseconds },
        )
        .map(|_| ())
    }

    fn click_capture_point(&mut self, x: i32, y: i32) -> bgi_task::Result<()>
    where
        I: CommonJobInputDriver,
    {
        self.common.input_driver_mut().click_capture_point(x, y)
    }

    fn normalize_underground_map(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let underground_switch = self.underground_switch_locator.clone();
        if !self.locator_matched(&underground_switch)? {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        let underground_to_ground = self.underground_to_ground_locator.clone();
        if !self.locator_matched(&underground_to_ground)? {
            return Err(TaskError::CommonJobExecution(
                "Teleport detected underground big-map state but could not click the to-ground switch"
                    .to_string(),
            ));
        }
        self.wait_ms(200)?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn read_big_map_zoom_level(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
    {
        let capture = self.capture_image_region()?;
        let object = desktop_quick_teleport_template_object(
            &self.quick_teleport_plan.locators.map_scale_button,
            self.quick_teleport_plan.capture_size,
        )
        .map_err(|error| {
            TaskError::VisionPlan(format!(
                "Teleport big-map zoom scale-button object failed: {error}"
            ))
        })?;
        let region = capture
            .find(&self.quick_teleport_vision_backend, &object)
            .map_err(|error| {
                TaskError::VisionPlan(format!(
                    "Teleport big-map zoom scale-button lookup failed: {error}"
                ))
            })?;
        if !region.is_exist() {
            return Err(TaskError::CommonJobExecution(
                "Teleport could not find the big-map zoom scale button".to_string(),
            ));
        }
        self.big_map_zoom_level = Some(desktop_teleport_zoom_level_from_scale_button_region(
            region.rect,
            capture.image.size,
        ));
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn adjust_big_map_zoom_level(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let current_zoom = self.big_map_zoom_level.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot adjust big-map zoom before reading the current zoom level"
                    .to_string(),
            )
        })?;
        let Some(target_zoom) = desktop_teleport_zoom_adjust_target(current_zoom) else {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        };
        let events = desktop_teleport_zoom_drag_events(
            current_zoom,
            target_zoom,
            self.quick_teleport_plan.capture_size,
        );
        CommonJobRuntime::dispatch_capture_input(&mut self.common, &events)?;
        self.wait_ms(100)?;
        self.big_map_zoom_level = Some(target_zoom);
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn verify_target_point_in_big_map_window(
        &self,
        target: &TeleportTargetPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        let rect = self.big_map_rect.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot verify the target point before big-map rect recognition is ported"
                    .to_string(),
            )
        })?;
        let map_name = desktop_teleport_target_map_name(target);
        Ok(CommonJobRuntimeOutcome::Matched(
            desktop_teleport_target_point_in_big_map_window(
                self.quick_teleport_plan.capture_size,
                map_name,
                rect,
                DesktopTeleportMapPoint {
                    x: target.x,
                    y: target.y,
                },
            ),
        ))
    }

    fn convert_map_coordinate_to_screen_point(
        &mut self,
        target: &TeleportTargetPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        let rect = self.big_map_rect.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot convert the target coordinate before big-map rect recognition is ported"
                    .to_string(),
            )
        })?;
        let map_name = desktop_teleport_target_map_name(target);
        let Some(point) = desktop_teleport_target_capture_point(
            self.quick_teleport_plan.capture_size,
            map_name,
            rect,
            DesktopTeleportMapPoint {
                x: target.x,
                y: target.y,
            },
        ) else {
            return Err(TaskError::CommonJobExecution(format!(
                "Teleport has no coordinate conversion rule for map {map_name}"
            )));
        };
        self.target_screen_point = Some(point);
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn click_map_teleport_point(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let point = self.target_screen_point.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot click the target point before map coordinate conversion"
                    .to_string(),
            )
        })?;
        self.click_capture_point(point.x.round() as i32, point.y.round() as i32)?;
        self.wait_ms(500)?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn recognize_big_map_center(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if self.big_map_center.is_some() {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        if let Some(rect) = self.big_map_rect {
            self.big_map_center = Some(rect.center());
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        Err(desktop_teleport_big_map_recognition_preflight_error(
            &self.big_map_feature_keypoints_asset,
            &self.big_map_feature_mat_asset,
            self.big_map_layer_256_to_2048_scale,
        ))
    }

    fn recognize_big_map_rect(&mut self) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        if self.big_map_rect.is_some() {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        Err(desktop_teleport_big_map_recognition_preflight_error(
            &self.big_map_feature_keypoints_asset,
            &self.big_map_feature_mat_asset,
            self.big_map_layer_256_to_2048_scale,
        ))
    }

    fn drag_big_map_to_target(
        &mut self,
        target: &TeleportTargetPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        self.drag_big_map_once_toward_target(DesktopTeleportMapPoint {
            x: target.x,
            y: target.y,
        })?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn drag_big_map_once_toward_target(
        &mut self,
        target: DesktopTeleportMapPoint,
    ) -> bgi_task::Result<Option<DesktopTeleportDragPlan>>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let current_center = self.big_map_center.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot drag the big map before big-map center recognition".to_string(),
            )
        })?;
        let zoom_level = self.big_map_zoom_level.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport cannot drag the big map before reading the zoom level".to_string(),
            )
        })?;
        let Some(drag_plan) = desktop_teleport_drag_plan_with_rule(
            current_center,
            target,
            zoom_level,
            &self.move_map_rule,
        ) else {
            return Ok(None);
        };
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let events = desktop_teleport_mouse_move_map_events_with_interval(
            drag_plan.mouse_move_x,
            drag_plan.mouse_move_y,
            drag_plan.steps,
            self.quick_teleport_plan.capture_size,
            seed,
            self.move_map_rule.step_interval_ms,
        );
        CommonJobRuntime::dispatch_capture_input(&mut self.common, &events)?;
        self.set_big_map_center_adjusting_rect(drag_plan.predicted_center);
        Ok(Some(drag_plan))
    }

    fn set_big_map_center_adjusting_rect(&mut self, center: DesktopTeleportMapPoint) {
        if let Some(previous_center) = self.big_map_center {
            let delta_x = center.x - previous_center.x;
            let delta_y = center.y - previous_center.y;
            if let Some(rect) = &mut self.big_map_rect {
                rect.x += delta_x;
                rect.y += delta_y;
            }
        }
        self.big_map_center = Some(center);
    }

    fn move_map_to_target(
        &mut self,
        target: &TeleportTargetPlan,
        final_zoom_level: f64,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        self.big_map_rect.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport MoveMapTo cannot verify convergence before big-map rect recognition"
                    .to_string(),
            )
        })?;
        let map_name = desktop_teleport_target_map_name(target);
        let target_point = DesktopTeleportMapPoint {
            x: target.x,
            y: target.y,
        };
        let move_map_rule = self.move_map_rule.clone();
        let max_iterations = move_map_rule.max_iterations.max(1);
        let mut exception_times = 0;

        for _ in 0..max_iterations {
            if self.target_point_in_big_map_window(map_name, target_point)? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }

            let current_center = self.big_map_center.ok_or_else(|| {
                TaskError::CommonJobExecution(
                    "Teleport MoveMapTo cannot continue before big-map center recognition"
                        .to_string(),
                )
            })?;
            let current_zoom = self.big_map_zoom_level.ok_or_else(|| {
                TaskError::CommonJobExecution(
                    "Teleport MoveMapTo cannot continue before reading the zoom level".to_string(),
                )
            })?;

            if let Some(drag_plan) = desktop_teleport_drag_plan_with_rule(
                current_center,
                target_point,
                current_zoom,
                &move_map_rule,
            ) {
                if let Some(target_zoom) = desktop_teleport_move_map_zoom_target(
                    current_zoom,
                    drag_plan.mouse_distance,
                    final_zoom_level,
                    &move_map_rule,
                ) {
                    self.adjust_big_map_zoom_to_target(current_zoom, target_zoom)?;
                    continue;
                }
            } else {
                return Ok(CommonJobRuntimeOutcome::Matched(false));
            }

            let Some(drag_plan) = self.drag_big_map_once_toward_target(target_point)? else {
                return Ok(CommonJobRuntimeOutcome::Matched(false));
            };
            let center_decision = decide_teleport_move_map_center_after_drag(
                drag_plan.predicted_center.into(),
                None,
                drag_plan.mouse_move_x,
                drag_plan.mouse_move_y,
                current_zoom,
                exception_times,
                &move_map_rule,
            );
            match center_decision {
                TeleportMoveMapCenterDecision::UseRecognized { center, .. } => {
                    self.set_big_map_center_adjusting_rect(center.into());
                    exception_times = 0;
                }
                TeleportMoveMapCenterDecision::BlindWalk {
                    center,
                    exception_times: next_exception_times,
                    ..
                } => {
                    self.set_big_map_center_adjusting_rect(center.into());
                    exception_times = next_exception_times;
                }
                TeleportMoveMapCenterDecision::AbortReTeleport { .. } => {
                    return Ok(CommonJobRuntimeOutcome::Matched(false));
                }
            }
            if self.target_point_in_big_map_window(map_name, target_point)? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
        }

        Ok(CommonJobRuntimeOutcome::Matched(false))
    }

    fn target_point_in_big_map_window(
        &self,
        map_name: &str,
        target: DesktopTeleportMapPoint,
    ) -> bgi_task::Result<bool> {
        let rect = self.big_map_rect.ok_or_else(|| {
            TaskError::CommonJobExecution(
                "Teleport MoveMapTo cannot verify convergence before big-map rect recognition"
                    .to_string(),
            )
        })?;
        Ok(desktop_teleport_target_point_in_big_map_window(
            self.quick_teleport_plan.capture_size,
            map_name,
            rect,
            target,
        ))
    }

    fn adjust_big_map_zoom_to_target(
        &mut self,
        current_zoom: f64,
        target_zoom: f64,
    ) -> bgi_task::Result<()>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let events = desktop_teleport_zoom_drag_events(
            current_zoom,
            target_zoom,
            self.quick_teleport_plan.capture_size,
        );
        CommonJobRuntime::dispatch_capture_input(&mut self.common, &events)?;
        self.wait_ms(100)?;
        self.big_map_zoom_level = Some(target_zoom);
        Ok(())
    }

    fn switch_country_or_map(
        &mut self,
        target: &TeleportTargetPlan,
        map_name: Option<&str>,
        force_country: Option<&str>,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let Some(area_name) = desktop_teleport_switch_area_name(
            map_name,
            force_country,
            self.nearest_teleport_point.as_ref(),
            Some(DesktopTeleportMapPoint {
                x: target.x,
                y: target.y,
            }),
            self.big_map_center,
            &self.move_map_rule,
        ) else {
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        };
        let (click_x, click_y) =
            desktop_teleport_area_menu_button_point(self.quick_teleport_plan.capture_size);
        self.click_capture_point(click_x, click_y)?;
        self.wait_ms(300)?;

        let candidates = self.recognize_area_menu_candidates()?;
        let matched = candidates
            .into_iter()
            .filter(|candidate| desktop_teleport_area_text_matches(&candidate.text, &area_name))
            .max_by_key(|candidate| candidate.rect.y);
        let Some(matched) = matched else {
            return Err(TaskError::CommonJobExecution(format!(
                "Teleport could not find area {area_name:?} in the big-map area menu"
            )));
        };
        let center = matched.rect.center();
        self.click_capture_point(center.x, center.y)?;
        self.wait_ms(500)?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn recognize_area_menu_candidates(
        &mut self,
    ) -> bgi_task::Result<Vec<DesktopTeleportAreaMenuCandidate>>
    where
        F: CommonJobFrameSource,
    {
        let frame = self.common.frame_source_mut().capture_frame()?;
        let roi = desktop_teleport_area_menu_ocr_roi(frame.size)?;
        let cropped = crop_bgr_image(&frame, roi)
            .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        let regions = desktop_winrt_ocr_bgr_image(&cropped).map_err(|error| {
            TaskError::CommonJobExecution(format!("Teleport area-menu WinRT OCR failed: {error}"))
        })?;
        desktop_teleport_area_menu_candidates_from_ocr_regions(&regions, roi)
    }

    fn dispatch_genshin_action(
        &mut self,
        action: GenshinAction,
        press: KeyActionType,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let events = input_events_for_action(&self.key_bindings_config, action, press)
            .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
        CommonJobRuntime::dispatch_input(&mut self.common, &events)?;
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn wait_for_main_ui(
        &mut self,
        max_attempts: u16,
        delay_ms: u32,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let locator = self.main_ui_locator.clone();
        for attempt in 0..max_attempts.max(1) {
            if self.locator_matched(&locator)? {
                return Ok(CommonJobRuntimeOutcome::Matched(true));
            }
            let teleport_button_locator = self.teleport_button_locator.clone();
            if self.locator_matched(&teleport_button_locator)? {
                self.mark_teleport_panel_clicked();
            }
            if attempt + 1 < max_attempts.max(1) {
                CommonJobRuntime::execute_page_command(
                    &mut self.common,
                    &BvPageCommand::Wait {
                        milliseconds: delay_ms,
                    },
                )?;
            }
        }
        Ok(CommonJobRuntimeOutcome::Matched(false))
    }

    fn click_teleport_panel_or_candidate(
        &mut self,
        allow_candidate_fallback: bool,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        self.teleport_panel_clicked = false;
        self.point_not_activated_reason = None;
        let locator = self.teleport_button_locator.clone();
        if self.locator_matched(&locator)? {
            self.mark_teleport_panel_clicked();
            return Ok(CommonJobRuntimeOutcome::Matched(true));
        }
        let map_close_locator = self.map_close_locator.clone();
        if self.locator_matched(&map_close_locator)? {
            if allow_candidate_fallback {
                return Ok(self.mark_point_not_activated(
                    "Teleport point is not activated or does not exist",
                ));
            }
            return Err(TaskError::CommonJobExecution(
                "Teleport desktop live adapter found the map close button instead of a teleport panel button"
                    .to_string(),
            ));
        }
        if allow_candidate_fallback {
            return self.click_quick_teleport_candidate_fallback();
        }
        Err(TaskError::CommonJobExecution(
            "Teleport desktop live adapter found no direct teleport button".to_string(),
        ))
    }

    fn click_quick_teleport_candidate_fallback(
        &mut self,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let candidates = self.observe_quick_teleport_candidates()?;
        let decision = decide_quick_teleport_tick(
            &self.quick_teleport_plan,
            QuickTeleportDecisionInput {
                elapsed_since_previous_tick_ms: self
                    .quick_teleport_plan
                    .throttle_rule
                    .tick_interval_ms
                    .saturating_add(1),
                hotkey_pressed: true,
                is_big_map_ui: true,
                teleport_button_detected: false,
                map_close_button_detected: false,
                map_choose_button_detected: false,
                map_choose_candidates: candidates,
            },
        );
        match decision.action {
            QuickTeleportDecisionAction::ClickCandidate {
                text,
                text_rect,
                teleport_list_click_delay_ms,
                ..
            } => {
                let click_delay_ms =
                    desktop_teleport_candidate_click_delay_ms(teleport_list_click_delay_ms);
                self.wait_ms(u32::try_from(click_delay_ms).unwrap_or(u32::MAX))?;
                let center = text_rect.center();
                self.click_capture_point(center.x, center.y)?;
                if !self.wait_for_teleport_button_appear(6, 300)? {
                    return Ok(self.mark_point_not_activated(format!(
                        "Teleport candidate fallback clicked candidate text {text:?} but the teleport button did not appear"
                    )));
                }
                self.click_teleport_button_until_disappear(6, 300)?;
                Ok(CommonJobRuntimeOutcome::Matched(true))
            }
            QuickTeleportDecisionAction::ClickTeleportButton => {
                let locator = self.teleport_button_locator.clone();
                let clicked = self.locator_matched(&locator)?;
                if clicked {
                    self.mark_teleport_panel_clicked();
                    Ok(CommonJobRuntimeOutcome::Matched(true))
                } else {
                    Ok(self.mark_point_not_activated(
                        "Teleport candidate fallback selected direct button action but no teleport button was clicked",
                    ))
                }
            }
            QuickTeleportDecisionAction::Skip { reason } => {
                Ok(self.mark_point_not_activated(format!(
                    "Teleport candidate fallback found no clickable candidate: {reason:?}; candidates={}",
                    decision.sorted_candidates.len()
                )))
            }
        }
    }

    fn wait_for_teleport_button_appear(
        &mut self,
        max_attempts: u16,
        interval_ms: u32,
    ) -> bgi_task::Result<bool>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let locator = self.teleport_button_detect_locator.clone();
        for attempt in 0..max_attempts.max(1) {
            if self.locator_matched(&locator)? {
                return Ok(true);
            }
            if attempt + 1 < max_attempts.max(1) {
                self.wait_ms(interval_ms)?;
            }
        }
        Ok(false)
    }

    fn click_teleport_button_until_disappear(
        &mut self,
        max_attempts: u16,
        interval_ms: u32,
    ) -> bgi_task::Result<()>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let locator = self.teleport_button_once_locator.clone();
        for attempt in 0..max_attempts.max(1) {
            if !self.locator_matched(&locator)? {
                return Ok(());
            }
            self.mark_teleport_panel_clicked();
            if attempt + 1 < max_attempts.max(1) {
                self.wait_ms(interval_ms)?;
            }
        }
        Ok(())
    }

    fn mark_teleport_panel_clicked(&mut self) {
        self.teleport_panel_clicked = true;
        self.point_not_activated_reason = None;
    }

    fn mark_point_not_activated(&mut self, reason: impl Into<String>) -> CommonJobRuntimeOutcome {
        self.point_not_activated_reason = Some(reason.into());
        CommonJobRuntimeOutcome::Matched(false)
    }

    fn handle_point_not_activated(
        &mut self,
        failure_policy: &TeleportFailurePolicy,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome>
    where
        F: CommonJobFrameSource,
        I: CommonJobInputDriver,
        C: CommonJobClock,
    {
        let should_reset_big_map =
            !self.teleport_panel_clicked && self.point_not_activated_reason.is_some();
        let outcome = desktop_teleport_point_not_activated_outcome(
            self.teleport_panel_clicked,
            self.point_not_activated_reason.as_deref(),
            failure_policy,
        )?;
        if should_reset_big_map {
            let events = input_events_for_key(KeyId::ESCAPE, KeyActionType::KeyPress)
                .map_err(|error| TaskError::CommonJobExecution(error.to_string()))?;
            CommonJobRuntime::dispatch_input(&mut self.common, &events)?;
            self.wait_ms(300)?;
        }
        Ok(outcome)
    }

    fn seed_navigation_previous_position_after_teleport(
        &self,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        Ok(CommonJobRuntimeOutcome::Matched(
            self.teleport_panel_clicked,
        ))
    }

    fn resolve_coordinate_target(
        &mut self,
        target: &TeleportTargetPlan,
        _force: bool,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        self.coordinate_target = Some(target.clone());
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn resolve_nearest_teleport_point(
        &mut self,
        target: &TeleportTargetPlan,
        force: bool,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        let json = fs::read_to_string(&self.tp_json_asset).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "Teleport failed to read tp.json asset {}: {error}",
                self.tp_json_asset.display()
            ))
        })?;
        let map_name = desktop_teleport_target_map_name(target);
        let points = desktop_teleport_points_from_json(&json, map_name).map_err(|error| {
            TaskError::CommonJobExecution(format!(
                "Teleport failed to parse tp.json asset {}: {error}",
                self.tp_json_asset.display()
            ))
        })?;
        let nearest = desktop_teleport_nearest_point(&points, target).ok_or_else(|| {
            TaskError::CommonJobExecution(format!(
                "Teleport found no teleport points in map {map_name}"
            ))
        })?;
        if !force {
            self.coordinate_target = Some(TeleportTargetPlan {
                x: nearest.x,
                y: nearest.y,
                map_name: target.map_name.clone(),
            });
        }
        self.nearest_teleport_point = Some(nearest);
        Ok(CommonJobRuntimeOutcome::Matched(true))
    }

    fn resolution_summary(&self) -> String {
        let target = self
            .coordinate_target
            .as_ref()
            .map(|target| {
                format!(
                    "resolved target=({:.3},{:.3}, map={})",
                    target.x,
                    target.y,
                    desktop_teleport_target_map_name(target)
                )
            })
            .unwrap_or_else(|| "resolved target=<none>".to_string());
        let nearest = self
            .nearest_teleport_point
            .as_ref()
            .map(|point| {
                format!(
                    "nearest point=({:.3},{:.3}, country={}, area={})",
                    point.x,
                    point.y,
                    point.country.as_deref().unwrap_or("<none>"),
                    point.level1_area.as_deref().unwrap_or("<none>")
                )
            })
            .unwrap_or_else(|| "nearest point=<none>".to_string());
        format!("{target}; {nearest}")
    }
}

impl<F, I, C> CommonJobRuntime for DesktopTeleportRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn log(&mut self, message: &str) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::log(&mut self.common, message)
    }

    fn dispatch_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_input(&mut self.common, events)
    }

    fn dispatch_capture_input(
        &mut self,
        events: &[bgi_input::InputEvent],
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::dispatch_capture_input(&mut self.common, events)
    }

    fn execute_page_command(
        &mut self,
        command: &BvPageCommand,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_page_command(&mut self.common, command)
    }

    fn execute_locator(
        &mut self,
        locator: &BvLocatorPlan,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        CommonJobRuntime::execute_locator(&mut self.common, locator)
    }
}

impl<F, I, C> TeleportRuntime for DesktopTeleportRuntime<F, I, C>
where
    F: CommonJobFrameSource,
    I: CommonJobInputDriver,
    C: CommonJobClock,
{
    fn execute_teleport_action(
        &mut self,
        action: &TeleportStepAction,
    ) -> bgi_task::Result<CommonJobRuntimeOutcome> {
        match action {
            TeleportStepAction::OpenBigMapUi => {
                self.dispatch_genshin_action(GenshinAction::OpenMap, KeyActionType::KeyPress)
            }
            TeleportStepAction::VerifyBigMapUi => {
                Ok(CommonJobRuntimeOutcome::Matched(self.detect_big_map_ui()?))
            }
            TeleportStepAction::ResolveCoordinateTarget { target, force } => {
                self.resolve_coordinate_target(target, *force)
            }
            TeleportStepAction::ResolveNearestTeleportPoint { target, force } => {
                self.resolve_nearest_teleport_point(target, *force)
            }
            TeleportStepAction::SwitchCountryOrMap {
                target,
                map_name,
                force_country,
            } => self.switch_country_or_map(target, map_name.as_deref(), force_country.as_deref()),
            TeleportStepAction::NormalizeUndergroundMap => self.normalize_underground_map(),
            TeleportStepAction::ClickTeleportPanelOrCandidate {
                allow_candidate_fallback,
            } => self.click_teleport_panel_or_candidate(*allow_candidate_fallback),
            TeleportStepAction::WaitForTeleportCompletion {
                max_attempts,
                delay_ms,
                ..
            } => self.wait_for_main_ui(*max_attempts, *delay_ms),
            TeleportStepAction::SeedNavigationPreviousPositionAfterTeleport { .. } => {
                self.seed_navigation_previous_position_after_teleport()
            }
            TeleportStepAction::HandlePointNotActivated { failure_policy } => {
                self.handle_point_not_activated(failure_policy)
            }
            TeleportStepAction::ReadBigMapZoomLevel => self.read_big_map_zoom_level(),
            TeleportStepAction::AdjustMapZoomLevel => self.adjust_big_map_zoom_level(),
            TeleportStepAction::VerifyTargetPointInBigMapWindow { target } => {
                self.verify_target_point_in_big_map_window(target)
            }
            TeleportStepAction::ConvertMapCoordinateToScreenPoint { target } => {
                self.convert_map_coordinate_to_screen_point(target)
            }
            TeleportStepAction::ClickMapTeleportPoint => self.click_map_teleport_point(),
            TeleportStepAction::RecognizeBigMapCenter => self.recognize_big_map_center(),
            TeleportStepAction::RecognizeBigMapRect => self.recognize_big_map_rect(),
            TeleportStepAction::DragBigMapToTarget { target } => {
                self.drag_big_map_to_target(target)
            }
            TeleportStepAction::MoveMapTo {
                target,
                final_zoom_level,
                ..
            } => self.move_map_to_target(target, *final_zoom_level),
            TeleportStepAction::SelectStatueOfTheSeven => {
                Err(TaskError::CommonJobExecution(format!(
                    "Teleport desktop live adapter has not ported native action {action:?}; {}",
                    self.resolution_summary()
                )))
            }
            TeleportStepAction::ReturnResult { .. } | TeleportStepAction::Log { .. } => {
                Ok(CommonJobRuntimeOutcome::None)
            }
        }
    }

    fn teleport_navigation_seed_target(&self) -> Option<TeleportTargetPlan> {
        self.coordinate_target.clone()
    }
}

fn desktop_teleport_main_ui_locator(capture_size: VisionSize) -> Result<BvLocatorPlan, String> {
    let return_main_ui_plan =
        bgi_task::plan_return_main_ui(capture_size, 1).map_err(|error| error.to_string())?;
    return_main_ui_plan
        .steps
        .into_iter()
        .find_map(|step| match step.action {
            CommonJobStepAction::Locator { locator }
                if step.label.contains("already in main UI") =>
            {
                Some(locator)
            }
            _ => None,
        })
        .ok_or_else(|| "Teleport live execution could not build main-UI locator".to_string())
}

fn desktop_teleport_quick_template_locator_plan(
    locator: &QuickTeleportTemplateLocator,
    capture_size: VisionSize,
    name: &str,
    operation: BvLocatorOperation,
    timeout_ms: u32,
    retry_interval_ms: u32,
    retry_count: u32,
) -> Result<BvLocatorPlan, String> {
    let mut object = desktop_quick_teleport_template_object(locator, capture_size)
        .map_err(|error| error.to_string())?;
    object.name = Some(name.to_string());
    Ok(BvLocatorPlan {
        operation,
        recognition_object: object,
        timeout_ms,
        retry_interval_ms,
        retry_count: retry_count.max(1),
        retry_action: None,
    })
}

fn desktop_teleport_point_not_activated_outcome(
    teleport_panel_clicked: bool,
    point_not_activated_reason: Option<&str>,
    failure_policy: &TeleportFailurePolicy,
) -> bgi_task::Result<CommonJobRuntimeOutcome> {
    if teleport_panel_clicked {
        return Ok(CommonJobRuntimeOutcome::Matched(true));
    }
    let Some(reason) = point_not_activated_reason else {
        return Err(TaskError::CommonJobExecution(
            "Teleport point-not-activated handler ran before any teleport panel click or point-not-activated observation"
                .to_string(),
        ));
    };
    match failure_policy {
        TeleportFailurePolicy::WarningOnly
        | TeleportFailurePolicy::ContinueAfterPointNotActivated => {
            Ok(CommonJobRuntimeOutcome::Matched(true))
        }
        TeleportFailurePolicy::HardError => Err(TaskError::CommonJobExecution(format!(
            "Teleport point is not activated: {reason}"
        ))),
    }
}

fn desktop_teleport_candidate_click_delay_ms(configured_delay_ms: u64) -> u64 {
    configured_delay_ms.max(500)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DesktopTeleportBigMapRecognitionPreflight {
    InvalidScale { layer_256_to_2048_scale: u64 },
    MissingAssets { assets: Vec<PathBuf> },
    AdapterUnavailable,
}

fn desktop_teleport_big_map_recognition_preflight(
    feature_keypoints_asset: &Path,
    feature_mat_asset: &Path,
    layer_256_to_2048_scale: u64,
) -> DesktopTeleportBigMapRecognitionPreflight {
    if layer_256_to_2048_scale == 0 {
        return DesktopTeleportBigMapRecognitionPreflight::InvalidScale {
            layer_256_to_2048_scale,
        };
    }

    let missing_assets = [feature_keypoints_asset, feature_mat_asset]
        .into_iter()
        .filter(|asset| !asset.exists())
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();

    if !missing_assets.is_empty() {
        return DesktopTeleportBigMapRecognitionPreflight::MissingAssets {
            assets: missing_assets,
        };
    }

    DesktopTeleportBigMapRecognitionPreflight::AdapterUnavailable
}

fn desktop_teleport_big_map_recognition_preflight_error(
    feature_keypoints_asset: &Path,
    feature_mat_asset: &Path,
    layer_256_to_2048_scale: u64,
) -> TaskError {
    match desktop_teleport_big_map_recognition_preflight(
        feature_keypoints_asset,
        feature_mat_asset,
        layer_256_to_2048_scale,
    ) {
        DesktopTeleportBigMapRecognitionPreflight::InvalidScale {
            layer_256_to_2048_scale,
        } => TaskError::CommonJobExecution(format!(
            "Teleport BigMap SIFT recognition has invalid 256-to-2048 layer scale {layer_256_to_2048_scale}"
        )),
        DesktopTeleportBigMapRecognitionPreflight::MissingAssets { assets } => {
            let missing = assets
                .iter()
                .map(|asset| asset.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            TaskError::CommonJobExecution(format!(
                "Teleport BigMap SIFT assets are missing: {missing}"
            ))
        }
        DesktopTeleportBigMapRecognitionPreflight::AdapterUnavailable => {
            TaskError::CommonJobExecution(format!(
                "Teleport BigMap SIFT assets exist ({}, {}) with 256-to-2048 layer scale {}, but the native OpenCV/SIFT map-matching adapter is not ported",
                feature_keypoints_asset.display(),
                feature_mat_asset.display(),
                layer_256_to_2048_scale
            ))
        }
    }
}

const DESKTOP_TELEPORT_ZOOM_START_Y_1080P: f64 = 468.0;
const DESKTOP_TELEPORT_ZOOM_END_Y_1080P: f64 = 612.0;
const DESKTOP_TELEPORT_ZOOM_BUTTON_X_1080P: i32 = 47;
const DESKTOP_TELEPORT_DISPLAY_TP_POINT_ZOOM_LEVEL: f64 = 4.4;
const DESKTOP_TELEPORT_MIN_ZOOM_LEVEL: f64 = 2.0;
const DESKTOP_TELEPORT_ZOOM_PRECISION_THRESHOLD: f64 = 0.05;
#[cfg(test)]
const DESKTOP_TELEPORT_MAP_MOVE_STEP_INTERVAL_MS: u64 = 20;

fn desktop_teleport_zoom_level_from_scale_button_region(
    region: Rect,
    capture_size: VisionSize,
) -> f64 {
    let center_y = region.y as f64 + region.height as f64 / 2.0;
    let center_y_1080p = center_y * 1920.0 / capture_size.width as f64;
    let scale = (DESKTOP_TELEPORT_ZOOM_END_Y_1080P - center_y_1080p)
        / (DESKTOP_TELEPORT_ZOOM_END_Y_1080P - DESKTOP_TELEPORT_ZOOM_START_Y_1080P);
    (-5.0 * scale) + 6.0
}

fn desktop_teleport_zoom_adjust_target(current_zoom: f64) -> Option<f64> {
    if current_zoom
        > DESKTOP_TELEPORT_DISPLAY_TP_POINT_ZOOM_LEVEL + DESKTOP_TELEPORT_ZOOM_PRECISION_THRESHOLD
    {
        Some(DESKTOP_TELEPORT_DISPLAY_TP_POINT_ZOOM_LEVEL)
    } else if current_zoom
        < DESKTOP_TELEPORT_MIN_ZOOM_LEVEL - DESKTOP_TELEPORT_ZOOM_PRECISION_THRESHOLD
    {
        Some(DESKTOP_TELEPORT_MIN_ZOOM_LEVEL)
    } else {
        None
    }
}

fn desktop_teleport_zoom_button_capture_point(
    zoom_level: f64,
    capture_size: VisionSize,
) -> (i32, i32) {
    let y_1080p = DESKTOP_TELEPORT_ZOOM_START_Y_1080P
        + (DESKTOP_TELEPORT_ZOOM_END_Y_1080P - DESKTOP_TELEPORT_ZOOM_START_Y_1080P)
            * (zoom_level - 1.0)
            / 5.0;
    (
        desktop_scaled_1080p(DESKTOP_TELEPORT_ZOOM_BUTTON_X_1080P, capture_size),
        desktop_scaled_1080p(y_1080p.round() as i32, capture_size),
    )
}

fn desktop_teleport_zoom_drag_events(
    current_zoom: f64,
    target_zoom: f64,
    capture_size: VisionSize,
) -> Vec<bgi_input::InputEvent> {
    let (start_x, start_y) = desktop_teleport_zoom_button_capture_point(current_zoom, capture_size);
    let (target_x, target_y) =
        desktop_teleport_zoom_button_capture_point(target_zoom, capture_size);
    vec![
        bgi_input::InputEvent::MouseMoveAbsolute {
            x: start_x,
            y: start_y,
            virtual_desktop: false,
        },
        bgi_input::InputEvent::Delay { milliseconds: 50 },
        bgi_input::InputEvent::MouseButtonDown {
            button: MouseButton::Left,
        },
        bgi_input::InputEvent::Delay { milliseconds: 50 },
        bgi_input::InputEvent::MouseMoveAbsolute {
            x: target_x,
            y: target_y,
            virtual_desktop: false,
        },
        bgi_input::InputEvent::Delay { milliseconds: 50 },
        bgi_input::InputEvent::MouseButtonUp {
            button: MouseButton::Left,
        },
        bgi_input::InputEvent::Delay { milliseconds: 50 },
        bgi_input::InputEvent::MouseMoveAbsolute {
            x: capture_size.width as i32 / 2,
            y: capture_size.width as i32 / 2,
            virtual_desktop: false,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopTeleportDragPlan {
    mouse_move_x: i32,
    mouse_move_y: i32,
    steps: i32,
    mouse_distance: f64,
    predicted_center: DesktopTeleportMapPoint,
}

#[cfg(test)]
fn desktop_teleport_drag_plan(
    current_center: DesktopTeleportMapPoint,
    target: DesktopTeleportMapPoint,
    zoom_level: f64,
) -> Option<DesktopTeleportDragPlan> {
    let rule = bgi_task::default_teleport_move_map_rule();
    desktop_teleport_drag_plan_with_rule(current_center, target, zoom_level, &rule)
}

fn desktop_teleport_drag_plan_with_rule(
    current_center: DesktopTeleportMapPoint,
    target: DesktopTeleportMapPoint,
    zoom_level: f64,
    rule: &TeleportMoveMapRule,
) -> Option<DesktopTeleportDragPlan> {
    if zoom_level <= 0.0 || rule.map_scale_factor <= 0.0 || rule.max_mouse_move <= 0.0 {
        return None;
    }
    let x_offset = target.x - current_center.x;
    let y_offset = target.y - current_center.y;
    let total_move_mouse_x = rule.map_scale_factor * x_offset.abs() / zoom_level;
    let total_move_mouse_y = rule.map_scale_factor * y_offset.abs() / zoom_level;
    let mouse_distance =
        (total_move_mouse_x * total_move_mouse_x + total_move_mouse_y * total_move_mouse_y).sqrt();
    if mouse_distance < rule.move_tolerance || mouse_distance <= f64::EPSILON {
        return None;
    }

    let sign_x = desktop_teleport_sign_i32(x_offset);
    let sign_y = desktop_teleport_sign_i32(y_offset);
    let mouse_move_x =
        (total_move_mouse_x.min(rule.max_mouse_move * total_move_mouse_x / mouse_distance) as i32)
            * sign_x;
    let mouse_move_y =
        (total_move_mouse_y.min(rule.max_mouse_move * total_move_mouse_y / mouse_distance) as i32)
            * sign_y;
    let move_mouse_length =
        ((mouse_move_x * mouse_move_x + mouse_move_y * mouse_move_y) as f64).sqrt();
    let steps = ((move_mouse_length as i32) / 10).max(3);
    Some(DesktopTeleportDragPlan {
        mouse_move_x,
        mouse_move_y,
        steps,
        mouse_distance,
        predicted_center: DesktopTeleportMapPoint {
            x: current_center.x + f64::from(mouse_move_x) * zoom_level / rule.map_scale_factor,
            y: current_center.y + f64::from(mouse_move_y) * zoom_level / rule.map_scale_factor,
        },
    })
}

fn desktop_teleport_move_map_zoom_target(
    current_zoom: f64,
    mouse_distance: f64,
    final_zoom_level: f64,
    rule: &TeleportMoveMapRule,
) -> Option<f64> {
    if !rule.map_zoom_enabled || current_zoom <= 0.0 || mouse_distance <= 0.0 {
        return None;
    }

    let target_zoom = if mouse_distance > rule.map_zoom_out_distance {
        (current_zoom * mouse_distance / rule.map_zoom_out_distance).min(rule.max_zoom_level)
    } else if mouse_distance < rule.map_zoom_in_distance {
        let min_zoom_level = final_zoom_level.min(rule.min_zoom_level);
        if current_zoom <= min_zoom_level + rule.zoom_precision_threshold {
            return None;
        }
        (current_zoom * mouse_distance / rule.map_zoom_in_distance).max(min_zoom_level)
    } else {
        return None;
    };

    if (target_zoom - current_zoom).abs() > rule.zoom_precision_threshold {
        Some(target_zoom)
    } else {
        None
    }
}

fn desktop_teleport_sign_i32(value: f64) -> i32 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn desktop_teleport_generate_drag_steps(delta: i32, steps: i32) -> Vec<i32> {
    let steps = steps.max(1) as usize;
    let mut factors = Vec::with_capacity(steps);
    let mut sum = 0.0;
    for index in 0..steps {
        let factor = ((index as f64) * std::f64::consts::PI / (2.0 * steps as f64)).cos();
        factors.push(factor);
        sum += factor;
    }

    let mut result = Vec::with_capacity(steps);
    let mut remaining = delta;
    for factor in factors {
        let value = (delta as f64 * factor / sum) as i32;
        result.push(value);
        remaining -= value;
    }

    let center = steps / 2;
    for offset in 0..remaining.unsigned_abs() as usize {
        let target = (center + offset) % steps;
        result[target] += if remaining > 0 { 1 } else { -1 };
    }
    result
}

fn desktop_teleport_drag_start_point(capture_size: VisionSize, seed: u64) -> (i32, i32) {
    let range = (capture_size.width as i32 / 6).max(1);
    let span = u64::try_from(range * 2 + 1).unwrap_or(1);
    let offset_x = i32::try_from(seed % span).unwrap_or(0) - range;
    let offset_y = i32::try_from(seed.rotate_left(17) % span).unwrap_or(0) - range;
    (
        capture_size.width as i32 / 2 + offset_x,
        capture_size.height as i32 / 2 + offset_y,
    )
}

#[cfg(test)]
fn desktop_teleport_mouse_move_map_events(
    pixel_delta_x: i32,
    pixel_delta_y: i32,
    steps: i32,
    capture_size: VisionSize,
    seed: u64,
) -> Vec<bgi_input::InputEvent> {
    desktop_teleport_mouse_move_map_events_with_interval(
        pixel_delta_x,
        pixel_delta_y,
        steps,
        capture_size,
        seed,
        DESKTOP_TELEPORT_MAP_MOVE_STEP_INTERVAL_MS,
    )
}

fn desktop_teleport_mouse_move_map_events_with_interval(
    pixel_delta_x: i32,
    pixel_delta_y: i32,
    steps: i32,
    capture_size: VisionSize,
    seed: u64,
    step_interval_ms: u64,
) -> Vec<bgi_input::InputEvent> {
    let step_x = desktop_teleport_generate_drag_steps(pixel_delta_x, steps);
    let step_y = desktop_teleport_generate_drag_steps(pixel_delta_y, steps);
    let (start_x, start_y) = desktop_teleport_drag_start_point(capture_size, seed);
    let mut events = Vec::with_capacity(step_x.len() * 2 + 3);
    events.push(bgi_input::InputEvent::MouseMoveAbsolute {
        x: start_x,
        y: start_y,
        virtual_desktop: false,
    });
    events.push(bgi_input::InputEvent::MouseButtonDown {
        button: MouseButton::Left,
    });
    let mut current_x = start_x;
    let mut current_y = start_y;
    for (dx, dy) in step_x.into_iter().zip(step_y) {
        current_x += dx;
        current_y += dy;
        events.push(bgi_input::InputEvent::Delay {
            milliseconds: step_interval_ms,
        });
        events.push(bgi_input::InputEvent::MouseMoveAbsolute {
            x: current_x,
            y: current_y,
            virtual_desktop: false,
        });
    }
    events.push(bgi_input::InputEvent::MouseButtonUp {
        button: MouseButton::Left,
    });
    events
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopTeleportMapPoint {
    x: f64,
    y: f64,
}

impl From<DesktopTeleportMapPoint> for TeleportMapPoint {
    fn from(value: DesktopTeleportMapPoint) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<TeleportMapPoint> for DesktopTeleportMapPoint {
    fn from(value: TeleportMapPoint) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopTeleportMapRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl DesktopTeleportMapRect {
    fn center(self) -> DesktopTeleportMapPoint {
        DesktopTeleportMapPoint {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    fn contains(self, point: DesktopTeleportMapPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopTeleportScreenPoint {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopTeleportMapCoordinateRule {
    origin_x: f64,
    origin_y: f64,
    block_width_scale: f64,
}

fn desktop_teleport_map_coordinate_rule(
    map_name: &str,
) -> Option<DesktopTeleportMapCoordinateRule> {
    match map_name.trim() {
        name if name.eq_ignore_ascii_case("Teyvat") => Some(DesktopTeleportMapCoordinateRule {
            origin_x: 32768.0,
            origin_y: 16384.0,
            block_width_scale: 2.0,
        }),
        _ => None,
    }
}

fn desktop_teleport_genshin_point_to_image_point(
    map_name: &str,
    point: DesktopTeleportMapPoint,
) -> Option<DesktopTeleportMapPoint> {
    let rule = desktop_teleport_map_coordinate_rule(map_name)?;
    Some(DesktopTeleportMapPoint {
        x: rule.origin_x - point.x * rule.block_width_scale,
        y: rule.origin_y - point.y * rule.block_width_scale,
    })
}

fn desktop_teleport_genshin_rect_to_image_rect(
    map_name: &str,
    rect: DesktopTeleportMapRect,
) -> Option<DesktopTeleportMapRect> {
    let rule = desktop_teleport_map_coordinate_rule(map_name)?;
    let center = desktop_teleport_genshin_point_to_image_point(map_name, rect.center())?;
    Some(DesktopTeleportMapRect {
        x: desktop_teleport_midpoint_even_round(
            center.x - rect.width / 2.0 * rule.block_width_scale,
        ),
        y: desktop_teleport_midpoint_even_round(
            center.y - rect.height / 2.0 * rule.block_width_scale,
        ),
        width: desktop_teleport_midpoint_even_round(rect.width * rule.block_width_scale),
        height: desktop_teleport_midpoint_even_round(rect.height * rule.block_width_scale),
    })
}

fn desktop_teleport_midpoint_even_round(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let floor = value.floor();
    let fraction = value - floor;
    if (fraction - 0.5).abs() < f64::EPSILON {
        if (floor as i64).rem_euclid(2) == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        value.round()
    }
}

fn desktop_teleport_target_capture_point(
    capture_size: VisionSize,
    map_name: &str,
    big_map_rect: DesktopTeleportMapRect,
    target: DesktopTeleportMapPoint,
) -> Option<DesktopTeleportScreenPoint> {
    let target_image = desktop_teleport_genshin_point_to_image_point(map_name, target)?;
    let rect_image = desktop_teleport_genshin_rect_to_image_rect(map_name, big_map_rect)?;
    if rect_image.width <= 0.0 || rect_image.height <= 0.0 {
        return None;
    }
    Some(DesktopTeleportScreenPoint {
        x: (target_image.x - rect_image.x) / rect_image.width * capture_size.width as f64,
        y: (target_image.y - rect_image.y) / rect_image.height * capture_size.height as f64,
    })
}

fn desktop_teleport_target_point_in_big_map_window(
    capture_size: VisionSize,
    map_name: &str,
    big_map_rect: DesktopTeleportMapRect,
    target: DesktopTeleportMapPoint,
) -> bool {
    if !big_map_rect.contains(target) {
        return false;
    }
    let Some(click_point) =
        desktop_teleport_target_capture_point(capture_size, map_name, big_map_rect, target)
    else {
        return false;
    };
    if click_point.x < f64::from(desktop_scaled_1080p(360, capture_size))
        && click_point.y < f64::from(desktop_scaled_1080p(400, capture_size))
    {
        return false;
    }
    let margin = f64::from(desktop_scaled_1080p(115, capture_size));
    click_point.x >= margin
        && click_point.y >= margin
        && click_point.x <= capture_size.width as f64 - margin
        && click_point.y <= capture_size.height as f64 - margin
}

#[derive(Debug, Clone, PartialEq)]
struct DesktopTeleportPoint {
    x: f64,
    y: f64,
    country: Option<String>,
    level1_area: Option<String>,
    tran_x: Option<f64>,
    tran_y: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DesktopTeleportAreaMenuCandidate {
    text: String,
    rect: Rect,
}

fn desktop_teleport_switch_area_name(
    map_name: Option<&str>,
    force_country: Option<&str>,
    nearest_point: Option<&DesktopTeleportPoint>,
    target: Option<DesktopTeleportMapPoint>,
    current_center: Option<DesktopTeleportMapPoint>,
    move_map_rule: &TeleportMoveMapRule,
) -> Option<String> {
    force_country
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            let map_name = map_name
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("Teyvat");
            if map_name.eq_ignore_ascii_case("Teyvat") {
                nearest_point
                    .and_then(|point| point.country.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        target.and_then(|target| {
                            desktop_teleport_nearest_country_for_target(
                                target,
                                current_center,
                                move_map_rule,
                            )
                        })
                    })
            } else {
                desktop_teleport_map_description(map_name).map(ToOwned::to_owned)
            }
        })
}

fn desktop_teleport_nearest_country_for_target(
    target: DesktopTeleportMapPoint,
    current_center: Option<DesktopTeleportMapPoint>,
    move_map_rule: &TeleportMoveMapRule,
) -> Option<String> {
    let mut min_distance = f64::MAX;
    if let Some(current_center) = current_center {
        min_distance = desktop_teleport_map_point_distance(current_center, target);
        if min_distance < move_map_rule.target_near_current_center_skip_distance {
            return None;
        }
    }

    let mut nearest_country = None;
    for country in &move_map_rule.country_positions {
        let country_point = DesktopTeleportMapPoint {
            x: country.x,
            y: country.y,
        };
        let distance = desktop_teleport_map_point_distance(country_point, target);
        if distance < min_distance {
            min_distance = distance;
            nearest_country = Some(country.name.clone());
        }
    }
    nearest_country
}

fn desktop_teleport_map_point_distance(
    left: DesktopTeleportMapPoint,
    right: DesktopTeleportMapPoint,
) -> f64 {
    ((left.x - right.x).powi(2) + (left.y - right.y).powi(2)).sqrt()
}

fn desktop_teleport_map_description(map_name: &str) -> Option<&'static str> {
    match map_name.trim() {
        name if name.eq_ignore_ascii_case("Teyvat") => Some("提瓦特大陆"),
        name if name.eq_ignore_ascii_case("TheChasm") => Some("层岩巨渊"),
        name if name.eq_ignore_ascii_case("Enkanomiya") => Some("渊下宫"),
        name if name.eq_ignore_ascii_case("SeaOfBygoneEras") => Some("旧日之海"),
        name if name.eq_ignore_ascii_case("AncientSacredMountain") => Some("远古圣山"),
        name if name.eq_ignore_ascii_case("TempleOfSpace") => Some("空之神殿"),
        _ => None,
    }
}

fn desktop_teleport_area_menu_button_point(capture_size: VisionSize) -> (i32, i32) {
    (
        capture_size.width as i32 - desktop_scaled_1080p(160, capture_size),
        capture_size.height as i32 - desktop_scaled_1080p(60, capture_size),
    )
}

fn desktop_teleport_area_menu_ocr_roi(capture_size: VisionSize) -> bgi_task::Result<Rect> {
    Rect::new(
        (capture_size.width as i32 * 2) / 3,
        0,
        capture_size.width as i32 / 3,
        capture_size.height as i32,
    )
    .map_err(|error| TaskError::VisionPlan(error.to_string()))
}

fn desktop_teleport_area_menu_candidates_from_ocr_regions(
    regions: &[OcrResultRegion],
    source_roi: Rect,
) -> bgi_task::Result<Vec<DesktopTeleportAreaMenuCandidate>> {
    let mut candidates = Vec::new();
    for region in regions {
        let text = region.text.trim();
        if text.is_empty() {
            continue;
        }
        let rect = Rect::new(
            source_roi.x + region.rect.x,
            source_roi.y + region.rect.y,
            region.rect.width,
            region.rect.height,
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
        candidates.push(DesktopTeleportAreaMenuCandidate {
            text: text.to_string(),
            rect,
        });
    }
    Ok(candidates)
}

fn desktop_teleport_area_text_matches(text: &str, area_name: &str) -> bool {
    let text = normalize_desktop_ocr_text(text);
    let area_name = area_name.trim();
    if area_name.is_empty() {
        return false;
    }
    if text.contains(&normalize_desktop_ocr_text(area_name)) {
        return true;
    }
    desktop_teleport_area_aliases(area_name)
        .iter()
        .any(|alias| text.contains(&normalize_desktop_ocr_text(alias)))
}

fn desktop_teleport_area_aliases(area_name: &str) -> &'static [&'static str] {
    match area_name {
        "渊下宫" => &["渊下宮"],
        _ => &[],
    }
}

fn desktop_teleport_target_map_name(target: &TeleportTargetPlan) -> &str {
    target
        .map_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Teyvat")
}

fn desktop_teleport_points_from_json(
    json: &str,
    map_name: &str,
) -> Result<Vec<DesktopTeleportPoint>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "tp.json missing data array".to_string())?;
    let scene = data
        .iter()
        .find(|scene| {
            scene
                .get("mapName")
                .and_then(Value::as_str)
                .is_some_and(|name| name.eq_ignore_ascii_case(map_name))
        })
        .ok_or_else(|| format!("tp.json missing mapName {map_name}"))?;
    let points = scene
        .get("points")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("tp.json scene {map_name} missing points array"))?;
    Ok(points
        .iter()
        .filter_map(desktop_teleport_point_from_json)
        .collect())
}

fn desktop_teleport_point_from_json(value: &Value) -> Option<DesktopTeleportPoint> {
    let position = value.get("position")?.as_array()?;
    let tran_position = value.get("tranPosition").and_then(Value::as_array);
    let y = position.first()?.as_f64()?;
    let x = position.get(2)?.as_f64()?;
    Some(DesktopTeleportPoint {
        x,
        y,
        country: value
            .get("country")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        level1_area: value
            .get("areas")
            .and_then(Value::as_array)
            .and_then(|areas| areas.first())
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        tran_x: tran_position
            .and_then(|position| position.get(2))
            .and_then(Value::as_f64),
        tran_y: tran_position
            .and_then(|position| position.first())
            .and_then(Value::as_f64),
    })
}

fn desktop_teleport_nearest_point(
    points: &[DesktopTeleportPoint],
    target: &TeleportTargetPlan,
) -> Option<DesktopTeleportPoint> {
    points
        .iter()
        .min_by(|left, right| {
            let left_distance = desktop_teleport_distance_squared(left, target);
            let right_distance = desktop_teleport_distance_squared(right, target);
            left_distance.total_cmp(&right_distance)
        })
        .cloned()
}

fn desktop_teleport_distance_squared(
    point: &DesktopTeleportPoint,
    target: &TeleportTargetPlan,
) -> f64 {
    (point.x - target.x).powi(2) + (point.y - target.y).powi(2)
}

fn execute_desktop_relogin_live(
    config: &AppConfig,
    window: &GameWindowMatch,
    plan: &ReloginExecutionPlan,
    cancellation: Arc<InputCancellationToken>,
) -> Result<ReloginExecutionReport, String> {
    if cancellation.is_cancelled() {
        return Err("Relogin live execution cancelled".to_string());
    }
    let (global_input, capture_size) = desktop_common_job_global_input(config, window, "Relogin")?;
    if plan.capture_size != capture_size {
        return Err(format!(
            "Relogin live execution requires plan capture size {}x{} to match current capture size {}x{}",
            plan.capture_size.width,
            plan.capture_size.height,
            capture_size.width,
            capture_size.height
        ));
    }
    let frame_source = global_input
        .common_job_frame_source()
        .ok_or_else(|| "Relogin live execution has no capture frame source".to_string())?;
    let input_driver = global_input
        .common_job_input_driver(GlobalInputDispatchMode::SendInput, Some(window.handle.0));
    execute_relogin_live(
        plan,
        frame_source,
        input_driver,
        CancellableCommonJobClock::new(cancellation),
    )
    .map_err(|error| error.to_string())
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

#[allow(clippy::result_large_err)]
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

#[allow(clippy::result_large_err)]
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
    app_root.join("bgi-updater.exe")
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

    retained.sort_by_key(|entry| std::cmp::Reverse(entry.0));
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
    let app_root = app_root(app.handle()).unwrap_or_else(|_| PathBuf::from("."));
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
                let _ = toggle_main_window(tray.app_handle());
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
            task_execute_quick_buy,
            task_execute_quick_serenitea_pot,
            task_execute_auto_open_chest,
            task_execute_auto_cook,
            task_execute_auto_eat_tick,
            task_execute_auto_fish_tick,
            task_execute_auto_pick_tick,
            task_execute_quick_teleport_tick,
            task_execute_auto_music_game_performance,
            task_execute_auto_music_game_album,
            task_plan_auto_pathing,
            task_execute_auto_pathing_action_boundary,
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
            redeem_code_auto_redeem_execute,
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
            log_parse_config_get,
            log_parse_config_save,
            log_parse_analyze,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_updater_path_defaults_to_rust_sidecar_name() {
        let root = desktop_test_temp_root("updater-default");
        fs::create_dir_all(&root).unwrap();

        assert_eq!(updater_exe_path(&root), root.join("bgi-updater.exe"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_log_parse_selected_files_limits_to_recent_legacy_logs() {
        let root = desktop_test_temp_root("log-parse-files");
        let log_root = root.join("log");
        fs::create_dir_all(&log_root).unwrap();
        for date in ["20260626", "20260627", "20260628"] {
            fs::write(
                log_root.join(format!("better-genshin-impact{date}.log")),
                "log",
            )
            .unwrap();
        }
        fs::write(log_root.join("ignored.log"), "log").unwrap();

        let limited = desktop_log_parse_selected_files(&root, Some("2")).unwrap();
        assert_eq!(
            limited
                .iter()
                .map(|file| file.date.as_str())
                .collect::<Vec<_>>(),
            vec!["2026-06-27", "2026-06-28"]
        );

        let all = desktop_log_parse_selected_files(&root, Some("All")).unwrap();
        assert_eq!(all.len(), 3);
        assert!(desktop_log_parse_selected_files(&root, Some("bad"))
            .unwrap_err()
            .contains("invalid log day range"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_common_job_capture_size_defaults_when_game_window_is_missing() {
        assert_eq!(
            desktop_common_job_capture_size(None),
            VisionSize::new(1920, 1080)
        );
    }

    #[test]
    fn desktop_common_job_live_plan_skips_unsupported_common_jobs() {
        let plan = bgi_task::plan_common_job(bgi_task::LINNEA_MINING_TASK_KEY, None)
            .unwrap()
            .unwrap();

        let result = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn desktop_common_job_live_plan_reports_supported_jobs_without_game_window() {
        let set_time = bgi_task::plan_common_job(
            bgi_task::SET_TIME_TASK_KEY,
            Some(&serde_json::json!({
                "hour": 8,
                "minute": 30,
                "skip": true
            })),
        )
        .unwrap()
        .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &set_time,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("SetTime live execution requires a detected game window")
        ));

        let choose_talk_option = bgi_task::plan_common_job(
            bgi_task::CHOOSE_TALK_OPTION_TASK_KEY,
            Some(&serde_json::json!({
                "option": "Target"
            })),
        )
        .unwrap()
        .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &choose_talk_option,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("ChooseTalkOption live execution requires a detected game window")
        ));

        let check_rewards = bgi_task::plan_common_job(bgi_task::CHECK_REWARDS_TASK_KEY, None)
            .unwrap()
            .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &check_rewards,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("CheckRewards live execution requires a detected game window")
        ));

        let claim_mail_rewards =
            bgi_task::plan_common_job(bgi_task::CLAIM_MAIL_REWARDS_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &claim_mail_rewards,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("ClaimMailRewards live execution requires a detected game window")
        ));

        let count_inventory_item = bgi_task::plan_common_job(
            bgi_task::COUNT_INVENTORY_ITEM_TASK_KEY,
            Some(&serde_json::json!({
                "gridScreenName": "Materials",
                "itemName": "晶核"
            })),
        )
        .unwrap()
        .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &count_inventory_item,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("CountInventoryItem live execution requires a detected game window")
        ));

        let claim_battle_pass_rewards =
            bgi_task::plan_common_job(bgi_task::CLAIM_BATTLE_PASS_REWARDS_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &claim_battle_pass_rewards,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("ClaimBattlePassRewards live execution requires a detected game window")
        ));

        let blessing_of_the_welkin_moon =
            bgi_task::plan_common_job(bgi_task::BLESSING_WELKIN_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &blessing_of_the_welkin_moon,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("BlessingOfTheWelkinMoon live execution requires a detected game window")
        ));

        let claim_encounter_points_rewards =
            bgi_task::plan_common_job(bgi_task::CLAIM_ENCOUNTER_POINTS_REWARDS_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &claim_encounter_points_rewards,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("ClaimEncounterPointsRewards live execution requires a detected game window")
        ));

        let switch_party = bgi_task::plan_common_job(
            bgi_task::SWITCH_PARTY_TASK_KEY,
            Some(&serde_json::json!({
                "partyName": "Daily.*"
            })),
        )
        .unwrap()
        .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &switch_party,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("SwitchParty live execution requires a detected game window")
        ));

        let wonderland_cycle = bgi_task::plan_common_job(bgi_task::WONDERLAND_CYCLE_TASK_KEY, None)
            .unwrap()
            .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &wonderland_cycle,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("WonderlandCycle live execution requires a detected game window")
        ));

        let relogin = bgi_task::plan_common_job(bgi_task::RELOGIN_TASK_KEY, None)
            .unwrap()
            .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &relogin,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("Relogin live execution requires a detected game window")
        ));

        let teleport = bgi_task::plan_common_job(
            bgi_task::TELEPORT_TASK_KEY,
            Some(&serde_json::json!({
                "x": 1.0,
                "y": 2.0
            })),
        )
        .unwrap()
        .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &teleport,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("Teleport live execution requires a detected game window")
        ));

        let one_key_expedition =
            bgi_task::plan_common_job(bgi_task::ONE_KEY_EXPEDITION_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &one_key_expedition,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("OneKeyExpedition live execution requires a detected game window")
        ));

        let go_to_crafting_bench =
            bgi_task::plan_common_job(bgi_task::GO_TO_CRAFTING_BENCH_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &go_to_crafting_bench,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("GoToCraftingBench live execution requires a detected game window")
        ));

        let go_to_adventurers_guild =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &go_to_adventurers_guild,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("GoToAdventurersGuild live execution requires a detected game window")
        ));

        let go_to_serenitea_pot =
            bgi_task::plan_common_job(bgi_task::GO_TO_SERENITEA_POT_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &go_to_serenitea_pot,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("GoToSereniteaPot live execution requires a detected game window")
        ));

        let scan_pick_drops = bgi_task::plan_common_job(bgi_task::SCAN_PICK_DROPS_TASK_KEY, None)
            .unwrap()
            .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &scan_pick_drops,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("ScanPickDrops live execution requires a detected game window")
        ));

        let lower_head_then_walk_to =
            bgi_task::plan_common_job(bgi_task::LOWER_HEAD_THEN_WALK_TO_TASK_KEY, None)
                .unwrap()
                .unwrap();

        let error = execute_desktop_common_job_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &lower_head_then_walk_to,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("LowerHeadThenWalkTo live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_lower_head_then_walk_to_live_rejects_capture_size_mismatch() {
        let Some(CommonJobExecutionPlan::LowerHeadThenWalkTo(plan)) =
            bgi_task::plan_common_job(bgi_task::LOWER_HEAD_THEN_WALK_TO_TASK_KEY, None).unwrap()
        else {
            panic!("expected LowerHeadThenWalkTo plan");
        };
        let window = GameWindowMatch {
            handle: WindowHandle(1),
            process_id: Some(1),
            process_name: Some("GenshinImpact".to_string()),
            class_name: Some("UnityWndClass".to_string()),
            title: Some("原神".to_string()),
            kind: bgi_capture::GameWindowMatchKind::ProcessName,
            metrics: Some(bgi_capture::GameWindowMetrics::from_legacy_capture_rect(
                1280,
                720,
                bgi_capture::WindowRect::new(0, 0, 1280, 720),
            )),
        };

        let error = execute_desktop_lower_head_then_walk_to_live(
            &AppConfig::default(),
            &window,
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains(
            "LowerHeadThenWalkTo live execution requires plan capture size 1920x1080 to match current capture size 1280x720"
        ));
    }

    #[test]
    fn desktop_scan_pick_drops_live_preflights_before_onnx_gap() {
        let Some(CommonJobExecutionPlan::ScanPickDrops(plan)) =
            bgi_task::plan_common_job(bgi_task::SCAN_PICK_DROPS_TASK_KEY, None).unwrap()
        else {
            panic!("expected ScanPickDrops plan");
        };

        let small_window = desktop_test_game_window(1280, 720);
        let size_error = execute_desktop_scan_pick_drops_live(
            &AppConfig::default(),
            &small_window,
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(size_error.contains(
            "ScanPickDrops live execution requires plan capture size 1920x1080 to match current capture size 1280x720"
        ));
        assert!(!size_error.contains("ONNX YOLO inference"));

        let wgc_config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        let bit_blt_error = execute_desktop_scan_pick_drops_live(
            &wgc_config,
            &desktop_test_game_window(1920, 1080),
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(bit_blt_error
            .contains("ScanPickDrops live execution requires the BitBlt capture backend"));
        assert!(!bit_blt_error.contains("ONNX YOLO inference"));

        let adapter_error = execute_desktop_scan_pick_drops_live(
            &AppConfig::default(),
            &desktop_test_game_window(1920, 1080),
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(adapter_error.contains(
            "ScanPickDrops live execution requires desktop BgiWorld ONNX YOLO inference and overlay adapters"
        ));
    }

    #[test]
    fn desktop_go_to_crafting_bench_preflight_reports_interaction_after_pathing_contract() {
        let Some(CommonJobExecutionPlan::GoToCraftingBench(plan)) = bgi_task::plan_common_job(
            bgi_task::GO_TO_CRAFTING_BENCH_TASK_KEY,
            Some(&serde_json::json!({ "country": "璃月" })),
        )
        .unwrap() else {
            panic!("expected GoToCraftingBench common job plan");
        };

        let pathing_report = desktop_common_job_pathing_live_preflight(
            "GoToCraftingBench",
            &plan.pathing_rule.pathing_json,
        )
        .unwrap();
        assert!(pathing_report.has_positions);
        assert!(pathing_report.waypoint_count > 0);
        assert!(!pathing_report.native_pathing_completed);

        let error = desktop_go_to_crafting_bench_live_preflight(&plan).unwrap_err();

        assert!(error.contains(
            "GoToCraftingBench live execution requires desktop crafting-bench interaction adapter"
        ));
        assert!(!error.contains("native PathExecutor adapter"));
    }

    #[test]
    fn desktop_go_to_adventurers_guild_preflight_reports_catherine_after_pathing_contract() {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(plan)) = bgi_task::plan_common_job(
            bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY,
            Some(&serde_json::json!({ "country": "蒙德" })),
        )
        .unwrap() else {
            panic!("expected GoToAdventurersGuild common job plan");
        };

        let pathing_report = desktop_common_job_pathing_live_preflight(
            "GoToAdventurersGuild",
            &plan.pathing_rule.pathing_json,
        )
        .unwrap();
        assert!(pathing_report.has_positions);
        assert!(pathing_report.waypoint_count > 0);
        assert!(!pathing_report.native_pathing_completed);

        let error = desktop_go_to_adventurers_guild_live_preflight(&plan).unwrap_err();

        assert!(error.contains(
            "GoToAdventurersGuild live execution requires desktop Catherine interaction adapter"
        ));
        assert!(error.contains("Pathing"));
        assert!(!error.contains("native PathExecutor adapter"));
    }

    #[test]
    fn desktop_go_to_adventurers_guild_live_preflights_capture_before_catherine_adapter() {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(plan)) = bgi_task::plan_common_job(
            bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY,
            Some(&serde_json::json!({ "country": "蒙德" })),
        )
        .unwrap() else {
            panic!("expected GoToAdventurersGuild common job plan");
        };

        let size_error = execute_desktop_go_to_adventurers_guild_live(
            &AppConfig::default(),
            &desktop_test_game_window(1280, 720),
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(size_error.contains(
            "GoToAdventurersGuild live execution requires plan capture size 1920x1080 to match current capture size 1280x720"
        ));
        assert!(!size_error.contains("Catherine interaction adapter"));

        let wgc_config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        let capture_backend_error = execute_desktop_go_to_adventurers_guild_live(
            &wgc_config,
            &desktop_test_game_window(1920, 1080),
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(capture_backend_error
            .contains("GoToAdventurersGuild live execution requires the BitBlt capture backend"));
        assert!(!capture_backend_error.contains("Catherine interaction adapter"));

        let adapter_error = execute_desktop_go_to_adventurers_guild_live(
            &AppConfig::default(),
            &desktop_test_game_window(1920, 1080),
            &plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(adapter_error.contains(
            "GoToAdventurersGuild live execution requires desktop Catherine interaction adapter"
        ));
    }

    #[test]
    fn desktop_go_to_adventurers_guild_preflight_skips_unknown_nested_when_static_condition_false()
    {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut default_plan)) =
            bgi_task::plan_common_job(
                bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY,
                Some(&serde_json::json!({ "country": "蒙德" })),
            )
            .unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        default_plan.steps[1].action = GoToAdventurersGuildStepAction::CommonJob {
            task_key: "SkippedUnsupportedPartyJob".to_string(),
            config: None,
        };

        let default_error =
            desktop_go_to_adventurers_guild_live_preflight(&default_plan).unwrap_err();

        assert!(default_error.contains("desktop Catherine interaction adapter"));
        assert!(!default_error.contains("SkippedUnsupportedPartyJob"));

        let config = serde_json::json!({
            "country": "蒙德",
            "onlyDoOnce": true
        });
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut only_once_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, Some(&config))
                .unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        only_once_plan.steps[2].action = GoToAdventurersGuildStepAction::CommonJob {
            task_key: "SkippedUnsupportedEncounterJob".to_string(),
            config: None,
        };

        let only_once_error =
            desktop_go_to_adventurers_guild_live_preflight(&only_once_plan).unwrap_err();

        assert!(only_once_error.contains("desktop Catherine interaction adapter"));
        assert!(!only_once_error.contains("SkippedUnsupportedEncounterJob"));
    }

    #[test]
    fn desktop_go_to_adventurers_guild_preflight_skips_static_conditions() {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(default_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        let config = serde_json::json!({
            "dailyRewardPartyName": "daily",
            "onlyDoOnce": true
        });
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(configured_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, Some(&config))
                .unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };

        assert!(
            !desktop_go_to_adventurers_guild_preflight_condition_applies(
                &default_plan,
                GoToAdventurersGuildStepCondition::WhenDailyRewardPartyConfigured
            )
        );
        assert!(desktop_go_to_adventurers_guild_preflight_condition_applies(
            &default_plan,
            GoToAdventurersGuildStepCondition::WhenOnlyDoOnceFalse
        ));
        assert!(desktop_go_to_adventurers_guild_preflight_condition_applies(
            &configured_plan,
            GoToAdventurersGuildStepCondition::WhenDailyRewardPartyConfigured
        ));
        assert!(
            !desktop_go_to_adventurers_guild_preflight_condition_applies(
                &configured_plan,
                GoToAdventurersGuildStepCondition::WhenOnlyDoOnceFalse
            )
        );
    }

    #[test]
    fn desktop_go_to_adventurers_guild_preflight_rejects_unknown_nested_bridge() {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        plan.steps.insert(
            1,
            bgi_task::GoToAdventurersGuildStep {
                phase: bgi_task::GoToAdventurersGuildStepPhase::Setup,
                condition: GoToAdventurersGuildStepCondition::Always,
                label: "probe unsupported nested bridge".to_string(),
                action: GoToAdventurersGuildStepAction::CommonJob {
                    task_key: "UnsupportedNestedJob".to_string(),
                    config: None,
                },
            },
        );

        let error = desktop_go_to_adventurers_guild_live_preflight(&plan).unwrap_err();

        assert!(error.contains(
            "GoToAdventurersGuild live execution has no desktop bridge for nested common-job UnsupportedNestedJob"
        ));
        assert!(error.contains("probe unsupported nested bridge"));
    }

    #[test]
    fn desktop_go_to_adventurers_guild_preflight_reports_next_adapter_boundaries() {
        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut catherine_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        for step in &mut catherine_plan.steps {
            if matches!(step.action, GoToAdventurersGuildStepAction::Pathing { .. }) {
                step.action = GoToAdventurersGuildStepAction::Log {
                    message: "skip pathing blocker in desktop preflight test".to_string(),
                };
            }
        }

        let catherine_error =
            desktop_go_to_adventurers_guild_live_preflight(&catherine_plan).unwrap_err();

        assert!(catherine_error.contains("desktop Catherine interaction adapter"));
        assert!(catherine_error.contains("Pathing"));
        assert!(catherine_error.contains("retry Catherine interaction until talk UI opens"));

        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut daily_drain_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        for step in &mut daily_drain_plan.steps {
            if matches!(
                step.action,
                GoToAdventurersGuildStepAction::Pathing { .. }
                    | GoToAdventurersGuildStepAction::InteractionRetry { .. }
            ) {
                step.action = GoToAdventurersGuildStepAction::Log {
                    message: "skip earlier blocker in desktop preflight test".to_string(),
                };
            }
        }

        let daily_drain_error =
            desktop_go_to_adventurers_guild_live_preflight(&daily_drain_plan).unwrap_err();

        assert!(daily_drain_error.contains("desktop talk-option drain adapter"));
        assert!(daily_drain_error.contains("DailyReward"));
        assert!(daily_drain_error.contains("select trailing dialogue options after daily reward"));

        let Some(CommonJobExecutionPlan::GoToAdventurersGuild(mut cleanup_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_ADVENTURERS_GUILD_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToAdventurersGuild common job plan");
        };
        for step in &mut cleanup_plan.steps {
            if matches!(
                step.action,
                GoToAdventurersGuildStepAction::Pathing { .. }
                    | GoToAdventurersGuildStepAction::InteractionRetry { .. }
            ) || (step.condition
                == GoToAdventurersGuildStepCondition::WhenDailyRewardOptionFound
                && matches!(
                    step.action,
                    GoToAdventurersGuildStepAction::SelectLastTalkOptionUntilEnd { .. }
                ))
            {
                step.action = GoToAdventurersGuildStepAction::Log {
                    message: "skip earlier blocker in desktop preflight test".to_string(),
                };
            }
        }

        let cleanup_error =
            desktop_go_to_adventurers_guild_live_preflight(&cleanup_plan).unwrap_err();

        assert!(cleanup_error.contains("desktop talk UI probe/drain adapter"));
        assert!(cleanup_error.contains("Cleanup"));
        assert!(cleanup_error.contains("select last option to exit remaining dialogue"));
    }

    #[test]
    fn desktop_go_to_serenitea_pot_live_preflight_rejects_before_side_effects() {
        let Some(CommonJobExecutionPlan::GoToSereniteaPot(plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_SERENITEA_POT_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToSereniteaPot common job plan");
        };

        let error = desktop_go_to_serenitea_pot_live_preflight(&plan).unwrap_err();

        assert!(error.contains(
            "GoToSereniteaPot live execution requires desktop Serenitea Pot map-entry adapter"
        ));
    }

    #[test]
    fn desktop_go_to_serenitea_pot_live_preflight_respects_bag_gadget_entry() {
        let config = serde_json::json!({
            "sereniteaPotTpType": "尘歌壶道具"
        });
        let Some(CommonJobExecutionPlan::GoToSereniteaPot(plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_SERENITEA_POT_TASK_KEY, Some(&config))
                .unwrap()
        else {
            panic!("expected GoToSereniteaPot common job plan");
        };

        let error = desktop_go_to_serenitea_pot_live_preflight(&plan).unwrap_err();

        assert!(error.contains(
            "GoToSereniteaPot live execution requires desktop Serenitea Pot bag-entry adapter"
        ));
        assert!(!error.contains("map-entry adapter"));
    }

    #[test]
    fn desktop_go_to_serenitea_pot_preflight_skips_empty_shop_config_statically() {
        let Some(CommonJobExecutionPlan::GoToSereniteaPot(empty_shop_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_SERENITEA_POT_TASK_KEY, None).unwrap()
        else {
            panic!("expected GoToSereniteaPot common job plan");
        };
        let shop_config = serde_json::json!({
            "secretTreasureObjects": ["摩拉"]
        });
        let Some(CommonJobExecutionPlan::GoToSereniteaPot(configured_shop_plan)) =
            bgi_task::plan_common_job(bgi_task::GO_TO_SERENITEA_POT_TASK_KEY, Some(&shop_config))
                .unwrap()
        else {
            panic!("expected GoToSereniteaPot common job plan");
        };

        assert!(!desktop_go_to_serenitea_pot_preflight_condition_applies(
            &empty_shop_plan,
            GoToSereniteaPotStepCondition::WhenShopConfiguredAndDue
        ));
        assert!(desktop_go_to_serenitea_pot_preflight_condition_applies(
            &configured_shop_plan,
            GoToSereniteaPotStepCondition::WhenShopConfiguredAndDue
        ));
    }

    #[test]
    fn desktop_teleport_tp_json_parser_preserves_legacy_coordinate_mapping() {
        let json = r#"{
            "data": [
                {
                    "mapName": "Teyvat",
                    "points": [
                        {
                            "id": 1,
                            "country": "蒙德",
                            "areas": ["苍风高地", "清泉镇"],
                            "position": [100.0, 0.0, 30.0],
                            "tranPosition": [110.0, 0.0, 31.0]
                        },
                        {
                            "id": 2,
                            "country": "璃月",
                            "areas": ["碧水原"],
                            "position": [200.0, 0.0, -10.0],
                            "tranPosition": [205.0, 0.0, -11.0]
                        }
                    ]
                }
            ]
        }"#;
        let points = desktop_teleport_points_from_json(json, "Teyvat").unwrap();
        let target = TeleportTargetPlan {
            x: -12.0,
            y: 198.0,
            map_name: Some("Teyvat".to_string()),
        };

        let nearest = desktop_teleport_nearest_point(&points, &target).unwrap();

        assert_eq!(
            points[0],
            DesktopTeleportPoint {
                x: 30.0,
                y: 100.0,
                country: Some("蒙德".to_string()),
                level1_area: Some("苍风高地".to_string()),
                tran_x: Some(31.0),
                tran_y: Some(110.0),
            }
        );
        assert_eq!(
            nearest,
            DesktopTeleportPoint {
                x: -10.0,
                y: 200.0,
                country: Some("璃月".to_string()),
                level1_area: Some("碧水原".to_string()),
                tran_x: Some(-11.0),
                tran_y: Some(205.0),
            }
        );
    }

    #[test]
    fn desktop_teleport_underground_locators_reuse_quick_teleport_templates() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let detect = desktop_teleport_quick_template_locator_plan(
            &plan.locators.map_underground_switch_button,
            plan.capture_size,
            "TeleportUndergroundSwitchButton",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )
        .unwrap();
        let click = desktop_teleport_quick_template_locator_plan(
            &plan.locators.map_underground_to_ground_button,
            plan.capture_size,
            "TeleportUndergroundToGroundButton",
            BvLocatorOperation::Click,
            600,
            100,
            6,
        )
        .unwrap();

        assert_eq!(detect.operation, BvLocatorOperation::IsExist);
        assert_eq!(detect.retry_count, 1);
        assert_eq!(
            detect.recognition_object.name.as_deref(),
            Some("TeleportUndergroundSwitchButton")
        );
        assert_eq!(
            detect.recognition_object.region_of_interest,
            Some(plan.locators.map_underground_switch_button.roi)
        );
        assert!(detect.recognition_object.template.use_3_channels);
        assert_eq!(click.operation, BvLocatorOperation::Click);
        assert_eq!(click.retry_count, 6);
        assert_eq!(
            click.recognition_object.name.as_deref(),
            Some("TeleportUndergroundToGroundButton")
        );
        assert_eq!(
            click.recognition_object.region_of_interest,
            Some(plan.locators.map_underground_to_ground_button.roi)
        );
    }

    #[test]
    fn desktop_teleport_point_not_activated_locator_reuses_quick_teleport_close_template() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let locator = desktop_teleport_quick_template_locator_plan(
            &plan.locators.map_close_button,
            plan.capture_size,
            "TeleportMapCloseButton",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )
        .unwrap();

        assert_eq!(locator.operation, BvLocatorOperation::IsExist);
        assert_eq!(locator.retry_count, 1);
        assert_eq!(
            locator.recognition_object.name.as_deref(),
            Some("TeleportMapCloseButton")
        );
        assert_eq!(
            locator.recognition_object.region_of_interest,
            Some(plan.locators.map_close_button.roi)
        );
        assert_eq!(
            locator.recognition_object.template.threshold,
            plan.locators.map_close_button.threshold
        );
        assert_eq!(
            locator.recognition_object.template.use_3_channels,
            plan.locators.map_close_button.use_3_channels
        );
    }

    #[test]
    fn desktop_teleport_teleport_button_detect_and_once_locators_share_quick_template() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let detect = desktop_teleport_quick_template_locator_plan(
            &plan.locators.teleport_button,
            plan.capture_size,
            "TeleportPanelButtonDetect",
            BvLocatorOperation::IsExist,
            100,
            100,
            1,
        )
        .unwrap();
        let once = desktop_teleport_quick_template_locator_plan(
            &plan.locators.teleport_button,
            plan.capture_size,
            "TeleportPanelButtonOnce",
            BvLocatorOperation::Click,
            100,
            100,
            1,
        )
        .unwrap();

        assert_eq!(detect.operation, BvLocatorOperation::IsExist);
        assert_eq!(once.operation, BvLocatorOperation::Click);
        assert_eq!(detect.retry_count, 1);
        assert_eq!(once.retry_count, 1);
        assert_eq!(
            detect.recognition_object.region_of_interest,
            Some(plan.locators.teleport_button.roi)
        );
        assert_eq!(
            once.recognition_object.region_of_interest,
            Some(plan.locators.teleport_button.roi)
        );
    }

    #[test]
    fn desktop_teleport_candidate_click_delay_has_legacy_minimum() {
        assert_eq!(desktop_teleport_candidate_click_delay_ms(0), 500);
        assert_eq!(desktop_teleport_candidate_click_delay_ms(333), 500);
        assert_eq!(desktop_teleport_candidate_click_delay_ms(500), 500);
        assert_eq!(desktop_teleport_candidate_click_delay_ms(750), 750);
    }

    #[test]
    fn desktop_teleport_big_map_recognition_preflight_reports_missing_assets() {
        let root = std::env::temp_dir().join(format!(
            "bgi-teleport-missing-assets-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let keypoints = root.join("Teyvat_0_256_SIFT.kp.bin");
        let mat = root.join("Teyvat_0_256_SIFT.mat.png");

        let status = desktop_teleport_big_map_recognition_preflight(&keypoints, &mat, 8);
        assert_eq!(
            status,
            DesktopTeleportBigMapRecognitionPreflight::MissingAssets {
                assets: vec![keypoints.clone(), mat.clone()]
            }
        );
        let error = desktop_teleport_big_map_recognition_preflight_error(&keypoints, &mat, 8);

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("Teleport BigMap SIFT assets are missing")
                    && message.contains("Teyvat_0_256_SIFT.kp.bin")
                    && message.contains("Teyvat_0_256_SIFT.mat.png")
        ));
    }

    #[test]
    fn desktop_teleport_big_map_recognition_preflight_reports_only_missing_asset() {
        let root = std::env::temp_dir().join(format!(
            "bgi-teleport-one-missing-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let keypoints = root.join("Teyvat_0_256_SIFT.kp.bin");
        let mat = root.join("Teyvat_0_256_SIFT.mat.png");
        std::fs::write(&keypoints, b"").unwrap();

        let status = desktop_teleport_big_map_recognition_preflight(&keypoints, &mat, 8);
        assert_eq!(
            status,
            DesktopTeleportBigMapRecognitionPreflight::MissingAssets {
                assets: vec![mat.clone()]
            }
        );
        let error = desktop_teleport_big_map_recognition_preflight_error(&keypoints, &mat, 8);
        let _ = std::fs::remove_dir_all(&root);

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("Teleport BigMap SIFT assets are missing")
                    && !message.contains("Teyvat_0_256_SIFT.kp.bin")
                    && message.contains("Teyvat_0_256_SIFT.mat.png")
        ));
    }

    #[test]
    fn desktop_teleport_big_map_recognition_preflight_rejects_invalid_scale() {
        let root = std::env::temp_dir().join("bgi-teleport-invalid-scale");

        let keypoints = root.join("Teyvat_0_256_SIFT.kp.bin");
        let mat = root.join("Teyvat_0_256_SIFT.mat.png");
        let status = desktop_teleport_big_map_recognition_preflight(&keypoints, &mat, 0);
        assert_eq!(
            status,
            DesktopTeleportBigMapRecognitionPreflight::InvalidScale {
                layer_256_to_2048_scale: 0
            }
        );
        let error = desktop_teleport_big_map_recognition_preflight_error(
            &root.join("Teyvat_0_256_SIFT.kp.bin"),
            &root.join("Teyvat_0_256_SIFT.mat.png"),
            0,
        );

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("invalid 256-to-2048 layer scale 0")
        ));
    }

    #[test]
    fn desktop_teleport_big_map_recognition_preflight_reports_pending_adapter() {
        let root = std::env::temp_dir().join(format!(
            "bgi-teleport-present-assets-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let keypoints = root.join("Teyvat_0_256_SIFT.kp.bin");
        let mat = root.join("Teyvat_0_256_SIFT.mat.png");
        std::fs::write(&keypoints, b"").unwrap();
        std::fs::write(&mat, b"").unwrap();

        let status = desktop_teleport_big_map_recognition_preflight(&keypoints, &mat, 8);
        assert_eq!(
            status,
            DesktopTeleportBigMapRecognitionPreflight::AdapterUnavailable
        );
        let error = desktop_teleport_big_map_recognition_preflight_error(&keypoints, &mat, 8);
        let _ = std::fs::remove_dir_all(&root);

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("native OpenCV/SIFT map-matching adapter is not ported")
                    && message.contains("256-to-2048 layer scale 8")
        ));
    }

    #[test]
    fn desktop_teleport_zoom_level_from_scale_button_region_matches_legacy_formula() {
        let capture_size = VisionSize::new(1920, 1080);
        let top = Rect::new(40, 467, 14, 2).unwrap();
        let bottom = Rect::new(40, 611, 14, 2).unwrap();
        let middle = Rect::new(40, 539, 14, 2).unwrap();

        assert!(
            (desktop_teleport_zoom_level_from_scale_button_region(top, capture_size) - 1.0).abs()
                < 0.001
        );
        assert!(
            (desktop_teleport_zoom_level_from_scale_button_region(bottom, capture_size) - 6.0)
                .abs()
                < 0.001
        );
        assert!(
            (desktop_teleport_zoom_level_from_scale_button_region(middle, capture_size) - 3.5)
                .abs()
                < 0.001
        );

        let scaled = Rect::new(27, 311, 10, 2).unwrap();
        assert!(
            (desktop_teleport_zoom_level_from_scale_button_region(
                scaled,
                VisionSize::new(1280, 720)
            ) - 1.0)
                .abs()
                < 0.02
        );
    }

    #[test]
    fn desktop_teleport_zoom_adjustment_targets_legacy_visible_range() {
        assert_eq!(desktop_teleport_zoom_adjust_target(4.46), Some(4.4));
        assert_eq!(desktop_teleport_zoom_adjust_target(4.45), None);
        assert_eq!(desktop_teleport_zoom_adjust_target(1.94), Some(2.0));
        assert_eq!(desktop_teleport_zoom_adjust_target(1.95), None);
        assert_eq!(desktop_teleport_zoom_adjust_target(3.0), None);
    }

    #[test]
    fn desktop_teleport_zoom_drag_events_match_legacy_slider_points() {
        let events = desktop_teleport_zoom_drag_events(6.0, 4.4, VisionSize::new(1920, 1080));

        assert_eq!(
            events.first(),
            Some(&bgi_input::InputEvent::MouseMoveAbsolute {
                x: 47,
                y: 612,
                virtual_desktop: false
            })
        );
        assert!(events.contains(&bgi_input::InputEvent::MouseButtonDown {
            button: MouseButton::Left
        }));
        assert!(events.contains(&bgi_input::InputEvent::MouseButtonUp {
            button: MouseButton::Left
        }));
        assert_eq!(
            events.get(4),
            Some(&bgi_input::InputEvent::MouseMoveAbsolute {
                x: 47,
                y: 566,
                virtual_desktop: false
            })
        );
        assert_eq!(
            events.last(),
            Some(&bgi_input::InputEvent::MouseMoveAbsolute {
                x: 960,
                y: 960,
                virtual_desktop: false
            })
        );
    }

    #[test]
    fn desktop_teleport_generate_drag_steps_preserves_legacy_cosine_distribution() {
        let positive = desktop_teleport_generate_drag_steps(100, 5);
        let negative = desktop_teleport_generate_drag_steps(-100, 5);

        assert_eq!(positive, vec![27, 26, 23, 16, 8]);
        assert_eq!(positive.iter().sum::<i32>(), 100);
        assert_eq!(negative, vec![-27, -26, -23, -16, -8]);
        assert_eq!(negative.iter().sum::<i32>(), -100);
    }

    #[test]
    fn desktop_teleport_drag_plan_limits_single_move_and_predicts_center() {
        let plan = desktop_teleport_drag_plan(
            DesktopTeleportMapPoint { x: 0.0, y: 0.0 },
            DesktopTeleportMapPoint {
                x: 1000.0,
                y: -500.0,
            },
            2.0,
        )
        .unwrap();

        assert_eq!(plan.mouse_move_x, 268);
        assert_eq!(plan.mouse_move_y, -134);
        assert_eq!(plan.steps, 29);
        assert!((plan.mouse_distance - 1319.839).abs() < 0.01);
        assert!((plan.predicted_center.x - 227.022).abs() < 0.01);
        assert!((plan.predicted_center.y + 113.511).abs() < 0.01);
        assert!(desktop_teleport_drag_plan(
            DesktopTeleportMapPoint { x: 0.0, y: 0.0 },
            DesktopTeleportMapPoint { x: 10.0, y: 0.0 },
            2.0
        )
        .is_none());
    }

    #[test]
    fn desktop_teleport_drag_plan_uses_move_map_rule_parameters() {
        let mut rule = bgi_task::default_teleport_move_map_rule();
        rule.map_scale_factor = 1.0;
        rule.max_mouse_move = 50.0;
        rule.move_tolerance = 10.0;

        let plan = desktop_teleport_drag_plan_with_rule(
            DesktopTeleportMapPoint { x: 0.0, y: 0.0 },
            DesktopTeleportMapPoint { x: 1000.0, y: 0.0 },
            2.0,
            &rule,
        )
        .unwrap();

        assert_eq!(plan.mouse_move_x, 50);
        assert_eq!(plan.mouse_move_y, 0);
        assert_eq!(plan.steps, 5);
        assert_eq!(plan.mouse_distance, 500.0);
        assert_eq!(
            plan.predicted_center,
            DesktopTeleportMapPoint { x: 100.0, y: 0.0 }
        );
    }

    #[test]
    fn desktop_teleport_move_map_zoom_target_matches_legacy_thresholds() {
        let mut rule = bgi_task::default_teleport_move_map_rule();

        assert_eq!(
            desktop_teleport_move_map_zoom_target(2.0, 1500.0, 2.0, &rule),
            Some(3.0)
        );
        assert_eq!(
            desktop_teleport_move_map_zoom_target(4.0, 2000.0, 2.0, &rule),
            Some(5.0)
        );
        assert_eq!(
            desktop_teleport_move_map_zoom_target(4.0, 300.0, 2.0, &rule),
            Some(3.0)
        );
        assert_eq!(
            desktop_teleport_move_map_zoom_target(2.03, 300.0, 2.0, &rule),
            None
        );
        assert_eq!(
            desktop_teleport_move_map_zoom_target(3.0, 700.0, 2.0, &rule),
            None
        );

        rule.map_zoom_enabled = false;
        assert_eq!(
            desktop_teleport_move_map_zoom_target(2.0, 1500.0, 2.0, &rule),
            None
        );
    }

    #[test]
    fn desktop_teleport_mouse_move_map_events_use_drag_steps_and_seeded_start() {
        let events =
            desktop_teleport_mouse_move_map_events(100, -50, 5, VisionSize::new(1920, 1080), 1);

        assert_eq!(
            events.first(),
            Some(&bgi_input::InputEvent::MouseMoveAbsolute {
                x: 641,
                y: 528,
                virtual_desktop: false
            })
        );
        assert_eq!(
            events.get(1),
            Some(&bgi_input::InputEvent::MouseButtonDown {
                button: MouseButton::Left
            })
        );
        assert_eq!(
            events.get(2),
            Some(&bgi_input::InputEvent::Delay { milliseconds: 20 })
        );
        assert_eq!(
            events.get(3),
            Some(&bgi_input::InputEvent::MouseMoveAbsolute {
                x: 668,
                y: 515,
                virtual_desktop: false
            })
        );
        assert_eq!(
            events.last(),
            Some(&bgi_input::InputEvent::MouseButtonUp {
                button: MouseButton::Left
            })
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, bgi_input::InputEvent::Delay { .. }))
                .count(),
            5
        );
        let absolute_points = events
            .iter()
            .filter_map(|event| match event {
                bgi_input::InputEvent::MouseMoveAbsolute { x, y, .. } => Some((*x, *y)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(absolute_points.first(), Some(&(641, 528)));
        assert_eq!(absolute_points.last(), Some(&(741, 478)));
    }

    #[test]
    fn desktop_teleport_mouse_move_map_events_use_configured_step_interval() {
        let events = desktop_teleport_mouse_move_map_events_with_interval(
            10,
            0,
            2,
            VisionSize::new(1920, 1080),
            1,
            7,
        );

        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, bgi_input::InputEvent::Delay { milliseconds: 7 }))
                .count(),
            2
        );
    }

    #[test]
    fn desktop_teleport_teyvat_coordinate_conversion_uses_legacy_origin_and_scale() {
        let origin = desktop_teleport_genshin_point_to_image_point(
            "Teyvat",
            DesktopTeleportMapPoint { x: 0.0, y: 0.0 },
        )
        .unwrap();
        let point = desktop_teleport_genshin_point_to_image_point(
            "Teyvat",
            DesktopTeleportMapPoint { x: 100.0, y: -50.0 },
        )
        .unwrap();
        let rect = desktop_teleport_genshin_rect_to_image_rect(
            "Teyvat",
            DesktopTeleportMapRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
        )
        .unwrap();

        assert_eq!(
            origin,
            DesktopTeleportMapPoint {
                x: 32768.0,
                y: 16384.0
            }
        );
        assert_eq!(
            point,
            DesktopTeleportMapPoint {
                x: 32568.0,
                y: 16484.0
            }
        );
        assert_eq!(
            rect,
            DesktopTeleportMapRect {
                x: 32568.0,
                y: 16284.0,
                width: 200.0,
                height: 100.0
            }
        );
        assert!(desktop_teleport_genshin_point_to_image_point(
            "Unknown",
            DesktopTeleportMapPoint { x: 0.0, y: 0.0 }
        )
        .is_none());
    }

    #[test]
    fn desktop_teleport_midpoint_even_round_matches_midpoint_to_even() {
        assert_eq!(desktop_teleport_midpoint_even_round(2046.5), 2046.0);
        assert_eq!(desktop_teleport_midpoint_even_round(2047.5), 2048.0);
        assert_eq!(desktop_teleport_midpoint_even_round(-1.5), -2.0);
        assert_eq!(desktop_teleport_midpoint_even_round(-2.5), -2.0);
        assert_eq!(desktop_teleport_midpoint_even_round(12.49), 12.0);
        assert_eq!(desktop_teleport_midpoint_even_round(12.51), 13.0);
    }

    #[test]
    fn desktop_teleport_target_capture_point_and_window_guard_match_legacy_rules() {
        let capture_size = VisionSize::new(1920, 1080);
        let big_map_rect = DesktopTeleportMapRect {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 500.0,
        };
        let center = DesktopTeleportMapPoint { x: 500.0, y: 250.0 };
        let top_left = DesktopTeleportMapPoint { x: 900.0, y: 450.0 };
        let left_margin = DesktopTeleportMapPoint { x: 990.0, y: 250.0 };
        let outside = DesktopTeleportMapPoint {
            x: 1200.0,
            y: 250.0,
        };
        let right_edge = DesktopTeleportMapPoint {
            x: 1000.0,
            y: 250.0,
        };
        let bottom_edge = DesktopTeleportMapPoint { x: 500.0, y: 500.0 };

        let click =
            desktop_teleport_target_capture_point(capture_size, "Teyvat", big_map_rect, center)
                .unwrap();
        assert!((click.x - 960.0).abs() < 0.001);
        assert!((click.y - 540.0).abs() < 0.001);
        assert!(desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            center
        ));
        assert!(!desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            top_left
        ));
        assert!(!desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            left_margin
        ));
        assert!(!desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            outside
        ));
        assert!(!desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            right_edge
        ));
        assert!(!desktop_teleport_target_point_in_big_map_window(
            capture_size,
            "Teyvat",
            big_map_rect,
            bottom_edge
        ));
    }

    #[test]
    fn desktop_teleport_point_not_activated_policy_outcomes_match_legacy_boundaries() {
        assert_eq!(
            desktop_teleport_point_not_activated_outcome(
                true,
                None,
                &TeleportFailurePolicy::HardError
            )
            .unwrap(),
            CommonJobRuntimeOutcome::Matched(true)
        );
        assert_eq!(
            desktop_teleport_point_not_activated_outcome(
                false,
                Some("map close button"),
                &TeleportFailurePolicy::ContinueAfterPointNotActivated
            )
            .unwrap(),
            CommonJobRuntimeOutcome::Matched(true)
        );
        assert_eq!(
            desktop_teleport_point_not_activated_outcome(
                false,
                Some("map close button"),
                &TeleportFailurePolicy::WarningOnly
            )
            .unwrap(),
            CommonJobRuntimeOutcome::Matched(true)
        );

        let hard_error = desktop_teleport_point_not_activated_outcome(
            false,
            Some("map close button"),
            &TeleportFailurePolicy::HardError,
        )
        .unwrap_err();
        assert!(matches!(
            hard_error,
            TaskError::CommonJobExecution(message)
                if message.contains("map close button")
        ));

        let missing_observation = desktop_teleport_point_not_activated_outcome(
            false,
            None,
            &TeleportFailurePolicy::ContinueAfterPointNotActivated,
        )
        .unwrap_err();
        assert!(matches!(
            missing_observation,
            TaskError::CommonJobExecution(message)
                if message.contains("before any teleport panel click")
        ));
    }

    #[test]
    fn desktop_teleport_switch_area_name_prefers_force_country_then_target_context() {
        let rule = bgi_task::default_teleport_move_map_rule();
        let nearest = DesktopTeleportPoint {
            x: 1.0,
            y: 2.0,
            country: Some("璃月".to_string()),
            level1_area: Some("碧水原".to_string()),
            tran_x: None,
            tran_y: None,
        };

        assert_eq!(
            desktop_teleport_switch_area_name(
                Some("Teyvat"),
                Some("蒙德"),
                Some(&nearest),
                None,
                None,
                &rule
            ),
            Some("蒙德".to_string())
        );
        assert_eq!(
            desktop_teleport_switch_area_name(
                Some("Teyvat"),
                None,
                Some(&nearest),
                None,
                None,
                &rule
            ),
            Some("璃月".to_string())
        );
        assert_eq!(
            desktop_teleport_switch_area_name(Some("Enkanomiya"), None, None, None, None, &rule),
            Some("渊下宫".to_string())
        );
        assert_eq!(
            desktop_teleport_switch_area_name(
                Some("Teyvat"),
                None,
                None,
                Some(DesktopTeleportMapPoint {
                    x: 9000.0,
                    y: -1800.0
                }),
                None,
                &rule
            ),
            Some("纳塔".to_string())
        );
        assert_eq!(
            desktop_teleport_switch_area_name(
                Some("Teyvat"),
                None,
                None,
                Some(DesktopTeleportMapPoint { x: 100.0, y: 100.0 }),
                Some(DesktopTeleportMapPoint { x: 110.0, y: 110.0 }),
                &rule
            ),
            None
        );
        assert_eq!(
            desktop_teleport_switch_area_name(Some("Teyvat"), None, None, None, None, &rule),
            None
        );
    }

    #[test]
    fn desktop_teleport_area_menu_helpers_preserve_legacy_roi_click_and_aliases() {
        let capture_size = VisionSize::new(1920, 1080);
        assert_eq!(
            desktop_teleport_area_menu_button_point(capture_size),
            (1760, 1020)
        );
        assert_eq!(
            desktop_teleport_area_menu_ocr_roi(capture_size).unwrap(),
            Rect {
                x: 1280,
                y: 0,
                width: 640,
                height: 1080
            }
        );
        assert!(desktop_teleport_area_text_matches("渊下宮", "渊下宫"));
        assert!(desktop_teleport_area_text_matches(
            "  层岩巨渊 ",
            "层岩巨渊"
        ));
        assert!(!desktop_teleport_area_text_matches("璃月", "蒙德"));
    }

    #[test]
    fn desktop_teleport_area_menu_candidates_offset_ocr_regions() {
        let source_roi = Rect {
            x: 1280,
            y: 0,
            width: 640,
            height: 1080,
        };
        let candidates = desktop_teleport_area_menu_candidates_from_ocr_regions(
            &[
                OcrResultRegion {
                    rect: Rect {
                        x: 20,
                        y: 300,
                        width: 80,
                        height: 28,
                    },
                    text: "蒙德".to_string(),
                    score: 0.9,
                },
                OcrResultRegion {
                    rect: Rect {
                        x: 10,
                        y: 340,
                        width: 50,
                        height: 20,
                    },
                    text: " ".to_string(),
                    score: 0.1,
                },
            ],
            source_roi,
        )
        .unwrap();

        assert_eq!(
            candidates,
            vec![DesktopTeleportAreaMenuCandidate {
                text: "蒙德".to_string(),
                rect: Rect {
                    x: 1300,
                    y: 300,
                    width: 80,
                    height: 28
                }
            }]
        );
    }

    #[test]
    fn desktop_auto_eat_tick_live_plan_reports_missing_game_window() {
        let state = Mutex::new(AutoEatTriggerState::default());
        let error = execute_desktop_auto_eat_tick_live_plan(
            &AppConfig::default(),
            None,
            &state,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoEat live tick requires a detected game window"));
    }

    #[test]
    fn desktop_auto_eat_template_object_preserves_locator_settings() {
        let plan = plan_auto_eat(AutoEatExecutionConfig::default());
        let object =
            desktop_auto_eat_template_object(&plan.locators.recovery_icon, plan.capture_size)
                .unwrap();

        assert_eq!(object.name.as_deref(), Some("RecoveryIcon"));
        assert_eq!(
            object.region_of_interest,
            Some(plan.locators.recovery_icon.roi)
        );
        assert_eq!(
            object.template.threshold,
            plan.locators.recovery_icon.threshold
        );
        assert_eq!(object.template.mode, plan.locators.recovery_icon.match_mode);
        assert_eq!(
            object.template.use_3_channels,
            plan.locators.recovery_icon.use_3_channels
        );
        assert_eq!(
            object.template.draw_on_window,
            plan.locators.recovery_icon.draw_on_window
        );
        assert!(object
            .template
            .template_asset
            .as_ref()
            .is_some_and(|path| path.ends_with("Recovery.png")));
    }

    #[test]
    fn desktop_auto_eat_low_hp_probe_reads_rgb_pixel_from_bgr_capture() {
        let plan = plan_auto_eat(AutoEatExecutionConfig {
            capture_size: VisionSize::new(4, 4),
            ..AutoEatExecutionConfig::default()
        });
        let point = plan.detection_rule.low_hp_pixel_probe.point;
        assert_eq!(point.x, 1);
        assert_eq!(point.y, 2);

        let mut pixels = vec![0u8; 4 * 4 * 3];
        let index = ((point.y as usize * 4) + point.x as usize) * 3;
        pixels[index..index + 3].copy_from_slice(&[90, 90, 255]);
        let image = BgrImage::new(VisionSize::new(4, 4), pixels).unwrap();

        assert!(desktop_auto_eat_current_avatar_low_hp(&image, &plan));
    }

    #[test]
    fn desktop_auto_eat_genshin_action_maps_quick_use_gadget_only() {
        assert_eq!(
            desktop_auto_eat_genshin_action("QuickUseGadget"),
            Some(GenshinAction::QuickUseGadget)
        );
        assert_eq!(
            desktop_auto_eat_genshin_action("GIActions.QuickUseGadget"),
            Some(GenshinAction::QuickUseGadget)
        );
        assert_eq!(desktop_auto_eat_genshin_action("OpenMap"), None);
    }

    #[test]
    fn desktop_auto_pick_tick_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_pick_tick_live_plan(
            &desktop_test_temp_root("auto-pick-missing-window"),
            &AppConfig::default(),
            None,
            0,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoPick live tick requires a detected game window"));
    }

    #[test]
    fn desktop_auto_pick_template_and_relative_locator_preserve_settings() {
        let plan = plan_auto_pick(AutoPickExecutionConfig::default());
        let object =
            desktop_auto_pick_template_object(&plan.template_rule.pick_template, plan.capture_size)
                .unwrap();

        assert_eq!(
            object.name.as_deref(),
            Some(plan.template_rule.pick_template.name.as_str())
        );
        assert_eq!(
            object.region_of_interest,
            Some(plan.template_rule.pick_template.region_of_interest)
        );
        assert_eq!(object.template.threshold, 0.8);
        assert_eq!(
            object.template.draw_on_window,
            plan.template_rule.pick_template.draw_on_window
        );
        assert!(object
            .template
            .template_asset
            .as_ref()
            .is_some_and(|path| path.ends_with("F.png")));

        let relative = desktop_auto_pick_relative_template_locator(
            &plan.template_rule.chat_icon_template,
            Rect::new(100, 200, 60, 40).unwrap(),
            plan.capture_size,
        )
        .unwrap();

        assert_eq!(relative.name, "ChatIcon");
        assert_eq!(relative.asset, bgi_task::AUTO_PICK_CHAT_ICON_ASSET);
        assert_eq!(
            relative.region_of_interest,
            Rect::new(160, 200, 55, 40).unwrap()
        );
        assert!(!relative.draw_on_window);
    }

    #[test]
    fn desktop_auto_pick_scroll_icon_detects_all_probe_points() {
        let plan = plan_auto_pick(AutoPickExecutionConfig::default());
        let mut pixels = vec![0u8; (1920 * 1080 * 3) as usize];
        for probe in &plan.scroll_rule.probe_points {
            let index = ((probe.y_1080p as usize * 1920) + probe.x_1080p as usize) * 3;
            pixels[index..index + 3].copy_from_slice(&[probe.rgb.b, probe.rgb.g, probe.rgb.r]);
        }
        let image = BgrImage::new(VisionSize::new(1920, 1080), pixels).unwrap();

        assert!(desktop_auto_pick_scroll_icon_detected(&image, &plan));
    }

    #[test]
    fn desktop_auto_pick_virtual_key_maps_ascii_and_common_keys() {
        assert_eq!(desktop_auto_pick_virtual_key("F"), Some(KeyId::F.vk()));
        assert_eq!(desktop_auto_pick_virtual_key("g"), Some(KeyId::G.vk()));
        assert_eq!(
            desktop_auto_pick_virtual_key("Space"),
            Some(KeyId::SPACE.vk())
        );
        assert_eq!(
            desktop_auto_pick_virtual_key("Esc"),
            Some(KeyId::ESCAPE.vk())
        );
        assert_eq!(desktop_auto_pick_virtual_key("MouseLeft"), None);
    }

    #[test]
    fn desktop_auto_pick_list_readers_parse_text_and_json_files() {
        let root = desktop_test_temp_root("auto-pick-lists");
        fs::create_dir_all(root.join("User")).unwrap();
        fs::create_dir_all(root.join("Assets/Config/Pick")).unwrap();
        fs::write(root.join("User/pick_white_lists.txt"), "晶蝶\r\n甜甜花\n").unwrap();
        fs::write(
            root.join("Assets/Config/Pick/default_pick_black_lists.json"),
            r#"{"base":["调查","对话"],"nested":{"one":"长时间"}}"#,
        )
        .unwrap();

        assert_eq!(
            desktop_auto_pick_read_text_list(&root, "User/pick_white_lists.txt").unwrap(),
            vec!["晶蝶".to_string(), "甜甜花".to_string()]
        );
        let mut default_black = desktop_auto_pick_read_json_string_list(
            &root,
            "Assets/Config/Pick/default_pick_black_lists.json",
        )
        .unwrap();
        default_black.sort();
        assert_eq!(
            default_black,
            vec!["对话".to_string(), "调查".to_string(), "长时间".to_string()]
        );
        assert!(desktop_auto_pick_read_text_list(&root, "User/missing.txt")
            .unwrap()
            .is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_auto_fish_tick_live_plan_reports_missing_game_window() {
        let state = Mutex::new(AutoFishTriggerState::default());
        let error = execute_desktop_auto_fish_tick_live_plan(
            &AppConfig::default(),
            None,
            &state,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoFish live tick requires a detected game window"));
    }

    #[test]
    fn desktop_auto_fish_tick_live_requires_bit_blt_before_capture() {
        let mut config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        config.auto_fishing_config.enabled = true;
        let window = desktop_test_game_window(1920, 1080);
        let plan = plan_auto_fish(AutoFishExecutionConfig {
            capture_size: VisionSize::new(1920, 1080),
            auto_fishing_config: config.auto_fishing_config.clone(),
        });
        let mut state = AutoFishTriggerState::default();

        let error = execute_desktop_auto_fish_tick_live(
            &config,
            &window,
            &plan,
            &mut state,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoFish live tick requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_auto_fish_input_events_preserve_left_button_actions() {
        assert_eq!(
            desktop_auto_fish_input_events(AutoFishInputAction::LeftButtonClick),
            vec![
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
            ]
        );
        assert_eq!(
            desktop_auto_fish_input_events(AutoFishInputAction::LeftButtonDown),
            vec![InputEvent::MouseButtonDown {
                button: MouseButton::Left,
            }]
        );
        assert_eq!(
            desktop_auto_fish_input_events(AutoFishInputAction::LeftButtonUp),
            vec![InputEvent::MouseButtonUp {
                button: MouseButton::Left,
            }]
        );
    }

    #[test]
    fn desktop_realtime_trigger_live_result_writes_auto_fish_execution_error() {
        let task_state = DesktopTaskRuntimeState::default();
        let mut config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        config.auto_fishing_config.enabled = true;
        let window = desktop_test_game_window(1920, 1080);
        let mut result = bgi_task::evaluate_task_invocation_plan(
            bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
                bgi_task::ScriptDispatcherCommandInput::AddRealtimeTimer(
                    bgi_task::DispatcherTimerInput {
                        name: "AutoFish".to_string(),
                        interval_ms: 67,
                        config: Some(serde_json::json!({
                            "autoFishingConfig": {
                                "enabled": true
                            }
                        })),
                        clears_existing_triggers: false,
                    },
                ),
            )
            .unwrap(),
            TaskInvocationExecutionMode::ExecuteReady,
        );

        execute_desktop_realtime_trigger_live_result(
            Path::new("."),
            &config,
            Some(&window),
            &task_state,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("AutoFish live tick requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_quick_teleport_tick_live_plan_reports_missing_game_window() {
        let state = Mutex::new(DesktopQuickTeleportTriggerState::default());
        let error = execute_desktop_quick_teleport_tick_live_plan(
            &AppConfig::default(),
            None,
            &state,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("QuickTeleport live tick requires a detected game window"));
    }

    #[test]
    fn desktop_quick_teleport_template_object_preserves_locator_settings() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let object = desktop_quick_teleport_template_object(
            &plan.locators.teleport_button,
            plan.capture_size,
        )
        .unwrap();

        assert_eq!(
            object.name.as_deref(),
            Some(plan.locators.teleport_button.name.as_str())
        );
        assert_eq!(
            object.region_of_interest,
            Some(plan.locators.teleport_button.roi)
        );
        assert_eq!(
            object.template.threshold,
            plan.locators.teleport_button.threshold
        );
        assert_eq!(
            object.template.mode,
            plan.locators.teleport_button.match_mode
        );
        assert_eq!(
            object.template.use_3_channels,
            plan.locators.teleport_button.use_3_channels
        );
        assert!(object
            .template
            .template_asset
            .as_ref()
            .is_some_and(|path| path.ends_with("GoTeleport.png")));

        let multi = desktop_quick_teleport_template_object(
            &plan.locators.map_choose_icon_templates[0],
            plan.capture_size,
        )
        .unwrap();
        assert_eq!(multi.template.max_match_count, 32);
    }

    #[test]
    fn desktop_quick_teleport_state_preserves_legacy_throttle_gate() {
        let mut config = AppConfig::default();
        config.quick_teleport_config.enabled = true;
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig {
            quick_teleport_config: config.quick_teleport_config,
            ..QuickTeleportExecutionConfig::default()
        });
        let mut state = DesktopQuickTeleportTriggerState::default();

        let first = state.elapsed_since_previous_tick_ms(&plan, true);
        let second = state.elapsed_since_previous_tick_ms(&plan, true);

        assert!(first > plan.throttle_rule.tick_interval_ms);
        assert!(second <= plan.throttle_rule.tick_interval_ms);
    }

    #[test]
    fn desktop_quick_teleport_hotkey_gate_uses_configured_hold_state() {
        let no_hotkey = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let mut latch = DesktopQuickTeleportHotkeyLatchState::default();
        assert!(desktop_quick_teleport_hotkey_pressed(
            &no_hotkey,
            false,
            true,
            &[],
            &mut latch,
        ));

        let mut config = AppConfig::default();
        config.quick_teleport_config.hotkey_tp_enabled = true;
        let invalid_hotkey = plan_quick_teleport(QuickTeleportExecutionConfig {
            quick_teleport_config: config.quick_teleport_config.clone(),
            quick_teleport_tick_hotkey: Some("not-a-hotkey".to_string()),
            ..QuickTeleportExecutionConfig::default()
        });
        assert!(invalid_hotkey.hotkey_rule.requires_pressed);
        assert!(!desktop_quick_teleport_hotkey_pressed(
            &invalid_hotkey,
            true,
            false,
            &[KeyId::F8.vk()],
            &mut latch,
        ));
        assert!(!desktop_quick_teleport_hotkey_pressed(
            &invalid_hotkey,
            false,
            false,
            &[KeyId::F8.vk()],
            &mut latch,
        ));

        let f8_hotkey = plan_quick_teleport(QuickTeleportExecutionConfig {
            quick_teleport_config: config.quick_teleport_config,
            quick_teleport_tick_hotkey: Some("F8".to_string()),
            ..QuickTeleportExecutionConfig::default()
        });
        assert!(!desktop_quick_teleport_hotkey_pressed(
            &f8_hotkey,
            true,
            false,
            &[],
            &mut latch,
        ));
        assert!(desktop_quick_teleport_hotkey_pressed(
            &f8_hotkey,
            true,
            false,
            &[KeyId::F8.vk()],
            &mut latch,
        ));
        assert!(desktop_quick_teleport_hotkey_pressed(
            &f8_hotkey,
            true,
            true,
            &[KeyId::F8.vk()],
            &mut latch,
        ));
        assert!(!desktop_quick_teleport_hotkey_pressed(
            &f8_hotkey,
            true,
            false,
            &[],
            &mut latch,
        ));
    }

    #[test]
    fn desktop_quick_teleport_hotkey_latch_matches_legacy_down_up_guard_semantics() {
        let mut latch = DesktopQuickTeleportHotkeyLatchState::default();
        let f8 = Some(KeyId::F8.vk());

        assert!(!latch.update(f8, true, true, true));
        assert!(!latch.update(f8, true, true, false));
        assert!(!latch.update(f8, false, true, false));

        assert!(latch.update(f8, true, true, false));
        assert!(latch.update(f8, true, true, true));
        assert!(!latch.update(f8, false, true, true));

        assert!(!latch.update(Some(KeyId::F9.vk()), true, true, false));
        assert!(!latch.update(Some(KeyId::F9.vk()), false, true, false));
        assert!(latch.update(Some(KeyId::F9.vk()), true, true, false));
    }

    #[test]
    fn desktop_quick_teleport_legacy_hold_hotkey_parses_keyboard_and_side_mouse() {
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("F8"),
            Some(KeyId::F8.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("D1"),
            Some(KeyId::D1.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("NumPad1"),
            Some(KeyId::NUM_PAD1.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("Space"),
            Some(KeyId::SPACE.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("OemMinus"),
            Some(KeyId::MINUS.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("XButton1"),
            Some(KeyId::MOUSE_SIDE_BUTTON1.vk())
        );
        assert_eq!(
            desktop_quick_teleport_legacy_hold_hotkey_vk("XButton2"),
            Some(KeyId::MOUSE_SIDE_BUTTON2.vk())
        );
        assert!(desktop_quick_teleport_legacy_hold_hotkey_vk("Ctrl + F8").is_none());
        assert!(desktop_quick_teleport_legacy_hold_hotkey_vk("MouseMiddleButton").is_none());
        assert!(desktop_quick_teleport_legacy_hold_hotkey_vk("< None >").is_none());
    }

    #[test]
    fn desktop_quick_teleport_candidate_rects_use_relative_icon_coordinates() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let roi = plan.locators.map_choose_icon_roi;
        let absolute_icon = Rect::new(roi.x + 12, roi.y + 34, 20, 24).unwrap();

        let relative =
            desktop_quick_teleport_relative_candidate_icon_rect(absolute_icon, &plan).unwrap();
        let text_rect = desktop_quick_teleport_candidate_text_rect(relative, &plan);

        assert_eq!(relative, Rect::new(12, 34, 20, 24).unwrap());
        assert_eq!(text_rect, Rect::new(1302, 126, 200, 40).unwrap());
    }

    #[test]
    fn desktop_quick_teleport_color_range_ocr_image_keeps_standard_white_text() {
        let plan = plan_quick_teleport(QuickTeleportExecutionConfig::default());
        let image =
            BgrImage::new(VisionSize::new(2, 1), vec![249, 250, 251, 10, 255, 255]).unwrap();

        let filtered = desktop_quick_teleport_color_range_ocr_image(&image, &plan).unwrap();

        assert_eq!(filtered.pixels, vec![255, 255, 255, 0, 0, 0]);
    }

    #[test]
    fn desktop_quick_teleport_ocr_text_groups_winrt_words_by_line() {
        let text = desktop_quick_teleport_ocr_text_from_regions(&[
            OcrResultRegion {
                rect: Rect::new(40, 31, 20, 12).unwrap(),
                text: "锚点".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 30, 20, 12).unwrap(),
                text: "璃月".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 70, 20, 12).unwrap(),
                text: "蒙德".to_string(),
                score: 1.0,
            },
        ]);

        assert_eq!(text, "璃月锚点\n蒙德");
    }

    #[test]
    fn desktop_lower_head_then_walk_to_ocr_text_orders_winrt_words() {
        let text = desktop_lower_head_then_walk_to_ocr_text_from_regions(&[
            OcrResultRegion {
                rect: Rect::new(34, 10, 18, 18).unwrap(),
                text: "活".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 11, 18, 18).unwrap(),
                text: "激".to_string(),
                score: 1.0,
            },
        ]);

        assert_eq!(text, "激活");
    }

    #[test]
    fn desktop_chat_ui_hotkey_guard_preserves_legacy_stable_frames_and_prime() {
        let mut guard = DesktopChatUiHotkeyGuardState::default();
        let now = Instant::now();
        let closed = DesktopChatUiDetection {
            state: DesktopChatUiState::Closed,
            has_back_button: false,
            has_more_button: false,
            has_add_conversation_button: false,
            bottom_circle_count: 0,
            has_send_button: false,
        };
        let open = DesktopChatUiDetection {
            state: DesktopChatUiState::InputOpen,
            has_back_button: true,
            has_more_button: false,
            has_add_conversation_button: true,
            bottom_circle_count: 2,
            has_send_button: false,
        };

        guard.update_visual_state(open, now);
        assert!(!guard.should_block_hotkey("QuickTeleportTickHotkey", now));
        guard.update_visual_state(open, now);
        assert!(guard.should_block_hotkey("QuickTeleportTickHotkey", now));
        assert!(!guard.should_block_hotkey("BgiEnabledHotkey", now));

        guard.update_visual_state(closed, now);
        assert!(guard.should_block_hotkey("QuickTeleportTickHotkey", now));
        guard.update_visual_state(closed, now);
        assert!(!guard.should_block_hotkey("QuickTeleportTickHotkey", now));

        guard.prime_from_open_chat_key(&[KeyId::ENTER.vk()], KeyId::ENTER.vk(), now);
        assert!(guard.should_block_hotkey("QuickTeleportTickHotkey", now));
        assert!(
            !guard.should_block_hotkey("QuickTeleportTickHotkey", now + Duration::from_millis(281))
        );
    }

    #[test]
    fn desktop_chat_ui_detection_reads_input_controls_from_synthetic_capture() {
        let mut pixels = vec![0u8; 1920 * 1080 * 3];
        fill_bgr_rect(
            &mut pixels,
            1920,
            Rect::new(20, 880, 40, 40).unwrap(),
            [210, 210, 210],
        );
        fill_bgr_rect(
            &mut pixels,
            1920,
            Rect::new(660, 900, 32, 32).unwrap(),
            [210, 210, 210],
        );
        fill_bgr_rect(
            &mut pixels,
            1920,
            Rect::new(730, 900, 32, 32).unwrap(),
            [210, 210, 210],
        );
        let image = BgrImage::new(VisionSize::new(1920, 1080), pixels).unwrap();
        let capture = ImageRegion::capture(image);

        let detection = desktop_detect_chat_ui(&capture, true);

        assert!(detection.has_back_button);
        assert!(detection.has_add_conversation_button);
        assert_eq!(detection.bottom_circle_count, 2);
        assert_eq!(detection.state, DesktopChatUiState::InputOpen);
    }

    fn fill_bgr_rect(pixels: &mut [u8], width: usize, rect: Rect, bgr: [u8; 3]) {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                let index = ((y as usize * width) + x as usize) * 3;
                pixels[index..index + 3].copy_from_slice(&bgr);
            }
        }
    }

    #[test]
    fn desktop_quick_teleport_deduplicates_overlapping_candidates() {
        let candidates = vec![
            QuickTeleportMapChooseCandidate {
                icon_rect: Rect::new(100, 100, 20, 20).unwrap(),
                text: String::new(),
            },
            QuickTeleportMapChooseCandidate {
                icon_rect: Rect::new(104, 104, 20, 20).unwrap(),
                text: String::new(),
            },
            QuickTeleportMapChooseCandidate {
                icon_rect: Rect::new(100, 150, 20, 20).unwrap(),
                text: String::new(),
            },
        ];

        let deduplicated = desktop_quick_teleport_deduplicate_candidates(candidates);

        assert_eq!(deduplicated.len(), 2);
        assert_eq!(
            deduplicated[0].icon_rect,
            Rect::new(100, 100, 20, 20).unwrap()
        );
        assert_eq!(
            deduplicated[1].icon_rect,
            Rect::new(100, 150, 20, 20).unwrap()
        );
    }

    #[test]
    fn desktop_quick_buy_live_plan_reports_missing_game_window() {
        let error = execute_desktop_quick_buy_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("QuickBuy live execution requires a detected game window"));
    }

    #[test]
    fn desktop_quick_buy_target_capture_point_uses_scaled_plan_coordinates() {
        let plan = plan_quick_buy(QuickBuyExecutionConfig {
            capture_size: VisionSize::new(1280, 720),
        })
        .unwrap();

        assert_eq!(
            quick_buy_target_capture_point(&plan.normal_purchase_rule.final_clicks[0]),
            (1130, 680)
        );
        assert_eq!(
            quick_buy_target_capture_point(&plan.serenitea_pot_purchase_rule.final_clicks[1]),
            (640, 567)
        );
    }

    #[test]
    fn desktop_quick_serenitea_pot_live_plan_reports_missing_game_window() {
        let error = execute_desktop_quick_serenitea_pot_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("QuickSereniteaPot live execution requires a detected game window"));
    }

    #[test]
    fn desktop_quick_serenitea_pot_points_use_scaled_plan_coordinates() {
        let plan = plan_quick_serenitea_pot(QuickSereniteaPotExecutionConfig {
            capture_size: VisionSize::new(1280, 720),
        })
        .unwrap();

        assert_eq!(
            quick_serenitea_pot_screen_point_capture_point(&plan.bag_rule.gadget_tab_click),
            (700, 33)
        );
        assert_eq!(
            quick_serenitea_pot_screen_point_capture_point(&plan.interaction_rule.confirm_click),
            (673, 507)
        );
    }

    #[test]
    fn desktop_quick_serenitea_pot_interaction_text_roi_matches_legacy_offsets() {
        let pick_rect = Rect::new(727, 220, 40, 280).unwrap();
        let roi =
            desktop_quick_serenitea_pot_interaction_text_roi(pick_rect, VisionSize::new(1280, 720))
                .unwrap();

        assert_eq!(roi, Rect::new(804, 220, 190, 280).unwrap());
        assert!(desktop_quick_serenitea_pot_interaction_text_roi(
            Rect::new(1200, 680, 40, 60).unwrap(),
            VisionSize::new(1280, 720)
        )
        .is_none());
    }

    #[test]
    fn desktop_quick_serenitea_pot_ocr_text_groups_winrt_words_by_line() {
        let text = desktop_quick_serenitea_pot_ocr_text_from_regions(&[
            OcrResultRegion {
                rect: Rect::new(45, 11, 34, 16).unwrap(),
                text: "尘歌壶".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 10, 30, 16).unwrap(),
                text: "进入".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 40, 30, 16).unwrap(),
                text: "离开".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(45, 41, 34, 16).unwrap(),
                text: "尘歌壶".to_string(),
                score: 1.0,
            },
        ]);

        assert_eq!(text, "进入尘歌壶\n离开尘歌壶");
    }

    #[test]
    fn desktop_quick_serenitea_pot_interaction_detects_enter_leave_text() {
        let plan = plan_quick_serenitea_pot(QuickSereniteaPotExecutionConfig::default()).unwrap();
        let rule = &plan.interaction_rule;

        assert_eq!(
            desktop_quick_serenitea_pot_interaction_from_text("进入\n尘歌壶", rule),
            QuickSereniteaPotInteractionOutcome::Enter
        );
        assert_eq!(
            desktop_quick_serenitea_pot_interaction_from_text("离开 尘歌壶", rule),
            QuickSereniteaPotInteractionOutcome::Leave
        );
        assert_eq!(
            desktop_quick_serenitea_pot_interaction_from_text("进入 洞天", rule),
            QuickSereniteaPotInteractionOutcome::Missing
        );
    }

    #[test]
    fn desktop_auto_cook_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_cook_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoCook live execution requires a detected game window"));
    }

    #[test]
    fn desktop_auto_track_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_track_live_plan(
            &AppConfig::default(),
            None,
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoTrack live execution requires a detected game window"));
    }

    #[test]
    fn desktop_auto_track_live_plan_requires_bit_blt_capture_backend() {
        let window = desktop_test_game_window(1920, 1080);
        let config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        let error = execute_desktop_auto_track_live_plan(
            &config,
            Some(&window),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoTrack live execution requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_auto_track_ocr_text_groups_winrt_words_by_line() {
        let text = desktop_auto_track_ocr_text_from_regions(&[
            OcrResultRegion {
                rect: Rect::new(85, 10, 20, 16).unwrap(),
                text: "m".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 10, 34, 16).unwrap(),
                text: "距离".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(50, 11, 30, 16).unwrap(),
                text: "123".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 40, 64, 16).unwrap(),
                text: "追踪目标".to_string(),
                score: 1.0,
            },
        ]);

        assert_eq!(text, "距离123m\n追踪目标");
    }

    #[test]
    fn desktop_auto_track_helpers_map_actions_deduplicate_candidates_and_scale_assets() {
        assert_eq!(
            desktop_auto_track_genshin_action("GIActions.OpenQuestMenu"),
            Some(GenshinAction::OpenQuestMenu)
        );
        assert_eq!(
            desktop_auto_track_genshin_action("QuestNavigation"),
            Some(GenshinAction::QuestNavigation)
        );
        assert_eq!(desktop_auto_track_genshin_action("Unknown"), None);
        assert_eq!(
            desktop_auto_track_asset_scale(VisionSize::new(1280, 720)),
            1280.0 / 1920.0
        );

        let candidates = desktop_auto_track_deduplicate_candidates(vec![
            AutoTrackTeleportCandidate {
                asset: "QuickTeleport:TeleportWaypoint.png".to_string(),
                rect: Rect::new(100, 100, 20, 20).unwrap(),
                score: 0.95,
            },
            AutoTrackTeleportCandidate {
                asset: "QuickTeleport:TeleportWaypoint.png".to_string(),
                rect: Rect::new(104, 104, 20, 20).unwrap(),
                score: 0.9,
            },
            AutoTrackTeleportCandidate {
                asset: "QuickTeleport:Domain.png".to_string(),
                rect: Rect::new(300, 300, 20, 20).unwrap(),
                score: 0.8,
            },
        ]);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].rect, Rect::new(100, 100, 20, 20).unwrap());
        assert_eq!(candidates[1].rect, Rect::new(300, 300, 20, 20).unwrap());
    }

    #[test]
    fn desktop_auto_track_template_object_resolves_task_assets_for_1080p() {
        let plan = bgi_task::plan_auto_track(AutoTrackExecutionConfig::default());
        let paimon = desktop_auto_track_template_object(
            &plan.locators.paimon_menu,
            VisionSize::new(1920, 1080),
        )
        .unwrap();
        let blue = desktop_auto_track_template_object(
            &plan.locators.blue_track_point,
            VisionSize::new(1920, 1080),
        )
        .unwrap();
        let teleport = desktop_auto_track_template_object(
            &plan.teleport_rule.map_choose_icon_assets[0],
            VisionSize::new(1920, 1080),
        )
        .unwrap();

        let backend = PureRustVisionBackend::new().with_template_root(bgi_task::task_asset_root());
        let template_paths = [
            paimon.template.template_asset.as_ref().unwrap(),
            blue.template.template_asset.as_ref().unwrap(),
            teleport.template.template_asset.as_ref().unwrap(),
        ];
        for template_path in template_paths {
            assert!(backend
                .template_roots()
                .iter()
                .any(|root| root.join(template_path).is_file()));
        }
    }

    #[test]
    fn desktop_auto_wood_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_wood_live_plan(
            &AppConfig::default(),
            None,
            Some(&serde_json::json!({
                "woodRoundNum": 1,
                "woodDailyMaxCount": 1
            })),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoWood live execution requires a detected game window"));
    }

    #[test]
    fn desktop_auto_wood_bilibili_config_detection_matches_legacy_suffix_rule() {
        assert!(desktop_auto_wood_config_text_has_bilibili_channel_14(
            "channel=14"
        ));
        assert!(desktop_auto_wood_config_text_has_bilibili_channel_14(
            "  channel=14  "
        ));
        assert!(desktop_auto_wood_config_text_has_bilibili_channel_14(
            "[General]\nchannel=14\n"
        ));
        assert!(!desktop_auto_wood_config_text_has_bilibili_channel_14(
            "channel=1"
        ));
        assert!(!desktop_auto_wood_config_text_has_bilibili_channel_14(
            "Channel=14"
        ));
    }

    #[test]
    fn desktop_auto_wood_live_plan_allows_ocr_until_capture_backend() {
        let window = desktop_test_game_window(1920, 1080);
        let config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        let error = execute_desktop_auto_wood_live_plan(
            &config,
            Some(&window),
            Some(&serde_json::json!({
                "woodRoundNum": 1,
                "woodDailyMaxCount": 1,
                "autoWoodConfig": {
                    "woodCountOcrEnabled": true
                }
            })),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoWood live execution requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_auto_wood_live_plan_inherits_app_auto_wood_config() {
        let window = desktop_test_game_window(1920, 1080);
        let mut config = AppConfig::default();
        config.auto_wood_config.wood_count_ocr_enabled = true;
        config.capture_mode = bgi_core::CaptureMode::WindowsGraphicsCapture;
        let error = execute_desktop_auto_wood_live_plan(
            &config,
            Some(&window),
            Some(&serde_json::json!({
                "woodRoundNum": 1,
                "woodDailyMaxCount": 1
            })),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoWood live execution requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_auto_wood_ocr_text_groups_winrt_words_by_line() {
        let text = desktop_auto_wood_ocr_text_from_regions(&[
            OcrResultRegion {
                rect: Rect::new(80, 42, 20, 16).unwrap(),
                text: "30".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 10, 32, 16).unwrap(),
                text: "获得".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(10, 42, 34, 16).unwrap(),
                text: "竹节".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(48, 43, 14, 14).unwrap(),
                text: "×".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(11, 74, 34, 16).unwrap(),
                text: "杉木".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(50, 75, 14, 14).unwrap(),
                text: "x".to_string(),
                score: 1.0,
            },
            OcrResultRegion {
                rect: Rect::new(81, 74, 20, 16).unwrap(),
                text: "20".to_string(),
                score: 1.0,
            },
        ]);

        assert_eq!(text, "获得\n竹节×30\n杉木x20");
    }

    #[test]
    fn desktop_auto_wood_live_plan_respects_top_level_auto_wood_task_config() {
        let window = desktop_test_game_window(1920, 1080);
        let mut config = AppConfig::default();
        config.auto_wood_config.wood_count_ocr_enabled = true;
        config.capture_mode = bgi_core::CaptureMode::WindowsGraphicsCapture;
        let error = execute_desktop_auto_wood_live_plan(
            &config,
            Some(&window),
            Some(&serde_json::json!({
                "woodRoundNum": 1,
                "woodDailyMaxCount": 1,
                "woodCountOcrEnabled": false
            })),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("AutoWood live execution requires the BitBlt capture backend"));
    }

    #[test]
    fn desktop_auto_wood_asset_scale_tracks_capture_width() {
        assert_eq!(
            desktop_auto_wood_asset_scale(VisionSize::new(1920, 1080)),
            1.0
        );
        assert!(
            (desktop_auto_wood_asset_scale(VisionSize::new(1280, 720)) - (2.0 / 3.0)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn desktop_script_dispatcher_live_plan_reports_auto_cook_missing_game_window() {
        let plan = ScriptDispatcherExecutionPlan::AutoCook(plan_auto_cook(
            AutoCookExecutionConfig::default(),
        ));
        let error = execute_desktop_script_dispatcher_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoCook live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_script_dispatcher_live_executes_auto_eat_food_missing_default_branch() {
        let plan = bgi_task::plan_auto_eat_food(
            bgi_task::AutoEatFoodExecutionConfig::from_value(Some(&serde_json::json!({
                "foodEffectType": "AdventurersDish",
                "autoEatConfig": {
                    "defaultAdventurersDishName": null
                }
            })))
            .unwrap(),
        )
        .unwrap();

        let result = execute_desktop_script_dispatcher_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &ScriptDispatcherExecutionPlan::AutoEatFood(plan),
        )
        .unwrap()
        .unwrap();

        let ScriptDispatcherLiveExecutionReport::AutoEatFood(report) = result else {
            panic!("expected AutoEatFood live report");
        };
        assert!(!report.completed);
        let decision = report.state.decision.unwrap();
        assert_eq!(
            decision.outcome,
            bgi_task::AutoEatFoodUseOutcome::MissingDefaultSkipped
        );
        assert!(report.state.vision_drawings_cleared);
        assert!(report
            .executed_actions
            .iter()
            .any(|action| action.action_kind == bgi_task::AutoEatFoodRuntimeActionKind::Skip));
    }

    #[test]
    fn desktop_script_dispatcher_live_rejects_auto_eat_food_inventory_branch_until_adapters_exist()
    {
        let plan = bgi_task::plan_auto_eat_food(
            bgi_task::AutoEatFoodExecutionConfig::from_value(Some(&serde_json::json!({
                "foodName": "甜甜花酿鸡"
            })))
            .unwrap(),
        )
        .unwrap();

        let error = execute_desktop_script_dispatcher_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &ScriptDispatcherExecutionPlan::AutoEatFood(plan.clone()),
        )
        .unwrap_err();

        let TaskError::CommonJobExecution(message) = error else {
            panic!("expected AutoEatFood common-job execution error");
        };
        assert!(message
            .contains("AutoEatFood inventory-food live execution requires a detected game window"));
        assert!(!message.contains("inventory grid/ONNX/OCR/click adapters"));

        let window = desktop_test_game_window(1920, 1080);
        let error = execute_desktop_script_dispatcher_live_plan(
            &AppConfig::default(),
            Some(&window),
            Arc::new(InputCancellationToken::new()),
            &ScriptDispatcherExecutionPlan::AutoEatFood(plan),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoEatFood inventory-food live execution requires desktop inventory-opening adapter")
                    && !message.contains("inventory grid/ONNX/OCR/click adapters")
        ));
    }

    #[test]
    fn desktop_inventory_live_preflight_reports_shared_capture_boundaries() {
        let count_plan = bgi_task::plan_common_job(
            bgi_task::COUNT_INVENTORY_ITEM_TASK_KEY,
            Some(&serde_json::json!({
                "gridScreenName": "Materials",
                "itemName": "晶核"
            })),
        )
        .unwrap()
        .unwrap();
        let CommonJobExecutionPlan::CountInventoryItem(count_plan) = count_plan else {
            panic!("expected CountInventoryItem plan");
        };
        let small_window = desktop_test_game_window(1280, 720);

        let count_error = execute_desktop_count_inventory_item_live(
            &AppConfig::default(),
            &small_window,
            &count_plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(count_error.contains(
            "CountInventoryItem live execution requires plan capture size 1920x1080 to match current capture size 1280x720"
        ));
        assert!(!count_error.contains("inventory grid/ONNX/OCR/click adapters"));

        let wgc_config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };
        let bit_blt_error = execute_desktop_count_inventory_item_live(
            &wgc_config,
            &desktop_test_game_window(1920, 1080),
            &count_plan,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(bit_blt_error
            .contains("CountInventoryItem live execution requires the BitBlt capture backend"));
        assert!(!bit_blt_error.contains("inventory grid/ONNX/OCR/click adapters"));

        let auto_eat_plan = bgi_task::plan_auto_eat_food(
            bgi_task::AutoEatFoodExecutionConfig::from_value(Some(&serde_json::json!({
                "foodName": "甜甜花酿鸡"
            })))
            .unwrap(),
        )
        .unwrap();
        let auto_eat_error = execute_desktop_script_dispatcher_live_plan(
            &AppConfig::default(),
            Some(&small_window),
            Arc::new(InputCancellationToken::new()),
            &ScriptDispatcherExecutionPlan::AutoEatFood(auto_eat_plan),
        )
        .unwrap_err();

        assert!(matches!(
            auto_eat_error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoEatFood inventory-food live execution requires plan capture size 1920x1080 to match current capture size 1280x720")
                    && !message.contains("inventory grid/ONNX/OCR/click adapters")
        ));
    }

    #[test]
    fn desktop_count_inventory_item_preflight_reports_next_adapter_boundary() {
        let count_plan = bgi_task::plan_common_job(
            bgi_task::COUNT_INVENTORY_ITEM_TASK_KEY,
            Some(&serde_json::json!({
                "gridScreenName": "Materials",
                "itemName": "晶核"
            })),
        )
        .unwrap()
        .unwrap();
        let CommonJobExecutionPlan::CountInventoryItem(plan) = count_plan else {
            panic!("expected CountInventoryItem plan");
        };

        let open_error = desktop_count_inventory_item_live_preflight(&plan).unwrap_err();
        assert!(open_error.contains(
            "CountInventoryItem live execution requires desktop inventory-opening adapter"
        ));
        assert!(!open_error.contains("grid/ONNX/OCR/click adapters"));

        let mut after_open = plan.clone();
        after_open.steps.retain(|step| {
            !matches!(
                step.action,
                CountInventoryItemStepAction::GenshinAction {
                    action: GenshinAction::OpenInventory
                }
            )
        });
        let prompt_error = desktop_count_inventory_item_live_preflight(&after_open).unwrap_err();
        assert!(prompt_error.contains(
            "CountInventoryItem live execution requires desktop expired-item prompt adapter"
        ));

        let mut after_prompt = after_open.clone();
        after_prompt.steps.retain(|step| {
            !matches!(
                step.action,
                CountInventoryItemStepAction::ConfirmExpiredItemPrompt { .. }
            )
        });
        let tab_error = desktop_count_inventory_item_live_preflight(&after_prompt).unwrap_err();
        assert!(tab_error
            .contains("CountInventoryItem live execution requires desktop inventory tab adapter"));

        let mut after_tab = after_prompt.clone();
        after_tab.steps.retain(|step| {
            !matches!(
                step.action,
                CountInventoryItemStepAction::OpenInventoryTab { .. }
            )
        });
        let classifier_error = desktop_count_inventory_item_live_preflight(&after_tab).unwrap_err();
        assert!(classifier_error.contains(
            "CountInventoryItem live execution requires desktop GridIcon ONNX/prototype adapter"
        ));
    }

    #[test]
    fn desktop_auto_eat_food_inventory_preflight_reports_count_inventory_boundary() {
        let plan = bgi_task::plan_auto_eat_food(
            bgi_task::AutoEatFoodExecutionConfig::from_value(Some(&serde_json::json!({
                "foodName": "甜甜花酿鸡"
            })))
            .unwrap(),
        )
        .unwrap();

        let open_error = desktop_auto_eat_food_inventory_live_preflight(&plan).unwrap_err();
        assert!(open_error.contains(
            "AutoEatFood inventory-food live execution requires desktop inventory-opening adapter"
        ));
        assert!(!open_error.contains("grid/ONNX/OCR/click adapters"));

        let mut after_open = plan.clone();
        after_open
            .inventory_plan
            .as_mut()
            .unwrap()
            .steps
            .retain(|step| {
                !matches!(
                    step.action,
                    CountInventoryItemStepAction::GenshinAction {
                        action: GenshinAction::OpenInventory
                    }
                )
            });
        let prompt_error = desktop_auto_eat_food_inventory_live_preflight(&after_open).unwrap_err();
        assert!(prompt_error.contains(
            "AutoEatFood inventory-food live execution requires desktop expired-item prompt adapter"
        ));

        let mut after_prompt = after_open.clone();
        after_prompt
            .inventory_plan
            .as_mut()
            .unwrap()
            .steps
            .retain(|step| {
                !matches!(
                    step.action,
                    CountInventoryItemStepAction::ConfirmExpiredItemPrompt { .. }
                )
            });
        let tab_error = desktop_auto_eat_food_inventory_live_preflight(&after_prompt).unwrap_err();
        assert!(tab_error.contains(
            "AutoEatFood inventory-food live execution requires desktop inventory tab adapter"
        ));

        let mut after_tab = after_prompt.clone();
        after_tab
            .inventory_plan
            .as_mut()
            .unwrap()
            .steps
            .retain(|step| {
                !matches!(
                    step.action,
                    CountInventoryItemStepAction::OpenInventoryTab { .. }
                )
            });
        let classifier_error =
            desktop_auto_eat_food_inventory_live_preflight(&after_tab).unwrap_err();
        assert!(classifier_error.contains(
            "AutoEatFood inventory-food live execution requires desktop GridIcon ONNX/prototype adapter"
        ));

        let mut after_inventory_scan = after_tab.clone();
        after_inventory_scan
            .inventory_plan
            .as_mut()
            .unwrap()
            .steps
            .retain(|step| {
                !matches!(
                    step.action,
                    CountInventoryItemStepAction::LoadGridIconClassifier { .. }
                        | CountInventoryItemStepAction::EnumerateGridItems { .. }
                        | CountInventoryItemStepAction::CropGridIcon { .. }
                        | CountInventoryItemStepAction::InferGridIcon { .. }
                        | CountInventoryItemStepAction::OcrGridItemCount { .. }
                )
            });
        let click_error =
            desktop_auto_eat_food_inventory_live_preflight(&after_inventory_scan).unwrap_err();
        assert!(click_error.contains(
            "AutoEatFood inventory-food live execution requires desktop matched-food click adapter"
        ));

        let mut after_click = after_inventory_scan.clone();
        after_click.steps.retain(|step| {
            !matches!(
                step.action,
                AutoEatFoodStepAction::ClickMatchedFoodItem { .. }
            )
        });
        let confirm_error =
            desktop_auto_eat_food_inventory_live_preflight(&after_click).unwrap_err();
        assert!(confirm_error.contains(
            "AutoEatFood inventory-food live execution requires desktop white-confirm click adapter"
        ));
    }

    #[test]
    fn desktop_auto_eat_food_inventory_preflight_requires_bit_blt_capture_backend() {
        let plan = bgi_task::plan_auto_eat_food(
            bgi_task::AutoEatFoodExecutionConfig::from_value(Some(&serde_json::json!({
                "foodName": "甜甜花酿鸡"
            })))
            .unwrap(),
        )
        .unwrap();
        let window = desktop_test_game_window(1920, 1080);
        let config = AppConfig {
            capture_mode: bgi_core::CaptureMode::WindowsGraphicsCapture,
            ..AppConfig::default()
        };

        let error = execute_desktop_script_dispatcher_live_plan(
            &config,
            Some(&window),
            Arc::new(InputCancellationToken::new()),
            &ScriptDispatcherExecutionPlan::AutoEatFood(plan),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoEatFood inventory-food live execution requires the BitBlt capture backend")
                    && !message.contains("inventory grid/ONNX/OCR/click adapters")
        ));
    }

    #[test]
    fn desktop_script_dispatcher_live_result_writes_auto_cook_execution_error() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoCook".to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );

        execute_desktop_script_dispatcher_live_result(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(!result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("AutoCook live execution requires a detected game window"));
        assert!(result.script_dispatcher_live_execution.is_none());
    }

    #[test]
    fn desktop_script_dispatcher_live_result_writes_auto_eat_food_skip_report() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: bgi_task::AUTO_EAT_FOOD_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "foodEffectType": "AdventurersDish",
                    "autoEatConfig": {
                        "defaultAdventurersDishName": null
                    }
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );

        execute_desktop_script_dispatcher_live_result(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Ready);
        assert!(result
            .message
            .contains("AutoEatFood live execution completed"));
        let Some(ScriptDispatcherLiveExecutionReport::AutoEatFood(report)) =
            result.script_dispatcher_live_execution
        else {
            panic!("expected AutoEatFood script-dispatcher live report");
        };
        assert!(matches!(
            report.state.decision.map(|decision| decision.outcome),
            Some(bgi_task::AutoEatFoodUseOutcome::MissingDefaultSkipped)
        ));
    }

    #[test]
    fn desktop_realtime_trigger_live_result_writes_auto_pick_execution_error() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::AddRealtimeTimer(
                bgi_task::DispatcherTimerInput {
                    name: bgi_task::AUTO_PICK_TASK_KEY.to_string(),
                    interval_ms: 250,
                    config: Some(serde_json::json!({})),
                    clears_existing_triggers: false,
                },
            ),
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );
        let task_state = DesktopTaskRuntimeState::default();

        execute_desktop_realtime_trigger_live_result(
            Path::new("."),
            &AppConfig::default(),
            None,
            &task_state,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(!result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("AutoPick live tick requires a detected game window"));
        assert!(result.realtime_trigger_live_execution.is_none());
    }

    #[test]
    fn desktop_realtime_trigger_live_result_writes_quick_teleport_execution_error() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::AddRealtimeTimer(
                bgi_task::DispatcherTimerInput {
                    name: bgi_task::QUICK_TELEPORT_TASK_KEY.to_string(),
                    interval_ms: 250,
                    config: Some(serde_json::json!({})),
                    clears_existing_triggers: false,
                },
            ),
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );
        let task_state = DesktopTaskRuntimeState::default();

        execute_desktop_realtime_trigger_live_result(
            Path::new("."),
            &AppConfig::default(),
            None,
            &task_state,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(!result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("QuickTeleport live tick requires a detected game window"));
        assert!(result.realtime_trigger_live_execution.is_none());
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_use_redeem_code_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: USE_REDEEM_CODE_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "codes": ["ABCD1234EFGH"]
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("UseRedeemCode live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_result_writes_use_redeem_code_execution_error() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: USE_REDEEM_CODE_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "codes": ["ABCD1234EFGH"]
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );

        execute_desktop_independent_task_live_result(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(!result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("UseRedeemCode live execution requires a detected game window"));
        assert!(result.independent_task_live_execution.is_none());
    }

    #[test]
    fn desktop_independent_task_live_plan_rejects_use_redeem_code_without_codes() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: USE_REDEEM_CODE_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("requires at least one redeem code")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_pathing_missing_game_window() {
        let root = desktop_test_temp_root("auto-pathing-missing-window");
        write_desktop_auto_pathing_log_route(&root);
        let plan = desktop_auto_pathing_log_route_invocation_plan();

        let error = execute_desktop_independent_task_live_plan(
            &root,
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoPathing action boundary live execution requires a detected game window")
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_independent_task_live_plan_rejects_auto_pathing_non_16_9_window() {
        let root = desktop_test_temp_root("auto-pathing-non-16-9");
        write_desktop_auto_pathing_log_route(&root);
        let plan = desktop_auto_pathing_log_route_invocation_plan();
        let window = desktop_test_game_window(1024, 768);

        let error = execute_desktop_independent_task_live_plan(
            &root,
            &AppConfig::default(),
            Some(&window),
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message) if message.contains("不是 16:9")
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_independent_task_live_plan_runs_auto_pathing_action_boundary() {
        let root = desktop_test_temp_root("auto-pathing-action-boundary");
        write_desktop_auto_pathing_log_route(&root);
        let plan = desktop_auto_pathing_log_route_invocation_plan();
        let window = desktop_test_game_window(1920, 1080);

        let live_report = execute_desktop_independent_task_live_plan(
            &root,
            &AppConfig::default(),
            Some(&window),
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap()
        .expect("expected AutoPathing live boundary report");

        assert_eq!(live_report.task_name(), "AutoPathing");
        assert!(!live_report.completed());
        let IndependentTaskLiveExecutionReport::AutoPathingActionBoundary(report) = live_report
        else {
            panic!("expected AutoPathing action-boundary report");
        };

        assert!(report.boundary_completed);
        assert!(!report.native_pathing_completed);
        assert_eq!(report.executed_actions, 0);
        assert_eq!(report.invalid_actions, 0);
        assert!(report.unsupported_phases > 0);
        let action_report = report
            .waypoint_reports
            .iter()
            .find_map(|waypoint| waypoint.action_report.as_ref())
            .expect("expected log_output action report");
        assert_eq!(
            action_report.status,
            bgi_task::PathingBoundaryStatus::Reported
        );
        assert!(action_report.message.contains("desktop live route reached"));
        assert!(report
            .waypoint_reports
            .iter()
            .flat_map(|waypoint| waypoint.phase_reports.iter())
            .any(|phase| {
                phase.phase == bgi_core::PathingWaypointPhase::MoveTo
                    && phase.status == bgi_task::PathingBoundaryStatus::Unsupported
            }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_fight_full_loop_adapter_gap() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoFight".to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();

        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoFight full fight-loop live execution requires native CombatScenes")
                    && message.contains("mode=finishProbe")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_fight_finish_probe_missing_game_window() {
        let root = desktop_test_temp_root("auto-fight-finish-probe-missing-window");
        write_desktop_auto_fight_strategy(&root);
        let plan = desktop_auto_fight_finish_probe_invocation_plan();

        let error = execute_desktop_independent_task_live_plan(
            &root,
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoFight finish probe live execution requires a detected game window")
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_auto_fight_finish_probe_helper_keeps_fight_loop_incomplete() {
        let root = desktop_test_temp_root("auto-fight-finish-probe-helper");
        write_desktop_auto_fight_strategy(&root);
        let auto_fight_config = AutoFightExecutionConfig::from_value(Some(&serde_json::json!({
            "strategyName": "daily",
            "teamNames": "钟离,夜兰,行秋,班尼特"
        })));
        let auto_fight_plan = plan_auto_fight(&root, auto_fight_config.param).unwrap();
        let window = desktop_test_game_window(1920, 1080);

        let report = execute_desktop_auto_fight_finish_probe_live_plan_with_capture(
            Some(&window),
            &auto_fight_plan,
            AutoFightFinishDetectionExecutionMode::PlanOnly,
            Arc::new(InputCancellationToken::new()),
            || Ok(desktop_auto_fight_finished_frame()),
        )
        .unwrap();

        assert!(report.captured);
        assert!(!report.dispatched);
        assert_eq!(report.dispatched_events, 0);
        assert!(report
            .detection
            .as_ref()
            .is_some_and(|detection| detection.finished));
        let live_report = IndependentTaskLiveExecutionReport::AutoFightFinishProbe(report);
        assert_eq!(live_report.task_name(), "AutoFight:FinishProbe");
        assert!(!live_report.completed());
        assert_eq!(live_report.executed_steps(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_artifact_salvage_adapter_gap() {
        let plan = desktop_independent_task_invocation_plan("AutoArtifactSalvage");

        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoArtifactSalvage desktop live adapter remains pending")
                    && message.contains("capture")
                    && message.contains("OCR")
                    && message.contains("OpenCV")
                    && message.contains("ONNX")
                    && message.contains("input/click")
                    && message.contains("overlay")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_get_grid_icons_adapter_gap() {
        let plan = desktop_independent_task_invocation_plan("GetGridIcons");

        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("GetGridIcons desktop live adapter remains pending")
                    && message.contains("capture")
                    && message.contains("OpenCV enumeration")
                    && message.contains("Paddle OCR")
                    && message.contains("filesystem")
                    && message.contains("overlay")
                    && message.contains("ONNX")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_orchestration_adapter_gaps() {
        let cases: [(&str, &[&str]); 6] = [
            (
                AUTO_DOMAIN_TASK_KEY,
                &["capture", "map teleport", "CombatScenes", "YOLO"],
            ),
            (
                AUTO_GENIUS_INVOKATION_TASK_KEY,
                &["capture", "card recognition", "duel-state"],
            ),
            (
                AUTO_TRACK_PATH_TASK_KEY,
                &["TpTask", "mini-map capture", "orientation"],
            ),
            (
                AUTO_BOSS_TASK_KEY,
                &["capture", "pathing/key-mouse", "CombatScenes"],
            ),
            (
                AUTO_LEY_LINE_OUTCROP_TASK_KEY,
                &["TpTask/PathExecutor", "CombatScenes", "ScanPickTask"],
            ),
            (
                AUTO_STYGIAN_ONSLAUGHT_TASK_KEY,
                &["combat command loop", "reward/resin", "artifact-salvage"],
            ),
        ];

        for (task_key, expected_terms) in cases {
            let plan = desktop_independent_task_invocation_plan(task_key);

            let error = execute_desktop_independent_task_live_plan(
                Path::new("."),
                &AppConfig::default(),
                None,
                Arc::new(InputCancellationToken::new()),
                &plan,
            )
            .unwrap_err();

            let TaskError::CommonJobExecution(message) = error else {
                panic!("expected common-job execution gap for {task_key}");
            };
            assert!(
                message.contains(&format!("{task_key} desktop live adapter remains pending")),
                "{message}"
            );
            for expected in expected_terms {
                assert!(message.contains(expected), "{message}");
            }
        }
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_music_performance_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_MUSIC_GAME_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "mode": "performance"
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoMusicGame performance live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_music_album_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_MUSIC_GAME_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "executionMode": "album"
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoMusicGame album live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_rejects_unknown_auto_music_mode() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_MUSIC_GAME_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "mode": "practice"
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("unsupported AutoMusicGame live execution mode: practice")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_quick_buy_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: QUICK_BUY_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("QuickBuy live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_quick_serenitea_pot_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: QUICK_SERENITEA_POT_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("QuickSereniteaPot live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_turn_around_macro_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: TURN_AROUND_MACRO_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("TurnAroundMacro live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_quick_enhance_artifact_macro_missing_game_window()
    {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: QUICK_ENHANCE_ARTIFACT_MACRO_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("QuickEnhanceArtifactMacro live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_macro_hotkey_execution_config_uses_app_macro_config_with_overrides() {
        let mut app_config = AppConfig::default();
        app_config.macro_config.enhance_wait_delay = 250;
        app_config.macro_config.runaround_interval = 77;

        let inherited = desktop_macro_hotkey_execution_config(&app_config, None);
        assert_eq!(inherited.macro_config.enhance_wait_delay, 250);
        assert_eq!(inherited.macro_config.runaround_interval, 77);

        let top_level = desktop_macro_hotkey_execution_config(
            &app_config,
            Some(&serde_json::json!({
                "enhanceWaitDelay": 400
            })),
        );
        assert_eq!(top_level.macro_config.enhance_wait_delay, 400);
        assert_eq!(top_level.macro_config.runaround_interval, 77);

        let nested = desktop_macro_hotkey_execution_config(
            &app_config,
            Some(&serde_json::json!({
                "macroConfig": {
                    "enhanceWaitDelay": 500
                }
            })),
        );
        assert_eq!(nested.macro_config.enhance_wait_delay, 500);
        assert_eq!(nested.macro_config.runaround_interval, 77);
    }

    #[test]
    fn desktop_quick_enhance_artifact_macro_plan_only_dispatches_legacy_capture_events() {
        let window = desktop_test_game_window(1920, 1080);
        let plan = plan_quick_enhance_artifact_macro(MacroHotkeyExecutionConfig::from_value(Some(
            &serde_json::json!({
                "macroConfig": {
                    "enhanceWaitDelay": 250
                }
            }),
        )));
        let (report, executions) = execute_desktop_macro_hotkey_live_with_mode(
            &plan,
            &window,
            GlobalInputDispatchMode::PlanOnly,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap();

        assert!(report.completed);
        assert_eq!(report.executed_steps.len(), 6);
        assert_eq!(report.state.input_actions_dispatched, 3);
        assert_eq!(report.state.waits_dispatched, 2);
        assert_eq!(executions.len(), 5);
        assert!(executions.iter().all(|execution| execution.mode
            == GlobalInputDispatchMode::PlanOnly
            && !execution.dispatched
            && execution.dispatched_events == 0));
        assert_eq!(
            executions[0].events,
            vec![
                InputEvent::MouseMoveAbsolute {
                    x: 1760,
                    y: 770,
                    virtual_desktop: false,
                },
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
            ]
        );
        assert_eq!(
            executions[1].events,
            vec![InputEvent::Delay { milliseconds: 100 }]
        );
        assert_eq!(
            executions[2].events,
            vec![
                InputEvent::MouseMoveAbsolute {
                    x: 1760,
                    y: 1020,
                    virtual_desktop: false,
                },
                InputEvent::MouseButtonDown {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
                InputEvent::MouseButtonUp {
                    button: MouseButton::Left,
                },
                InputEvent::Delay { milliseconds: 50 },
            ]
        );
        assert_eq!(
            executions[3].events,
            vec![InputEvent::Delay { milliseconds: 350 }]
        );
        assert_eq!(
            executions[4].events,
            vec![InputEvent::MouseMoveAbsolute {
                x: 1760,
                y: 770,
                virtual_desktop: false,
            }]
        );
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_open_chest_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_OPEN_CHEST_TASK_KEY.to_string(),
                config: serde_json::json!({}),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoOpenChest live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_plan_reports_auto_wood_missing_game_window() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_WOOD_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "woodRoundNum": 1,
                    "woodDailyMaxCount": 1
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let error = execute_desktop_independent_task_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &plan,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            TaskError::CommonJobExecution(message)
                if message.contains("AutoWood live execution requires a detected game window")
        ));
    }

    #[test]
    fn desktop_independent_task_live_result_writes_auto_wood_execution_error() {
        let plan = bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: AUTO_WOOD_TASK_KEY.to_string(),
                config: serde_json::json!({
                    "woodRoundNum": 1,
                    "woodDailyMaxCount": 1
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap();
        let mut result = bgi_task::evaluate_task_invocation_plan(
            plan,
            TaskInvocationExecutionMode::ExecuteReady,
        );

        execute_desktop_independent_task_live_result(
            Path::new("."),
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
            &mut result,
        );

        assert!(!result.executed);
        assert_eq!(result.status, TaskInvocationExecutionStatus::Invalid);
        assert!(result
            .message
            .contains("AutoWood live execution requires a detected game window"));
        assert!(result.independent_task_live_execution.is_none());
    }

    #[test]
    fn desktop_auto_open_chest_asset_scale_tracks_capture_width() {
        assert_eq!(
            desktop_auto_open_chest_asset_scale(VisionSize::new(1920, 1080)),
            1.0
        );
        assert!(
            (desktop_auto_open_chest_asset_scale(VisionSize::new(1280, 720)) - (2.0 / 3.0)).abs()
                < f64::EPSILON
        );
    }

    fn desktop_test_temp_root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("bettergi-desktop-{name}-{suffix}"))
    }

    fn desktop_auto_pathing_log_route_invocation_plan() -> bgi_task::TaskInvocationPlan {
        bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoPathing".to_string(),
                config: serde_json::json!({
                    "route": "liyue/live_log_route.json"
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap()
    }

    fn desktop_auto_fight_finish_probe_invocation_plan() -> bgi_task::TaskInvocationPlan {
        bgi_task::TaskInvocationPlan::from_script_dispatcher_command(
            bgi_task::ScriptDispatcherCommandInput::RunBuiltinTask {
                name: "AutoFight".to_string(),
                config: serde_json::json!({
                    "mode": "finishProbe",
                    "strategyName": "daily",
                    "teamNames": "钟离,夜兰,行秋,班尼特"
                }),
                uses_linked_cancellation: true,
            },
        )
        .unwrap()
    }

    fn desktop_independent_task_invocation_plan(task_key: &str) -> bgi_task::TaskInvocationPlan {
        bgi_task::TaskInvocationPlan {
            kind: bgi_task::TaskInvocationKind::RunIndependentTask,
            task_key: Some(task_key.to_string()),
            catalog_entry: bgi_task::find_task_catalog_entry(task_key),
            interval_ms: None,
            clears_existing_triggers: false,
            config: Some(serde_json::json!({})),
            uses_linked_cancellation: true,
        }
    }

    fn write_desktop_auto_pathing_log_route(root: &Path) {
        let route_dir = root.join("User").join("AutoPathing").join("liyue");
        fs::create_dir_all(&route_dir).unwrap();
        fs::write(
            route_dir.join("live_log_route.json"),
            r#"{
                "info": { "name": "desktop live log route", "type": "collect", "map_name": "Teyvat" },
                "positions": [
                    { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "walk", "action": "log_output", "action_params": "desktop live route reached" }
                ]
            }"#,
        )
        .unwrap();
    }

    fn write_desktop_auto_fight_strategy(root: &Path) {
        let strategy_dir = root.join("User").join("AutoFight");
        fs::create_dir_all(&strategy_dir).unwrap();
        fs::write(
            strategy_dir.join("daily.txt"),
            "钟离 keypress(e), wait(0.05)\n夜兰 keypress(q)",
        )
        .unwrap();
        let catalog_path = root
            .join("GameTask")
            .join("AutoFight")
            .join("Assets")
            .join("combat_avatar.json");
        fs::create_dir_all(catalog_path.parent().unwrap()).unwrap();
        fs::write(
            catalog_path,
            r#"[
  { "alias": ["钟离", "Zhongli"], "burstCD": 12, "id": "10000030", "name": "钟离", "nameEn": "Zhongli", "skillCD": 4, "skillHoldCD": 12, "weapon": "13" },
  { "alias": ["夜兰", "Yelan"], "burstCD": 18, "id": "10000060", "name": "夜兰", "nameEn": "Yelan", "skillCD": 10, "weapon": "12" },
  { "alias": ["行秋", "Xingqiu"], "burstCD": 20, "id": "10000025", "name": "行秋", "nameEn": "Xingqiu", "skillCD": 21, "weapon": "1" },
  { "alias": ["班尼特", "Bennett"], "burstCD": 15, "id": "10000032", "name": "班尼特", "nameEn": "Bennett", "skillCD": 5, "weapon": "1" }
]"#,
        )
        .unwrap();
    }

    fn desktop_auto_fight_finished_frame() -> BgrImage {
        let size = VisionSize::new(1920, 1080);
        let mut pixels = vec![0; size.width as usize * size.height as usize * 3];
        set_desktop_bgr_pixel(
            &mut pixels,
            size,
            bgi_task::AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
            [40, 220, 230],
        );
        set_desktop_bgr_pixel(
            &mut pixels,
            size,
            bgi_task::AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
            [251, 250, 248],
        );
        BgrImage::new(size, pixels).unwrap()
    }

    fn set_desktop_bgr_pixel(
        pixels: &mut [u8],
        size: VisionSize,
        position: (u32, u32),
        bgr: [u8; 3],
    ) {
        let offset = ((position.1 * size.width + position.0) * 3) as usize;
        pixels[offset..offset + 3].copy_from_slice(&bgr);
    }

    #[test]
    fn desktop_auto_cook_asset_scale_tracks_capture_width() {
        assert_eq!(
            desktop_auto_cook_asset_scale(VisionSize::new(1920, 1080)),
            1.0
        );
        assert!(
            (desktop_auto_cook_asset_scale(VisionSize::new(1280, 720)) - (2.0 / 3.0)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn desktop_auto_cook_template_object_preserves_locator_settings() {
        let plan = plan_auto_cook(AutoCookExecutionConfig::default());
        let object =
            desktop_auto_cook_template_object(&plan.locators.cook_icon, plan.capture_size).unwrap();

        assert_eq!(
            object.name.as_deref(),
            Some(plan.locators.cook_icon.asset.as_str())
        );
        assert_eq!(object.region_of_interest, plan.locators.cook_icon.roi);
        assert_eq!(object.template.threshold, plan.locators.cook_icon.threshold);
        assert_eq!(object.template.mode, plan.locators.cook_icon.match_mode);
        assert_eq!(
            object.template.use_3_channels,
            plan.locators.cook_icon.use_3_channels
        );
        assert_eq!(
            object.template.template_asset,
            Some(
                Path::new("GameTask")
                    .join("Common/Element")
                    .join("Assets")
                    .join("1920x1080")
                    .join("ui_left_top_cook_icon.png")
            )
        );
    }

    fn desktop_test_game_window(width: u32, height: u32) -> GameWindowMatch {
        let bounds = bgi_capture::WindowRect::new(0, 0, width as i32, height as i32);
        GameWindowMatch {
            handle: WindowHandle(1),
            process_id: None,
            process_name: Some("GenshinImpact".to_string()),
            class_name: None,
            title: None,
            kind: bgi_capture::GameWindowMatchKind::ProcessName,
            metrics: Some(bgi_capture::GameWindowMetrics::from_legacy_capture_rect(
                width, height, bounds,
            )),
        }
    }

    #[test]
    fn desktop_auto_music_game_performance_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_music_game_performance_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error
            .contains("AutoMusicGame performance live execution requires a detected game window"));
    }

    #[test]
    fn desktop_auto_music_game_performance_live_rejects_non_16_9_window_before_sampling() {
        let window = desktop_test_game_window(1024, 768);
        let error = execute_desktop_auto_music_game_performance_live_plan(
            &AppConfig::default(),
            Some(&window),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("requires a 16:9 game window"));
    }

    #[test]
    fn desktop_auto_music_game_album_live_plan_reports_missing_game_window() {
        let error = execute_desktop_auto_music_game_album_live_plan(
            &AppConfig::default(),
            None,
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(
            error.contains("AutoMusicGame album live execution requires a detected game window")
        );
    }

    #[test]
    fn desktop_auto_music_game_album_live_rejects_non_16_9_window_before_capture() {
        let window = desktop_test_game_window(1024, 768);
        let error = execute_desktop_auto_music_game_album_live_plan(
            &AppConfig::default(),
            Some(&window),
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("requires a 16:9 game window"));
    }

    #[test]
    fn desktop_auto_music_game_lane_points_use_scaled_capture_coordinates() {
        let capture_size = VisionSize::new(1280, 720);
        let asset_scale = desktop_auto_music_game_asset_scale(capture_size);
        let plan = plan_auto_music_game(AutoMusicGameExecutionConfig {
            capture_size,
            asset_scale,
            ..AutoMusicGameExecutionConfig::default()
        });

        assert!((asset_scale - (2.0 / 3.0)).abs() < f64::EPSILON);
        assert_eq!(
            desktop_auto_music_lane_capture_point(&plan.key_lanes[0], asset_scale),
            (278, 614)
        );
        assert_eq!(
            desktop_auto_music_lane_capture_point(&plan.key_lanes[5], asset_scale),
            (995, 614)
        );
    }

    #[test]
    fn desktop_auto_music_game_album_points_use_scaled_capture_coordinates() {
        let capture_size = VisionSize::new(1280, 720);

        assert_eq!(
            desktop_auto_music_1080p_capture_point(310, 220, capture_size),
            (206, 146)
        );
        assert_eq!(
            desktop_auto_music_1080p_capture_point(480, 600, capture_size),
            (320, 400)
        );
        assert_eq!(
            desktop_auto_music_1080p_capture_point(1400, 600, capture_size),
            (933, 400)
        );
    }

    #[test]
    fn desktop_auto_music_game_template_object_preserves_album_locator_settings() {
        let plan = plan_auto_music_game(AutoMusicGameExecutionConfig::default());
        let object = desktop_auto_music_template_object(
            &plan.locators.album_music_complete,
            VisionSize::new(1280, 720),
        )
        .unwrap();

        assert_eq!(
            object.name.as_deref(),
            Some("AutoMusicGame:album_music_complate.png")
        );
        assert_eq!(
            object.region_of_interest,
            Some(Rect {
                x: 600,
                y: 213,
                width: 66,
                height: 53
            })
        );
        assert_eq!(
            object.template.template_asset,
            Some(
                Path::new("GameTask")
                    .join("AutoMusicGame")
                    .join("Assets")
                    .join("1920x1080")
                    .join("album_music_complate.png")
            )
        );
        assert_eq!(
            object.template.mode,
            plan.locators.album_music_complete.match_mode
        );
        assert_eq!(object.template.threshold, 0.8);
    }

    #[test]
    fn desktop_auto_music_game_template_object_applies_roi_rules() {
        let plan = plan_auto_music_game(AutoMusicGameExecutionConfig::default());
        let object =
            desktop_auto_music_template_object(&plan.locators.btn_list, VisionSize::new(1280, 720))
                .unwrap();

        assert_eq!(
            object.region_of_interest,
            Some(Rect {
                x: 768,
                y: 576,
                width: 512,
                height: 144
            })
        );
    }

    #[test]
    fn desktop_auto_music_game_white_confirm_uses_common_asset() {
        let object = desktop_auto_music_white_confirm_object().unwrap();

        assert_eq!(
            object.name.as_deref(),
            Some("Common/Element:btn_white_confirm.png")
        );
        assert_eq!(object.region_of_interest, None);
        assert_eq!(
            object.template.template_asset,
            Some(
                Path::new("GameTask")
                    .join("Common/Element")
                    .join("Assets")
                    .join("1920x1080")
                    .join("btn_white_confirm.png")
            )
        );
    }

    #[test]
    fn desktop_auto_music_game_album_all_songs_ocr_roi_and_text_match() {
        let roi = desktop_auto_music_album_all_songs_ocr_roi(
            VisionSize::new(1920, 1080),
            Rect {
                x: 0,
                y: 0,
                width: 150,
                height: 120,
            },
        )
        .unwrap();

        assert_eq!(
            roi,
            Rect {
                x: 150,
                y: 0,
                width: 307,
                height: 120
            }
        );
        assert!(desktop_auto_music_ocr_contains_all_songs(&[
            OcrResultRegion {
                rect: Rect::new(0, 0, 20, 20).unwrap(),
                text: "全部歌曲".to_string(),
                score: 0.9,
            }
        ]));
        assert!(!desktop_auto_music_ocr_contains_all_songs(&[
            OcrResultRegion {
                rect: Rect::new(0, 0, 20, 20).unwrap(),
                text: "主题专辑".to_string(),
                score: 0.9,
            }
        ]));
    }

    #[test]
    fn desktop_use_redeem_code_live_plan_reports_missing_game_window() {
        let error = execute_desktop_use_redeem_code_live_plan(
            Path::new("."),
            &AppConfig::default(),
            None,
            vec![RedeemCodeEntry::new("GENSHINGIFT", None).unwrap()],
            Arc::new(InputCancellationToken::new()),
        )
        .unwrap_err();

        assert!(error.contains("UseRedeemCode live execution requires a detected game window"));
    }

    #[test]
    fn desktop_use_redeem_code_ocr_text_matching_supports_text_and_regex_locators() {
        let page = bgi_vision::BvPage {
            capture_size: VisionSize::new(1920, 1080),
            ..bgi_vision::BvPage::default()
        };
        let text_locator = page
            .locator_for_text("前往兑换", None)
            .plan(BvLocatorOperation::WaitFor, Some(100));
        assert!(desktop_redeem_code_text_matches_object(
            &text_locator.recognition_object,
            "前 往 兑 换"
        )
        .unwrap());

        let mut regex_object =
            bgi_vision::RecognitionObject::ocr_match(Rect::new(0, 0, 100, 40).unwrap(), ["unused"]);
        regex_object.ocr.one_contain_match_text.clear();
        regex_object.ocr.regex_match_text = vec![r"兑换(成功|奖励)".to_string()];
        assert!(desktop_redeem_code_text_matches_object(&regex_object, "兑换 成功").unwrap());
    }

    #[test]
    fn desktop_use_redeem_code_ocr_roi_and_match_offsets_regions() {
        let page = bgi_vision::BvPage {
            capture_size: VisionSize::new(1920, 1080),
            ..bgi_vision::BvPage::default()
        };
        let full_screen_locator = page
            .locator_for_text("兑换成功", None)
            .plan(BvLocatorOperation::WaitFor, Some(100));
        assert_eq!(
            desktop_redeem_code_ocr_roi_for_locator(
                VisionSize::new(1920, 1080),
                &full_screen_locator
            )
            .unwrap(),
            Rect::new(0, 0, 1920, 1080).unwrap()
        );

        let roi = Rect::new(900, 100, 500, 400).unwrap();
        let object = page
            .locator_for_text("兑换成功", Some(roi))
            .plan(BvLocatorOperation::WaitFor, Some(100))
            .recognition_object;
        let regions = vec![
            OcrResultRegion {
                rect: Rect::new(30, 20, 40, 16).unwrap(),
                text: "兑换".to_string(),
                score: 0.9,
            },
            OcrResultRegion {
                rect: Rect::new(75, 20, 40, 16).unwrap(),
                text: "成功".to_string(),
                score: 0.8,
            },
        ];
        let matched = desktop_redeem_code_match_ocr_regions(&object, &regions, roi)
            .unwrap()
            .unwrap();

        assert_eq!(matched.rect, Rect::new(930, 120, 85, 16).unwrap());
        assert_eq!(matched.text, "兑换\n成功");
    }
}
