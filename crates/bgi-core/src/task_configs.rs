#[path = "task_configs_activity.rs"]
mod task_configs_activity;
#[path = "task_configs_boss.rs"]
mod task_configs_boss;
#[path = "task_configs_combat.rs"]
mod task_configs_combat;
#[path = "task_configs_common.rs"]
mod task_configs_common;
#[path = "task_configs_domain.rs"]
mod task_configs_domain;
#[path = "task_configs_ley_line.rs"]
mod task_configs_ley_line;
#[path = "task_configs_misc.rs"]
mod task_configs_misc;

pub use task_configs_activity::*;
pub use task_configs_boss::*;
pub use task_configs_combat::*;
pub use task_configs_common::*;
pub use task_configs_domain::*;
pub use task_configs_ley_line::*;
pub use task_configs_misc::*;
