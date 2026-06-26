use super::vision_pixels::{count_exact_white_components, detect_side_burst_circle};
use super::*;
use bgi_vision::BgrImage;
use std::path::Path;

pub fn detect_combat_skill_readiness(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    index: usize,
    is_burst: bool,
) -> Result<CombatSkillReadinessDetection> {
    if index == 0 {
        return Err(TaskError::VisionPlan(
            "combat skill readiness index is 1-based".to_string(),
        ));
    }
    let kind = if is_burst {
        CombatSkillReadinessKind::ElementalBurst
    } else {
        CombatSkillReadinessKind::ElementalSkill
    };
    let active_detection =
        detect_active_combat_avatar_index_from_default_rects_with_arrow(working_directory, image)?;
    let Some(active_index) = active_detection.active_index else {
        return Ok(CombatSkillReadinessDetection {
            kind,
            requested_index: index,
            active_index: None,
            status: CombatSkillReadinessStatus::ActiveAvatarUnresolved,
            ready: None,
            active_detection,
            cooldown_rect: None,
            white_component_count: 0,
            legacy_connected_component_labels: 0,
            side_burst_rect: None,
            side_burst_circle: None,
            message: "active avatar index is unresolved; skill readiness cannot be checked"
                .to_string(),
        });
    };
    if active_index != index {
        if is_burst {
            let side_burst_rect = combat_avatar_side_burst_rect_for_index(image.size, index)?;
            let side_burst_circle = detect_side_burst_circle(image, side_burst_rect)?;
            let ready = side_burst_circle.detected;
            return Ok(CombatSkillReadinessDetection {
                kind,
                requested_index: index,
                active_index: Some(active_index),
                status: if ready {
                    CombatSkillReadinessStatus::Ready
                } else {
                    CombatSkillReadinessStatus::CooldownOrUnavailable
                },
                ready: Some(ready),
                active_detection,
                cooldown_rect: None,
                white_component_count: 0,
                legacy_connected_component_labels: 0,
                side_burst_rect: Some(side_burst_rect),
                side_burst_circle: Some(side_burst_circle),
                message: if ready {
                    "inactive avatar burst readiness was resolved by the legacy side-icon circle probe".to_string()
                } else {
                    "inactive avatar burst side-icon circle was not detected".to_string()
                },
            });
        }
        return Ok(CombatSkillReadinessDetection {
            kind,
            requested_index: index,
            active_index: Some(active_index),
            status: CombatSkillReadinessStatus::UnsupportedForInactiveAvatar,
            ready: None,
            active_detection,
            cooldown_rect: None,
            white_component_count: 0,
            legacy_connected_component_labels: 0,
            side_burst_rect: None,
            side_burst_circle: None,
            message: "inactive avatar elemental-skill readiness is not exposed by the legacy UI"
                .to_string(),
        });
    }

    let cooldown_rect = active_combat_skill_cooldown_rect(image.size, is_burst)?;
    let white_component_count = count_exact_white_components(image, cooldown_rect)?;
    let legacy_connected_component_labels = white_component_count + 1;
    let ready = legacy_connected_component_labels <= 2;
    Ok(CombatSkillReadinessDetection {
        kind,
        requested_index: index,
        active_index: Some(active_index),
        status: if ready {
            CombatSkillReadinessStatus::Ready
        } else {
            CombatSkillReadinessStatus::CooldownOrUnavailable
        },
        ready: Some(ready),
        active_detection,
        cooldown_rect: Some(cooldown_rect),
        white_component_count,
        legacy_connected_component_labels,
        side_burst_rect: None,
        side_burst_circle: None,
        message: if ready {
            "active avatar skill has no legacy cooldown digit components".to_string()
        } else {
            "active avatar skill has legacy cooldown digit/lock components".to_string()
        },
    })
}
