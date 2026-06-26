use bgi_core::{AutoGeniusInvokationConfig, RectConfig};
use bgi_vision::{Rect, Size, TemplateMatchMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{Result, TaskError};

pub const AUTO_GENIUS_INVOKATION_TASK_KEY: &str = "AutoGeniusInvokation";
pub const AUTO_GENIUS_INVOKATION_DISPLAY_NAME: &str = "自动七圣召唤";
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_WIDTH: u32 = 1920;
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_HEIGHT: u32 = 1080;
pub const AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR: &str = "User/AutoGeniusInvokation";
pub const AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET: &str =
    "AutoGeniusInvokation:tcg_character_card.json";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationExecutionPlan {
    pub task_key: String,
    pub display_name: String,
    pub capture_size: Size,
    pub asset_scale: f64,
    pub config_rule: AutoGeniusInvokationConfigRule,
    pub strategy_source: AutoGeniusInvokationStrategySource,
    pub strategy: AutoGeniusStrategyPlan,
    pub startup_rule: AutoGeniusInvokationStartupRule,
    pub locators: AutoGeniusInvokationLocators,
    pub dice_rule: AutoGeniusInvokationDiceRule,
    pub action_rule: AutoGeniusInvokationActionRule,
    pub ocr_rule: AutoGeniusInvokationOcrRule,
    pub wait_rule: AutoGeniusInvokationWaitRule,
    pub exception_rule: AutoGeniusInvokationExceptionRule,
    pub steps: Vec<AutoGeniusInvokationStep>,
    pub executor_ready: bool,
    pub pending_native: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoGeniusInvokationExecutionConfig {
    pub capture_size: Size,
    pub asset_scale: f64,
    pub strategy_name: Option<String>,
    pub strategy: Option<String>,
    pub auto_genius_invokation_config: AutoGeniusInvokationConfig,
}

impl Default for AutoGeniusInvokationExecutionConfig {
    fn default() -> Self {
        Self {
            capture_size: Size::new(
                AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_WIDTH,
                AUTO_GENIUS_INVOKATION_DEFAULT_CAPTURE_HEIGHT,
            ),
            asset_scale: 1.0,
            strategy_name: None,
            strategy: None,
            auto_genius_invokation_config: AutoGeniusInvokationConfig::default(),
        }
    }
}

impl AutoGeniusInvokationExecutionConfig {
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
        if let Some(strategy) = string_member(value, ["strategy", "Strategy", "script", "Script"]) {
            config.strategy = Some(strategy);
        }

        let auto_genius_value = value
            .get("autoGeniusInvokationConfig")
            .or_else(|| value.get("AutoGeniusInvokationConfig"))
            .or_else(|| value.get("auto_genius_invokation_config"))
            .unwrap_or(value);
        config.auto_genius_invokation_config =
            serde_json::from_value(auto_genius_value.clone()).unwrap_or_default();

        config.strategy_name = string_member(
            value,
            [
                "strategyName",
                "StrategyName",
                "strategy_name",
                "autoGeniusInvokationStrategyName",
            ],
        )
        .or_else(|| {
            string_member(
                auto_genius_value,
                ["strategyName", "StrategyName", "strategy_name"],
            )
        });
        config
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationConfigRule {
    pub strategy_name: String,
    pub sleep_delay_ms: u64,
    pub sleep_delay_min_ms: u64,
    pub sleep_delay_max_ms: u64,
    pub default_character_card_rects: Vec<Rect>,
    pub active_character_card_space: i64,
    pub my_dice_count_rect: Rect,
    pub character_card_extend_hp_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStrategySource {
    pub strategy_name: String,
    pub user_strategy_directory: String,
    pub strategy_path: Option<String>,
    pub inline_strategy: bool,
    pub default_card_config_asset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusStrategyPlan {
    pub characters: Vec<AutoGeniusCharacterPlan>,
    pub action_commands: Vec<AutoGeniusActionCommandPlan>,
    pub skipped_line_count: usize,
    pub stage_order: Vec<String>,
    pub preserves_legacy_stage_headers: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusCharacterPlan {
    pub index: u8,
    pub name: String,
    pub element: Option<AutoGeniusElementalType>,
    pub skills: Vec<AutoGeniusSkillPlan>,
    pub uses_default_card_config: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusSkillPlan {
    pub index: u8,
    pub element: AutoGeniusElementalType,
    pub specific_element_cost: u8,
    pub any_element_cost: u8,
    pub all_cost: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusActionCommandPlan {
    pub character_index: u8,
    pub character_name: String,
    pub action: AutoGeniusActionKind,
    pub target_index: u8,
    pub dice_delta: i8,
    pub all_cost: Option<i16>,
    pub dice_element: Option<AutoGeniusElementalType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoGeniusActionKind {
    UseSkill,
    SwitchLater,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoGeniusElementalType {
    Omni,
    Cryo,
    Hydro,
    Pyro,
    Electro,
    Dendro,
    Anemo,
    Geo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStartupRule {
    pub skips_task_runner_main_ui_wait: bool,
    pub requires_exact_1920x1080: bool,
    pub sends_tcg_start_notification: bool,
    pub sends_tcg_end_notification: bool,
    pub destroys_asset_singleton_before_start: bool,
    pub initializes_control_with_cancellation_token: bool,
    pub prepares_initial_hand: bool,
    pub detects_character_rects_with_fallback: bool,
    pub chooses_first_action_character: bool,
    pub clears_draw_content_after_round: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationLocators {
    pub confirm_button: AutoGeniusTemplateLocator,
    pub round_end_button: AutoGeniusTemplateLocator,
    pub elemental_tuning_confirm_button: AutoGeniusTemplateLocator,
    pub exit_duel_button: AutoGeniusTemplateLocator,
    pub in_opponent_action: AutoGeniusTemplateLocator,
    pub end_phase: AutoGeniusTemplateLocator,
    pub elemental_dice_lack_warning: AutoGeniusTemplateLocator,
    pub character_taken_out: AutoGeniusTemplateLocator,
    pub in_character_pick: AutoGeniusTemplateLocator,
    pub character_hp_upper: AutoGeniusTemplateLocator,
    pub grayscale_assets: Vec<String>,
    pub roll_phase_dice_assets: Vec<AutoGeniusDiceAsset>,
    pub action_phase_dice_assets: Vec<AutoGeniusDiceAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusTemplateLocator {
    pub name: String,
    pub asset: String,
    pub roi: Option<Rect>,
    pub threshold: f64,
    pub match_mode: TemplateMatchMode,
    pub draw_on_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusDiceAsset {
    pub element: AutoGeniusElementalType,
    pub asset: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationDiceRule {
    pub initial_dice_count: u8,
    pub roll_phase_threshold: f64,
    pub roll_phase_expected_count: u8,
    pub roll_phase_expected_upper_count: u8,
    pub roll_phase_expected_lower_count: u8,
    pub roll_phase_initial_wait_ms: u64,
    pub roll_phase_retry_interval_ms: u64,
    pub roll_phase_retry_attempts: u64,
    pub post_roll_confirm_sleep_ms: u64,
    pub opponent_reroll_wait_ms: u64,
    pub action_phase_threshold: f64,
    pub action_phase_roi_right_width_divisor: u8,
    pub action_phase_count_retry_attempts: u64,
    pub action_phase_count_retry_interval_ms: u64,
    pub action_phase_expected_8_actual_9_omni_retry_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusInvokationActionRule {
    pub first_round_card_count: u8,
    pub next_round_card_increment: u8,
    pub switch_character_dice_cost: u8,
    pub switch_button_click_offset_from_right_1080p: f64,
    pub action_button_y_offset_from_bottom_1080p: f64,
    pub switch_button_double_click: bool,
    pub switch_animation_sleep_ms: u64,
    pub skill_click_offset_multiplier_1080p: f64,
    pub skill_popup_sleep_ms: u64,
    pub skill_confirm_sleep_ms: u64,
    pub skill_center_reset_before_click: bool,
    pub elemental_tuning_confirm_threshold: f64,
    pub elemental_tuning_hand_layouts: Vec<AutoGeniusHandLayout>,
    pub keqing_skill_2_alternates_card_count: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoGeniusHandLayout {
    pub card_count: u8,
    pub start_x_1080p: f64,
    pub spacing_1080p: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationOcrRule {
    pub dice_count_rect: Rect,
    pub dice_ocr_without_detector: bool,
    pub invalid_dice_count_sentinel: i32,
    pub replaces_circled_digit_text: bool,
    pub active_character_space_offset: i64,
    pub character_hp_empty_uses_active_offset: bool,
    pub active_character_fallback_by_exclusion: bool,
    pub active_character_fallback_by_template_shape: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationWaitRule {
    pub wait_my_turn_max_attempts: u64,
    pub wait_my_turn_interval_ms: u64,
    pub wait_my_turn_required_consecutive_hits: u64,
    pub wait_opponent_action_max_attempts: u64,
    pub wait_opponent_action_interval_ms: u64,
    pub default_after_action_wait_ms: u64,
    pub burst_after_action_wait_ms: u64,
    pub mona_switch_after_action_wait_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationExceptionRule {
    pub normal_end_is_logged_without_rethrow_in_duel: bool,
    pub task_cancelled_rethrows_from_duel: bool,
    pub outer_task_boundary_catches_all_and_logs: bool,
    pub check_task_verifies_game_foreground: bool,
    pub check_task_pause_retry_attempts: u64,
    pub check_task_pause_retry_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutoGeniusInvokationStep {
    pub phase: AutoGeniusInvokationPhase,
    pub action: AutoGeniusInvokationStepAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoGeniusInvokationPhase {
    Startup,
    Prepare,
    RoundLoop,
    RollDice,
    MyTurn,
    Action,
    ElementalTuning,
    RoundEnd,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AutoGeniusInvokationStepAction {
    ValidateResolution,
    ParseStrategy,
    NotifyStart,
    PrepareInitialHand,
    ResolveCharacterRects,
    ChooseFirstCharacter,
    PredictDiceTypes,
    ReRollDice,
    WaitForMyTurn,
    DetectActiveCharacter,
    CalibrateDiceCountByOcr,
    SwitchCharacterIfNeeded,
    UseSkillOrTuneCards,
    RemoveExecutedCommand,
    ClickRoundEnd,
    WaitOpponentActionAndEndPhase,
    NotifyEnd,
}

pub fn plan_auto_genius_invokation(
    working_directory: impl AsRef<Path>,
    config: AutoGeniusInvokationExecutionConfig,
) -> Result<AutoGeniusInvokationExecutionPlan> {
    let working_directory = working_directory.as_ref();
    let strategy_name = config
        .strategy_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| config.auto_genius_invokation_config.strategy_name.clone());
    let (strategy_source, strategy_text) =
        resolve_strategy_text(working_directory, &config, &strategy_name)?;
    let strategy = parse_auto_genius_strategy(&strategy_text)?;

    Ok(AutoGeniusInvokationExecutionPlan {
        task_key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
        display_name: AUTO_GENIUS_INVOKATION_DISPLAY_NAME.to_string(),
        capture_size: config.capture_size,
        asset_scale: config.asset_scale,
        config_rule: AutoGeniusInvokationConfigRule {
            strategy_name,
            sleep_delay_ms: config.auto_genius_invokation_config.sleep_delay,
            sleep_delay_min_ms: 0,
            sleep_delay_max_ms: 5_000,
            default_character_card_rects: config
                .auto_genius_invokation_config
                .default_character_card_rects
                .iter()
                .map(|rect| scale_rect(rect_config_to_rect(*rect), config.asset_scale))
                .collect(),
            active_character_card_space: config
                .auto_genius_invokation_config
                .active_character_card_space,
            my_dice_count_rect: scale_rect(
                Rect {
                    x: 68,
                    y: 642,
                    width: 25,
                    height: 31,
                },
                config.asset_scale,
            ),
            character_card_extend_hp_rect: scale_rect(
                Rect {
                    x: -20,
                    y: 0,
                    width: 60,
                    height: 55,
                },
                config.asset_scale,
            ),
        },
        strategy_source,
        strategy,
        startup_rule: AutoGeniusInvokationStartupRule {
            skips_task_runner_main_ui_wait: true,
            requires_exact_1920x1080: true,
            sends_tcg_start_notification: true,
            sends_tcg_end_notification: true,
            destroys_asset_singleton_before_start: true,
            initializes_control_with_cancellation_token: true,
            prepares_initial_hand: true,
            detects_character_rects_with_fallback: true,
            chooses_first_action_character: true,
            clears_draw_content_after_round: true,
        },
        locators: auto_genius_locators(config.capture_size),
        dice_rule: AutoGeniusInvokationDiceRule {
            initial_dice_count: 8,
            roll_phase_threshold: 0.73,
            roll_phase_expected_count: 8,
            roll_phase_expected_upper_count: 4,
            roll_phase_expected_lower_count: 4,
            roll_phase_initial_wait_ms: 5_000,
            roll_phase_retry_interval_ms: 500,
            roll_phase_retry_attempts: 35,
            post_roll_confirm_sleep_ms: 1_000,
            opponent_reroll_wait_ms: 5_000,
            action_phase_threshold: 0.7,
            action_phase_roi_right_width_divisor: 5,
            action_phase_count_retry_attempts: 20,
            action_phase_count_retry_interval_ms: 1_000,
            action_phase_expected_8_actual_9_omni_retry_limit: 5,
        },
        action_rule: AutoGeniusInvokationActionRule {
            first_round_card_count: 5,
            next_round_card_increment: 2,
            switch_character_dice_cost: 1,
            switch_button_click_offset_from_right_1080p: 100.0,
            action_button_y_offset_from_bottom_1080p: 120.0,
            switch_button_double_click: true,
            switch_animation_sleep_ms: 800,
            skill_click_offset_multiplier_1080p: 100.0,
            skill_popup_sleep_ms: 1_200,
            skill_confirm_sleep_ms: 500,
            skill_center_reset_before_click: true,
            elemental_tuning_confirm_threshold: 0.9,
            elemental_tuning_hand_layouts: elemental_tuning_hand_layouts(),
            keqing_skill_2_alternates_card_count: true,
        },
        ocr_rule: AutoGeniusInvokationOcrRule {
            dice_count_rect: scale_rect(
                Rect {
                    x: 68,
                    y: 642,
                    width: 25,
                    height: 31,
                },
                config.asset_scale,
            ),
            dice_ocr_without_detector: true,
            invalid_dice_count_sentinel: -10,
            replaces_circled_digit_text: true,
            active_character_space_offset: config
                .auto_genius_invokation_config
                .active_character_card_space,
            character_hp_empty_uses_active_offset: true,
            active_character_fallback_by_exclusion: true,
            active_character_fallback_by_template_shape: true,
        },
        wait_rule: AutoGeniusInvokationWaitRule {
            wait_my_turn_max_attempts: 60,
            wait_my_turn_interval_ms: 1_000,
            wait_my_turn_required_consecutive_hits: 3,
            wait_opponent_action_max_attempts: 60,
            wait_opponent_action_interval_ms: 1_000,
            default_after_action_wait_ms: 10_000,
            burst_after_action_wait_ms: 15_000,
            mona_switch_after_action_wait_ms: 3_000,
        },
        exception_rule: AutoGeniusInvokationExceptionRule {
            normal_end_is_logged_without_rethrow_in_duel: true,
            task_cancelled_rethrows_from_duel: true,
            outer_task_boundary_catches_all_and_logs: true,
            check_task_verifies_game_foreground: true,
            check_task_pause_retry_attempts: 100,
            check_task_pause_retry_interval_ms: 1_000,
        },
        steps: auto_genius_steps(),
        executor_ready: false,
        pending_native: vec![
            "TaskRunner solo-task lock, trigger clearing, window activation, and skip-main-UI launch boundary".to_string(),
            "live capture, template matching, OpenCV masking, OCR, and default tcg_character_card fallback loading".to_string(),
            "GeniusInvokationControl click/drag/input dispatch, pause handling, and game foreground checks".to_string(),
            "dice recognition/re-roll execution, elemental tuning drag attempts, and skill/switch clicks".to_string(),
            "character active/HP/status/energy recognition and duel loop mutation against live frames".to_string(),
            "notification dispatch and legacy outer task exception swallowing".to_string(),
        ],
    })
}

pub fn parse_auto_genius_strategy(script: &str) -> Result<AutoGeniusStrategyPlan> {
    let mut stage = String::new();
    let mut stage_order = Vec::new();
    let mut skipped_line_count = 0;
    let mut characters: Vec<Option<AutoGeniusCharacterPlan>> = vec![None, None, None, None];
    let mut action_commands = Vec::new();

    for (line_index, raw_line) in script.lines().enumerate() {
        let line = raw_line.trim();
        if line.contains(':') {
            stage = line.to_string();
            if !stage_order.contains(&stage) {
                stage_order.push(stage.clone());
            }
            continue;
        }
        if line == "---" || line.starts_with("//") || line.is_empty() {
            skipped_line_count += 1;
            continue;
        }

        match stage.as_str() {
            "角色定义:" => {
                let character = parse_auto_genius_character(line, line_index + 1)?;
                let index = character.index as usize;
                characters[index] = Some(character);
            }
            "策略定义:" => {
                let defined_characters: Vec<_> = characters.iter().flatten().cloned().collect();
                if defined_characters.len() != 3 {
                    return Err(strategy_error(line_index + 1, "角色未定义"));
                }
                action_commands.push(parse_auto_genius_action(
                    line,
                    line_index + 1,
                    &defined_characters,
                )?);
            }
            _ => {
                return Err(strategy_error(
                    line_index + 1,
                    format!("未知的定义字段：{stage}"),
                ));
            }
        }
    }

    let characters: Vec<_> = characters.into_iter().flatten().collect();
    if characters.len() != 3 {
        return Err(strategy_error(
            script.lines().count().max(1),
            "角色未定义，请确认策略文本格式是否为UTF-8",
        ));
    }

    Ok(AutoGeniusStrategyPlan {
        characters,
        action_commands,
        skipped_line_count,
        stage_order,
        preserves_legacy_stage_headers: true,
    })
}

fn parse_auto_genius_character(line: &str, line_number: usize) -> Result<AutoGeniusCharacterPlan> {
    let (header, skill_block) = line
        .split_once('{')
        .map(|(header, rest)| (header, Some(rest.trim_end_matches('}'))))
        .unwrap_or((line, None));
    let (index_text, value) = header
        .split_once('=')
        .ok_or_else(|| strategy_error(line_number, "角色定义解析错误"))?;
    let index = digits_from(index_text)
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "角色序号必须在1-3之间"))?;
    if !(1..=3).contains(&index) {
        return Err(strategy_error(line_number, "角色序号必须在1-3之间"));
    }

    if let Some((name, element)) = value.split_once('|') {
        let element = chinese_to_elemental_type(
            element
                .chars()
                .next()
                .ok_or_else(|| strategy_error(line_number, "角色元素解析错误"))?,
        )
        .map_err(|message| strategy_error(line_number, message))?;
        let skills = skill_block
            .ok_or_else(|| strategy_error(line_number, "角色技能定义缺失"))?
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| parse_auto_genius_skill(part.trim(), line_number))
            .collect::<Result<Vec<_>>>()?;
        Ok(AutoGeniusCharacterPlan {
            index,
            name: name.to_string(),
            element: Some(element),
            skills,
            uses_default_card_config: false,
        })
    } else {
        Ok(AutoGeniusCharacterPlan {
            index,
            name: value.to_string(),
            element: None,
            skills: Vec::new(),
            uses_default_card_config: true,
        })
    }
}

fn parse_auto_genius_skill(line: &str, line_number: usize) -> Result<AutoGeniusSkillPlan> {
    let (skill_name, cost_text) = line
        .split_once('=')
        .ok_or_else(|| strategy_error(line_number, "技能定义解析错误"))?;
    let index = digits_from(skill_name)
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "技能序号必须在1-5之间"))?;
    if !(1..=5).contains(&index) {
        return Err(strategy_error(line_number, "技能序号必须在1-5之间"));
    }

    let mut parts = cost_text.split('+');
    let specific = parts
        .next()
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?;
    let specific_element_cost = specific
        .chars()
        .next()
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?
        .to_digit(10)
        .ok_or_else(|| strategy_error(line_number, "技能消耗解析错误"))?
        as u8;
    let element = chinese_to_elemental_type(
        specific
            .chars()
            .nth(1)
            .ok_or_else(|| strategy_error(line_number, "技能元素解析错误"))?,
    )
    .map_err(|message| strategy_error(line_number, message))?;
    let any_element_cost = parts
        .next()
        .and_then(|part| part.chars().next())
        .and_then(|value| value.to_digit(10))
        .unwrap_or(0) as u8;

    Ok(AutoGeniusSkillPlan {
        index,
        element,
        specific_element_cost,
        any_element_cost,
        all_cost: specific_element_cost + any_element_cost,
    })
}

fn parse_auto_genius_action(
    line: &str,
    line_number: usize,
    characters: &[AutoGeniusCharacterPlan],
) -> Result<AutoGeniusActionCommandPlan> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() < 3 || parts.len() > 4 || parts[1] != "使用" {
        return Err(strategy_error(line_number, "策略中的行动命令解析错误"));
    }
    let character = characters
        .iter()
        .find(|character| character.name == parts[0])
        .ok_or_else(|| {
            strategy_error(
                line_number,
                "策略中的行动命令解析错误：角色名称无法从角色定义中匹配到",
            )
        })?;
    let target_index = digits_from(parts[2])
        .parse::<u8>()
        .map_err(|_| strategy_error(line_number, "策略中的行动命令解析错误：技能编号错误"))?;
    if target_index >= 5 {
        return Err(strategy_error(
            line_number,
            "策略中的行动命令解析错误：技能编号错误",
        ));
    }
    let dice_delta = if let Some(delta) = parts.get(3) {
        if let Some(value) = delta.strip_prefix("骰子增加") {
            digits_from(value).parse::<i8>().map_err(|_| {
                strategy_error(
                    line_number,
                    "策略中的行动命令解析错误：骰子增减参数格式不正确",
                )
            })?
        } else if let Some(value) = delta.strip_prefix("骰子减少") {
            -digits_from(value).parse::<i8>().map_err(|_| {
                strategy_error(
                    line_number,
                    "策略中的行动命令解析错误：骰子增减参数格式不正确",
                )
            })?
        } else {
            return Err(strategy_error(
                line_number,
                format!(
                    "策略中的行动命令解析错误：骰子增减参数格式不正确（应为 骰子增加N 或 骰子减少N ），实际：{delta}"
                ),
            ));
        }
    } else {
        0
    };

    let skill = character
        .skills
        .iter()
        .find(|skill| skill.index == target_index);
    Ok(AutoGeniusActionCommandPlan {
        character_index: character.index,
        character_name: character.name.clone(),
        action: AutoGeniusActionKind::UseSkill,
        target_index,
        dice_delta,
        all_cost: skill.map(|skill| skill.all_cost as i16 + dice_delta as i16),
        dice_element: skill.map(|skill| skill.element),
    })
}

fn resolve_strategy_text(
    working_directory: &Path,
    config: &AutoGeniusInvokationExecutionConfig,
    strategy_name: &str,
) -> Result<(AutoGeniusInvokationStrategySource, String)> {
    if let Some(strategy) = config.strategy.clone() {
        return Ok((
            AutoGeniusInvokationStrategySource {
                strategy_name: strategy_name.to_string(),
                user_strategy_directory: AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR.to_string(),
                strategy_path: None,
                inline_strategy: true,
                default_card_config_asset: AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET.to_string(),
            },
            strategy,
        ));
    }

    let strategy_path = normalize_auto_genius_strategy_path(strategy_name)?;
    let absolute_strategy_path = working_directory.join(&strategy_path);
    let strategy = fs::read_to_string(&absolute_strategy_path).map_err(|error| {
        TaskError::InvalidTaskConfig {
            key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
            message: format!(
                "failed to read AutoGeniusInvokation strategy {}: {error}",
                absolute_strategy_path.display()
            ),
        }
    })?;

    Ok((
        AutoGeniusInvokationStrategySource {
            strategy_name: strategy_name.to_string(),
            user_strategy_directory: AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR.to_string(),
            strategy_path: Some(strategy_path.to_string_lossy().replace('\\', "/")),
            inline_strategy: false,
            default_card_config_asset: AUTO_GENIUS_INVOKATION_DEFAULT_CARD_ASSET.to_string(),
        },
        strategy,
    ))
}

pub fn normalize_auto_genius_strategy_path(strategy_name: &str) -> Result<PathBuf> {
    let strategy_name = strategy_name.trim();
    if strategy_name.is_empty() {
        return Err(TaskError::InvalidTaskConfig {
            key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
            message: "AutoGeniusInvokation strategy name is empty".to_string(),
        });
    }
    let mut relative = PathBuf::from(AUTO_GENIUS_INVOKATION_USER_STRATEGY_DIR);
    let mut name_path = PathBuf::from(strategy_name.replace('\\', "/"));
    if name_path.extension().is_none() {
        name_path.set_extension("txt");
    }
    for component in name_path.components() {
        match component {
            Component::Normal(value) => relative.push(value),
            _ => {
                return Err(TaskError::InvalidTaskConfig {
                    key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
                    message: format!("invalid AutoGeniusInvokation strategy path: {strategy_name}"),
                });
            }
        }
    }
    Ok(relative)
}

fn auto_genius_locators(capture_size: Size) -> AutoGeniusInvokationLocators {
    AutoGeniusInvokationLocators {
        confirm_button: template(
            "ConfirmButton",
            "AutoGeniusInvokation:other/确定.png",
            None,
            0.8,
            false,
        ),
        round_end_button: template(
            "RoundEndButton",
            "AutoGeniusInvokation:other/回合结束.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        elemental_tuning_confirm_button: template(
            "ElementalTuningConfirmButton",
            "AutoGeniusInvokation:other/元素调和.png",
            Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: capture_size.width as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.9,
            false,
        ),
        exit_duel_button: template(
            "ExitDuelButton",
            "AutoGeniusInvokation:other/退出挑战.png",
            Some(Rect {
                x: 0,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.8,
            true,
        ),
        in_opponent_action: template(
            "InOpponentAction",
            "AutoGeniusInvokation:other/对方行动中.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        end_phase: template(
            "EndPhase",
            "AutoGeniusInvokation:other/回合结算阶段.png",
            Some(Rect {
                x: 0,
                y: 0,
                width: (capture_size.width / 5) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        elemental_dice_lack_warning: template(
            "ElementalDiceLackWarning",
            "AutoGeniusInvokation:other/元素骰子不足.png",
            Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: 0,
                width: (capture_size.width / 2) as i32,
                height: capture_size.height as i32,
            }),
            0.8,
            true,
        ),
        character_taken_out: template(
            "CharacterTakenOut",
            "AutoGeniusInvokation:other/角色死亡.png",
            None,
            0.8,
            true,
        ),
        in_character_pick: template(
            "InCharacterPick",
            "AutoGeniusInvokation:other/出战角色.png",
            Some(Rect {
                x: (capture_size.width / 2) as i32,
                y: (capture_size.height / 2) as i32,
                width: (capture_size.width / 2) as i32,
                height: (capture_size.height / 2) as i32,
            }),
            0.8,
            true,
        ),
        character_hp_upper: template(
            "CharacterHpUpper",
            "AutoGeniusInvokation:other/角色血量上方.png",
            None,
            0.8,
            true,
        ),
        grayscale_assets: vec![
            "AutoGeniusInvokation:other/角色被打败.png".to_string(),
            "AutoGeniusInvokation:other/角色状态_冻结.png".to_string(),
            "AutoGeniusInvokation:other/角色状态_水泡.png".to_string(),
            "AutoGeniusInvokation:other/满能量.png".to_string(),
        ],
        roll_phase_dice_assets: dice_assets("roll"),
        action_phase_dice_assets: dice_assets("action"),
    }
}

fn template(
    name: &str,
    asset: &str,
    roi: Option<Rect>,
    threshold: f64,
    draw_on_window: bool,
) -> AutoGeniusTemplateLocator {
    AutoGeniusTemplateLocator {
        name: name.to_string(),
        asset: asset.to_string(),
        roi,
        threshold,
        match_mode: TemplateMatchMode::CCoeffNormed,
        draw_on_window,
    }
}

fn dice_assets(prefix: &str) -> Vec<AutoGeniusDiceAsset> {
    [
        (AutoGeniusElementalType::Anemo, "anemo"),
        (AutoGeniusElementalType::Electro, "electro"),
        (AutoGeniusElementalType::Dendro, "dendro"),
        (AutoGeniusElementalType::Hydro, "hydro"),
        (AutoGeniusElementalType::Pyro, "pyro"),
        (AutoGeniusElementalType::Cryo, "cryo"),
        (AutoGeniusElementalType::Geo, "geo"),
        (AutoGeniusElementalType::Omni, "omni"),
    ]
    .into_iter()
    .map(|(element, name)| AutoGeniusDiceAsset {
        element,
        asset: format!("AutoGeniusInvokation:dice/{prefix}_{name}.png"),
    })
    .collect()
}

fn elemental_tuning_hand_layouts() -> Vec<AutoGeniusHandLayout> {
    [
        (10, 570.0, 120.0),
        (9, 570.0, 130.0),
        (8, 600.0, 145.0),
        (7, 630.0, 160.0),
        (6, 620.0, 200.0),
        (5, 720.0, 200.0),
        (4, 820.0, 200.0),
        (3, 920.0, 200.0),
        (2, 1020.0, 200.0),
        (1, 1120.0, 200.0),
    ]
    .into_iter()
    .map(
        |(card_count, start_x_1080p, spacing_1080p)| AutoGeniusHandLayout {
            card_count,
            start_x_1080p,
            spacing_1080p,
        },
    )
    .collect()
}

fn auto_genius_steps() -> Vec<AutoGeniusInvokationStep> {
    use AutoGeniusInvokationPhase::*;
    use AutoGeniusInvokationStepAction::*;
    vec![
        AutoGeniusInvokationStep {
            phase: Startup,
            action: ValidateResolution,
        },
        AutoGeniusInvokationStep {
            phase: Startup,
            action: ParseStrategy,
        },
        AutoGeniusInvokationStep {
            phase: Startup,
            action: NotifyStart,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: PrepareInitialHand,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: ResolveCharacterRects,
        },
        AutoGeniusInvokationStep {
            phase: Prepare,
            action: ChooseFirstCharacter,
        },
        AutoGeniusInvokationStep {
            phase: RollDice,
            action: PredictDiceTypes,
        },
        AutoGeniusInvokationStep {
            phase: RollDice,
            action: ReRollDice,
        },
        AutoGeniusInvokationStep {
            phase: MyTurn,
            action: WaitForMyTurn,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: DetectActiveCharacter,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: CalibrateDiceCountByOcr,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: SwitchCharacterIfNeeded,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: UseSkillOrTuneCards,
        },
        AutoGeniusInvokationStep {
            phase: Action,
            action: RemoveExecutedCommand,
        },
        AutoGeniusInvokationStep {
            phase: RoundEnd,
            action: ClickRoundEnd,
        },
        AutoGeniusInvokationStep {
            phase: RoundEnd,
            action: WaitOpponentActionAndEndPhase,
        },
        AutoGeniusInvokationStep {
            phase: Cleanup,
            action: NotifyEnd,
        },
    ]
}

fn rect_config_to_rect(rect: RectConfig) -> Rect {
    Rect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn scale_rect(rect: Rect, scale: f64) -> Rect {
    Rect {
        x: (rect.x as f64 * scale) as i32,
        y: (rect.y as f64 * scale) as i32,
        width: (rect.width as f64 * scale) as i32,
        height: (rect.height as f64 * scale) as i32,
    }
}

fn chinese_to_elemental_type(value: char) -> std::result::Result<AutoGeniusElementalType, String> {
    match value {
        '全' => Ok(AutoGeniusElementalType::Omni),
        '冰' => Ok(AutoGeniusElementalType::Cryo),
        '水' => Ok(AutoGeniusElementalType::Hydro),
        '火' => Ok(AutoGeniusElementalType::Pyro),
        '雷' => Ok(AutoGeniusElementalType::Electro),
        '草' => Ok(AutoGeniusElementalType::Dendro),
        '风' => Ok(AutoGeniusElementalType::Anemo),
        '岩' => Ok(AutoGeniusElementalType::Geo),
        _ => Err(format!("unknown elemental type: {value}")),
    }
}

fn digits_from(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

fn strategy_error(line: usize, message: impl Into<String>) -> TaskError {
    TaskError::InvalidTaskConfig {
        key: AUTO_GENIUS_INVOKATION_TASK_KEY.to_string(),
        message: format!("strategy parse error at line {line}: {}", message.into()),
    }
}

fn capture_size_from_value(value: &Value) -> Option<Size> {
    value
        .get("captureSize")
        .or_else(|| value.get("CaptureSize"))
        .or_else(|| value.get("capture_size"))
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn f64_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<f64> {
    member_value(value, &names).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|value| value.parse::<f64>().ok()))
    })
}

fn string_member<const N: usize>(value: &Value, names: [&str; N]) -> Option<String> {
    member_value(value, &names).and_then(|value| value.as_str().map(str::to_string))
}

fn member_value<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| value.get(*name))
}
