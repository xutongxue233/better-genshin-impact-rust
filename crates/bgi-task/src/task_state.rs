use crate::RunnableTrigger;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskProgress {
    pub name: String,
    pub current_step: Option<String>,
    pub completed: u32,
    pub total: Option<u32>,
    pub message: Option<String>,
}

impl TaskProgress {
    pub fn percentage(&self) -> Option<u8> {
        let total = self.total?;
        if total == 0 {
            return None;
        }
        Some(((self.completed.min(total) * 100) / total) as u8)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRegistry {
    trigger_enabled: BTreeMap<String, bool>,
}

impl TaskRegistry {
    pub fn from_triggers(triggers: &[RunnableTrigger]) -> Self {
        Self {
            trigger_enabled: triggers
                .iter()
                .map(|trigger| (trigger.descriptor.key.to_string(), trigger.enabled))
                .collect(),
        }
    }

    pub fn is_enabled(&self, key: &str) -> Option<bool> {
        self.trigger_enabled
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(key))
            .map(|(_, enabled)| *enabled)
    }
}
