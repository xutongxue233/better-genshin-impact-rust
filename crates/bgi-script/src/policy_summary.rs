use super::policy_file::{
    DEFAULT_ALLOWED_FILE_EXTENSIONS, DEFAULT_IMAGE_EXTENSIONS, DEFAULT_MAX_WRITE_BYTES,
};
use super::policy_notification::{
    DEFAULT_FORBIDDEN_NOTIFICATION_PATTERNS, DEFAULT_NOTIFICATION_MAX_CHARS,
    DEFAULT_NOTIFICATION_MAX_PER_WINDOW, DEFAULT_NOTIFICATION_WINDOW_MS,
};
use serde::Serialize;

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
