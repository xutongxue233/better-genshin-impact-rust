use bgi_core::{AutoRestartConfig, PathingFarmingExecutionPlan};

#[cfg(test)]
#[path = "pathing_result_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathingRunResult {
    pub success_end: bool,
    pub success_fight_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathingResultException {
    PathingFailure { message: String },
    FightFailure { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathingResultDecision {
    pub execution_record_success: bool,
    pub should_record_farming_session: bool,
    pub success_fight: bool,
    pub expected_fight_count: usize,
    pub warnings: Vec<String>,
    pub exception: Option<PathingResultException>,
}

pub fn decide_pathing_result(
    result: PathingRunResult,
    farming: &PathingFarmingExecutionPlan,
    auto_restart: &AutoRestartConfig,
) -> PathingResultDecision {
    let mut warnings = Vec::new();
    if !result.success_end {
        warnings.push("此追踪脚本未正常走完！".to_string());
        if auto_restart.enabled && auto_restart.is_pathing_failure_exceptional {
            return PathingResultDecision {
                execution_record_success: result.success_end,
                should_record_farming_session: false,
                success_fight: false,
                expected_fight_count: 0,
                warnings,
                exception: Some(PathingResultException::PathingFailure {
                    message: "路径追踪任务未完全走完，判定失败，触发异常！".to_string(),
                }),
            };
        }
    }

    if !farming.allow_farming_count {
        return PathingResultDecision {
            execution_record_success: result.success_end,
            should_record_farming_session: false,
            success_fight: result.success_end,
            expected_fight_count: 0,
            warnings,
            exception: None,
        };
    }

    let mut success_fight = result.success_end;
    let mut expected_fight_count = 0;
    if !success_fight {
        expected_fight_count = farming.expected_fight_count;
        success_fight = result.success_fight_count >= expected_fight_count;

        if farming.primary_target != "disable"
            && auto_restart.enabled
            && auto_restart.is_fight_failure_exceptional
            && !success_fight
        {
            return PathingResultDecision {
                execution_record_success: result.success_end,
                should_record_farming_session: false,
                success_fight,
                expected_fight_count,
                warnings,
                exception: Some(PathingResultException::FightFailure {
                    message: format!(
                        "实际战斗次数({})<预期战斗次数（{}），判定失败，触发异常！",
                        result.success_fight_count, expected_fight_count
                    ),
                }),
            };
        }
    }

    if !success_fight {
        warnings.push(format!(
            "实际战斗次数({})<预期战斗次数（{}），判定失败，此次不纳入成功锄地规划的统计上限！",
            result.success_fight_count, expected_fight_count
        ));
    }

    PathingResultDecision {
        execution_record_success: result.success_end,
        should_record_farming_session: success_fight,
        success_fight,
        expected_fight_count,
        warnings,
        exception: None,
    }
}
