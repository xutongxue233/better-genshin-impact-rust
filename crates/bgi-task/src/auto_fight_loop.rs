use super::*;
use crate::common_job::{plan_scan_pick_drops, ScanPickDropsExecutionConfig};

pub fn plan_combat_fight_loop(
    param: &AutoFightParam,
    team_selection: &CombatScriptTeamSelectionPlan,
    team_plan: Option<&CombatTeamPlan>,
    selected_script_execution: Option<&CombatScriptExecutionPlan>,
) -> Result<CombatFightLoopPlan> {
    let mut steps = Vec::new();
    let command_count = selected_script_execution
        .map(|script| script.commands.len())
        .unwrap_or(0);
    let executable_command_count = team_selection.executable_commands.len();
    let guardian_avatar_index = parse_guardian_avatar_index(&param.guardian_avatar);
    let guardian_avatar_name = guardian_avatar_index.and_then(|index| {
        team_plan
            .and_then(|team| team.avatars.iter().find(|avatar| avatar.index == index))
            .map(|avatar| avatar.name.clone())
    });
    let guardian_enabled = guardian_avatar_index.is_some();
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Setup,
        CombatFightLoopStepKind::InitializeCancellation,
        None,
        None,
        true,
        Vec::new(),
        "create linked cancellation token and initialize combat avatar runtime context",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Setup,
        CombatFightLoopStepKind::StartExperienceDetector,
        None,
        None,
        param.kazuha_pickup_enabled && param.exp_based_pickup_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "start background elite-experience detector before command loop",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::LoopStart,
        CombatFightLoopStepKind::WaitAllConfiguredSkillCooldowns,
        None,
        None,
        team_plan
            .map(|team| team.all_command_avatars_can_be_skipped)
            .unwrap_or(false),
        vec![CombatExecutionContextRequirement::SkillCooldown],
        "when all command avatars are configured for CD skip, wait for the minimum remaining skill cooldown",
    );
    let commands = selected_script_execution
        .map(|script| script.commands.as_slice())
        .unwrap_or(&[]);
    for command in commands {
        let avatar = Some(command.command.avatar.clone());
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::EnsureGuardianSkill,
            Some(command.index),
            avatar.clone(),
            guardian_enabled,
            vec![
                CombatExecutionContextRequirement::AvatarSelection,
                CombatExecutionContextRequirement::SkillCooldown,
                CombatExecutionContextRequirement::InputEvents,
            ],
            "ensure configured guardian avatar skill before switching to a different combat avatar",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::InitialSeekEnemy,
            Some(command.index),
            avatar.clone(),
            command.index == 0
                && param.finish_detect_config.rotate_find_enemy_enabled
                && param.is_first_check,
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run initial seek-and-fight before the first command when rotate-find-enemy is enabled",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::SkipGuardianCommand,
            Some(command.index),
            avatar.clone(),
            guardian_avatar_name
                .as_ref()
                .map(|guardian| guardian == &command.command.avatar)
                .unwrap_or(false)
                && (param.guardian_combat_skip || param.burst_enabled),
            Vec::new(),
            "skip commands owned by the guardian avatar when guardian combat skip or burst mode is enabled",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::SkipCommandBySkillCooldown,
            Some(command.index),
            avatar.clone(),
            team_plan
                .map(|team| {
                    team.all_command_avatars_can_be_skipped
                        || team
                            .can_be_skipped_avatar_names
                            .iter()
                            .any(|name| name == &command.command.avatar)
                })
                .unwrap_or(false),
            vec![CombatExecutionContextRequirement::SkillCooldown],
            "skip this command when the selected avatar skill cooldown is still active",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::EnforceTimeout,
            Some(command.index),
            avatar.clone(),
            true,
            Vec::new(),
            "stop combat when timeout elapses or seek rotation reaches the legacy cap",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::BeforeCommand,
            CombatFightLoopStepKind::CheckBeforeBurst,
            Some(command.index),
            avatar.clone(),
            param.finish_detect_config.rotate_find_enemy_enabled
                && param.check_before_burst
                && combat_command_may_trigger_burst(&command.command),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "check fight finish before burst-like commands when configured",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::Command,
            CombatFightLoopStepKind::ExecuteCommand,
            Some(command.index),
            avatar.clone(),
            team_selection
                .executable_commands
                .iter()
                .any(|executable| executable == &command.command),
            command.pending_context.clone(),
            "execute the planned combat command through the native avatar/input/fight context",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::AfterCommand,
            CombatFightLoopStepKind::CountFightAvatarSwitch,
            Some(command.index),
            avatar.clone(),
            true,
            Vec::new(),
            "increment fight-count when this command ends an avatar command group",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::FinishDetection,
            CombatFightLoopStepKind::CheckCommandFinish,
            Some(command.index),
            avatar.clone(),
            param.fight_finish_detect_enabled
                && matches!(
                    command.action,
                    CombatCommandActionPlan::Check {
                        handled_by_fight_loop: true
                    }
                ),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run explicit finish detection for check commands",
        );
        push_fight_loop_step(
            &mut steps,
            CombatFightLoopStepPhase::FinishDetection,
            CombatFightLoopStepKind::FastCheckAfterAvatar,
            Some(command.index),
            avatar,
            param.fight_finish_detect_enabled
                && (is_last_command_group(command.index, commands)
                    || param.finish_detect_config.fast_check_enabled),
            vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
            "run end-of-script or fast avatar-bound finish detection",
        );
    }
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Cleanup,
        CombatFightLoopStepKind::ReleaseAllKeys,
        None,
        None,
        true,
        vec![CombatExecutionContextRequirement::InputEvents],
        "release all pressed keys when the fight loop exits",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Cleanup,
        CombatFightLoopStepKind::StopExperienceDetector,
        None,
        None,
        param.kazuha_pickup_enabled && param.exp_based_pickup_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "stop and dispose the background experience detector",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::ApplyBattleThresholdForLoot,
        None,
        None,
        param.battle_threshold_for_loot >= 2,
        Vec::new(),
        "skip loot pickup when fight-count is below the configured threshold",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::KazuhaOrJeanPickup,
        None,
        None,
        param.kazuha_pickup_enabled,
        vec![
            CombatExecutionContextRequirement::AvatarSelection,
            CombatExecutionContextRequirement::SkillCooldown,
            CombatExecutionContextRequirement::InputEvents,
        ],
        "perform Kazuha long-skill or Jean pickup when a picker avatar is available",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::SwitchPickupParty,
        None,
        None,
        param.kazuha_pickup_enabled && !param.kazuha_party_name.trim().is_empty(),
        vec![
            CombatExecutionContextRequirement::AvatarSelection,
            CombatExecutionContextRequirement::FightLoopFinishDetection,
        ],
        "switch to the configured pickup party when the current team lacks a picker",
    );
    push_fight_loop_step(
        &mut steps,
        CombatFightLoopStepPhase::Loot,
        CombatFightLoopStepKind::ScanPickDrops,
        None,
        None,
        param.pick_drops_after_fight_enabled,
        vec![CombatExecutionContextRequirement::FightLoopFinishDetection],
        "scan and approach loot beams after combat",
    );
    let native_dispatch_ready = !steps.iter().any(|step| {
        step.enabled
            && !step.requires_native_context.is_empty()
            && matches!(
                step.phase,
                CombatFightLoopStepPhase::Command
                    | CombatFightLoopStepPhase::FinishDetection
                    | CombatFightLoopStepPhase::Loot
                    | CombatFightLoopStepPhase::Cleanup
            )
    });
    let scan_pick_drops_plan = if param.pick_drops_after_fight_enabled {
        Some(plan_scan_pick_drops(ScanPickDropsExecutionConfig {
            scan_seconds: param.pick_drops_after_fight_seconds.try_into().unwrap_or(0),
            ..ScanPickDropsExecutionConfig::default()
        })?)
    } else {
        None
    };
    Ok(CombatFightLoopPlan {
        timeout_seconds: param.timeout,
        command_count,
        executable_command_count,
        fight_finish_detect_enabled: param.fight_finish_detect_enabled,
        rotate_find_enemy_enabled: param.finish_detect_config.rotate_find_enemy_enabled,
        check_before_burst_enabled: param.check_before_burst,
        guardian_enabled,
        guardian_avatar_index,
        guardian_avatar_name,
        kazuha_pickup_enabled: param.kazuha_pickup_enabled,
        pickup_drops_after_fight_enabled: param.pick_drops_after_fight_enabled,
        scan_pick_drops_plan,
        exp_based_pickup_enabled: param.exp_based_pickup_enabled,
        battle_threshold_for_loot: param.battle_threshold_for_loot,
        steps,
        native_dispatch_ready,
    })
}

fn push_fight_loop_step(
    steps: &mut Vec<CombatFightLoopStepPlan>,
    phase: CombatFightLoopStepPhase,
    kind: CombatFightLoopStepKind,
    command_index: Option<usize>,
    avatar: Option<String>,
    enabled: bool,
    requires_native_context: Vec<CombatExecutionContextRequirement>,
    message: &str,
) {
    steps.push(CombatFightLoopStepPlan {
        phase,
        kind,
        command_index,
        avatar,
        enabled,
        requires_native_context,
        message: message.to_string(),
    });
}

fn parse_guardian_avatar_index(value: &str) -> Option<usize> {
    let index = value.trim().parse::<usize>().ok()?;
    (1..=EXPECTED_COMBAT_TEAM_AVATAR_COUNT)
        .contains(&index)
        .then_some(index)
}

fn combat_command_may_trigger_burst(command: &CombatCommandPlan) -> bool {
    command.method == CombatCommandMethod::Burst
        || command.args.iter().any(|arg| arg.eq_ignore_ascii_case("q"))
}

fn is_last_command_group(index: usize, commands: &[CombatCommandExecutionPlan]) -> bool {
    let Some(command) = commands.get(index) else {
        return false;
    };
    commands
        .get(index + 1)
        .map(|next| next.command.avatar != command.command.avatar)
        .unwrap_or(true)
}
