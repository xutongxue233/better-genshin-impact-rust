use super::input_events::combat_action_events;
use super::*;
use crate::task_params::FightFinishDetectParam;
use crate::{Result, TaskError};
use bgi_core::{GenshinAction, KeyBindingsConfig};
use bgi_input::{
    send_events, send_events_with_cancellation, InputCancellationToken, InputError, InputEvent,
    KeyActionType,
};
use bgi_vision::BgrImage;

pub fn plan_auto_fight_finish_detection(
    config: &FightFinishDetectParam,
    delay_ms: u64,
    detect_delay_ms: u64,
) -> Result<AutoFightFinishDetectionPlan> {
    let bindings = KeyBindingsConfig::default();
    let open_party_events = combat_action_events(
        &bindings,
        GenshinAction::OpenPartySetupScreen,
        KeyActionType::KeyPress,
    )?;
    let drop_events =
        combat_action_events(&bindings, GenshinAction::Drop, KeyActionType::KeyPress)?;
    let mut steps = Vec::new();
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::PreDetectDelay,
        !config.rotate_find_enemy_enabled && delay_ms > 0,
        Vec::new(),
        delay_ms,
        false,
        false,
        "wait before opening party setup when rotate-find-enemy is disabled",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::SeekEnemy,
        config.rotate_find_enemy_enabled,
        Vec::new(),
        delay_ms,
        true,
        true,
        "run seek-and-fight finish probe before party-screen pixel detection",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::OpenPartySetup,
        true,
        open_party_events.clone(),
        0,
        false,
        false,
        "press the configured party setup key before checking fight-finish pixels",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::WaitForPartySetup,
        detect_delay_ms > 0,
        Vec::new(),
        detect_delay_ms,
        false,
        false,
        "wait for the party setup screen before capturing a frame",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::CaptureFrame,
        true,
        Vec::new(),
        0,
        true,
        false,
        "capture the game frame after opening party setup",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::SampleFinishPixels,
        true,
        Vec::new(),
        0,
        false,
        true,
        "sample the legacy white tile and yellow progress pixels",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::DropFromPartySetup,
        true,
        drop_events,
        0,
        false,
        false,
        "press the configured drop key to leave the party setup probe",
    );
    push_finish_detection_step(
        &mut steps,
        AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished,
        true,
        open_party_events,
        0,
        false,
        false,
        "press party setup again when finish pixels indicate combat has ended",
    );
    let native_ready_without_capture = !steps
        .iter()
        .any(|step| step.enabled && (step.requires_capture || step.requires_vision));
    Ok(AutoFightFinishDetectionPlan {
        pre_detect_delay_ms: delay_ms,
        detect_delay_ms,
        rotate_find_enemy_enabled: config.rotate_find_enemy_enabled,
        progress_pixel: AUTO_FIGHT_FINISH_PROGRESS_PIXEL,
        white_tile_pixel: AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL,
        steps,
        native_ready_without_capture,
    })
}

fn push_finish_detection_step(
    steps: &mut Vec<AutoFightFinishDetectionStepPlan>,
    kind: AutoFightFinishDetectionStepKind,
    enabled: bool,
    input_events: Vec<InputEvent>,
    delay_ms: u64,
    requires_capture: bool,
    requires_vision: bool,
    message: &str,
) {
    steps.push(AutoFightFinishDetectionStepPlan {
        kind,
        enabled,
        input_events,
        delay_ms,
        requires_capture,
        requires_vision,
        message: message.to_string(),
    });
}

pub fn execute_auto_fight_finish_detection_probe(
    plan: &AutoFightFinishDetectionPlan,
    image: &BgrImage,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Option<&InputCancellationToken>,
) -> Result<AutoFightFinishDetectionExecution> {
    let before_capture_events = finish_detection_events_before_capture(plan);
    let detection = detect_auto_fight_finished_from_image(image)?;
    let after_detection_events = finish_detection_events_after_detection(plan, detection.finished);
    let mut dispatched_events = 0;
    let mut cancelled = false;
    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let mut events = before_capture_events.clone();
        events.extend(after_detection_events.iter().copied());
        let result = dispatch_auto_fight_input_events(&events, cancellation)?;
        dispatched_events = result.0;
        cancelled = result.1;
    }
    Ok(AutoFightFinishDetectionExecution {
        mode,
        plan: plan.clone(),
        detection,
        before_capture_events,
        after_detection_events,
        dispatched: matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput),
        dispatched_events,
        cancelled,
    })
}

pub fn execute_auto_fight_finish_detection_live_probe(
    plan: &AutoFightFinishDetectionPlan,
    mode: AutoFightFinishDetectionExecutionMode,
    cancellation: Option<&InputCancellationToken>,
    capture: impl FnOnce() -> Result<BgrImage>,
) -> Result<AutoFightFinishDetectionLiveExecution> {
    let before_capture_events = finish_detection_events_before_capture(plan);
    let mut dispatched_events = 0;
    let mut cancelled = false;

    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let result = dispatch_auto_fight_input_events(&before_capture_events, cancellation)?;
        dispatched_events += result.0;
        cancelled = result.1;
        if cancelled {
            return Ok(AutoFightFinishDetectionLiveExecution {
                mode,
                plan: plan.clone(),
                detection: None,
                before_capture_events,
                after_detection_events: Vec::new(),
                dispatched: true,
                dispatched_events,
                cancelled,
                captured: false,
            });
        }
    }

    let image = capture()?;
    let detection = detect_auto_fight_finished_from_image(&image)?;
    let after_detection_events = finish_detection_events_after_detection(plan, detection.finished);

    if matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput) {
        let result = dispatch_auto_fight_input_events(&after_detection_events, cancellation)?;
        dispatched_events += result.0;
        cancelled = result.1;
    }

    Ok(AutoFightFinishDetectionLiveExecution {
        mode,
        plan: plan.clone(),
        detection: Some(detection),
        before_capture_events,
        after_detection_events,
        dispatched: matches!(mode, AutoFightFinishDetectionExecutionMode::SendInput),
        dispatched_events,
        cancelled,
        captured: true,
    })
}

pub(super) fn dispatch_auto_fight_input_events(
    events: &[InputEvent],
    cancellation: Option<&InputCancellationToken>,
) -> Result<(usize, bool)> {
    if events.is_empty() {
        return Ok((0, false));
    }
    let result = if let Some(cancellation) = cancellation {
        match send_events_with_cancellation(events, cancellation) {
            Ok(report) => Ok((report.dispatched_events, report.cancelled)),
            Err(InputError::Cancelled {
                dispatched_events, ..
            }) => Ok((dispatched_events, true)),
            Err(error) => Err(error),
        }
    } else {
        send_events(events).map(|_| (events.len(), false))
    };
    result.map_err(|error| TaskError::CombatInputDispatch(error.to_string()))
}

pub fn finish_detection_events_before_capture(
    plan: &AutoFightFinishDetectionPlan,
) -> Vec<InputEvent> {
    plan.steps
        .iter()
        .filter(|step| {
            step.enabled
                && matches!(
                    step.kind,
                    AutoFightFinishDetectionStepKind::PreDetectDelay
                        | AutoFightFinishDetectionStepKind::OpenPartySetup
                        | AutoFightFinishDetectionStepKind::WaitForPartySetup
                )
        })
        .flat_map(finish_detection_step_events)
        .collect()
}

pub fn finish_detection_events_after_detection(
    plan: &AutoFightFinishDetectionPlan,
    finished: bool,
) -> Vec<InputEvent> {
    plan.steps
        .iter()
        .filter(|step| {
            step.enabled
                && matches!(
                    step.kind,
                    AutoFightFinishDetectionStepKind::DropFromPartySetup
                        | AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished
                )
                && (step.kind != AutoFightFinishDetectionStepKind::CancelPartySwitchWhenFinished
                    || finished)
        })
        .flat_map(finish_detection_step_events)
        .collect()
}

fn finish_detection_step_events(step: &AutoFightFinishDetectionStepPlan) -> Vec<InputEvent> {
    let mut events = Vec::new();
    events.extend(step.input_events.iter().copied());
    if step.delay_ms > 0 {
        events.push(InputEvent::Delay {
            milliseconds: step.delay_ms,
        });
    }
    events
}
