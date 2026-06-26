use super::*;

#[test]
fn pathing_failure_exception_preempts_farming_recording() {
    let auto_restart = AutoRestartConfig {
        enabled: true,
        is_pathing_failure_exceptional: true,
        is_fight_failure_exceptional: true,
        ..AutoRestartConfig::default()
    };

    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: false,
            success_fight_count: 3,
        },
        &farming("elite", 3, true),
        &auto_restart,
    );

    assert!(!decision.execution_record_success);
    assert!(!decision.should_record_farming_session);
    assert_eq!(
        decision.exception,
        Some(PathingResultException::PathingFailure {
            message: "路径追踪任务未完全走完，判定失败，触发异常！".to_string()
        })
    );
}

#[test]
fn fight_failure_exception_applies_after_pathing_failure_policy() {
    let auto_restart = AutoRestartConfig {
        enabled: true,
        is_fight_failure_exceptional: true,
        ..AutoRestartConfig::default()
    };

    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: false,
            success_fight_count: 1,
        },
        &farming("elite", 2, true),
        &auto_restart,
    );

    assert!(!decision.execution_record_success);
    assert!(!decision.should_record_farming_session);
    assert_eq!(decision.expected_fight_count, 2);
    assert_eq!(
        decision.exception,
        Some(PathingResultException::FightFailure {
            message: "实际战斗次数(1)<预期战斗次数（2），判定失败，触发异常！".to_string()
        })
    );
}

#[test]
fn enough_fights_record_farming_but_keep_execution_record_failed() {
    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: false,
            success_fight_count: 2,
        },
        &farming("elite", 2, true),
        &AutoRestartConfig::default(),
    );

    assert!(!decision.execution_record_success);
    assert!(decision.success_fight);
    assert!(decision.should_record_farming_session);
    assert!(decision.exception.is_none());
}

#[test]
fn success_end_records_farming_without_checking_fight_count() {
    let auto_restart = AutoRestartConfig {
        enabled: true,
        is_fight_failure_exceptional: true,
        ..AutoRestartConfig::default()
    };

    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: true,
            success_fight_count: 0,
        },
        &farming("elite", 2, true),
        &auto_restart,
    );

    assert!(decision.execution_record_success);
    assert!(decision.success_fight);
    assert!(decision.should_record_farming_session);
    assert_eq!(decision.expected_fight_count, 0);
    assert!(decision.exception.is_none());
}

#[test]
fn disable_primary_target_suppresses_fight_exception_only() {
    let auto_restart = AutoRestartConfig {
        enabled: true,
        is_fight_failure_exceptional: true,
        ..AutoRestartConfig::default()
    };

    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: false,
            success_fight_count: 1,
        },
        &farming("disable", 2, true),
        &auto_restart,
    );

    assert!(!decision.execution_record_success);
    assert!(!decision.should_record_farming_session);
    assert!(decision.exception.is_none());
    assert!(decision
        .warnings
        .iter()
        .any(|warning| warning.contains("此次不纳入成功锄地规划的统计上限")));
}

#[test]
fn non_counting_route_does_not_record_or_check_fights() {
    let auto_restart = AutoRestartConfig {
        enabled: true,
        is_fight_failure_exceptional: true,
        ..AutoRestartConfig::default()
    };

    let decision = decide_pathing_result(
        PathingRunResult {
            success_end: false,
            success_fight_count: 0,
        },
        &farming("elite", 2, false),
        &auto_restart,
    );

    assert!(!decision.execution_record_success);
    assert!(!decision.should_record_farming_session);
    assert_eq!(decision.expected_fight_count, 0);
    assert!(decision.exception.is_none());
}

fn farming(
    primary_target: impl Into<String>,
    expected_fight_count: usize,
    allow_farming_count: bool,
) -> PathingFarmingExecutionPlan {
    PathingFarmingExecutionPlan {
        allow_farming_count,
        primary_target: primary_target.into(),
        normal_mob_count: 0.0,
        elite_mob_count: 0.0,
        expected_fight_count,
    }
}
