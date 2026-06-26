use serde::Serialize;

#[path = "host_catalog.rs"]
mod host_catalog;

pub use host_catalog::{
    host_binding_count_by_kind, host_bindings, host_member_count, host_permissions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum HostPermission {
    BackgroundInput,
    Capture,
    Filesystem,
    GameAutomation,
    GameState,
    Input,
    Logging,
    Network,
    Notification,
    Overlay,
    Scheduler,
    Settings,
    Time,
    Vision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HostBindingKind {
    GlobalFunctionSet,
    Object,
    Type,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HostBindingPortState {
    MetadataOnly,
    RustModelReady,
    NativePending,
    SecurityReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostBindingDescriptor {
    pub name: &'static str,
    pub kind: HostBindingKind,
    pub legacy_type: &'static str,
    pub members: &'static [&'static str],
    pub permissions: &'static [HostPermission],
    pub port_state: HostBindingPortState,
    pub notes: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn host_binding_names_are_unique() {
        let bindings = host_bindings();
        let names = bindings
            .iter()
            .map(|binding| binding.name)
            .collect::<BTreeSet<_>>();

        assert_eq!(names.len(), bindings.len());
    }

    #[test]
    fn global_methods_include_legacy_input_surface() {
        let global = host_bindings()
            .into_iter()
            .find(|binding| binding.name == "global")
            .unwrap();

        assert!(global.members.contains(&"keyDown"));
        assert!(global.members.contains(&"captureGameRegion"));
        assert!(global.permissions.contains(&HostPermission::Input));
    }
}
