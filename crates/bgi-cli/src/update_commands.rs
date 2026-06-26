use anyhow::{Context, Result};
use bgi_core::{
    redeem_code_feed_update_decision, update_decision, update_request_plan,
    MirrorChyanLatestResponse, UpdateChannel, UpdateOption, UpdateTrigger,
};

use crate::commands::UpdateCommand;

pub(crate) fn update(command: UpdateCommand) -> Result<()> {
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
