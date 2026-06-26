use crate::config::NotificationConfig;

#[path = "notification_delivery.rs"]
mod notification_delivery;
#[path = "notification_dispatch.rs"]
mod notification_dispatch;
#[path = "notification_events.rs"]
mod notification_events;
#[path = "notification_model.rs"]
mod notification_model;
#[path = "notification_plans.rs"]
mod notification_plans;

use notification_plans::non_empty;

pub use notification_dispatch::*;
pub use notification_events::*;
pub use notification_model::*;
pub use notification_plans::*;

#[cfg(test)]
use notification_delivery::*;
#[cfg(test)]
use notification_plans::provider;
#[cfg(test)]
use serde_json::{json, Value};

#[cfg(test)]
#[path = "notification_tests.rs"]
mod notification_tests;
