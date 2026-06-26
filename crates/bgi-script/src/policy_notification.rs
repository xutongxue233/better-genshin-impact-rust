use super::policy_error::{Result, ScriptHostPolicyError};
use serde::Serialize;
use std::collections::VecDeque;

pub(crate) const DEFAULT_NOTIFICATION_MAX_CHARS: usize = 500;
pub(crate) const DEFAULT_NOTIFICATION_WINDOW_MS: u64 = 60_000;
pub(crate) const DEFAULT_NOTIFICATION_MAX_PER_WINDOW: usize = 5;
pub(crate) const DEFAULT_FORBIDDEN_NOTIFICATION_PATTERNS: &[&str] =
    &["<script>", "http://", "https://"];

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
