use crate::{Result, TaskError};
use bgi_input::InputEvent;
use bgi_vision::{BgrImage, Rect, RgbPixel};
use serde::{Deserialize, Serialize};

use super::CombatTeamPlan;

pub const AUTO_FIGHT_FINISH_PROGRESS_PIXEL: (u32, u32) = (790, 50);
pub const AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL: (u32, u32) = (768, 50);
pub const AUTO_FIGHT_DEFAULT_FINISH_DELAY_MS: u64 = 1_500;
pub const AUTO_FIGHT_DEFAULT_FINISH_DETECT_DELAY_MS: u64 = 450;
pub const AUTO_FIGHT_AVATAR_INDEX_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1859, 256, 28, 24),
    (1859, 352, 28, 24),
    (1859, 448, 28, 24),
    (1859, 544, 28, 24),
];
pub const AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS: [&str; 4] =
    ["index_1.png", "index_2.png", "index_3.png", "index_4.png"];
pub const AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ROI_1080P: (i32, i32, i32, i32) = (1855, 155, 35, 600);
pub const AUTO_FIGHT_AVATAR_INDEX_DISTANCE_Y_1080P: i32 = 96;
pub const AUTO_FIGHT_AVATAR_SIDE_ICON_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1765, 225, 76, 76),
    (1765, 315, 76, 76),
    (1765, 410, 76, 76),
    (1765, 500, 76, 76),
];
pub const AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P: [(i32, i32, i32, i32); 4] = [
    (1584, 216, 64, 84),
    (1584, 316, 64, 84),
    (1584, 416, 64, 84),
    (1584, 516, 64, 84),
];
pub const AUTO_FIGHT_AVATAR_SIDE_ICON_FROM_INDEX_RECT_1080P: (i32, i32, i32, i32) =
    (-91, -47, 82, 82);
pub const AUTO_FIGHT_COOP_ONE_P_ASSET: &str = "1p.png";
pub const AUTO_FIGHT_COOP_P_ASSET: &str = "p.png";
pub const AUTO_FIGHT_FEATURE: &str = "AutoFight";
pub const AUTO_FIGHT_COOP_SIDE_INDEX_RECTS_1080P: [(&str, &[(i32, i32, i32, i32)]); 6] = [
    ("1p_2", &[(1859, 412, 28, 24), (1859, 508, 28, 24)]),
    ("1p_3", &[(1859, 459, 28, 24), (1859, 555, 28, 24)]),
    ("1p_4", &[(1859, 552, 28, 24)]),
    ("p_2", &[(1859, 412, 28, 24), (1859, 508, 28, 24)]),
    ("p_3", &[(1859, 412, 28, 24)]),
    ("p_4", &[(1859, 507, 28, 24)]),
];
pub const AUTO_FIGHT_COOP_SIDE_ICON_RECTS_1080P: [(&str, &[(i32, i32, i32, i32)]); 6] = [
    ("1p_2", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("1p_3", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("1p_4", &[(1765, 515, 76, 76)]),
    ("p_2", &[(1765, 375, 76, 76), (1765, 470, 76, 76)]),
    ("p_3", &[(1765, 475, 76, 76)]),
    ("p_4", &[(1765, 515, 76, 76)]),
];
pub const AUTO_FIGHT_CURRENT_AVATAR_FEATURE: &str = "Common/Element";
pub const AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET: &str = "current_avatar_threshold.png";
pub const AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ROI_1080P: (i32, i32, i32, i32) =
    (1680, 155, 210, 600);
pub const AUTO_FIGHT_CURRENT_AVATAR_FLAG_TO_INDEX_RECT_1080P: (i32, i32, i32, i32) =
    (126, -194, 16, 17);
pub const AUTO_FIGHT_ACTIVE_SKILL_COOLDOWN_RECT_1080P: (i32, i32, i32, i32) = (1688, 988, 22, 12);
pub const AUTO_FIGHT_ACTIVE_BURST_COOLDOWN_RECT_1080P: (i32, i32, i32, i32) = (1809, 968, 30, 15);
pub const AUTO_FIGHT_SIDE_BURST_MIN_RADIUS_1080P: i32 = 25;
pub const AUTO_FIGHT_SIDE_BURST_MAX_RADIUS_1080P: i32 = 34;
pub const AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES: usize = 25;
pub const AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionResult {
    pub finished: bool,
    pub progress_pixel: RgbPixel,
    pub white_tile_pixel: RgbPixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFightFinishDetectionStepKind {
    PreDetectDelay,
    SeekEnemy,
    OpenPartySetup,
    WaitForPartySetup,
    CaptureFrame,
    SampleFinishPixels,
    DropFromPartySetup,
    CancelPartySwitchWhenFinished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionStepPlan {
    pub kind: AutoFightFinishDetectionStepKind,
    pub enabled: bool,
    pub input_events: Vec<InputEvent>,
    pub delay_ms: u64,
    pub requires_capture: bool,
    pub requires_vision: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionPlan {
    pub pre_detect_delay_ms: u64,
    pub detect_delay_ms: u64,
    pub rotate_find_enemy_enabled: bool,
    pub progress_pixel: (u32, u32),
    pub white_tile_pixel: (u32, u32),
    pub steps: Vec<AutoFightFinishDetectionStepPlan>,
    pub native_ready_without_capture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoFightFinishDetectionExecutionMode {
    PlanOnly,
    SendInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionExecution {
    pub mode: AutoFightFinishDetectionExecutionMode,
    pub plan: AutoFightFinishDetectionPlan,
    pub detection: AutoFightFinishDetectionResult,
    pub before_capture_events: Vec<InputEvent>,
    pub after_detection_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoFightFinishDetectionLiveExecution {
    pub mode: AutoFightFinishDetectionExecutionMode,
    pub plan: AutoFightFinishDetectionPlan,
    pub detection: Option<AutoFightFinishDetectionResult>,
    pub before_capture_events: Vec<InputEvent>,
    pub after_detection_events: Vec<InputEvent>,
    pub dispatched: bool,
    pub dispatched_events: usize,
    pub cancelled: bool,
    pub captured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatActiveAvatarDetectionMethod {
    SingleAvatar,
    WhiteRectMajority,
    EdgeWhiteRatio,
    ImageDifferenceVote,
    ArrowTemplate,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatActiveAvatarDetectionResult {
    pub active_index: Option<usize>,
    pub method: CombatActiveAvatarDetectionMethod,
    pub rects: Vec<Rect>,
    pub white_rect_count: usize,
    pub not_white_rect_index: Option<usize>,
    pub edge_white_ratios: Vec<f64>,
    pub difference_votes: Vec<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillReadinessKind {
    ElementalSkill,
    ElementalBurst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatSkillReadinessStatus {
    Ready,
    CooldownOrUnavailable,
    UnsupportedForInactiveAvatar,
    ActiveAvatarUnresolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatSideBurstCircleDetection {
    pub rect: Rect,
    pub detected: bool,
    pub edge_pixel_count: usize,
    pub best_center: Option<(i32, i32)>,
    pub best_radius: Option<i32>,
    pub best_votes: usize,
    pub required_votes: usize,
    pub sampled_points: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatSkillReadinessDetection {
    pub kind: CombatSkillReadinessKind,
    pub requested_index: usize,
    pub active_index: Option<usize>,
    pub status: CombatSkillReadinessStatus,
    pub ready: Option<bool>,
    pub active_detection: CombatActiveAvatarDetectionResult,
    pub cooldown_rect: Option<Rect>,
    pub white_component_count: usize,
    pub legacy_connected_component_labels: usize,
    pub side_burst_rect: Option<Rect>,
    pub side_burst_circle: Option<CombatSideBurstCircleDetection>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarIndexRectsDetection {
    pub rects_by_index: Vec<Option<Rect>>,
    pub resolved_rects: Vec<Rect>,
    pub inferred_from_current_avatar_arrow: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMultiGameStatus {
    pub is_in_multi_game: bool,
    pub is_host: bool,
    pub player_count: usize,
}

impl Default for CombatMultiGameStatus {
    fn default() -> Self {
        Self {
            is_in_multi_game: false,
            is_host: false,
            player_count: 1,
        }
    }
}

impl CombatMultiGameStatus {
    pub fn max_control_avatar_count(self) -> Result<usize> {
        if !self.is_in_multi_game {
            return Ok(4);
        }
        match (self.is_host, self.player_count) {
            (true, 1) => Ok(4),
            (true, 2 | 3) => Ok(2),
            (true, 4) => Ok(1),
            (false, 2) => Ok(2),
            (false, 3 | 4) => Ok(1),
            (true, _) => Err(TaskError::VisionPlan(format!(
                "invalid host co-op player count: {}",
                self.player_count
            ))),
            (false, _) => Err(TaskError::VisionPlan(format!(
                "invalid guest co-op player count: {}",
                self.player_count
            ))),
        }
    }

    pub fn rect_map_key(self) -> Option<String> {
        self.is_in_multi_game.then(|| {
            format!(
                "{}_{}",
                if self.is_host { "1p" } else { "p" },
                self.player_count
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatMultiGameDetection {
    pub status: CombatMultiGameStatus,
    pub p_icon_count: usize,
    pub one_p_icon_found: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarSideClassification {
    pub class_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatAvatarSideRecognition {
    pub index: usize,
    pub avatar_name: String,
    pub name_en: String,
    pub costume_name: Option<String>,
    pub display_name: String,
    pub confidence: f32,
    pub index_rect: Rect,
    pub side_icon_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatTeamRecognitionExecution {
    pub index_rect_detection: CombatAvatarIndexRectsDetection,
    pub avatars: Vec<CombatAvatarSideRecognition>,
    pub team_avatar_names: Vec<String>,
    pub team_plan: CombatTeamPlan,
}

pub trait CombatAvatarSideClassifier {
    fn classify_avatar_side(
        &mut self,
        index: usize,
        image: &BgrImage,
        side_icon_rect: Rect,
    ) -> Result<CombatAvatarSideClassification>;
}

impl<F> CombatAvatarSideClassifier for F
where
    F: FnMut(usize, &BgrImage, Rect) -> Result<CombatAvatarSideClassification>,
{
    fn classify_avatar_side(
        &mut self,
        index: usize,
        image: &BgrImage,
        side_icon_rect: Rect,
    ) -> Result<CombatAvatarSideClassification> {
        self(index, image, side_icon_rect)
    }
}
