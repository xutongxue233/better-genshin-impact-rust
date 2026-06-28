use super::*;
use crate::task_asset_root;
use bgi_core::{AssetResolver, ScreenSize};
use bgi_vision::{
    BgrImage, PureRustVisionBackend, RecognitionObject, Rect, Size, TemplateMatchMode,
    VisionBackend,
};
use std::path::Path;

pub(super) fn find_combat_avatar_index_template_rect(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    asset_name: &str,
    roi: Rect,
    object_name: &str,
) -> Result<Option<Rect>> {
    let resolver = auto_fight_asset_resolver(working_directory);
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_CURRENT_AVATAR_FEATURE,
            asset_name,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(None);
    }
    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some(object_name.to_string());
    object.template.threshold = 0.95;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.max_match_count = 1;
    let region = task_vision_result(backend.find(&image.pixels, image.size, &object))?;
    Ok(region.is_exist().then_some(region.rect))
}

pub(super) fn find_auto_fight_template_matches(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    asset_name: &str,
    roi: Rect,
    object_name: &str,
    max_match_count: i32,
) -> Result<Vec<Rect>> {
    let resolver = auto_fight_asset_resolver(working_directory);
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_FEATURE,
            asset_name,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(Vec::new());
    }
    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some(object_name.to_string());
    object.template.threshold = 0.8;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.max_match_count = max_match_count;
    let matches = task_vision_result(backend.find_multi(&image.pixels, image.size, &object))?;
    Ok(matches.into_iter().map(|region| region.rect).collect())
}

pub(super) fn validate_avatar_side_classification(
    index: usize,
    classification: &CombatAvatarSideClassification,
) -> Result<()> {
    let minimum_confidence = if classification.class_name.starts_with("Qin")
        || classification.class_name.contains("Costume")
    {
        0.51
    } else {
        0.7
    };
    if classification.confidence < minimum_confidence {
        return Err(TaskError::CombatStrategy(format!(
            "无法识别第{index}位角色，置信度{:.2}，结果：{}",
            classification.confidence, classification.class_name
        )));
    }
    Ok(())
}

pub(super) fn split_avatar_side_class_name(class_name: &str) -> (String, Option<String>) {
    if let Some(index) = class_name.find("Costume") {
        let name_en = class_name[..index].to_string();
        let costume_name = class_name[index + "Costume".len()..].to_string();
        return (name_en, (!costume_name.is_empty()).then_some(costume_name));
    }
    (class_name.to_string(), None)
}

pub(super) fn display_avatar_costume_name(costume_name: &str) -> String {
    match costume_name {
        "Flamme" => "殷红终夜",
        "Bamboo" => "雨化竹身",
        "Dai" => "冷花幽露",
        "Yu" => "玄玉瑶芳",
        "Dancer" => "帆影游风",
        "Witch" => "琪花星烛",
        "Wic" => "和谐",
        "Studentin" => "叶隐芳名",
        "Fruhling" => "花时来信",
        "Highness" => "极夜真梦",
        "Feather" => "霓裾翩跹",
        "Floral" => "纱中幽兰",
        "Summertime" => "闪耀协奏",
        "Sea" => "海风之梦",
        _ => costume_name,
    }
    .to_string()
}

pub(super) fn find_current_avatar_arrow_template_rect(
    working_directory: impl AsRef<Path>,
    image: &BgrImage,
    roi: Rect,
) -> Result<Option<Rect>> {
    let resolver = auto_fight_asset_resolver(working_directory);
    find_current_avatar_arrow_template_rect_with_resolver(&resolver, image, roi)
}

pub(super) fn find_current_avatar_arrow_template_rect_with_resolver(
    resolver: &AssetResolver,
    image: &BgrImage,
    roi: Rect,
) -> Result<Option<Rect>> {
    let asset_path = resolver
        .resolve_feature_asset(
            AUTO_FIGHT_CURRENT_AVATAR_FEATURE,
            AUTO_FIGHT_CURRENT_AVATAR_THRESHOLD_ASSET,
            screen_size_from_image(image),
        )
        .map_err(|error| TaskError::VisionPlan(error.to_string()))?;
    let roi = task_vision_result(roi.clamp_to(image.size))?;
    if roi.is_empty() {
        return Ok(None);
    }

    let backend = PureRustVisionBackend::new();
    let mut object = RecognitionObject::template_match_in(&asset_path, roi);
    object.name = Some("CurrentAvatarThreshold".to_string());
    object.template.threshold = 0.8;
    object.template.mode = TemplateMatchMode::CCoeffNormed;
    object.template.use_binary_match = true;
    object.template.binary_threshold = 200;
    object.template.max_match_count = 1;
    let region = task_vision_result(backend.find(&image.pixels, image.size, &object))?;
    Ok(region.is_exist().then_some(region.rect))
}

pub(super) fn first_known_avatar_index_rect(
    rects_by_index: &[Option<Rect>],
) -> Option<(usize, Rect)> {
    rects_by_index
        .iter()
        .copied()
        .enumerate()
        .find_map(|(index, rect)| rect.map(|rect| (index + 1, rect)))
}

pub(super) fn infer_avatar_index_rects_from_known_and_arrow(
    image_size: Size,
    rects_by_index: &[Option<Rect>],
    known_index: usize,
    known_rect: Rect,
    current_avatar: Rect,
) -> Result<Vec<Option<Rect>>> {
    let mut inferred = rects_by_index.to_vec();
    for (index, inferred_rect) in inferred.iter_mut().enumerate() {
        if inferred_rect.is_some() {
            continue;
        }
        let rect =
            index_rect_from_known_index_rect(image_size, known_index, known_rect, index + 1)?;
        if rects_intersect_vertically(current_avatar, rect) {
            *inferred_rect = Some(rect);
        }
    }
    Ok(inferred)
}

fn index_rect_from_known_index_rect(
    image_size: Size,
    known_index: usize,
    known_rect: Rect,
    target_index: usize,
) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let distance = (AUTO_FIGHT_AVATAR_INDEX_DISTANCE_Y_1080P as f64 * scale).round() as i32;
    task_vision_result(Rect::new(
        known_rect.x,
        known_rect.y + (target_index as i32 - known_index as i32) * distance,
        known_rect.width,
        known_rect.height,
    ))
}

pub(super) fn index_rect_from_current_avatar_arrow(
    image_size: Size,
    current_avatar: Rect,
) -> Result<Rect> {
    let scale = image_size.height as f64 / 1080.0;
    let (offset_x, offset_y, width, height) = AUTO_FIGHT_CURRENT_AVATAR_FLAG_TO_INDEX_RECT_1080P;
    task_vision_result(Rect::new(
        current_avatar.x + (offset_x as f64 * scale).round() as i32,
        current_avatar.y + (offset_y as f64 * scale).round() as i32,
        (width as f64 * scale).round() as i32,
        (height as f64 * scale).round() as i32,
    ))
}

pub(super) fn avatar_index_rects_are_contiguous(rects_by_index: &[Option<Rect>]) -> bool {
    let mut seen_gap = false;
    for rect in rects_by_index {
        if rect.is_some() {
            if seen_gap {
                return false;
            }
        } else {
            seen_gap = true;
        }
    }
    true
}

fn screen_size_from_image(image: &BgrImage) -> ScreenSize {
    ScreenSize {
        width: image.size.width,
        height: image.size.height,
    }
}

fn auto_fight_asset_resolver(working_directory: impl AsRef<Path>) -> AssetResolver {
    AssetResolver::new(working_directory.as_ref()).with_fallback_root(task_asset_root())
}

pub(super) fn rects_intersect_vertically(left: Rect, right: Rect) -> bool {
    left.y < right.bottom() && right.y < left.bottom()
}
