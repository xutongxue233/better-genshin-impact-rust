use anyhow::{Context, Result};
use bgi_capture::capture_mode_infos;
use bgi_core::{
    config_path, initial_triggers, migration_capabilities, notification_dispatch_plan,
    notification_events, notification_provider_plans, read_config, read_pathing_task,
    redeem_code_feed_update_decision, update_decision, update_request_plan, write_config,
    AppConfig, AssetResolver, GenshinAction, MirrorChyanLatestResponse, NotificationPayload,
    ScreenSize, UpdateChannel, UpdateOption, UpdateTrigger,
};
use bgi_hotkey::Hotkey;
use bgi_input::{
    post_message_events_for_action, release_pressed_keys_sequence, InputSequence, KeyActionType,
    MouseButton, PostMessageMode,
};
use bgi_script::{
    checkout_git_repo_path, clear_repo_bridge_update, execute_file_repo_import,
    execute_git_repo_update, execute_repo_import_with_git, execute_zip_repo_import,
    git_update_plan, host_bindings, mark_repo_bridge_path_updated, parse_import_uri,
    read_repo_bridge_file, read_repo_bridge_file_with_git, read_repo_bridge_repo_json,
    read_script_group_file, read_subscription_file, repo_bridge_index_nodes,
    repo_bridge_subscribed_paths_json, script_engines, script_group_file_path,
    script_host_security_summary, script_import_plan, script_repo_bridge_paths,
    script_repo_channels, script_repo_layout, script_repo_update_plan, script_runtime_summary,
    script_settings_summary, zip_import_plan, GameCaptureArea, GlobalInputHost, HttpHost,
    KeyMouseScript, KeyMouseScriptHost, MacroPlaybackContext, PreparedScriptExecution,
    RecordingHttpClient, RecordingNotificationSink, ScriptGroupProject, ScriptHostCall,
    ScriptHostRuntime, ScriptHostRuntimeConfig, ScriptHostTarget, ScriptHttpPolicy,
    ScriptModuleLoader, ScriptNotificationHost, ScriptNotificationPolicy,
    ScriptProjectLoaderSummary, ScriptProjectType, SystemGitRunner,
};
use bgi_task::{
    independent_tasks, runtime_triggers, select_triggers_for_tick, task_catalog,
    task_parameter_models, DispatcherRuntime, RunnerRuntime,
};
use bgi_vision::{
    recognition_type_infos, registered_onnx_models, BgrImage, BvImage, BvLocatorOperation, BvPage,
    Rect, Size,
};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "bgi")]
#[command(about = "Rust migration CLI for BetterGI")]
struct Cli {
    #[arg(long, default_value = ".", global = true)]
    project_root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    About {
        #[arg(long)]
        json: bool,
    },
    Capabilities {
        #[arg(long)]
        json: bool,
    },
    Triggers {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        all_enabled: bool,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Assets {
        #[command(subcommand)]
        command: AssetsCommand,
    },
    Pathing {
        #[command(subcommand)]
        command: PathingCommand,
    },
    Capture {
        #[command(subcommand)]
        command: CaptureCommand,
    },
    Vision {
        #[command(subcommand)]
        command: VisionCommand,
    },
    Script {
        #[command(subcommand)]
        command: ScriptCommand,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    Notification {
        #[command(subcommand)]
        command: NotificationCommand,
    },
    Update {
        #[command(subcommand)]
        command: UpdateCommand,
    },
    Hotkey {
        hotkey: String,
    },
    InputDemo,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    WriteDefault {
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum AssetsCommand {
    Features,
    Resolve {
        feature: String,
        asset_name: String,
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
    },
    List {
        feature: String,
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
    },
}

#[derive(Debug, Subcommand)]
enum PathingCommand {
    Validate {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum CaptureCommand {
    Modes {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum VisionCommand {
    Types {
        #[arg(long)]
        json: bool,
    },
    Models {
        #[arg(long)]
        json: bool,
    },
    Bv {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ScriptCommand {
    Runtime {
        #[arg(long)]
        json: bool,
    },
    Hosts {
        #[arg(long)]
        json: bool,
    },
    Engines {
        #[arg(long)]
        json: bool,
    },
    Policy {
        #[arg(long)]
        json: bool,
    },
    Loader {
        #[arg(long)]
        json: bool,
    },
    Settings {
        #[arg(long)]
        json: bool,
    },
    Macro {
        file: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        events: bool,
    },
    KeyMouseHost {
        root: PathBuf,
        file: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        events: bool,
    },
    GlobalInput {
        #[arg(long)]
        json: bool,
    },
    HostRuntime {
        #[arg(long)]
        json: bool,
    },
    ModuleLoad {
        root: PathBuf,
        specifier: String,
        #[arg(long)]
        referrer: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    PrepareJs {
        scripts_root: PathBuf,
        folder: String,
        #[arg(long)]
        json: bool,
    },
    ExecuteJs {
        scripts_root: PathBuf,
        folder: String,
        #[arg(long)]
        settings_json: Option<String>,
        #[arg(long)]
        json: bool,
    },
    ExecuteGroup {
        app_root: PathBuf,
        group: String,
        #[arg(long)]
        json: bool,
    },
    RepoChannels {
        #[arg(long)]
        json: bool,
    },
    RepoLayout {
        #[arg(long)]
        json: bool,
    },
    RepoUpdatePlan {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        manual: bool,
        paths: Vec<String>,
    },
    RepoImportUri {
        uri: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        clipboard: bool,
    },
    RepoImportPlan {
        #[arg(long)]
        json: bool,
        paths: Vec<String>,
    },
    RepoImportExec {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        git_repo: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
        paths: Vec<String>,
    },
    RepoZipPlan {
        zip: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoZipExec {
        zip: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoGitPlan {
        repo_url: String,
        #[arg(long)]
        json: bool,
    },
    RepoGitUpdate {
        repo_url: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
    },
    RepoGitCheckout {
        repo: PathBuf,
        source: String,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
        #[arg(long)]
        root: bool,
    },
    RepoBridgePaths {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoBridgeJson {
        repo: PathBuf,
    },
    RepoBridgeIndex {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
    },
    RepoBridgeSubscribed {
        repo: PathBuf,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoBridgeFile {
        repo: PathBuf,
        rel_path: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        git_repo: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
    },
    RepoBridgeMarkUpdated {
        repo: PathBuf,
        path: String,
        #[arg(long)]
        json: bool,
    },
    RepoBridgeClearUpdate {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TaskCommand {
    Runtime {
        #[arg(long)]
        json: bool,
    },
    Independent {
        #[arg(long)]
        json: bool,
    },
    Catalog {
        #[arg(long)]
        json: bool,
    },
    Params {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum NotificationCommand {
    Events {
        #[arg(long)]
        json: bool,
    },
    Providers {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Dispatch {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "notify.test")]
        event: String,
        #[arg(long, default_value = "success")]
        result: String,
        #[arg(long, default_value = "这是一条测试通知信息")]
        message: String,
    },
}

#[derive(Debug, Subcommand)]
enum UpdateCommand {
    Plan {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "stable")]
        channel: String,
    },
    Decision {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "auto")]
        trigger: String,
        #[arg(long, default_value = "stable")]
        channel: String,
        #[arg(long, default_value = env!("CARGO_PKG_VERSION"))]
        current: String,
        #[arg(long)]
        latest: Option<String>,
        #[arg(long)]
        ignored: Option<String>,
    },
    Mirror {
        response_json: String,
        #[arg(long)]
        json: bool,
    },
    RedeemFeed {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "20251013")]
        local: String,
        #[arg(long)]
        remote: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::About { json } => about(json),
        Commands::Capabilities { json } => capabilities(json),
        Commands::Triggers { json, all_enabled } => triggers(json, all_enabled),
        Commands::Config { command } => config(command, &cli.project_root),
        Commands::Assets { command } => assets(command, &cli.project_root),
        Commands::Pathing { command } => pathing(command),
        Commands::Capture { command } => capture(command),
        Commands::Vision { command } => vision(command),
        Commands::Script { command } => script(command),
        Commands::Task { command } => task(command),
        Commands::Notification { command } => notification(command, &cli.project_root),
        Commands::Update { command } => update(command),
        Commands::Hotkey { hotkey } => hotkey_command(&hotkey),
        Commands::InputDemo => input_demo(),
    }
}

fn about(json: bool) -> Result<()> {
    if json {
        let payload = serde_json::json!({
            "name": "better-genshin-impact-rust",
            "cli": env!("CARGO_PKG_VERSION"),
            "workspace": "Cargo workspace",
            "status": "migration scaffold"
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("better-genshin-impact-rust {}", env!("CARGO_PKG_VERSION"));
        println!(
            "Rust workspace with a ported core model, trigger registry, assets resolver, and pathing loader."
        );
    }
    Ok(())
}

fn capabilities(json: bool) -> Result<()> {
    let capabilities = migration_capabilities();
    if json {
        println!("{}", serde_json::to_string_pretty(&capabilities)?);
    } else {
        for capability in capabilities {
            println!(
                "{:<22} {:<22} {}",
                capability.area,
                format!("{:?}", capability.state),
                capability.notes
            );
        }
    }
    Ok(())
}

fn triggers(json: bool, all_enabled: bool) -> Result<()> {
    let triggers = initial_triggers();
    if json {
        println!("{}", serde_json::to_string_pretty(&triggers)?);
    } else {
        for trigger in triggers {
            let enabled = all_enabled || trigger.default_enabled;
            println!(
                "{:<18} priority={:<5} enabled={:<5} ui={:?} state={:?}",
                trigger.key,
                trigger.priority,
                enabled,
                trigger.supported_game_ui_category,
                trigger.port_state
            );
        }
    }
    Ok(())
}

fn config(command: ConfigCommand, project_root: &Path) -> Result<()> {
    match command {
        ConfigCommand::Show { path, json } => {
            let path = path.unwrap_or_else(|| config_path(project_root));
            let config = if path.exists() {
                read_config(&path).with_context(|| format!("failed to read config at {path:?}"))?
            } else {
                AppConfig::default()
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("config: {:?}", path);
                println!("capture_mode: {:?}", config.capture_mode);
                println!("trigger_interval: {}ms", config.trigger_interval);
                println!("auto_pick_enabled: {}", config.auto_pick_config.enabled);
                println!("auto_skip_enabled: {}", config.auto_skip_config.enabled);
                println!("bgi_hotkey: {}", config.hot_key_config.bgi_enabled_hotkey);
                let coverage = config.coverage();
                println!(
                    "config_sections: {} strong={} compatibility={} unknown_top_level={}",
                    coverage.modeled_config_sections,
                    coverage.strongly_typed_sections,
                    coverage.compatibility_sections,
                    coverage.preserved_unknown_top_level_fields
                );
            }
        }
        ConfigCommand::WriteDefault { path } => {
            write_config(&path, &AppConfig::default())
                .with_context(|| format!("failed to write config at {path:?}"))?;
            println!("wrote {:?}", path);
        }
    }

    Ok(())
}

fn assets(command: AssetsCommand, project_root: &Path) -> Result<()> {
    let resolver = AssetResolver::new(project_root.to_path_buf());

    match command {
        AssetsCommand::Features => {
            for feature in resolver.known_feature_asset_dirs()? {
                println!("{feature}");
            }
        }
        AssetsCommand::Resolve {
            feature,
            asset_name,
            width,
            height,
        } => {
            let path = resolver.resolve_feature_asset(
                &feature,
                &asset_name,
                ScreenSize { width, height },
            )?;
            println!("{}", path.display());
        }
        AssetsCommand::List {
            feature,
            width,
            height,
        } => {
            for path in resolver.list_feature_assets(&feature, ScreenSize { width, height })? {
                println!("{}", path.display());
            }
        }
    }

    Ok(())
}

fn pathing(command: PathingCommand) -> Result<()> {
    match command {
        PathingCommand::Validate { file, json } => {
            let task = read_pathing_task(&file)
                .with_context(|| format!("failed to read route {file:?}"))?;
            let summary = task.summary();
            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                println!("route: {}", summary.name);
                println!("type: {} ({})", summary.task_type, summary.type_description);
                println!("map: {}", summary.map_name);
                println!("waypoints: {}", summary.waypoint_count);
                println!("actions: {}", summary.actions.join(", "));
                println!(
                    "realtime_triggers: {}",
                    summary.realtime_triggers.join(", ")
                );
            }
        }
    }

    Ok(())
}

fn capture(command: CaptureCommand) -> Result<()> {
    match command {
        CaptureCommand::Modes { json } => {
            let modes = capture_mode_infos();
            if json {
                println!("{}", serde_json::to_string_pretty(&modes)?);
            } else {
                for mode in modes {
                    println!(
                        "{:<28} legacy_value={} implemented={} {}",
                        mode.mode, mode.legacy_value, mode.implemented, mode.notes
                    );
                }
            }
        }
    }

    Ok(())
}

fn vision(command: VisionCommand) -> Result<()> {
    match command {
        VisionCommand::Types { json } => {
            let types = recognition_type_infos();
            if json {
                println!("{}", serde_json::to_string_pretty(&types)?);
            } else {
                for ty in types {
                    println!(
                        "{:<20} implemented={} {}",
                        format!("{:?}", ty.recognition_type),
                        ty.implemented,
                        ty.notes
                    );
                }
            }
        }
        VisionCommand::Models { json } => {
            let models = registered_onnx_models();
            if json {
                println!("{}", serde_json::to_string_pretty(&models)?);
            } else {
                for model in models {
                    println!(
                        "{:<24} legacy_name={:<18} {}",
                        model.rust_name, model.legacy_registered_name, model.model_relative_path
                    );
                }
            }
        }
        VisionCommand::Bv { json } => {
            let page = BvPage {
                capture_size: Size::new(2560, 1440),
                ..BvPage::default()
            };
            let image = BvImage::new("AutoPick:F.png")?;
            let roi = Rect::new(1600, 780, 320, 160)?;
            let locator = page
                .locator_for_image(&image, Some(roi), 0.86)?
                .with_timeout(1_500)?
                .with_retry_interval(250)?;
            let payload = serde_json::json!({
                "image": image,
                "screenshot": page.screenshot(),
                "click_1080p": page.click_1080p(960.0, 540.0),
                "locator_wait": locator.plan(BvLocatorOperation::WaitFor, None),
                "ocr": page.ocr(Some(roi)),
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("image: {}", payload["image"]["template_assert"]);
                println!(
                    "locator_retry_count: {}",
                    payload["locator_wait"]["retry_count"]
                );
                println!(
                    "click_screen: {},{}",
                    payload["click_1080p"]["Click1080p"]["screen_x"],
                    payload["click_1080p"]["Click1080p"]["screen_y"]
                );
            }
        }
    }

    Ok(())
}

fn script(command: ScriptCommand) -> Result<()> {
    match command {
        ScriptCommand::Runtime { json } => {
            let summary = script_runtime_summary();
            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                println!("state: {:?}", summary.state);
                println!("engines: {}", summary.engines.len());
                println!(
                    "host_bindings: {} (objects={}, types={}, members={})",
                    summary.host_binding_count,
                    summary.host_object_count,
                    summary.host_type_count,
                    summary.host_member_count
                );
                println!("permissions: {}", summary.permissions.len());
            }
        }
        ScriptCommand::Hosts { json } => {
            let hosts = host_bindings();
            if json {
                println!("{}", serde_json::to_string_pretty(&hosts)?);
            } else {
                for host in hosts {
                    println!(
                        "{:<18} {:<16} members={:<3} state={:?}",
                        host.name,
                        format!("{:?}", host.kind),
                        host.members.len(),
                        host.port_state
                    );
                }
            }
        }
        ScriptCommand::Engines { json } => {
            let engines = script_engines();
            if json {
                println!("{}", serde_json::to_string_pretty(&engines)?);
            } else {
                for engine in engines {
                    println!(
                        "{:<18} {:<16} {}",
                        format!("{:?}", engine.kind),
                        format!("{:?}", engine.port_state),
                        engine.notes
                    );
                }
            }
        }
        ScriptCommand::Policy { json } => {
            let policy = script_host_security_summary();
            if json {
                println!("{}", serde_json::to_string_pretty(&policy)?);
            } else {
                println!(
                    "file_extensions: {}",
                    policy.file_allowed_extensions.join(", ")
                );
                println!("max_write_bytes: {}", policy.file_max_write_bytes);
                println!(
                    "notifications: max_chars={}, max_per_{}ms={}",
                    policy.notification_max_chars,
                    policy.notification_window_ms,
                    policy.notification_max_per_window
                );
                println!(
                    "forbidden_notification_patterns: {}",
                    policy.forbidden_notification_patterns.join(", ")
                );
            }
        }
        ScriptCommand::Loader { json } => {
            let loader = ScriptProjectLoaderSummary::default();
            if json {
                println!("{}", serde_json::to_string_pretty(&loader)?);
            } else {
                println!("search_paths: {}", loader.default_search_paths.join(", "));
                println!("package_alias: {}", loader.package_alias_rewrite);
                println!("module_detection: {}", loader.module_detection.join(", "));
                println!(
                    "resource_rewrites: {}",
                    loader.resource_import_rewrites.join(", ")
                );
            }
        }
        ScriptCommand::Settings { json } => {
            let settings = script_settings_summary();
            if json {
                println!("{}", serde_json::to_string_pretty(&settings)?);
            } else {
                println!("supported_types: {}", settings.supported_types.len());
                println!("defaulted_types: {}", settings.defaulted_types.len());
                println!(
                    "cleans_multi_checkbox_options: {}",
                    settings.cleans_multi_checkbox_options
                );
                println!(
                    "preserves_unknown_fields: {}",
                    settings.preserves_unknown_fields
                );
            }
        }
        ScriptCommand::Macro { file, json, events } => {
            let script = KeyMouseScript::read(&file)
                .with_context(|| format!("failed to read key/mouse macro {file:?}"))?;
            if events {
                let context = script
                    .playback_context_from_info()
                    .with_context(|| format!("failed to build playback context for {file:?}"))?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&script.to_input_events(context)?)?
                );
            } else {
                let summary = script.summary();
                if json {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else {
                    println!("macro: {}", file.display());
                    println!("events: {}", summary.event_count);
                    println!("duration: {}ms", summary.duration_ms);
                    println!(
                        "keys={} absolute_mouse={} relative_mouse={} wheel={}",
                        summary.key_events,
                        summary.absolute_mouse_events,
                        summary.relative_mouse_events,
                        summary.wheel_events
                    );
                    println!("info: {}", summary.has_info);
                    println!("camera_orientation: {}", summary.uses_camera_orientation);
                }
            }
        }
        ScriptCommand::KeyMouseHost {
            root,
            file,
            json,
            events,
        } => {
            let host = KeyMouseScriptHost::new(&root, MacroPlaybackContext::default());
            let plan = host
                .run_file(&file)
                .with_context(|| format!("failed to plan key/mouse host file {file:?}"))?;
            if events {
                println!("{}", serde_json::to_string_pretty(&plan.input_events)?);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("root: {}", root.display());
                println!("file: {file}");
                if let Some(path) = &plan.normalized_path {
                    println!("normalized: {}", path.display());
                }
                println!("events: {}", plan.summary.event_count);
                println!("duration: {}ms", plan.summary.duration_ms);
                println!("input_events: {}", plan.input_events.len());
            }
        }
        ScriptCommand::GlobalInput { json } => {
            let mut host = GlobalInputHost::new(
                GameCaptureArea {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                1.0,
            )?;
            host.set_game_metrics(1920, 1080, 1.0)?;
            let payload = serde_json::json!({
                "metrics": host.game_metrics(),
                "key_press": host.key_press("VK_F")?.events(),
                "left_button_click": host.left_button_click().events(),
                "move_mouse_to": host.move_mouse_to(960, 540)?.events(),
                "input_text": host.input_text("BetterGI").events(),
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("metrics: 1920x1080 dpi=1");
                println!(
                    "key_press_events: {}",
                    payload["key_press"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "left_button_click_events: {}",
                    payload["left_button_click"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "move_mouse_to_events: {}",
                    payload["move_mouse_to"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "input_text_events: {}",
                    payload["input_text"].as_array().map_or(0, Vec::len)
                );
            }
        }
        ScriptCommand::HostRuntime { json } => {
            let demo_root = std::env::temp_dir().join("bgi-cli-host-runtime-demo");
            let _ = fs::remove_dir_all(&demo_root);
            fs::create_dir_all(demo_root.join("assets"))?;
            fs::create_dir_all(demo_root.join("strategy"))?;
            BgrImage::new(Size::new(2, 2), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])?
                .write_png(demo_root.join("assets").join("avatar.png"))?;

            let mut config = ScriptHostRuntimeConfig::new(&demo_root, demo_root.join("strategy"));
            config.http_policy = ScriptHttpPolicy::new(true, ["https://example.com/*".to_string()]);
            let mut runtime = ScriptHostRuntime::new(config)?;
            let mat_payload = serde_json::json!({
                "width": 2,
                "height": 2,
                "pixelFormat": "BGR24",
                "pixels": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            });
            let calls = [
                ScriptHostCall::new(
                    ScriptHostTarget::Global,
                    "KeyPress",
                    vec![serde_json::json!("VK_F")],
                ),
                ScriptHostCall::new(ScriptHostTarget::Global, "GetVersion", Vec::new()),
                ScriptHostCall::new(ScriptHostTarget::Global, "CaptureGameRegion", Vec::new()),
                ScriptHostCall::new(ScriptHostTarget::Global, "GetAvatars", Vec::new()),
                ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "ReadImageMatWithResizeSync",
                    vec![
                        serde_json::json!("assets/avatar.png"),
                        serde_json::json!(128),
                        serde_json::json!(128),
                        serde_json::json!(1),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::File,
                    "WriteImageSync",
                    vec![serde_json::json!("output/avatar"), mat_payload],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::CustomHostFunctions,
                    "NewVarOfArr",
                    vec![
                        serde_json::json!("OpenCvSharp.Point2f"),
                        serde_json::json!(2),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::PostMessage,
                    "KeyPress",
                    vec![serde_json::json!("VK_F")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Log,
                    "Info",
                    vec![serde_json::json!("host runtime ready")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Notification,
                    "Send",
                    vec![serde_json::json!("host runtime ready")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Http,
                    "Request",
                    vec![
                        serde_json::json!("POST"),
                        serde_json::json!("https://example.com/api"),
                        serde_json::json!("{\"ok\":true}"),
                        serde_json::json!(
                            "{\"Content-Type\":\"application/json\",\"X-Test\":\"1\"}"
                        ),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Dispatcher,
                    "AddTrigger",
                    vec![serde_json::json!({
                        "name": "AutoPick",
                        "interval": 50,
                        "config": { "enabled": true }
                    })],
                ),
                ScriptHostCall::new(ScriptHostTarget::Genshin, "Uid", Vec::new()),
                ScriptHostCall::new(
                    ScriptHostTarget::Genshin,
                    "Tp",
                    vec![
                        serde_json::json!("100.5"),
                        serde_json::json!(200.25),
                        serde_json::json!(true),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Genshin,
                    "SwitchParty",
                    vec![serde_json::json!("default")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::Genshin,
                    "SetTime",
                    vec![
                        serde_json::json!(8),
                        serde_json::json!("30"),
                        serde_json::json!(true),
                    ],
                ),
                ScriptHostCall::new(ScriptHostTarget::Genshin, "ReturnMainUi", Vec::new()),
                ScriptHostCall::new(
                    ScriptHostTarget::PathingScript,
                    "Run",
                    vec![serde_json::json!(
                        r#"{
                          "info": {
                            "name": "host runtime route",
                            "type": "collect",
                            "map_name": "Teyvat"
                          },
                          "positions": [
                            { "x": 100.0, "y": 200.0, "type": "path", "move_mode": "dash" }
                          ]
                        }"#
                    )],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::HtmlMask,
                    "Show",
                    vec![
                        serde_json::json!("overlay.html"),
                        serde_json::json!("demo-mask"),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::HtmlMask,
                    "Send",
                    vec![
                        serde_json::json!("demo-mask"),
                        serde_json::json!("/status"),
                        serde_json::json!("{\"ready\":true}"),
                    ],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::HtmlMask,
                    "FlushPendingMessages",
                    vec![serde_json::json!("demo-mask")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::HtmlMask,
                    "ToggleClickThrough",
                    vec![serde_json::json!("demo-mask")],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::KeyMouseHook,
                    "OnKeyDown",
                    vec![serde_json::json!("key-down"), serde_json::json!(true)],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::KeyMouseHook,
                    "OnMouseMove",
                    vec![serde_json::json!("mouse-move"), serde_json::json!(50)],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::KeyMouseHook,
                    "DispatchEvent",
                    vec![serde_json::json!({
                        "type": "keyDown",
                        "keyData": "Control, F",
                        "keyCode": "F"
                    })],
                ),
                ScriptHostCall::new(
                    ScriptHostTarget::ServerTime,
                    "GetServerTimeZoneOffset",
                    Vec::new(),
                ),
            ];

            let mut results = Vec::new();
            for call in calls {
                let target = call.target;
                let method = call.method.clone();
                let result = runtime.call(call)?;
                results.push(serde_json::json!({
                    "target": target,
                    "method": method,
                    "result": result,
                }));
            }

            let http_host = HttpHost::new(ScriptHttpPolicy::new(
                true,
                ["https://example.com/*".to_string()],
            ));
            let mut http_client = RecordingHttpClient::ok_json("{\"ok\":true}");
            let http_response = http_host.execute_request(
                "GET",
                "https://example.com/status",
                None,
                Some("{\"Accept\":\"application/json\"}"),
                &mut http_client,
            )?;

            let mut notification_host =
                ScriptNotificationHost::new(ScriptNotificationPolicy::new(true, true));
            let mut notification_sink = RecordingNotificationSink::default();
            let notification_delivery =
                notification_host.send_to("host runtime ready", 1, &mut notification_sink)?;

            let payload = serde_json::json!({
                "results": results,
                "http_response": http_response,
                "http_requests": http_client.requests(),
                "metrics": runtime.game_metrics(),
                "log_records": runtime.log_records(),
                "notification_records": runtime.notification_records(),
                "notification_delivery": notification_delivery,
                "notification_deliveries": notification_sink.deliveries(),
                "dispatcher_commands": runtime.dispatcher_commands(),
                "dispatcher_task_invocations": runtime.dispatcher_task_invocation_plans()?,
                "genshin_commands": runtime.genshin_commands(),
                "genshin_task_invocations": runtime.genshin_task_invocation_plans()?,
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!(
                    "calls: {}",
                    payload["results"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "logs: {}",
                    payload["log_records"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "notifications: {}",
                    payload["notification_records"]
                        .as_array()
                        .map_or(0, Vec::len)
                );
                println!(
                    "genshin_commands: {}",
                    payload["genshin_commands"].as_array().map_or(0, Vec::len)
                );
            }
        }
        ScriptCommand::ModuleLoad {
            root,
            specifier,
            referrer,
            json,
        } => {
            let mut loader = ScriptModuleLoader::new(
                &root,
                vec![PathBuf::from("."), PathBuf::from("./packages")],
            )?;
            let first = loader
                .load_js_module(&specifier, referrer.as_deref())
                .with_context(|| {
                    format!(
                        "failed to load module {specifier:?} from root {}",
                        root.display()
                    )
                })?;
            let second = loader.load_js_module(&specifier, referrer.as_deref())?;
            let payload = serde_json::json!({
                "resolution": first.resolution,
                "code_bytes": first.code.len(),
                "original_code_bytes": first.original_code.len(),
                "import_rewrites": first.import_rewrites,
                "cache_hit_on_first_load": first.cache_hit,
                "cache_hit_on_second_load": second.cache_hit,
                "cache_len": loader.cache_len(),
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("root: {}", root.display());
                println!("specifier: {specifier}");
                println!("resolved: {}", first.resolution.resolved_path.display());
                println!("kind: {:?}", first.resolution.kind);
                println!("code_bytes: {}", first.code.len());
                println!("import_rewrites: {}", first.import_rewrites.len());
                println!("cache_hit_on_second_load: {}", second.cache_hit);
            }
        }
        ScriptCommand::PrepareJs {
            scripts_root,
            folder,
            json,
        } => {
            let project = ScriptGroupProject {
                name: folder.clone(),
                folder_name: folder.clone(),
                project_type: ScriptProjectType::Javascript,
                ..ScriptGroupProject::default()
            };
            let manifest =
                bgi_script::Manifest::read_from(scripts_root.join(&folder).join("manifest.json"))
                    .with_context(|| {
                    format!(
                        "failed to read manifest for script project {}",
                        scripts_root.join(&folder).display()
                    )
                })?;
            let step = bgi_script::ScriptExecutionStep::from_group_project(
                &project,
                Some(&manifest),
                &scripts_root,
            )?;
            let prepared = PreparedScriptExecution::prepare_javascript(&step, &scripts_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&prepared)?);
            } else {
                println!("project: {}", prepared.step.folder_name);
                println!("mode: {:?}", prepared.execution_mode);
                println!(
                    "main: {}",
                    prepared.main_module.resolution.resolved_path.display()
                );
                println!("code_bytes: {}", prepared.main_module.code.len());
                println!("imports: {}", prepared.main_module.import_rewrites.len());
                println!(
                    "host_root: {}",
                    prepared.host_runtime_config.script_root.display()
                );
            }
        }
        ScriptCommand::ExecuteJs {
            scripts_root,
            folder,
            settings_json,
            json,
        } => {
            let settings = settings_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .context("failed to parse --settings-json as JSON")?;
            let outcome =
                bgi_script_engine::execute_javascript_project(scripts_root, folder, settings)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!("project: {}", outcome.folder_name);
                println!("runtime: {:?}", outcome.runtime);
                println!("mode: {:?}", outcome.execution_mode);
                println!("main: {}", outcome.main_script_path.display());
                println!("result: {}", outcome.result_display);
                println!("console_lines: {}", outcome.console.len());
                println!("log_records: {}", outcome.logs.len());
                println!("host_calls: {}", outcome.host_calls.len());
            }
        }
        ScriptCommand::ExecuteGroup {
            app_root,
            group,
            json,
        } => {
            let group_path =
                script_group_file_path(app_root.join("User").join("ScriptGroup"), &group);
            let group = read_script_group_file(&group_path)
                .with_context(|| format!("failed to read script group {}", group_path.display()))?;
            let outcome = bgi_script_engine::execute_script_group(&app_root, &group);
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!("group: {}", outcome.group_name);
                println!("projects: {}", outcome.requested_projects);
                println!("attempted_steps: {}", outcome.attempted_steps);
                println!("completed_steps: {}", outcome.completed_steps);
                println!("planned_steps: {}", outcome.planned_steps);
                println!("failed_steps: {}", outcome.failed_steps);
                println!("skipped_steps: {}", outcome.skipped_steps);
                for step in outcome.steps {
                    println!(
                        "{:?} {:<10} {:<10} {}/{} {}",
                        step.status,
                        format!("{:?}", step.project_type),
                        step.folder_name,
                        step.run_iteration,
                        step.run_count,
                        step.error.unwrap_or(step.name)
                    );
                }
            }
        }
        ScriptCommand::RepoChannels { json } => {
            let channels = script_repo_channels();
            if json {
                println!("{}", serde_json::to_string_pretty(&channels)?);
            } else {
                for channel in channels {
                    println!("{:<8} {}", channel.name, channel.url);
                }
            }
        }
        ScriptCommand::RepoLayout { json } => {
            let config = AppConfig::default();
            let layout = script_repo_layout(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&layout)?);
            } else {
                println!("repos: {}", layout.repos_path.display());
                println!("center: {}", layout.center_repo_path.display());
                println!("subscriptions: {}", layout.subscription_file_path.display());
            }
        }
        ScriptCommand::RepoUpdatePlan {
            json,
            manual,
            paths,
        } => {
            let config = AppConfig::default();
            let plan = script_repo_update_plan(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                manual,
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("enabled: {}", plan.enabled);
                println!("reason: {}", plan.reason.unwrap_or("-"));
                println!("repo: {}", plan.repo_url.as_deref().unwrap_or("-"));
                println!("subscriptions: {}", plan.subscribed_paths.len());
            }
        }
        ScriptCommand::RepoImportUri {
            uri,
            json,
            clipboard,
        } => {
            let plan = parse_import_uri(&uri, clipboard).map_err(anyhow::Error::msg)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else if let Some(plan) = plan {
                println!("paths: {}", plan.paths.join(","));
                println!("clear_clipboard: {}", plan.clear_clipboard_after_import);
            } else {
                println!("not_import_uri");
            }
        }
        ScriptCommand::RepoImportPlan { json, paths } => {
            let config = AppConfig::default();
            let plan = script_import_plan(
                ".",
                "Repos/bettergi-scripts-list/repo",
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                Vec::<String>::new(),
                &std::collections::BTreeMap::new(),
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("targets: {}", plan.targets.len());
                println!("unknown: {}", plan.unknown_paths.len());
                println!("subscriptions: {}", plan.merged_subscriptions.len());
            }
        }
        ScriptCommand::RepoImportExec {
            repo,
            json,
            git_repo,
            git,
            paths,
        } => {
            let config = AppConfig::default();
            let layout = script_repo_layout(
                ".",
                &config.script_config,
                &std::collections::BTreeMap::new(),
            );
            let existing =
                read_subscription_file(&layout.subscription_file_path).unwrap_or_default();
            let plan = script_import_plan(
                ".",
                &repo,
                &config.script_config,
                &std::collections::BTreeMap::new(),
                paths,
                existing,
                &std::collections::BTreeMap::new(),
            );
            let result = if git_repo {
                let mut runner = SystemGitRunner::new(git);
                execute_repo_import_with_git(&plan, Some(&mut runner))?
            } else {
                execute_file_repo_import(&plan)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("imported: {}", result.imported_targets.len());
                println!("unknown: {}", result.skipped_unknown_paths.len());
                println!("subscriptions: {}", result.subscriptions.len());
                println!("dependencies: {}", result.dependency_files_copied.len());
                println!("git_checkouts: {}", result.git_checkouts.len());
            }
        }
        ScriptCommand::RepoZipPlan { zip, json, folder } => {
            let plan = zip_import_plan(".", zip, folder.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("target: {}", plan.target_path.display());
                println!("temp: {}", plan.temp_unzip_dir.display());
                println!("marker: {}", plan.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoZipExec { zip, json, folder } => {
            let plan = zip_import_plan(".", zip, folder.as_deref());
            let result = execute_zip_repo_import(&plan)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("target: {}", result.target_path.display());
                println!("repo_json: {}", result.repo_json_path.display());
                println!("marker: {}", result.repo_updated_json_path.display());
                println!(
                    "best_overlap: {}",
                    result
                        .best_overlap_ratio
                        .map(|ratio| format!("{ratio:.3}"))
                        .unwrap_or_else(|| "-".to_string())
                );
                println!("marker_generated: {}", result.marker_generated);
            }
        }
        ScriptCommand::RepoGitPlan { repo_url, json } => {
            let plan = git_update_plan(".", repo_url, &std::collections::BTreeMap::new());
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("repo: {}", plan.repo_url);
                println!("branch: {}", plan.branch);
                println!("target: {}", plan.repo_path.display());
                println!("marker: {}", plan.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoGitUpdate {
            repo_url,
            json,
            git,
        } => {
            let plan = git_update_plan(".", repo_url, &std::collections::BTreeMap::new());
            let mut runner = SystemGitRunner::new(git);
            let result = execute_git_repo_update(&plan, &mut runner)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("target: {}", result.repo_path.display());
                println!("updated: {}", result.updated);
                println!("cloned: {}", result.cloned);
                println!("remote_changed: {}", result.remote_changed);
                println!("marker: {}", result.repo_updated_json_path.display());
            }
        }
        ScriptCommand::RepoGitCheckout {
            repo,
            source,
            destination,
            json,
            git,
            root,
        } => {
            let mut runner = SystemGitRunner::new(git);
            let result = checkout_git_repo_path(&mut runner, &repo, &source, &destination, !root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if let Some(result) = result {
                println!("source: {}", result.git_tree_path);
                println!("destination: {}", result.destination_path.display());
                println!("directory: {}", result.is_directory);
                println!("files: {}", result.files_written.len());
            } else {
                println!("not_found");
            }
        }
        ScriptCommand::RepoBridgePaths { repo, json, folder } => {
            let paths = script_repo_bridge_paths(".", &repo, folder.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&paths)?);
            } else {
                println!("repo: {}", paths.repo_path.display());
                println!("repo_json: {}", paths.repo_json_path.display());
                println!("subscriptions: {}", paths.subscription_file_path.display());
            }
        }
        ScriptCommand::RepoBridgeJson { repo } => {
            println!("{}", read_repo_bridge_repo_json(&repo)?);
        }
        ScriptCommand::RepoBridgeIndex { repo, json } => {
            let nodes = repo_bridge_index_nodes(&repo)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&nodes)?);
            } else {
                for node in nodes {
                    println!(
                        "{:<8} {:<5} {:<6} {}",
                        node.node_type,
                        if node.importable { "yes" } else { "no" },
                        if node.has_update { "update" } else { "-" },
                        node.path
                    );
                }
            }
        }
        ScriptCommand::RepoBridgeSubscribed { repo, folder } => {
            let paths = script_repo_bridge_paths(".", &repo, folder.as_deref())?;
            println!(
                "{}",
                repo_bridge_subscribed_paths_json(&paths.subscription_file_path)?
            );
        }
        ScriptCommand::RepoBridgeFile {
            repo,
            rel_path,
            json,
            git_repo,
            git,
        } => {
            let response = if git_repo {
                let mut runner = SystemGitRunner::new(git);
                read_repo_bridge_file_with_git(&repo, &rel_path, Some(&mut runner))?
            } else {
                read_repo_bridge_file(&repo, &rel_path)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else if let Some(response) = response {
                println!("{}", response.content);
            } else {
                println!("404");
            }
        }
        ScriptCommand::RepoBridgeMarkUpdated { repo, path, json } => {
            let paths = script_repo_bridge_paths(".", &repo, None)?;
            let updated = mark_repo_bridge_path_updated(&paths.repo_json_path, &path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "updated": updated,
                        "repo_json_path": paths.repo_json_path,
                    }))?
                );
            } else {
                println!("{updated}");
            }
        }
        ScriptCommand::RepoBridgeClearUpdate { repo, json } => {
            let path = clear_repo_bridge_update(&repo)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "repo_updated_json_path": path,
                    }))?
                );
            } else {
                println!("{}", path.display());
            }
        }
    }

    Ok(())
}

fn task(command: TaskCommand) -> Result<()> {
    match command {
        TaskCommand::Runtime { json } => {
            let triggers = runtime_triggers(false);
            let dispatcher = DispatcherRuntime::default();
            let selection = select_triggers_for_tick(
                &triggers,
                &dispatcher,
                std::time::Duration::from_secs(60),
            );
            let selected_count = selection.triggers.len();
            let payload = serde_json::json!({
                "dispatcher": dispatcher,
                "runner": RunnerRuntime::default(),
                "enabled_triggers": triggers.iter().filter(|trigger| trigger.enabled).count(),
                "selected_triggers": selection.triggers,
                "selection_reason": selection.reason,
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("dispatcher: {:?}", dispatcher.state);
                println!("runner: {:?}", RunnerRuntime::default().state);
                println!(
                    "selected_triggers: {} ({:?})",
                    selected_count, selection.reason
                );
            }
        }
        TaskCommand::Independent { json } => {
            let tasks = independent_tasks();
            if json {
                println!("{}", serde_json::to_string_pretty(&tasks)?);
            } else {
                for task in tasks {
                    println!(
                        "{:<22} main_ui_wait={} ported={} {}",
                        task.key, task.requires_main_ui_wait, task.ported, task.notes
                    );
                }
            }
        }
        TaskCommand::Catalog { json } => {
            let tasks = task_catalog();
            if json {
                println!("{}", serde_json::to_string_pretty(&tasks)?);
            } else {
                for task in tasks {
                    println!(
                        "{:<24} {:<16} config={:<30} state={:?}",
                        task.key,
                        format!("{:?}", task.kind),
                        task.config_section.unwrap_or("-"),
                        task.port_state
                    );
                }
            }
        }
        TaskCommand::Params { json } => {
            let params = task_parameter_models();
            if json {
                println!("{}", serde_json::to_string_pretty(&params)?);
            } else {
                println!(
                    "auto_domain_rounds: {}",
                    params.auto_domain.domain_round_num
                );
                println!(
                    "auto_boss_strategy: {} -> {}",
                    params.auto_boss.strategy_name, params.auto_boss.combat_strategy_path
                );
                println!("auto_fight_timeout: {}", params.auto_fight.timeout);
                println!(
                    "auto_stygian_resin_priority: {}",
                    params.auto_stygian_onslaught.resin_priority_list.join(",")
                );
            }
        }
    }

    Ok(())
}

fn notification(command: NotificationCommand, project_root: &Path) -> Result<()> {
    match command {
        NotificationCommand::Events { json } => {
            let events = notification_events();
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else {
                for event in events {
                    println!("{:<18} {}", event.code, event.message);
                }
            }
        }
        NotificationCommand::Providers { json, config } => {
            let config = load_app_config_or_default(project_root, config)?;
            let providers = notification_provider_plans(&config.notification_config);
            if json {
                println!("{}", serde_json::to_string_pretty(&providers)?);
            } else if providers.is_empty() {
                println!("providers: 0");
            } else {
                for provider in providers {
                    println!(
                        "{:<18} target={}",
                        provider.name,
                        provider.target_summary.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        NotificationCommand::Dispatch {
            json,
            config,
            event,
            result,
            message,
        } => {
            let config = load_app_config_or_default(project_root, config)?;
            let result = parse_notification_result(&result)?;
            let payload = NotificationPayload {
                event,
                result,
                message: Some(message),
                data: None,
                timestamp_ms: None,
                has_screenshot: false,
                screenshot: None,
            };
            let plan = notification_dispatch_plan(&config.notification_config, payload);
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("should_send: {}", plan.should_send);
                println!(
                    "providers: {} screenshot={}",
                    plan.providers.len(),
                    plan.include_screenshot
                );
                if let Some(reason) = plan.skipped_reason {
                    println!("skipped_reason: {reason}");
                }
            }
        }
    }

    Ok(())
}

fn update(command: UpdateCommand) -> Result<()> {
    match command {
        UpdateCommand::Plan { json, channel } => {
            let option = UpdateOption {
                trigger: UpdateTrigger::Auto,
                channel: parse_update_channel(&channel)?,
            };
            let plan = update_request_plan(option);
            if json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                println!("channel: {:?}", plan.channel);
                println!("url: {}", plan.url);
                if !plan.query.is_empty() {
                    println!(
                        "query: {}",
                        plan.query
                            .iter()
                            .map(|(key, value)| format!("{key}={value}"))
                            .collect::<Vec<_>>()
                            .join("&")
                    );
                }
            }
        }
        UpdateCommand::Decision {
            json,
            trigger,
            channel,
            current,
            latest,
            ignored,
        } => {
            let option = UpdateOption {
                trigger: parse_update_trigger(&trigger)?,
                channel: parse_update_channel(&channel)?,
            };
            let decision = update_decision(option, &current, ignored.as_deref(), latest.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&decision)?);
            } else {
                println!("action: {:?}", decision.action);
                println!(
                    "new_version: {}",
                    decision.new_version.as_deref().unwrap_or("-")
                );
                println!("download: {}", decision.download_page_url.unwrap_or("-"));
            }
        }
        UpdateCommand::Mirror {
            response_json,
            json,
        } => {
            let response: MirrorChyanLatestResponse = serde_json::from_str(&response_json)
                .context("failed to parse MirrorChyan latest response JSON")?;
            let outcome = bgi_core::mirror_chyan_latest_outcome(Some(&response));
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!("{outcome:?}");
            }
        }
        UpdateCommand::RedeemFeed {
            json,
            local,
            remote,
        } => {
            let decision = redeem_code_feed_update_decision(&local, remote.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&decision)?);
            } else {
                println!("request_url: {}", decision.request_url);
                println!("has_update: {}", decision.has_update);
                println!(
                    "remote_version: {}",
                    decision.remote_version.as_deref().unwrap_or("-")
                );
            }
        }
    }

    Ok(())
}

fn load_app_config_or_default(project_root: &Path, path: Option<PathBuf>) -> Result<AppConfig> {
    let path = path.unwrap_or_else(|| config_path(project_root));
    if path.exists() {
        read_config(&path).with_context(|| format!("failed to read config at {path:?}"))
    } else {
        Ok(AppConfig::default())
    }
}

fn parse_notification_result(value: &str) -> Result<bgi_core::NotificationEventResult> {
    match value.to_ascii_lowercase().as_str() {
        "success" => Ok(bgi_core::NotificationEventResult::Success),
        "fail" | "failed" | "error" => Ok(bgi_core::NotificationEventResult::Fail),
        "partial" | "partialsuccess" | "partial_success" => {
            Ok(bgi_core::NotificationEventResult::PartialSuccess)
        }
        _ => anyhow::bail!("notification result must be success, fail, or partial"),
    }
}

fn parse_update_channel(value: &str) -> Result<UpdateChannel> {
    match value.to_ascii_lowercase().as_str() {
        "stable" => Ok(UpdateChannel::Stable),
        "alpha" => Ok(UpdateChannel::Alpha),
        _ => anyhow::bail!("update channel must be stable or alpha"),
    }
}

fn parse_update_trigger(value: &str) -> Result<UpdateTrigger> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Ok(UpdateTrigger::Auto),
        "manual" => Ok(UpdateTrigger::Manual),
        _ => anyhow::bail!("update trigger must be auto or manual"),
    }
}

fn hotkey_command(value: &str) -> Result<()> {
    let hotkey: Hotkey = value.parse()?;
    println!("{}", serde_json::to_string_pretty(&hotkey)?);
    println!("{hotkey}");
    Ok(())
}

fn input_demo() -> Result<()> {
    let sequence = InputSequence::new()
        .modified_key_stroke([0x11], 0x41)
        .text("BetterGI")
        .mouse_click(MouseButton::Left)
        .vertical_scroll(1)
        .genshin_action(
            &AppConfig::default().key_bindings_config,
            GenshinAction::QuickUseGadget,
            KeyActionType::KeyPress,
        )?;
    let release_sequence = release_pressed_keys_sequence([0x57, 0xA0]);
    let post_message_events = post_message_events_for_action(
        &AppConfig::default().key_bindings_config,
        GenshinAction::QuickUseGadget,
        KeyActionType::KeyPress,
        PostMessageMode::Background,
    );
    let payload = serde_json::json!({
        "demo": sequence.events(),
        "release_pressed_keys": release_sequence.events(),
        "post_message_background_action": post_message_events,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
