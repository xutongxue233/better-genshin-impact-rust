use std::path::PathBuf;

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
