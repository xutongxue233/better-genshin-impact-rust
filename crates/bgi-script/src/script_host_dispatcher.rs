#[path = "script_host_dispatcher_commands.rs"]
mod commands;
#[path = "script_host_dispatcher_genshin.rs"]
mod genshin;
#[path = "script_host_dispatcher_host.rs"]
mod host;
#[path = "script_host_dispatcher_plans.rs"]
mod plans;

pub use commands::{DispatcherCommand, GenshinCommand};
pub use genshin::{genshin_command_to_task_input, GenshinHost};
pub use host::ScriptDispatcherHost;
pub use plans::{AutoPickExternalConfig, RealtimeTimerHostPlan, SoloTaskHostPlan};
