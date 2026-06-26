use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NOTICE_URL: &str = "https://hui-config.oss-cn-hangzhou.aliyuncs.com/bgi/notice.json";
pub const DOWNLOAD_PAGE_URL: &str = "https://www.bettergi.com/download.html";
pub const MIRROR_CHYAN_LATEST_URL: &str = "https://mirrorchyan.com/api/resources/BGI/latest";
pub const ALPHA_RELEASES_URL: &str = "https://cnb.cool/bettergi/better-genshin-impact/-/releases";
pub const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/babalae/better-genshin-impact/releases/latest";

#[path = "update_redeem_code.rs"]
mod update_redeem_code;

pub use update_redeem_code::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateTrigger {
    Auto,
    Manual,
}

impl Default for UpdateTrigger {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Alpha,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UpdateOption {
    pub trigger: UpdateTrigger,
    pub channel: UpdateChannel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateRequestPlan {
    pub channel: UpdateChannel,
    pub url: String,
    pub query: BTreeMap<&'static str, &'static str>,
    pub user_agent: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub version: String,
    pub gray: i32,
}

impl Default for Notice {
    fn default() -> Self {
        Self {
            version: String::new(),
            gray: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorChyanLatestResponse {
    pub code: i64,
    pub data: Option<MirrorChyanLatestData>,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorChyanLatestData {
    pub arch: String,
    pub cdk_expired_time: Option<f64>,
    pub channel: String,
    pub custom_data: Option<String>,
    pub filesize: Option<i64>,
    pub os: String,
    pub release_note: String,
    pub sha256: Option<String>,
    pub update_type: Option<String>,
    pub url: Option<String>,
    pub version_name: String,
    pub version_number: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MirrorChyanLatestOutcome {
    Version(String),
    Empty,
    Warning { code: i64, message: String },
    Severe { code: i64, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum UpdateDecisionAction {
    Noop,
    ShowUpToDateMessage,
    OpenUpdateWindow,
    SuppressedByIgnoredVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateDecision {
    pub action: UpdateDecisionAction,
    pub new_version: Option<String>,
    pub download_page_url: Option<&'static str>,
    pub release_notes_request: Option<UpdateRequestPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdaterSource {
    Default,
    Cnb,
    Github,
    Dfs,
    DfsAlpha,
    MirrorChyan,
    MirrorChyanAlpha,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdaterLaunchPlan {
    pub source: UpdaterSource,
    pub display_name: &'static str,
    pub args: Vec<String>,
    pub source_arg: Option<&'static str>,
    pub requires_cdk: bool,
    pub warning: Option<&'static str>,
}

pub fn update_request_plan(option: UpdateOption) -> UpdateRequestPlan {
    match option.channel {
        UpdateChannel::Stable => UpdateRequestPlan {
            channel: UpdateChannel::Stable,
            url: NOTICE_URL.to_string(),
            query: BTreeMap::new(),
            user_agent: None,
        },
        UpdateChannel::Alpha => UpdateRequestPlan {
            channel: UpdateChannel::Alpha,
            url: MIRROR_CHYAN_LATEST_URL.to_string(),
            query: BTreeMap::from([
                ("user_agent", "BetterGI"),
                ("os", "win"),
                ("arch", "x64"),
                ("channel", "alpha"),
            ]),
            user_agent: None,
        },
    }
}

pub fn stable_release_notes_request_plan() -> UpdateRequestPlan {
    UpdateRequestPlan {
        channel: UpdateChannel::Stable,
        url: GITHUB_LATEST_RELEASE_URL.to_string(),
        query: BTreeMap::new(),
        user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
    }
}

pub fn latest_version_from_notice(notice: &Notice, device_hash_mod10: u32) -> Option<String> {
    if notice.version.trim().is_empty() {
        return None;
    }
    (device_hash_mod10 % 10 < notice.gray.max(0) as u32).then(|| notice.version.clone())
}

pub fn mirror_chyan_latest_outcome(
    response: Option<&MirrorChyanLatestResponse>,
) -> MirrorChyanLatestOutcome {
    let Some(response) = response else {
        return MirrorChyanLatestOutcome::Empty;
    };

    if response.code == 0 {
        return response
            .data
            .as_ref()
            .map(|data| MirrorChyanLatestOutcome::Version(data.version_name.clone()))
            .unwrap_or(MirrorChyanLatestOutcome::Empty);
    }

    if response.code < 0 {
        return MirrorChyanLatestOutcome::Severe {
            code: response.code,
            message: format!(
                "Mirror酱源更新检查失败，意料之外的严重错误，请及时联系 Mirror 酱的技术支持处理，错误代码：{}，错误信息：{}",
                response.code, response.msg
            ),
        };
    }

    MirrorChyanLatestOutcome::Warning {
        code: response.code,
        message: mirror_chyan_warning_message(response.code, &response.msg),
    }
}

pub fn mirror_chyan_warning_message(code: i64, msg: &str) -> String {
    match code {
        7001 => "Mirror酱 CDK 已过期，请重新获取CDK".to_string(),
        7002 => "Mirror酱 CDK 错误!".to_string(),
        7003 => "Mirror酱 CDK 今日下载次数已达上限".to_string(),
        7004 => "Mirror酱 CDK 类型和待下载的资源不匹配".to_string(),
        7005 => "Mirror酱 CDK 已被封禁".to_string(),
        _ => format!("Mirror酱源更新检查失败，错误信息：{msg}"),
    }
}

pub fn is_new_version(old_version: &str, current_version: &str) -> bool {
    let Ok(old_version) = Version::parse(old_version) else {
        return false;
    };
    let Ok(current_version) = Version::parse(current_version) else {
        return false;
    };
    current_version > old_version
}

pub fn update_decision(
    option: UpdateOption,
    app_version: &str,
    ignored_end_version: Option<&str>,
    latest_version: Option<&str>,
) -> UpdateDecision {
    let Some(latest_version) = latest_version
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return UpdateDecision {
            action: UpdateDecisionAction::Noop,
            new_version: None,
            download_page_url: None,
            release_notes_request: None,
        };
    };

    if !is_new_version(app_version, latest_version) {
        return UpdateDecision {
            action: if option.trigger == UpdateTrigger::Manual {
                UpdateDecisionAction::ShowUpToDateMessage
            } else {
                UpdateDecisionAction::Noop
            },
            new_version: Some(latest_version.to_string()),
            download_page_url: None,
            release_notes_request: None,
        };
    }

    if option.trigger == UpdateTrigger::Auto
        && ignored_end_version
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some_and(|ignored| !is_new_version(ignored, latest_version))
    {
        return UpdateDecision {
            action: UpdateDecisionAction::SuppressedByIgnoredVersion,
            new_version: Some(latest_version.to_string()),
            download_page_url: None,
            release_notes_request: None,
        };
    }

    UpdateDecision {
        action: UpdateDecisionAction::OpenUpdateWindow,
        new_version: Some(latest_version.to_string()),
        download_page_url: Some(update_download_page_url(option.channel)),
        release_notes_request: (option.channel == UpdateChannel::Stable)
            .then(stable_release_notes_request_plan),
    }
}

pub fn update_download_page_url(channel: UpdateChannel) -> &'static str {
    match channel {
        UpdateChannel::Stable => DOWNLOAD_PAGE_URL,
        UpdateChannel::Alpha => ALPHA_RELEASES_URL,
    }
}

pub fn updater_launch_options(channel: UpdateChannel) -> Vec<UpdaterLaunchPlan> {
    match channel {
        UpdateChannel::Stable => vec![
            updater_launch_plan_for_source(UpdaterSource::Default),
            updater_launch_plan_for_source(UpdaterSource::Cnb),
            updater_launch_plan_for_source(UpdaterSource::Github),
            updater_launch_plan_for_source(UpdaterSource::MirrorChyan),
        ],
        UpdateChannel::Alpha => vec![
            updater_launch_plan_for_source(UpdaterSource::DfsAlpha),
            updater_launch_plan_for_source(UpdaterSource::MirrorChyanAlpha),
        ],
    }
}

pub fn updater_launch_plan(source: Option<&str>) -> std::result::Result<UpdaterLaunchPlan, String> {
    let source = parse_updater_source(source)?;
    Ok(updater_launch_plan_for_source(source))
}

pub fn parse_updater_source(source: Option<&str>) -> std::result::Result<UpdaterSource, String> {
    let normalized = source.unwrap_or_default().trim().to_ascii_lowercase();
    let source = match normalized.as_str() {
        "" | "default" | "steambird-stable" | "stable" => UpdaterSource::Default,
        "cnb" => UpdaterSource::Cnb,
        "github" => UpdaterSource::Github,
        "dfs" | "steambird" => UpdaterSource::Dfs,
        "dfs-alpha" | "dfsalpha" | "steambird-alpha" | "steambirdalpha" => UpdaterSource::DfsAlpha,
        "mirrorc"
        | "mirror-chyan"
        | "mirrorchyan"
        | "mirror-chyan-stable"
        | "mirrorchyanstable" => UpdaterSource::MirrorChyan,
        "mirrorc-alpha" | "mirror-chyan-alpha" | "mirrorchyan-alpha" | "mirrorchyanalpha" => {
            UpdaterSource::MirrorChyanAlpha
        }
        other => return Err(format!("unsupported updater source: {other}")),
    };
    Ok(source)
}

pub fn updater_launch_plan_for_source(source: UpdaterSource) -> UpdaterLaunchPlan {
    let (display_name, source_arg, requires_cdk, warning) = match source {
        UpdaterSource::Default => ("Steambird", None, false, None),
        UpdaterSource::Cnb => ("CNB", Some("cnb"), false, None),
        UpdaterSource::Github => (
            "GitHub",
            Some("github"),
            false,
            Some("GitHub source may be slow or unavailable on restricted networks"),
        ),
        UpdaterSource::Dfs => ("Steambird", Some("dfs"), false, None),
        UpdaterSource::DfsAlpha => ("Steambird Alpha", Some("dfs-alpha"), false, None),
        UpdaterSource::MirrorChyan => (
            "MirrorChyan",
            Some("mirrorc"),
            true,
            Some("MirrorChyan may require a CDK"),
        ),
        UpdaterSource::MirrorChyanAlpha => (
            "MirrorChyan Alpha",
            Some("mirrorc-alpha"),
            true,
            Some("MirrorChyan may require a CDK"),
        ),
    };
    let mut args = vec!["-I".to_string()];
    if let Some(source_arg) = source_arg {
        args.push("--source".to_string());
        args.push(source_arg.to_string());
    }
    UpdaterLaunchPlan {
        source,
        display_name,
        args,
        source_arg,
        requires_cdk,
        warning,
    }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
