#[derive(Debug, clap::Subcommand)]
pub(crate) enum TaskCommand {
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
