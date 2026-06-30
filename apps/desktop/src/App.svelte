<script lang="ts">
  import {
    Activity,
    ArrowDown,
    ArrowUp,
    Bell,
    Bot,
    Cog,
    CornerDownRight,
    Crosshair,
    Download,
    FileArchive,
    Gamepad2,
    Home,
    Keyboard,
    Link2,
    ListChecks,
    Map,
    PackageOpen,
    Play,
    RefreshCw,
    Route,
    ScanLine,
    ScrollText,
    Search,
    Square,
    Wrench,
    X,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  type Capability = {
    area: string;
    state: string;
    rust_module: string;
    legacy_reference: string;
    notes: string;
  };

  type Trigger = {
    key: string;
    display_name: string;
    priority: number;
    default_enabled: boolean;
    exclusive: boolean;
    background: boolean;
    supported_game_ui_category: string;
    port_state: string;
  };

  type ConfigSummary = {
    capture_mode: string;
    trigger_interval: number;
    auto_pick_enabled: boolean;
    auto_skip_enabled: boolean;
    bgi_enabled_hotkey: string;
    modeled_config_sections: number;
    strongly_typed_config_sections: number;
    compatibility_config_sections: number;
    unknown_top_level_fields: number;
  };

  type CaptureModeInfo = {
    mode: string;
    legacy_value: number;
    description: string;
    implemented: boolean;
    notes: string;
  };

  type RecognitionTypeInfo = {
    recognition_type: string;
    implemented: boolean;
    notes: string;
  };

  type NavigationItem = {
    key: string;
    label: string;
    route: string;
    children: NavigationItem[];
  };

  type RunnableTrigger = {
    descriptor: Trigger;
    enabled: boolean;
  };

  type IndependentTask = {
    key: string;
    display_name: string;
    requires_main_ui_wait: boolean;
    ported: boolean;
    notes: string;
  };

  type ScriptEngine = {
    kind: string;
    port_state: string;
    legacy_reference: string;
    notes: string;
  };

  type HostBinding = {
    name: string;
    kind: string;
    legacy_type: string;
    members: string[];
    permissions: string[];
    port_state: string;
    notes: string;
  };

  type ScriptRuntimeSummary = {
    state: string;
    engines: ScriptEngine[];
    host_binding_count: number;
    host_member_count: number;
    host_object_count: number;
    host_type_count: number;
    permissions: string[];
    supported_project_types: string[];
    schedule_kinds: string[];
    project_loader: {
      default_search_paths: string[];
      package_alias_rewrite: string;
      module_detection: string[];
      resource_import_rewrites: string[];
      supported_module_extensions: string[];
    };
    settings: {
      supported_types: string[];
      defaulted_types: string[];
      cleans_multi_checkbox_options: boolean;
      preserves_unknown_fields: boolean;
    };
  };

  type ScriptHostSecurity = {
    file_allowed_extensions: string[];
    image_extensions: string[];
    file_max_write_bytes: number;
    http_uses_manifest_wildcards: boolean;
    notification_max_chars: number;
    notification_window_ms: number;
    notification_max_per_window: number;
    forbidden_notification_patterns: string[];
  };

  type DashboardState = {
    navigation: NavigationItem[];
    capabilities: Capability[];
    triggers: Trigger[];
    config: ConfigSummary;
    ui_shell: {
      shell: string;
      frontend: string;
      reason: string;
      fallback: string;
    };
    native_backend: {
      input_events_in_demo_sequence: number;
      post_message_events_in_demo_sequence: number;
      sample_hotkey: string;
      capture_modes: CaptureModeInfo[];
      recognition_types: RecognitionTypeInfo[];
      registered_onnx_models: number;
      avatar_side_model: null | {
        model_name: string;
        model_path: string;
        source_model_path: string;
        cache_dir: string;
        anonymous_tensor_rt_cache_path: string;
        named_tensor_rt_cache_path: string;
        source: string;
        exists: boolean;
        will_generate_tensor_rt_cache: boolean;
        message: string;
      };
      windows_only_backends: string[];
    };
    task_runtime: {
      dispatcher: {
        state: string;
        interval_ms: number;
        frame_index: number;
        capture_mode: string;
        previous_ui: string;
        current_ui: string;
        ui_grace_period_ms: number;
        game_active: boolean;
        game_minimized: boolean;
        picture_in_picture: boolean;
        registered_realtime_triggers: {
          task_key: string;
          interval_ms: number;
          config: unknown | null;
          registered_at_frame: number;
        }[];
      };
      runner: {
        state: string;
        current_task: string | null;
        continuous_run_group: boolean;
        pre_execution: boolean;
        suspended: boolean;
        auto_pick_pause_count: number;
        party_name: string | null;
      };
      enabled_triggers: number;
      selected_triggers: RunnableTrigger[];
      selection_reason: string;
      independent_tasks: IndependentTask[];
      catalog_entries: number;
      config_bound_catalog_entries: number;
      native_pending_catalog_entries: number;
    };
    script_runtime: {
      summary: ScriptRuntimeSummary;
      hosts: HostBinding[];
      security: ScriptHostSecurity;
      sample_macro: {
        event_count: number;
        duration_ms: number;
        has_info: boolean;
        key_events: number;
        absolute_mouse_events: number;
        relative_mouse_events: number;
        wheel_events: number;
        uses_camera_orientation: boolean;
      };
    };
  };

  type ScriptRepoState = {
    repo_path: string;
    repo_exists: boolean;
    repo_json_path: string | null;
    subscription_file_path: string;
    repo_json_bytes: number;
    subscribed_paths: string[];
    index_nodes: RepoIndexNode[];
  };

  type RepoFileResponse = {
    rel_path: string;
    extension: string;
    kind: "Text" | "ImageBase64";
    content: string;
  };

  type RepoIndexNode = {
    path: string;
    name: string;
    node_type: string;
    has_update: boolean;
    last_updated: string | null;
    depth: number;
    child_count: number;
    importable: boolean;
  };

  type RepoImportResult = {
    imported_targets: number;
    skipped_unknown_paths: string[];
    subscriptions: string[];
    dependency_files_copied: number;
    git_checkouts: number;
  };

  type RepoUriImportResult = {
    paths: string[];
    clear_clipboard_after_import: boolean;
    result: RepoImportResult;
  };

  type ScriptRepoClipboardImportResult = {
    uri: string;
    hash: string;
    path_json: string;
    paths: string[];
    source: string;
  };

  type RepoUpdateResult = {
    repo_url: string;
    branch: string;
    repo_folder_name: string;
    repo_path: string;
    repo_updated_json_path: string;
    updated: boolean;
    cloned: boolean;
    remote_changed: boolean;
    created_new_folder: boolean;
    fallback_reclone: boolean;
    marker_generated: boolean;
    old_repo_overlap_ratio: number | null;
    current_commit: string | null;
    remote_commit: string | null;
  };

  type RepoZipImportResult = {
    zip_path: string;
    repo_json_path: string;
    target_folder_name: string;
    target_path: string;
    repo_updated_json_path: string;
    best_overlap_ratio: number | null;
    matched_existing_folder: string | null;
    old_repo_overlap_ratio: number | null;
    marker_generated: boolean;
  };

  type ScriptGroupSummary = {
    name: string;
    index: number;
    path: string;
    project_count: number;
    projects: ScriptGroupProjectSummary[];
  };

  type ScriptGroupProjectSummary = {
    index: number;
    project_index: number;
    name: string;
    folder_name: string;
    project_type: string;
    status: string;
    schedule: string;
    run_num: number;
    allow_js_notification: boolean | null;
    allow_js_http_hash: string | null;
    has_settings: boolean;
  };

  type AvailableJsProject = {
    folder_name: string;
    name: string;
    version: string;
    description: string;
    settings_ui: string;
    has_settings_ui: boolean;
  };

  type AvailableKeyMouseScript = {
    name: string;
    relative_path: string;
  };

  type AvailablePathingScript = {
    name: string;
    folder_name: string;
    relative_path: string;
  };

  type AvailablePathingTreeNode = {
    name: string;
    relative_path: string;
    folder_name: string;
    route: AvailablePathingScript | null;
    children: AvailablePathingTreeNode[];
  };

  type ScriptSettingItem = {
    name: string;
    type: string;
    label: string;
    options: string[] | null;
    cascadeOptions: Record<string, string[]> | null;
    default: unknown;
  };

  type ScriptSettingsDocument = {
    group_name: string;
    project_index: number;
    project_folder_name: string;
    project_path: string;
    manifest_name: string;
    manifest_version: string;
    settings_ui_path: string | null;
    items: ScriptSettingItem[];
    values: Record<string, unknown>;
    defaults_applied: boolean;
    cleaned_invalid_values: number;
  };

  type ScriptSettingsSaveResult = {
    group_path: string;
    group_name: string;
    project_index: number;
    project_folder_name: string;
    settings: Record<string, unknown>;
    cleaned_invalid_values: number;
  };

  type KeyMouseMacroSummary = {
    event_count: number;
    duration_ms: number;
    key_events: number;
    absolute_mouse_events: number;
    relative_mouse_events: number;
    wheel_events: number;
    has_info: boolean;
    uses_camera_orientation: boolean;
  };

  type KeyMouseExecution = {
    mode: string;
    dispatched: boolean;
    dispatched_events: number;
    processed_events: number;
    cancelled: boolean;
    plan: {
      source: string;
      normalized_path: string | null;
      summary: KeyMouseMacroSummary;
      input_events: unknown[];
    };
  };

  type PathingSummary = {
    name: string;
    task_type: string;
    type_description: string;
    map_name: string;
    waypoint_count: number;
    actions: string[];
    realtime_triggers: string[];
  };

  type PathingExecution = {
    dispatched: boolean;
    completed: boolean;
    plan: {
      source: string;
      normalized_path: string | null;
      summary: PathingSummary;
      party_config: unknown;
    };
    execution_plan: {
      map_name: string;
      segment_count: number;
      waypoint_count: number;
      action_count: number;
      expected_fight_count: number;
      autopick_realtime_trigger_enabled: boolean;
    };
  };

  type PathingMovementContract = {
    contract_version: number;
    movement_executor_ready: boolean;
    native_pathing_completed: boolean;
    pending_dependencies: string[];
    segment_count: number;
    waypoint_count: number;
  };

  type AutoPathingExecutionPlan = {
    source: string;
    route: string;
    normalized_path: string;
    summary: PathingSummary;
    execution_plan: PathingExecution["execution_plan"] & {
      movement_contract: PathingMovementContract;
    };
    dispatched: boolean;
    completed: boolean;
    notes: string;
  };

  type PathingBoundaryStatus = "reported" | "executed" | "skipped" | "unsupported" | "invalid";

  type PathingActionBoundaryReport = {
    action_code: string;
    status: PathingBoundaryStatus;
    message: string;
    common_job_task_key: string | null;
    common_job_plan: unknown | null;
    common_job_live_execution: unknown | null;
  };

  type PathingPhaseBoundaryReport = {
    phase: string;
    status: PathingBoundaryStatus;
    reason: string;
    common_job_task_key: string | null;
    common_job_plan: unknown | null;
    common_job_live_execution: unknown | null;
    navigation_seed: unknown | null;
  };

  type PathingWaypointBoundaryReport = {
    global_index: number;
    segment_index: number;
    segment_waypoint_index: number;
    waypoint_type: string;
    action: string | null;
    phase_reports: PathingPhaseBoundaryReport[];
    action_report: PathingActionBoundaryReport | null;
  };

  type AutoPathingPhaseExecutionStatus = "executed" | "skipped" | "unsupported" | "failed" | "cancelled";

  type PathingMovementPhaseBoundaryReport = {
    phase: string;
    status: AutoPathingPhaseExecutionStatus;
    message: string;
    pending_dependencies: string[];
  };

  type PathingMovementWaypointBoundaryReport = {
    global_index: number;
    segment_index: number;
    segment_waypoint_index: number;
    waypoint_type: string;
    move_mode: string;
    action: string | null;
    phase_reports: PathingMovementPhaseBoundaryReport[];
  };

  type PathingMovementSegmentBoundaryReport = {
    segment_index: number;
    waypoint_reports: PathingMovementWaypointBoundaryReport[];
  };

  type AutoPathingMovementBoundaryReport = {
    source: string;
    route: string;
    normalized_path: string;
    movement_contract_consumed: boolean;
    movement_completed: boolean;
    movement_completion_status: string;
    native_pathing_completed: boolean;
    movement_executor_ready: boolean;
    movement_contract_version: number;
    movement_pending_dependencies: string[];
    movement_segment_count: number;
    movement_waypoint_count: number;
    executed_phases: number;
    skipped_phases: number;
    unsupported_phases: number;
    failed_phases: number;
    cancelled_phases: number;
    failed_phase: {
      global_index: number;
      segment_index: number;
      segment_waypoint_index: number;
      phase: string;
      message: string;
    } | null;
    segment_reports: PathingMovementSegmentBoundaryReport[];
    notes: string;
  };

  type GlobalInputExecution = {
    mode: string;
    dispatched: boolean;
    dispatched_events: number;
    events: unknown[];
  };

  type HttpExecution = {
    mode: string;
    dispatched: boolean;
    request: {
      method: string;
      url: string;
      headers: unknown[];
    };
    response: {
      status_code: number;
      body: string;
      headers: Record<string, string>;
    } | null;
  };

  type ShellExecution = {
    command: string;
    working_directory: string;
    timeout_seconds: number;
    no_window: boolean;
    output_enabled: boolean;
    status: string;
    waited_for_exit: boolean;
    exit_code: number | null;
    output_shell: string;
    output: string;
  };

  type DesktopShellTaskExecution = {
    task: string;
    result: ShellExecution;
  };

  type DesktopAutoPathingTaskExecution = {
    task: string;
    result: AutoPathingExecutionPlan;
  };

  type AutoPathingActionBoundaryReport = {
    source: string;
    route: string;
    normalized_path: string;
    completion_scope: string;
    boundary_completed: boolean;
    movement_attempted: boolean;
    movement_completion_status: string;
    native_pathing_completed: boolean;
    movement_executor_ready: boolean;
    movement_contract_version: number;
    movement_pending_dependencies: string[];
    movement_segment_count: number;
    movement_waypoint_count: number;
    movement_report: AutoPathingMovementBoundaryReport | null;
    executed_actions: number;
    skipped_actions: number;
    unsupported_actions: number;
    invalid_actions: number;
    unsupported_phases: number;
    waypoint_reports: PathingWaypointBoundaryReport[];
    notes: string;
  };

  type DesktopAutoPathingActionBoundaryExecution = {
    task: string;
    plan: AutoPathingExecutionPlan;
    boundary: AutoPathingActionBoundaryReport;
  };

  type IndependentLiveTaskReport = {
    task_key: string;
    completed: boolean;
    state: Record<string, unknown>;
    executed_steps: unknown[];
    skipped_steps: unknown[];
  };

  type AutoCookLiveTaskReport = {
    task_key: string;
    status: string;
    state: {
      frames_processed?: number;
      space_press_count?: number;
      white_confirm_click_count?: number;
      delay_count?: number;
      [key: string]: unknown;
    };
    events: unknown[];
  };

  type AutoMusicPerformanceReport = {
    task_key: string;
    stop_reason: string;
    frames_processed: number;
    held_keys_before_release: string[];
    events: unknown[];
  };

  type AutoMusicAlbumReport = {
    task_key: string;
    status: string;
    difficulty_count: number;
    songs_checked: number;
    skipped_songs: number;
    performed_songs: number;
    events: unknown[];
  };

  type AutoOpenChestLiveTaskReport = {
    task_key: string;
    completed: boolean;
    status: string;
    state: {
      iterations?: number;
      elapsed_ms?: number;
      final_result?: string | null;
      timed_out?: boolean;
      cancelled?: boolean;
      [key: string]: unknown;
    };
    decisions: unknown[];
    dispatched_actions: unknown[];
    cleanup_actions: unknown[];
    post_loop_actions: unknown[];
  };

  type DesktopIndependentLiveTaskExecution = {
    task: string;
    result:
      | IndependentLiveTaskReport
      | AutoCookLiveTaskReport
      | AutoMusicPerformanceReport
      | AutoMusicAlbumReport
      | AutoOpenChestLiveTaskReport;
  };

  type CombatCommandPlan = {
    avatar: string;
    method: string;
    args: string[];
    activating_rounds: number[];
    raw: string;
  };

  type CombatCommandExecutionPlan = {
    index: number;
    command: CombatCommandPlan;
    switch_policy: string;
    action: { kind: string; payload?: unknown };
    default_input_events: Array<unknown>;
    requires_combat_context: boolean;
    static_input_ready: boolean;
    pending_context: string[];
  };

  type DesktopAutoFightTaskExecution = {
    task: string;
    result: {
      param: {
        combat_strategy_path: string;
        timeout: number;
        fight_finish_detect_enabled: boolean;
        pick_drops_after_fight_enabled: boolean;
      };
      combat_scripts: {
        source_path: string;
        scripts: Array<{
          name: string;
          path: string | null;
          avatar_names: string[];
          commands: CombatCommandPlan[];
        }>;
        parse_failures: Array<{ path: string; message: string }>;
      };
      script_execution_plans: Array<{
        name: string;
        path: string | null;
        avatar_names: string[];
        commands: CombatCommandExecutionPlan[];
      }>;
      playback_evaluation: {
        total_commands: number;
        static_ready_commands: number;
        context_bound_commands: number;
        default_input_event_count: number;
        dispatch_ready: boolean;
      };
      team_selection: {
        status: string;
        team_avatar_names: string[];
        matched_avatar_count: number;
        full_match: boolean;
        command_avatar_names: string[];
        executable_avatar_names: string[];
        filtered_out_avatar_names: string[];
        executable_commands: CombatCommandPlan[];
        message: string;
      };
      team_plan: {
        avatars: Array<{
          index: number;
          name: string;
          id: string;
          name_en: string;
          weapon: string;
          skill_cd_seconds: number | null;
          skill_hold_cd_seconds: number | null;
          burst_cd_seconds: number | null;
          manual_skill_cd_seconds: number;
          action_scheduler_configured: boolean;
        }>;
        command_avatar_names: string[];
        can_be_skipped_avatar_names: string[];
        all_command_avatars_can_be_skipped: boolean;
      } | null;
      team_playback: AutoFightTeamPlaybackExecution | null;
      fight_loop_plan: {
        timeout_seconds: number;
        command_count: number;
        executable_command_count: number;
        fight_finish_detect_enabled: boolean;
        rotate_find_enemy_enabled: boolean;
        check_before_burst_enabled: boolean;
        guardian_enabled: boolean;
        guardian_avatar_index: number | null;
        guardian_avatar_name: string | null;
        kazuha_pickup_enabled: boolean;
        pickup_drops_after_fight_enabled: boolean;
        exp_based_pickup_enabled: boolean;
        battle_threshold_for_loot: number;
        steps: Array<{
          phase: string;
          kind: string;
          command_index: number | null;
          avatar: string | null;
          enabled: boolean;
          requires_native_context: string[];
          message: string;
        }>;
        native_dispatch_ready: boolean;
      };
      finish_detection_plan: {
        pre_detect_delay_ms: number;
        detect_delay_ms: number;
        rotate_find_enemy_enabled: boolean;
        progress_pixel: [number, number];
        white_tile_pixel: [number, number];
        steps: Array<{
          kind: string;
          enabled: boolean;
          input_events: unknown[];
          delay_ms: number;
          requires_capture: boolean;
          requires_vision: boolean;
          message: string;
        }>;
        native_ready_without_capture: boolean;
      };
      action_scheduler_plans: Array<{
        script_name: string;
        script_path: string | null;
        command_avatar_names: string[];
        scheduler: {
          entries: Array<{ avatar: string; manual_skill_cd_seconds: number; has_explicit_cd: boolean }>;
          configured_avatar_names: string[];
          skipped_avatar_names: string[];
          all_command_avatars_can_be_skipped: boolean;
        };
      }>;
      dispatched: boolean;
      completed: boolean;
      notes: string;
    };
  };

  type AutoFightFinishDetectionExecution = {
    mode: string;
    plan: DesktopAutoFightTaskExecution["result"]["finish_detection_plan"];
    detection: {
      finished: boolean;
      progress_pixel: { r: number; g: number; b: number };
      white_tile_pixel: { r: number; g: number; b: number };
    } | null;
    before_capture_events: unknown[];
    after_detection_events: unknown[];
    dispatched: boolean;
    dispatched_events: number;
    cancelled: boolean;
    captured: boolean;
  };

  type DesktopAutoFightFinishProbeExecution = {
    task: string;
    result: AutoFightFinishDetectionExecution;
  };

  type ActiveAvatarDetectionExecution = {
    task: string;
    result: {
      active_index: number | null;
      method: string;
      rects: Array<{ x: number; y: number; width: number; height: number }>;
      white_rect_count: number;
      not_white_rect_index: number | null;
      edge_white_ratios: number[];
      difference_votes: number[];
      message: string;
    };
  };

  type AutoFightTeamPlaybackExecution = {
    mode: string;
    script_name: string;
    total_commands: number;
    candidate_commands: number;
    planned_commands: Array<{
      command_index: number;
      avatar: string;
      team_index: number | null;
      switch_events: unknown[];
      action_events: unknown[];
      input_events: unknown[];
      resolved_context: string[];
      pending_context: string[];
      executable: boolean;
      message: string;
    }>;
    playable_commands: number;
    blocked_command_index: number | null;
    blocked_requirements: string[];
    input_events: unknown[];
    dispatch_ready: boolean;
    dispatched: boolean;
    dispatched_events: number;
    cancelled: boolean;
  };

  type DesktopAutoFightTeamPlaybackResult = {
    task: string;
    result: AutoFightTeamPlaybackExecution;
  };

  type NotificationExecution = {
    mode: string;
    dispatched: boolean;
    record: {
      kind: string;
      message: string;
      timestamp_ms: number;
    };
    delivery: {
      event_code: string;
      result: string;
      message: string;
      timestamp_ms: number;
    } | null;
    app_dispatch?: NotificationDispatchExecution | null;
    app_dispatch_error?: string | null;
  };

  type HtmlMaskDesktopDispatch = {
    action: string;
    windowId: string | null;
    windowLabel: string | null;
    dispatched: boolean;
    message: string;
  };

  type NotificationProviderDelivery = {
    provider: string;
    provider_name: string;
    status: "Sent" | "Skipped" | "Failed" | "Unsupported";
    requests: number;
    message: string | null;
  };

  type NotificationDispatchExecution = {
    attempted: boolean;
    skipped_reason: string | null;
    deliveries: NotificationProviderDelivery[];
  };

  type NotificationServiceState = {
    initialized: boolean;
    config_path: string;
    enabled_providers: string[];
    provider_count: number;
    refreshed_at_ms: number | null;
  };

  type UpdateChannel = "Stable" | "Alpha";

  type UpdateRequestPlan = {
    channel: UpdateChannel;
    url: string;
    query: Record<string, string>;
    user_agent: string | null;
  };

  type UpdateDecision = {
    action: "Noop" | "ShowUpToDateMessage" | "OpenUpdateWindow" | "SuppressedByIgnoredVersion";
    new_version: string | null;
    download_page_url: string | null;
    release_notes_request: UpdateRequestPlan | null;
  };

  type UpdateReleaseNotes = {
    name: string | null;
    body: string | null;
    html_url: string | null;
  };

  type UpdateCheckResult = {
    trigger: "Auto" | "Manual";
    app_version: string;
    channel: UpdateChannel;
    request: UpdateRequestPlan;
    latest_version: string | null;
    decision: UpdateDecision;
    mirror_outcome: unknown | null;
    release_notes: UpdateReleaseNotes | null;
    download_page_url: string;
    updater_path: string;
    updater_exists: boolean;
    updater_options: UpdaterLaunchPlan[];
    ignored_version: string;
  };

  type UpdaterLaunchPlan = {
    source:
      | "Default"
      | "Cnb"
      | "Github"
      | "Dfs"
      | "DfsAlpha"
      | "MirrorChyan"
      | "MirrorChyanAlpha";
    display_name: string;
    args: string[];
    source_arg: string | null;
    requires_cdk: boolean;
    warning: string | null;
  };

  type BackgroundUpdateState = {
    running: boolean;
    last_result: UpdateCheckResult | null;
    last_error: string | null;
  };

  type UpdateActionResult = {
    ok: boolean;
    action: string;
    detail: string;
    exit_scheduled: boolean;
  };

  type RedeemCodeFeedResult = {
    request_url: string;
    local_version: string;
    remote_text: string | null;
    decision: {
      request_url: string;
      has_update: boolean;
      remote_version: string | null;
    };
  };

  type RedeemCodeFeedItem = {
    title: string;
    content: string;
    time: string;
    tag: string;
    valid: string;
    codes: string[];
  };

  type RedeemCodeFeedItemsResult = {
    request_url: string;
    items: RedeemCodeFeedItem[];
    raw_bytes: number;
  };

  type RedeemCodeLiveCode = {
    code: string;
    items: string;
  };

  type RedeemCodeLiveResult = {
    act_id_sources: string[];
    index_url: string;
    refresh_code_url: string;
    data: {
      act_id: string;
      code_version: string;
      title: string;
      codes: RedeemCodeLiveCode[];
    };
  };

  type UseRedeemCodePlanResult = {
    extracted_codes: string[];
    plan: {
      task_key: string;
      display_name: string;
      port_state: string;
      executor_ready: boolean;
      codes: RedeemCodeLiveCode[];
      steps: Array<{
        phase: string;
        condition: string;
        code: string | null;
        label: string;
        action: { kind: string; payload: unknown };
      }>;
      notes: string;
    };
  };

  type UseRedeemCodeExecutionResult = UseRedeemCodePlanResult & {
    report: IndependentLiveTaskReport;
  };

  type RedeemCodeClipboardState = {
    clipboard_listener_enabled: boolean;
    ignored_hash_count: number;
  };

  type RedeemCodeClipboardCheckResult = {
    clipboard_listener_enabled: boolean;
    ignored: boolean;
    hash: string;
    extracted_codes: string[];
    plan: UseRedeemCodePlanResult["plan"] | null;
  };

  type DesktopLogState = {
    path: string;
    exists: boolean;
    bytes: number;
    tail: string[];
  };

  type DesktopShellState = {
    exit_to_tray: boolean;
    tray_enabled: boolean;
    config_path: string;
    log_path: string;
  };

  type OverlayLayoutRect = {
    left_ratio: number;
    top_ratio: number;
    width_ratio: number;
    height_ratio: number;
  };

  type OverlayMetricDescriptor = {
    key: string;
    display_name: string;
    tooltip: string;
    enabled_by_default: boolean;
  };

  type DesktopOverlayState = {
    mask_enabled: boolean;
    show_log_box: boolean;
    show_status: boolean;
    display_recognition_results_on_mask: boolean;
    directions_enabled: boolean;
    uid_cover_enabled: boolean;
    show_fps: boolean;
    show_overlay_metrics: boolean;
    text_opacity: number;
    overlay_layout_edit_enabled: boolean;
    log_box_layout: OverlayLayoutRect;
    status_layout: OverlayLayoutRect;
    metrics_layout: OverlayLayoutRect;
    metrics: OverlayMetricDescriptor[];
    enabled_metric_keys: string[];
    migrated_legacy_metrics_layout: boolean;
  };

  type OverlayPatchPayload = {
    maskEnabled?: boolean;
    showLogBox?: boolean;
    showStatus?: boolean;
    displayRecognitionResultsOnMask?: boolean;
    showOverlayMetrics?: boolean;
    overlayLayoutEditEnabled?: boolean;
    metricKey?: string;
    metricEnabled?: boolean;
    resetMetricsLayout?: boolean;
  };

  type DesktopShellActionResult = {
    ok: boolean;
    action: string;
    detail: string;
  };

  type ExecutionSummary = {
    title: string;
    meta: string;
    details: { label: string; value: string }[];
  };

  type ScriptHostCall = {
    target: string;
    method: string;
    args: unknown[];
    result: unknown;
  };

  type TaskInvocationPlan = {
    kind: string;
    task_key: string | null;
    interval_ms: number | null;
    clears_existing_triggers: boolean;
    config: unknown | null;
    uses_linked_cancellation: boolean;
  };

  type JavaScriptTaskInvocations = {
    dispatcher: TaskInvocationPlan[];
    genshin: TaskInvocationPlan[];
    errors: string[];
  };

  type IndependentTaskLiveExecution =
    | {
        AutoPathingActionBoundary: AutoPathingActionBoundaryReport;
      }
    | Record<string, unknown>;

  type TaskInvocationExecutionResult = {
    plan: TaskInvocationPlan;
    mode: string;
    status: string;
    message: string;
    executed: boolean;
    independent_task_live_execution: IndependentTaskLiveExecution | null;
    live_completed: boolean | null;
  };

  type JavaScriptTaskExecution = {
    mode: string;
    dispatcher: TaskInvocationExecutionResult[];
    genshin: TaskInvocationExecutionResult[];
  };

  type ScriptExecutionOutcome = {
    runtime: string;
    project: string;
    folder_name: string;
    execution_mode: string;
    main_script_path: string;
    result: unknown | null;
    result_display: string;
    console: string[];
    logs: { level: string; message: string }[];
    host_calls: ScriptHostCall[];
    task_invocations: JavaScriptTaskInvocations;
    task_execution: JavaScriptTaskExecution;
    html_mask_from_html: unknown[];
  };

  type ScriptGroupExecutionOutcome = {
    group_name: string;
    requested_projects: number;
    steps: ScriptGroupStepExecutionOutcome[];
    attempted_steps: number;
    completed_steps: number;
    planned_steps: number;
    cancelled_steps: number;
    failed_steps: number;
    skipped_steps: number;
  };

  type ScriptGroupStepExecutionOutcome = {
    project_index: number;
    project_order: number;
    run_iteration: number;
    run_count: number;
    name: string;
    folder_name: string;
    project_type: string;
    status: string;
    javascript: ScriptExecutionOutcome | null;
    key_mouse_execution: KeyMouseExecution | null;
    pathing_execution: PathingExecution | null;
    shell_result: ShellExecution | null;
    error: string | null;
    skip_reason: string | null;
  };

  type ScriptStopResult = {
    requested: boolean;
    runner_state: {
      state: string;
      current_task: string | null;
      continuous_run_group: boolean;
      pre_execution: boolean;
      suspended: boolean;
      auto_pick_pause_count: number;
      party_name: string | null;
    };
  };

  const iconByKey = {
    home: Home,
    realtime: Activity,
    tasks: ListChecks,
    "one-dragon": Bot,
    automation: Route,
    scheduler: ScrollText,
    scripts: Play,
    pathing: Map,
    recorder: Gamepad2,
    macro: Wrench,
    hotkeys: Keyboard,
    notifications: Bell,
    settings: Cog,
  };

  const legacyScheduleOptions = [
    { value: "", label: "Manual" },
    { value: "Daily", label: "Daily" },
    { value: "EveryTwoDays", label: "Every 2 Days" },
    { value: "Monday", label: "Monday" },
    { value: "Tuesday", label: "Tuesday" },
    { value: "Wednesday", label: "Wednesday" },
    { value: "Thursday", label: "Thursday" },
    { value: "Friday", label: "Friday" },
    { value: "Saturday", label: "Saturday" },
    { value: "Sunday", label: "Sunday" },
  ];

  let state = $state<DashboardState | null>(null);
  let selected = $state("home");
  let error = $state<string | null>(null);
  let repoState = $state<ScriptRepoState | null>(null);
  let repoPath = $state("Repos/bettergi-scripts-list");
  let repoUrl = $state("https://cnb.cool/bettergi/bettergi-scripts-list");
  let repoZipPath = $state("");
  let repoImportUri = $state("");
  let repoJsonPreview = $state("");
  let repoFilePath = $state("js/demo/manifest.json");
  let repoFile = $state<RepoFileResponse | null>(null);
  let repoStatus = $state("");
  let repoSearch = $state("");
  let selectedImportPaths = $state<string[]>([]);
  let repoGitMode = $state(false);
  let scriptClipboardResult = $state<ScriptRepoClipboardImportResult | null>(null);
  let scriptClipboardNoticeOpen = $state(false);
  let scriptGroupsState = $state<ScriptGroupSummary[]>([]);
  let selectedGroupName = $state("");
  let selectedProjectIndex = $state(0);
  let draggedProjectIndex = $state<number | null>(null);
  let newGroupName = $state("");
  let renameGroupName = $state("");
  let availableJsProjects = $state<AvailableJsProject[]>([]);
  let selectedAvailableJsFolder = $state("");
  let availableKeyMouseScripts = $state<AvailableKeyMouseScript[]>([]);
  let selectedKeyMouseScript = $state("");
  let availablePathingScripts = $state<AvailablePathingScript[]>([]);
  let availablePathingTree = $state<AvailablePathingTreeNode | null>(null);
  let selectedPathingScript = $state("");
  let shellCommand = $state("");
  let scriptSettings = $state<ScriptSettingsDocument | null>(null);
  let scriptSettingsStatus = $state("");
  let scriptRunStatus = $state("");
  let scriptRunResult = $state<ScriptExecutionOutcome | null>(null);
  let scriptGroupRunResult = $state<ScriptGroupExecutionOutcome | null>(null);
  let scriptRunBusy = $state(false);
  let notificationMessage = $state("BetterGI Rust test notification");
  let notificationProvider = $state("");
  let notificationSendBusy = $state(false);
  let notificationSendStatus = $state("");
  let notificationResult = $state<NotificationDispatchExecution | null>(null);
  let notificationServiceState = $state<NotificationServiceState | null>(null);
  let notificationServiceBusy = $state(false);
  let notificationServiceStatus = $state("");
  let updateChannel = $state<"stable" | "alpha">("stable");
  let updateCheckBusy = $state(false);
  let updateActionBusy = $state(false);
  let updateStatus = $state("");
  let updateResult = $state<UpdateCheckResult | null>(null);
  let backgroundUpdateState = $state<BackgroundUpdateState | null>(null);
  let backgroundUpdateNoticeOpen = $state(false);
  let updaterExitAfterLaunch = $state(true);
  let redeemFeedBusy = $state(false);
  let redeemFeedStatus = $state("");
  let redeemFeedResult = $state<RedeemCodeFeedResult | null>(null);
  let redeemFeedItems = $state<RedeemCodeFeedItemsResult | null>(null);
  let redeemLiveResult = $state<RedeemCodeLiveResult | null>(null);
  let redeemManualText = $state("");
  let redeemPlanResult = $state<UseRedeemCodePlanResult | null>(null);
  let redeemExecutionResult = $state<UseRedeemCodeExecutionResult | null>(null);
  let redeemClipboardState = $state<RedeemCodeClipboardState | null>(null);
  let redeemClipboardResult = $state<RedeemCodeClipboardCheckResult | null>(null);
  let redeemFeedNoticeOpen = $state(false);
  let redeemFeedNoticeVersion = $state<string | null>(null);
  let redeemClipboardNoticeOpen = $state(false);
  let redeemFeedAutoChecked = false;
  let redeemFeedNoticeTimer: ReturnType<typeof setTimeout> | null = null;
  let shellState = $state<DesktopShellState | null>(null);
  let shellStatus = $state("");
  let shellBusy = $state(false);
  let independentShellBusy = $state(false);
  let independentShellCommand = $state("");
  let independentShellTimeout = $state(60);
  let independentShellResult = $state<DesktopShellTaskExecution | null>(null);
  let independentLiveTaskBusy = $state(false);
  let independentLiveTaskResult = $state<DesktopIndependentLiveTaskExecution | null>(null);
  let independentPathingBusy = $state(false);
  let independentPathingResult = $state<DesktopAutoPathingTaskExecution | null>(null);
  let independentPathingBoundaryResult = $state<DesktopAutoPathingActionBoundaryExecution | null>(null);
  let independentFightBusy = $state(false);
  let independentFightPlaybackBusy = $state(false);
  let independentFightProbeBusy = $state(false);
  let independentFightAvatarBusy = $state(false);
  let independentFightStrategy = $state("");
  let independentFightTeamNames = $state("");
  let independentFightResult = $state<DesktopAutoFightTaskExecution | null>(null);
  let independentFightPlaybackResult = $state<DesktopAutoFightTeamPlaybackResult | null>(null);
  let independentFightProbeResult = $state<DesktopAutoFightFinishProbeExecution | null>(null);
  let independentFightAvatarResult = $state<ActiveAvatarDetectionExecution | null>(null);
  let overlayState = $state<DesktopOverlayState | null>(null);
  let overlayStatus = $state("");
  let overlayBusy = $state(false);
  let logState = $state<DesktopLogState | null>(null);
  let logStatus = $state("");

  $effect(() => {
    void load();
    void loadNotificationServiceState();
    void loadBackgroundUpdateState();
    void autoCheckRedeemCodeFeed();
    void loadRedeemClipboardState();
    void loadDesktopOverlay();
  });

  $effect(() => {
    const unlisten = listen<string>("desktop-shell://check-update", (event) => {
      selected = "settings";
      updateChannel = event.payload === "alpha" ? "alpha" : "stable";
      void checkForUpdate();
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });

  $effect(() => {
    const unlisten = listen<BackgroundUpdateState>("desktop-update://background-check", (event) => {
      backgroundUpdateState = event.payload;
      const result = event.payload.last_result;
      if (result) {
        updateResult = result;
        updateStatus = event.payload.running ? "background checking" : updateDecisionLabel(result.decision.action);
        if (result.decision.action === "OpenUpdateWindow") {
          backgroundUpdateNoticeOpen = true;
        }
      } else if (event.payload.last_error) {
        updateStatus = "background check failed";
      } else if (event.payload.running) {
        updateStatus = "background checking";
      }
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });

  $effect(() => {
    const unlisten = listen<RedeemCodeClipboardCheckResult>("redeem-code://clipboard-detected", (event) => {
      redeemClipboardResult = event.payload;
      redeemClipboardNoticeOpen = true;
      redeemManualText = event.payload.text;
      if (event.payload.plan) {
        redeemPlanResult = {
          extracted_codes: event.payload.extracted_codes,
          plan: event.payload.plan,
        };
      }
      redeemFeedStatus = `${event.payload.extracted_codes.length} clipboard codes`;
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });

  $effect(() => {
    const unlisten = listen<ScriptRepoClipboardImportResult>("script-repo://clipboard-import-detected", (event) => {
      scriptClipboardResult = event.payload;
      scriptClipboardNoticeOpen = true;
      repoImportUri = event.payload.uri;
      repoStatus = `${event.payload.paths.length} clipboard path(s) ready`;
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });

  async function load() {
    try {
      state = await invoke<DashboardState>("dashboard_state");
      selected = state.navigation[0]?.key ?? "home";
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadRepoState() {
    try {
      repoState = await invoke<ScriptRepoState>("script_repo_state", { repoPath });
      repoPath = repoState.repo_path;
      repoStatus = repoState.repo_exists ? "repository loaded" : "repository missing";
      selectedImportPaths = selectedImportPaths.filter((path) =>
        repoState?.index_nodes.some((node) => node.path === path),
      );
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadRepoJson() {
    try {
      repoJsonPreview = await invoke<string>("script_repo_json", { repoPath });
      repoStatus = "repo.json loaded";
    } catch (err) {
      error = String(err);
    }
  }

  async function loadRepoFile() {
    try {
      repoFile = await invoke<RepoFileResponse | null>("script_repo_file", {
        repoPath,
        relPath: repoFilePath,
      });
      repoStatus = repoFile ? "file loaded" : "file not found";
    } catch (err) {
      error = String(err);
    }
  }

  async function markSelectedUpdated(path: string) {
    try {
      const updated = await invoke<boolean>("script_repo_mark_updated", { repoPath, path });
      await loadRepoState();
      repoStatus = updated ? `cleared ${path}` : "path not changed";
    } catch (err) {
      error = String(err);
    }
  }

  async function clearRepoUpdates() {
    try {
      const path = await invoke<string>("script_repo_clear_update", { repoPath });
      await loadRepoState();
      repoStatus = `reset ${path}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function importSelectedPaths() {
    try {
      const result = await invoke<RepoImportResult>("script_repo_import_paths", {
        repoPath,
        paths: selectedImportPaths,
        gitRepo: repoGitMode,
      });
      selectedImportPaths = [];
      await loadRepoState();
      repoStatus = importResultLabel("imported selection", result);
    } catch (err) {
      error = String(err);
    }
  }

  async function updateRepoFromGit() {
    try {
      const result = await invoke<RepoUpdateResult>("script_repo_update_from_git", { repoUrl });
      repoPath = result.repo_path;
      await loadRepoState();
      const action = result.cloned ? "cloned" : result.updated ? "updated" : "current";
      repoStatus = `${action} ${result.repo_folder_name}${result.marker_generated ? ", update markers ready" : ""}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function importZipRepo() {
    try {
      const result = await invoke<RepoZipImportResult>("script_repo_import_zip", {
        zipPath: repoZipPath,
        folder: null,
      });
      repoPath = result.target_path;
      await loadRepoState();
      repoStatus = `imported zip ${result.target_folder_name}${result.marker_generated ? ", update markers ready" : ""}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function importBettergiUri() {
    try {
      const result = await invoke<RepoUriImportResult>("script_repo_import_uri", {
        repoPath,
        uri: repoImportUri,
        gitRepo: repoGitMode,
      });
      await loadRepoState();
      repoStatus = `${importResultLabel("imported link", result.result)} from ${result.paths.length} paths`;
    } catch (err) {
      error = String(err);
    }
  }

  async function importScriptClipboardUri() {
    if (!scriptClipboardResult) return;
    try {
      const result = await invoke<RepoUriImportResult>("script_repo_import_clipboard_uri", {
        repoPath,
        uri: scriptClipboardResult.uri,
        gitRepo: repoGitMode,
      });
      repoImportUri = scriptClipboardResult.uri;
      scriptClipboardResult = null;
      scriptClipboardNoticeOpen = false;
      await loadRepoState();
      await loadRedeemClipboardState();
      repoStatus = `${importResultLabel("imported clipboard link", result.result)} from ${result.paths.length} paths`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function ignoreScriptClipboardUri() {
    if (!scriptClipboardResult) return;
    try {
      redeemClipboardState = await invoke<RedeemCodeClipboardState>("script_repo_clipboard_ignore", {
        payload: { text: scriptClipboardResult.uri },
      });
      scriptClipboardResult = null;
      scriptClipboardNoticeOpen = false;
      repoStatus = "clipboard link ignored";
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function updateSubscribedScripts() {
    try {
      const result = await invoke<RepoImportResult>("script_repo_update_subscribed", {
        repoPath,
        gitRepo: repoGitMode,
      });
      await loadRepoState();
      repoStatus = importResultLabel("updated subscriptions", result);
    } catch (err) {
      error = String(err);
    }
  }

  async function loadScriptGroups() {
    try {
      scriptGroupsState = await invoke<ScriptGroupSummary[]>("script_groups");
      if (!selectedGroupName && scriptGroupsState.length > 0) {
        selectedGroupName = scriptGroupsState[0].name;
      }
      if (selectedGroupName && !renameGroupName) {
        renameGroupName = selectedGroupName;
      }
      const group = selectedScriptGroup();
      if (group && !group.projects.some((project) => project.index === selectedProjectIndex)) {
        selectedProjectIndex = group.projects[0]?.index ?? 0;
      }
      scriptSettingsStatus = scriptGroupsState.length
        ? `loaded ${scriptGroupsState.length} groups`
        : "no script groups";
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadAvailableJsProjects() {
    try {
      availableJsProjects = await invoke<AvailableJsProject[]>("script_available_js_projects");
      if (!selectedAvailableJsFolder && availableJsProjects.length > 0) {
        selectedAvailableJsFolder = availableJsProjects[0].folder_name;
      }
      scriptSettingsStatus = `loaded ${availableJsProjects.length} JS projects`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadAvailableKeyMouseScripts() {
    try {
      availableKeyMouseScripts = await invoke<AvailableKeyMouseScript[]>(
        "script_available_key_mouse_scripts",
      );
      if (!selectedKeyMouseScript && availableKeyMouseScripts.length > 0) {
        selectedKeyMouseScript = availableKeyMouseScripts[0].name;
      }
      scriptSettingsStatus = `loaded ${availableKeyMouseScripts.length} key-mouse scripts`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadAvailablePathingScripts() {
    try {
      const [scripts, tree] = await Promise.all([
        invoke<AvailablePathingScript[]>("script_available_pathing_scripts"),
        invoke<AvailablePathingTreeNode>("script_available_pathing_tree"),
      ]);
      availablePathingScripts = scripts;
      availablePathingTree = tree;
      if (!selectedPathingScript && availablePathingScripts.length > 0) {
        selectedPathingScript = availablePathingScripts[0].relative_path;
      }
      scriptSettingsStatus = `loaded ${availablePathingScripts.length} pathing scripts`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function createScriptGroup() {
    if (!newGroupName.trim()) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_create", {
        name: newGroupName,
      });
      selectedGroupName = group.name;
      renameGroupName = group.name;
      selectedProjectIndex = 0;
      newGroupName = "";
      await loadScriptGroups();
      scriptSettingsStatus = `created group ${group.name}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function renameScriptGroup() {
    if (!selectedGroupName || !renameGroupName.trim()) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_rename", {
        oldName: selectedGroupName,
        newName: renameGroupName,
      });
      selectedGroupName = group.name;
      renameGroupName = group.name;
      await loadScriptGroups();
      scriptSettingsStatus = `renamed group ${group.name}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function deleteScriptGroup() {
    if (!selectedGroupName) return;
    try {
      const deleted = await invoke<boolean>("script_group_delete", { name: selectedGroupName });
      selectedGroupName = "";
      renameGroupName = "";
      selectedProjectIndex = 0;
      scriptSettings = null;
      await loadScriptGroups();
      scriptSettingsStatus = deleted ? "deleted group" : "group missing";
    } catch (err) {
      error = String(err);
    }
  }

  async function addSelectedJsProject() {
    if (!selectedGroupName || !selectedAvailableJsFolder) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_project_add_js", {
        groupName: selectedGroupName,
        folderName: selectedAvailableJsFolder,
      });
      selectedGroupName = group.name;
      selectedProjectIndex = group.projects.at(-1)?.index ?? 0;
      await loadScriptGroups();
      scriptSettingsStatus = `added ${selectedAvailableJsFolder}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function addSelectedKeyMouseScript() {
    if (!selectedGroupName || !selectedKeyMouseScript) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_project_add_key_mouse", {
        groupName: selectedGroupName,
        name: selectedKeyMouseScript,
      });
      selectedGroupName = group.name;
      selectedProjectIndex = group.projects.at(-1)?.index ?? 0;
      await loadScriptGroups();
      scriptSettingsStatus = `added ${selectedKeyMouseScript}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function addSelectedPathingScript() {
    if (!selectedGroupName || !selectedPathingScript) return;
    const script = availablePathingScripts.find((item) => item.relative_path === selectedPathingScript);
    if (!script) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_project_add_pathing", {
        groupName: selectedGroupName,
        name: script.name,
        folderName: script.folder_name,
      });
      selectedGroupName = group.name;
      selectedProjectIndex = group.projects.at(-1)?.index ?? 0;
      await loadScriptGroups();
      scriptSettingsStatus = `added ${script.relative_path}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function addShellProject() {
    if (!selectedGroupName || !shellCommand.trim()) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_project_add_shell", {
        groupName: selectedGroupName,
        command: shellCommand,
      });
      selectedGroupName = group.name;
      selectedProjectIndex = group.projects.at(-1)?.index ?? 0;
      shellCommand = "";
      await loadScriptGroups();
      scriptSettingsStatus = "added shell command";
    } catch (err) {
      error = String(err);
    }
  }

  async function saveSelectedProject() {
    const project = selectedScriptProject();
    if (!selectedGroupName || !project) return;
    try {
      await invoke<ScriptGroupSummary>("script_group_project_update", {
        groupName: selectedGroupName,
        projectIndex: project.index,
        patch: {
          status: project.status,
          schedule: project.schedule,
          runNum: project.run_num,
          allowJsNotification: project.allow_js_notification,
        },
      });
      await loadScriptGroups();
      scriptSettingsStatus = `saved ${project.name || project.folder_name}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function removeSelectedProject() {
    const project = selectedScriptProject();
    if (!selectedGroupName || !project) return;
    try {
      const group = await invoke<ScriptGroupSummary>("script_group_project_remove", {
        groupName: selectedGroupName,
        projectIndex: project.index,
      });
      selectedProjectIndex = group.projects[0]?.index ?? 0;
      scriptSettings = null;
      await loadScriptGroups();
      scriptSettingsStatus = `removed ${project.name || project.folder_name}`;
    } catch (err) {
      error = String(err);
    }
  }

  async function moveSelectedProject(delta: number) {
    const group = selectedScriptGroup();
    const project = selectedScriptProject();
    if (!selectedGroupName || !group || !project) return;
    const targetIndex = project.index + delta;
    await moveProjectTo(project.index, targetIndex);
  }

  async function moveProjectTo(projectIndex: number, targetIndex: number) {
    const group = selectedScriptGroup();
    if (!selectedGroupName || !group) return;
    if (targetIndex < 0 || targetIndex >= group.projects.length || projectIndex === targetIndex) return;
    const project = group.projects.find((item) => item.index === projectIndex);
    if (!project) return;
    try {
      const updated = await invoke<ScriptGroupSummary>("script_group_project_move", {
        groupName: selectedGroupName,
        projectIndex,
        targetIndex,
      });
      selectedGroupName = updated.name;
      selectedProjectIndex = targetIndex;
      scriptSettings = null;
      await loadScriptGroups();
      scriptSettingsStatus = `moved ${project.name || project.folder_name}`;
    } catch (err) {
      error = String(err);
    }
  }

  function dragScriptProject(event: DragEvent, projectIndex: number) {
    draggedProjectIndex = projectIndex;
    event.dataTransfer?.setData("text/plain", String(projectIndex));
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  async function dropScriptProject(event: DragEvent, targetIndex: number) {
    event.preventDefault();
    const sourceIndex = Number.parseInt(
      event.dataTransfer?.getData("text/plain") ?? String(draggedProjectIndex ?? ""),
      10,
    );
    draggedProjectIndex = null;
    if (!Number.isFinite(sourceIndex)) return;
    await moveProjectTo(sourceIndex, targetIndex);
  }

  async function loadScriptSettings() {
    try {
      scriptSettings = await invoke<ScriptSettingsDocument>("script_settings_document", {
        groupName: selectedGroupName,
        projectIndex: selectedProjectIndex,
      });
      selectedGroupName = scriptSettings.group_name;
      selectedProjectIndex = scriptSettings.project_index;
      scriptSettingsStatus = `${scriptSettings.manifest_name} settings loaded`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function saveScriptSettings() {
    if (!scriptSettings) return;
    try {
      const result = await invoke<ScriptSettingsSaveResult>("script_settings_save", {
        groupName: scriptSettings.group_name,
        projectIndex: scriptSettings.project_index,
        values: scriptSettings.values,
      });
      scriptSettings.values = result.settings;
      scriptSettings.cleaned_invalid_values = result.cleaned_invalid_values;
      scriptSettingsStatus = `saved ${result.project_folder_name}, cleaned ${result.cleaned_invalid_values}`;
      await loadScriptGroups();
    } catch (err) {
      error = String(err);
    }
  }

  async function runSelectedScriptProject(honorRunCount = false) {
    const project = selectedScriptProject();
    if (!selectedGroupName || !project) return;
    try {
      scriptRunBusy = true;
      scriptRunStatus = honorRunCount
        ? `running ${project.name || project.folder_name} x${project.run_num}`
        : `running ${project.name || project.folder_name}`;
      scriptRunResult = null;
      scriptGroupRunResult = await invoke<ScriptGroupExecutionOutcome>("script_execute_group_project", {
        groupName: selectedGroupName,
        projectIndex: project.index,
        settings: scriptSettings?.project_index === project.index ? scriptSettings.values : null,
        honorRunCount,
      });
      const step = scriptGroupRunResult.steps[0];
      scriptRunStatus = step
        ? `${step.project_type} ${step.status.toLowerCase()}: ${step.name || step.folder_name}`
        : "project run produced no steps";
      error = null;
    } catch (err) {
      scriptRunStatus = "run failed";
      error = String(err);
    } finally {
      scriptRunBusy = false;
    }
  }

  async function runSelectedScriptGroup() {
    if (!selectedGroupName) return;
    try {
      scriptRunBusy = true;
      scriptRunStatus = `running group ${selectedGroupName}`;
      scriptRunResult = null;
      scriptGroupRunResult = await invoke<ScriptGroupExecutionOutcome>("script_execute_group", {
        groupName: selectedGroupName,
      });
      scriptRunStatus = `group completed: ${scriptGroupRunResult.completed_steps} completed, ${scriptGroupRunResult.planned_steps} planned, ${scriptGroupRunResult.cancelled_steps} stopped, ${scriptGroupRunResult.failed_steps} failed`;
      error = null;
    } catch (err) {
      scriptRunStatus = "group run failed";
      error = String(err);
    } finally {
      scriptRunBusy = false;
    }
  }

  async function runSelectedScriptGroupFromProject() {
    const project = selectedScriptProject();
    if (!selectedGroupName || !project) return;
    try {
      scriptRunBusy = true;
      scriptRunStatus = `continuing ${selectedGroupName} from ${project.name || project.folder_name}`;
      scriptRunResult = null;
      scriptGroupRunResult = await invoke<ScriptGroupExecutionOutcome>("script_execute_group_from_project", {
        groupName: selectedGroupName,
        projectIndex: project.index,
      });
      scriptRunStatus = `continued group: ${scriptGroupRunResult.completed_steps} completed, ${scriptGroupRunResult.skipped_steps} skipped, ${scriptGroupRunResult.failed_steps} failed`;
      error = null;
    } catch (err) {
      scriptRunStatus = "continue failed";
      error = String(err);
    } finally {
      scriptRunBusy = false;
    }
  }

  async function stopScriptRun() {
    try {
      const result = await invoke<ScriptStopResult>("script_stop");
      scriptRunStatus = result.requested
        ? `stop requested: ${result.runner_state.current_task ?? "script"}`
        : "no running script to stop";
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function sendTestNotification() {
    try {
      notificationSendBusy = true;
      notificationSendStatus = "sending";
      await loadNotificationServiceState();
      notificationResult = await invoke<NotificationDispatchExecution>("notification_send_test", {
        payload: { message: notificationMessage, provider: notificationProvider || null },
      });
      const sent = notificationResult.deliveries.filter((delivery) => delivery.status === "Sent").length;
      const failed = notificationResult.deliveries.filter((delivery) => delivery.status === "Failed").length;
      const unsupported = notificationResult.deliveries.filter(
        (delivery) => delivery.status === "Unsupported",
      ).length;
      notificationSendStatus = notificationResult.attempted
        ? `${sent} sent, ${failed} failed, ${unsupported} unsupported`
        : `skipped: ${notificationResult.skipped_reason ?? "not subscribed"}`;
      error = null;
    } catch (err) {
      notificationSendStatus = "send failed";
      error = String(err);
    } finally {
      notificationSendBusy = false;
    }
  }

  async function loadNotificationServiceState() {
    try {
      notificationServiceState = await invoke<NotificationServiceState>("notification_service_state");
      notificationServiceStatus = notificationServiceState.initialized ? "initialized" : "pending";
      error = null;
    } catch (err) {
      notificationServiceStatus = "failed";
      error = String(err);
    }
  }

  async function refreshNotificationServiceState() {
    try {
      notificationServiceBusy = true;
      notificationServiceStatus = "refreshing";
      notificationServiceState = await invoke<NotificationServiceState>("notification_service_refresh");
      notificationServiceStatus = `${notificationServiceState.provider_count} provider(s)`;
      error = null;
    } catch (err) {
      notificationServiceStatus = "refresh failed";
      error = String(err);
    } finally {
      notificationServiceBusy = false;
    }
  }

  async function checkForUpdate() {
    try {
      updateCheckBusy = true;
      updateStatus = "checking";
      updateResult = await invoke<UpdateCheckResult>("update_check", {
        payload: { channel: updateChannel },
      });
      updateStatus = updateDecisionLabel(updateResult.decision.action);
      error = null;
    } catch (err) {
      updateStatus = "failed";
      error = String(err);
    } finally {
      updateCheckBusy = false;
    }
  }

  async function loadBackgroundUpdateState() {
    try {
      backgroundUpdateState = await invoke<BackgroundUpdateState>("update_background_state");
      if (backgroundUpdateState.last_result) {
        updateResult = backgroundUpdateState.last_result;
        updateStatus = backgroundUpdateState.running
          ? "background checking"
          : updateDecisionLabel(backgroundUpdateState.last_result.decision.action);
        if (backgroundUpdateState.last_result.decision.action === "OpenUpdateWindow") {
          backgroundUpdateNoticeOpen = true;
        }
      } else if (backgroundUpdateState.last_error) {
        updateStatus = "background check failed";
      } else if (backgroundUpdateState.running) {
        updateStatus = "background checking";
      }
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function openUpdateDownloadPage() {
    try {
      updateActionBusy = true;
      const result = await invoke<UpdateActionResult>("update_open_download_page", {
        channel: updateChannel,
      });
      updateStatus = result.detail;
      error = null;
    } catch (err) {
      error = String(err);
    } finally {
      updateActionBusy = false;
    }
  }

  async function launchUpdater(option: UpdaterLaunchPlan) {
    try {
      updateActionBusy = true;
      const result = await invoke<UpdateActionResult>("update_launch_updater", {
        payload: { source: option.source_arg ?? "default", exitAfterLaunch: updaterExitAfterLaunch },
      });
      updateStatus = result.exit_scheduled ? `${result.detail}; exiting` : result.detail;
      error = null;
    } catch (err) {
      error = String(err);
    } finally {
      updateActionBusy = false;
    }
  }

  async function ignoreDetectedUpdate() {
    const version = updateResult?.decision.new_version ?? updateResult?.latest_version;
    if (!version) return;
    try {
      updateActionBusy = true;
      const result = await invoke<UpdateActionResult>("update_ignore_version", { version });
      updateStatus = `ignored ${result.detail}`;
      if (updateResult) updateResult.ignored_version = result.detail;
      backgroundUpdateNoticeOpen = false;
      error = null;
    } catch (err) {
      error = String(err);
    } finally {
      updateActionBusy = false;
    }
  }

  async function checkRedeemCodeFeed() {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "checking";
      redeemFeedResult = await invoke<RedeemCodeFeedResult>("redeem_code_feed_check");
      redeemFeedStatus = redeemFeedResult.decision.has_update ? "new codes" : "up to date";
      const version = redeemFeedResult.decision.remote_version ?? redeemFeedResult.remote_text?.trim();
      if (redeemFeedResult.decision.has_update && version) {
        showRedeemFeedNotice(version);
      }
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function autoCheckRedeemCodeFeed() {
    if (redeemFeedAutoChecked) return;
    redeemFeedAutoChecked = true;
    try {
      const result = await invoke<RedeemCodeFeedResult>("redeem_code_feed_check");
      redeemFeedResult = result;
      redeemFeedStatus = result.decision.has_update ? "new codes" : "up to date";
      const version = result.decision.remote_version ?? result.remote_text?.trim();
      if (result.decision.has_update && version) {
        showRedeemFeedNotice(version);
      }
    } catch {
      // Match the legacy startup behavior: manual feed checks report errors, startup checks stay quiet.
    }
  }

  async function loadRedeemCodeFeedItems() {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "loading feed";
      redeemFeedItems = await invoke<RedeemCodeFeedItemsResult>("redeem_code_feed_items");
      redeemFeedStatus = `${redeemFeedItems.items.length} items`;
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function loadRedeemCodeLiveCodes() {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "loading live";
      redeemLiveResult = await invoke<RedeemCodeLiveResult>("redeem_code_live_codes");
      redeemFeedStatus =
        redeemLiveResult.data.codes.length > 0 ? `${redeemLiveResult.data.codes.length} live codes` : "no live codes";
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  function redeemPayloadForSource(source: "manual" | "feed" | "live") {
    return source === "manual"
      ? { text: redeemManualText }
      : source === "feed"
        ? { feedItems: redeemFeedItems?.items ?? [] }
        : { liveCodes: redeemLiveResult?.data.codes ?? [] };
  }

  async function planRedeemCodes(source: "manual" | "feed" | "live") {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "planning redeem";
      redeemPlanResult = await invoke<UseRedeemCodePlanResult>("redeem_code_auto_redeem_plan", {
        payload: redeemPayloadForSource(source),
      });
      redeemFeedStatus = `${redeemPlanResult.extracted_codes.length} codes planned`;
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function executeRedeemCodes(source: "manual" | "feed" | "live") {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "executing redeem";
      redeemExecutionResult = await invoke<UseRedeemCodeExecutionResult>("redeem_code_auto_redeem_execute", {
        payload: redeemPayloadForSource(source),
      });
      redeemPlanResult = {
        extracted_codes: redeemExecutionResult.extracted_codes,
        plan: redeemExecutionResult.plan,
      };
      redeemFeedStatus = `${redeemExecutionResult.extracted_codes.length} codes ${
        redeemExecutionResult.report.completed ? "completed" : "incomplete"
      }`;
      error = null;
    } catch (err) {
      redeemFeedStatus = "execution failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function loadRedeemClipboardState() {
    try {
      redeemClipboardState = await invoke<RedeemCodeClipboardState>("redeem_code_clipboard_state");
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  async function setRedeemClipboardEnabled(enabled: boolean) {
    try {
      redeemFeedBusy = true;
      redeemClipboardState = await invoke<RedeemCodeClipboardState>("redeem_code_clipboard_set_enabled", {
        enabled,
      });
      redeemFeedStatus = enabled ? "clipboard listener enabled" : "clipboard listener disabled";
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function checkRedeemClipboardText() {
    try {
      redeemFeedBusy = true;
      redeemFeedStatus = "checking clipboard text";
      redeemClipboardResult = await invoke<RedeemCodeClipboardCheckResult>("redeem_code_clipboard_check", {
        payload: { text: redeemManualText },
      });
      if (redeemClipboardResult.plan) {
        redeemPlanResult = {
          extracted_codes: redeemClipboardResult.extracted_codes,
          plan: redeemClipboardResult.plan,
        };
      }
      redeemFeedStatus = redeemClipboardResult.ignored
        ? "clipboard text ignored"
        : `${redeemClipboardResult.extracted_codes.length} clipboard codes`;
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function ignoreRedeemClipboardText() {
    try {
      redeemFeedBusy = true;
      redeemClipboardState = await invoke<RedeemCodeClipboardState>("redeem_code_clipboard_ignore", {
        payload: { text: redeemManualText },
      });
      redeemFeedStatus = "clipboard text ignored";
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  function clearRedeemFeedNoticeTimer() {
    if (!redeemFeedNoticeTimer) return;
    clearTimeout(redeemFeedNoticeTimer);
    redeemFeedNoticeTimer = null;
  }

  function showRedeemFeedNotice(version: string) {
    redeemFeedNoticeVersion = version;
    redeemFeedNoticeOpen = true;
    clearRedeemFeedNoticeTimer();
    redeemFeedNoticeTimer = setTimeout(() => {
      redeemFeedNoticeOpen = false;
      redeemFeedNoticeTimer = null;
    }, 20_000);
  }

  function dismissRedeemFeedNotice(event?: MouseEvent) {
    event?.stopPropagation();
    redeemFeedNoticeOpen = false;
    clearRedeemFeedNoticeTimer();
  }

  function dismissRedeemClipboardNotice(event?: MouseEvent) {
    event?.stopPropagation();
    redeemClipboardNoticeOpen = false;
  }

  function dismissScriptClipboardNotice(event?: MouseEvent) {
    event?.stopPropagation();
    scriptClipboardNoticeOpen = false;
  }

  function dismissBackgroundUpdateNotice(event?: MouseEvent) {
    event?.stopPropagation();
    backgroundUpdateNoticeOpen = false;
  }

  function openBackgroundUpdateFromNotice() {
    selected = "settings";
    updateChannel = updateResult?.channel === "Alpha" ? "alpha" : "stable";
    backgroundUpdateNoticeOpen = false;
  }

  function openScriptClipboardFromNotice() {
    selected = "scripts";
    scriptClipboardNoticeOpen = false;
  }

  function openRedeemClipboardFromNotice() {
    selected = "settings";
    redeemClipboardNoticeOpen = false;
  }

  async function openRedeemCodeFeedFromNotice() {
    const version = redeemFeedNoticeVersion;
    selected = "settings";
    dismissRedeemFeedNotice();
    if (version) {
      await markRedeemCodeFeedVersionRead(version);
    }
    await loadRedeemCodeFeedItems();
  }

  async function markRedeemCodeFeedVersionRead(version: string) {
    const trimmedVersion = version.trim();
    if (!trimmedVersion) return;
    try {
      redeemFeedBusy = true;
      redeemFeedResult = await invoke<RedeemCodeFeedResult>("redeem_code_feed_mark_read", {
        version: trimmedVersion,
      });
      redeemFeedStatus = "marked read";
      error = null;
    } catch (err) {
      redeemFeedStatus = "failed";
      error = String(err);
    } finally {
      redeemFeedBusy = false;
    }
  }

  async function markRedeemCodeFeedRead() {
    const version = redeemFeedResult?.decision.remote_version ?? redeemFeedResult?.remote_text;
    if (!version) return;
    await markRedeemCodeFeedVersionRead(version);
  }

  async function loadDesktopLog() {
    try {
      logState = await invoke<DesktopLogState>("desktop_log_state", { tail: 120 });
      logStatus = logState.exists ? `${logState.bytes} bytes` : "missing";
      error = null;
    } catch (err) {
      logStatus = "failed";
      error = String(err);
    }
  }

  async function loadDesktopShell() {
    try {
      shellBusy = true;
      shellState = await invoke<DesktopShellState>("desktop_shell_state");
      shellStatus = shellState.exit_to_tray ? "exit to tray" : "close exits";
      error = null;
    } catch (err) {
      shellStatus = "failed";
      error = String(err);
    } finally {
      shellBusy = false;
    }
  }

  async function setExitToTray(enabled: boolean) {
    try {
      shellBusy = true;
      shellState = await invoke<DesktopShellState>("desktop_shell_set_exit_to_tray", { enabled });
      shellStatus = shellState.exit_to_tray ? "exit to tray" : "close exits";
      error = null;
    } catch (err) {
      shellStatus = "failed";
      error = String(err);
    } finally {
      shellBusy = false;
    }
  }

  async function loadDesktopOverlay() {
    try {
      overlayBusy = true;
      overlayState = await invoke<DesktopOverlayState>("desktop_overlay_state");
      overlayStatus = overlayState.show_overlay_metrics
        ? `${overlayState.enabled_metric_keys.length} metric(s)`
        : "metrics hidden";
      error = null;
    } catch (err) {
      overlayStatus = "failed";
      error = String(err);
    } finally {
      overlayBusy = false;
    }
  }

  async function updateDesktopOverlay(payload: OverlayPatchPayload) {
    try {
      overlayBusy = true;
      overlayState = await invoke<DesktopOverlayState>("desktop_overlay_update", { payload });
      overlayStatus = payload.resetMetricsLayout
        ? "metrics layout reset"
        : `${overlayState.enabled_metric_keys.length} metric(s) enabled`;
      error = null;
    } catch (err) {
      overlayStatus = "update failed";
      error = String(err);
    } finally {
      overlayBusy = false;
    }
  }

  async function runDesktopShellAction(command: string) {
    try {
      shellBusy = true;
      const result = await invoke<DesktopShellActionResult>(command);
      shellStatus = result.detail;
      error = null;
    } catch (err) {
      shellStatus = "failed";
      error = String(err);
    } finally {
      shellBusy = false;
    }
  }

  async function runIndependentShellTask() {
    if (!independentShellCommand.trim()) return;
    try {
      independentShellBusy = true;
      shellStatus = "running shell task";
      independentShellResult = await invoke<DesktopShellTaskExecution>("task_execute_shell", {
        payload: {
          command: independentShellCommand,
          timeoutSeconds: independentShellTimeout,
          noWindow: true,
          output: true,
          disable: false,
          workingDirectory: null,
        },
      });
      shellStatus = `shell ${independentShellResult.result.status.toLowerCase()}`;
      error = null;
    } catch (err) {
      shellStatus = "shell failed";
      error = String(err);
    } finally {
      independentShellBusy = false;
    }
  }

  async function stopIndependentShellTask() {
    await stopIndependentTask("shell task");
  }

  async function stopIndependentLiveTask() {
    await stopIndependentTask("live task");
  }

  async function stopIndependentTask(fallbackTask: string) {
    try {
      const result = await invoke<ScriptStopResult>("script_stop");
      shellStatus = result.requested
        ? `stop requested: ${result.runner_state.current_task ?? fallbackTask}`
        : `no running ${fallbackTask} to stop`;
      error = null;
    } catch (err) {
      error = String(err);
    }
  }

  type IndependentLiveTaskCommand =
    | "task_execute_quick_buy"
    | "task_execute_quick_serenitea_pot"
    | "task_execute_auto_open_chest"
    | "task_execute_auto_cook"
    | "task_execute_auto_music_game_performance"
    | "task_execute_auto_music_game_album";

  function independentLiveTaskLabel(command: IndependentLiveTaskCommand) {
    switch (command) {
      case "task_execute_quick_buy":
        return "quick buy";
      case "task_execute_quick_serenitea_pot":
        return "quick serenitea pot";
      case "task_execute_auto_open_chest":
        return "auto open chest";
      case "task_execute_auto_cook":
        return "auto cook";
      case "task_execute_auto_music_game_performance":
        return "auto music performance";
      case "task_execute_auto_music_game_album":
        return "auto music album";
    }
  }

  function isAutoCookLiveTaskReport(
    report:
      | IndependentLiveTaskReport
      | AutoCookLiveTaskReport
      | AutoMusicPerformanceReport
      | AutoMusicAlbumReport
      | AutoOpenChestLiveTaskReport,
  ): report is AutoCookLiveTaskReport {
    return "status" in report && "state" in report && "events" in report;
  }

  function isAutoMusicPerformanceReport(
    report:
      | IndependentLiveTaskReport
      | AutoCookLiveTaskReport
      | AutoMusicPerformanceReport
      | AutoMusicAlbumReport
      | AutoOpenChestLiveTaskReport,
  ): report is AutoMusicPerformanceReport {
    return "stop_reason" in report && "frames_processed" in report && "held_keys_before_release" in report;
  }

  function isAutoMusicAlbumReport(
    report:
      | IndependentLiveTaskReport
      | AutoCookLiveTaskReport
      | AutoMusicPerformanceReport
      | AutoMusicAlbumReport
      | AutoOpenChestLiveTaskReport,
  ): report is AutoMusicAlbumReport {
    return "difficulty_count" in report && "songs_checked" in report && "performed_songs" in report;
  }

  function isAutoOpenChestReport(
    report:
      | IndependentLiveTaskReport
      | AutoCookLiveTaskReport
      | AutoMusicPerformanceReport
      | AutoMusicAlbumReport
      | AutoOpenChestLiveTaskReport,
  ): report is AutoOpenChestLiveTaskReport {
    return "status" in report && "decisions" in report && "dispatched_actions" in report;
  }

  async function runIndependentLiveTask(command: IndependentLiveTaskCommand) {
    const label = independentLiveTaskLabel(command);
    try {
      independentLiveTaskBusy = true;
      shellStatus = `running ${label}`;
      independentLiveTaskResult = await invoke<DesktopIndependentLiveTaskExecution>(command);
      if (isAutoCookLiveTaskReport(independentLiveTaskResult.result)) {
        shellStatus = `${label} ${independentLiveTaskResult.result.status}`;
      } else if (isAutoMusicPerformanceReport(independentLiveTaskResult.result)) {
        shellStatus = `${label} ${independentLiveTaskResult.result.stop_reason}`;
      } else if (isAutoMusicAlbumReport(independentLiveTaskResult.result)) {
        shellStatus = `${label} ${independentLiveTaskResult.result.status}`;
      } else if (isAutoOpenChestReport(independentLiveTaskResult.result)) {
        shellStatus = `${label} ${independentLiveTaskResult.result.status}`;
      } else {
        shellStatus = `${label} ${independentLiveTaskResult.result.completed ? "completed" : "incomplete"}`;
      }
      error = null;
    } catch (err) {
      shellStatus = `${label} failed`;
      error = String(err);
    } finally {
      independentLiveTaskBusy = false;
    }
  }

  async function planIndependentAutoPathingTask() {
    if (!selectedPathingScript) {
      await loadAvailablePathingScripts();
    }
    if (!selectedPathingScript) return;
    try {
      independentPathingBusy = true;
      independentPathingBoundaryResult = null;
      shellStatus = "planning auto-pathing";
      independentPathingResult = await invoke<DesktopAutoPathingTaskExecution>("task_plan_auto_pathing", {
        payload: { route: selectedPathingScript },
      });
      shellStatus = `pathing ${independentPathingResult.result.execution_plan.waypoint_count} waypoint(s)`;
      error = null;
    } catch (err) {
      shellStatus = "pathing plan failed";
      error = String(err);
    } finally {
      independentPathingBusy = false;
    }
  }

  async function runIndependentAutoPathingActionBoundary() {
    if (!selectedPathingScript) {
      await loadAvailablePathingScripts();
    }
    if (!selectedPathingScript) return;
    try {
      independentPathingBusy = true;
      independentPathingBoundaryResult = null;
      shellStatus = "running auto-pathing boundary";
      independentPathingBoundaryResult = await invoke<DesktopAutoPathingActionBoundaryExecution>(
        "task_execute_auto_pathing_action_boundary",
        {
          payload: { route: selectedPathingScript },
        },
      );
      const boundary = independentPathingBoundaryResult.boundary;
      independentPathingResult = {
        task: independentPathingBoundaryResult.task,
        result: independentPathingBoundaryResult.plan,
      };
      shellStatus = `pathing boundary ${boundary.movement_completion_status}`;
      error = null;
    } catch (err) {
      shellStatus = "pathing boundary failed";
      error = String(err);
    } finally {
      independentPathingBusy = false;
    }
  }

  async function planIndependentAutoFightTask() {
    try {
      independentFightBusy = true;
      shellStatus = "planning auto-fight";
      independentFightResult = await invoke<DesktopAutoFightTaskExecution>("task_plan_auto_fight", {
        payload: {
          strategyName: independentFightStrategy.trim() || null,
          teamNames: independentFightTeamNames.trim() || null,
        },
      });
      const scriptCount = independentFightResult.result.combat_scripts.scripts.length;
      shellStatus = `fight ${scriptCount} strategy script(s)`;
      error = null;
    } catch (err) {
      shellStatus = "fight plan failed";
      error = String(err);
    } finally {
      independentFightBusy = false;
    }
  }

  async function executeIndependentAutoFightTeamPlayback(sendInput: boolean, useLiveFrame = false) {
    try {
      independentFightPlaybackBusy = true;
      shellStatus = sendInput
        ? useLiveFrame
          ? "dispatching live team playback"
          : "dispatching team playback"
        : useLiveFrame
          ? "planning live team playback"
          : "planning team playback";
      independentFightPlaybackResult = await invoke<DesktopAutoFightTeamPlaybackResult>(
        "task_execute_auto_fight_team_playback",
        {
          payload: {
            strategyName: independentFightStrategy.trim() || null,
            teamNames: independentFightTeamNames.trim() || null,
            sendInput,
            useLiveFrame,
          },
        },
      );
      const playback = independentFightPlaybackResult.result;
      shellStatus = `${playback.playable_commands}/${playback.candidate_commands} team command(s) ready`;
      error = null;
    } catch (err) {
      shellStatus = "team playback failed";
      error = String(err);
    } finally {
      independentFightPlaybackBusy = false;
    }
  }

  async function detectIndependentAutoFightActiveAvatar() {
    try {
      independentFightAvatarBusy = true;
      shellStatus = "detecting active avatar";
      independentFightAvatarResult = await invoke<ActiveAvatarDetectionExecution>(
        "task_detect_auto_fight_active_avatar",
      );
      const result = independentFightAvatarResult.result;
      shellStatus = result.active_index ? `active avatar ${result.active_index}` : "active avatar unresolved";
      error = null;
    } catch (err) {
      shellStatus = "active avatar detection failed";
      error = String(err);
    } finally {
      independentFightAvatarBusy = false;
    }
  }

  async function probeIndependentAutoFightFinish(sendInput: boolean) {
    try {
      independentFightProbeBusy = true;
      shellStatus = sendInput ? "probing finish with input" : "capturing finish probe";
      independentFightProbeResult = await invoke<DesktopAutoFightFinishProbeExecution>(
        "task_probe_auto_fight_finish",
        {
          payload: {
            strategyName: independentFightStrategy.trim() || null,
            teamNames: independentFightTeamNames.trim() || null,
            sendInput,
          },
        },
      );
      const detection = independentFightProbeResult.result.detection;
      shellStatus = detection ? `finish ${detection.finished ? "detected" : "not detected"}` : "finish probe cancelled";
      error = null;
    } catch (err) {
      shellStatus = "finish probe failed";
      error = String(err);
    } finally {
      independentFightProbeBusy = false;
    }
  }

  function iconFor(key: string) {
    return iconByKey[key as keyof typeof iconByKey] ?? Home;
  }

  function triggerStateLabel(trigger: Trigger) {
    if (trigger.port_state === "CoreReady") return "core";
    if (trigger.port_state === "NativePending") return "native pending";
    return "metadata";
  }

  function isScriptRepoView() {
    return selected === "scripts" || selected === "pathing" || selected === "recorder";
  }

  function isNotificationView() {
    return selected === "notifications";
  }

  function isSettingsView() {
    return selected === "settings";
  }

  function updateDecisionLabel(action: UpdateDecision["action"]) {
    switch (action) {
      case "OpenUpdateWindow":
        return "update available";
      case "ShowUpToDateMessage":
        return "current";
      case "SuppressedByIgnoredVersion":
        return "ignored";
      case "Noop":
      default:
        return "no update";
    }
  }

  function timestampLabel(value: number | null | undefined) {
    return value ? new Date(value).toLocaleString() : "-";
  }

  function overlayLayoutLabel(layout: OverlayLayoutRect | null | undefined) {
    if (!layout) return "-";
    const left = Math.round(layout.left_ratio * 1000) / 10;
    const top = Math.round(layout.top_ratio * 1000) / 10;
    const width = Math.round(layout.width_ratio * 1000) / 10;
    const height = Math.round(layout.height_ratio * 1000) / 10;
    return `${left}%/${top}% · ${width}%x${height}%`;
  }

  function updateRequestDisplay(request: UpdateRequestPlan | null | undefined) {
    if (!request) return "-";
    const query = Object.entries(request.query)
      .map(([key, value]) => `${key}=${value}`)
      .join("&");
    return query ? `${request.url}?${query}` : request.url;
  }

  function selectedScriptGroup() {
    return scriptGroupsState.find((group) => group.name === selectedGroupName) ?? null;
  }

  function selectedScriptProject() {
    return selectedScriptGroup()?.projects.find((project) => project.index === selectedProjectIndex) ?? null;
  }

  function setSelectedProjectStatus(value: string) {
    const project = selectedScriptProject();
    if (project) project.status = value;
  }

  function setSelectedProjectSchedule(value: string) {
    const project = selectedScriptProject();
    if (project) project.schedule = value;
  }

  function setSelectedProjectRunNum(value: string) {
    const project = selectedScriptProject();
    if (project) {
      const parsed = Number.parseInt(value, 10);
      project.run_num = Number.isFinite(parsed) ? Math.max(1, parsed) : 1;
    }
  }

  function setSelectedProjectNotification(value: boolean) {
    const project = selectedScriptProject();
    if (project) project.allow_js_notification = value;
  }

  function visibleRepoNodes() {
    const nodes = repoState?.index_nodes ?? [];
    const query = repoSearch.trim().toLowerCase();
    if (!query) return nodes.slice(0, 240);
    return nodes
      .filter(
        (node) =>
          node.path.toLowerCase().includes(query) ||
          node.name.toLowerCase().includes(query) ||
          node.node_type.toLowerCase().includes(query),
      )
      .slice(0, 240);
  }

  function toggleImportPath(path: string) {
    selectedImportPaths = selectedImportPaths.includes(path)
      ? selectedImportPaths.filter((item) => item !== path)
      : [...selectedImportPaths, path];
  }

  function importResultLabel(label: string, result: RepoImportResult) {
    return `${label}: ${result.imported_targets} targets, ${result.dependency_files_copied} dependencies, ${result.git_checkouts} git checkouts`;
  }

  function settingLabel(item: ScriptSettingItem) {
    return item.label || item.name;
  }

  function settingStringValue(name: string) {
    const value = scriptSettings?.values[name];
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    return "";
  }

  function setSettingString(name: string, value: string) {
    if (!scriptSettings) return;
    scriptSettings.values = { ...scriptSettings.values, [name]: value };
  }

  function settingBoolValue(name: string) {
    return scriptSettings?.values[name] === true;
  }

  function setSettingBool(name: string, value: boolean) {
    if (!scriptSettings) return;
    scriptSettings.values = { ...scriptSettings.values, [name]: value };
  }

  function settingArrayValue(name: string) {
    const value = scriptSettings?.values[name];
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
  }

  function toggleSettingArrayValue(name: string, option: string) {
    if (!scriptSettings) return;
    const current = settingArrayValue(name);
    const next = current.includes(option)
      ? current.filter((item) => item !== option)
      : [...current, option];
    scriptSettings.values = { ...scriptSettings.values, [name]: next };
  }

  function cascadeFirstValue(item: ScriptSettingItem) {
    const value = settingStringValue(item.name);
    const groups = Object.entries(item.cascadeOptions ?? {});
    return groups.find(([, values]) => values.includes(value))?.[0] ?? groups[0]?.[0] ?? "";
  }

  function cascadeSecondOptions(item: ScriptSettingItem, first: string) {
    return item.cascadeOptions?.[first] ?? [];
  }

  function setCascadeFirst(item: ScriptSettingItem, first: string) {
    const next = cascadeSecondOptions(item, first)[0] ?? "";
    setSettingString(item.name, next);
  }

  function scriptRunResultJson() {
    if (scriptRunResult?.result === undefined || scriptRunResult?.result === null) return "undefined";
    return JSON.stringify(scriptRunResult.result, null, 2);
  }

  function scriptRunHostCallLabel(call: ScriptHostCall) {
    return `${call.target}.${call.method}`;
  }

  function objectValue(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    return value as Record<string, unknown>;
  }

  function maybeKeyMouseExecution(value: unknown): KeyMouseExecution | null {
    const root = objectValue(value);
    const plan = objectValue(root?.plan);
    const summary = objectValue(plan?.summary);
    if (!root || !plan || !summary || !("input_events" in plan)) return null;
    return value as KeyMouseExecution;
  }

  function maybePathingExecution(value: unknown): PathingExecution | null {
    const root = objectValue(value);
    const plan = objectValue(root?.plan);
    const executionPlan = objectValue(root?.execution_plan);
    if (!root || !plan || !executionPlan || !("segment_count" in executionPlan)) return null;
    return value as PathingExecution;
  }

  function maybeGlobalInputExecution(value: unknown): GlobalInputExecution | null {
    const root = objectValue(value);
    if (!root || !("events" in root) || !("dispatched_events" in root)) return null;
    return value as GlobalInputExecution;
  }

  function maybeHttpExecution(value: unknown): HttpExecution | null {
    const root = objectValue(value);
    const request = objectValue(root?.request);
    if (!root || !request || !("response" in root) || !("url" in request)) return null;
    return value as HttpExecution;
  }

  function maybeNotificationExecution(value: unknown): NotificationExecution | null {
    const root = objectValue(value);
    const record = objectValue(root?.record);
    if (!root || !record || !("message" in record) || !("dispatched" in root)) return null;
    return value as NotificationExecution;
  }

  function maybeHtmlMaskDesktopDispatch(value: unknown): HtmlMaskDesktopDispatch | null {
    const root = objectValue(value);
    const dispatch = objectValue(root?.desktop_html_mask);
    if (!dispatch || typeof dispatch.action !== "string" || typeof dispatch.dispatched !== "boolean") return null;
    return dispatch as HtmlMaskDesktopDispatch;
  }

  function maybeAutoPathingBoundaryFromInvocation(
    result: TaskInvocationExecutionResult,
  ): AutoPathingActionBoundaryReport | null {
    const live = objectValue(result.independent_task_live_execution);
    const boundary = objectValue(live?.AutoPathingActionBoundary);
    if (!boundary || typeof boundary.completion_scope !== "string") return null;
    return boundary as AutoPathingActionBoundaryReport;
  }

  function keyMouseExecutionSummary(execution: KeyMouseExecution): ExecutionSummary {
    return {
      title: "KeyMouse",
      meta: `${execution.mode} · ${execution.dispatched ? "dispatched" : "planned"}`,
      details: [
        { label: "Macro", value: `${execution.plan.summary.event_count} events` },
        { label: "Input", value: `${execution.plan.input_events.length} native events` },
        { label: "Duration", value: `${execution.plan.summary.duration_ms} ms` },
        { label: "Sent", value: `${execution.dispatched_events}` },
        { label: "Stopped", value: execution.cancelled ? "yes" : "no" },
        { label: "Keys", value: `${execution.plan.summary.key_events}` },
        {
          label: "Mouse",
          value: `${execution.plan.summary.absolute_mouse_events + execution.plan.summary.relative_mouse_events}`,
        },
      ],
    };
  }

  function pathingExecutionSummary(execution: PathingExecution): ExecutionSummary {
    const actions = execution.plan.summary.actions.length;
    return {
      title: execution.plan.summary.name || "Pathing",
      meta: `${execution.plan.source} · ${execution.dispatched ? "dispatched" : "prepared"}`,
      details: [
        { label: "Map", value: execution.execution_plan.map_name || execution.plan.summary.map_name || "-" },
        { label: "Segments", value: `${execution.execution_plan.segment_count}` },
        { label: "Waypoints", value: `${execution.execution_plan.waypoint_count}` },
        { label: "Actions", value: `${actions}` },
        { label: "Fights", value: `${execution.execution_plan.expected_fight_count}` },
        { label: "AutoPick", value: execution.execution_plan.autopick_realtime_trigger_enabled ? "on" : "off" },
      ],
    };
  }

  function autoPathingExecutionSummary(execution: AutoPathingExecutionPlan): ExecutionSummary {
    const movement = execution.execution_plan.movement_contract;
    return {
      title: execution.summary.name || "AutoPathing",
      meta: `${execution.source} · ${execution.dispatched ? "dispatched" : "prepared"}`,
      details: [
        { label: "Map", value: execution.execution_plan.map_name || execution.summary.map_name || "-" },
        { label: "Segments", value: `${execution.execution_plan.segment_count}` },
        { label: "Waypoints", value: `${execution.execution_plan.waypoint_count}` },
        { label: "Actions", value: `${execution.execution_plan.action_count}` },
        { label: "Fights", value: `${execution.execution_plan.expected_fight_count}` },
        {
          label: "Movement",
          value: movement.movement_executor_ready ? "ready" : "native pending",
        },
        { label: "Pending", value: `${movement.pending_dependencies.length}` },
        { label: "AutoPick", value: execution.execution_plan.autopick_realtime_trigger_enabled ? "on" : "off" },
      ],
    };
  }

  function pathingBoundaryStatusCounts(reports: Array<{ status: PathingBoundaryStatus }>) {
    return reports.reduce(
      (counts, report) => {
        counts[report.status] += 1;
        return counts;
      },
      {
        reported: 0,
        executed: 0,
        skipped: 0,
        unsupported: 0,
        invalid: 0,
      } satisfies Record<PathingBoundaryStatus, number>,
    );
  }

  function autoPathingBoundarySummary(
    execution: DesktopAutoPathingActionBoundaryExecution,
  ): ExecutionSummary {
    const boundary = execution.boundary;
    const movement = boundary.movement_report;
    const actionCounts = pathingBoundaryStatusCounts(
      boundary.waypoint_reports.flatMap((waypoint) =>
        waypoint.action_report ? [waypoint.action_report] : [],
      ),
    );
    const phaseCounts = pathingBoundaryStatusCounts(
      boundary.waypoint_reports.flatMap((waypoint) => waypoint.phase_reports),
    );
    return {
      title: execution.task,
      meta: `${boundary.completion_scope} · ${boundary.movement_completion_status}`,
      details: [
        { label: "Boundary", value: boundary.boundary_completed ? "done" : "pending" },
        { label: "Movement", value: boundary.movement_attempted ? "attempted" : "not attempted" },
        {
          label: "Move Contract",
          value: movement?.movement_contract_consumed ? "consumed" : "not consumed",
        },
        { label: "Native Done", value: boundary.native_pathing_completed ? "yes" : "no" },
        { label: "Contract", value: `v${boundary.movement_contract_version}` },
        { label: "Segments", value: `${boundary.movement_segment_count}` },
        { label: "Waypoints", value: `${boundary.movement_waypoint_count}` },
        { label: "Pending", value: `${boundary.movement_pending_dependencies.length}` },
        {
          label: "Move Phases",
          value: movement
            ? `${movement.executed_phases}/${movement.skipped_phases}/${movement.unsupported_phases}/${movement.failed_phases}/${movement.cancelled_phases}`
            : "-",
        },
        {
          label: "Move Failed",
          value: movement?.failed_phase
            ? `${movement.failed_phase.phase} #${movement.failed_phase.global_index}`
            : "-",
        },
        { label: "Action Reported", value: `${actionCounts.reported}` },
        { label: "Action Done", value: `${actionCounts.executed}` },
        { label: "Action Skip", value: `${actionCounts.skipped}` },
        { label: "Action Pending", value: `${actionCounts.unsupported}` },
        { label: "Action Invalid", value: `${actionCounts.invalid}` },
        { label: "Phase Reported", value: `${phaseCounts.reported}` },
        { label: "Native Phases", value: `${boundary.unsupported_phases}` },
      ],
    };
  }

  function autoFightExecutionSummary(execution: DesktopAutoFightTaskExecution["result"]): ExecutionSummary {
    const scripts = execution.combat_scripts.scripts;
    const commandCount = scripts.reduce((sum, script) => sum + script.commands.length, 0);
    const avatarCount = new Set(scripts.flatMap((script) => script.avatar_names)).size;
    const playback = execution.playback_evaluation;
    const selection = execution.team_selection;
    const teamPlan = execution.team_plan;
    const teamPlayback = execution.team_playback;
    const fightLoop = execution.fight_loop_plan;
    const enabledLoopSteps = fightLoop.steps.filter((step) => step.enabled);
    const finishPlan = execution.finish_detection_plan;
    const finishSteps = finishPlan.steps.filter((step) => step.enabled);
    const schedulerConfigured = execution.action_scheduler_plans.reduce(
      (sum, script) => sum + script.scheduler.configured_avatar_names.length,
      0,
    );
    const schedulerSkipped = execution.action_scheduler_plans.reduce(
      (sum, script) => sum + script.scheduler.skipped_avatar_names.length,
      0,
    );
    return {
      title: "AutoFight",
      meta: `${execution.dispatched ? "dispatched" : "prepared"} · ${execution.param.combat_strategy_path}`,
      details: [
        { label: "Scripts", value: `${scripts.length}` },
        { label: "Commands", value: `${commandCount}` },
        { label: "Static", value: `${playback.static_ready_commands}` },
        { label: "Context", value: `${playback.context_bound_commands}` },
        { label: "Input", value: `${playback.default_input_event_count}` },
        { label: "Dispatch", value: playback.dispatch_ready ? "ready" : "pending" },
        { label: "Loop", value: fightLoop.native_dispatch_ready ? "ready" : "planned" },
        { label: "Loop Steps", value: `${enabledLoopSteps.length}` },
        { label: "Finish Probe", value: finishPlan.native_ready_without_capture ? "ready" : "capture" },
        { label: "Probe Steps", value: `${finishSteps.length}` },
        { label: "Avatars", value: `${avatarCount}` },
        { label: "Team", value: selection.status },
        { label: "Team Meta", value: teamPlan ? `${teamPlan.avatars.length}` : "-" },
        { label: "Matched", value: `${selection.matched_avatar_count}` },
        { label: "Executable", value: `${selection.executable_commands.length}` },
        { label: "Team Play", value: teamPlayback ? (teamPlayback.dispatch_ready ? "ready" : "blocked") : "-" },
        { label: "Playable", value: teamPlayback ? `${teamPlayback.playable_commands}` : "-" },
        { label: "Runtime Skip", value: teamPlan ? `${teamPlan.can_be_skipped_avatar_names.length}` : "-" },
        { label: "All Skip", value: teamPlan?.all_command_avatars_can_be_skipped ? "yes" : "no" },
        { label: "CD Config", value: `${schedulerConfigured}` },
        { label: "CD Skip", value: `${schedulerSkipped}` },
        { label: "Failures", value: `${execution.combat_scripts.parse_failures.length}` },
        { label: "Timeout", value: `${execution.param.timeout}s` },
        { label: "Finish", value: execution.param.fight_finish_detect_enabled ? "detect" : "off" },
      ],
    };
  }

  function autoFightTeamPlaybackSummary(execution: AutoFightTeamPlaybackExecution): ExecutionSummary {
    return {
      title: "Team Playback",
      meta: `${execution.mode} · ${execution.dispatch_ready ? "ready" : "blocked"}`,
      details: [
        { label: "Script", value: execution.script_name },
        { label: "Candidates", value: `${execution.candidate_commands}` },
        { label: "Playable", value: `${execution.playable_commands}` },
        { label: "Events", value: `${execution.input_events.length}` },
        { label: "Sent", value: `${execution.dispatched_events}` },
        { label: "Blocked", value: execution.blocked_command_index === null ? "-" : `${execution.blocked_command_index}` },
        { label: "Needs", value: execution.blocked_requirements.length ? execution.blocked_requirements.join(", ") : "-" },
        { label: "Cancelled", value: execution.cancelled ? "yes" : "no" },
      ],
    };
  }

  function activeAvatarSummary(execution: ActiveAvatarDetectionExecution["result"]): ExecutionSummary {
    return {
      title: "Active Avatar",
      meta: `${execution.method} · ${execution.active_index ?? "unresolved"}`,
      details: [
        { label: "Index", value: execution.active_index === null ? "-" : `${execution.active_index}` },
        { label: "Rects", value: `${execution.rects.length}` },
        { label: "White", value: `${execution.white_rect_count}` },
        { label: "Not White", value: execution.not_white_rect_index === null ? "-" : `${execution.not_white_rect_index}` },
        { label: "Edges", value: execution.edge_white_ratios.map((ratio) => ratio.toFixed(2)).join(", ") || "-" },
        { label: "Votes", value: execution.difference_votes.join(", ") || "-" },
      ],
    };
  }

  function autoFightFinishProbeSummary(execution: AutoFightFinishDetectionExecution): ExecutionSummary {
    const detection = execution.detection;
    const progress = detection
      ? `${detection.progress_pixel.r}/${detection.progress_pixel.g}/${detection.progress_pixel.b}`
      : "-";
    const white = detection
      ? `${detection.white_tile_pixel.r}/${detection.white_tile_pixel.g}/${detection.white_tile_pixel.b}`
      : "-";
    return {
      title: "Finish Probe",
      meta: `${execution.mode} · ${execution.captured ? "captured" : "not captured"}`,
      details: [
        { label: "Finished", value: detection ? (detection.finished ? "yes" : "no") : "-" },
        { label: "Progress", value: progress },
        { label: "White Tile", value: white },
        { label: "Before", value: `${execution.before_capture_events.length}` },
        { label: "After", value: `${execution.after_detection_events.length}` },
        { label: "Sent", value: `${execution.dispatched_events}` },
        { label: "Cancelled", value: execution.cancelled ? "yes" : "no" },
      ],
    };
  }

  function globalInputExecutionSummary(execution: GlobalInputExecution): ExecutionSummary {
    return {
      title: "Global Input",
      meta: `${execution.mode} · ${execution.dispatched ? "dispatched" : "planned"}`,
      details: [
        { label: "Events", value: `${execution.events.length}` },
        { label: "Sent", value: `${execution.dispatched_events}` },
      ],
    };
  }

  function httpExecutionSummary(execution: HttpExecution): ExecutionSummary {
    return {
      title: "HTTP",
      meta: `${execution.mode} · ${execution.dispatched ? "dispatched" : "planned"}`,
      details: [
        { label: "Method", value: execution.request.method },
        { label: "Status", value: execution.response ? `${execution.response.status_code}` : "-" },
        { label: "Headers", value: `${execution.request.headers.length}` },
      ],
    };
  }

  function shellExecutionSummary(execution: ShellExecution): ExecutionSummary {
    return {
      title: "Shell",
      meta: `${execution.status} · ${execution.waited_for_exit ? "waited" : "started"}`,
      details: [
        { label: "Timeout", value: `${execution.timeout_seconds}s` },
        { label: "Exit", value: execution.exit_code === null ? "-" : `${execution.exit_code}` },
        { label: "Output", value: execution.output_enabled ? "captured" : "disabled" },
        { label: "Window", value: execution.no_window ? "hidden" : "visible" },
      ],
    };
  }

  function independentLiveTaskSummary(execution: DesktopIndependentLiveTaskExecution): ExecutionSummary {
    if (isAutoCookLiveTaskReport(execution.result)) {
      return {
        title: execution.task,
        meta: execution.result.status,
        details: [
          { label: "Frames", value: `${execution.result.state.frames_processed ?? 0}` },
          { label: "Space", value: `${execution.result.state.space_press_count ?? 0}` },
          { label: "Confirm", value: `${execution.result.state.white_confirm_click_count ?? 0}` },
          { label: "Events", value: `${execution.result.events.length}` },
        ],
      };
    }

    if (isAutoMusicPerformanceReport(execution.result)) {
      return {
        title: execution.task,
        meta: execution.result.stop_reason,
        details: [
          { label: "Frames", value: `${execution.result.frames_processed}` },
          { label: "Held", value: `${execution.result.held_keys_before_release.length}` },
          { label: "Events", value: `${execution.result.events.length}` },
        ],
      };
    }

    if (isAutoMusicAlbumReport(execution.result)) {
      return {
        title: execution.task,
        meta: execution.result.status,
        details: [
          { label: "Difficulties", value: `${execution.result.difficulty_count}` },
          { label: "Songs", value: `${execution.result.songs_checked}` },
          { label: "Played", value: `${execution.result.performed_songs}` },
          { label: "Skipped", value: `${execution.result.skipped_songs}` },
          { label: "Events", value: `${execution.result.events.length}` },
        ],
      };
    }

    if (isAutoOpenChestReport(execution.result)) {
      return {
        title: execution.task,
        meta: execution.result.status,
        details: [
          { label: "Iterations", value: `${execution.result.state.iterations ?? 0}` },
          {
            label: "Actions",
            value: `${
              execution.result.dispatched_actions.length +
              execution.result.cleanup_actions.length +
              execution.result.post_loop_actions.length
            }`,
          },
          { label: "Elapsed", value: `${execution.result.state.elapsed_ms ?? 0}ms` },
          {
            label: "Result",
            value:
              execution.result.state.final_result === undefined || execution.result.state.final_result === null
                ? "-"
                : String(execution.result.state.final_result),
          },
        ],
      };
    }

    const stateResult = execution.result.state?.result;
    return {
      title: execution.task,
      meta: execution.result.completed ? "completed" : "incomplete",
      details: [
        { label: "Executed", value: `${execution.result.executed_steps.length}` },
        { label: "Skipped", value: `${execution.result.skipped_steps.length}` },
        { label: "Result", value: stateResult === undefined || stateResult === null ? "-" : String(stateResult) },
      ],
    };
  }

  function redeemReportList(result: UseRedeemCodeExecutionResult, key: string): string[] {
    const value = result.report.state[key];
    return Array.isArray(value) ? value.map(String) : [];
  }

  function notificationExecutionSummary(execution: NotificationExecution): ExecutionSummary {
    const appDispatch = execution.app_dispatch;
    const providerSent =
      appDispatch?.deliveries.filter((delivery) => delivery.status === "Sent").length ?? 0;
    const providerFailed =
      appDispatch?.deliveries.filter((delivery) => delivery.status === "Failed").length ?? 0;
    const providerUnsupported =
      appDispatch?.deliveries.filter((delivery) => delivery.status === "Unsupported").length ?? 0;
    const details = [
      { label: "Kind", value: execution.record.kind },
      { label: "Event", value: execution.delivery?.event_code ?? "-" },
      { label: "Time", value: `${execution.record.timestamp_ms}` },
    ];
    if (appDispatch) {
      details.push(
        { label: "Providers", value: `${appDispatch.deliveries.length}` },
        { label: "Sent", value: `${providerSent}` },
        { label: "Failed", value: `${providerFailed + providerUnsupported}` },
      );
    } else if (execution.app_dispatch_error) {
      details.push({ label: "App", value: execution.app_dispatch_error });
    }

    return {
      title: "Notification",
      meta: `${execution.mode} · ${execution.dispatched ? "delivered" : "recorded"}`,
      details,
    };
  }

  function htmlMaskDesktopDispatchSummary(dispatch: HtmlMaskDesktopDispatch): ExecutionSummary {
    return {
      title: "HtmlMask",
      meta: `${dispatch.action} · ${dispatch.dispatched ? "dispatched" : "pending"}`,
      details: [
        { label: "Window", value: dispatch.windowId ?? "-" },
        { label: "Label", value: dispatch.windowLabel ?? "-" },
        { label: "Message", value: dispatch.message },
      ],
    };
  }

  function taskInvocationTotal(invocations: JavaScriptTaskInvocations | null | undefined) {
    if (!invocations) return 0;
    return invocations.dispatcher.length + invocations.genshin.length;
  }

  function taskExecutionResults(execution: JavaScriptTaskExecution | null | undefined) {
    if (!execution) return [];
    return [...execution.dispatcher, ...execution.genshin];
  }

  function taskInvocationSummary(
    invocations: JavaScriptTaskInvocations,
    execution?: JavaScriptTaskExecution,
  ): ExecutionSummary | null {
    const total = taskInvocationTotal(invocations);
    if (total === 0 && invocations.errors.length === 0) return null;
    const firstPlans = [...invocations.dispatcher, ...invocations.genshin].slice(0, 3);
    const executionResults = taskExecutionResults(execution);
    const nativePending = executionResults.filter((result) => result.status === "NativePending").length;
    const runtimeOnly = executionResults.filter((result) => result.status === "RuntimeOnly").length;
    const liveCompleted = executionResults.filter((result) => result.live_completed === true).length;
    const livePartial = executionResults.filter((result) => result.live_completed === false).length;
    const autoPathingBoundary = executionResults
      .map(maybeAutoPathingBoundaryFromInvocation)
      .find((boundary): boundary is AutoPathingActionBoundaryReport => boundary !== null);
    const details = [
      { label: "Dispatcher", value: `${invocations.dispatcher.length}` },
      { label: "Genshin", value: `${invocations.genshin.length}` },
      { label: "Pending", value: `${nativePending}` },
      { label: "Runtime", value: `${runtimeOnly}` },
      { label: "Live Done", value: `${liveCompleted}` },
      { label: "Live Partial", value: `${livePartial}` },
      {
        label: "First",
        value: firstPlans.map((plan) => plan.task_key ?? plan.kind).join(", ") || "-",
      },
    ];
    if (autoPathingBoundary) {
      details.push(
        { label: "Pathing Scope", value: autoPathingBoundary.completion_scope },
        { label: "Pathing Move", value: autoPathingBoundary.movement_completion_status },
        { label: "Pathing Done", value: autoPathingBoundary.native_pathing_completed ? "yes" : "no" },
      );
    }
    return {
      title: "Task Invocations",
      meta: `${execution?.mode ?? "PlanOnly"} · ${total} planned${invocations.errors.length > 0 ? ` · ${invocations.errors.length} errors` : ""}`,
      details,
    };
  }

  function hostCallExecutionSummary(call: ScriptHostCall): ExecutionSummary | null {
    const keyMouse = maybeKeyMouseExecution(call.result);
    if (keyMouse) return keyMouseExecutionSummary(keyMouse);
    const pathing = maybePathingExecution(call.result);
    if (pathing) return pathingExecutionSummary(pathing);
    const globalInput = maybeGlobalInputExecution(call.result);
    if (globalInput) return globalInputExecutionSummary(globalInput);
    const http = maybeHttpExecution(call.result);
    if (http) return httpExecutionSummary(http);
    const notification = maybeNotificationExecution(call.result);
    if (notification) return notificationExecutionSummary(notification);
    const htmlMask = maybeHtmlMaskDesktopDispatch(call.result);
    if (htmlMask) return htmlMaskDesktopDispatchSummary(htmlMask);
    return null;
  }

  function scriptGroupStepPayload(step: ScriptGroupStepExecutionOutcome) {
    if (step.javascript) {
      return {
        result: step.javascript.result ?? "undefined",
        logs: step.javascript.logs,
        host_calls: step.javascript.host_calls.length,
        task_invocations: step.javascript.task_invocations,
        task_execution: step.javascript.task_execution,
      };
    }
    if (step.key_mouse_execution) return step.key_mouse_execution;
    if (step.pathing_execution) return step.pathing_execution;
    if (step.shell_result) return step.shell_result;
    if (step.skip_reason) return step.skip_reason;
    return step.error ?? "No payload.";
  }

  function scriptGroupStepExecutionSummary(step: ScriptGroupStepExecutionOutcome): ExecutionSummary | null {
    if (step.key_mouse_execution) return keyMouseExecutionSummary(step.key_mouse_execution);
    if (step.pathing_execution) return pathingExecutionSummary(step.pathing_execution);
    if (step.shell_result) return shellExecutionSummary(step.shell_result);
    if (step.javascript) {
      return taskInvocationSummary(step.javascript.task_invocations, step.javascript.task_execution);
    }
    return null;
  }
</script>

{#snippet pathingTree(nodes: AvailablePathingTreeNode[], depth = 0)}
  {#each nodes as node}
    {#if node.route}
      <button
        type="button"
        class:selected={selectedPathingScript === node.route.relative_path}
        style={`--depth: ${depth}`}
        onclick={() => (selectedPathingScript = node.route?.relative_path ?? selectedPathingScript)}
      >
        <span>{node.name}</span>
        <em>{node.route.relative_path}</em>
      </button>
    {:else}
      <div class="pathing-folder" style={`--depth: ${depth}`}>
        <span>{node.name}</span>
      </div>
      {@render pathingTree(node.children, depth + 1)}
    {/if}
  {/each}
{/snippet}

<main class="shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">BG</div>
      <div>
        <h1>BetterGI Rust</h1>
        <p>{state?.ui_shell.shell ?? "Rust desktop"}</p>
      </div>
    </div>

    <nav aria-label="Primary">
      {#if state}
        {#each state.navigation as item}
          {@const Icon = iconFor(item.key)}
          <button class:selected={selected === item.key} onclick={() => (selected = item.key)}>
            <Icon size={18} />
            <span>{item.label}</span>
          </button>
          {#if item.children.length > 0}
            <div class="subnav">
              {#each item.children as child}
                {@const ChildIcon = iconFor(child.key)}
                <button
                  class:selected={selected === child.key}
                  onclick={() => (selected = child.key)}
                >
                  <ChildIcon size={16} />
                  <span>{child.label}</span>
                </button>
              {/each}
            </div>
          {/if}
        {/each}
      {/if}
    </nav>
  </aside>

  <section class="workspace">
    <header class="topbar">
      <div>
        <h2>Migration Console</h2>
        <p>{state?.ui_shell.frontend ?? "Svelte + TypeScript"} command surface for the Rust port.</p>
      </div>
      <button class="primary" onclick={load}>
        <Activity size={17} />
        Refresh
      </button>
    </header>

    {#if redeemFeedNoticeOpen}
      <section class="redeem-notice-card" aria-live="polite">
        <button class="redeem-notice-main" onclick={openRedeemCodeFeedFromNotice}>
          <span class="redeem-notice-accent" aria-hidden="true"></span>
          <span class="redeem-notice-copy">
            <strong>Redeem codes updated</strong>
            <small>{redeemFeedNoticeVersion ? `Version ${redeemFeedNoticeVersion}` : "Open feed details"}</small>
          </span>
        </button>
        <button
          type="button"
          class="redeem-notice-close"
          aria-label="Dismiss redeem code update"
          onclick={dismissRedeemFeedNotice}
        >
          <X size={16} />
        </button>
      </section>
    {/if}

    {#if redeemClipboardNoticeOpen && redeemClipboardResult}
      <section class="redeem-notice-card clipboard" aria-live="polite">
        <button class="redeem-notice-main" onclick={openRedeemClipboardFromNotice}>
          <span class="redeem-notice-accent" aria-hidden="true"></span>
          <span class="redeem-notice-copy">
            <strong>Redeem codes detected</strong>
            <small>{`${redeemClipboardResult.extracted_codes.length} code(s) from system clipboard`}</small>
          </span>
        </button>
        <button
          type="button"
          class="redeem-notice-close"
          aria-label="Dismiss clipboard redeem code detection"
          onclick={dismissRedeemClipboardNotice}
        >
          <X size={16} />
        </button>
      </section>
    {/if}

    {#if scriptClipboardNoticeOpen && scriptClipboardResult}
      <section class="redeem-notice-card script-import" aria-live="polite">
        <button class="redeem-notice-main" onclick={openScriptClipboardFromNotice}>
          <span class="redeem-notice-accent" aria-hidden="true"></span>
          <span class="redeem-notice-copy">
            <strong>Script import detected</strong>
            <small>{`${scriptClipboardResult.paths.length} path(s) from system clipboard`}</small>
          </span>
        </button>
        <button
          type="button"
          class="redeem-notice-close"
          aria-label="Dismiss script import detection"
          onclick={dismissScriptClipboardNotice}
        >
          <X size={16} />
        </button>
      </section>
    {/if}

    {#if backgroundUpdateNoticeOpen && updateResult}
      <section class="redeem-notice-card update-check" aria-live="polite">
        <button class="redeem-notice-main" onclick={openBackgroundUpdateFromNotice}>
          <span class="redeem-notice-accent" aria-hidden="true"></span>
          <span class="redeem-notice-copy">
            <strong>Version update available</strong>
            <small>{`${updateResult.app_version} -> ${updateResult.decision.new_version ?? updateResult.latest_version ?? "latest"}`}</small>
          </span>
        </button>
        <button
          type="button"
          class="redeem-notice-close"
          aria-label="Dismiss version update notice"
          onclick={dismissBackgroundUpdateNotice}
        >
          <X size={16} />
        </button>
      </section>
    {/if}

    {#if error}
      <section class="notice error">{error}</section>
    {:else if !state}
      <section class="notice">Loading runtime state...</section>
    {:else}
      {#if isScriptRepoView()}
        <section class="repo-toolbar">
          <label>
            <span>Repository</span>
            <input bind:value={repoPath} />
          </label>
          <button class="primary" onclick={loadRepoState}>
            <Search size={17} />
            Load
          </button>
          <button onclick={loadRepoJson}>Index</button>
          <button onclick={clearRepoUpdates}>Clear Updates</button>
          <label class="repo-toggle">
            <input type="checkbox" bind:checked={repoGitMode} />
            <span>Git</span>
          </label>
        </section>

        <section class="repo-actions">
          <div class="repo-action">
            <label>
              <span>Git URL</span>
              <input bind:value={repoUrl} />
            </label>
            <button class="primary" disabled={!repoUrl.trim()} onclick={updateRepoFromGit}>
              <Download size={17} />
              Update
            </button>
          </div>
          <div class="repo-action">
            <label>
              <span>ZIP File</span>
              <input bind:value={repoZipPath} placeholder="D:\Downloads\bettergi-scripts-list.zip" />
            </label>
            <button disabled={!repoZipPath.trim()} onclick={importZipRepo}>
              <FileArchive size={17} />
              Import
            </button>
          </div>
          <div class="repo-action">
            <label>
              <span>bettergi Link</span>
              <input bind:value={repoImportUri} placeholder="bettergi://script?import=..." />
            </label>
            <button disabled={!repoImportUri.trim()} onclick={importBettergiUri}>
              <Link2 size={17} />
              Import
            </button>
          </div>
          <div class="repo-action repo-action-compact">
            <div class="repo-action-text">
              <span>Subscriptions</span>
              <strong>{repoState?.subscribed_paths.length ?? 0} paths</strong>
            </div>
            <button onclick={updateSubscribedScripts}>
              <RefreshCw size={17} />
              Update
            </button>
          </div>
        </section>

        {#if scriptClipboardResult}
          <section class="script-import-banner" aria-live="polite">
            <div class="script-import-copy">
              <strong>Clipboard script import</strong>
              <span>{scriptClipboardResult.paths.slice(0, 3).join(", ")}</span>
              {#if scriptClipboardResult.paths.length > 3}
                <small>{`+${scriptClipboardResult.paths.length - 3} more path(s)`}</small>
              {/if}
            </div>
            <div class="script-import-actions">
              <button class="primary" onclick={importScriptClipboardUri}>
                <Link2 size={17} />
                Import Link
              </button>
              <button onclick={ignoreScriptClipboardUri}>Ignore</button>
            </div>
          </section>
        {/if}

        <section class="repo-layout">
          <section class="panel repo-summary">
            <div class="panel-heading">
              <h3>Script Repository</h3>
              <span>{repoStatus || "idle"}</span>
            </div>
            {#if repoState}
              <div class="repo-details">
                <article>
                  <span>Path</span>
                  <strong>{repoState.repo_path}</strong>
                </article>
                <article>
                  <span>Index</span>
                  <strong>{repoState.repo_json_path ?? "missing"}</strong>
                </article>
                <article>
                  <span>Subscriptions</span>
                  <strong>{repoState.subscribed_paths.length}</strong>
                </article>
                <article>
                  <span>Index Bytes</span>
                  <strong>{repoState.repo_json_bytes}</strong>
                </article>
              </div>
            {:else}
              <section class="notice">Load a repository to inspect bridge data.</section>
            {/if}
          </section>

          <section class="panel repo-subscriptions">
            <div class="panel-heading">
              <h3>Subscribed Paths</h3>
              <span>{repoState?.subscribed_paths.length ?? 0}</span>
            </div>
            <div class="repo-list">
              {#each repoState?.subscribed_paths ?? [] as path}
                <article>
                  <strong>{path}</strong>
                  <button onclick={() => markSelectedUpdated(path)}>Clear</button>
                </article>
              {/each}
            </div>
          </section>
        </section>

        <section class="panel script-settings-panel">
          <div class="panel-heading">
            <h3>Script Settings</h3>
            <span>{scriptSettingsStatus || "idle"}</span>
          </div>
          <div class="script-group-admin">
            <label>
              <span>New Group</span>
              <input bind:value={newGroupName} placeholder="Daily route" />
            </label>
            <button disabled={!newGroupName.trim()} onclick={createScriptGroup}>Create</button>
            <label>
              <span>Rename To</span>
              <input bind:value={renameGroupName} />
            </label>
            <button disabled={!selectedGroupName || !renameGroupName.trim()} onclick={renameScriptGroup}>
              Rename
            </button>
            <button disabled={!selectedGroupName} onclick={deleteScriptGroup}>Delete</button>
          </div>
          <div class="script-settings-toolbar">
            <label>
              <span>Group</span>
              <select bind:value={selectedGroupName} onchange={() => (renameGroupName = selectedGroupName)}>
                {#each scriptGroupsState as group}
                  <option value={group.name}>{group.name}</option>
                {/each}
              </select>
            </label>
            <label>
              <span>Project</span>
              <select bind:value={selectedProjectIndex}>
                {#each selectedScriptGroup()?.projects ?? [] as project}
                  <option value={project.index}>
                    {project.name || project.folder_name} · {project.project_type}
                  </option>
                {/each}
              </select>
            </label>
            <button onclick={loadScriptGroups}>
              <RefreshCw size={17} />
              Groups
            </button>
            <button
              class="primary"
              disabled={!selectedGroupName || !selectedScriptProject()}
              onclick={loadScriptSettings}
            >
              <Cog size={17} />
              Load
            </button>
            <button disabled={!scriptSettings} onclick={saveScriptSettings}>Save</button>
            <button disabled={!selectedGroupName || scriptRunBusy} onclick={runSelectedScriptGroup}>
              <Play size={17} />
              Group
            </button>
            <button disabled={!scriptRunBusy} title="Stop running script" onclick={stopScriptRun}>
              <Square size={17} />
              Stop
            </button>
          </div>
          {#if selectedScriptGroup()?.projects.length}
            <div class="script-project-list">
              {#each selectedScriptGroup()?.projects ?? [] as project}
                <button
                  type="button"
                  draggable="true"
                  class:selected={project.index === selectedProjectIndex}
                  class:dragging={project.index === draggedProjectIndex}
                  ondragstart={(event) => dragScriptProject(event, project.index)}
                  ondragover={(event) => event.preventDefault()}
                  ondrop={(event) => dropScriptProject(event, project.index)}
                  ondragend={() => (draggedProjectIndex = null)}
                  onclick={() => (selectedProjectIndex = project.index)}
                  title={project.name || project.folder_name}
                >
                  <span>{project.project_index}</span>
                  <strong>{project.name || project.folder_name}</strong>
                  <em>{project.project_type} · {project.schedule || "Manual"}</em>
                </button>
              {/each}
            </div>
          {/if}
          {#if selectedScriptProject()}
            {@const project = selectedScriptProject()}
            <div class="script-project-editor">
              <article>
                <span>Selected</span>
                <strong>{project?.name || project?.folder_name}</strong>
                <em>{project?.folder_name}</em>
              </article>
              <label>
                <span>Status</span>
                <select
                  value={project?.status ?? "Enabled"}
                  onchange={(event) => setSelectedProjectStatus(event.currentTarget.value)}
                >
                  <option value="Enabled">Enabled</option>
                  <option value="Disabled">Disabled</option>
                </select>
              </label>
              <label>
                <span>Schedule</span>
                <select
                  value={legacyScheduleOptions.some((option) => option.value === (project?.schedule ?? "")) ? (project?.schedule ?? "") : "__custom"}
                  onchange={(event) => {
                    if (event.currentTarget.value !== "__custom") setSelectedProjectSchedule(event.currentTarget.value);
                  }}
                >
                  {#each legacyScheduleOptions as option}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                  <option value="__custom">Custom</option>
                </select>
              </label>
              <label>
                <span>Custom Schedule</span>
                <input
                  value={project?.schedule ?? ""}
                  placeholder="0 0 * * *"
                  oninput={(event) => setSelectedProjectSchedule(event.currentTarget.value)}
                />
              </label>
              <label>
                <span>Run Count</span>
                <input
                  type="number"
                  min="1"
                  value={project?.run_num ?? 1}
                  oninput={(event) => setSelectedProjectRunNum(event.currentTarget.value)}
                />
              </label>
              <label class="script-settings-check">
                <input
                  type="checkbox"
                  checked={project?.allow_js_notification !== false}
                  onchange={(event) => setSelectedProjectNotification(event.currentTarget.checked)}
                />
                <span>JS notifications</span>
              </label>
              <button disabled={(project?.index ?? 0) <= 0} title="Move up" onclick={() => moveSelectedProject(-1)}>
                <ArrowUp size={16} />
                Up
              </button>
              <button
                disabled={(project?.index ?? 0) >= (selectedScriptGroup()?.projects.length ?? 1) - 1}
                title="Move down"
                onclick={() => moveSelectedProject(1)}
              >
                <ArrowDown size={16} />
                Down
              </button>
              <button onclick={saveSelectedProject}>Save Project</button>
              <button onclick={removeSelectedProject}>Remove</button>
              <button
                class="primary"
                disabled={scriptRunBusy}
                onclick={() => runSelectedScriptProject(false)}
              >
                <Play size={16} />
                Run
              </button>
              <button disabled={scriptRunBusy} title="Continue group from this project" onclick={runSelectedScriptGroupFromProject}>
                <CornerDownRight size={16} />
                Continue
              </button>
              <button
                disabled={scriptRunBusy || (project?.run_num ?? 1) <= 1}
                onclick={() => runSelectedScriptProject(true)}
              >
                <Play size={16} />
                Run Count
              </button>
            </div>
          {/if}
          <div class="script-run-panel">
            <div class="script-run-heading">
              <div>
                <span>Execution</span>
                <strong>{scriptRunStatus || "idle"}</strong>
              </div>
              {#if scriptRunResult}
                <em>{scriptRunResult.runtime} · {scriptRunResult.execution_mode}</em>
              {:else if scriptGroupRunResult}
                <em>{scriptGroupRunResult.group_name} · {scriptGroupRunResult.steps.length} steps</em>
              {/if}
            </div>
            {#if scriptGroupRunResult}
              <div class="script-run-grid">
                <article>
                  <span>Group</span>
                  <strong>{scriptGroupRunResult.group_name}</strong>
                  <em>{scriptGroupRunResult.requested_projects} projects</em>
                </article>
                <article>
                  <span>Attempted</span>
                  <strong>{scriptGroupRunResult.attempted_steps}</strong>
                  <em>{scriptGroupRunResult.skipped_steps} skipped</em>
                </article>
                <article>
                  <span>Completed</span>
                  <strong>{scriptGroupRunResult.completed_steps}</strong>
                  <em>{scriptGroupRunResult.planned_steps} planned · {scriptGroupRunResult.cancelled_steps} stopped</em>
                </article>
                <article>
                  <span>Failed</span>
                  <strong>{scriptGroupRunResult.failed_steps}</strong>
                  <em>{scriptGroupRunResult.failed_steps === 0 ? "none" : "inspect steps"}</em>
                </article>
              </div>
              <div class="script-run-calls">
                <span>Steps</span>
                {#if scriptGroupRunResult.steps.length > 0}
                  <div>
                    {#each scriptGroupRunResult.steps.slice(0, 12) as step}
                      {@const summary = scriptGroupStepExecutionSummary(step)}
                      <article class:failed={step.status === "Failed"}>
                        <strong>{step.project_order}. {step.name || step.folder_name}</strong>
                        <em>{step.project_type} · {step.status} · {step.run_iteration || 0}/{step.run_count}</em>
                        {#if summary}
                          <div class="execution-summary">
                            <div>
                              <strong>{summary.title}</strong>
                              <em>{summary.meta}</em>
                            </div>
                            <dl>
                              {#each summary.details as detail}
                                <div>
                                  <dt>{detail.label}</dt>
                                  <dd>{detail.value}</dd>
                                </div>
                              {/each}
                            </dl>
                          </div>
                        {/if}
                        <pre>{JSON.stringify(scriptGroupStepPayload(step), null, 2)}</pre>
                      </article>
                    {/each}
                  </div>
                {:else}
                  <p>No steps were produced.</p>
                {/if}
              </div>
            {:else if scriptRunResult}
              <div class="script-run-grid">
                <article>
                  <span>Project</span>
                  <strong>{scriptRunResult.project || scriptRunResult.folder_name}</strong>
                  <em>{scriptRunResult.main_script_path}</em>
                </article>
                <article>
                  <span>Result</span>
                  <pre>{scriptRunResultJson()}</pre>
                </article>
                <article>
                  <span>Console</span>
                  <pre>{scriptRunResult.console.join("\n") || "No console output."}</pre>
                </article>
                <article>
                  <span>Logs</span>
                  <pre>{scriptRunResult.logs.map((log) => `${log.level}: ${log.message}`).join("\n") || "No log records."}</pre>
                </article>
              </div>
              {@const taskSummary = taskInvocationSummary(scriptRunResult.task_invocations, scriptRunResult.task_execution)}
              {#if taskSummary}
                <div class="execution-summary">
                  <div>
                    <strong>{taskSummary.title}</strong>
                    <em>{taskSummary.meta}</em>
                  </div>
                  <dl>
                    {#each taskSummary.details as detail}
                      <div>
                        <dt>{detail.label}</dt>
                        <dd>{detail.value}</dd>
                      </div>
                    {/each}
                  </dl>
                </div>
              {/if}
              <div class="script-run-calls">
                <span>Host Calls</span>
                {#if scriptRunResult.host_calls.length > 0}
                  <div>
                    {#each scriptRunResult.host_calls.slice(0, 12) as call}
                      {@const summary = hostCallExecutionSummary(call)}
                      <article>
                        <strong>{scriptRunHostCallLabel(call)}</strong>
                        {#if summary}
                          <div class="execution-summary">
                            <div>
                              <strong>{summary.title}</strong>
                              <em>{summary.meta}</em>
                            </div>
                            <dl>
                              {#each summary.details as detail}
                                <div>
                                  <dt>{detail.label}</dt>
                                  <dd>{detail.value}</dd>
                                </div>
                              {/each}
                            </dl>
                          </div>
                        {/if}
                        <pre>{JSON.stringify(call.result, null, 2)}</pre>
                      </article>
                    {/each}
                  </div>
                {:else}
                  <p>No host calls recorded.</p>
                {/if}
              </div>
            {:else}
              <section class="notice">Select a JavaScript project or run a group to inspect execution output, plans, logs, and host calls.</section>
            {/if}
          </div>
          <div class="script-project-add">
            <div class="script-add-row">
              <label>
                <span>JavaScript Project</span>
                <select bind:value={selectedAvailableJsFolder}>
                  {#each availableJsProjects as project}
                    <option value={project.folder_name}>
                      {project.name} · {project.folder_name}
                    </option>
                  {/each}
                </select>
              </label>
              <button onclick={loadAvailableJsProjects}>
                <RefreshCw size={17} />
                JS
              </button>
              <button
                class="primary"
                disabled={!selectedGroupName || !selectedAvailableJsFolder}
                onclick={addSelectedJsProject}
              >
                Add
              </button>
            </div>
            <div class="script-add-row">
              <label>
                <span>KeyMouse Script</span>
                <select bind:value={selectedKeyMouseScript}>
                  {#each availableKeyMouseScripts as script}
                    <option value={script.name}>{script.relative_path}</option>
                  {/each}
                </select>
              </label>
              <button onclick={loadAvailableKeyMouseScripts}>
                <RefreshCw size={17} />
                KM
              </button>
              <button
                disabled={!selectedGroupName || !selectedKeyMouseScript}
                onclick={addSelectedKeyMouseScript}
              >
                Add
              </button>
            </div>
            <div class="script-add-row">
              <label>
                <span>Pathing Route</span>
                <select bind:value={selectedPathingScript}>
                  {#each availablePathingScripts as script}
                    <option value={script.relative_path}>{script.relative_path}</option>
                  {/each}
                </select>
              </label>
              <button onclick={loadAvailablePathingScripts}>
                <RefreshCw size={17} />
                Path
              </button>
              <button
                disabled={!selectedGroupName || !selectedPathingScript}
                onclick={addSelectedPathingScript}
              >
                Add
              </button>
            </div>
            {#if availablePathingTree?.children.length}
              <div class="pathing-tree-picker">
                {@render pathingTree(availablePathingTree.children)}
              </div>
            {/if}
            <div class="script-add-row">
              <label>
                <span>Shell Command</span>
                <input bind:value={shellCommand} placeholder="echo ok" />
              </label>
              <button disabled={!selectedGroupName || !shellCommand.trim()} onclick={addShellProject}>
                Add Shell
              </button>
            </div>
          </div>
          {#if scriptSettings}
            <div class="script-settings-meta">
              <article>
                <span>Manifest</span>
                <strong>{scriptSettings.manifest_name} {scriptSettings.manifest_version}</strong>
              </article>
              <article>
                <span>Project</span>
                <strong>{scriptSettings.project_path}</strong>
              </article>
              <article>
                <span>Settings UI</span>
                <strong>{scriptSettings.settings_ui_path ?? "missing"}</strong>
              </article>
              <article>
                <span>Cleanup</span>
                <strong>{scriptSettings.cleaned_invalid_values}</strong>
              </article>
            </div>
            {#if scriptSettings.items.length > 0}
              <div class="script-settings-form">
                {#each scriptSettings.items as item}
                  <article class:separator={item.type === "separator"}>
                    {#if item.type === "separator"}
                      <hr />
                      {#if settingLabel(item)}
                        <strong>{settingLabel(item)}</strong>
                      {/if}
                    {:else if item.type === "input-text"}
                      <label>
                        <span>{settingLabel(item)}</span>
                        <input
                          value={settingStringValue(item.name)}
                          oninput={(event) =>
                            setSettingString(item.name, event.currentTarget.value)}
                        />
                      </label>
                    {:else if item.type === "select"}
                      <label>
                        <span>{settingLabel(item)}</span>
                        <select
                          value={settingStringValue(item.name)}
                          onchange={(event) =>
                            setSettingString(item.name, event.currentTarget.value)}
                        >
                          {#each item.options ?? [] as option}
                            <option value={option}>{option}</option>
                          {/each}
                        </select>
                      </label>
                    {:else if item.type === "checkbox"}
                      <label class="script-settings-check">
                        <input
                          type="checkbox"
                          checked={settingBoolValue(item.name)}
                          onchange={(event) =>
                            setSettingBool(item.name, event.currentTarget.checked)}
                        />
                        <span>{settingLabel(item)}</span>
                      </label>
                    {:else if item.type === "multi-checkbox"}
                      <div class="script-settings-multi">
                        <span>{settingLabel(item)}</span>
                        <div>
                          {#each item.options ?? [] as option}
                            <label>
                              <input
                                type="checkbox"
                                checked={settingArrayValue(item.name).includes(option)}
                                onchange={() => toggleSettingArrayValue(item.name, option)}
                              />
                              <span>{option}</span>
                            </label>
                          {/each}
                        </div>
                      </div>
                    {:else if item.type === "cascade-select"}
                      {@const first = cascadeFirstValue(item)}
                      <div class="script-settings-cascade">
                        <span>{settingLabel(item)}</span>
                        <div>
                          <select
                            value={first}
                            onchange={(event) => setCascadeFirst(item, event.currentTarget.value)}
                          >
                            {#each Object.keys(item.cascadeOptions ?? {}) as option}
                              <option value={option}>{option}</option>
                            {/each}
                          </select>
                          <select
                            value={settingStringValue(item.name)}
                            onchange={(event) =>
                              setSettingString(item.name, event.currentTarget.value)}
                          >
                            {#each cascadeSecondOptions(item, first) as option}
                              <option value={option}>{option}</option>
                            {/each}
                          </select>
                        </div>
                      </div>
                    {:else}
                      <div class="script-settings-unknown">
                        <span>{settingLabel(item)}</span>
                        <strong>{item.type || "unknown"}</strong>
                      </div>
                    {/if}
                  </article>
                {/each}
              </div>
            {:else}
              <section class="notice">This JavaScript script has no settings schema.</section>
            {/if}
          {:else}
            <section class="notice">Load script groups, choose a JavaScript project, then load settings.</section>
          {/if}
        </section>

        <section class="panel repo-index-panel">
          <div class="panel-heading">
            <h3>Repository Index</h3>
            <span>{repoState?.index_nodes.length ?? 0}</span>
          </div>
          <div class="repo-index-tools">
            <input bind:value={repoSearch} placeholder="Search path, name, type" />
            <button class="primary" disabled={selectedImportPaths.length === 0} onclick={importSelectedPaths}>
              Import {selectedImportPaths.length}
            </button>
          </div>
          <div class="repo-index-list">
            {#each visibleRepoNodes() as node}
              <article class:updated={node.has_update}>
                <label style={`padding-left: ${Math.min(node.depth, 6) * 14}px`}>
                  <input
                    type="checkbox"
                    disabled={!node.importable}
                    checked={selectedImportPaths.includes(node.path)}
                    onchange={() => toggleImportPath(node.path)}
                  />
                  <span>{node.path}</span>
                </label>
                <span>{node.node_type}</span>
                <span>{node.last_updated ?? "-"}</span>
                <button disabled={!node.has_update} onclick={() => markSelectedUpdated(node.path)}>Clear</button>
              </article>
            {/each}
          </div>
        </section>

        <section class="panel repo-file-panel">
          <div class="panel-heading">
            <h3>Repository File</h3>
            <span>{repoFile?.kind ?? "none"}</span>
          </div>
          <div class="repo-file-tools">
            <input bind:value={repoFilePath} />
            <button onclick={loadRepoFile}>Open</button>
          </div>
          {#if repoFile?.kind === "ImageBase64"}
            <img class="repo-image" src={`data:image/${repoFile.extension.slice(1)};base64,${repoFile.content}`} alt={repoFile.rel_path} />
          {:else if repoFile}
            <pre class="repo-preview">{repoFile.content}</pre>
          {:else}
            <section class="notice">No file loaded.</section>
          {/if}
        </section>

        <section class="panel repo-json-panel">
          <div class="panel-heading">
            <h3>Repository Index</h3>
            <span>{repoJsonPreview.length}</span>
          </div>
          <pre class="repo-preview">{repoJsonPreview || "No index loaded."}</pre>
        </section>
      {:else if isNotificationView()}
        <section class="panel notification-panel">
          <div class="panel-heading">
            <h3>Notification Providers</h3>
            <span>{notificationSendStatus || "idle"}</span>
          </div>
          <div class="notification-service">
            <div class="notification-service-header">
              <div>
                <strong>Service Lifecycle</strong>
                <span>{notificationServiceStatus || "idle"}</span>
              </div>
              <button disabled={notificationServiceBusy} onclick={refreshNotificationServiceState}>
                <RefreshCw size={17} />
                Refresh
              </button>
            </div>
            {#if notificationServiceState}
              <div class="notification-summary service">
                <article>
                  <span>Initialized</span>
                  <strong>{notificationServiceState.initialized ? "yes" : "no"}</strong>
                </article>
                <article>
                  <span>Enabled Providers</span>
                  <strong>{notificationServiceState.provider_count}</strong>
                </article>
                <article>
                  <span>Refreshed</span>
                  <strong>{timestampLabel(notificationServiceState.refreshed_at_ms)}</strong>
                </article>
                <article>
                  <span>Config</span>
                  <strong>{notificationServiceState.config_path}</strong>
                </article>
              </div>
              <p>{notificationServiceState.enabled_providers.join(", ") || "No providers enabled."}</p>
            {:else}
              <section class="notice">Load the notification service state to inspect the Rust provider lifecycle snapshot.</section>
            {/if}
          </div>
          <div class="notification-composer">
            <label>
              <span>Provider</span>
              <select bind:value={notificationProvider}>
                <option value="">All enabled</option>
                <option value="Webhook">Webhook</option>
                <option value="WindowsUwp">Windows UWP</option>
                <option value="Feishu">Feishu</option>
                <option value="OneBot">OneBot</option>
                <option value="WorkWeixin">Work Weixin</option>
                <option value="WebSocket">WebSocket</option>
                <option value="Bark">Bark</option>
                <option value="Email">Email</option>
                <option value="DingDingWebhook">DingDing</option>
                <option value="Telegram">Telegram</option>
                <option value="Xxtui">Xxtui</option>
                <option value="DiscordWebhook">Discord Webhook</option>
                <option value="ServerChan">ServerChan</option>
                <option value="Meow">Meow</option>
              </select>
            </label>
            <label>
              <span>Test Message</span>
              <input bind:value={notificationMessage} />
            </label>
            <button
              class="primary"
              disabled={notificationSendBusy || !notificationMessage.trim()}
              onclick={sendTestNotification}
            >
              <Bell size={17} />
              Send
            </button>
          </div>
          {#if notificationResult}
            <div class="notification-summary">
              <article>
                <span>Attempted</span>
                <strong>{notificationResult.attempted ? "yes" : "no"}</strong>
              </article>
              <article>
                <span>Skipped</span>
                <strong>{notificationResult.skipped_reason ?? "-"}</strong>
              </article>
              <article>
                <span>Providers</span>
                <strong>{notificationResult.deliveries.length}</strong>
              </article>
              <article>
                <span>Requests</span>
                <strong>{notificationResult.deliveries.reduce((sum, item) => sum + item.requests, 0)}</strong>
              </article>
            </div>
            {#if notificationResult.deliveries.length > 0}
              <div class="notification-deliveries">
                {#each notificationResult.deliveries as delivery}
                  <article>
                    <div>
                      <strong>{delivery.provider_name}</strong>
                      <span>{delivery.provider}</span>
                    </div>
                    <span class:sent={delivery.status === "Sent"} class:failed={delivery.status === "Failed"} class:unsupported={delivery.status === "Unsupported"}>
                      {delivery.status}
                    </span>
                    <span>{delivery.requests} requests</span>
                    <em>{delivery.message ?? "-"}</em>
                  </article>
                {/each}
              </div>
            {:else}
              <section class="notice">No providers were selected by the current notification configuration.</section>
            {/if}
          {:else}
            <section class="notice">Send a test notification with the current configuration. HTTP, WebSocket, Email, and Windows UWP providers dispatch through the Rust notification boundary; screenshots are attached when enabled and BitBlt capture is available.</section>
          {/if}
        </section>
      {:else if isSettingsView()}
        <section class="panel update-panel">
          <div class="panel-heading">
            <h3>Version Update</h3>
            <span>{updateStatus || "idle"}</span>
          </div>
          <div class="update-toolbar">
            <label>
              <span>Channel</span>
              <select bind:value={updateChannel}>
                <option value="stable">Stable</option>
                <option value="alpha">Alpha</option>
              </select>
            </label>
            <button class="primary" disabled={updateCheckBusy} onclick={checkForUpdate}>
              <RefreshCw size={17} />
              Check
            </button>
            <button disabled={updateActionBusy} onclick={openUpdateDownloadPage}>
              <Download size={17} />
              Download Page
            </button>
          </div>

          {#if backgroundUpdateState}
            <div class="update-background-strip">
              <span>{backgroundUpdateState.running ? "Background check running" : "Background check idle"}</span>
              {#if backgroundUpdateState.last_result}
                <strong>{updateDecisionLabel(backgroundUpdateState.last_result.decision.action)}</strong>
                <em>{backgroundUpdateState.last_result.latest_version ?? "-"}</em>
              {:else if backgroundUpdateState.last_error}
                <strong>failed</strong>
                <em>{backgroundUpdateState.last_error}</em>
              {:else}
                <strong>pending</strong>
              {/if}
            </div>
          {/if}

          {#if updateResult}
            <div class="update-summary">
              <article>
                <span>Current</span>
                <strong>{updateResult.app_version}</strong>
              </article>
              <article>
                <span>Latest</span>
                <strong>{updateResult.latest_version ?? "-"}</strong>
              </article>
              <article>
                <span>Decision</span>
                <strong>{updateDecisionLabel(updateResult.decision.action)}</strong>
              </article>
              <article>
                <span>Ignored</span>
                <strong>{updateResult.ignored_version || "-"}</strong>
              </article>
              <article>
                <span>Updater</span>
                <strong>{updateResult.updater_exists ? "available" : "missing"}</strong>
                <em>{updateResult.updater_path}</em>
              </article>
              <article>
                <span>Request</span>
                <strong>{`${updateResult.channel} / ${updateResult.trigger}`}</strong>
                <em>{updateRequestDisplay(updateResult.request)}</em>
              </article>
            </div>

            <div class="update-actions">
              <label class="update-exit-toggle">
                <input type="checkbox" bind:checked={updaterExitAfterLaunch} />
                <span>Exit after launch</span>
              </label>
              {#each updateResult.updater_options as option}
                <button
                  disabled={updateActionBusy || !updateResult.updater_exists}
                  title={option.warning ?? option.args.join(" ")}
                  onclick={() => launchUpdater(option)}
                >
                  {#if option.source === "Default" || option.source === "Dfs" || option.source === "DfsAlpha"}
                    <PackageOpen size={17} />
                  {/if}
                  {option.display_name}
                </button>
              {/each}
              <button disabled={updateActionBusy || !updateResult.latest_version} onclick={ignoreDetectedUpdate}>
                Ignore Version
              </button>
            </div>

            {#if updateResult.release_notes}
              <section class="update-notes">
                <div>
                  <strong>{updateResult.release_notes.name ?? "Release Notes"}</strong>
                  <span>{updateResult.release_notes.html_url ?? ""}</span>
                </div>
                <pre>{updateResult.release_notes.body ?? "No release notes returned."}</pre>
              </section>
            {:else}
              <section class="notice">Stable checks use the OSS notice gray-release gate. Alpha checks use the MirrorChyan latest endpoint. Updater sources are planned by Rust core and executed through the local updater sidecar.</section>
            {/if}
          {:else}
            <section class="notice">Check a channel to inspect the Rust update decision, remote request, updater availability, and available manual actions.</section>
          {/if}
        </section>
        <section class="panel redeem-panel">
          <div class="panel-heading">
            <h3>Redeem Code Feed</h3>
            <span>{redeemFeedStatus || "idle"}</span>
          </div>
          <div class="redeem-toolbar">
            <button class="primary" disabled={redeemFeedBusy} onclick={checkRedeemCodeFeed}>
              <RefreshCw size={17} />
              Check Feed
            </button>
            <button disabled={redeemFeedBusy} onclick={loadRedeemCodeFeedItems}>
              Load Items
            </button>
            <button disabled={redeemFeedBusy} onclick={loadRedeemCodeLiveCodes}>
              Live Codes
            </button>
            <button disabled={redeemFeedBusy || !redeemFeedResult?.decision.remote_version} onclick={markRedeemCodeFeedRead}>
              Mark Read
            </button>
            <button disabled={redeemFeedBusy} onclick={() => planRedeemCodes("manual")}>
              Plan Manual
            </button>
            <button disabled={redeemFeedBusy || !redeemFeedItems} onclick={() => planRedeemCodes("feed")}>
              Plan Feed
            </button>
            <button disabled={redeemFeedBusy || !redeemLiveResult} onclick={() => planRedeemCodes("live")}>
              Plan Live
            </button>
            <button disabled={redeemFeedBusy || !redeemManualText.trim()} onclick={() => executeRedeemCodes("manual")}>
              <Play size={17} />
              Execute Manual
            </button>
            <button disabled={redeemFeedBusy || !redeemFeedItems} onclick={() => executeRedeemCodes("feed")}>
              Execute Feed
            </button>
            <button disabled={redeemFeedBusy || !redeemLiveResult} onclick={() => executeRedeemCodes("live")}>
              Execute Live
            </button>
            <button disabled={redeemFeedBusy} onclick={checkRedeemClipboardText}>
              Check Clipboard
            </button>
            <button disabled={redeemFeedBusy || !redeemManualText.trim()} onclick={ignoreRedeemClipboardText}>
              Ignore Text
            </button>
            <label class="redeem-toggle">
              <input
                type="checkbox"
                disabled={redeemFeedBusy || !redeemClipboardState}
                checked={redeemClipboardState?.clipboard_listener_enabled ?? false}
                onchange={(event) => setRedeemClipboardEnabled(event.currentTarget.checked)}
              />
              <span>Clipboard</span>
            </label>
            {#if redeemFeedResult}
              <span>{redeemFeedResult.request_url}</span>
            {:else if redeemFeedItems}
              <span>{redeemFeedItems.request_url}</span>
            {/if}
          </div>
          <label class="redeem-manual">
            <span>Manual text</span>
            <textarea
              rows="3"
              bind:value={redeemManualText}
              placeholder="Paste text containing 12-character redeem codes"
            ></textarea>
          </label>
          {#if redeemClipboardState || redeemClipboardResult}
            <div class="redeem-summary compact">
              <article>
                <span>Clipboard Listener</span>
                <strong>{redeemClipboardState?.clipboard_listener_enabled ? "enabled" : "disabled"}</strong>
              </article>
              <article>
                <span>Ignored Hashes</span>
                <strong>{redeemClipboardState?.ignored_hash_count ?? 0}</strong>
              </article>
              <article>
                <span>Detected Codes</span>
                <strong>{redeemClipboardResult?.extracted_codes.length ?? 0}</strong>
              </article>
              <article>
                <span>Text Hash</span>
                <strong>{redeemClipboardResult?.hash ?? "-"}</strong>
              </article>
            </div>
          {/if}
          {#if redeemFeedResult}
            <div class="redeem-summary">
              <article>
                <span>Local</span>
                <strong>{redeemFeedResult.local_version || "-"}</strong>
              </article>
              <article>
                <span>Remote</span>
                <strong>{redeemFeedResult.decision.remote_version ?? "-"}</strong>
              </article>
              <article>
                <span>Status</span>
                <strong>{redeemFeedResult.decision.has_update ? "new codes" : "up to date"}</strong>
              </article>
              <article>
                <span>Raw</span>
                <strong>{redeemFeedResult.remote_text?.trim() || "-"}</strong>
              </article>
            </div>
          {/if}
          {#if redeemFeedItems}
            <div class="redeem-summary compact">
              <article>
                <span>Items</span>
                <strong>{redeemFeedItems.items.length}</strong>
              </article>
              <article>
                <span>Bytes</span>
                <strong>{redeemFeedItems.raw_bytes}</strong>
              </article>
              <article>
                <span>Codes</span>
                <strong>{redeemFeedItems.items.reduce((sum, item) => sum + item.codes.length, 0)}</strong>
              </article>
              <article>
                <span>Source</span>
                <strong>{redeemFeedItems.request_url}</strong>
              </article>
            </div>
            <div class="redeem-feed-list">
              {#each redeemFeedItems.items as item}
                <article>
                  <div>
                    <strong>{item.title || "Untitled"}</strong>
                    <span>{item.time || "-"} {item.valid ? `Valid: ${item.valid}` : ""}</span>
                  </div>
                  {#if item.tag}
                    <em>{item.tag}</em>
                  {/if}
                  {#if item.content}
                    <p>{item.content}</p>
                  {/if}
                  <code>{item.codes.join("\n") || "No codes"}</code>
                </article>
              {/each}
            </div>
          {/if}
          {#if redeemLiveResult}
            <div class="redeem-summary compact">
              <article>
                <span>Live Title</span>
                <strong>{redeemLiveResult.data.title || "-"}</strong>
              </article>
              <article>
                <span>Act Id</span>
                <strong>{redeemLiveResult.data.act_id}</strong>
              </article>
              <article>
                <span>Code Version</span>
                <strong>{redeemLiveResult.data.code_version}</strong>
              </article>
              <article>
                <span>Codes</span>
                <strong>{redeemLiveResult.data.codes.length}</strong>
              </article>
            </div>
            <div class="redeem-feed-list">
              <article>
                <div>
                  <strong>Live Preview Codes</strong>
                  <span>{redeemLiveResult.refresh_code_url}</span>
                </div>
                <em>live</em>
                <p>{redeemLiveResult.data.codes.map((code) => code.items).filter(Boolean).join("\n") || "No reward text returned."}</p>
                <code>{redeemLiveResult.data.codes.map((code) => code.code).join("\n") || "No codes"}</code>
              </article>
            </div>
          {:else if !redeemFeedResult && !redeemFeedItems}
            <section class="notice">Check the migrated redeem-code feed update timestamp, load the remote feed items, and mark the current feed version as read in the Rust config model.</section>
          {/if}
          {#if redeemPlanResult}
            <div class="redeem-summary compact">
              <article>
                <span>Plan Task</span>
                <strong>{redeemPlanResult.plan.task_key}</strong>
              </article>
              <article>
                <span>Codes</span>
                <strong>{redeemPlanResult.extracted_codes.length}</strong>
              </article>
              <article>
                <span>Steps</span>
                <strong>{redeemPlanResult.plan.steps.length}</strong>
              </article>
              <article>
                <span>Executor</span>
                <strong>{redeemPlanResult.plan.executor_ready ? "ready" : "pending"}</strong>
              </article>
            </div>
            <div class="redeem-feed-list">
              <article>
                <div>
                  <strong>Auto Redeem Plan</strong>
                  <span>{redeemPlanResult.plan.port_state}</span>
                </div>
                <em>plan</em>
                <p>{redeemPlanResult.plan.notes}</p>
                <code>{redeemPlanResult.plan.steps
                  .slice(0, 18)
                  .map((step) => `${step.phase}/${step.condition}${step.code ? `/${step.code}` : ""}: ${step.label}`)
                  .join("\n")}</code>
              </article>
            </div>
          {/if}
          {#if redeemExecutionResult}
            {@const successfulCodes = redeemReportList(redeemExecutionResult, "successful_codes")}
            {@const failedCodes = redeemReportList(redeemExecutionResult, "failed_codes")}
            <div class="redeem-summary compact">
              <article>
                <span>Execution</span>
                <strong>{redeemExecutionResult.report.completed ? "completed" : "incomplete"}</strong>
              </article>
              <article>
                <span>Executed</span>
                <strong>{redeemExecutionResult.report.executed_steps.length}</strong>
              </article>
              <article>
                <span>Skipped</span>
                <strong>{redeemExecutionResult.report.skipped_steps.length}</strong>
              </article>
              <article>
                <span>Success / Failed</span>
                <strong>{successfulCodes.length} / {failedCodes.length}</strong>
              </article>
            </div>
            <div class="redeem-feed-list">
              <article>
                <div>
                  <strong>Auto Redeem Execution</strong>
                  <span>{redeemExecutionResult.report.task_key}</span>
                </div>
                <em>{redeemExecutionResult.report.completed ? "done" : "incomplete"}</em>
                <p>{failedCodes.length ? `Failed: ${failedCodes.join(", ")}` : "No failed codes reported."}</p>
                <code>{successfulCodes.concat(failedCodes).join("\n") || "No code result reported."}</code>
              </article>
            </div>
          {/if}
        </section>
        <section class="panel overlay-panel">
          <div class="panel-heading">
            <h3>Desktop Overlay</h3>
            <span>{overlayStatus || "idle"}</span>
          </div>
          <div class="overlay-toolbar">
            <button disabled={overlayBusy} onclick={loadDesktopOverlay}>
              <RefreshCw size={17} />
              Load Overlay
            </button>
            <button disabled={overlayBusy || !overlayState} onclick={() => updateDesktopOverlay({ resetMetricsLayout: true })}>
              Reset Metrics Layout
            </button>
          </div>
          {#if overlayState}
            <div class="overlay-switches">
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.mask_enabled}
                  onchange={(event) => updateDesktopOverlay({ maskEnabled: event.currentTarget.checked })}
                />
                <span>Mask</span>
              </label>
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.show_log_box}
                  onchange={(event) => updateDesktopOverlay({ showLogBox: event.currentTarget.checked })}
                />
                <span>Log box</span>
              </label>
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.show_status}
                  onchange={(event) => updateDesktopOverlay({ showStatus: event.currentTarget.checked })}
                />
                <span>Status</span>
              </label>
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.display_recognition_results_on_mask}
                  onchange={(event) => updateDesktopOverlay({ displayRecognitionResultsOnMask: event.currentTarget.checked })}
                />
                <span>Recognition</span>
              </label>
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.show_overlay_metrics}
                  onchange={(event) => updateDesktopOverlay({ showOverlayMetrics: event.currentTarget.checked })}
                />
                <span>Metrics</span>
              </label>
              <label>
                <input
                  type="checkbox"
                  disabled={overlayBusy}
                  checked={overlayState.overlay_layout_edit_enabled}
                  onchange={(event) => updateDesktopOverlay({ overlayLayoutEditEnabled: event.currentTarget.checked })}
                />
                <span>Layout edit</span>
              </label>
            </div>
            <div class="overlay-summary">
              <article>
                <span>Mask</span>
                <strong>{overlayState.mask_enabled ? "enabled" : "disabled"}</strong>
              </article>
              <article>
                <span>Log Box</span>
                <strong>{overlayState.show_log_box ? "visible" : "hidden"}</strong>
              </article>
              <article>
                <span>Status</span>
                <strong>{overlayState.show_status ? "visible" : "hidden"}</strong>
              </article>
              <article>
                <span>Metrics</span>
                <strong>{overlayState.show_overlay_metrics ? "visible" : "hidden"}</strong>
              </article>
              <article>
                <span>Opacity</span>
                <strong>{Math.round(overlayState.text_opacity * 100)}%</strong>
              </article>
              <article>
                <span>Layout Edit</span>
                <strong>{overlayState.overlay_layout_edit_enabled ? "enabled" : "disabled"}</strong>
              </article>
            </div>
            <div class="overlay-layouts">
              <article>
                <span>Log</span>
                <strong>{overlayLayoutLabel(overlayState.log_box_layout)}</strong>
              </article>
              <article>
                <span>Status</span>
                <strong>{overlayLayoutLabel(overlayState.status_layout)}</strong>
              </article>
              <article>
                <span>Metrics</span>
                <strong>{overlayLayoutLabel(overlayState.metrics_layout)}</strong>
                {#if overlayState.migrated_legacy_metrics_layout}
                  <em>legacy layout normalized</em>
                {/if}
              </article>
            </div>
            <div class="overlay-metrics">
              {#each overlayState.metrics as metric}
                <label class:enabled={overlayState.enabled_metric_keys.includes(metric.key)} title={metric.tooltip}>
                  <input
                    type="checkbox"
                    disabled={overlayBusy}
                    checked={overlayState.enabled_metric_keys.includes(metric.key)}
                    onchange={(event) => updateDesktopOverlay({ metricKey: metric.key, metricEnabled: event.currentTarget.checked })}
                  />
                  <span>
                    <strong>{metric.display_name}</strong>
                    <small>{metric.key}</small>
                  </span>
                </label>
              {/each}
            </div>
          {:else}
            <section class="notice">Load the migrated overlay configuration summary. The transparent native overlay window and hardware sensors still require separate desktop integration.</section>
          {/if}
        </section>
        <section class="panel shell-panel">
          <div class="panel-heading">
            <h3>Desktop Shell</h3>
            <span>{shellStatus || "idle"}</span>
          </div>
          <div class="shell-toolbar">
            <button disabled={shellBusy} onclick={loadDesktopShell}>
              <Cog size={17} />
              Load Shell
            </button>
            <button disabled={shellBusy} onclick={() => runDesktopShellAction("desktop_shell_show_main_window")}>
              Show
            </button>
            <button disabled={shellBusy} onclick={() => runDesktopShellAction("desktop_shell_hide_main_window")}>
              Hide
            </button>
            <button disabled={shellBusy} onclick={() => runDesktopShellAction("desktop_shell_toggle_main_window")}>
              Toggle
            </button>
            <label class="shell-toggle">
              <input
                type="checkbox"
                disabled={shellBusy || !shellState}
                checked={shellState?.exit_to_tray ?? false}
                onchange={(event) => setExitToTray(event.currentTarget.checked)}
              />
              <span>Exit to tray on window close</span>
            </label>
          </div>
          {#if shellState}
            <div class="shell-summary">
              <article>
                <span>Tray</span>
                <strong>{shellState.tray_enabled ? "enabled" : "disabled"}</strong>
              </article>
              <article>
                <span>Close Behavior</span>
                <strong>{shellState.exit_to_tray ? "hide to tray" : "exit app"}</strong>
              </article>
              <article>
                <span>Config</span>
                <strong>{shellState.config_path}</strong>
              </article>
              <article>
                <span>Log</span>
                <strong>{shellState.log_path}</strong>
              </article>
            </div>
          {:else}
            <section class="notice">Load the desktop shell state to inspect the tray lifecycle configuration and change the close-button behavior.</section>
          {/if}
        </section>
        <section class="panel log-panel">
          <div class="panel-heading">
            <h3>Desktop Log</h3>
            <span>{logStatus || "idle"}</span>
          </div>
          <div class="log-toolbar">
            <button onclick={loadDesktopLog}>
              <ScrollText size={17} />
              Load Log
            </button>
            {#if logState}
              <span>{logState.path}</span>
            {/if}
          </div>
          {#if logState}
            <div class="log-summary">
              <article>
                <span>Exists</span>
                <strong>{logState.exists ? "yes" : "no"}</strong>
              </article>
              <article>
                <span>Bytes</span>
                <strong>{logState.bytes}</strong>
              </article>
              <article>
                <span>Lines</span>
                <strong>{logState.tail.length}</strong>
              </article>
            </div>
            <pre class="log-tail">{logState.tail.join("\n") || "No log lines."}</pre>
          {:else}
            <section class="notice">The Rust desktop shell writes command and tray lifecycle events to daily log files under the log folder.</section>
          {/if}
        </section>
      {:else}
      <section class="summary-grid">
        <article>
          <span>Capture</span>
          <strong>{state.config.capture_mode}</strong>
        </article>
        <article>
          <span>Trigger Loop</span>
          <strong>{state.config.trigger_interval} ms</strong>
        </article>
        <article>
          <span>Auto Pick</span>
          <strong>{state.config.auto_pick_enabled ? "enabled" : "disabled"}</strong>
        </article>
        <article>
          <span>Main Hotkey</span>
          <strong>{state.config.bgi_enabled_hotkey}</strong>
        </article>
        <article>
          <span>ONNX Models</span>
          <strong>{state.native_backend.registered_onnx_models}</strong>
        </article>
        <article>
          <span>Avatar Model</span>
          <strong>{state.native_backend.avatar_side_model?.exists ? state.native_backend.avatar_side_model.source : "missing"}</strong>
        </article>
        <article>
          <span>Task Runtime</span>
          <strong>{state.task_runtime.dispatcher.state}</strong>
        </article>
        <article>
          <span>Task Catalog</span>
          <strong>{state.task_runtime.config_bound_catalog_entries}/{state.task_runtime.catalog_entries}</strong>
        </article>
        <article>
          <span>Script Hosts</span>
          <strong>{state.script_runtime.summary.host_binding_count}</strong>
        </article>
        <article>
          <span>Post Messages</span>
          <strong>{state.native_backend.post_message_events_in_demo_sequence}</strong>
        </article>
        <article>
          <span>Config Sections</span>
          <strong>{state.config.strongly_typed_config_sections}/{state.config.modeled_config_sections}</strong>
        </article>
      </section>

      <section class="panel-grid">
        <section class="panel">
          <div class="panel-heading">
            <h3>Realtime Triggers</h3>
            <span>{state.triggers.length}</span>
          </div>
          <div class="table">
            {#each state.triggers as trigger}
              <div class="row">
                <div>
                  <strong>{trigger.display_name}</strong>
                  <span>{trigger.key}</span>
                </div>
                <span>{trigger.priority}</span>
                <span class="badge">{triggerStateLabel(trigger)}</span>
              </div>
            {/each}
          </div>
        </section>

        <section class="panel">
          <div class="panel-heading">
            <h3>Migration Coverage</h3>
            <span>{state.capabilities.length}</span>
          </div>
          <div class="capability-list">
            {#each state.capabilities as capability}
              <article>
                <div>
                  <strong>{capability.area}</strong>
                  <span>{capability.rust_module}</span>
                </div>
                <em>{capability.state}</em>
              </article>
            {/each}
          </div>
        </section>
      </section>

      <section class="panel capture-panel">
        <div class="panel-heading">
          <h3>Capture Backends</h3>
          <span>{state.native_backend.capture_modes.length}</span>
        </div>
        <div class="capture-grid">
          {#each state.native_backend.capture_modes as mode}
            <article>
              <div>
                <strong>{mode.description}</strong>
                <span>legacy={mode.legacy_value}</span>
              </div>
              <span class:ready={mode.implemented}>{mode.implemented ? "ready" : "pending"}</span>
            </article>
          {/each}
        </div>
      </section>

      <section class="panel vision-panel">
        <div class="panel-heading">
          <h3>Vision Types</h3>
          <span>{state.native_backend.recognition_types.length}</span>
        </div>
        <div class="vision-grid">
          {#each state.native_backend.recognition_types as item}
            <article>
              <strong>{item.recognition_type}</strong>
              <span class:ready={item.implemented}>{item.implemented ? "ready" : "pending"}</span>
            </article>
          {/each}
        </div>
      </section>

      <section class="panel task-panel">
        <div class="panel-heading">
          <h3>Task Runtime</h3>
          <span>{state.task_runtime.selection_reason}</span>
        </div>
        <div class="task-grid">
          <article>
            <span>Enabled Triggers</span>
            <strong>{state.task_runtime.enabled_triggers}</strong>
          </article>
          <article>
            <span>Selected Triggers</span>
            <strong>{state.task_runtime.selected_triggers.length}</strong>
          </article>
          <article>
            <span>Registered Timers</span>
            <strong>{state.task_runtime.dispatcher.registered_realtime_triggers.length}</strong>
          </article>
          <article>
            <span>Independent Tasks</span>
            <strong>{state.task_runtime.independent_tasks.length}</strong>
          </article>
          <article>
            <span>Runner</span>
            <strong>{state.task_runtime.runner.state}</strong>
          </article>
        </div>
        <div class="script-settings-toolbar">
          <label>
            <span>Shell</span>
            <input bind:value={independentShellCommand} placeholder="echo BetterGI Rust" />
          </label>
          <label>
            <span>Timeout</span>
            <input type="number" min="0" bind:value={independentShellTimeout} />
          </label>
          <button disabled={independentShellBusy || !independentShellCommand.trim()} onclick={runIndependentShellTask}>
            <Play size={17} />
            Run Shell
          </button>
          <button disabled={!independentShellBusy} title="Stop running shell task" onclick={stopIndependentShellTask}>
            <Square size={17} />
            Stop
          </button>
        </div>
        {#if independentShellResult}
          {@const shellSummary = shellExecutionSummary(independentShellResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{shellSummary.title}</strong>
              <em>{shellSummary.meta}</em>
            </div>
            <dl>
              {#each shellSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        <div class="script-settings-toolbar">
          <label>
            <span>Live Tasks</span>
            <input value="QuickBuy / QuickSereniteaPot / AutoOpenChest / AutoCook / AutoMusic" readonly />
          </label>
          <button disabled={independentLiveTaskBusy} onclick={() => runIndependentLiveTask("task_execute_quick_buy")}>
            <PackageOpen size={17} />
            QuickBuy
          </button>
          <button
            disabled={independentLiveTaskBusy}
            onclick={() => runIndependentLiveTask("task_execute_quick_serenitea_pot")}
          >
            <Play size={17} />
            Quick Pot
          </button>
          <button
            disabled={independentLiveTaskBusy}
            onclick={() => runIndependentLiveTask("task_execute_auto_open_chest")}
          >
            <PackageOpen size={17} />
            AutoChest
          </button>
          <button disabled={independentLiveTaskBusy} onclick={() => runIndependentLiveTask("task_execute_auto_cook")}>
            <Play size={17} />
            AutoCook
          </button>
          <button
            disabled={independentLiveTaskBusy}
            onclick={() => runIndependentLiveTask("task_execute_auto_music_game_performance")}
          >
            <Keyboard size={17} />
            Music Play
          </button>
          <button
            disabled={independentLiveTaskBusy}
            onclick={() => runIndependentLiveTask("task_execute_auto_music_game_album")}
          >
            <Play size={17} />
            Music Album
          </button>
          <button disabled={!independentLiveTaskBusy} title="Stop running live task" onclick={stopIndependentLiveTask}>
            <Square size={17} />
            Stop
          </button>
        </div>
        {#if independentLiveTaskResult}
          {@const liveTaskSummary = independentLiveTaskSummary(independentLiveTaskResult)}
          <div class="execution-summary">
            <div>
              <strong>{liveTaskSummary.title}</strong>
              <em>{liveTaskSummary.meta}</em>
            </div>
            <dl>
              {#each liveTaskSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        <div class="script-settings-toolbar">
          <label>
            <span>AutoPathing</span>
            <select bind:value={selectedPathingScript}>
              {#each availablePathingScripts as script}
                <option value={script.relative_path}>{script.relative_path}</option>
              {/each}
            </select>
          </label>
          <button disabled={independentPathingBusy} onclick={loadAvailablePathingScripts}>
            <RefreshCw size={17} />
            Routes
          </button>
          <button disabled={independentPathingBusy || !selectedPathingScript} onclick={planIndependentAutoPathingTask}>
            <Route size={17} />
            Plan
          </button>
          <button
            disabled={independentPathingBusy || !selectedPathingScript}
            onclick={runIndependentAutoPathingActionBoundary}
          >
            <Play size={17} />
            Boundary
          </button>
          <button disabled={!independentPathingBusy} title="Stop auto-pathing boundary" onclick={stopIndependentLiveTask}>
            <Square size={17} />
            Stop
          </button>
        </div>
        {#if independentPathingResult}
          {@const pathingSummary = autoPathingExecutionSummary(independentPathingResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{pathingSummary.title}</strong>
              <em>{pathingSummary.meta}</em>
            </div>
            <dl>
              {#each pathingSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        {#if independentPathingBoundaryResult}
          {@const boundarySummary = autoPathingBoundarySummary(independentPathingBoundaryResult)}
          <div class="execution-summary">
            <div>
              <strong>{boundarySummary.title}</strong>
              <em>{boundarySummary.meta}</em>
            </div>
            <dl>
              {#each boundarySummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        <div class="script-settings-toolbar">
          <label>
            <span>AutoFight</span>
            <input bind:value={independentFightStrategy} placeholder="strategy name or blank for auto" />
          </label>
          <label>
            <span>Team</span>
            <input bind:value={independentFightTeamNames} placeholder="optional names, comma separated" />
          </label>
          <button disabled={independentFightBusy} onclick={planIndependentAutoFightTask}>
            <Bot size={17} />
            Plan Fight
          </button>
          <button disabled={independentFightPlaybackBusy} onclick={() => executeIndependentAutoFightTeamPlayback(false)}>
            <Play size={17} />
            Plan Team
          </button>
          <button disabled={independentFightPlaybackBusy} onclick={() => executeIndependentAutoFightTeamPlayback(true)}>
            <Gamepad2 size={17} />
            Send Team
          </button>
          <button disabled={independentFightPlaybackBusy} onclick={() => executeIndependentAutoFightTeamPlayback(false, true)}>
            <Search size={17} />
            Plan Live Team
          </button>
          <button disabled={independentFightPlaybackBusy} onclick={() => executeIndependentAutoFightTeamPlayback(true, true)}>
            <Crosshair size={17} />
            Send Live Team
          </button>
          <button disabled={independentFightAvatarBusy} onclick={detectIndependentAutoFightActiveAvatar}>
            <Search size={17} />
            Active Slot
          </button>
          <button disabled={independentFightProbeBusy} onclick={() => probeIndependentAutoFightFinish(false)}>
            <ScanLine size={17} />
            Capture Probe
          </button>
          <button disabled={independentFightProbeBusy} onclick={() => probeIndependentAutoFightFinish(true)}>
            <Crosshair size={17} />
            Live Probe
          </button>
        </div>
        {#if independentFightResult}
          {@const fightSummary = autoFightExecutionSummary(independentFightResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{fightSummary.title}</strong>
              <em>{fightSummary.meta}</em>
            </div>
            <dl>
              {#each fightSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        {#if independentFightPlaybackResult}
          {@const playbackSummary = autoFightTeamPlaybackSummary(independentFightPlaybackResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{playbackSummary.title}</strong>
              <em>{playbackSummary.meta}</em>
            </div>
            <dl>
              {#each playbackSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        {#if independentFightAvatarResult}
          {@const avatarSummary = activeAvatarSummary(independentFightAvatarResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{avatarSummary.title}</strong>
              <em>{avatarSummary.meta}</em>
            </div>
            <dl>
              {#each avatarSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
        {#if independentFightProbeResult}
          {@const probeSummary = autoFightFinishProbeSummary(independentFightProbeResult.result)}
          <div class="execution-summary">
            <div>
              <strong>{probeSummary.title}</strong>
              <em>{probeSummary.meta}</em>
            </div>
            <dl>
              {#each probeSummary.details as detail}
                <div>
                  <dt>{detail.label}</dt>
                  <dd>{detail.value}</dd>
                </div>
              {/each}
            </dl>
          </div>
        {/if}
      </section>

      <section class="panel script-panel">
        <div class="panel-heading">
          <h3>Script Runtime</h3>
          <span>{state.script_runtime.summary.state}</span>
        </div>
        <div class="script-grid">
          <article>
            <span>Engines</span>
            <strong>{state.script_runtime.summary.engines.length}</strong>
          </article>
          <article>
            <span>Host Members</span>
            <strong>{state.script_runtime.summary.host_member_count}</strong>
          </article>
          <article>
            <span>Permissions</span>
            <strong>{state.script_runtime.summary.permissions.length}</strong>
          </article>
          <article>
            <span>Project Types</span>
            <strong>{state.script_runtime.summary.supported_project_types.length}</strong>
          </article>
          <article>
            <span>Module Search</span>
            <strong>{state.script_runtime.summary.project_loader.default_search_paths.length}</strong>
          </article>
          <article>
            <span>Setting Types</span>
            <strong>{state.script_runtime.summary.settings.supported_types.length}</strong>
          </article>
          <article>
            <span>Macro Events</span>
            <strong>{state.script_runtime.sample_macro.event_count}</strong>
          </article>
        </div>
        <div class="policy-grid">
          <article>
            <span>File Extensions</span>
            <strong>{state.script_runtime.security.file_allowed_extensions.length}</strong>
          </article>
          <article>
            <span>Max Write</span>
            <strong>{Math.round(state.script_runtime.security.file_max_write_bytes / 1024 / 1024)} MB</strong>
          </article>
          <article>
            <span>HTTP Rules</span>
            <strong>{state.script_runtime.security.http_uses_manifest_wildcards ? "wildcard" : "exact"}</strong>
          </article>
          <article>
            <span>Notify Limit</span>
            <strong>{state.script_runtime.security.notification_max_per_window}/min</strong>
          </article>
          <article>
            <span>Package Alias</span>
            <strong>{state.script_runtime.summary.project_loader.package_alias_rewrite}</strong>
          </article>
          <article>
            <span>Settings Cleanup</span>
            <strong>{state.script_runtime.summary.settings.cleans_multi_checkbox_options ? "multi-select" : "off"}</strong>
          </article>
        </div>
        <div class="host-table">
          {#each state.script_runtime.hosts.slice(0, 8) as host}
            <article>
              <div>
                <strong>{host.name}</strong>
                <span>{host.legacy_type}</span>
              </div>
              <span>{host.members.length}</span>
              <span class="badge">{host.port_state}</span>
            </article>
          {/each}
        </div>
      </section>
      {/if}
    {/if}
  </section>
</main>
