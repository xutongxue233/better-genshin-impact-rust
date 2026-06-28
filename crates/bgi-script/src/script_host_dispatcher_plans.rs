use super::super::{Result, ScriptHostRuntimeError};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RealtimeTimerHostPlan {
    pub name: String,
    pub interval_ms: u64,
    pub config: Option<Value>,
    pub clears_existing_triggers: bool,
}

impl RealtimeTimerHostPlan {
    pub fn new(name: impl Into<String>, config: Option<Value>) -> Self {
        Self {
            name: name.into(),
            interval_ms: 50,
            config,
            clears_existing_triggers: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SoloTaskHostPlan {
    pub name: String,
    pub config: Option<Value>,
    pub uses_linked_cancellation: bool,
}

impl SoloTaskHostPlan {
    pub fn new(name: impl Into<String>, config: Option<Value>) -> Self {
        Self {
            name: name.into(),
            config,
            uses_linked_cancellation: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AutoPickExternalConfig {
    pub text_list: Vec<String>,
    pub force_interaction: bool,
}

impl AutoPickExternalConfig {
    pub fn from_value(value: Option<&Value>) -> Result<Self> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let Value::Object(map) = value else {
            return Err(ScriptHostRuntimeError::InvalidArgument {
                method: "AutoPickExternalConfig".to_string(),
                index: 0,
                expected: "object",
            });
        };
        let text_list = map
            .get("textList")
            .or_else(|| map.get("TextList"))
            .or_else(|| map.get("text_list"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let force_interaction = map
            .get("forceInteraction")
            .or_else(|| map.get("ForceInteraction"))
            .or_else(|| map.get("force_interaction"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        Ok(Self {
            text_list,
            force_interaction,
        })
    }

    pub fn to_legacy_config_value(&self) -> Value {
        serde_json::json!({
            "TextList": self.text_list,
            "ForceInteraction": self.force_interaction
        })
    }
}

impl From<RealtimeTimerHostPlan> for bgi_task::DispatcherTimerInput {
    fn from(value: RealtimeTimerHostPlan) -> Self {
        Self {
            name: value.name,
            interval_ms: value.interval_ms,
            config: value.config,
            clears_existing_triggers: value.clears_existing_triggers,
        }
    }
}

impl From<SoloTaskHostPlan> for bgi_task::DispatcherSoloTaskInput {
    fn from(value: SoloTaskHostPlan) -> Self {
        Self {
            name: value.name,
            config: value.config,
            uses_linked_cancellation: value.uses_linked_cancellation,
        }
    }
}
