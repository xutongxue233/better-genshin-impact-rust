use super::script::parse_f64_arg;
use super::*;
use bgi_input::input_events_for_action;

#[path = "auto_fight_input_events_actions.rs"]
mod actions;
#[path = "auto_fight_input_events_keys.rs"]
mod keys;

pub(super) use actions::default_input_events_for_combat_action;
pub(super) use keys::{combat_mouse_button_plan, combat_virtual_key_plan};

pub(super) fn combat_action_events(
    bindings: &KeyBindingsConfig,
    action: GenshinAction,
    action_type: KeyActionType,
) -> Result<Vec<InputEvent>> {
    input_events_for_action(bindings, action, action_type)
        .map_err(|error| TaskError::CombatStrategy(error.to_string()))
}

pub(super) fn combat_attack_repeat_count(duration_ms: u64) -> u32 {
    (duration_ms / COMBAT_ATTACK_INTERVAL_MILLISECONDS + 1) as u32
}

pub(super) fn optional_duration_ms(args: &[String], index: usize, default_ms: u64) -> Result<u64> {
    match args.get(index) {
        Some(value) if !value.trim().is_empty() => duration_ms_from_seconds(value, "duration"),
        _ => Ok(default_ms),
    }
}

pub(super) fn duration_ms_from_seconds(value: &str, label: &str) -> Result<u64> {
    let seconds = parse_f64_arg(value, label)?;
    if seconds < 0.0 {
        return Err(TaskError::CombatStrategy(format!(
            "{label} must be non-negative"
        )));
    }
    Ok((seconds * 1000.0) as u64)
}
