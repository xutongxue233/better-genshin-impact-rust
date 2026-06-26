use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDispatchReport {
    pub dispatched_events: usize,
    pub total_events: usize,
    pub cancelled: bool,
}

impl InputDispatchReport {
    pub fn completed(total_events: usize) -> Self {
        Self {
            dispatched_events: total_events,
            total_events,
            cancelled: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputCancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl InputCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.cancelled
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}
