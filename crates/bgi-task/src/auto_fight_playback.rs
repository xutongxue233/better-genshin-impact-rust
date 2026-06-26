use super::finish_detection::dispatch_auto_fight_input_events;
use super::input::{
    combat_avatar_switch_events, combat_avatar_switch_policy, combat_command_action_plan,
    combat_command_context_requirements, refresh_team_playback_execution,
};
use super::input_events::default_input_events_for_combat_action;
use super::model::{
    CombatCommandExecutionPlan, CombatCommandPlan, CombatCommandPlaybackExecution,
    CombatCommandPlaybackMode, CombatExecutionContextRequirement, CombatScriptBagPlan,
    CombatScriptExecutionPlan, CombatScriptPlan, CombatScriptPlaybackBatchEvaluation,
    CombatScriptPlaybackEvaluation, CombatTeamPlan, CombatTeamPlaybackCommandPlan,
    CombatTeamPlaybackExecution, CURRENT_COMBAT_AVATAR_NAME,
};
use super::vision::{
    detect_active_combat_avatar_index_from_default_rects_with_arrow, detect_combat_skill_readiness,
};
use crate::{Result, TaskError};
use bgi_input::{send_events, send_events_with_cancellation, InputCancellationToken, InputError};
use bgi_vision::BgrImage;
use std::path::Path;

pub fn plan_combat_script_execution(
    script: &CombatScriptPlan,
) -> Result<CombatScriptExecutionPlan> {
    let mut commands = Vec::with_capacity(script.commands.len());
    let mut previous = None;
    for (index, command) in script.commands.iter().enumerate() {
        let plan = plan_combat_command_execution(index, command, previous)?;
        previous = Some(command);
        commands.push(plan);
    }
    Ok(CombatScriptExecutionPlan {
        name: script.name.clone(),
        path: script.path.clone(),
        avatar_names: script.avatar_names.clone(),
        commands,
    })
}

pub fn plan_combat_command_execution(
    index: usize,
    command: &CombatCommandPlan,
    previous: Option<&CombatCommandPlan>,
) -> Result<CombatCommandExecutionPlan> {
    let switch_policy = combat_avatar_switch_policy(command, previous);
    let action = combat_command_action_plan(command)?;
    let default_input_events = default_input_events_for_combat_action(&action)?;
    let pending_context = combat_command_context_requirements(
        switch_policy,
        &action,
        default_input_events.is_empty(),
    );
    let requires_combat_context = !pending_context.is_empty();
    let static_input_ready = !requires_combat_context && !default_input_events.is_empty();
    Ok(CombatCommandExecutionPlan {
        index,
        command: command.clone(),
        switch_policy,
        action,
        default_input_events,
        requires_combat_context,
        static_input_ready,
        pending_context,
    })
}

pub fn plan_combat_script_bag_execution(
    bag: &CombatScriptBagPlan,
) -> Result<Vec<CombatScriptExecutionPlan>> {
    bag.scripts
        .iter()
        .map(plan_combat_script_execution)
        .collect()
}

pub fn evaluate_combat_script_playback(
    script: &CombatScriptExecutionPlan,
) -> CombatScriptPlaybackEvaluation {
    let static_ready_commands = script
        .commands
        .iter()
        .filter(|command| command.static_input_ready)
        .count();
    let context_bound_commands = script
        .commands
        .iter()
        .filter(|command| command.requires_combat_context)
        .count();
    let default_input_event_count = script
        .commands
        .iter()
        .map(|command| command.default_input_events.len())
        .sum();
    let first_blocking_command = script
        .commands
        .iter()
        .find(|command| command.requires_combat_context);
    CombatScriptPlaybackEvaluation {
        script_name: script.name.clone(),
        script_path: script.path.clone(),
        total_commands: script.commands.len(),
        static_ready_commands,
        context_bound_commands,
        default_input_event_count,
        first_blocking_command_index: first_blocking_command.map(|command| command.index),
        first_blocking_requirements: first_blocking_command
            .map(|command| command.pending_context.clone())
            .unwrap_or_default(),
    }
}

pub fn evaluate_combat_script_batch_playback(
    scripts: &[CombatScriptExecutionPlan],
) -> CombatScriptPlaybackBatchEvaluation {
    let scripts: Vec<_> = scripts
        .iter()
        .map(evaluate_combat_script_playback)
        .collect();
    let total_commands = scripts.iter().map(|script| script.total_commands).sum();
    let static_ready_commands = scripts
        .iter()
        .map(|script| script.static_ready_commands)
        .sum();
    let context_bound_commands = scripts
        .iter()
        .map(|script| script.context_bound_commands)
        .sum();
    let default_input_event_count = scripts
        .iter()
        .map(|script| script.default_input_event_count)
        .sum();
    let dispatch_ready = total_commands > 0 && context_bound_commands == 0;
    CombatScriptPlaybackBatchEvaluation {
        scripts,
        total_commands,
        static_ready_commands,
        context_bound_commands,
        default_input_event_count,
        dispatch_ready,
    }
}

pub fn execute_static_combat_script_inputs(
    script: &CombatScriptExecutionPlan,
    mode: CombatCommandPlaybackMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<CombatCommandPlaybackExecution> {
    let evaluation = evaluate_combat_script_playback(script);
    if evaluation.context_bound_commands > 0 {
        return Err(TaskError::CombatStrategy(format!(
            "combat script requires native combat context before input dispatch: {:?}",
            evaluation.first_blocking_requirements
        )));
    }
    let input_events = script
        .commands
        .iter()
        .flat_map(|command| command.default_input_events.iter().copied())
        .collect::<Vec<_>>();
    if input_events.is_empty() && matches!(mode, CombatCommandPlaybackMode::SendInput) {
        return Err(TaskError::CombatStrategy(
            "combat script has no static input events to dispatch".to_string(),
        ));
    }
    let mut dispatched_events = 0;
    let mut cancelled = false;
    if matches!(mode, CombatCommandPlaybackMode::SendInput) {
        let result = if let Some(cancellation) = cancellation {
            match send_events_with_cancellation(&input_events, cancellation) {
                Ok(report) => Ok((report.dispatched_events, report.cancelled)),
                Err(InputError::Cancelled {
                    dispatched_events, ..
                }) => Ok((dispatched_events, true)),
                Err(error) => Err(error),
            }
        } else {
            send_events(&input_events).map(|_| (input_events.len(), false))
        };
        let result = result.map_err(|error| TaskError::CombatInputDispatch(error.to_string()))?;
        dispatched_events = result.0;
        cancelled = result.1;
    }
    Ok(CombatCommandPlaybackExecution {
        mode,
        script_name: script.name.clone(),
        total_commands: evaluation.total_commands,
        static_ready_commands: evaluation.static_ready_commands,
        context_bound_commands: evaluation.context_bound_commands,
        input_events,
        dispatched: matches!(mode, CombatCommandPlaybackMode::SendInput),
        dispatched_events,
        cancelled,
    })
}

pub fn plan_team_context_combat_script_playback(
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
) -> Result<CombatTeamPlaybackExecution> {
    let candidate_commands: Vec<_> = script
        .commands
        .iter()
        .filter(|command| {
            executable_commands
                .iter()
                .any(|executable| executable == &command.command)
        })
        .collect();
    let mut planned_commands = Vec::with_capacity(candidate_commands.len());
    let mut input_events = Vec::new();
    let mut blocked_command_index = None;
    let mut blocked_requirements = Vec::new();

    for command in candidate_commands {
        let team_avatar = if command.command.avatar == CURRENT_COMBAT_AVATAR_NAME {
            None
        } else {
            team_plan
                .avatars
                .iter()
                .find(|avatar| avatar.name == command.command.avatar)
        };
        let team_index = team_avatar.map(|avatar| avatar.index);
        let mut switch_events = Vec::new();
        let mut resolved_context = Vec::new();
        let mut pending_context = command.pending_context.clone();
        let mut executable = true;
        let mut message = "command input is ready for known team context".to_string();

        if pending_context.contains(&CombatExecutionContextRequirement::AvatarSelection) {
            if let Some(index) = team_index {
                switch_events = combat_avatar_switch_events(index)?;
                pending_context
                    .retain(|item| item != &CombatExecutionContextRequirement::AvatarSelection);
                resolved_context.push(CombatExecutionContextRequirement::AvatarSelection);
            } else {
                executable = false;
                message = format!(
                    "avatar {} is not available in the configured team",
                    command.command.avatar
                );
            }
        }
        if pending_context.contains(&CombatExecutionContextRequirement::InputEvents)
            && !command.default_input_events.is_empty()
        {
            pending_context.retain(|item| item != &CombatExecutionContextRequirement::InputEvents);
            resolved_context.push(CombatExecutionContextRequirement::InputEvents);
        }
        if !pending_context.is_empty() {
            executable = false;
            message = format!(
                "command still requires native combat context: {:?}",
                pending_context
            );
        }

        let mut command_events = Vec::new();
        if executable {
            command_events.extend(switch_events.iter().copied());
            command_events.extend(command.default_input_events.iter().copied());
            input_events.extend(command_events.iter().copied());
        } else if blocked_command_index.is_none() {
            blocked_command_index = Some(command.index);
            blocked_requirements = pending_context.clone();
        }

        planned_commands.push(CombatTeamPlaybackCommandPlan {
            command_index: command.index,
            avatar: command.command.avatar.clone(),
            team_index,
            switch_events,
            action_events: command.default_input_events.clone(),
            input_events: command_events,
            resolved_context,
            pending_context,
            executable,
            message,
        });
    }

    let playable_commands = planned_commands
        .iter()
        .filter(|command| command.executable)
        .count();
    let dispatch_ready = !planned_commands.is_empty()
        && blocked_command_index.is_none()
        && playable_commands == planned_commands.len()
        && !input_events.is_empty();
    Ok(CombatTeamPlaybackExecution {
        mode: CombatCommandPlaybackMode::PlanOnly,
        script_name: script.name.clone(),
        total_commands: script.commands.len(),
        candidate_commands: planned_commands.len(),
        planned_commands,
        playable_commands,
        blocked_command_index,
        blocked_requirements,
        input_events,
        dispatch_ready,
        dispatched: false,
        dispatched_events: 0,
        cancelled: false,
    })
}

pub fn plan_team_context_combat_script_playback_with_frame(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
) -> Result<CombatTeamPlaybackExecution> {
    let working_directory = working_directory.as_ref();
    let mut execution =
        plan_team_context_combat_script_playback(script, team_plan, executable_commands)?;
    let active_detection =
        detect_active_combat_avatar_index_from_default_rects_with_arrow(working_directory, image)?;
    let active_index = active_detection.active_index;

    for command in &mut execution.planned_commands {
        let target_is_active = command.avatar == CURRENT_COMBAT_AVATAR_NAME
            || command
                .team_index
                .zip(active_index)
                .map(|(target, active)| target == active)
                .unwrap_or(false);

        if target_is_active {
            if !command.switch_events.is_empty() {
                command.switch_events.clear();
                if !command
                    .resolved_context
                    .contains(&CombatExecutionContextRequirement::AvatarSelection)
                {
                    command
                        .resolved_context
                        .push(CombatExecutionContextRequirement::AvatarSelection);
                }
                command.message =
                    "target avatar is already active in the supplied frame; switch input was removed"
                        .to_string();
            }

            if command
                .pending_context
                .contains(&CombatExecutionContextRequirement::SkillCooldown)
            {
                let requested_index = command.team_index.or(active_index).unwrap_or(1);
                let readiness = detect_combat_skill_readiness(
                    working_directory,
                    image,
                    requested_index,
                    false,
                )?;
                if readiness.ready == Some(true) {
                    command
                        .pending_context
                        .retain(|item| item != &CombatExecutionContextRequirement::SkillCooldown);
                    command
                        .resolved_context
                        .push(CombatExecutionContextRequirement::SkillCooldown);
                    command.message =
                        "skill cooldown was resolved from the supplied active-avatar frame"
                            .to_string();
                } else {
                    command.message = format!(
                        "skill cooldown remains pending after frame readiness check: {:?}",
                        readiness.status
                    );
                }
            }
        }

        if command
            .pending_context
            .contains(&CombatExecutionContextRequirement::BurstReadiness)
        {
            let requested_index = command.team_index.or(active_index).unwrap_or(1);
            let readiness =
                detect_combat_skill_readiness(working_directory, image, requested_index, true)?;
            if readiness.ready == Some(true) {
                command
                    .pending_context
                    .retain(|item| item != &CombatExecutionContextRequirement::BurstReadiness);
                command
                    .resolved_context
                    .push(CombatExecutionContextRequirement::BurstReadiness);
                command.message =
                    "burst readiness was resolved from the supplied active-avatar frame"
                        .to_string();
            } else {
                command.message = format!(
                    "burst readiness remains pending after frame readiness check: {:?}",
                    readiness.status
                );
            }
        }

        command.resolved_context.sort();
        command.resolved_context.dedup();
    }
    refresh_team_playback_execution(&mut execution);
    Ok(execution)
}

pub fn execute_team_context_combat_script_inputs(
    script: &CombatScriptExecutionPlan,
    team_plan: &CombatTeamPlan,
    executable_commands: &[CombatCommandPlan],
    mode: CombatCommandPlaybackMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<CombatTeamPlaybackExecution> {
    let mut execution =
        plan_team_context_combat_script_playback(script, team_plan, executable_commands)?;
    execution.mode = mode;
    if matches!(mode, CombatCommandPlaybackMode::SendInput) {
        if !execution.dispatch_ready {
            return Err(TaskError::CombatStrategy(format!(
                "team-context combat playback is not dispatch ready; first blocked command: {:?}, requirements: {:?}",
                execution.blocked_command_index, execution.blocked_requirements
            )));
        }
        let result = dispatch_auto_fight_input_events(&execution.input_events, cancellation)?;
        execution.dispatched_events = result.0;
        execution.cancelled = result.1;
    }
    execution.dispatched = matches!(mode, CombatCommandPlaybackMode::SendInput);
    Ok(execution)
}
