use super::{
    AssetsCommand, CaptureCommand, ConfigCommand, NotificationCommand, PathingCommand,
    ScriptCommand, TaskCommand, UpdateCommand, VisionCommand,
};
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(name = "bgi")]
#[command(about = "Rust migration CLI for BetterGI")]
pub(crate) struct Cli {
    #[arg(long, default_value = ".", global = true)]
    pub(crate) project_root: PathBuf,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Commands {
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
