use super::{ScriptRepoChannel, DEFAULT_REPO_FOLDER_NAME};
use bgi_core::config::ScriptConfig;
use std::collections::BTreeMap;

pub fn script_repo_channels() -> Vec<ScriptRepoChannel> {
    vec![
        ScriptRepoChannel {
            name: "CNB",
            url: "https://cnb.cool/bettergi/bettergi-scripts-list",
        },
        ScriptRepoChannel {
            name: "GitCode",
            url: "https://gitcode.com/huiyadanli/bettergi-scripts-list",
        },
        ScriptRepoChannel {
            name: "GitHub",
            url: "https://github.com/babalae/bettergi-scripts-list",
        },
    ]
}

pub fn resolve_repo_url(config: &ScriptConfig) -> Option<String> {
    if config.selected_channel_name.is_empty() {
        return Some(script_repo_channels()[0].url.to_string());
    }
    if config.selected_channel_name == "自定义" {
        let custom_url = config.custom_repo_url.trim();
        if !custom_url.is_empty() && custom_url != "https://example.com/custom-repo" {
            return Some(custom_url.to_string());
        }
        return None;
    }

    script_repo_channels()
        .into_iter()
        .find(|channel| channel.name == config.selected_channel_name)
        .map(|channel| channel.url.to_string())
        .or_else(|| Some("https://cnb.cool/bettergi/bettergi-scripts-list".to_string()))
}

pub fn repo_folder_name(
    repo_url: Option<&str>,
    folder_mapping: &BTreeMap<String, String>,
) -> String {
    let Some(repo_url) = repo_url.filter(|value| !value.trim().is_empty()) else {
        return DEFAULT_REPO_FOLDER_NAME.to_string();
    };
    let trimmed_url = repo_url.trim_end_matches('/');
    if let Some(saved) = folder_mapping
        .get(trimmed_url)
        .filter(|value| !value.trim().is_empty())
    {
        return saved.clone();
    }
    derive_base_folder_name(trimmed_url)
}

pub fn derive_base_folder_name(repo_url: &str) -> String {
    let trimmed_url = repo_url.trim_end_matches('/');
    let last_segment = trimmed_url
        .rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(DEFAULT_REPO_FOLDER_NAME);
    let without_git = if last_segment
        .get(last_segment.len().saturating_sub(4)..)
        .map(|suffix| suffix.eq_ignore_ascii_case(".git"))
        .unwrap_or(false)
    {
        &last_segment[..last_segment.len().saturating_sub(4)]
    } else {
        last_segment
    };
    sanitize_folder_name(without_git)
}

pub fn sanitize_folder_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control()
            {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        DEFAULT_REPO_FOLDER_NAME.to_string()
    } else {
        sanitized
    }
}
