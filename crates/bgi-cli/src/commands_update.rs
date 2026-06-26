#[derive(Debug, clap::Subcommand)]
pub(crate) enum UpdateCommand {
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
