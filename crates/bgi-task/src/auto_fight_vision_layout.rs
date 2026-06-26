use super::*;
use bgi_vision::{BgrImage, Rect, Size};
use std::path::Path;

pub fn default_combat_avatar_index_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_INDEX_RECTS_1080P)
}

pub fn default_combat_avatar_index_template_roi(size: Size) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (_, y, width, height) = AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ROI_1080P;
    task_vision_result(Rect::new(
        size.width as i32 - (65.0 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn default_combat_avatar_side_icon_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_SIDE_ICON_RECTS_1080P)
}

fn scaled_1080p_rects(size: Size, rects: &[(i32, i32, i32, i32)]) -> Result<Vec<Rect>> {
    rects
        .iter()
        .copied()
        .map(|rect| scaled_1080p_rect(size, rect))
        .collect()
}

fn scaled_1080p_rect(size: Size, rect: (i32, i32, i32, i32)) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (x, y, width, height) = rect;
    task_vision_result(Rect::new(
        size.width as i32 - ((1920 - x) as f64 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn combat_multi_game_detection_from_icon_counts(
    p_icon_count: usize,
    one_p_icon_found: bool,
) -> Result<CombatMultiGameDetection> {
    if p_icon_count > 0 {
        let player_count = p_icon_count + 1;
        if player_count > 4 {
            return Err(TaskError::VisionPlan(
                "当前处于联机状态，但是队伍人数超过4人，无法识别".to_string(),
            ));
        }
        let status = CombatMultiGameStatus {
            is_in_multi_game: true,
            is_host: one_p_icon_found,
            player_count,
        };
        let message = if one_p_icon_found {
            format!("当前处于联机状态，且当前账号是房主，联机人数{player_count}人")
        } else {
            format!("当前处于联机状态，且在别人世界中，联机人数{player_count}人")
        };
        return Ok(CombatMultiGameDetection {
            status,
            p_icon_count,
            one_p_icon_found,
            message,
        });
    }

    if one_p_icon_found {
        return Ok(CombatMultiGameDetection {
            status: CombatMultiGameStatus {
                is_in_multi_game: true,
                is_host: true,
                player_count: 1,
            },
            p_icon_count,
            one_p_icon_found,
            message: "当前处于联机状态，但是没有其他玩家连入".to_string(),
        });
    }

    Ok(CombatMultiGameDetection {
        status: CombatMultiGameStatus::default(),
        p_icon_count,
        one_p_icon_found,
        message: "current scene is not in co-op mode".to_string(),
    })
}

pub fn detect_combat_multi_game_status(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatMultiGameDetection> {
    let p_icon_count = find_auto_fight_template_matches(
        &working_directory,
        image,
        AUTO_FIGHT_COOP_P_ASSET,
        default_auto_fight_coop_p_roi(image.size)?,
        "P",
        4,
    )?
    .len();
    let one_p_icon_found = !find_auto_fight_template_matches(
        working_directory,
        image,
        AUTO_FIGHT_COOP_ONE_P_ASSET,
        default_auto_fight_coop_one_p_roi(image.size)?,
        "1P",
        1,
    )?
    .is_empty();
    combat_multi_game_detection_from_icon_counts(p_icon_count, one_p_icon_found)
}

pub fn default_auto_fight_coop_one_p_roi(size: Size) -> Result<Rect> {
    task_vision_result(Rect::new(
        0,
        0,
        size.width as i32 / 4,
        size.height as i32 / 7,
    ))
}

pub fn default_auto_fight_coop_p_roi(size: Size) -> Result<Rect> {
    let width = (size.width as f64 / 12.5).round() as i32;
    let height = size.height as i32 / 2 - size.width as i32 / 7;
    task_vision_result(Rect::new(
        size.width as i32 - width,
        size.height as i32 / 5,
        width,
        height,
    ))
}

pub fn combat_avatar_index_rects_for_multi_game_status(
    size: Size,
    status: CombatMultiGameStatus,
) -> Result<Vec<Rect>> {
    if let Some(key) = status.rect_map_key() {
        if let Some((_, rects)) = AUTO_FIGHT_COOP_SIDE_INDEX_RECTS_1080P
            .iter()
            .find(|(entry_key, _)| *entry_key == key)
        {
            return scaled_1080p_rects(size, rects);
        }
    }
    default_combat_avatar_index_rects(size)
}

pub fn combat_avatar_side_icon_rects_for_multi_game_status(
    size: Size,
    status: CombatMultiGameStatus,
) -> Result<Vec<Rect>> {
    if let Some(key) = status.rect_map_key() {
        if let Some((_, rects)) = AUTO_FIGHT_COOP_SIDE_ICON_RECTS_1080P
            .iter()
            .find(|(entry_key, _)| *entry_key == key)
        {
            return scaled_1080p_rects(size, rects);
        }
    }
    default_combat_avatar_side_icon_rects(size)
}

pub fn default_current_avatar_threshold_roi(size: Size) -> Result<Rect> {
    let scale = size.height as f64 / 1080.0;
    let (_, y, width, height) = AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ROI_1080P;
    task_vision_result(Rect::new(
        size.width as i32 - (240.0 * scale).round() as i32,
        (y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub fn active_combat_skill_cooldown_rect(size: Size, is_burst: bool) -> Result<Rect> {
    let rect = if is_burst {
        AUTO_FIGHT_ACTIVE_BURST_COOLDOWN_RECT_1080P
    } else {
        AUTO_FIGHT_ACTIVE_SKILL_COOLDOWN_RECT_1080P
    };
    scaled_1080p_rect(size, rect)
}

pub fn default_combat_avatar_side_burst_rects(size: Size) -> Result<Vec<Rect>> {
    scaled_1080p_rects(size, &AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P)
}

pub fn combat_avatar_side_burst_rect_for_index(size: Size, index: usize) -> Result<Rect> {
    if !(1..=AUTO_FIGHT_AVATAR_SIDE_BURST_RECTS_1080P.len()).contains(&index) {
        return Err(TaskError::VisionPlan(format!(
            "combat avatar side burst index {index} is outside the supported party range"
        )));
    }
    default_combat_avatar_side_burst_rects(size)?
        .get(index - 1)
        .copied()
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "combat avatar side burst rect {index} is unavailable"
            ))
        })
}

pub fn combat_avatar_side_icon_rects_for_index_rects(
    image_size: Size,
    index_rects: &[Rect],
) -> Result<Vec<Rect>> {
    index_rects
        .iter()
        .copied()
        .map(|rect| combat_avatar_side_icon_rect_from_index_rect(image_size, rect))
        .collect()
}

pub fn combat_avatar_side_icon_rect_from_index_rect(
    image_size: Size,
    index_rect: Rect,
) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let (offset_x, offset_y, width, height) = AUTO_FIGHT_AVATAR_SIDE_ICON_FROM_INDEX_RECT_1080P;
    task_vision_result(Rect::new(
        index_rect.x + (offset_x as f64 * scale).round() as i32,
        index_rect.y + (offset_y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}
