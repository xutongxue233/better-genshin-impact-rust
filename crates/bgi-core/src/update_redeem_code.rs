use serde::{Deserialize, Serialize};

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
