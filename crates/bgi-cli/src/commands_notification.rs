use std::path::PathBuf;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum NotificationCommand {
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
