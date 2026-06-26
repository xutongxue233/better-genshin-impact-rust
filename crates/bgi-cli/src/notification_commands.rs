use super::*;

pub(crate) fn notification(command: NotificationCommand, project_root: &Path) -> Result<()> {
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
