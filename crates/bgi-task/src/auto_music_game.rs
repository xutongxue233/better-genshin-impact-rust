use bgi_core::AutoMusicGameConfig;
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, TaskError};

pub const AUTO_MUSIC_GAME_TASK_KEY: &str = "AutoMusicGame";
pub const AUTO_MUSIC_GAME_DISPLAY_NAME: &str = "自动音游";
pub const AUTO_MUSIC_ALBUM_DISPLAY_NAME: &str = "自动音游专辑";
pub const AUTO_MUSIC_GAME_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_MUSIC_GAME_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_MUSIC_GAME_SAMPLE_Y_1080P: i32 = 921;
pub const AUTO_MUSIC_GAME_POLL_INTERVAL_MS: u64 = 5;
pub const AUTO_MUSIC_GAME_BLUE_PRESS_THRESHOLD: u8 = 220;
pub const AUTO_MUSIC_GAME_BLUE_RELEASE_THRESHOLD: u8 = 220;
pub const AUTO_MUSIC_GAME_DEFAULT_MUSIC_LEVEL: &str = "传说";
pub const AUTO_MUSIC_GAME_ALL_LEVELS: &str = "所有";
pub const AUTO_MUSIC_GAME_SONGS_PER_DIFFICULTY_LOOP_COUNT: u64 = 13;
pub const AUTO_MUSIC_GAME_ALBUM_CHECK_INTERVAL_MS: u64 = 5_000;

pub const AUTO_MUSIC_UI_LEFT_TOP_ALBUM_ICON_ASSET: &str =
    "AutoMusicGame:ui_left_top_album_icon.png";
pub const AUTO_MUSIC_BTN_PAUSE_ASSET: &str = "AutoMusicGame:btn_pause.png";
pub const AUTO_MUSIC_ALBUM_COMPLETE_ASSET: &str = "AutoMusicGame:album_music_complate.png";
pub const AUTO_MUSIC_BTN_LIST_ASSET: &str = "AutoMusicGame:btn_list.png";
pub const AUTO_MUSIC_CANORUS_ASSET: &str = "AutoMusicGame:music_canorus.png";

const AUTO_MUSIC_GAME_KEY_LANES: &[(&str, i32)] = &[
    ("A", 417),
    ("S", 628),
    ("D", 844),
    ("J", 1061),
    ("K", 1277),
    ("L", 1493),
];

const AUTO_MUSIC_GAME_DIFFICULTIES: &[(&str, i32, i32, &str)] = &[
    ("普通", 480, 600, "MusicCanorusLevel1"),
    ("困难", 800, 600, "MusicCanorusLevel2"),
    ("大师", 1150, 600, "MusicCanorusLevel3"),
    ("传说", 1400, 600, "MusicCanorusLevel4"),
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoMusicGameExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub album_display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub config_rule: AutoMusicGameConfigRule,
    pub startup_rule: AutoMusicGameStartupRule,
    pub performance_rule: AutoMusicGamePerformanceRule,
    pub key_lanes: Vec<AutoMusicGameKeyLane>,
    pub album_rule: AutoMusicAlbumRule,
    pub locators: AutoMusicGameLocators,
    pub steps: Vec<AutoMusicGameStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoMusicGameExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub auto_music_game_config: AutoMusicGameConfig,
}

impl Default for AutoMusicGameExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_MUSIC_GAME_DEFAULT_CAPTURE_WIDTH,
                AUTO_MUSIC_GAME_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            auto_music_game_config: AutoMusicGameConfig::default(),
        }
    }
}

impl AutoMusicGameExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };

        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        if let Some(asset_scale) = f64_member(value, ["assetScale", "AssetScale", "asset_scale"]) {
            config.asset_scale = asset_scale.max(0.0);
        }

        let auto_music_value = value
            .get("autoMusicGameConfig")
            .or_else(|| value.get("AutoMusicGameConfig"))
            .or_else(|| value.get("auto_music_game_config"))
            .unwrap_or(value);
        config.auto_music_game_config =
            serde_json::from_value(auto_music_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicGameConfigRule {
    pub must_canorus_level: bool,
    pub configured_music_level: String,
    pub normalized_music_level: String,
    pub empty_music_level_defaults_to: String,
    pub all_music_level_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicGameStartupRule {
    pub checks_game_resolution: bool,
    pub logs_close_task_reminder: bool,
    pub warns_default_style_unusable: bool,
    pub default_style_name: String,
    pub required_hutao_style_name: String,
    pub required_coin_count: u64,
    pub task_runner_skips_main_ui_wait: bool,
    pub releases_all_keys_on_finish: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicGamePerformanceRule {
    pub uses_six_parallel_lane_tasks: bool,
    pub converts_1080p_points_to_game_capture_region: bool,
    pub win32_get_pixel_source: String,
    pub poll_interval_ms: u64,
    pub press_when_blue_below: u8,
    pub release_when_blue_greater_or_equal: u8,
    pub holds_key_until_release_threshold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicGameKeyLane {
    pub key: String,
    pub x_1080p: i32,
    pub y_1080p: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicLaneState {
    pub key_down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicLaneAction {
    None,
    KeyDown,
    KeyUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicLaneSampleDecision {
    pub key: String,
    pub blue: u8,
    pub action: AutoMusicLaneAction,
    pub key_down_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicLaneBlueSample {
    pub key: String,
    pub blue: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicPerformanceFrame {
    pub lane_blues: Vec<AutoMusicLaneBlueSample>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicPerformanceStopReason {
    SamplesExhausted,
    CancelledBeforeFrame,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoMusicPerformanceEvent {
    LaneKeyDown {
        frame_index: usize,
        key: String,
        blue: u8,
    },
    LaneKeyUp {
        frame_index: usize,
        key: String,
        blue: u8,
    },
    PollDelay {
        frame_index: usize,
        duration_ms: u64,
    },
    ReleaseAllKeys {
        held_keys_before_release: Vec<String>,
        reason: AutoMusicPerformanceStopReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicPerformanceReport {
    pub task_key: String,
    pub stop_reason: AutoMusicPerformanceStopReason,
    pub frames_processed: usize,
    pub held_keys_before_release: Vec<String>,
    pub events: Vec<AutoMusicPerformanceEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicAlbumPageStatus {
    ThemeAlbum,
    NotAlbumPage,
    AllSongsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicAlbumExecutionStatus {
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicAlbumCompletionMode {
    CanorusLevel,
    AllRewards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicAlbumSkipReason {
    CanorusLevelComplete,
    AllRewardsComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoMusicAlbumConfirmPhase {
    EnterSong,
    StartPerformance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum AutoMusicAlbumEvent {
    AlbumPageChecked {
        status: AutoMusicAlbumPageStatus,
    },
    DifficultyStarted {
        difficulty: String,
    },
    SongCompletionChecked {
        difficulty: String,
        song_index: u64,
        mode: AutoMusicAlbumCompletionMode,
        completed: bool,
    },
    SongSkipped {
        difficulty: String,
        song_index: u64,
        reason: AutoMusicAlbumSkipReason,
    },
    WhiteConfirmClicked {
        difficulty: String,
        song_index: u64,
        phase: AutoMusicAlbumConfirmPhase,
    },
    DifficultySelected {
        difficulty: String,
        song_index: u64,
    },
    PerformanceCompleted {
        difficulty: String,
        song_index: u64,
        stop_reason: AutoMusicPerformanceStopReason,
        frames_processed: usize,
    },
    AlbumPageWaited {
        difficulty: String,
        song_index: u64,
    },
    NextSongClicked {
        difficulty: String,
        song_index: u64,
    },
    Delay {
        difficulty: Option<String>,
        song_index: Option<u64>,
        duration_ms: u64,
    },
    DifficultyCompleted {
        difficulty: String,
    },
    AlbumCompleted,
    Cancelled {
        difficulty: Option<String>,
        song_index: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoMusicAlbumExecutionReport {
    pub task_key: String,
    pub status: AutoMusicAlbumExecutionStatus,
    pub difficulty_count: usize,
    pub songs_checked: u64,
    pub skipped_songs: u64,
    pub performed_songs: u64,
    pub events: Vec<AutoMusicAlbumEvent>,
}

pub trait AutoMusicPerformanceRuntime {
    fn is_auto_music_performance_cancelled(&mut self) -> Result<bool>;

    fn next_auto_music_performance_frame(&mut self) -> Result<Option<AutoMusicPerformanceFrame>>;

    fn auto_music_key_down(&mut self, key: &str) -> Result<()>;

    fn auto_music_key_up(&mut self, key: &str) -> Result<()>;

    fn delay_auto_music_poll(&mut self, duration_ms: u64) -> Result<()>;

    fn release_all_auto_music_keys(&mut self, held_keys_before_release: &[String]) -> Result<()>;
}

pub trait AutoMusicAlbumRuntime {
    fn is_auto_music_album_cancelled(&mut self) -> Result<bool>;

    fn check_auto_music_album_page(
        &mut self,
        icon_locator: &AutoMusicTemplateLocator,
    ) -> Result<AutoMusicAlbumPageStatus>;

    fn is_auto_music_song_completed(&mut self, locator: &AutoMusicTemplateLocator) -> Result<bool>;

    fn click_auto_music_next_song(&mut self, x_1080p: i32, y_1080p: i32) -> Result<()>;

    fn click_auto_music_white_confirm(&mut self) -> Result<()>;

    fn click_auto_music_difficulty(&mut self, difficulty: &AutoMusicDifficultyRule) -> Result<()>;

    fn delay_auto_music_album(&mut self, duration_ms: u64) -> Result<()>;

    fn execute_auto_music_song(
        &mut self,
        difficulty: &AutoMusicDifficultyRule,
        song_index: u64,
    ) -> Result<AutoMusicPerformanceReport>;

    fn wait_auto_music_album_page(&mut self, icon_locator: &AutoMusicTemplateLocator)
        -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoMusicAlbumRule {
    pub selected_music_level: String,
    pub selected_difficulties: Vec<AutoMusicDifficultyRule>,
    pub default_difficulties: Vec<AutoMusicDifficultyRule>,
    pub songs_per_difficulty_loop_count: u64,
    pub checks_album_icon_before_start: bool,
    pub rejects_all_songs_page_by_ocr: bool,
    pub album_start_notification: String,
    pub album_end_notification: String,
    pub canorus_level_skip_when_enabled: bool,
    pub complete_reward_skip_when_canorus_disabled: bool,
    pub next_song_click_x_1080p: i32,
    pub next_song_click_y_1080p: i32,
    pub after_next_song_sleep_ms: u64,
    pub select_difficulty_sleep_ms: u64,
    pub after_confirm_sleep_ms: u64,
    pub after_start_performance_sleep_ms: u64,
    pub album_check_interval_ms: u64,
    pub completion_check_uses_btn_list: bool,
    pub after_song_finished_sleep_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicDifficultyRule {
    pub name: String,
    pub click_x_1080p: i32,
    pub click_y_1080p: i32,
    pub canorus_locator_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoMusicGameLocators {
    pub ui_left_top_album_icon: AutoMusicTemplateLocator,
    pub btn_pause: AutoMusicTemplateLocator,
    pub album_music_complete: AutoMusicTemplateLocator,
    pub btn_list: AutoMusicTemplateLocator,
    pub music_canorus_levels: Vec<AutoMusicTemplateLocator>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoMusicTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub roi_rule: Option<String>,
    pub threshold: Option<f64>,
    pub match_mode: TemplateMatchMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoMusicGameStep {
    pub phase: AutoMusicGamePhase,
    pub action: AutoMusicGameAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoMusicGamePhase {
    Startup,
    Performance,
    Album,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoMusicGameAction {
    CheckResolutionAndLogWarnings,
    ConvertLaneCoordinates,
    SpawnLanePixelLoops,
    PressKeyWhenPixelDarkens,
    ReleaseKeyWhenPixelBrightens,
    CheckAlbumPage,
    SkipCompletedSongOrSelectDifficulty,
    RunPerformanceUntilListButtonAppears,
    ReleaseAllKeys,
}

pub fn plan_auto_music_game(config: AutoMusicGameExecutionConfig) -> AutoMusicGameExecutionPlan {
    let configured_music_level = config.auto_music_game_config.music_level.clone();
    let normalized_music_level = if configured_music_level.trim().is_empty() {
        AUTO_MUSIC_GAME_DEFAULT_MUSIC_LEVEL.to_string()
    } else {
        configured_music_level.trim().to_string()
    };

    AutoMusicGameExecutionPlan {
        task_key: AUTO_MUSIC_GAME_TASK_KEY.to_string(),
        display_name: AUTO_MUSIC_GAME_DISPLAY_NAME.to_string(),
        album_display_name: AUTO_MUSIC_ALBUM_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        config_rule: AutoMusicGameConfigRule {
            must_canorus_level: config.auto_music_game_config.must_canorus_level,
            configured_music_level,
            normalized_music_level: normalized_music_level.clone(),
            empty_music_level_defaults_to: AUTO_MUSIC_GAME_DEFAULT_MUSIC_LEVEL.to_string(),
            all_music_level_value: AUTO_MUSIC_GAME_ALL_LEVELS.to_string(),
        },
        startup_rule: AutoMusicGameStartupRule {
            checks_game_resolution: true,
            logs_close_task_reminder: true,
            warns_default_style_unusable: true,
            default_style_name: "轻漾涟漪".to_string(),
            required_hutao_style_name: "疏影引蝶映梅红".to_string(),
            required_coin_count: 600,
            task_runner_skips_main_ui_wait: true,
            releases_all_keys_on_finish: true,
        },
        performance_rule: AutoMusicGamePerformanceRule {
            uses_six_parallel_lane_tasks: true,
            converts_1080p_points_to_game_capture_region: true,
            win32_get_pixel_source: "User32.GetDC + Gdi32.GetPixel".to_string(),
            poll_interval_ms: AUTO_MUSIC_GAME_POLL_INTERVAL_MS,
            press_when_blue_below: AUTO_MUSIC_GAME_BLUE_PRESS_THRESHOLD,
            release_when_blue_greater_or_equal: AUTO_MUSIC_GAME_BLUE_RELEASE_THRESHOLD,
            holds_key_until_release_threshold: true,
        },
        key_lanes: auto_music_key_lanes(),
        album_rule: AutoMusicAlbumRule {
            selected_music_level: normalized_music_level.clone(),
            selected_difficulties: selected_difficulties(&normalized_music_level),
            default_difficulties: default_difficulties(),
            songs_per_difficulty_loop_count: AUTO_MUSIC_GAME_SONGS_PER_DIFFICULTY_LOOP_COUNT,
            checks_album_icon_before_start: true,
            rejects_all_songs_page_by_ocr: true,
            album_start_notification: "AlbumStart".to_string(),
            album_end_notification: "AlbumEnd".to_string(),
            canorus_level_skip_when_enabled: config.auto_music_game_config.must_canorus_level,
            complete_reward_skip_when_canorus_disabled: !config
                .auto_music_game_config
                .must_canorus_level,
            next_song_click_x_1080p: 310,
            next_song_click_y_1080p: 220,
            after_next_song_sleep_ms: 800,
            select_difficulty_sleep_ms: 200,
            after_confirm_sleep_ms: 800,
            after_start_performance_sleep_ms: 500,
            album_check_interval_ms: AUTO_MUSIC_GAME_ALBUM_CHECK_INTERVAL_MS,
            completion_check_uses_btn_list: true,
            after_song_finished_sleep_ms: 2_000,
        },
        locators: auto_music_locators(),
        steps: auto_music_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop manual performance live command and generic independent-task live route now cover 16:9 resolution check, game-window handle/metrics, 1080p-to-capture coordinate conversion, Win32 User32.GetDC/Gdi32.GetPixel lane sampling, concurrent cancellation, Simulation keyboard KeyDown/KeyUp dispatch, and ReleaseAllKeys cleanup".to_string(),
            "injectable AutoAlbumTask executor plus desktop album live command now preserve album page validation, all-songs OCR rejection, completion skipping, difficulty selection, white-confirm/template/click IO, BtnList performance stop handoff, next-song navigation, cancellation, timing, and AlbumStart/AlbumEnd/AlbumError notification dispatch; generic independent task live route selects performance by default or album with mode/executionMode=album".to_string(),
            "full legacy task-runner hotkey retirement, real-game regression notes, and non-Windows GetPixel fallback remain pending".to_string(),
        ],
    }
}

pub fn update_auto_music_lane_state(
    state: &mut AutoMusicLaneState,
    lane: &AutoMusicGameKeyLane,
    blue: u8,
    performance_rule: &AutoMusicGamePerformanceRule,
) -> AutoMusicLaneSampleDecision {
    let action = if !state.key_down && blue < performance_rule.press_when_blue_below {
        state.key_down = true;
        AutoMusicLaneAction::KeyDown
    } else if state.key_down && blue >= performance_rule.release_when_blue_greater_or_equal {
        state.key_down = false;
        AutoMusicLaneAction::KeyUp
    } else {
        AutoMusicLaneAction::None
    };

    AutoMusicLaneSampleDecision {
        key: lane.key.clone(),
        blue,
        action,
        key_down_after: state.key_down,
    }
}

pub fn auto_music_lane_sample_decisions(
    lane: &AutoMusicGameKeyLane,
    blue_samples: &[u8],
    performance_rule: &AutoMusicGamePerformanceRule,
) -> Vec<AutoMusicLaneSampleDecision> {
    let mut state = AutoMusicLaneState::default();
    blue_samples
        .iter()
        .copied()
        .map(|blue| update_auto_music_lane_state(&mut state, lane, blue, performance_rule))
        .collect()
}

pub fn execute_auto_music_performance_samples(
    plan: &AutoMusicGameExecutionPlan,
    frames: &[AutoMusicPerformanceFrame],
    cancel_before_frame: Option<usize>,
) -> AutoMusicPerformanceReport {
    let mut lane_states = plan
        .key_lanes
        .iter()
        .map(|lane| (lane, AutoMusicLaneState::default()))
        .collect::<Vec<_>>();
    let mut events = Vec::new();
    let mut frames_processed = 0;
    let mut stop_reason = AutoMusicPerformanceStopReason::SamplesExhausted;

    for (frame_index, frame) in frames.iter().enumerate() {
        if cancel_before_frame == Some(frame_index) {
            stop_reason = AutoMusicPerformanceStopReason::CancelledBeforeFrame;
            break;
        }

        for (lane, state) in &mut lane_states {
            let Some(sample) = frame
                .lane_blues
                .iter()
                .find(|sample| sample.key == lane.key)
            else {
                continue;
            };
            let decision =
                update_auto_music_lane_state(state, lane, sample.blue, &plan.performance_rule);
            match decision.action {
                AutoMusicLaneAction::None => {}
                AutoMusicLaneAction::KeyDown => {
                    events.push(AutoMusicPerformanceEvent::LaneKeyDown {
                        frame_index,
                        key: decision.key,
                        blue: decision.blue,
                    })
                }
                AutoMusicLaneAction::KeyUp => events.push(AutoMusicPerformanceEvent::LaneKeyUp {
                    frame_index,
                    key: decision.key,
                    blue: decision.blue,
                }),
            }
        }

        events.push(AutoMusicPerformanceEvent::PollDelay {
            frame_index,
            duration_ms: plan.performance_rule.poll_interval_ms,
        });
        frames_processed += 1;
    }

    let held_keys_before_release = lane_states
        .iter()
        .filter(|(_, state)| state.key_down)
        .map(|(lane, _)| lane.key.clone())
        .collect::<Vec<_>>();
    events.push(AutoMusicPerformanceEvent::ReleaseAllKeys {
        held_keys_before_release: held_keys_before_release.clone(),
        reason: stop_reason,
    });

    AutoMusicPerformanceReport {
        task_key: plan.task_key.clone(),
        stop_reason,
        frames_processed,
        held_keys_before_release,
        events,
    }
}

pub fn execute_auto_music_performance_plan<R>(
    plan: &AutoMusicGameExecutionPlan,
    runtime: &mut R,
) -> Result<AutoMusicPerformanceReport>
where
    R: AutoMusicPerformanceRuntime,
{
    let mut lane_states = plan
        .key_lanes
        .iter()
        .map(|lane| (lane, AutoMusicLaneState::default()))
        .collect::<Vec<_>>();
    let mut held_key_names = Vec::<String>::new();
    let mut events = Vec::new();
    let mut frames_processed = 0usize;
    let mut stop_reason = AutoMusicPerformanceStopReason::SamplesExhausted;

    let execution_result = (|| -> Result<()> {
        loop {
            if runtime.is_auto_music_performance_cancelled()? {
                stop_reason = AutoMusicPerformanceStopReason::CancelledBeforeFrame;
                break;
            }

            let Some(frame) = runtime.next_auto_music_performance_frame()? else {
                break;
            };
            let frame_index = frames_processed;

            for (lane, state) in &mut lane_states {
                let Some(sample) = frame
                    .lane_blues
                    .iter()
                    .find(|sample| sample.key == lane.key)
                else {
                    continue;
                };
                let decision =
                    update_auto_music_lane_state(state, lane, sample.blue, &plan.performance_rule);
                match decision.action {
                    AutoMusicLaneAction::None => {}
                    AutoMusicLaneAction::KeyDown => {
                        runtime.auto_music_key_down(&decision.key)?;
                        if !held_key_names.iter().any(|key| key == &decision.key) {
                            held_key_names.push(decision.key.clone());
                        }
                        events.push(AutoMusicPerformanceEvent::LaneKeyDown {
                            frame_index,
                            key: decision.key,
                            blue: decision.blue,
                        });
                    }
                    AutoMusicLaneAction::KeyUp => {
                        runtime.auto_music_key_up(&decision.key)?;
                        held_key_names.retain(|key| key != &decision.key);
                        events.push(AutoMusicPerformanceEvent::LaneKeyUp {
                            frame_index,
                            key: decision.key,
                            blue: decision.blue,
                        });
                    }
                }
            }

            runtime.delay_auto_music_poll(plan.performance_rule.poll_interval_ms)?;
            events.push(AutoMusicPerformanceEvent::PollDelay {
                frame_index,
                duration_ms: plan.performance_rule.poll_interval_ms,
            });
            frames_processed += 1;
        }
        Ok(())
    })();

    let held_keys_before_release = held_key_names;
    let release_result = runtime.release_all_auto_music_keys(&held_keys_before_release);
    events.push(AutoMusicPerformanceEvent::ReleaseAllKeys {
        held_keys_before_release: held_keys_before_release.clone(),
        reason: stop_reason,
    });

    match (execution_result, release_result) {
        (Ok(()), Ok(())) => {}
        (Err(error), Ok(())) | (Err(error), Err(_)) => return Err(error),
        (Ok(()), Err(error)) => return Err(error),
    }

    Ok(AutoMusicPerformanceReport {
        task_key: plan.task_key.clone(),
        stop_reason,
        frames_processed,
        held_keys_before_release,
        events,
    })
}

pub fn execute_auto_music_album_plan<R>(
    plan: &AutoMusicGameExecutionPlan,
    runtime: &mut R,
) -> Result<AutoMusicAlbumExecutionReport>
where
    R: AutoMusicAlbumRuntime,
{
    if plan.album_rule.selected_difficulties.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: plan.task_key.clone(),
            message: format!(
                "unknown auto music difficulty level: {}",
                plan.album_rule.selected_music_level
            ),
        });
    }

    let mut events = Vec::new();
    let mut songs_checked = 0u64;
    let mut skipped_songs = 0u64;
    let mut performed_songs = 0u64;

    let page_status = runtime.check_auto_music_album_page(&plan.locators.ui_left_top_album_icon)?;
    events.push(AutoMusicAlbumEvent::AlbumPageChecked {
        status: page_status,
    });
    match page_status {
        AutoMusicAlbumPageStatus::ThemeAlbum => {}
        AutoMusicAlbumPageStatus::NotAlbumPage => {
            return Err(TaskError::CommonJobExecution(
                "当前未处于主题专辑界面，请在专辑界面运行本任务。注意全部歌曲列表页面无法运行本任务！"
                    .to_string(),
            ));
        }
        AutoMusicAlbumPageStatus::AllSongsPage => {
            return Err(TaskError::CommonJobExecution(
                "当前在全部歌曲页面，此页面无法运行本任务。请返回到主界面选择专辑列表中以国家为主题的专辑页！"
                    .to_string(),
            ));
        }
    }

    for difficulty in &plan.album_rule.selected_difficulties {
        if runtime.is_auto_music_album_cancelled()? {
            events.push(AutoMusicAlbumEvent::Cancelled {
                difficulty: Some(difficulty.name.clone()),
                song_index: None,
            });
            return Ok(auto_music_album_report(
                plan,
                AutoMusicAlbumExecutionStatus::Cancelled,
                songs_checked,
                skipped_songs,
                performed_songs,
                events,
            ));
        }

        events.push(AutoMusicAlbumEvent::DifficultyStarted {
            difficulty: difficulty.name.clone(),
        });
        let (completion_locator, completion_mode, skip_reason) =
            auto_music_album_completion_probe(plan, difficulty)?;

        for song_index in 1..=plan.album_rule.songs_per_difficulty_loop_count {
            if runtime.is_auto_music_album_cancelled()? {
                events.push(AutoMusicAlbumEvent::Cancelled {
                    difficulty: Some(difficulty.name.clone()),
                    song_index: Some(song_index),
                });
                return Ok(auto_music_album_report(
                    plan,
                    AutoMusicAlbumExecutionStatus::Cancelled,
                    songs_checked,
                    skipped_songs,
                    performed_songs,
                    events,
                ));
            }

            let completed = runtime.is_auto_music_song_completed(completion_locator)?;
            songs_checked += 1;
            events.push(AutoMusicAlbumEvent::SongCompletionChecked {
                difficulty: difficulty.name.clone(),
                song_index,
                mode: completion_mode,
                completed,
            });

            if completed {
                skipped_songs += 1;
                events.push(AutoMusicAlbumEvent::SongSkipped {
                    difficulty: difficulty.name.clone(),
                    song_index,
                    reason: skip_reason,
                });
                auto_music_album_click_next_song(
                    plan,
                    runtime,
                    difficulty,
                    song_index,
                    &mut events,
                )?;
                continue;
            }

            runtime.click_auto_music_white_confirm()?;
            events.push(AutoMusicAlbumEvent::WhiteConfirmClicked {
                difficulty: difficulty.name.clone(),
                song_index,
                phase: AutoMusicAlbumConfirmPhase::EnterSong,
            });
            auto_music_album_delay(
                runtime,
                &mut events,
                Some(&difficulty.name),
                Some(song_index),
                plan.album_rule.after_confirm_sleep_ms,
            )?;

            runtime.click_auto_music_difficulty(difficulty)?;
            events.push(AutoMusicAlbumEvent::DifficultySelected {
                difficulty: difficulty.name.clone(),
                song_index,
            });
            auto_music_album_delay(
                runtime,
                &mut events,
                Some(&difficulty.name),
                Some(song_index),
                plan.album_rule.select_difficulty_sleep_ms,
            )?;

            runtime.click_auto_music_white_confirm()?;
            events.push(AutoMusicAlbumEvent::WhiteConfirmClicked {
                difficulty: difficulty.name.clone(),
                song_index,
                phase: AutoMusicAlbumConfirmPhase::StartPerformance,
            });
            auto_music_album_delay(
                runtime,
                &mut events,
                Some(&difficulty.name),
                Some(song_index),
                plan.album_rule.after_start_performance_sleep_ms,
            )?;

            let performance = runtime.execute_auto_music_song(difficulty, song_index)?;
            performed_songs += 1;
            events.push(AutoMusicAlbumEvent::PerformanceCompleted {
                difficulty: difficulty.name.clone(),
                song_index,
                stop_reason: performance.stop_reason,
                frames_processed: performance.frames_processed,
            });

            auto_music_album_delay(
                runtime,
                &mut events,
                Some(&difficulty.name),
                Some(song_index),
                plan.album_rule.after_song_finished_sleep_ms,
            )?;
            runtime.wait_auto_music_album_page(&plan.locators.ui_left_top_album_icon)?;
            events.push(AutoMusicAlbumEvent::AlbumPageWaited {
                difficulty: difficulty.name.clone(),
                song_index,
            });
            auto_music_album_click_next_song(plan, runtime, difficulty, song_index, &mut events)?;
        }

        events.push(AutoMusicAlbumEvent::DifficultyCompleted {
            difficulty: difficulty.name.clone(),
        });
    }

    events.push(AutoMusicAlbumEvent::AlbumCompleted);
    Ok(auto_music_album_report(
        plan,
        AutoMusicAlbumExecutionStatus::Completed,
        songs_checked,
        skipped_songs,
        performed_songs,
        events,
    ))
}

fn auto_music_album_report(
    plan: &AutoMusicGameExecutionPlan,
    status: AutoMusicAlbumExecutionStatus,
    songs_checked: u64,
    skipped_songs: u64,
    performed_songs: u64,
    events: Vec<AutoMusicAlbumEvent>,
) -> AutoMusicAlbumExecutionReport {
    AutoMusicAlbumExecutionReport {
        task_key: plan.task_key.clone(),
        status,
        difficulty_count: plan.album_rule.selected_difficulties.len(),
        songs_checked,
        skipped_songs,
        performed_songs,
        events,
    }
}

fn auto_music_album_completion_probe<'a>(
    plan: &'a AutoMusicGameExecutionPlan,
    difficulty: &AutoMusicDifficultyRule,
) -> Result<(
    &'a AutoMusicTemplateLocator,
    AutoMusicAlbumCompletionMode,
    AutoMusicAlbumSkipReason,
)> {
    if plan.album_rule.canorus_level_skip_when_enabled {
        let locator = plan
            .locators
            .music_canorus_levels
            .iter()
            .find(|locator| locator.name == difficulty.canorus_locator_name)
            .ok_or_else(|| TaskError::InvalidTaskConfig {
                key: plan.task_key.clone(),
                message: format!("missing canorus locator for difficulty {}", difficulty.name),
            })?;
        Ok((
            locator,
            AutoMusicAlbumCompletionMode::CanorusLevel,
            AutoMusicAlbumSkipReason::CanorusLevelComplete,
        ))
    } else {
        Ok((
            &plan.locators.album_music_complete,
            AutoMusicAlbumCompletionMode::AllRewards,
            AutoMusicAlbumSkipReason::AllRewardsComplete,
        ))
    }
}

fn auto_music_album_click_next_song<R>(
    plan: &AutoMusicGameExecutionPlan,
    runtime: &mut R,
    difficulty: &AutoMusicDifficultyRule,
    song_index: u64,
    events: &mut Vec<AutoMusicAlbumEvent>,
) -> Result<()>
where
    R: AutoMusicAlbumRuntime,
{
    runtime.click_auto_music_next_song(
        plan.album_rule.next_song_click_x_1080p,
        plan.album_rule.next_song_click_y_1080p,
    )?;
    events.push(AutoMusicAlbumEvent::NextSongClicked {
        difficulty: difficulty.name.clone(),
        song_index,
    });
    auto_music_album_delay(
        runtime,
        events,
        Some(&difficulty.name),
        Some(song_index),
        plan.album_rule.after_next_song_sleep_ms,
    )
}

fn auto_music_album_delay<R>(
    runtime: &mut R,
    events: &mut Vec<AutoMusicAlbumEvent>,
    difficulty: Option<&str>,
    song_index: Option<u64>,
    duration_ms: u64,
) -> Result<()>
where
    R: AutoMusicAlbumRuntime,
{
    runtime.delay_auto_music_album(duration_ms)?;
    events.push(AutoMusicAlbumEvent::Delay {
        difficulty: difficulty.map(ToOwned::to_owned),
        song_index,
        duration_ms,
    });
    Ok(())
}

fn auto_music_key_lanes() -> Vec<AutoMusicGameKeyLane> {
    AUTO_MUSIC_GAME_KEY_LANES
        .iter()
        .map(|(key, x)| AutoMusicGameKeyLane {
            key: (*key).to_string(),
            x_1080p: *x,
            y_1080p: AUTO_MUSIC_GAME_SAMPLE_Y_1080P,
        })
        .collect()
}

fn default_difficulties() -> Vec<AutoMusicDifficultyRule> {
    AUTO_MUSIC_GAME_DIFFICULTIES
        .iter()
        .map(|(name, x, y, locator)| AutoMusicDifficultyRule {
            name: (*name).to_string(),
            click_x_1080p: *x,
            click_y_1080p: *y,
            canorus_locator_name: (*locator).to_string(),
        })
        .collect()
}

fn selected_difficulties(music_level: &str) -> Vec<AutoMusicDifficultyRule> {
    if music_level == AUTO_MUSIC_GAME_ALL_LEVELS {
        return default_difficulties();
    }
    default_difficulties()
        .into_iter()
        .filter(|difficulty| difficulty.name == music_level)
        .collect()
}

fn auto_music_locators() -> AutoMusicGameLocators {
    AutoMusicGameLocators {
        ui_left_top_album_icon: template_locator(
            "UiLeftTopAlbumIcon",
            AUTO_MUSIC_UI_LEFT_TOP_ALBUM_ICON_ASSET,
            Some(Rect {
                x: 0,
                y: 0,
                width: 150,
                height: 120,
            }),
            None,
        ),
        btn_pause: template_locator(
            "BtnPause",
            AUTO_MUSIC_BTN_PAUSE_ASSET,
            None,
            Some("CaptureRect.CutRightTop(0.2, 0.2)".to_string()),
        ),
        album_music_complete: template_locator(
            "AlbumMusicComplate",
            AUTO_MUSIC_ALBUM_COMPLETE_ASSET,
            Some(Rect {
                x: 900,
                y: 320,
                width: 100,
                height: 80,
            }),
            None,
        ),
        btn_list: template_locator(
            "BtnList",
            AUTO_MUSIC_BTN_LIST_ASSET,
            None,
            Some("CaptureRect.CutRightBottom(0.4, 0.2)".to_string()),
        ),
        music_canorus_levels: vec![
            template_locator(
                "MusicCanorusLevel1",
                AUTO_MUSIC_CANORUS_ASSET,
                Some(Rect {
                    x: 450,
                    y: 430,
                    width: 200,
                    height: 60,
                }),
                None,
            ),
            template_locator(
                "MusicCanorusLevel2",
                AUTO_MUSIC_CANORUS_ASSET,
                Some(Rect {
                    x: 450,
                    y: 520,
                    width: 200,
                    height: 60,
                }),
                None,
            ),
            template_locator(
                "MusicCanorusLevel3",
                AUTO_MUSIC_CANORUS_ASSET,
                Some(Rect {
                    x: 450,
                    y: 610,
                    width: 200,
                    height: 60,
                }),
                None,
            ),
            template_locator(
                "MusicCanorusLevel4",
                AUTO_MUSIC_CANORUS_ASSET,
                Some(Rect {
                    x: 450,
                    y: 690,
                    width: 200,
                    height: 60,
                }),
                None,
            ),
        ],
    }
}

fn template_locator(
    name: &str,
    asset: &str,
    roi: Option<Rect>,
    roi_rule: Option<String>,
) -> AutoMusicTemplateLocator {
    AutoMusicTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        roi_rule,
        threshold: None,
        match_mode: TemplateMatchMode::CCoeffNormed,
    }
}

fn auto_music_steps() -> Vec<AutoMusicGameStep> {
    use AutoMusicGameAction::*;
    use AutoMusicGamePhase::*;
    vec![
        AutoMusicGameStep {
            phase: Startup,
            action: CheckResolutionAndLogWarnings,
        },
        AutoMusicGameStep {
            phase: Performance,
            action: ConvertLaneCoordinates,
        },
        AutoMusicGameStep {
            phase: Performance,
            action: SpawnLanePixelLoops,
        },
        AutoMusicGameStep {
            phase: Performance,
            action: PressKeyWhenPixelDarkens,
        },
        AutoMusicGameStep {
            phase: Performance,
            action: ReleaseKeyWhenPixelBrightens,
        },
        AutoMusicGameStep {
            phase: Album,
            action: CheckAlbumPage,
        },
        AutoMusicGameStep {
            phase: Album,
            action: SkipCompletedSongOrSelectDifficulty,
        },
        AutoMusicGameStep {
            phase: Album,
            action: RunPerformanceUntilListButtonAppears,
        },
        AutoMusicGameStep {
            phase: Cleanup,
            action: ReleaseAllKeys,
        },
    ]
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| value.get(*name))
        .and_then(Value::as_f64)
}
