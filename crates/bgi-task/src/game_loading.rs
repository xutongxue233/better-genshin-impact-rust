use crate::Result;
use bgi_core::GenshinStartConfig;
use bgi_vision::{Rect, Size};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub const GAME_LOADING_TASK_KEY: &str = "GameLoading";
pub const GAME_LOADING_ENTER_GAME_ASSET: &str = "GameLoading:enter_game.png";
pub const GAME_LOADING_CHOOSE_ENTER_GAME_ASSET: &str = "GameLoading:choose_enter_game.png";
pub const GAME_LOADING_WELKIN_MOON_ASSET: &str = "GameLoading:welkin_moon_logo.png";
pub const GAME_LOADING_GIRL_MOON_ASSET: &str = "GameLoading:girl_moon.png";
pub const GAME_LOADING_WHITE_CONFIRM_ASSET: &str = "Common/Element:btn_white_confirm.png";
pub const GAME_LOADING_PRIMOGEM_ASSET: &str = "Common/Element:primogem.png";
pub const GAME_LOADING_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const GAME_LOADING_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameLoadingExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub config_rule: GameLoadingConfigRule,
    pub throttle_rule: GameLoadingThrottleRule,
    pub finish_rule: GameLoadingFinishRule,
    pub starward_rule: GameLoadingStarwardRule,
    pub bili_rule: GameLoadingBiliRule,
    pub locators: GameLoadingLocators,
    pub steps: Vec<GameLoadingTickStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLoadingExecutionConfig {
    pub capture_size: Size,
    pub genshin_start_config: GenshinStartConfig,
}

impl Default for GameLoadingExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                GAME_LOADING_DEFAULT_CAPTURE_WIDTH,
                GAME_LOADING_DEFAULT_CAPTURE_HEIGHT,
            ),
            genshin_start_config: GenshinStartConfig::default(),
        }
    }
}

impl GameLoadingExecutionConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else {
            return config;
        };
        if let Some(capture_size) = capture_size_from_value(value) {
            config.capture_size = capture_size;
        }
        let start_config_value = value
            .get("genshinStartConfig")
            .or_else(|| value.get("GenshinStartConfig"))
            .or_else(|| value.get("genshin_start_config"))
            .unwrap_or(value);
        config.genshin_start_config =
            serde_json::from_value(start_config_value.clone()).unwrap_or_default();
        config
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingConfigRule {
    pub auto_enter_game_enabled: bool,
    pub record_game_time_enabled: bool,
    pub install_path: String,
    pub install_file_name: Option<String>,
    pub linked_start_enabled: bool,
    pub start_game_with_cmd: bool,
    pub auto_disable_genshin_hdr_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingThrottleRule {
    pub tick_interval_ms: u64,
    pub max_runtime_minutes: u64,
    pub age_prompt_ocr_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingFinishRule {
    pub stop_when_main_ui_detected: bool,
    pub stop_when_any_closable_ui_detected: bool,
    pub stop_when_in_domain_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingStarwardRule {
    pub enabled: bool,
    pub protocol: String,
    pub protocol_registry_key: String,
    pub protocol_value_name: String,
    pub protocol_value_expected: String,
    pub server_candidates: Vec<GameLoadingServerCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingServerCandidate {
    pub source: GameLoadingServerSource,
    pub install_file_name: Option<String>,
    pub config_ini_channel: Option<String>,
    pub registry_key: Option<String>,
    pub registry_value: Option<String>,
    pub server: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GameLoadingServerSource {
    InstallFileName,
    ConfigIniChannel,
    Registry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingRegistryHit {
    pub registry_key: String,
    pub registry_value: Option<String>,
    pub value_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingResolvedServer {
    pub source: GameLoadingServerSource,
    pub server: String,
    pub matched_install_file_name: Option<String>,
    pub matched_config_ini_channel: Option<String>,
    pub matched_registry_key: Option<String>,
    pub matched_registry_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingBiliRule {
    pub detect_channel_from_config_ini: bool,
    pub bili_channel: String,
    pub process_name: String,
    pub title_contains: String,
    pub protocol_window_title_contains: String,
    pub login_window_title_contains: String,
    pub owner_must_match_process: bool,
    pub protocol_click: GameLoadingClickPoint,
    pub login_click: GameLoadingClickPoint,
    pub wait_before_login_click_ms: u64,
    pub wait_after_login_click_ms: u64,
    pub wait_after_login_window_closed_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GameLoadingClickPoint {
    pub x_1080p: i32,
    pub y_1080p: i32,
    pub applies_dpi_scale_to_offset: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameLoadingLocators {
    pub choose_enter_game: GameLoadingTemplateLocator,
    pub enter_game: GameLoadingTemplateLocator,
    pub welkin_moon: GameLoadingTemplateLocator,
    pub girl_moon: GameLoadingTemplateLocator,
    pub white_confirm: GameLoadingTemplateLocator,
    pub primogem: GameLoadingTemplateLocator,
    pub age_prompt_ocr: GameLoadingOcrRule,
    pub background_click: GameLoadingClickPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameLoadingTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Rect,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingOcrRule {
    pub keywords: Vec<String>,
    pub confirm_asset: String,
    pub ocr_this_full_capture: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingTickStep {
    pub phase: GameLoadingTickPhase,
    pub condition: GameLoadingTickCondition,
    pub action: GameLoadingTickAction,
}

impl GameLoadingTickStep {
    fn new(
        phase: GameLoadingTickPhase,
        condition: GameLoadingTickCondition,
        action: GameLoadingTickAction,
    ) -> Self {
        Self {
            phase,
            condition,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GameLoadingTickPhase {
    Init,
    Throttle,
    StopGate,
    AgePrompt,
    BiliDetection,
    EnterGame,
    BiliLogin,
    RewardPopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GameLoadingTickCondition {
    Always,
    WhenAutoEnterDisabled,
    WhenTickIntervalNotElapsed,
    WhenMaxRuntimeReached,
    WhenGameUiEntered,
    WhenAgePromptOcrIntervalElapsed,
    WhenAgePromptTextMatched,
    WhenNotBiliServerAndChooseEnterGameDetected,
    WhenEnterGameDetected,
    WhenBiliServerAndEnterGameMissing,
    WhenBiliProtocolWindowDetected,
    WhenBiliLoginWindowDetected,
    WhenWelkinMoonDetected,
    WhenPrimogemDetected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GameLoadingTickAction {
    DisableTrigger,
    SkipTick,
    DetectGameUiEntered,
    OcrAgePrompt,
    ClickTemplate { asset: String },
    ClickBackground,
    DetectBiliFromConfigIni,
    DetectBiliLoginWindow,
    ClickPoint(GameLoadingClickPoint),
    StartStarwardRecording,
}

pub fn plan_game_loading(config: GameLoadingExecutionConfig) -> GameLoadingExecutionPlan {
    let capture_size = config.capture_size;
    let start_config = config.genshin_start_config;
    let install_file_name = file_name(&start_config.install_path);
    let locators = game_loading_locators(capture_size);
    let starward_rule = GameLoadingStarwardRule {
        enabled: start_config.record_game_time_enabled,
        protocol: "starward".to_string(),
        protocol_registry_key: r"HKEY_CLASSES_ROOT\starward".to_string(),
        protocol_value_name: "URL Protocol".to_string(),
        protocol_value_expected: String::new(),
        server_candidates: game_loading_server_candidates(),
    };

    GameLoadingExecutionPlan {
        task_key: GAME_LOADING_TASK_KEY.to_string(),
        display_name: "Game Loading".to_string(),
        capture_size,
        config_rule: GameLoadingConfigRule {
            auto_enter_game_enabled: start_config.auto_enter_game_enabled,
            record_game_time_enabled: start_config.record_game_time_enabled,
            install_path: start_config.install_path,
            install_file_name,
            linked_start_enabled: start_config.linked_start_enabled,
            start_game_with_cmd: start_config.start_game_with_cmd,
            auto_disable_genshin_hdr_enabled: start_config.auto_disable_genshin_hdr_enabled,
        },
        throttle_rule: GameLoadingThrottleRule {
            tick_interval_ms: 2_000,
            max_runtime_minutes: 5,
            age_prompt_ocr_interval_ms: 1_000,
        },
        finish_rule: GameLoadingFinishRule {
            stop_when_main_ui_detected: true,
            stop_when_any_closable_ui_detected: true,
            stop_when_in_domain_detected: true,
        },
        starward_rule,
        bili_rule: GameLoadingBiliRule {
            detect_channel_from_config_ini: true,
            bili_channel: "14".to_string(),
            process_name: "YuanShen".to_string(),
            title_contains: "bilibili".to_string(),
            protocol_window_title_contains: "协议".to_string(),
            login_window_title_contains: "登录".to_string(),
            owner_must_match_process: true,
            protocol_click: GameLoadingClickPoint {
                x_1080p: 1030,
                y_1080p: 615,
                applies_dpi_scale_to_offset: true,
            },
            login_click: GameLoadingClickPoint {
                x_1080p: 960,
                y_1080p: 630,
                applies_dpi_scale_to_offset: true,
            },
            wait_before_login_click_ms: 2_000,
            wait_after_login_click_ms: 2_000,
            wait_after_login_window_closed_ms: 2_000,
        },
        locators,
        steps: game_loading_steps(),
        executor_ready: true,
        pending_native: vec![
            "desktop adapter for BV main UI / closable UI / domain / welkin moon detection"
                .to_string(),
            "desktop adapter for template matching and OCR against live capture frames".to_string(),
            "desktop adapter for background mouse click and 1080p/DPI-aware point input dispatch"
                .to_string(),
            "desktop adapter for Bilibili login window enumeration and owner-process matching"
                .to_string(),
            "desktop adapter for config.ini, registry, and Starward protocol execution".to_string(),
        ],
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GameLoadingTriggerState {
    pub disabled: bool,
    pub started_at_ms: Option<u64>,
    pub last_tick_ms: Option<u64>,
    pub last_age_prompt_ocr_ms: Option<u64>,
    pub starward_recording_started: bool,
    pub game_ui_entered: Option<bool>,
    pub age_prompt_text_matched: Option<bool>,
    pub bili_server_detected: Option<bool>,
    pub bili_protocol_window_detected: Option<bool>,
    pub bili_login_window_detected: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GameLoadingTickObservation {
    pub now_ms: u64,
    pub game_ui_entered: bool,
    pub age_prompt_text_matched: bool,
    pub bili_server: bool,
    pub choose_enter_game_detected: bool,
    pub enter_game_detected: bool,
    pub bili_protocol_window_detected: bool,
    pub bili_login_window_detected: bool,
    pub welkin_moon_detected: bool,
    pub primogem_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameLoadingTickExecutionReport {
    pub task_key: String,
    pub observation: Option<GameLoadingTickObservation>,
    pub executed_actions: Vec<GameLoadingExecutedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum GameLoadingExecutedAction {
    DisableTrigger,
    SkipTick,
    StartStarwardRecording,
    DetectGameUiEntered {
        entered: bool,
    },
    OcrAgePrompt {
        matched: bool,
    },
    ClickTemplate {
        asset: String,
    },
    ClickBackground {
        point: GameLoadingClickPoint,
    },
    DetectBiliFromConfigIni {
        bili_server: bool,
    },
    DetectBiliLoginWindow {
        protocol_window_detected: bool,
        login_window_detected: bool,
    },
    ClickPoint {
        point: GameLoadingClickPoint,
    },
}

pub trait GameLoadingRuntime {
    fn observe_game_loading_tick(
        &mut self,
        plan: &GameLoadingExecutionPlan,
    ) -> Result<GameLoadingTickObservation>;

    fn disable_game_loading_trigger(&mut self) -> Result<()>;

    fn start_game_loading_starward_recording(
        &mut self,
        rule: &GameLoadingStarwardRule,
    ) -> Result<()>;

    fn click_game_loading_template(&mut self, asset: &str) -> Result<()>;

    fn click_game_loading_background(&mut self, point: GameLoadingClickPoint) -> Result<()>;

    fn click_game_loading_point(&mut self, point: GameLoadingClickPoint) -> Result<()>;
}

pub fn execute_game_loading_tick_plan<R>(
    plan: &GameLoadingExecutionPlan,
    state: &mut GameLoadingTriggerState,
    runtime: &mut R,
) -> Result<GameLoadingTickExecutionReport>
where
    R: GameLoadingRuntime,
{
    let mut executed_actions = Vec::new();

    if state.disabled {
        return Ok(GameLoadingTickExecutionReport {
            task_key: plan.task_key.clone(),
            observation: None,
            executed_actions,
        });
    }

    if !plan.config_rule.auto_enter_game_enabled {
        runtime.disable_game_loading_trigger()?;
        state.disabled = true;
        executed_actions.push(GameLoadingExecutedAction::DisableTrigger);
        return Ok(GameLoadingTickExecutionReport {
            task_key: plan.task_key.clone(),
            observation: None,
            executed_actions,
        });
    }

    let observation = runtime.observe_game_loading_tick(plan)?;
    state.started_at_ms.get_or_insert(observation.now_ms);

    if plan.starward_rule.enabled && !state.starward_recording_started {
        runtime.start_game_loading_starward_recording(&plan.starward_rule)?;
        state.starward_recording_started = true;
        executed_actions.push(GameLoadingExecutedAction::StartStarwardRecording);
    }

    if elapsed_ms_since(state.last_tick_ms, observation.now_ms)
        < plan.throttle_rule.tick_interval_ms
    {
        executed_actions.push(GameLoadingExecutedAction::SkipTick);
        return Ok(GameLoadingTickExecutionReport {
            task_key: plan.task_key.clone(),
            observation: Some(observation),
            executed_actions,
        });
    }
    state.last_tick_ms = Some(observation.now_ms);

    if elapsed_ms_since(state.started_at_ms, observation.now_ms)
        >= plan
            .throttle_rule
            .max_runtime_minutes
            .saturating_mul(60_000)
    {
        runtime.disable_game_loading_trigger()?;
        state.disabled = true;
        executed_actions.push(GameLoadingExecutedAction::DisableTrigger);
        return Ok(GameLoadingTickExecutionReport {
            task_key: plan.task_key.clone(),
            observation: Some(observation),
            executed_actions,
        });
    }

    state.game_ui_entered = Some(observation.game_ui_entered);
    executed_actions.push(GameLoadingExecutedAction::DetectGameUiEntered {
        entered: observation.game_ui_entered,
    });
    if observation.game_ui_entered {
        runtime.disable_game_loading_trigger()?;
        state.disabled = true;
        executed_actions.push(GameLoadingExecutedAction::DisableTrigger);
        return Ok(GameLoadingTickExecutionReport {
            task_key: plan.task_key.clone(),
            observation: Some(observation),
            executed_actions,
        });
    }

    if elapsed_ms_since(state.last_age_prompt_ocr_ms, observation.now_ms)
        >= plan.throttle_rule.age_prompt_ocr_interval_ms
    {
        state.last_age_prompt_ocr_ms = Some(observation.now_ms);
        state.age_prompt_text_matched = Some(observation.age_prompt_text_matched);
        executed_actions.push(GameLoadingExecutedAction::OcrAgePrompt {
            matched: observation.age_prompt_text_matched,
        });
        if observation.age_prompt_text_matched {
            runtime.click_game_loading_template(&plan.locators.age_prompt_ocr.confirm_asset)?;
            executed_actions.push(GameLoadingExecutedAction::ClickTemplate {
                asset: plan.locators.age_prompt_ocr.confirm_asset.clone(),
            });
        }
    }

    state.bili_server_detected = Some(observation.bili_server);
    executed_actions.push(GameLoadingExecutedAction::DetectBiliFromConfigIni {
        bili_server: observation.bili_server,
    });

    if !observation.bili_server && observation.choose_enter_game_detected {
        runtime.click_game_loading_template(&plan.locators.choose_enter_game.asset)?;
        executed_actions.push(GameLoadingExecutedAction::ClickTemplate {
            asset: plan.locators.choose_enter_game.asset.clone(),
        });
    }

    if observation.enter_game_detected {
        runtime.click_game_loading_background(plan.locators.background_click)?;
        executed_actions.push(GameLoadingExecutedAction::ClickBackground {
            point: plan.locators.background_click,
        });
    }

    if observation.bili_server && !observation.enter_game_detected {
        state.bili_protocol_window_detected = Some(observation.bili_protocol_window_detected);
        state.bili_login_window_detected = Some(observation.bili_login_window_detected);
        executed_actions.push(GameLoadingExecutedAction::DetectBiliLoginWindow {
            protocol_window_detected: observation.bili_protocol_window_detected,
            login_window_detected: observation.bili_login_window_detected,
        });

        if observation.bili_protocol_window_detected {
            runtime.click_game_loading_point(plan.bili_rule.protocol_click)?;
            executed_actions.push(GameLoadingExecutedAction::ClickPoint {
                point: plan.bili_rule.protocol_click,
            });
        }
        if observation.bili_login_window_detected {
            runtime.click_game_loading_point(plan.bili_rule.login_click)?;
            executed_actions.push(GameLoadingExecutedAction::ClickPoint {
                point: plan.bili_rule.login_click,
            });
        }
    }

    if observation.welkin_moon_detected {
        runtime.click_game_loading_background(plan.locators.background_click)?;
        executed_actions.push(GameLoadingExecutedAction::ClickBackground {
            point: plan.locators.background_click,
        });
    }
    if observation.primogem_detected {
        runtime.click_game_loading_background(plan.locators.background_click)?;
        executed_actions.push(GameLoadingExecutedAction::ClickBackground {
            point: plan.locators.background_click,
        });
    }

    Ok(GameLoadingTickExecutionReport {
        task_key: plan.task_key.clone(),
        observation: Some(observation),
        executed_actions,
    })
}

pub fn parse_game_loading_config_ini_channel(config_ini: &str) -> Option<String> {
    let mut in_general_section = false;

    for line in config_ini.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let section = line[1..line.len() - 1].trim();
            in_general_section = section.eq_ignore_ascii_case("General");
            continue;
        }

        if !in_general_section {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("channel") {
            continue;
        }

        let channel = value.split_whitespace().next().unwrap_or_default();
        if channel.is_empty() {
            return None;
        }

        return Some(channel.to_string());
    }

    None
}

pub fn resolve_game_loading_server(
    install_file_name: Option<&str>,
    config_ini_channel: Option<&str>,
    registry_hits: &[GameLoadingRegistryHit],
) -> Option<GameLoadingResolvedServer> {
    game_loading_server_candidates()
        .into_iter()
        .find(|candidate| {
            game_loading_server_candidate_matches(
                candidate,
                install_file_name,
                config_ini_channel,
                registry_hits,
            )
        })
        .map(|candidate| GameLoadingResolvedServer {
            source: candidate.source,
            server: candidate.server,
            matched_install_file_name: candidate.install_file_name,
            matched_config_ini_channel: candidate.config_ini_channel,
            matched_registry_key: candidate.registry_key,
            matched_registry_value: candidate.registry_value,
        })
}

fn game_loading_server_candidate_matches(
    candidate: &GameLoadingServerCandidate,
    install_file_name: Option<&str>,
    config_ini_channel: Option<&str>,
    registry_hits: &[GameLoadingRegistryHit],
) -> bool {
    match candidate.source {
        GameLoadingServerSource::InstallFileName => option_str_eq_ignore_ascii_case(
            install_file_name,
            candidate.install_file_name.as_deref(),
        ),
        GameLoadingServerSource::ConfigIniChannel => {
            option_str_eq_ignore_ascii_case(
                install_file_name,
                candidate.install_file_name.as_deref(),
            ) && config_ini_channel == candidate.config_ini_channel.as_deref()
        }
        GameLoadingServerSource::Registry => registry_hits.iter().any(|hit| {
            hit.value_exists
                && option_str_eq_ignore_ascii_case(
                    Some(hit.registry_key.as_str()),
                    candidate.registry_key.as_deref(),
                )
                && option_str_eq_ignore_ascii_case(
                    hit.registry_value.as_deref(),
                    candidate.registry_value.as_deref(),
                )
        }),
    }
}

fn option_str_eq_ignore_ascii_case(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
        (None, None) => true,
        _ => false,
    }
}

fn game_loading_server_candidates() -> Vec<GameLoadingServerCandidate> {
    vec![
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::InstallFileName,
            install_file_name: Some("GenshinImpact.exe".to_string()),
            config_ini_channel: None,
            registry_key: None,
            registry_value: None,
            server: "hk4e_global".to_string(),
        },
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::ConfigIniChannel,
            install_file_name: Some("YuanShen.exe".to_string()),
            config_ini_channel: Some("1".to_string()),
            registry_key: None,
            registry_value: None,
            server: "hk4e_cn".to_string(),
        },
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::ConfigIniChannel,
            install_file_name: Some("YuanShen.exe".to_string()),
            config_ini_channel: Some("14".to_string()),
            registry_key: None,
            registry_value: None,
            server: "hk4e_bilibili".to_string(),
        },
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::Registry,
            install_file_name: None,
            config_ini_channel: None,
            registry_key: Some(r"HKEY_CURRENT_USER\Software\miHoYo\HYP\1_1\hk4e_cn".to_string()),
            registry_value: Some("GameInstallPath".to_string()),
            server: "hk4e_cn".to_string(),
        },
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::Registry,
            install_file_name: None,
            config_ini_channel: None,
            registry_key: Some(
                r"HKEY_CURRENT_USER\Software\Cognosphere\HYP\1_0\hk4e_global".to_string(),
            ),
            registry_value: Some("GameInstallPath".to_string()),
            server: "hk4e_global".to_string(),
        },
        GameLoadingServerCandidate {
            source: GameLoadingServerSource::Registry,
            install_file_name: None,
            config_ini_channel: None,
            registry_key: Some(
                r"HKEY_CURRENT_USER\Software\miHoYo\HYP\standalone\14_0\hk4e_cn\umfgRO5gh5\hk4e_cn"
                    .to_string(),
            ),
            registry_value: Some("GameInstallPath".to_string()),
            server: "hk4e_bilibili".to_string(),
        },
    ]
}

fn game_loading_locators(capture_size: Size) -> GameLoadingLocators {
    GameLoadingLocators {
        choose_enter_game: GameLoadingTemplateLocator {
            name: "ChooseEnterGame".to_string(),
            asset: GAME_LOADING_CHOOSE_ENTER_GAME_ASSET.to_string(),
            roi: bottom_half(capture_size),
            draw_on_window: false,
        },
        enter_game: GameLoadingTemplateLocator {
            name: "EnterGame".to_string(),
            asset: GAME_LOADING_ENTER_GAME_ASSET.to_string(),
            roi: middle_bottom_third(capture_size),
            draw_on_window: false,
        },
        welkin_moon: GameLoadingTemplateLocator {
            name: "WelkinMoon".to_string(),
            asset: GAME_LOADING_WELKIN_MOON_ASSET.to_string(),
            roi: bottom_half(capture_size),
            draw_on_window: false,
        },
        girl_moon: GameLoadingTemplateLocator {
            name: "GirlMoon".to_string(),
            asset: GAME_LOADING_GIRL_MOON_ASSET.to_string(),
            roi: bottom_half(capture_size),
            draw_on_window: false,
        },
        white_confirm: GameLoadingTemplateLocator {
            name: "WhiteConfirm".to_string(),
            asset: GAME_LOADING_WHITE_CONFIRM_ASSET.to_string(),
            roi: Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32,
                height: capture_size.height as i32,
            },
            draw_on_window: false,
        },
        primogem: GameLoadingTemplateLocator {
            name: "Primogem".to_string(),
            asset: GAME_LOADING_PRIMOGEM_ASSET.to_string(),
            roi: Rect {
                x: 0,
                y: 0,
                width: capture_size.width as i32,
                height: capture_size.height as i32,
            },
            draw_on_window: false,
        },
        age_prompt_ocr: GameLoadingOcrRule {
            keywords: vec!["适龄".to_string(), "监护".to_string()],
            confirm_asset: GAME_LOADING_WHITE_CONFIRM_ASSET.to_string(),
            ocr_this_full_capture: true,
        },
        background_click: GameLoadingClickPoint {
            x_1080p: 100,
            y_1080p: 100,
            applies_dpi_scale_to_offset: false,
        },
    }
}

fn game_loading_steps() -> Vec<GameLoadingTickStep> {
    vec![
        GameLoadingTickStep::new(
            GameLoadingTickPhase::Init,
            GameLoadingTickCondition::WhenAutoEnterDisabled,
            GameLoadingTickAction::DisableTrigger,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::Init,
            GameLoadingTickCondition::Always,
            GameLoadingTickAction::StartStarwardRecording,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::Throttle,
            GameLoadingTickCondition::WhenTickIntervalNotElapsed,
            GameLoadingTickAction::SkipTick,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::StopGate,
            GameLoadingTickCondition::WhenMaxRuntimeReached,
            GameLoadingTickAction::DisableTrigger,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::StopGate,
            GameLoadingTickCondition::WhenGameUiEntered,
            GameLoadingTickAction::DetectGameUiEntered,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::StopGate,
            GameLoadingTickCondition::WhenGameUiEntered,
            GameLoadingTickAction::DisableTrigger,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::AgePrompt,
            GameLoadingTickCondition::WhenAgePromptOcrIntervalElapsed,
            GameLoadingTickAction::OcrAgePrompt,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::AgePrompt,
            GameLoadingTickCondition::WhenAgePromptTextMatched,
            GameLoadingTickAction::ClickTemplate {
                asset: GAME_LOADING_WHITE_CONFIRM_ASSET.to_string(),
            },
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::BiliDetection,
            GameLoadingTickCondition::Always,
            GameLoadingTickAction::DetectBiliFromConfigIni,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::EnterGame,
            GameLoadingTickCondition::WhenNotBiliServerAndChooseEnterGameDetected,
            GameLoadingTickAction::ClickTemplate {
                asset: GAME_LOADING_CHOOSE_ENTER_GAME_ASSET.to_string(),
            },
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::EnterGame,
            GameLoadingTickCondition::WhenEnterGameDetected,
            GameLoadingTickAction::ClickBackground,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::BiliLogin,
            GameLoadingTickCondition::WhenBiliServerAndEnterGameMissing,
            GameLoadingTickAction::DetectBiliLoginWindow,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::BiliLogin,
            GameLoadingTickCondition::WhenBiliProtocolWindowDetected,
            GameLoadingTickAction::ClickPoint(GameLoadingClickPoint {
                x_1080p: 1030,
                y_1080p: 615,
                applies_dpi_scale_to_offset: true,
            }),
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::BiliLogin,
            GameLoadingTickCondition::WhenBiliLoginWindowDetected,
            GameLoadingTickAction::ClickPoint(GameLoadingClickPoint {
                x_1080p: 960,
                y_1080p: 630,
                applies_dpi_scale_to_offset: true,
            }),
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::RewardPopup,
            GameLoadingTickCondition::WhenWelkinMoonDetected,
            GameLoadingTickAction::ClickBackground,
        ),
        GameLoadingTickStep::new(
            GameLoadingTickPhase::RewardPopup,
            GameLoadingTickCondition::WhenPrimogemDetected,
            GameLoadingTickAction::ClickBackground,
        ),
    ]
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    let capture = value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .unwrap_or(value);
    let width = u32_member(capture, ["width", "Width", "captureWidth", "CaptureWidth"])?;
    let height = u32_member(
        capture,
        ["height", "Height", "captureHeight", "CaptureHeight"],
    )?;
    Some(Size::new(width, height))
}

fn u32_member<const N: usize>(value: &Value, keys: [&str; N]) -> Option<u32> {
    keys.into_iter()
        .filter_map(|key| value.get(key))
        .find_map(|value| value.as_u64().and_then(|value| u32::try_from(value).ok()))
}

fn elapsed_ms_since(previous_ms: Option<u64>, now_ms: u64) -> u64 {
    previous_ms
        .map(|previous| now_ms.saturating_sub(previous))
        .unwrap_or(u64::MAX)
}

fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn bottom_half(size: Size) -> Rect {
    Rect {
        x: 0,
        y: size.height as i32 / 2,
        width: size.width as i32,
        height: size.height as i32 - size.height as i32 / 2,
    }
}

fn middle_bottom_third(size: Size) -> Rect {
    Rect {
        x: size.width as i32 / 3,
        y: size.height as i32 / 2,
        width: size.width as i32 / 3,
        height: size.height as i32 - size.height as i32 / 2,
    }
}
