#[path = "policy_error.rs"]
mod policy_error;
#[path = "policy_file.rs"]
mod policy_file;
#[path = "policy_http.rs"]
mod policy_http;
#[path = "policy_notification.rs"]
mod policy_notification;
#[path = "policy_path.rs"]
mod policy_path;
#[path = "policy_summary.rs"]
mod policy_summary;

pub use policy_error::{Result, ScriptHostPolicyError};
pub use policy_file::ScriptFilePolicy;
pub use policy_http::{http_allowed_urls_hash, ScriptHttpPolicy};
pub use policy_notification::{NotificationRateLimiter, ScriptNotificationPolicy};
pub use policy_summary::{script_host_security_summary, ScriptHostSecuritySummary};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

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

        let max_per_window = policy.max_per_window;
        let window_ms = policy.window_ms;
        let mut limiter = NotificationRateLimiter::new(policy);
        for index in 0..max_per_window {
            limiter.check_send_at("done", index as u64).unwrap();
        }

        assert_eq!(
            limiter.check_send_at("done", 10).unwrap_err(),
            ScriptHostPolicyError::NotificationRateLimited
        );
        assert!(limiter.check_send_at("done", window_ms + 1).is_ok());
    }
}
