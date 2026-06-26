use super::super::{
    CombatChargeExecutionVariant, CombatCommandActionPlan, CombatSkillExecutionVariant,
};
use super::combat_action_events;
use super::keys::combat_virtual_key_events;
use crate::{Result, TaskError};
use bgi_core::{GenshinAction, KeyBindingsConfig};
use bgi_input::{InputEvent, KeyActionType};

pub(crate) fn default_input_events_for_combat_action(
    action: &CombatCommandActionPlan,
) -> Result<Vec<InputEvent>> {
    let bindings = KeyBindingsConfig::default();
    let mut events = Vec::new();
    match action {
        CombatCommandActionPlan::Skill { variant, .. } => match variant {
            CombatSkillExecutionVariant::Tap => events.extend(combat_action_events(
                &bindings,
                GenshinAction::ElementalSkill,
                KeyActionType::KeyPress,
            )?),
            CombatSkillExecutionVariant::GenericHold => events.extend(combat_action_events(
                &bindings,
                GenshinAction::ElementalSkill,
                KeyActionType::Hold,
            )?),
            CombatSkillExecutionVariant::NahidaCameraSweepHold => {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyDown,
                )?);
                events.push(InputEvent::Delay { milliseconds: 300 });
                for _ in 0..10 {
                    events.push(InputEvent::MouseMoveRelative { dx: 1000, dy: 0 });
                    events.push(InputEvent::Delay { milliseconds: 50 });
                }
                events.push(InputEvent::Delay { milliseconds: 300 });
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyUp,
                )?);
            }
            CombatSkillExecutionVariant::CandaceLongHold => {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyDown,
                )?);
                events.push(InputEvent::Delay {
                    milliseconds: 3_000,
                });
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::ElementalSkill,
                    KeyActionType::KeyUp,
                )?);
            }
        },
        CombatCommandActionPlan::Burst { .. } => events.extend(combat_action_events(
            &bindings,
            GenshinAction::ElementalBurst,
            KeyActionType::KeyPress,
        )?),
        CombatCommandActionPlan::Attack {
            click_interval_ms,
            repeat_count,
            ..
        } => {
            for _ in 0..*repeat_count {
                events.extend(combat_action_events(
                    &bindings,
                    GenshinAction::NormalAttack,
                    KeyActionType::KeyPress,
                )?);
                events.push(InputEvent::Delay {
                    milliseconds: *click_interval_ms,
                });
            }
        }
        CombatCommandActionPlan::Charge {
            duration_ms,
            variant: CombatChargeExecutionVariant::GenericHold,
        } => {
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::NormalAttack,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::NormalAttack,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Charge { .. } => {}
        CombatCommandActionPlan::Walk {
            direction,
            duration_ms,
        } => {
            let action = walk_direction_action(direction)?;
            events.extend(combat_action_events(
                &bindings,
                action,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                action,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Wait { duration_ms } => events.push(InputEvent::Delay {
            milliseconds: *duration_ms,
        }),
        CombatCommandActionPlan::Ready {
            initial_delay_ms, ..
        } => events.push(InputEvent::Delay {
            milliseconds: *initial_delay_ms,
        }),
        CombatCommandActionPlan::Check { .. } => {}
        CombatCommandActionPlan::Dash { duration_ms } => {
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::SprintMouse,
                KeyActionType::KeyDown,
            )?);
            events.push(InputEvent::Delay {
                milliseconds: *duration_ms,
            });
            events.extend(combat_action_events(
                &bindings,
                GenshinAction::SprintMouse,
                KeyActionType::KeyUp,
            )?);
        }
        CombatCommandActionPlan::Jump => events.extend(combat_action_events(
            &bindings,
            GenshinAction::Jump,
            KeyActionType::KeyPress,
        )?),
        CombatCommandActionPlan::MouseDown { button } => {
            events.push(InputEvent::MouseButtonDown {
                button: button.button,
            });
        }
        CombatCommandActionPlan::MouseUp { button } => {
            events.push(InputEvent::MouseButtonUp {
                button: button.button,
            });
        }
        CombatCommandActionPlan::Click { button } => {
            events.push(InputEvent::MouseButtonDown {
                button: button.button,
            });
            events.push(InputEvent::MouseButtonUp {
                button: button.button,
            });
        }
        CombatCommandActionPlan::MoveBy { x, y } => {
            events.push(InputEvent::MouseMoveRelative { dx: *x, dy: *y });
        }
        CombatCommandActionPlan::KeyDown { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyDown)?);
        }
        CombatCommandActionPlan::KeyUp { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyUp)?);
        }
        CombatCommandActionPlan::KeyPress { key } => {
            events.extend(combat_virtual_key_events(key, KeyActionType::KeyPress)?);
        }
        CombatCommandActionPlan::Scroll { clicks } => {
            events.push(InputEvent::MouseWheel {
                amount: *clicks * 120,
                horizontal: false,
            });
        }
    }
    Ok(events)
}

fn walk_direction_action(direction: &str) -> Result<GenshinAction> {
    match direction.trim().to_ascii_lowercase().as_str() {
        "w" => Ok(GenshinAction::MoveForward),
        "s" => Ok(GenshinAction::MoveBackward),
        "a" => Ok(GenshinAction::MoveLeft),
        "d" => Ok(GenshinAction::MoveRight),
        other => Err(TaskError::CombatStrategy(format!(
            "unsupported walk direction: {other}"
        ))),
    }
}
