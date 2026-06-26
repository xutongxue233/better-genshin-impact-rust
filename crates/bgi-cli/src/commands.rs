#[path = "commands_cli.rs"]
mod commands_cli;
#[path = "commands_core.rs"]
mod commands_core;
#[path = "commands_notification.rs"]
mod commands_notification;
#[path = "commands_script.rs"]
mod commands_script;
#[path = "commands_task.rs"]
mod commands_task;
#[path = "commands_update.rs"]
mod commands_update;

pub(crate) use commands_cli::{Cli, Commands};
pub(crate) use commands_core::{
    AssetsCommand, CaptureCommand, ConfigCommand, PathingCommand, VisionCommand,
};
pub(crate) use commands_notification::NotificationCommand;
pub(crate) use commands_script::ScriptCommand;
pub(crate) use commands_task::TaskCommand;
pub(crate) use commands_update::UpdateCommand;
