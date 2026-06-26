use crate::common_job::ScanPickDropsExecutionPlan;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[path = "auto_fight_detection_model.rs"]
mod detection_model;
#[path = "auto_fight_execution_model.rs"]
mod execution_model;

pub use detection_model::*;
pub use execution_model::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CombatCommandMethod {
    Skill,
    Burst,
    Attack,
    Charge,
    Wait,
    Ready,
    Check,
    Walk,
    W,
    A,
    S,
    D,
    Dash,
    Jump,
    MouseDown,
    MouseUp,
    Click,
    MoveBy,
    KeyDown,
    KeyUp,
    KeyPress,
    Scroll,
    Round,
}

impl CombatCommandMethod {
    pub fn from_code(code: &str) -> Option<Self> {
        match code.trim() {
            "skill" | "e" => Some(Self::Skill),
            "burst" | "q" => Some(Self::Burst),
            "attack" | "普攻" | "普通攻击" => Some(Self::Attack),
            "charge" | "重击" => Some(Self::Charge),
            "wait" | "after" | "等待" => Some(Self::Wait),
            "ready" | "完成" => Some(Self::Ready),
            "check" | "检测" => Some(Self::Check),
            "walk" | "行走" => Some(Self::Walk),
            "w" => Some(Self::W),
            "a" => Some(Self::A),
            "s" => Some(Self::S),
            "d" => Some(Self::D),
            "dash" | "冲刺" => Some(Self::Dash),
            "jump" | "j" | "跳跃" => Some(Self::Jump),
            "mousedown" => Some(Self::MouseDown),
            "mouseup" => Some(Self::MouseUp),
            "click" => Some(Self::Click),
            "moveby" => Some(Self::MoveBy),
            "keydown" => Some(Self::KeyDown),
            "keyup" => Some(Self::KeyUp),
            "keypress" => Some(Self::KeyPress),
            "scroll" | "verticalscroll" => Some(Self::Scroll),
            "round" => Some(Self::Round),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatCommandPlan {
    pub avatar: String,
    pub method: CombatCommandMethod,
    pub args: Vec<String>,
    pub activating_rounds: Vec<u32>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptPlan {
    pub name: String,
    pub path: Option<PathBuf>,
    pub avatar_names: Vec<String>,
    pub commands: Vec<CombatCommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptBagPlan {
    pub source_path: PathBuf,
    pub scripts: Vec<CombatScriptPlan>,
    pub parse_failures: Vec<CombatScriptParseFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptParseFailure {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptMatchPlan {
    pub script_name: String,
    pub script_path: Option<PathBuf>,
    pub matched_avatar_count: usize,
    pub full_match: bool,
    pub commands: Vec<CombatCommandPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatScriptTeamSelectionStatus {
    NoTeamContext,
    FullMatch,
    PartialFallback,
    NoMatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatScriptTeamSelectionPlan {
    pub status: CombatScriptTeamSelectionStatus,
    pub team_avatar_names: Vec<String>,
    pub script_name: Option<String>,
    pub script_path: Option<PathBuf>,
    pub matched_avatar_count: usize,
    pub full_match: bool,
    pub command_avatar_names: Vec<String>,
    pub executable_avatar_names: Vec<String>,
    pub filtered_out_avatar_names: Vec<String>,
    pub executable_commands: Vec<CombatCommandPlan>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamAvatarPlan {
    pub index: usize,
    pub name: String,
    pub id: String,
    pub name_en: String,
    pub weapon: String,
    pub skill_cd_seconds: Option<f64>,
    pub skill_hold_cd_seconds: Option<f64>,
    pub burst_cd_seconds: Option<f64>,
    pub manual_skill_cd_seconds: f64,
    pub action_scheduler_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamPlan {
    pub avatars: Vec<CombatTeamAvatarPlan>,
    pub command_avatar_names: Vec<String>,
    pub can_be_skipped_avatar_names: Vec<String>,
    pub all_command_avatars_can_be_skipped: bool,
}

pub const EXPECTED_COMBAT_TEAM_AVATAR_COUNT: usize = 4;
pub const CURRENT_COMBAT_AVATAR_NAME: &str = "当前角色";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatFightLoopStepPhase {
    Setup,
    LoopStart,
    BeforeCommand,
    Command,
    AfterCommand,
    FinishDetection,
    Cleanup,
    Loot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatFightLoopStepKind {
    InitializeCancellation,
    StartExperienceDetector,
    WaitAllConfiguredSkillCooldowns,
    EnsureGuardianSkill,
    InitialSeekEnemy,
    SkipGuardianCommand,
    SkipCommandBySkillCooldown,
    EnforceTimeout,
    CheckBeforeBurst,
    ExecuteCommand,
    CountFightAvatarSwitch,
    CheckCommandFinish,
    FastCheckAfterAvatar,
    ReleaseAllKeys,
    StopExperienceDetector,
    ApplyBattleThresholdForLoot,
    KazuhaOrJeanPickup,
    SwitchPickupParty,
    ScanPickDrops,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatFightLoopStepPlan {
    pub phase: CombatFightLoopStepPhase,
    pub kind: CombatFightLoopStepKind,
    pub command_index: Option<usize>,
    pub avatar: Option<String>,
    pub enabled: bool,
    pub requires_native_context: Vec<CombatExecutionContextRequirement>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatFightLoopPlan {
    pub timeout_seconds: i32,
    pub command_count: usize,
    pub executable_command_count: usize,
    pub fight_finish_detect_enabled: bool,
    pub rotate_find_enemy_enabled: bool,
    pub check_before_burst_enabled: bool,
    pub guardian_enabled: bool,
    pub guardian_avatar_index: Option<usize>,
    pub guardian_avatar_name: Option<String>,
    pub kazuha_pickup_enabled: bool,
    pub pickup_drops_after_fight_enabled: bool,
    pub scan_pick_drops_plan: Option<ScanPickDropsExecutionPlan>,
    pub exp_based_pickup_enabled: bool,
    pub battle_threshold_for_loot: i32,
    pub steps: Vec<CombatFightLoopStepPlan>,
    pub native_dispatch_ready: bool,
}
