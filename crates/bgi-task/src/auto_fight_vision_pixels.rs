use super::*;
use bgi_vision::{BgrImage, Rect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GrayRegion {
    pub(super) width: usize,
    pub(super) height: usize,
    pixels: Vec<u8>,
}

pub(super) fn gray_region(image: &BgrImage, rect: Rect) -> Result<GrayRegion> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "avatar index rectangle is outside capture: {rect:?}"
        )));
    }
    let width = rect.width as usize;
    let height = rect.height as usize;
    let mut pixels = Vec::with_capacity(width * height);
    for y in rect.y as u32..rect.bottom() as u32 {
        for x in rect.x as u32..rect.right() as u32 {
            let pixel = image.bgr_pixel_at(x, y).ok_or_else(|| {
                TaskError::VisionPlan(format!("avatar index pixel ({x}, {y}) is outside capture"))
            })?;
            pixels.push(gray_from_rgb(pixel.r, pixel.g, pixel.b));
        }
    }
    Ok(GrayRegion {
        width,
        height,
        pixels,
    })
}

fn gray_from_rgb(r: u8, g: u8, b: u8) -> u8 {
    (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8
}

pub(super) fn is_avatar_index_white_rect(gray: &GrayRegion) -> bool {
    let white_count = count_gray_range(gray, 251, 255);
    let black_text_count = count_gray_range(gray, 50, 54);
    (white_count + black_text_count) as f64 / gray.pixels.len() as f64 > 0.35
}

pub(super) fn count_gray_range(gray: &GrayRegion, lower: u8, upper: u8) -> usize {
    gray.pixels
        .iter()
        .filter(|pixel| (lower..=upper).contains(pixel))
        .count()
}

pub(super) fn avatar_index_white_edge_ratio(gray: &GrayRegion) -> f64 {
    if gray.width == 0 || gray.height == 0 {
        return 0.0;
    }
    let mut white_count = 0usize;
    let total_count = 2 * (gray.width + gray.height).saturating_sub(4);
    for x in 0..gray.width {
        if gray_pixel(gray, x, 0) == Some(255) {
            white_count += 1;
        }
        if gray.height > 1 && gray_pixel(gray, x, gray.height - 1) == Some(255) {
            white_count += 1;
        }
    }
    for y in 1..gray.height.saturating_sub(1) {
        if gray_pixel(gray, 0, y) == Some(255) {
            white_count += 1;
        }
        if gray.width > 1 && gray_pixel(gray, gray.width - 1, y) == Some(255) {
            white_count += 1;
        }
    }
    if total_count == 0 {
        0.0
    } else {
        white_count as f64 / total_count as f64
    }
}

fn gray_pixel(gray: &GrayRegion, x: usize, y: usize) -> Option<u8> {
    (x < gray.width && y < gray.height).then(|| gray.pixels[y * gray.width + x])
}

pub(super) fn most_different_avatar_index(
    gray_regions: &[(Rect, GrayRegion)],
) -> (Option<usize>, Vec<usize>) {
    let mut votes = vec![0usize; gray_regions.len()];
    for i in 0..gray_regions.len() {
        let mut max_diff_index = None;
        let mut max_difference = 0usize;
        for j in 0..gray_regions.len() {
            if i == j {
                continue;
            }
            let difference = gray_difference_count(&gray_regions[i].1, &gray_regions[j].1);
            if difference > max_difference {
                max_difference = difference;
                max_diff_index = Some(j);
            }
        }
        if let Some(index) = max_diff_index {
            votes[index] += 1;
        }
    }
    let active = votes
        .iter()
        .enumerate()
        .find(|(_, votes)| **votes >= 3)
        .map(|(index, _)| index + 1);
    (active, votes)
}

fn gray_difference_count(left: &GrayRegion, right: &GrayRegion) -> usize {
    if left.width != right.width || left.height != right.height {
        return 0;
    }
    left.pixels
        .iter()
        .zip(&right.pixels)
        .filter(|(left, right)| left != right)
        .count()
}

pub(super) fn count_exact_white_components(image: &BgrImage, rect: Rect) -> Result<usize> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "skill cooldown rectangle is outside capture: {rect:?}"
        )));
    }
    let width = rect.width as usize;
    let height = rect.height as usize;
    let mut mask = vec![false; width * height];
    for local_y in 0..height {
        for local_x in 0..width {
            let x = rect.x as u32 + local_x as u32;
            let y = rect.y as u32 + local_y as u32;
            let pixel = image.rgb_pixel_at(x, y).ok_or_else(|| {
                TaskError::VisionPlan(format!(
                    "skill cooldown pixel ({x}, {y}) is outside capture"
                ))
            })?;
            mask[local_y * width + local_x] = pixel.r == 255 && pixel.g == 255 && pixel.b == 255;
        }
    }
    Ok(count_binary_components_8_connected(&mask, width, height))
}

fn count_binary_components_8_connected(mask: &[bool], width: usize, height: usize) -> usize {
    if width == 0 || height == 0 {
        return 0;
    }
    let mut visited = vec![false; mask.len()];
    let mut components = 0usize;
    let mut stack = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if !mask[index] || visited[index] {
                continue;
            }
            components += 1;
            visited[index] = true;
            stack.push((x, y));
            while let Some((cx, cy)) = stack.pop() {
                let min_x = cx.saturating_sub(1);
                let max_x = (cx + 1).min(width - 1);
                let min_y = cy.saturating_sub(1);
                let max_y = (cy + 1).min(height - 1);
                for ny in min_y..=max_y {
                    for nx in min_x..=max_x {
                        let neighbor = ny * width + nx;
                        if mask[neighbor] && !visited[neighbor] {
                            visited[neighbor] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }
    }
    components
}

pub fn detect_side_burst_circle(
    image: &BgrImage,
    rect: Rect,
) -> Result<CombatSideBurstCircleDetection> {
    let rect = task_vision_result(rect.clamp_to(image.size))?;
    if rect.width <= 0 || rect.height <= 0 {
        return Err(TaskError::VisionPlan(format!(
            "side burst rectangle is outside capture: {rect:?}"
        )));
    }
    let gray = gray_region(image, rect)?;
    let edge_mask = side_burst_edge_mask(&gray);
    let edge_pixel_count = edge_mask.iter().filter(|edge| **edge).count();
    let scale = image.size.height as f64 / 1080.0;
    let min_radius =
        ((AUTO_FIGHT_SIDE_BURST_MIN_RADIUS_1080P as f64 * scale).round() as i32).max(1);
    let max_radius =
        ((AUTO_FIGHT_SIDE_BURST_MAX_RADIUS_1080P as f64 * scale).round() as i32).max(min_radius);
    let mut best_center = None;
    let mut best_radius = None;
    let mut best_votes = 0usize;
    for radius in min_radius..=max_radius {
        if radius * 2 > rect.width.max(rect.height) + 16 {
            continue;
        }
        for cy in radius..(rect.height - radius).max(radius + 1) {
            for cx in radius..(rect.width - radius).max(radius + 1) {
                let votes = circle_edge_votes(
                    &edge_mask,
                    gray.width,
                    gray.height,
                    cx,
                    cy,
                    radius,
                    AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES,
                );
                if votes > best_votes {
                    best_votes = votes;
                    best_center = Some((rect.x + cx, rect.y + cy));
                    best_radius = Some(radius);
                }
            }
        }
    }
    Ok(CombatSideBurstCircleDetection {
        rect,
        detected: best_votes >= AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES,
        edge_pixel_count,
        best_center,
        best_radius,
        best_votes,
        required_votes: AUTO_FIGHT_SIDE_BURST_REQUIRED_CIRCLE_VOTES,
        sampled_points: AUTO_FIGHT_SIDE_BURST_CIRCLE_SAMPLES,
    })
}

fn side_burst_edge_mask(gray: &GrayRegion) -> Vec<bool> {
    let mut edges = vec![false; gray.width * gray.height];
    if gray.width < 3 || gray.height < 3 {
        return edges;
    }
    let mean = gray.pixels.iter().map(|pixel| *pixel as u64).sum::<u64>() as f64
        / gray.pixels.len() as f64;
    let bright_threshold = (mean + 35.0).clamp(150.0, 235.0).round() as u8;
    for y in 1..gray.height - 1 {
        for x in 1..gray.width - 1 {
            let center = gray_pixel(gray, x, y).unwrap_or_default();
            let left = gray_pixel(gray, x - 1, y).unwrap_or_default() as i32;
            let right = gray_pixel(gray, x + 1, y).unwrap_or_default() as i32;
            let up = gray_pixel(gray, x, y - 1).unwrap_or_default() as i32;
            let down = gray_pixel(gray, x, y + 1).unwrap_or_default() as i32;
            let gradient = (right - left).abs() + (down - up).abs();
            edges[y * gray.width + x] = gradient >= 45 || center >= bright_threshold;
        }
    }
    edges
}

fn circle_edge_votes(
    edge_mask: &[bool],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
    samples: usize,
) -> usize {
    let mut votes = 0usize;
    let mut last_point = None;
    for sample in 0..samples {
        let angle = sample as f64 * std::f64::consts::TAU / samples as f64;
        let x = center_x + (radius as f64 * angle.cos()).round() as i32;
        let y = center_y + (radius as f64 * angle.sin()).round() as i32;
        let point = (x, y);
        if last_point == Some(point) {
            continue;
        }
        last_point = Some(point);
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            votes += edge_mask[y as usize * width + x as usize] as usize;
        }
    }
    votes
}
