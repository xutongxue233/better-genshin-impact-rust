use crate::manifest::Manifest;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::{Component, Path, PathBuf};

const DEFAULT_ALLOWED_FILE_EXTENSIONS: &[&str] = &[
    ".txt", ".json", ".log", ".csv", ".xml", ".html", ".css", ".png", ".jpg", ".jpeg", ".bmp",
    ".tiff", ".webp",
];
const DEFAULT_IMAGE_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".bmp", ".tiff", ".webp"];
const DEFAULT_MAX_WRITE_BYTES: u64 = 999 * 1024 * 1024;
const DEFAULT_NOTIFICATION_MAX_CHARS: usize = 500;
const DEFAULT_NOTIFICATION_WINDOW_MS: u64 = 60_000;
const DEFAULT_NOTIFICATION_MAX_PER_WINDOW: usize = 5;
const DEFAULT_FORBIDDEN_NOTIFICATION_PATTERNS: &[&str] = &["<script>", "http://", "https://"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScriptHostPolicyError {
    #[error("file path is empty")]
    EmptyPath,
    #[error("file path contains invalid characters: {0}")]
    InvalidPathCharacter(String),
    #[error("file path escapes the script root: {path:?} is outside {root:?}")]
    PathTraversal { path: PathBuf, root: PathBuf },
    #[error("file extension is not allowed: {0}")]
    ExtensionNotAllowed(String),
    #[error("file content is too large: {actual_bytes} > {max_bytes}")]
    ContentTooLarge { actual_bytes: u64, max_bytes: u64 },
    #[error("HTTP requests are disabled for this script project")]
    HttpDisabled,
    #[error("manifest does not declare HTTP allow-list entries")]
    EmptyHttpAllowList,
    #[error("URL is not allowed by the script manifest: {0}")]
    HttpUrlDenied(String),
    #[error("script notifications are disabled")]
    NotificationDisabled,
    #[error("notification content is too long: {actual_chars} > {max_chars}")]
    NotificationTooLong {
        actual_chars: usize,
        max_chars: usize,
    },
    #[error("notification content contains a forbidden pattern: {0}")]
    NotificationForbiddenPattern(String),
    #[error("notification rate limit exceeded")]
    NotificationRateLimited,
}

pub type Result<T> = std::result::Result<T, ScriptHostPolicyError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptFilePolicy {
    pub root: PathBuf,
    pub allowed_extensions: Vec<&'static str>,
    pub image_extensions: Vec<&'static str>,
    pub max_write_bytes: u64,
}

impl ScriptFilePolicy {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            allowed_extensions: DEFAULT_ALLOWED_FILE_EXTENSIONS.to_vec(),
            image_extensions: DEFAULT_IMAGE_EXTENSIONS.to_vec(),
            max_write_bytes: DEFAULT_MAX_WRITE_BYTES,
        }
    }

    pub fn normalize_path(&self, path: &str) -> Result<PathBuf> {
        if path.trim().is_empty() {
            return Err(ScriptHostPolicyError::EmptyPath);
        }

        if let Some(file_name) = Path::new(path).file_name().and_then(|name| name.to_str()) {
            if contains_invalid_windows_file_name_char(file_name) {
                return Err(ScriptHostPolicyError::InvalidPathCharacter(
                    file_name.to_string(),
                ));
            }
        }

        let root = absolute_lexical_path(&self.root);
        let requested = Path::new(path);
        let candidate = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            root.join(requested)
        };
        let normalized = lexical_normalize(&candidate);

        if !path_starts_with(&normalized, &root) {
            return Err(ScriptHostPolicyError::PathTraversal {
                path: normalized,
                root,
            });
        }

        Ok(normalized)
    }

    pub fn validate_text_write(&self, path: &str, content: &str) -> Result<PathBuf> {
        let normalized = self.normalize_path(path)?;
        self.validate_write_extension(&normalized)?;
        let actual_bytes = content.len() as u64;
        if actual_bytes > self.max_write_bytes {
            return Err(ScriptHostPolicyError::ContentTooLarge {
                actual_bytes,
                max_bytes: self.max_write_bytes,
            });
        }
        Ok(normalized)
    }

    pub fn normalize_image_write_target(&self, path: &str) -> Result<PathBuf> {
        let path = ensure_image_extension(path, &self.image_extensions);
        let normalized = self.normalize_path(&path)?;
        self.validate_image_extension(&normalized)?;
        Ok(normalized)
    }

    pub fn validate_write_extension(&self, path: &Path) -> Result<()> {
        let extension = normalized_extension(path);
        if self
            .allowed_extensions
            .iter()
            .any(|allowed| *allowed == extension)
        {
            Ok(())
        } else {
            Err(ScriptHostPolicyError::ExtensionNotAllowed(extension))
        }
    }

    pub fn validate_image_extension(&self, path: &Path) -> Result<()> {
        let extension = normalized_extension(path);
        if self
            .image_extensions
            .iter()
            .any(|allowed| *allowed == extension)
        {
            Ok(())
        } else {
            Err(ScriptHostPolicyError::ExtensionNotAllowed(extension))
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptNotificationPolicy {
    pub app_enabled: bool,
    pub project_enabled: bool,
    pub max_chars: usize,
    pub window_ms: u64,
    pub max_per_window: usize,
    pub forbidden_patterns: Vec<&'static str>,
}

impl ScriptNotificationPolicy {
    pub fn new(app_enabled: bool, project_enabled: bool) -> Self {
        Self {
            app_enabled,
            project_enabled,
            max_chars: DEFAULT_NOTIFICATION_MAX_CHARS,
            window_ms: DEFAULT_NOTIFICATION_WINDOW_MS,
            max_per_window: DEFAULT_NOTIFICATION_MAX_PER_WINDOW,
            forbidden_patterns: DEFAULT_FORBIDDEN_NOTIFICATION_PATTERNS.to_vec(),
        }
    }

    pub fn validate_message(&self, message: &str) -> Result<()> {
        if !self.app_enabled || !self.project_enabled {
            return Err(ScriptHostPolicyError::NotificationDisabled);
        }

        let actual_chars = message.chars().count();
        if actual_chars > self.max_chars {
            return Err(ScriptHostPolicyError::NotificationTooLong {
                actual_chars,
                max_chars: self.max_chars,
            });
        }

        let lower = message.to_ascii_lowercase();
        if let Some(pattern) = self
            .forbidden_patterns
            .iter()
            .find(|pattern| lower.contains(&pattern.to_ascii_lowercase()))
        {
            return Err(ScriptHostPolicyError::NotificationForbiddenPattern(
                (*pattern).to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NotificationRateLimiter {
    policy: ScriptNotificationPolicy,
    call_records: VecDeque<u64>,
}

impl NotificationRateLimiter {
    pub fn new(policy: ScriptNotificationPolicy) -> Self {
        Self {
            policy,
            call_records: VecDeque::new(),
        }
    }

    pub fn check_send_at(&mut self, message: &str, now_ms: u64) -> Result<()> {
        self.policy.validate_message(message)?;
        self.retain_recent(now_ms);
        if self.call_records.len() >= self.policy.max_per_window {
            return Err(ScriptHostPolicyError::NotificationRateLimited);
        }
        self.call_records.push_back(now_ms);
        Ok(())
    }

    fn retain_recent(&mut self, now_ms: u64) {
        while let Some(recorded_at) = self.call_records.front().copied() {
            if now_ms.saturating_sub(recorded_at) > self.policy.window_ms {
                self.call_records.pop_front();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptHostSecuritySummary {
    pub file_allowed_extensions: Vec<&'static str>,
    pub image_extensions: Vec<&'static str>,
    pub file_max_write_bytes: u64,
    pub http_uses_manifest_wildcards: bool,
    pub notification_max_chars: usize,
    pub notification_window_ms: u64,
    pub notification_max_per_window: usize,
    pub forbidden_notification_patterns: Vec<&'static str>,
}

pub fn script_host_security_summary() -> ScriptHostSecuritySummary {
    ScriptHostSecuritySummary {
        file_allowed_extensions: DEFAULT_ALLOWED_FILE_EXTENSIONS.to_vec(),
        image_extensions: DEFAULT_IMAGE_EXTENSIONS.to_vec(),
        file_max_write_bytes: DEFAULT_MAX_WRITE_BYTES,
        http_uses_manifest_wildcards: true,
        notification_max_chars: DEFAULT_NOTIFICATION_MAX_CHARS,
        notification_window_ms: DEFAULT_NOTIFICATION_WINDOW_MS,
        notification_max_per_window: DEFAULT_NOTIFICATION_MAX_PER_WINDOW,
        forbidden_notification_patterns: DEFAULT_FORBIDDEN_NOTIFICATION_PATTERNS.to_vec(),
    }
}

fn contains_invalid_windows_file_name_char(value: &str) -> bool {
    value
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*') || ch.is_control())
}

fn absolute_lexical_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        lexical_normalize(path)
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        lexical_normalize(&cwd.join(path))
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path_components = comparable_components(path);
    let root_components = comparable_components(root);
    path_components.len() >= root_components.len()
        && path_components
            .iter()
            .zip(root_components.iter())
            .all(|(left, right)| left == right)
}

fn comparable_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| {
            let value = component.as_os_str().to_string_lossy().to_string();
            if cfg!(windows) {
                value.to_ascii_lowercase()
            } else {
                value
            }
        })
        .collect()
}

fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{}", extension.to_ascii_lowercase()))
        .unwrap_or_default()
}

fn ensure_image_extension(path: &str, image_extensions: &[&str]) -> String {
    let extension = normalized_extension(Path::new(path));
    if image_extensions.iter().any(|allowed| *allowed == extension) {
        path.to_string()
    } else {
        format!("{path}.png")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_policy_blocks_parent_traversal() {
        let policy = ScriptFilePolicy::new("scripts/example");
        let err = policy.normalize_path("../outside.json").unwrap_err();

        assert!(matches!(err, ScriptHostPolicyError::PathTraversal { .. }));
    }

    #[test]
    fn file_policy_allows_legacy_extensions_and_appends_image_suffix() {
        let policy = ScriptFilePolicy::new("scripts/example");
        let text_path = policy
            .validate_text_write("data/result.json", "{}")
            .unwrap();
        let image_path = policy.normalize_image_write_target("shots/frame").unwrap();

        assert_eq!(text_path.file_name().unwrap(), "result.json");
        assert_eq!(image_path.file_name().unwrap(), "frame.png");
    }

    #[test]
    fn http_policy_uses_manifest_wildcards_and_hash_gate() {
        let manifest = Manifest {
            http_allowed_urls: vec!["https://example.com/api/*".to_string()],
            ..Manifest::default()
        };
        let hash = http_allowed_urls_hash(&manifest.http_allowed_urls);
        let policy = ScriptHttpPolicy::enabled_by_project_hash(&manifest, Some(&hash));

        assert!(policy.check_url("https://example.com/api/items").is_ok());
        assert!(matches!(
            policy.check_url("https://example.org/api/items"),
            Err(ScriptHostPolicyError::HttpUrlDenied(_))
        ));
    }

    #[test]
    fn notification_policy_blocks_forbidden_content_and_rate_limits() {
        let policy = ScriptNotificationPolicy::new(true, true);
        assert!(matches!(
            policy.validate_message("see https://example.com"),
            Err(ScriptHostPolicyError::NotificationForbiddenPattern(_))
        ));

        let mut limiter = NotificationRateLimiter::new(policy);
        for index in 0..DEFAULT_NOTIFICATION_MAX_PER_WINDOW {
            limiter.check_send_at("done", index as u64).unwrap();
        }

        assert_eq!(
            limiter.check_send_at("done", 10).unwrap_err(),
            ScriptHostPolicyError::NotificationRateLimited
        );
        assert!(limiter
            .check_send_at("done", DEFAULT_NOTIFICATION_WINDOW_MS + 1)
            .is_ok());
    }
}
