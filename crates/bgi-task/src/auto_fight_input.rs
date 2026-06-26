use super::input_events::*;
use super::script::parse_i32_arg;
use super::*;

pub(super) fn refresh_team_playback_execution(execution: &mut CombatTeamPlaybackExecution) {
    execution.input_events.clear();
    execution.blocked_command_index = None;
    execution.blocked_requirements.clear();
    for command in &mut execution.planned_commands {
        command.executable = command.pending_context.is_empty();
        command.input_events.clear();
        if command.executable {
            command
                .input_events
                .extend(command.switch_events.iter().copied());
            command
                .input_events
                .extend(command.action_events.iter().copied());
            execution
                .input_events
                .extend(command.input_events.iter().copied());
        } else if execution.blocked_command_index.is_none() {
            execution.blocked_command_index = Some(command.command_index);
            execution.blocked_requirements = command.pending_context.clone();
        }
    }
    execution.playable_commands = execution
        .planned_commands
        .iter()
        .filter(|command| command.executable)
        .count();
    execution.dispatch_ready = !execution.planned_commands.is_empty()
        && execution.blocked_command_index.is_none()
        && execution.playable_commands == execution.planned_commands.len()
        && !execution.input_events.is_empty();
}

pub(super) fn combat_avatar_switch_events(index: usize) -> Result<Vec<InputEvent>> {
    let action = match index {
        1 => GenshinAction::SwitchMember1,
        2 => GenshinAction::SwitchMember2,
        3 => GenshinAction::SwitchMember3,
        4 => GenshinAction::SwitchMember4,
        5 => GenshinAction::SwitchMember5,
        _ => {
            return Err(TaskError::CombatStrategy(format!(
                "combat avatar switch index {index} is outside the supported party range"
            )))
        }
    };
    let bindings = KeyBindingsConfig::default();
    let mut events = combat_action_events(&bindings, GenshinAction::Drop, KeyActionType::KeyPress)?;
    events.extend(combat_action_events(
        &bindings,
        action,
        KeyActionType::KeyPress,
    )?);
    events.push(InputEvent::Delay {
        milliseconds: COMBAT_AVATAR_SWITCH_SETTLE_MILLISECONDS,
    });
    Ok(events)
}

pub(super) fn combat_command_context_requirements(
    switch_policy: CombatAvatarSwitchPolicy,
    action: &CombatCommandActionPlan,
    default_input_events_empty: bool,
) -> Vec<CombatExecutionContextRequirement> {
    let mut requirements = Vec::new();
    if !matches!(
        switch_policy,
        CombatAvatarSwitchPolicy::CurrentAvatar | CombatAvatarSwitchPolicy::NoSwitch
    ) {
        requirements.push(CombatExecutionContextRequirement::AvatarSelection);
    }
    match action {
        CombatCommandActionPlan::Skill {
            cooldown_policy,
            variant,
            ..
        } => {
            if !matches!(cooldown_policy, CombatSkillCooldownPolicy::None) {
                requirements.push(CombatExecutionContextRequirement::SkillCooldown);
            }
            if matches!(
                variant,
                CombatSkillExecutionVariant::NahidaCameraSweepHold
                    | CombatSkillExecutionVariant::CandaceLongHold
            ) {
                requirements
                    .push(CombatExecutionContextRequirement::CharacterSpecificCameraControl);
            }
        }
        CombatCommandActionPlan::Burst { .. } => {
            requirements.push(CombatExecutionContextRequirement::BurstReadiness);
        }
        CombatCommandActionPlan::Ready { .. } => {
            requirements.push(CombatExecutionContextRequirement::ReadyStateDetection);
        }
        CombatCommandActionPlan::Check {
            handled_by_fight_loop,
        } => {
            if *handled_by_fight_loop {
                requirements.push(CombatExecutionContextRequirement::FightLoopFinishDetection);
            }
        }
        CombatCommandActionPlan::Charge { variant, .. }
            if !matches!(variant, CombatChargeExecutionVariant::GenericHold) =>
        {
            requirements.push(CombatExecutionContextRequirement::CharacterSpecificCameraControl);
        }
        _ => {}
    }
    if default_input_events_empty
        && !matches!(
            action,
            CombatCommandActionPlan::Check { .. } | CombatCommandActionPlan::Ready { .. }
        )
    {
        requirements.push(CombatExecutionContextRequirement::InputEvents);
    }
    requirements.sort();
    requirements.dedup();
    requirements
}

pub(super) fn combat_avatar_switch_policy(
    command: &CombatCommandPlan,
    previous: Option<&CombatCommandPlan>,
) -> CombatAvatarSwitchPolicy {
    if command.avatar == CURRENT_COMBAT_AVATAR_NAME {
        return CombatAvatarSwitchPolicy::CurrentAvatar;
    }
    if previous
        .map(|previous| previous.avatar != command.avatar)
        .unwrap_or(false)
    {
        return CombatAvatarSwitchPolicy::SwitchOnAvatarChange;
    }
    if combat_command_skips_avatar_switch(command.method) {
        CombatAvatarSwitchPolicy::NoSwitch
    } else {
        CombatAvatarSwitchPolicy::EnsureSelectedBeforeAction
    }
}

pub(super) fn combat_command_skips_avatar_switch(method: CombatCommandMethod) -> bool {
    matches!(
        method,
        CombatCommandMethod::Wait
            | CombatCommandMethod::Ready
            | CombatCommandMethod::MouseDown
            | CombatCommandMethod::MouseUp
            | CombatCommandMethod::Click
            | CombatCommandMethod::MoveBy
            | CombatCommandMethod::KeyDown
            | CombatCommandMethod::KeyUp
            | CombatCommandMethod::KeyPress
            | CombatCommandMethod::Scroll
    )
}

pub(super) fn combat_command_action_plan(
    command: &CombatCommandPlan,
) -> Result<CombatCommandActionPlan> {
    match command.method {
        CombatCommandMethod::Skill => {
            let options: Vec<String> = command.args.iter().map(|arg| arg.to_lowercase()).collect();
            let hold = options.iter().any(|arg| arg == "hold");
            let cooldown_policy = if options.iter().any(|arg| arg == "fast") {
                CombatSkillCooldownPolicy::FastSkipIfCoolingDown
            } else if options.iter().any(|arg| arg == "wait") {
                CombatSkillCooldownPolicy::WaitUntilReady
            } else {
                CombatSkillCooldownPolicy::None
            };
            let variant = if !hold {
                CombatSkillExecutionVariant::Tap
            } else {
                match command.avatar.as_str() {
                    "纳西妲" => CombatSkillExecutionVariant::NahidaCameraSweepHold,
                    "坎蒂丝" => CombatSkillExecutionVariant::CandaceLongHold,
                    _ => CombatSkillExecutionVariant::GenericHold,
                }
            };
            Ok(CombatCommandActionPlan::Skill {
                hold,
                variant,
                cooldown_policy,
                options,
            })
        }
        CombatCommandMethod::Burst => Ok(CombatCommandActionPlan::Burst {
            requires_readiness_check: true,
        }),
        CombatCommandMethod::Attack => {
            let duration_ms = optional_duration_ms(&command.args, 0, 0)?;
            Ok(CombatCommandActionPlan::Attack {
                duration_ms,
                click_interval_ms: COMBAT_ATTACK_INTERVAL_MILLISECONDS,
                repeat_count: combat_attack_repeat_count(duration_ms),
            })
        }
        CombatCommandMethod::Charge => {
            let duration_ms =
                optional_duration_ms(&command.args, 0, COMBAT_DEFAULT_CHARGE_MILLISECONDS)?;
            let variant = match command.avatar.as_str() {
                "那维莱特" => CombatChargeExecutionVariant::NeuvilletteCameraSweep,
                "恰斯卡" => CombatChargeExecutionVariant::ChascaCameraSweep,
                _ => CombatChargeExecutionVariant::GenericHold,
            };
            Ok(CombatCommandActionPlan::Charge {
                duration_ms,
                variant,
            })
        }
        CombatCommandMethod::Walk => Ok(CombatCommandActionPlan::Walk {
            direction: command.args[0].trim().to_ascii_lowercase(),
            duration_ms: duration_ms_from_seconds(&command.args[1], "walk duration")?,
        }),
        CombatCommandMethod::W
        | CombatCommandMethod::A
        | CombatCommandMethod::S
        | CombatCommandMethod::D => Ok(CombatCommandActionPlan::Walk {
            direction: match command.method {
                CombatCommandMethod::W => "w",
                CombatCommandMethod::A => "a",
                CombatCommandMethod::S => "s",
                CombatCommandMethod::D => "d",
                _ => unreachable!(),
            }
            .to_string(),
            duration_ms: duration_ms_from_seconds(&command.args[0], "walk duration")?,
        }),
        CombatCommandMethod::Wait => {
            let Some(seconds) = command.args.first() else {
                return Err(TaskError::CombatStrategy(
                    "wait command requires one duration argument".to_string(),
                ));
            };
            Ok(CombatCommandActionPlan::Wait {
                duration_ms: duration_ms_from_seconds(seconds, "wait duration")?,
            })
        }
        CombatCommandMethod::Ready => Ok(CombatCommandActionPlan::Ready {
            initial_delay_ms: COMBAT_READY_INITIAL_DELAY_MILLISECONDS,
            poll_count: COMBAT_READY_POLL_COUNT,
            poll_interval_ms: COMBAT_READY_POLL_INTERVAL_MILLISECONDS,
        }),
        CombatCommandMethod::Check => Ok(CombatCommandActionPlan::Check {
            handled_by_fight_loop: true,
        }),
        CombatCommandMethod::Dash => Ok(CombatCommandActionPlan::Dash {
            duration_ms: optional_duration_ms(&command.args, 0, COMBAT_DEFAULT_DASH_MILLISECONDS)?,
        }),
        CombatCommandMethod::Jump => Ok(CombatCommandActionPlan::Jump),
        CombatCommandMethod::MouseDown => Ok(CombatCommandActionPlan::MouseDown {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::MouseUp => Ok(CombatCommandActionPlan::MouseUp {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::Click => Ok(CombatCommandActionPlan::Click {
            button: combat_mouse_button_plan(command.args.first().map(String::as_str))?,
        }),
        CombatCommandMethod::MoveBy => Ok(CombatCommandActionPlan::MoveBy {
            x: parse_i32_arg(&command.args[0], "moveby x")?,
            y: parse_i32_arg(&command.args[1], "moveby y")?,
        }),
        CombatCommandMethod::KeyDown => Ok(CombatCommandActionPlan::KeyDown {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::KeyUp => Ok(CombatCommandActionPlan::KeyUp {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::KeyPress => Ok(CombatCommandActionPlan::KeyPress {
            key: combat_virtual_key_plan(&command.args[0])?,
        }),
        CombatCommandMethod::Scroll => Ok(CombatCommandActionPlan::Scroll {
            clicks: parse_i32_arg(&command.args[0], "scroll amount")?,
        }),
        CombatCommandMethod::Round => Ok(CombatCommandActionPlan::Check {
            handled_by_fight_loop: false,
        }),
    }
}
