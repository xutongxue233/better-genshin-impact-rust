use super::*;
use bgi_vision::{BgrImage, RgbPixel};

pub fn detect_auto_fight_finished_from_image(
    image: &BgrImage,
) -> Result<AutoFightFinishDetectionResult> {
    let progress_pixel = image
        .rgb_pixel_at(
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL.0,
            AUTO_FIGHT_FINISH_PROGRESS_PIXEL.1,
        )
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "auto-fight finish progress pixel is outside capture: {:?}",
                AUTO_FIGHT_FINISH_PROGRESS_PIXEL
            ))
        })?;
    let white_tile_pixel = image
        .rgb_pixel_at(
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL.0,
            AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL.1,
        )
        .ok_or_else(|| {
            TaskError::VisionPlan(format!(
                "auto-fight finish white tile pixel is outside capture: {:?}",
                AUTO_FIGHT_FINISH_WHITE_TILE_PIXEL
            ))
        })?;
    Ok(AutoFightFinishDetectionResult {
        finished: is_auto_fight_finish_white(white_tile_pixel)
            && is_auto_fight_finish_yellow(progress_pixel),
        progress_pixel,
        white_tile_pixel,
    })
}

pub fn is_auto_fight_finish_yellow(pixel: RgbPixel) -> bool {
    (200..=255).contains(&pixel.r) && (200..=255).contains(&pixel.g) && (0..=100).contains(&pixel.b)
}

pub fn is_auto_fight_finish_white(pixel: RgbPixel) -> bool {
    (240..=255).contains(&pixel.r)
        && (240..=255).contains(&pixel.g)
        && (240..=255).contains(&pixel.b)
}
