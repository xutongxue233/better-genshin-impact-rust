use super::*;

#[path = "script_execution_commands.rs"]
mod script_execution_commands;
#[path = "script_host_runtime_command.rs"]
mod script_host_runtime_command;
#[path = "script_repo_commands.rs"]
mod script_repo_commands;

use script_execution_commands::{
    script_execute_group, script_execute_js, script_module_load, script_prepare_js,
};
use script_host_runtime_command::script_host_runtime;
use script_repo_commands::script_repo;

pub(crate) fn script(command: ScriptCommand) -> Result<()> {
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
        ScriptCommand::HostRuntime { json } => script_host_runtime(json)?,
        ScriptCommand::ModuleLoad {
            root,
            specifier,
            referrer,
            json,
        } => script_module_load(root, specifier, referrer, json)?,
        ScriptCommand::PrepareJs {
            scripts_root,
            folder,
            json,
        } => script_prepare_js(scripts_root, folder, json)?,
        ScriptCommand::ExecuteJs {
            scripts_root,
            folder,
            settings_json,
            json,
        } => script_execute_js(scripts_root, folder, settings_json, json)?,
        ScriptCommand::ExecuteGroup {
            app_root,
            group,
            json,
        } => script_execute_group(app_root, group, json)?,
        command @ (ScriptCommand::RepoChannels { .. }
        | ScriptCommand::RepoLayout { .. }
        | ScriptCommand::RepoUpdatePlan { .. }
        | ScriptCommand::RepoImportUri { .. }
        | ScriptCommand::RepoImportPlan { .. }
        | ScriptCommand::RepoImportExec { .. }
        | ScriptCommand::RepoZipPlan { .. }
        | ScriptCommand::RepoZipExec { .. }
        | ScriptCommand::RepoGitPlan { .. }
        | ScriptCommand::RepoGitUpdate { .. }
        | ScriptCommand::RepoGitCheckout { .. }
        | ScriptCommand::RepoBridgePaths { .. }
        | ScriptCommand::RepoBridgeJson { .. }
        | ScriptCommand::RepoBridgeIndex { .. }
        | ScriptCommand::RepoBridgeSubscribed { .. }
        | ScriptCommand::RepoBridgeFile { .. }
        | ScriptCommand::RepoBridgeMarkUpdated { .. }
        | ScriptCommand::RepoBridgeClearUpdate { .. }) => script_repo(command)?,
    }

    Ok(())
}
