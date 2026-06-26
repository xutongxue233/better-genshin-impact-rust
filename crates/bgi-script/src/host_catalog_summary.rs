use crate::host::{HostBindingDescriptor, HostBindingKind, HostPermission};
use std::collections::BTreeSet;

pub fn host_permissions(bindings: &[HostBindingDescriptor]) -> Vec<HostPermission> {
    bindings
        .iter()
        .flat_map(|binding| binding.permissions.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn host_binding_count_by_kind(
    bindings: &[HostBindingDescriptor],
    kind: HostBindingKind,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.kind == kind)
        .count()
}

pub fn host_member_count(bindings: &[HostBindingDescriptor]) -> usize {
    bindings.iter().map(|binding| binding.members.len()).sum()
}
