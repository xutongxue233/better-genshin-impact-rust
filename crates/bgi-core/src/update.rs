use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NOTICE_URL: &str = "https://hui-config.oss-cn-hangzhou.aliyuncs.com/bgi/notice.json";
pub const DOWNLOAD_PAGE_URL: &str = "https://www.bettergi.com/download.html";
pub const MIRROR_CHYAN_LATEST_URL: &str = "https://mirrorchyan.com/api/resources/BGI/latest";
pub const ALPHA_RELEASES_URL: &str = "https://cnb.cool/bettergi/better-genshin-impact/-/releases";
pub const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/babalae/better-genshin-impact/releases/latest";
pub const REDEEM_CODE_UPDATE_TIME_URL: &str =
    "https://cnb.cool/bettergi/genshin-redeem-code/-/git/raw/main/update_time.txt";
pub const REDEEM_CODE_CODES_URL: &str =
    "https://cnb.cool/bettergi/genshin-redeem-code/-/git/raw/main/codes.json";
pub const REDEEM_CODE_BBS_ACT_ID_1_URL: &str =
    "https://bbs-api.mihoyo.com/painter/api/user_instant/list?offset=0&size=20&uid=75276539";
pub const REDEEM_CODE_BBS_ACT_ID_2_URL: &str =
    "https://bbs-api.mihoyo.com/painter/api/user_instant/list?offset=0&size=20&uid=75276550";
pub const REDEEM_CODE_LIVE_INDEX_URL: &str = "https://api-takumi.mihoyo.com/event/miyolive/index";
pub const REDEEM_CODE_LIVE_REFRESH_CODE_URL: &str =
    "https://api-takumi-static.mihoyo.com/event/miyolive/refreshCode";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedeemCodeFeedUpdateDecision {
    pub request_url: &'static str,
    pub has_update: bool,
    pub remote_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase", default)]
pub struct RedeemCodeFeedItem {
    pub title: String,
    pub content: String,
    pub time: String,
    pub tag: String,
    pub valid: String,
    pub codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemCodeLiveCode {
    pub code: String,
    pub items: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedeemCodeLiveData {
    pub act_id: String,
    pub code_version: String,
    pub title: String,
    pub codes: Vec<RedeemCodeLiveCode>,
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

pub fn redeem_code_feed_update_decision(
    local_version: &str,
    remote_version_text: Option<&str>,
) -> RedeemCodeFeedUpdateDecision {
    let remote_version = remote_version_text
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let has_update = remote_version
        .as_deref()
        .and_then(|remote| {
            Some((
                local_version.trim().parse::<i64>().ok()?,
                remote.parse::<i64>().ok()?,
            ))
        })
        .is_some_and(|(local, remote)| remote > local);

    RedeemCodeFeedUpdateDecision {
        request_url: REDEEM_CODE_UPDATE_TIME_URL,
        has_update,
        remote_version,
    }
}

pub fn parse_redeem_code_feed_items(json: &str) -> serde_json::Result<Vec<RedeemCodeFeedItem>> {
    serde_json::from_str(json)
}

pub fn redeem_code_live_act_id_from_bbs_response(json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    if value.get("error").is_some() || value.get("retcode")?.as_i64()? != 0 {
        return None;
    }
    let list = value.get("data")?.get("list")?.as_array()?;
    for entry in list {
        let Some(post) = entry.get("post").and_then(|post| post.get("post")) else {
            continue;
        };
        let Some(subject) = post.get("subject").and_then(|subject| subject.as_str()) else {
            continue;
        };
        if !subject.contains("前瞻特别节目") {
            continue;
        }
        let Some(structured_content) = post
            .get("structured_content")
            .and_then(|content| content.as_str())
        else {
            continue;
        };
        let Ok(segments) = serde_json::from_str::<serde_json::Value>(structured_content) else {
            continue;
        };
        let Some(segments) = segments.as_array() else {
            continue;
        };
        for segment in segments {
            let link = segment
                .get("attributes")
                .and_then(|attributes| attributes.get("link"))
                .and_then(|link| link.as_str())
                .unwrap_or_default();
            if link.is_empty() {
                continue;
            }
            let insert = segment
                .get("insert")
                .and_then(|insert| insert.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| {
                    segment
                        .get("insert")
                        .map(ToString::to_string)
                        .unwrap_or_default()
                });
            if !(insert.contains("观看") || insert.contains("米游社直播间")) {
                continue;
            }
            if let Some(act_id) = act_id_from_link(link) {
                return Some(act_id);
            }
        }
    }
    None
}

pub fn redeem_code_live_index_from_response(json: &str) -> Option<(String, String)> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    if value.get("error").is_some() || value.get("retcode")?.as_i64()? != 0 {
        return None;
    }
    let live = value.get("data")?.get("live")?;
    let code_version = live.get("code_ver")?.as_str()?.trim();
    if code_version.is_empty() {
        return None;
    }
    let title = live
        .get("title")
        .and_then(|title| title.as_str())
        .unwrap_or_default()
        .to_string();
    Some((code_version.to_string(), title))
}

pub fn redeem_code_live_codes_from_response(json: &str) -> Vec<RedeemCodeLiveCode> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    if value.get("error").is_some()
        || value.get("retcode").and_then(|code| code.as_i64()) != Some(0)
    {
        return Vec::new();
    }
    let Some(code_list) = value
        .get("data")
        .and_then(|data| data.get("code_list"))
        .and_then(|code_list| code_list.as_array())
    else {
        return Vec::new();
    };
    code_list
        .iter()
        .filter_map(|code_info| {
            let code = code_info.get("code")?.as_str()?.trim();
            if code.is_empty() {
                return None;
            }
            let items = strip_html_tags(
                code_info
                    .get("title")
                    .and_then(|title| title.as_str())
                    .unwrap_or_default(),
            );
            Some(RedeemCodeLiveCode {
                code: code.to_string(),
                items,
            })
        })
        .collect()
}

fn act_id_from_link(link: &str) -> Option<String> {
    let start = link.find("act_id=")? + "act_id=".len();
    let tail = &link[start..];
    let end = tail.find('&').unwrap_or(tail.len());
    let act_id = tail[..end].trim();
    (!act_id.is_empty()).then(|| act_id.to_string())
}

fn strip_html_tags(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_request_plans_preserve_legacy_urls_and_alpha_query() {
        let stable = update_request_plan(UpdateOption::default());
        assert_eq!(stable.url, NOTICE_URL);
        assert!(stable.query.is_empty());

        let alpha = update_request_plan(UpdateOption {
            trigger: UpdateTrigger::Manual,
            channel: UpdateChannel::Alpha,
        });
        assert_eq!(alpha.url, MIRROR_CHYAN_LATEST_URL);
        assert_eq!(alpha.query["user_agent"], "BetterGI");
        assert_eq!(alpha.query["os"], "win");
        assert_eq!(alpha.query["arch"], "x64");
        assert_eq!(alpha.query["channel"], "alpha");
    }

    #[test]
    fn stable_notice_applies_gray_gate_and_empty_version_guard() {
        let notice = Notice {
            version: "1.2.3".to_string(),
            gray: 10,
        };
        assert_eq!(
            latest_version_from_notice(&notice, 9),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            latest_version_from_notice(&Notice { gray: 0, ..notice }, 0),
            None
        );
        assert_eq!(latest_version_from_notice(&Notice::default(), 0), None);
    }

    #[test]
    fn mirror_chyan_outcome_matches_legacy_error_mapping() {
        let response = MirrorChyanLatestResponse {
            code: 0,
            msg: "ok".to_string(),
            data: Some(MirrorChyanLatestData {
                arch: "x64".to_string(),
                cdk_expired_time: None,
                channel: "alpha".to_string(),
                custom_data: None,
                filesize: None,
                os: "win".to_string(),
                release_note: String::new(),
                sha256: None,
                update_type: Some("full".to_string()),
                url: None,
                version_name: "0.99.0-alpha.1".to_string(),
                version_number: 99,
            }),
        };
        assert_eq!(
            mirror_chyan_latest_outcome(Some(&response)),
            MirrorChyanLatestOutcome::Version("0.99.0-alpha.1".to_string())
        );

        let warning = MirrorChyanLatestResponse {
            code: 7002,
            msg: "bad cdk".to_string(),
            data: None,
        };
        assert_eq!(
            mirror_chyan_latest_outcome(Some(&warning)),
            MirrorChyanLatestOutcome::Warning {
                code: 7002,
                message: "Mirror酱 CDK 错误!".to_string()
            }
        );

        let severe = MirrorChyanLatestResponse {
            code: -1,
            msg: "boom".to_string(),
            data: None,
        };
        assert!(matches!(
            mirror_chyan_latest_outcome(Some(&severe)),
            MirrorChyanLatestOutcome::Severe { code: -1, .. }
        ));
    }

    #[test]
    fn update_decision_matches_legacy_manual_auto_and_ignore_rules() {
        let auto = UpdateOption::default();
        assert_eq!(
            update_decision(auto, "1.0.0", None, None).action,
            UpdateDecisionAction::Noop
        );
        assert_eq!(
            update_decision(auto, "1.0.0", None, Some("1.0.0")).action,
            UpdateDecisionAction::Noop
        );
        assert_eq!(
            update_decision(
                UpdateOption {
                    trigger: UpdateTrigger::Manual,
                    channel: UpdateChannel::Stable,
                },
                "1.0.0",
                None,
                Some("1.0.0"),
            )
            .action,
            UpdateDecisionAction::ShowUpToDateMessage
        );
        assert_eq!(
            update_decision(auto, "1.0.0", Some("1.2.0"), Some("1.1.0")).action,
            UpdateDecisionAction::SuppressedByIgnoredVersion
        );
        let decision = update_decision(auto, "1.0.0", Some("1.0.5"), Some("1.1.0"));
        assert_eq!(decision.action, UpdateDecisionAction::OpenUpdateWindow);
        assert_eq!(decision.download_page_url, Some(DOWNLOAD_PAGE_URL));
        assert!(decision.release_notes_request.is_some());
    }

    #[test]
    fn updater_launch_options_preserve_legacy_sources_and_args() {
        let stable = updater_launch_options(UpdateChannel::Stable);
        assert_eq!(
            stable
                .iter()
                .map(|option| option.source)
                .collect::<Vec<_>>(),
            vec![
                UpdaterSource::Default,
                UpdaterSource::Cnb,
                UpdaterSource::Github,
                UpdaterSource::MirrorChyan,
            ]
        );
        assert_eq!(stable[0].args, ["-I"]);
        assert_eq!(stable[1].args, ["-I", "--source", "cnb"]);
        assert_eq!(stable[2].args, ["-I", "--source", "github"]);
        assert_eq!(stable[3].args, ["-I", "--source", "mirrorc"]);
        assert!(stable[3].requires_cdk);

        let alpha = updater_launch_options(UpdateChannel::Alpha);
        assert_eq!(
            alpha.iter().map(|option| option.source).collect::<Vec<_>>(),
            vec![UpdaterSource::DfsAlpha, UpdaterSource::MirrorChyanAlpha]
        );
        assert_eq!(alpha[0].args, ["-I", "--source", "dfs-alpha"]);
        assert_eq!(alpha[1].args, ["-I", "--source", "mirrorc-alpha"]);
    }

    #[test]
    fn updater_launch_plan_parses_legacy_aliases_and_rejects_unknown_sources() {
        assert_eq!(
            updater_launch_plan(None).unwrap().source,
            UpdaterSource::Default
        );
        assert_eq!(updater_launch_plan(Some("default")).unwrap().args, ["-I"]);
        assert_eq!(
            updater_launch_plan(Some("steambird-alpha"))
                .unwrap()
                .source_arg,
            Some("dfs-alpha")
        );
        assert_eq!(
            updater_launch_plan(Some("mirror-chyan-alpha"))
                .unwrap()
                .source,
            UpdaterSource::MirrorChyanAlpha
        );
        assert!(updater_launch_plan(Some("unknown")).is_err());
    }

    #[test]
    fn version_comparison_returns_false_on_invalid_versions() {
        assert!(is_new_version("1.0.0", "1.0.1"));
        assert!(!is_new_version("1.0.1", "1.0.0"));
        assert!(!is_new_version("dev", "1.0.0"));
        assert!(!is_new_version("1.0.0", "dev"));
    }

    #[test]
    fn redeem_code_feed_update_decision_matches_legacy_numeric_compare() {
        let update = redeem_code_feed_update_decision("20251013", Some("20251014"));
        assert!(update.has_update);
        assert_eq!(update.remote_version, Some("20251014".to_string()));
        assert_eq!(update.request_url, REDEEM_CODE_UPDATE_TIME_URL);

        assert!(!redeem_code_feed_update_decision("20251013", Some("20251013")).has_update);
        assert!(!redeem_code_feed_update_decision("20251013", Some("abc")).has_update);
        assert!(!redeem_code_feed_update_decision("abc", Some("20251014")).has_update);
        assert!(!redeem_code_feed_update_decision("20251013", None).has_update);
    }

    #[test]
    fn redeem_code_feed_items_deserialize_legacy_pascal_case_json() {
        let items = parse_redeem_code_feed_items(
            r#"[
              {
                "Title": "前瞻直播兑换码",
                "Content": "原石 * 300",
                "Time": "2026-06-20 20:00",
                "Tag": "前瞻",
                "Valid": "2026-06-21",
                "Codes": ["GENSHINGIFT", "ABC123"]
              }
            ]"#,
        )
        .expect("feed json should parse");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "前瞻直播兑换码");
        assert_eq!(items[0].codes, ["GENSHINGIFT", "ABC123"]);
        assert_eq!(items[0].tag, "前瞻");
    }

    #[test]
    fn redeem_code_live_act_id_is_extracted_from_legacy_bbs_shape() {
        let structured = serde_json::json!([
            { "insert": "无关文本", "attributes": { "link": "" } },
            {
                "insert": "观看直播",
                "attributes": {
                    "link": "https://webstatic.mihoyo.com/event?act_id=e20260620preview&utm=1"
                }
            }
        ])
        .to_string();
        let response = serde_json::json!({
            "retcode": 0,
            "data": {
                "list": [
                    {
                        "post": {
                            "post": {
                                "subject": "原神前瞻特别节目",
                                "structured_content": structured
                            }
                        }
                    }
                ]
            }
        });

        assert_eq!(
            redeem_code_live_act_id_from_bbs_response(&response.to_string()).as_deref(),
            Some("e20260620preview")
        );
    }

    #[test]
    fn redeem_code_live_index_extracts_code_version_and_title() {
        let response = serde_json::json!({
            "retcode": 0,
            "data": {
                "live": {
                    "code_ver": "v5",
                    "title": "5.0 前瞻特别节目"
                }
            }
        });

        assert_eq!(
            redeem_code_live_index_from_response(&response.to_string()),
            Some(("v5".to_string(), "5.0 前瞻特别节目".to_string()))
        );
    }

    #[test]
    fn redeem_code_live_codes_strip_html_titles() {
        let response = serde_json::json!({
            "retcode": 0,
            "data": {
                "code_list": [
                    { "title": "<b>原石</b> * 100", "code": "LIVE100" },
                    { "title": "摩拉", "code": "LIVE200" },
                    { "title": "empty", "code": "" }
                ]
            }
        });

        let codes = redeem_code_live_codes_from_response(&response.to_string());
        assert_eq!(
            codes,
            vec![
                RedeemCodeLiveCode {
                    code: "LIVE100".to_string(),
                    items: "原石 * 100".to_string(),
                },
                RedeemCodeLiveCode {
                    code: "LIVE200".to_string(),
                    items: "摩拉".to_string(),
                },
            ]
        );
    }
}
