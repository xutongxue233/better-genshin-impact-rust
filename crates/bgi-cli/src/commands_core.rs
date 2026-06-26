use std::path::PathBuf;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum ConfigCommand {
    Show {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    WriteDefault {
        path: PathBuf,
    },
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum AssetsCommand {
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

#[derive(Debug, clap::Subcommand)]
pub(crate) enum PathingCommand {
    Validate {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum CaptureCommand {
    Modes {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum VisionCommand {
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
