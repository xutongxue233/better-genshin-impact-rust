use crate::TaskPortState;
use bgi_core::{plan_linnea_mining_action, GenshinAction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LINNEA_MINING_TASK_KEY: &str = "LinneaMining";
pub const LINNEA_MINING_DEFAULT_MINE_COUNT: i32 = 1;
pub const LINNEA_MINING_DEFAULT_SCAN_ROUNDS: i32 = 1;
pub const LINNEA_MINING_MODEL_NAME: &str = "BgiMine";
pub const LINNEA_MINING_MODEL_PATH: &str = "Assets/Model/Mine/bgi_mine.onnx";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub port_state: TaskPortState,
    pub executor_ready: bool,
    pub action_code: String,
    pub raw_params: Option<String>,
    pub mine_count: i32,
    pub scan_rounds: i32,
    pub prefer_right: bool,
    pub avatar_rule: LinneaMiningAvatarRule,
    pub aiming_rule: LinneaMiningAimingRule,
    pub detection_rule: LinneaMiningDetectionRule,
    pub cluster_rule: LinneaMiningClusterRule,
    pub alignment_rule: LinneaMiningAlignmentRule,
    pub scan_rule: LinneaMiningScanRule,
    pub mine_rule: LinneaMiningMineRule,
    pub cleanup_rule: LinneaMiningCleanupRule,
    pub steps: Vec<LinneaMiningStep>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LinneaMiningExecutionConfig {
    pub action_params: Option<String>,
    pub mine_count: Option<i32>,
    pub scan_rounds: Option<i32>,
}

impl Default for LinneaMiningExecutionConfig {
    fn default() -> Self {
        Self {
            action_params: None,
            mine_count: None,
            scan_rounds: None,
        }
    }
}

impl LinneaMiningExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let Some(value) = value else {
            return Self::default();
        };

        if let Some(params) = value.as_str() {
            return Self {
                action_params: Some(params.to_string()),
                ..Self::default()
            };
        }

        let mut config = Self::default();
        config.action_params = string_member(
            value,
            [
                "actionParams",
                "ActionParams",
                "action_params",
                "rawParams",
                "raw_params",
            ],
        );
        config.mine_count = i32_member(value, ["mineCount", "MineCount", "mine_count", "mines"]);
        config.scan_rounds =
            i32_member(value, ["scanRounds", "ScanRounds", "scan_rounds", "rounds"]);
        config
    }

    fn effective_action_params(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(mine_count) = self.mine_count {
            parts.push(format!("mines={mine_count}"));
        }
        if let Some(scan_rounds) = self.scan_rounds {
            parts.push(format!("rounds={scan_rounds}"));
        }
        if parts.is_empty() {
            self.action_params.clone()
        } else {
            Some(parts.join(","))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningAvatarRule {
    pub avatar_name: String,
    pub switch_avatar_before_mining: bool,
    pub switch_wait_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningAimingRule {
    pub aiming_mode_action: GenshinAction,
    pub enter_aim_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningDetectionRule {
    pub model_name: String,
    pub model_relative_path: String,
    pub accepted_label: String,
    pub confidence_threshold: f32,
    pub source: LinneaMiningDetectionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinneaMiningDetectionSource {
    FullCapture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningClusterRule {
    pub base_cluster_distance_1080p: f64,
    pub base_cluster_area_1080p: f64,
    pub base_alignment_expansion_1080p: f64,
    pub base_edge_ignore_1080p: f64,
    pub area_ratio_threshold: f64,
    pub prefer_right_when_scan_rounds_gt_one: bool,
    pub target_selection: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningAlignmentRule {
    pub max_inner_retry: u8,
    pub element_sight_refresh_ms: u32,
    pub refresh_release_ms: u32,
    pub refresh_hold_ms: u32,
    pub aim_sensitivity_factor_x: f64,
    pub aim_sensitivity_factor_y: f64,
    pub aim_move_delay_ms: u32,
    pub fallback_shot_on_last_successful_detection: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinneaMiningScanRule {
    pub middle_button_hold_ms: u32,
    pub middle_button_release_ms: u32,
    pub compensate_detection_hold_ms: u32,
    pub compensate_move_wait_ms: u32,
    pub left_turn_step_1080p: i32,
    pub left_turn_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningMineRule {
    pub compensate_up_pixels: i32,
    pub compensate_up_wait_ms: u32,
    pub attack_button: String,
    pub after_attack_wait_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningCleanupRule {
    pub leave_aiming_mode_action: GenshinAction,
    pub middle_button_up: bool,
    pub clear_vision_drawings: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinneaMiningStep {
    pub phase: LinneaMiningStepPhase,
    pub condition: LinneaMiningStepCondition,
    pub label: String,
    pub action: LinneaMiningStepAction,
}

impl LinneaMiningStep {
    fn new(
        phase: LinneaMiningStepPhase,
        condition: LinneaMiningStepCondition,
        label: impl Into<String>,
        action: LinneaMiningStepAction,
    ) -> Self {
        Self {
            phase,
            condition,
            label: label.into(),
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningStepPhase {
    Setup,
    ScanLoop,
    Detection,
    Alignment,
    Mine,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinneaMiningStepCondition {
    Always,
    PerScanRound,
    WhenClusterDetected,
    WhenClusterMissing,
    WhenAlignedOrFallback,
    WhenDetectionLost,
    BetweenScanRounds,
    Finally,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum LinneaMiningStepAction {
    SwitchAvatar {
        avatar_name: String,
        wait_ms: u32,
    },
    ToggleAimingMode {
        action: GenshinAction,
        wait_ms: u32,
    },
    HoldElementalSight {
        hold_ms: u32,
    },
    DetectMineralCluster,
    AlignTarget {
        max_retries: u8,
    },
    MineTarget {
        attack_button: String,
        after_attack_wait_ms: u32,
    },
    CompensateLostDetection,
    TurnForNextScanRound {
        left_turn_step_1080p: i32,
        wait_ms: u32,
    },
    ReleaseMiddleButton,
    ClearVisionDrawings,
    MarkNativePending {
        message: String,
    },
}

pub fn plan_linnea_mining(config: LinneaMiningExecutionConfig) -> LinneaMiningExecutionPlan {
    let effective_action_params = config.effective_action_params();
    let core = plan_linnea_mining_action(effective_action_params.as_deref());

    let avatar_rule = LinneaMiningAvatarRule {
        avatar_name: core.avatar_name.clone(),
        switch_avatar_before_mining: core.switch_avatar_before_mining,
        switch_wait_ms: core.switch_wait_ms,
    };
    let aiming_rule = LinneaMiningAimingRule {
        aiming_mode_action: core.aiming_mode_action,
        enter_aim_wait_ms: core.enter_aim_wait_ms,
    };
    let detection_rule = LinneaMiningDetectionRule {
        model_name: core.detection_rule.model_name.clone(),
        model_relative_path: core.detection_rule.model_relative_path.clone(),
        accepted_label: core.detection_rule.accepted_label.clone(),
        confidence_threshold: core.detection_rule.confidence_threshold,
        source: LinneaMiningDetectionSource::FullCapture,
    };
    let cluster_rule = LinneaMiningClusterRule {
        base_cluster_distance_1080p: core.cluster_rule.base_cluster_distance_1080p,
        base_cluster_area_1080p: core.cluster_rule.base_cluster_area_1080p,
        base_alignment_expansion_1080p: core.cluster_rule.base_alignment_expansion_1080p,
        base_edge_ignore_1080p: core.cluster_rule.base_edge_ignore_1080p,
        area_ratio_threshold: core.cluster_rule.area_ratio_threshold,
        prefer_right_when_scan_rounds_gt_one: core
            .cluster_rule
            .prefer_right_when_scan_rounds_gt_one,
        target_selection: core.cluster_rule.target_selection.clone(),
    };
    let alignment_rule = LinneaMiningAlignmentRule {
        max_inner_retry: core.alignment_rule.max_inner_retry,
        element_sight_refresh_ms: core.alignment_rule.element_sight_refresh_ms,
        refresh_release_ms: core.alignment_rule.refresh_release_ms,
        refresh_hold_ms: core.alignment_rule.refresh_hold_ms,
        aim_sensitivity_factor_x: core.alignment_rule.aim_sensitivity_factor_x,
        aim_sensitivity_factor_y: core.alignment_rule.aim_sensitivity_factor_y,
        aim_move_delay_ms: core.alignment_rule.aim_move_delay_ms,
        fallback_shot_on_last_successful_detection: core
            .alignment_rule
            .fallback_shot_on_last_successful_detection,
    };
    let scan_rule = LinneaMiningScanRule {
        middle_button_hold_ms: core.scan_rule.middle_button_hold_ms,
        middle_button_release_ms: core.scan_rule.middle_button_release_ms,
        compensate_detection_hold_ms: core.scan_rule.compensate_detection_hold_ms,
        compensate_move_wait_ms: core.scan_rule.compensate_move_wait_ms,
        left_turn_step_1080p: core.scan_rule.left_turn_step_1080p,
        left_turn_wait_ms: core.scan_rule.left_turn_wait_ms,
    };
    let mine_rule = LinneaMiningMineRule {
        compensate_up_pixels: core.mine_rule.compensate_up_pixels,
        compensate_up_wait_ms: core.mine_rule.compensate_up_wait_ms,
        attack_button: core.mine_rule.attack_button.clone(),
        after_attack_wait_ms: core.mine_rule.after_attack_wait_ms,
    };
    let cleanup_rule = LinneaMiningCleanupRule {
        leave_aiming_mode_action: core.cleanup_rule.leave_aiming_mode_action,
        middle_button_up: core.cleanup_rule.middle_button_up,
        clear_vision_drawings: core.cleanup_rule.clear_vision_drawings,
    };

    LinneaMiningExecutionPlan {
        task_key: LINNEA_MINING_TASK_KEY.to_string(),
        display_name: "莉奈娅挖矿".to_string(),
        port_state: TaskPortState::RuntimeScaffolded,
        executor_ready: false,
        action_code: core.action_code,
        raw_params: core.raw_params,
        mine_count: core.mine_count,
        scan_rounds: core.scan_rounds,
        prefer_right: core.prefer_right,
        avatar_rule: avatar_rule.clone(),
        aiming_rule,
        detection_rule,
        cluster_rule,
        alignment_rule: alignment_rule.clone(),
        scan_rule: scan_rule.clone(),
        mine_rule: mine_rule.clone(),
        cleanup_rule,
        steps: linnea_mining_steps(&avatar_rule, aiming_rule, &alignment_rule, &scan_rule, &mine_rule),
        notes:
            "LinneaMining preserves the legacy avatar requirement, BgiMine detection, clustering, aiming, scan, mine, compensation, and cleanup rules as a Rust common-job plan; live avatar switching, capture, ONNX inference, mouse input, and overlay execution remain pending."
                .to_string(),
    }
}

fn linnea_mining_steps(
    avatar_rule: &LinneaMiningAvatarRule,
    aiming_rule: LinneaMiningAimingRule,
    alignment_rule: &LinneaMiningAlignmentRule,
    scan_rule: &LinneaMiningScanRule,
    mine_rule: &LinneaMiningMineRule,
) -> Vec<LinneaMiningStep> {
    vec![
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningStepCondition::Always,
            "switch to Linnea before mining",
            LinneaMiningStepAction::SwitchAvatar {
                avatar_name: avatar_rule.avatar_name.clone(),
                wait_ms: avatar_rule.switch_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Setup,
            LinneaMiningStepCondition::Always,
            "enter aiming mode",
            LinneaMiningStepAction::ToggleAimingMode {
                action: aiming_rule.aiming_mode_action,
                wait_ms: aiming_rule.enter_aim_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::ScanLoop,
            LinneaMiningStepCondition::PerScanRound,
            "hold elemental sight before cluster detection",
            LinneaMiningStepAction::HoldElementalSight {
                hold_ms: scan_rule.middle_button_hold_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Detection,
            LinneaMiningStepCondition::PerScanRound,
            "detect nearest mineral cluster",
            LinneaMiningStepAction::DetectMineralCluster,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Alignment,
            LinneaMiningStepCondition::WhenClusterDetected,
            "align target with retry and elemental-sight refresh",
            LinneaMiningStepAction::AlignTarget {
                max_retries: alignment_rule.max_inner_retry,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Mine,
            LinneaMiningStepCondition::WhenAlignedOrFallback,
            "shoot mineral target",
            LinneaMiningStepAction::MineTarget {
                attack_button: mine_rule.attack_button.clone(),
                after_attack_wait_ms: mine_rule.after_attack_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Alignment,
            LinneaMiningStepCondition::WhenDetectionLost,
            "compensate camera after lost detection",
            LinneaMiningStepAction::CompensateLostDetection,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::ScanLoop,
            LinneaMiningStepCondition::BetweenScanRounds,
            "turn left before next scan round",
            LinneaMiningStepAction::TurnForNextScanRound {
                left_turn_step_1080p: scan_rule.left_turn_step_1080p,
                wait_ms: scan_rule.left_turn_wait_ms,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "leave aiming mode",
            LinneaMiningStepAction::ToggleAimingMode {
                action: aiming_rule.aiming_mode_action,
                wait_ms: 0,
            },
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "release elemental sight",
            LinneaMiningStepAction::ReleaseMiddleButton,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "clear mining overlay drawings",
            LinneaMiningStepAction::ClearVisionDrawings,
        ),
        LinneaMiningStep::new(
            LinneaMiningStepPhase::Cleanup,
            LinneaMiningStepCondition::Finally,
            "mark live executor pending",
            LinneaMiningStepAction::MarkNativePending {
                message: "capture, ONNX inference, avatar switching, mouse input, and overlay adapters are pending".to_string(),
            },
        ),
    ]
}

fn string_member(value: &Value, keys: impl IntoIterator<Item = &'static str>) -> Option<String> {
    keys.into_iter().find_map(|key| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn i32_member(value: &Value, keys: impl IntoIterator<Item = &'static str>) -> Option<i32> {
    keys.into_iter().find_map(|key| {
        value.get(key).and_then(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .or_else(|| value.as_str().and_then(|value| value.parse::<i32>().ok()))
        })
    })
}
