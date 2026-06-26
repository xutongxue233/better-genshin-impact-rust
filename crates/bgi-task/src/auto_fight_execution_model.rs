use bgi_core::GenshinAction;
use bgi_input::{InputEvent, MouseButton};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::CombatCommandPlan;

pub const COMBAT_ATTACK_INTERVAL_MILLISECONDS: u64 = 200;
pub const COMBAT_DEFAULT_DASH_MILLISECONDS: u64 = 200;
pub const COMBAT_DEFAULT_CHARGE_MILLISECONDS: u64 = 1_000;
pub const COMBAT_READY_INITIAL_DELAY_MILLISECONDS: u64 = 10;
pub const COMBAT_READY_POLL_COUNT: u32 = 20;
pub const COMBAT_READY_POLL_INTERVAL_MILLISECONDS: u64 = 150;
pub const COMBAT_AVATAR_SWITCH_SETTLE_MILLISECONDS: u64 = 250;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatAvatarSwitchPolicy {
    CurrentAvatar,
    SwitchOnAvatarChange,
    EnsureSelectedBeforeAction,
    NoSwitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillCooldownPolicy {
    None,
    WaitUntilReady,
    FastSkipIfCoolingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillExecutionVariant {
    Tap,
    GenericHold,
    NahidaCameraSweepHold,
    CandaceLongHold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatChargeExecutionVariant {
    GenericHold,
    NeuvilletteCameraSweep,
    ChascaCameraSweep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMouseButtonPlan {
    pub raw: String,
    pub button: MouseButton,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatVirtualKeyPlan {
    pub raw: String,
    pub vk: u16,
    pub mouse_button: Option<MouseButton>,
    pub mapped_action: Option<GenshinAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum CombatCommandActionPlan {
    Skill {
        hold: bool,
        variant: CombatSkillExecutionVariant,
        cooldown_policy: CombatSkillCooldownPolicy,
        options: Vec<String>,
    },
    Burst {
        requires_readiness_check: bool,
    },
    Attack {
        duration_ms: u64,
        click_interval_ms: u64,
        repeat_count: u32,
    },
    Charge {
        duration_ms: u64,
        variant: CombatChargeExecutionVariant,
    },
    Walk {
        direction: String,
        duration_ms: u64,
    },
    Wait {
        duration_ms: u64,
    },
    Ready {
        initial_delay_ms: u64,
        poll_count: u32,
        poll_interval_ms: u64,
    },
    Check {
        handled_by_fight_loop: bool,
    },
    Dash {
        duration_ms: u64,
    },
    Jump,
    MouseDown {
        button: CombatMouseButtonPlan,
    },
    MouseUp {
        button: CombatMouseButtonPlan,
    },
    Click {
        button: CombatMouseButtonPlan,
    },
    MoveBy {
        x: i32,
        y: i32,
    },
    KeyDown {
        key: CombatVirtualKeyPlan,
    },
    KeyUp {
        key: CombatVirtualKeyPlan,
    },
    KeyPress {
        key: CombatVirtualKeyPlan,
    },
    Scroll {
        clicks: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandExecutionPlan {
    pub index: usize,
    pub command: CombatCommandPlan,
    pub switch_policy: CombatAvatarSwitchPolicy,
    pub action: CombatCommandActionPlan,
    pub default_input_events: Vec<InputEvent>,
    pub requires_combat_context: bool,
    pub static_input_ready: bool,
    pub pending_context: Vec<CombatExecutionContextRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptExecutionPlan {
    pub name: String,
    pub path: Option<PathBuf>,
    pub avatar_names: Vec<String>,
    pub commands: Vec<CombatCommandExecutionPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombatExecutionContextRequirement {
    AvatarSelection,
    SkillCooldown,
    BurstReadiness,
    FightLoopFinishDetection,
    ReadyStateDetection,
    CharacterSpecificCameraControl,
    InputEvents,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlaybackEvaluation {
    pub script_name: String,
    pub script_path: Option<PathBuf>,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub default_input_event_count: usize,
    pub first_blocking_command_index: Option<usize>,
    pub first_blocking_requirements: Vec<CombatExecutionContextRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlaybackBatchEvaluation {
    pub scripts: Vec<CombatScriptPlaybackEvaluation>,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub default_input_event_count: usize,
    pub dispatch_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatCommandPlaybackMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandPlaybackExecution {
    pub mode: CombatCommandPlaybackMode,
    pub script_name: String,
    pub total_commands: usize,
    pub static_ready_commands: usize,
    pub context_bound_commands: usize,
    pub input_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTeamPlaybackCommandPlan {
    pub command_index: usize,
    pub avatar: String,
    pub team_index: Option<usize>,
    pub switch_events: Vec<InputEvent>,
    pub action_events: Vec<InputEvent>,
    pub input_events: Vec<InputEvent>,
    pub resolved_context: Vec<CombatExecutionContextRequirement>,
    pub pending_context: Vec<CombatExecutionContextRequirement>,
    pub executable: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTeamPlaybackExecution {
    pub mode: CombatCommandPlaybackMode,
    pub script_name: String,
    pub total_commands: usize,
    pub candidate_commands: usize,
    pub planned_commands: Vec<CombatTeamPlaybackCommandPlan>,
    pub playable_commands: usize,
    pub blocked_command_index: Option<usize>,
    pub blocked_requirements: Vec<CombatExecutionContextRequirement>,
    pub input_events: Vec<InputEvent>,
    pub dispatch_ready: bool,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}
