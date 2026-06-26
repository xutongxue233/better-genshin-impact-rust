use super::{
    script_engines, ScriptCancellationPolicy, ScriptEngineDescriptor, ScriptRuntimeState,
    ScriptScheduleKind,
};
use crate::group::ScriptProjectType;
use crate::host::{
    host_binding_count_by_kind, host_bindings, host_member_count, host_permissions,
    HostBindingKind, HostPermission,
};
use crate::project::ScriptProjectLoaderSummary;
use crate::settings::{script_settings_summary, ScriptSettingsSummary};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptRuntimeSummary {
    pub state: ScriptRuntimeState,
    pub engines: Vec<ScriptEngineDescriptor>,
    pub host_binding_count: usize,
    pub host_member_count: usize,
    pub host_object_count: usize,
    pub host_type_count: usize,
    pub permissions: Vec<HostPermission>,
    pub default_cancellation_policy: ScriptCancellationPolicy,
    pub supported_project_types: Vec<ScriptProjectType>,
    pub schedule_kinds: Vec<ScriptScheduleKind>,
    pub project_loader: ScriptProjectLoaderSummary,
    pub settings: ScriptSettingsSummary,
}

pub fn script_runtime_summary() -> ScriptRuntimeSummary {
    let bindings = host_bindings();
    ScriptRuntimeSummary {
        state: ScriptRuntimeState::Stopped,
        engines: script_engines(),
        host_binding_count: bindings.len(),
        host_member_count: host_member_count(&bindings),
        host_object_count: host_binding_count_by_kind(&bindings, HostBindingKind::Object),
        host_type_count: host_binding_count_by_kind(&bindings, HostBindingKind::Type),
        permissions: host_permissions(&bindings),
        default_cancellation_policy: ScriptCancellationPolicy::default(),
        supported_project_types: vec![
            ScriptProjectType::Javascript,
            ScriptProjectType::KeyMouse,
            ScriptProjectType::Pathing,
            ScriptProjectType::Shell,
        ],
        schedule_kinds: vec![
            ScriptScheduleKind::Daily,
            ScriptScheduleKind::EveryTwoDays,
            ScriptScheduleKind::Weekday,
            ScriptScheduleKind::Cron,
            ScriptScheduleKind::Manual,
        ],
        project_loader: ScriptProjectLoaderSummary::default(),
        settings: script_settings_summary(),
    }
}
