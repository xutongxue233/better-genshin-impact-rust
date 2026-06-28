#![allow(clippy::result_large_err)]

pub mod execution_records;
pub mod farming_plan;
pub mod group;
pub mod host;
pub mod log_parse;
pub mod r#macro;
pub mod manifest;
pub mod pathing_result;
pub mod pathing_skip;
pub mod policy;
pub mod pre_execution;
pub mod project;
pub mod repo;
pub mod repo_bridge;
pub mod runtime;
pub mod script_host;
pub mod settings;

pub use bgi_core::{FarmingPlanConfig, MiyousheDataSupportConfig};
pub use bgi_input::InputCancellationToken;
pub use execution_records::{
    convert_seconds_to_days_up, is_skip_task, is_today_by_boundary, DailyExecutionRecord,
    ExecutionRecord, ExecutionRecordClock, ExecutionRecordProjectRef, ExecutionRecordResult,
    ExecutionRecordSkipDecision, ExecutionRecordStorage, ExecutionRecordStorageError,
    TaskCompletionSkipRuleConfig,
};
pub use farming_plan::{
    daily_farming_data_path, daily_farming_date_key, farming_plan_skip_decision,
    farming_plan_skip_decision_from_pathing_file, read_daily_farming_data, record_farming_session,
    save_daily_farming_data, DailyFarmingData, FarmingPlanDailyTotals, FarmingPlanError,
    FarmingPlanExecutionContext, FarmingPlanRecordOutcome, FarmingPlanResult,
    FarmingPlanSkipDecision, FarmingRecord, FarmingRouteRef,
};
pub use group::{
    add_key_mouse_script_project, add_pathing_script_project, add_script_group_project,
    add_shell_script_project, available_js_script_projects, available_key_mouse_scripts,
    available_pathing_scripts, available_pathing_tree, create_script_group, delete_script_group,
    move_script_group_project, parse_script_group_json, read_script_group_file, read_script_groups,
    remove_script_group_project, rename_script_group, script_group_file_path,
    select_script_group_projects_from_resume, select_script_groups_from_resume,
    update_script_group_project, write_script_group_file, AvailableJsScriptProject,
    AvailableKeyMouseScript, AvailablePathingScript, AvailablePathingTreeNode, ScriptGroup,
    ScriptGroupConfig, ScriptGroupFile, ScriptGroupProject, ScriptGroupProjectPatch,
    ScriptGroupResumePointer, ScriptProjectStatus, ScriptProjectType,
};
pub use host::{
    host_bindings, host_permissions, HostBindingDescriptor, HostBindingKind, HostBindingPortState,
    HostPermission,
};
pub use log_parse::{
    analyze_log_groups, analyze_log_lines, are_dates_in_same_custom_day, convert_seconds_to_time,
    discover_log_files, filter_travel_diary_mora_items, format_number_with_style, involved_months,
    load_log_parse_config, log_parse_config_path, log_parse_custom_day_start,
    merge_log_config_groups, merge_log_config_task_lists, merge_pick_dictionaries,
    number_or_empty_string, parse_bgi_line, parse_log_file_entries, parse_log_files,
    parse_log_lines, safe_read_log_lines, subtract_seconds, write_log_parse_config, LogActionItem,
    LogAnalysisOptions, LogAnalysisReport, LogConfigGroup, LogConfigTask, LogFaultScenario,
    LogFileEntry, LogGameInfo, LogParseConfig, LogParseError, LogParseResult, MoraStatistics,
    MoraStatisticsSummary, ScriptGroupLogParseConfig,
};
pub use manifest::{Author, Manifest, ManifestError};
pub use pathing_result::{
    decide_pathing_result, PathingResultDecision, PathingResultException, PathingRunResult,
};
pub use pathing_skip::{
    is_current_hour_equal, pathing_pre_run_skip_decision, PathingPartySkipConfig,
    PathingPartyTaskCycleConfig, PathingPreRunSkipDecision,
};
pub use policy::{
    http_allowed_urls_hash, script_host_security_summary, NotificationRateLimiter,
    ScriptFilePolicy, ScriptHostPolicyError, ScriptHostSecuritySummary, ScriptHttpPolicy,
    ScriptNotificationPolicy,
};
pub use pre_execution::{
    parse_pre_execution_group_names, plan_pre_execution_priority_projects,
    pre_execution_project_key, try_plan_pre_execution_priority_projects,
    PreExecutionPriorityCandidate, PreExecutionPriorityConfig, PreExecutionPriorityPlan,
};
pub use project::{
    execution_mode_for_code, import_rewrites_for_code, normalize_package_specifier,
    normalized_search_paths, rewrite_script_code, ImportRewrite, ImportedResourceKind,
    LoadedScriptModule, ModuleLoaderPlan, ModuleResolution, ModuleResolutionKind,
    ScriptCodeExecutionMode, ScriptModuleLoader, ScriptProject, ScriptProjectError,
    ScriptProjectLayout, ScriptProjectLoaderSummary,
};
pub use r#macro::{
    KeyMouseMacroError, KeyMouseMacroSummary, KeyMouseScript, KeyMouseScriptInfo, MacroCaptureArea,
    MacroEvent, MacroEventType, MacroPlaybackContext,
};
pub use repo::{
    add_update_markers_to_new_repo, calculate_repo_overlap_ratio, checkout_git_repo_path,
    derive_base_folder_name, execute_file_repo_import, execute_git_repo_update,
    execute_repo_import_with_git, execute_zip_repo_import, expand_top_level_paths,
    first_folder_and_remaining_path, git_update_plan, merge_subscription_paths,
    normalize_subscription_paths, parse_import_uri, read_subscription_file, repo_directory_paths,
    repo_folder_name, resolve_repo_url, sanitize_folder_name, script_import_plan,
    script_repo_channels, script_repo_layout, script_repo_update_plan, write_subscription_file,
    zip_import_plan, ScriptImportPlan, ScriptImportUriPlan, ScriptRepoChannel, ScriptRepoError,
    ScriptRepoGitCheckout, ScriptRepoGitCommandOutput, ScriptRepoGitRunner,
    ScriptRepoGitUpdateExecution, ScriptRepoGitUpdatePlan, ScriptRepoImportExecution,
    ScriptRepoLayout, ScriptRepoPathKind, ScriptRepoPathTarget, ScriptRepoUpdatePlan,
    ScriptRepoZipImportExecution, ScriptRepoZipImportPlan, SystemGitRunner,
    DEFAULT_REPO_FOLDER_NAME, IMPORT_URI_PREFIX, OLD_CENTER_REPO_FOLDER_NAME, REPOS_DIR,
    REPOS_TEMP_DIR, SUBSCRIPTIONS_DIR,
};
pub use repo_bridge::{
    clear_repo_bridge_update, mark_repo_bridge_path_updated, read_repo_bridge_file,
    read_repo_bridge_file_with_git, read_repo_bridge_repo_json, read_repo_bridge_user_config,
    repo_bridge_index_nodes, repo_bridge_index_nodes_from_json, repo_bridge_repo_json_path,
    repo_bridge_subscribed_paths_json, script_repo_bridge_paths, ScriptRepoBridgeFileKind,
    ScriptRepoBridgeFileResponse, ScriptRepoBridgeGuideState, ScriptRepoBridgeIndexNode,
    ScriptRepoBridgePaths, REPO_BRIDGE_NOT_FOUND,
};
pub use runtime::{
    engine_for_project_type, script_engines, script_runtime_summary, PreparedScriptExecution,
    RealtimeTimerPlan, ScriptCancellationKind, ScriptCancellationPolicy, ScriptEngineDescriptor,
    ScriptEngineKind, ScriptEnginePortState, ScriptExecutionPlan, ScriptExecutionStep,
    ScriptHostExecutionRoots, ScriptRuntimeError, ScriptRuntimeState, ScriptRuntimeSummary,
    ScriptSchedule, ScriptScheduleKind, SoloTaskPlan,
};
pub use script_host::{
    genshin_command_to_task_input, virtual_key_code_for_script, AutoPickExternalConfig,
    AvatarRecognitionPlan, CaptureGameRegionExecution, CaptureGameRegionPlan,
    CustomHostFunctionCommand, DispatcherCommand, GameCaptureArea, GameCaptureFrameSource,
    GameMetrics, GenshinCommand, GenshinHost, GlobalInputDispatchMode, GlobalInputExecution,
    GlobalInputHost, HtmlMaskCommand, HtmlMaskHost, HtmlMaskInitialState, HtmlMaskMessage,
    HtmlMaskSnapshot, HtmlMaskWindowPlan, HttpDispatchMode, HttpExecution, HttpHost,
    HttpRequestPlan, HttpResponseRecord, ImageMatReadExecution, ImageMatReadPlan,
    ImageMatResizePlan, ImageMatWriteExecution, ImageMatWritePlan, KeyMouseHookCommand,
    KeyMouseHookDispatch, KeyMouseHookEvent, KeyMouseHookEventKind, KeyMouseHookHost,
    KeyMouseHookListener, KeyMouseHookSnapshot, KeyMouseScriptDispatchMode,
    KeyMouseScriptExecution, KeyMouseScriptHost, KeyMouseScriptRunPlan, KeyMouseScriptSource,
    LimitedFileHost, NotificationDispatchMode, NotificationExecution, PathingScriptExecution,
    PathingScriptHost, PathingScriptRunPlan, PathingScriptSource, RealtimeTimerHostPlan,
    RecordingHttpClient, RecordingNotificationSink, ReqwestScriptHttpClient, ScriptDispatcherHost,
    ScriptHostCall, ScriptHostCallResult, ScriptHostRuntime, ScriptHostRuntimeConfig,
    ScriptHostRuntimeError, ScriptHostTarget, ScriptHttpClient, ScriptLogHost, ScriptLogLevel,
    ScriptLogRecord, ScriptNotificationDelivery, ScriptNotificationHost, ScriptNotificationKind,
    ScriptNotificationRecord, ScriptNotificationSink, ServerTimeHost, SoloTaskHostPlan,
    StrategyFileHost,
};
pub use settings::{
    read_script_settings_document, save_script_group_project_settings, script_settings_summary,
    ScriptGroupSettingsSaveResult, ScriptSettingItem, ScriptSettingKind, ScriptSettingsDocument,
    ScriptSettingsSchema, ScriptSettingsSummary,
};
