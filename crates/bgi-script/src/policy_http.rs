use super::policy_error::{Result, ScriptHostPolicyError};
use crate::manifest::Manifest;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptHttpPolicy {
    pub enabled: bool,
    pub allowed_urls: Vec<String>,
}

impl ScriptHttpPolicy {
    pub fn new(enabled: bool, allowed_urls: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            enabled,
            allowed_urls: allowed_urls.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_manifest(enabled: bool, manifest: &Manifest) -> Self {
        Self::new(enabled, manifest.http_allowed_urls.clone())
    }

    pub fn enabled_by_project_hash(manifest: &Manifest, project_hash: Option<&str>) -> Self {
        let enabled = project_hash
            .map(|hash| hash == http_allowed_urls_hash(&manifest.http_allowed_urls))
            .unwrap_or(false);
        Self::from_manifest(enabled, manifest)
    }

    pub fn check_url(&self, url: &str) -> Result<()> {
        if !self.enabled {
            return Err(ScriptHostPolicyError::HttpDisabled);
        }
        if self.allowed_urls.is_empty() {
            return Err(ScriptHostPolicyError::EmptyHttpAllowList);
        }
        if self
            .allowed_urls
            .iter()
            .any(|allowed| wildcard_match(allowed, url))
        {
            Ok(())
        } else {
            Err(ScriptHostPolicyError::HttpUrlDenied(url.to_string()))
        }
    }
}

pub fn http_allowed_urls_hash(urls: &[String]) -> String {
    urls.join("|")
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut pattern_index = 0;
    let mut text_index = 0;
    let mut last_star = None;
    let mut match_after_star = 0;

    while text_index < text.len() {
        if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            last_star = Some(pattern_index);
            pattern_index += 1;
            match_after_star = text_index;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == text[text_index] {
            pattern_index += 1;
            text_index += 1;
        } else if let Some(star_index) = last_star {
            pattern_index = star_index + 1;
            match_after_star += 1;
            text_index = match_after_star;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}
