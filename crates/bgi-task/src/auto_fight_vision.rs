use super::*;
use bgi_vision::{crop_bgr_image, BgrImage, Rect};
use std::path::Path;

#[path = "auto_fight_vision_finish.rs"]
mod vision_finish;
#[path = "auto_fight_vision_layout.rs"]
mod vision_layout;
#[path = "auto_fight_vision_pixels.rs"]
mod vision_pixels;
#[path = "auto_fight_vision_skill.rs"]
mod vision_skill;
#[path = "auto_fight_vision_templates.rs"]
mod vision_templates;

pub use vision_finish::*;
pub use vision_layout::*;
pub use vision_pixels::detect_side_burst_circle;
use vision_pixels::{
    avatar_index_white_edge_ratio, count_gray_range, gray_region, is_avatar_index_white_rect,
    most_different_avatar_index,
};
pub use vision_skill::*;
use vision_templates::{
    avatar_index_rects_are_contiguous, display_avatar_costume_name,
    find_auto_fight_template_matches, find_combat_avatar_index_template_rect,
    find_current_avatar_arrow_template_rect, first_known_avatar_index_rect,
    index_rect_from_current_avatar_arrow, infer_avatar_index_rects_from_known_and_arrow,
    rects_intersect_vertically, split_avatar_side_class_name, validate_avatar_side_classification,
};

fn task_vision_result<T>(result: bgi_vision::Result<T>) -> Result<T> {
    result.map_err(|error| TaskError::VisionPlan(error.to_string()))
}

pub fn detect_active_combat_avatar_index_from_default_rects(
    image: &BgrImage,
) -> Result<CombatActiveAvatarDetectionResult> {
    let rects = default_combat_avatar_index_rects(image.size)?;
    detect_active_combat_avatar_index_by_color(image, &rects)
}

pub fn detect_active_combat_avatar_index_from_default_rects_with_arrow(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatActiveAvatarDetectionResult> {
    let rects =
        combat_avatar_index_rect_detection_for_active_avatar_detection(&working_directory, image)?
            .resolved_rects;
    let roi = default_current_avatar_threshold_roi(image.size)?;
    detect_active_combat_avatar_index_by_color_then_arrow(working_directory, image, &rects, roi)
}

pub fn combat_avatar_index_rect_detection_for_active_avatar_detection(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatAvatarIndexRectsDetection> {
    match detect_combat_multi_game_status(&working_directory, image) {
        Ok(detection) if detection.status.is_in_multi_game => {
            let rects =
                combat_avatar_index_rects_for_multi_game_status(image.size, detection.status)?;
            Ok(CombatAvatarIndexRectsDetection {
                rects_by_index: rects.iter().copied().map(Some).collect(),
                resolved_rects: rects,
                inferred_from_current_avatar_arrow: false,
                message: detection.message,
            })
        }
        _ => match detect_combat_avatar_index_rects_from_templates(&working_directory, image) {
            Ok(dynamic_rects)
                if dynamic_rects.resolved_rects.len()
                    == AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS.len()
                    || dynamic_rects.inferred_from_current_avatar_arrow =>
            {
                Ok(dynamic_rects)
            }
            _ => {
                let rects = default_combat_avatar_index_rects(image.size)?;
                Ok(CombatAvatarIndexRectsDetection {
                    rects_by_index: rects.iter().copied().map(Some).collect(),
                    resolved_rects: rects,
                    inferred_from_current_avatar_arrow: false,
                    message: "fell back to default avatar index rectangles".to_string(),
                })
            }
        },
    }
}

pub fn detect_combat_avatar_index_rects_from_templates(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
) -> Result<CombatAvatarIndexRectsDetection> {
    let index_roi = default_combat_avatar_index_template_roi(image.size)?;
    let arrow_roi = default_current_avatar_threshold_roi(image.size)?;
    detect_combat_avatar_index_rects_from_templates_in(
        working_directory,
        image,
        index_roi,
        arrow_roi,
    )
}

pub fn detect_combat_avatar_index_rects_from_templates_in(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    index_roi: Rect,
    arrow_roi: Rect,
) -> Result<CombatAvatarIndexRectsDetection> {
    let mut rects_by_index = vec![None; AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS.len()];
    for (index, asset_name) in AUTO_FIGHT_AVATAR_INDEX_TEMPLATE_ASSETS
        .iter()
        .copied()
        .enumerate()
    {
        rects_by_index[index] = find_combat_avatar_index_template_rect(
            &working_directory,
            image,
            asset_name,
            index_roi,
            &format!("Index{}", index + 1),
        )?;
    }

    let existing_count = rects_by_index.iter().filter(|rect| rect.is_some()).count();
    if existing_count == rects_by_index.len() {
        return Ok(CombatAvatarIndexRectsDetection {
            resolved_rects: rects_by_index.iter().flatten().copied().collect(),
            rects_by_index,
            inferred_from_current_avatar_arrow: false,
            message: "detected all avatar index rectangles by template matching".to_string(),
        });
    }

    let current_avatar =
        find_current_avatar_arrow_template_rect(&working_directory, image, arrow_roi)?;
    if let Some(current_avatar) = current_avatar {
        if let Some((known_index, known_rect)) = first_known_avatar_index_rect(&rects_by_index) {
            let inferred = infer_avatar_index_rects_from_known_and_arrow(
                image.size,
                &rects_by_index,
                known_index,
                known_rect,
                current_avatar,
            )?;
            if avatar_index_rects_are_contiguous(&inferred) {
                let resolved_rects = inferred.iter().flatten().copied().collect();
                return Ok(CombatAvatarIndexRectsDetection {
                    rects_by_index: inferred,
                    resolved_rects,
                    inferred_from_current_avatar_arrow: true,
                    message: "inferred a missing active avatar index rectangle from the current-avatar arrow".to_string(),
                });
            }
        } else {
            let inferred = index_rect_from_current_avatar_arrow(image.size, current_avatar)?;
            return Ok(CombatAvatarIndexRectsDetection {
                rects_by_index: vec![Some(inferred), None, None, None],
                resolved_rects: vec![inferred],
                inferred_from_current_avatar_arrow: true,
                message: "inferred a single avatar index rectangle from the current-avatar arrow"
                    .to_string(),
            });
        }
    }

    Ok(CombatAvatarIndexRectsDetection {
        resolved_rects: rects_by_index.iter().flatten().copied().collect(),
        rects_by_index,
        inferred_from_current_avatar_arrow: false,
        message: format!("detected {existing_count} avatar index rectangles by template matching"),
    })
}

pub fn recognize_combat_team_from_avatar_side_icons(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    action_scheduler_by_cd: &str,
    classifier: &mut impl CombatAvatarSideClassifier,
) -> Result<CombatTeamRecognitionExecution> {
    let catalog = read_combat_avatar_catalog(&working_directory)?;
    let multi_game_detection = detect_combat_multi_game_status(&working_directory, image)
        .unwrap_or_else(|error| CombatMultiGameDetection {
            status: CombatMultiGameStatus::default(),
            p_icon_count: 0,
            one_p_icon_found: false,
            message: format!(
                "co-op status detection failed; using single-player rectangles: {error}"
            ),
        });
    let index_rect_detection = if multi_game_detection.status.is_in_multi_game {
        let rects = combat_avatar_index_rects_for_multi_game_status(
            image.size,
            multi_game_detection.status,
        )?;
        CombatAvatarIndexRectsDetection {
            rects_by_index: rects.iter().copied().map(Some).collect(),
            resolved_rects: rects,
            inferred_from_current_avatar_arrow: false,
            message: multi_game_detection.message.clone(),
        }
    } else {
        match detect_combat_avatar_index_rects_from_templates(&working_directory, image) {
            Ok(detection) if !detection.resolved_rects.is_empty() => detection,
            _ => {
                let rects = default_combat_avatar_index_rects(image.size)?;
                CombatAvatarIndexRectsDetection {
                    rects_by_index: rects.iter().copied().map(Some).collect(),
                    resolved_rects: rects,
                    inferred_from_current_avatar_arrow: false,
                    message: "fell back to default avatar index rectangles".to_string(),
                }
            }
        }
    };
    let side_icon_rects = if multi_game_detection.status.is_in_multi_game {
        combat_avatar_side_icon_rects_for_multi_game_status(
            image.size,
            multi_game_detection.status,
        )?
    } else {
        combat_avatar_side_icon_rects_for_index_rects(
            image.size,
            &index_rect_detection.resolved_rects,
        )?
    };
    if side_icon_rects.len() != index_rect_detection.resolved_rects.len() {
        return Err(TaskError::VisionPlan(format!(
            "avatar index rectangle count ({}) does not match side-icon rectangle count ({})",
            index_rect_detection.resolved_rects.len(),
            side_icon_rects.len()
        )));
    }
    let mut avatars = Vec::with_capacity(side_icon_rects.len());
    for (position, (index_rect, side_icon_rect)) in index_rect_detection
        .resolved_rects
        .iter()
        .copied()
        .zip(side_icon_rects.iter().copied())
        .enumerate()
    {
        let crop = task_vision_result(crop_bgr_image(image, side_icon_rect))?;
        let classification =
            classifier.classify_avatar_side(position + 1, &crop, side_icon_rect)?;
        let recognition = combat_avatar_side_recognition_from_classification(
            &catalog,
            position + 1,
            index_rect,
            side_icon_rect,
            classification,
        )?;
        avatars.push(recognition);
    }
    let team_avatar_names: Vec<_> = avatars
        .iter()
        .map(|avatar| avatar.avatar_name.clone())
        .collect();
    let team_plan = plan_combat_team(
        &catalog,
        &team_avatar_names,
        &team_avatar_names,
        action_scheduler_by_cd,
    )?;
    Ok(CombatTeamRecognitionExecution {
        index_rect_detection,
        avatars,
        team_avatar_names,
        team_plan,
    })
}

pub fn combat_avatar_side_recognition_from_classification(
    catalog: &CombatAvatarCatalog,
    index: usize,
    index_rect: Rect,
    side_icon_rect: Rect,
    classification: CombatAvatarSideClassification,
) -> Result<CombatAvatarSideRecognition> {
    validate_avatar_side_classification(index, &classification)?;
    let (name_en, costume_name) = split_avatar_side_class_name(&classification.class_name);
    let avatar = catalog.avatar_by_name_en(&name_en).ok_or_else(|| {
        TaskError::CombatStrategy(format!(
            "avatar-side classifier returned an unknown class for slot {index}: {}",
            classification.class_name
        ))
    })?;
    let display_name = costume_name
        .as_deref()
        .map(|costume| format!("{}({})", avatar.name, display_avatar_costume_name(costume)))
        .unwrap_or_else(|| avatar.name.clone());
    Ok(CombatAvatarSideRecognition {
        index,
        avatar_name: avatar.name.clone(),
        name_en,
        costume_name,
        display_name,
        confidence: classification.confidence,
        index_rect,
        side_icon_rect,
    })
}

pub fn detect_active_combat_avatar_index_by_color_then_arrow(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    rects: &[Rect],
    arrow_roi: Rect,
) -> Result<CombatActiveAvatarDetectionResult> {
    let color_result = detect_active_combat_avatar_index_by_color(image, rects)?;
    if color_result.active_index.is_some() {
        return Ok(color_result);
    }

    detect_active_combat_avatar_index_by_arrow_template(
        working_directory,
        image,
        rects,
        arrow_roi,
        color_result,
    )
}

pub fn detect_active_combat_avatar_index_by_color(
    image: &BgrImage,
    rects: &[Rect],
) -> Result<CombatActiveAvatarDetectionResult> {
    if rects.is_empty() {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: None,
            method: CombatActiveAvatarDetectionMethod::Unresolved,
            rects: Vec::new(),
            white_rect_count: 0,
            not_white_rect_index: None,
            edge_white_ratios: Vec::new(),
            difference_votes: Vec::new(),
            message: "no avatar index rectangles were provided".to_string(),
        });
    }
    if rects.len() == 1 {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: Some(1),
            method: CombatActiveAvatarDetectionMethod::SingleAvatar,
            rects: rects.to_vec(),
            white_rect_count: 0,
            not_white_rect_index: Some(1),
            edge_white_ratios: Vec::new(),
            difference_votes: Vec::new(),
            message: "single controllable avatar is active by definition".to_string(),
        });
    }

    let mut white_rect_count = 0;
    let mut not_white_rect_index = None;
    let mut gray_regions = Vec::with_capacity(rects.len());
    for (index, rect) in rects.iter().copied().enumerate() {
        let gray = gray_region(image, rect)?;
        if is_avatar_index_white_rect(&gray) {
            white_rect_count += 1;
        } else {
            not_white_rect_index = Some(index + 1);
        }
        gray_regions.push((rect, gray));
    }

    if white_rect_count == rects.len() - 1 {
        let active_index = not_white_rect_index;
        return Ok(CombatActiveAvatarDetectionResult {
            active_index,
            method: CombatActiveAvatarDetectionMethod::WhiteRectMajority,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios: gray_regions
                .iter()
                .map(|(_, gray)| avatar_index_white_edge_ratio(gray))
                .collect(),
            difference_votes: Vec::new(),
            message: format!(
                "detected active avatar by white index blocks: {:?}",
                active_index
            ),
        });
    }

    let edge_white_ratios: Vec<_> = gray_regions
        .iter()
        .map(|(_, gray)| avatar_index_white_edge_ratio(gray))
        .collect();
    let edge_white_count = edge_white_ratios
        .iter()
        .filter(|ratio| **ratio > 0.5)
        .count();
    let edge_not_white_index = edge_white_ratios
        .iter()
        .enumerate()
        .rev()
        .find(|(_, ratio)| **ratio <= 0.5)
        .map(|(index, _)| index + 1);
    if edge_white_count == rects.len() - 1 {
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: edge_not_white_index,
            method: CombatActiveAvatarDetectionMethod::EdgeWhiteRatio,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios,
            difference_votes: Vec::new(),
            message: format!(
                "detected active avatar by white edge ratio: {:?}",
                edge_not_white_index
            ),
        });
    }
    if edge_white_count == rects.len() {
        let black_indexes: Vec<_> = gray_regions
            .iter()
            .enumerate()
            .filter(|(_, (_, gray))| count_gray_range(gray, 50, 50) > 0)
            .map(|(index, _)| index + 1)
            .collect();
        let not_black_index = (1..=rects.len()).find(|index| !black_indexes.contains(index));
        if let Some(active_index) = not_black_index {
            return Ok(CombatActiveAvatarDetectionResult {
                active_index: Some(active_index),
                method: CombatActiveAvatarDetectionMethod::EdgeWhiteRatio,
                rects: rects.to_vec(),
                white_rect_count,
                not_white_rect_index,
                edge_white_ratios,
                difference_votes: Vec::new(),
                message: format!(
                    "all index edges are white; active avatar inferred from missing black digit: {active_index}"
                ),
            });
        }
    }

    if gray_regions.len() == 4
        && gray_regions
            .iter()
            .map(|(_, gray)| (gray.width, gray.height))
            .all(|size| size == (gray_regions[0].1.width, gray_regions[0].1.height))
    {
        let (active_index, votes) = most_different_avatar_index(&gray_regions);
        if let Some(active_index) = active_index {
            return Ok(CombatActiveAvatarDetectionResult {
                active_index: Some(active_index),
                method: CombatActiveAvatarDetectionMethod::ImageDifferenceVote,
                rects: rects.to_vec(),
                white_rect_count,
                not_white_rect_index,
                edge_white_ratios,
                difference_votes: votes,
                message: format!("detected active avatar by image-difference vote: {active_index}"),
            });
        }
        return Ok(CombatActiveAvatarDetectionResult {
            active_index: None,
            method: CombatActiveAvatarDetectionMethod::Unresolved,
            rects: rects.to_vec(),
            white_rect_count,
            not_white_rect_index,
            edge_white_ratios,
            difference_votes: votes,
            message: "active avatar index could not be resolved by color or difference voting"
                .to_string(),
        });
    }

    Ok(CombatActiveAvatarDetectionResult {
        active_index: None,
        method: CombatActiveAvatarDetectionMethod::Unresolved,
        rects: rects.to_vec(),
        white_rect_count,
        not_white_rect_index,
        edge_white_ratios,
        difference_votes: Vec::new(),
        message: "active avatar index could not be resolved by color heuristics".to_string(),
    })
}

pub fn detect_active_combat_avatar_index_by_arrow_template(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    rects: &[Rect],
    arrow_roi: Rect,
    mut base_result: CombatActiveAvatarDetectionResult,
) -> Result<CombatActiveAvatarDetectionResult> {
    if rects.len() == 1 {
        base_result.active_index = Some(1);
        base_result.method = CombatActiveAvatarDetectionMethod::SingleAvatar;
        base_result.message = "single controllable avatar is active by definition".to_string();
        return Ok(base_result);
    }
    if rects.is_empty() {
        return Ok(base_result);
    }

    let current_avatar =
        find_current_avatar_arrow_template_rect(working_directory, image, arrow_roi)?;

    let Some(current_avatar) = current_avatar else {
        base_result.message =
            "active avatar index could not be resolved by color heuristics or arrow template"
                .to_string();
        return Ok(base_result);
    };

    if let Some((index, _)) = rects
        .iter()
        .copied()
        .enumerate()
        .find(|(_, rect)| rects_intersect_vertically(current_avatar, *rect))
    {
        base_result.active_index = Some(index + 1);
        base_result.method = CombatActiveAvatarDetectionMethod::ArrowTemplate;
        base_result.message = format!(
            "detected active avatar by current-avatar arrow template: {}",
            index + 1
        );
        return Ok(base_result);
    }

    base_result.message = format!(
        "current-avatar arrow template was found at {:?}, but it did not intersect any avatar index rectangle",
        current_avatar
    );
    Ok(base_result)
}
