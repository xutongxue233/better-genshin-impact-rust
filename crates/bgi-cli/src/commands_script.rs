use std::path::PathBuf;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum ScriptCommand {
    Runtime {
        #[arg(long)]
        json: bool,
    },
    Hosts {
        #[arg(long)]
        json: bool,
    },
    Engines {
        #[arg(long)]
        json: bool,
    },
    Policy {
        #[arg(long)]
        json: bool,
    },
    Loader {
        #[arg(long)]
        json: bool,
    },
    Settings {
        #[arg(long)]
        json: bool,
    },
    Macro {
        file: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        events: bool,
    },
    KeyMouseHost {
        root: PathBuf,
        file: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        events: bool,
    },
    GlobalInput {
        #[arg(long)]
        json: bool,
    },
    HostRuntime {
        #[arg(long)]
        json: bool,
    },
    ModuleLoad {
        root: PathBuf,
        specifier: String,
        #[arg(long)]
        referrer: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    PrepareJs {
        scripts_root: PathBuf,
        folder: String,
        #[arg(long)]
        json: bool,
    },
    ExecuteJs {
        scripts_root: PathBuf,
        folder: String,
        #[arg(long)]
        settings_json: Option<String>,
        #[arg(long)]
        json: bool,
    },
    ExecuteGroup {
        app_root: PathBuf,
        group: String,
        #[arg(long)]
        json: bool,
    },
    RepoChannels {
        #[arg(long)]
        json: bool,
    },
    RepoLayout {
        #[arg(long)]
        json: bool,
    },
    RepoUpdatePlan {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        manual: bool,
        paths: Vec<String>,
    },
    RepoImportUri {
        uri: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        clipboard: bool,
    },
    RepoImportPlan {
        #[arg(long)]
        json: bool,
        paths: Vec<String>,
    },
    RepoImportExec {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        git_repo: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
        paths: Vec<String>,
    },
    RepoZipPlan {
        zip: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoZipExec {
        zip: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoGitPlan {
        repo_url: String,
        #[arg(long)]
        json: bool,
    },
    RepoGitUpdate {
        repo_url: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
    },
    RepoGitCheckout {
        repo: PathBuf,
        source: String,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
        #[arg(long)]
        root: bool,
    },
    RepoBridgePaths {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoBridgeJson {
        repo: PathBuf,
    },
    RepoBridgeIndex {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
    },
    RepoBridgeSubscribed {
        repo: PathBuf,
        #[arg(long)]
        folder: Option<String>,
    },
    RepoBridgeFile {
        repo: PathBuf,
        rel_path: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        git_repo: bool,
        #[arg(long, default_value = "git")]
        git: PathBuf,
    },
    RepoBridgeMarkUpdated {
        repo: PathBuf,
        path: String,
        #[arg(long)]
        json: bool,
    },
    RepoBridgeClearUpdate {
        repo: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
