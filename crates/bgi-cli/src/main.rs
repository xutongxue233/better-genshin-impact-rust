use anyhow::{Context, Result};
use bgi_capture::capture_mode_infos;
use bgi_core::{
    config_path, initial_triggers, migration_capabilities, notification_dispatch_plan,
    notification_events, notification_provider_plans, read_config, read_pathing_task, write_config,
    AppConfig, AssetResolver, GenshinAction, NotificationPayload, ScreenSize,
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
    independent_tasks, runtime_triggers, select_triggers_for_tick, task_asset_root, task_catalog,
    task_parameter_models, DispatcherRuntime, RunnerRuntime,
};
use bgi_vision::{
    recognition_type_infos, registered_onnx_models, BgrImage, BvImage, BvLocatorOperation, BvPage,
    Rect, Size,
};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

mod commands;
mod notification_commands;
mod script_commands;
mod update_commands;

use commands::*;
use notification_commands::notification;
use script_commands::script;
use update_commands::update;

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
    let resolver =
        AssetResolver::new(project_root.to_path_buf()).with_fallback_root(task_asset_root());

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
