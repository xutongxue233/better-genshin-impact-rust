use super::Result;
use crate::policy::{NotificationRateLimiter, ScriptNotificationPolicy};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptLogRecord {
    pub level: ScriptLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptNotificationKind {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptNotificationRecord {
    pub kind: ScriptNotificationKind,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptNotificationDelivery {
    pub event_code: &'static str,
    pub result: &'static str,
    pub message: String,
    pub timestamp_ms: u64,
}

impl ScriptNotificationDelivery {
    pub fn from_record(record: &ScriptNotificationRecord) -> Self {
        let (event_code, result) = match record.kind {
            ScriptNotificationKind::Success => ("js.custom", "success"),
            ScriptNotificationKind::Error => ("js.error", "fail"),
        };
        Self {
            event_code,
            result,
            message: record.message.clone(),
            timestamp_ms: record.timestamp_ms,
        }
    }
}

pub trait ScriptNotificationSink {
    fn deliver(&mut self, delivery: ScriptNotificationDelivery) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NotificationDispatchMode {
    RecordOnly,
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotificationExecution {
    pub mode: NotificationDispatchMode,
    pub record: ScriptNotificationRecord,
    pub delivery: Option<ScriptNotificationDelivery>,
    pub dispatched: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RecordingNotificationSink {
    deliveries: Vec<ScriptNotificationDelivery>,
}

impl RecordingNotificationSink {
    pub fn deliveries(&self) -> &[ScriptNotificationDelivery] {
        &self.deliveries
    }
}

impl ScriptNotificationSink for RecordingNotificationSink {
    fn deliver(&mut self, delivery: ScriptNotificationDelivery) -> Result<()> {
        self.deliveries.push(delivery);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptLogHost {
    records: Vec<ScriptLogRecord>,
}

impl ScriptLogHost {
    pub fn debug(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Debug, message);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Info, message);
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Warn, message);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(ScriptLogLevel::Error, message);
    }

    pub fn records(&self) -> &[ScriptLogRecord] {
        &self.records
    }

    fn push(&mut self, level: ScriptLogLevel, message: impl Into<String>) {
        self.records.push(ScriptLogRecord {
            level,
            message: message.into(),
        });
    }
}

#[derive(Debug, Clone)]
pub struct ScriptNotificationHost {
    limiter: NotificationRateLimiter,
    records: Vec<ScriptNotificationRecord>,
}

impl ScriptNotificationHost {
    pub fn new(policy: ScriptNotificationPolicy) -> Self {
        Self {
            limiter: NotificationRateLimiter::new(policy),
            records: Vec::new(),
        }
    }

    pub fn send_at(&mut self, message: &str, now_ms: u64) -> Result<ScriptNotificationRecord> {
        self.push(ScriptNotificationKind::Success, message, now_ms)
    }

    pub fn send_to<S: ScriptNotificationSink>(
        &mut self,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        self.push_to(ScriptNotificationKind::Success, message, now_ms, sink)
    }

    pub fn error_at(&mut self, message: &str, now_ms: u64) -> Result<ScriptNotificationRecord> {
        self.push(ScriptNotificationKind::Error, message, now_ms)
    }

    pub fn error_to<S: ScriptNotificationSink>(
        &mut self,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        self.push_to(ScriptNotificationKind::Error, message, now_ms, sink)
    }

    pub fn records(&self) -> &[ScriptNotificationRecord] {
        &self.records
    }

    fn push(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
    ) -> Result<ScriptNotificationRecord> {
        self.validated_record(kind, message, now_ms)
    }

    fn push_to<S: ScriptNotificationSink>(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
        sink: &mut S,
    ) -> Result<ScriptNotificationDelivery> {
        let record = self.validated_record(kind, message, now_ms)?;
        let delivery = ScriptNotificationDelivery::from_record(&record);
        sink.deliver(delivery.clone())?;
        Ok(delivery)
    }

    fn validated_record(
        &mut self,
        kind: ScriptNotificationKind,
        message: &str,
        now_ms: u64,
    ) -> Result<ScriptNotificationRecord> {
        self.limiter.check_send_at(message, now_ms)?;
        let record = ScriptNotificationRecord {
            kind,
            message: message.to_string(),
            timestamp_ms: now_ms,
        };
        self.records.push(record.clone());
        Ok(record)
    }
}
