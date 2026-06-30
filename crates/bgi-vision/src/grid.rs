use serde::{Deserialize, Serialize};

use crate::{
    convert_bgr_image, crop_bgr_image, BgrImage, BgrPixel, ColorConversion, Rect, Result, Size,
    VisionError,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridCell {
    pub rect: Rect,
    pub row: i32,
    pub col: i32,
    pub is_phantom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VisibleGridEnumerationSpec {
    pub roi: Rect,
    pub columns: u8,
    pub min_width_per_column_ratio: f64,
    pub shape_ratio_target: f64,
    pub shape_ratio_tolerance: f64,
    pub top_right_exclusion_x_ratio: f64,
    pub top_right_exclusion_y_ratio: f64,
    pub canny_low_threshold: f64,
    pub canny_high_threshold: f64,
    pub close_kernel_width: i32,
    pub close_kernel_height: i32,
    pub fill_missing_threshold: i32,
    pub phantom_bottom_color: BgrPixel,
    pub phantom_tolerance: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridIconCropSpec {
    pub normalized_size: Size,
    pub icon_crop: Rect,
    pub bottom_crop: Rect,
}

impl GridIconCropSpec {
    pub fn legacy_inventory() -> Self {
        Self {
            normalized_size: Size::new(125, 153),
            icon_crop: Rect {
                x: 0,
                y: 0,
                width: 125,
                height: 125,
            },
            bottom_crop: Rect {
                x: 0,
                y: 126,
                width: 125,
                height: 27,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridIconCrops {
    pub normalized: BgrImage,
    pub icon: BgrImage,
    pub bottom: BgrImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridItemTextCropSpec {
    pub crop_top_numerator: u32,
    pub crop_bottom_numerator: u32,
    pub crop_left_numerator: u32,
    pub crop_right_numerator: u32,
    pub crop_height_denominator: u32,
    pub crop_width_denominator: u32,
    pub resize_scale: u32,
}

impl GridItemTextCropSpec {
    pub fn legacy_inventory_count() -> Self {
        Self {
            crop_top_numerator: 128,
            crop_bottom_numerator: 150,
            crop_left_numerator: 5,
            crop_right_numerator: 120,
            crop_height_denominator: 153,
            crop_width_denominator: 125,
            resize_scale: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GridScrollDecision {
    pub is_scrolling: bool,
    pub shift_x: i32,
    pub shift_y: i32,
    pub response: f64,
}

pub fn cluster_grid_cells(rects: &[Rect], threshold: i32) -> Vec<GridCell> {
    let mut cells = assign_grid_cell_rows_and_columns(rects, threshold);
    fill_missing_grid_cells(&mut cells);
    cells
}

pub fn enumerate_visible_grid_cells(
    image: &BgrImage,
    spec: VisibleGridEnumerationSpec,
) -> Result<Vec<GridCell>> {
    validate_visible_grid_enumeration_spec(image.size, spec)?;
    let roi = spec.roi.clamp_to(image.size)?;
    if roi.width <= 0 || roi.height <= 0 {
        return Err(VisionError::InvalidRect);
    }
    let roi_image = crop_bgr_image(image, roi)?;
    let candidates = visible_grid_candidate_rects(&roi_image, spec)?
        .into_iter()
        .map(|rect| Rect {
            x: rect.x + roi.x,
            y: rect.y + roi.y,
            width: rect.width,
            height: rect.height,
        })
        .collect::<Vec<_>>();
    let mut cells = post_process_grid_cells(
        image,
        &candidates,
        spec.fill_missing_threshold,
        spec.phantom_bottom_color,
        spec.phantom_tolerance,
    )?;
    cells.retain(|cell| cell.col >= 0 && cell.col < spec.columns as i32);
    cells.sort_by_key(|cell| (cell.row, cell.col, cell.rect.y, cell.rect.x));
    Ok(cells)
}

pub fn assign_grid_cell_rows_and_columns(rects: &[Rect], threshold: i32) -> Vec<GridCell> {
    if rects.is_empty() {
        return Vec::new();
    }

    let mut cells = rects
        .iter()
        .copied()
        .map(|rect| GridCell {
            rect,
            row: 0,
            col: 0,
            is_phantom: false,
        })
        .collect::<Vec<_>>();

    let avg_width =
        (rects.iter().map(|rect| rect.width).sum::<i32>() as f64 / rects.len() as f64) as i32;
    let avg_height =
        (rects.iter().map(|rect| rect.height).sum::<i32>() as f64 / rects.len() as f64) as i32;

    let mut by_x = (0..cells.len()).collect::<Vec<_>>();
    by_x.sort_by_key(|index| cells[*index].rect.x);
    let mut col = 0;
    let mut last_x = None;
    for index in by_x {
        if let Some(last_x) = last_x {
            let delta = cells[index].rect.x - last_x;
            let denominator = avg_width + threshold;
            if delta > threshold && denominator > 0 {
                col += round_ties_even_i32(delta as f64 / denominator as f64);
            }
        }
        cells[index].col = col;
        last_x = Some(cells[index].rect.x);
    }

    let mut by_y = (0..cells.len()).collect::<Vec<_>>();
    by_y.sort_by_key(|index| cells[*index].rect.y);
    let mut row = 0;
    let mut last_y = None;
    for index in by_y {
        if let Some(last_y) = last_y {
            let delta = cells[index].rect.y - last_y;
            let denominator = avg_height + threshold;
            if delta > threshold && denominator > 0 {
                row += round_ties_even_i32(delta as f64 / denominator as f64);
            }
        }
        cells[index].row = row;
        last_y = Some(cells[index].rect.y);
    }

    cells
}

pub fn fill_missing_grid_cells(cells: &mut Vec<GridCell>) {
    if cells.is_empty() {
        return;
    }

    let avg_width =
        cells.iter().map(|cell| cell.rect.width).sum::<i32>() as f64 / cells.len() as f64;
    let avg_height =
        cells.iter().map(|cell| cell.rect.height).sum::<i32>() as f64 / cells.len() as f64;
    let avg_col_spacing = average_col_spacing(cells);
    let avg_row_spacing = average_row_spacing(cells);
    let avg_left = cells
        .iter()
        .map(|cell| cell.rect.x as f64 - (avg_width + avg_col_spacing) * cell.col as f64)
        .sum::<f64>()
        / cells.len() as f64;
    let avg_top = cells
        .iter()
        .map(|cell| cell.rect.y as f64 - (avg_height + avg_row_spacing) * cell.row as f64)
        .sum::<f64>()
        / cells.len() as f64;
    let max_col = cells.iter().map(|cell| cell.col).max().unwrap_or(0);
    let max_row = cells.iter().map(|cell| cell.row).max().unwrap_or(0);

    for col in 0..=max_col {
        for row in 0..=max_row {
            if cells.iter().any(|cell| cell.col == col && cell.row == row) {
                continue;
            }
            cells.push(GridCell {
                rect: Rect {
                    x: round_away_from_zero_i32(
                        avg_left + (avg_width + avg_col_spacing) * col as f64,
                    ),
                    y: round_away_from_zero_i32(
                        avg_top + (avg_height + avg_row_spacing) * row as f64,
                    ),
                    width: round_away_from_zero_i32(avg_width),
                    height: round_away_from_zero_i32(avg_height),
                },
                row,
                col,
                is_phantom: true,
            });
        }
    }
}

pub fn post_process_grid_cells(
    image: &BgrImage,
    rects: &[Rect],
    threshold: i32,
    phantom_bottom_color: BgrPixel,
    phantom_tolerance: u8,
) -> Result<Vec<GridCell>> {
    let cells = cluster_grid_cells(rects, threshold);
    let mut result = Vec::with_capacity(cells.len());
    for cell in cells {
        if !cell.is_phantom {
            result.push(cell);
            continue;
        }
        if !rect_within_image(cell.rect, image.size) {
            continue;
        }
        let cell_image = crop_bgr_image(image, cell.rect)?;
        let bottom = crop_grid_icon(&cell_image, GridIconCropSpec::legacy_inventory())?.bottom;
        if validate_phantom_bottom_color(&bottom, phantom_bottom_color, phantom_tolerance)? {
            result.push(cell);
        }
    }
    Ok(result)
}

pub fn validate_phantom_bottom_color(
    image: &BgrImage,
    target: BgrPixel,
    tolerance: u8,
) -> Result<bool> {
    if image.size.width == 0 || image.size.height == 0 {
        return Err(VisionError::InvalidRect);
    }
    let pixels = image.size.width as f64 * image.size.height as f64;
    let mut b = 0.0;
    let mut g = 0.0;
    let mut r = 0.0;
    for chunk in image.pixels.chunks_exact(3) {
        b += chunk[0] as f64;
        g += chunk[1] as f64;
        r += chunk[2] as f64;
    }
    let diff = (b / pixels - target.b as f64).abs()
        + (g / pixels - target.g as f64).abs()
        + (r / pixels - target.r as f64).abs();
    Ok(diff <= tolerance as f64 * 3.0)
}

pub fn crop_grid_icon(image: &BgrImage, spec: GridIconCropSpec) -> Result<GridIconCrops> {
    let normalized = resize_bgr_linear(image, spec.normalized_size)?;
    let icon = crop_bgr_image(&normalized, spec.icon_crop)?;
    let bottom = crop_bgr_image(&normalized, spec.bottom_crop)?;
    Ok(GridIconCrops {
        normalized,
        icon,
        bottom,
    })
}

pub fn crop_grid_item_count_text(image: &BgrImage, spec: GridItemTextCropSpec) -> Result<BgrImage> {
    if spec.crop_height_denominator == 0
        || spec.crop_width_denominator == 0
        || spec.resize_scale == 0
    {
        return Err(VisionError::InvalidRect);
    }
    let left = image.size.width * spec.crop_left_numerator / spec.crop_width_denominator;
    let right = image.size.width * spec.crop_right_numerator / spec.crop_width_denominator;
    let top = image.size.height * spec.crop_top_numerator / spec.crop_height_denominator;
    let bottom = image.size.height * spec.crop_bottom_numerator / spec.crop_height_denominator;
    let crop = crop_bgr_image(
        image,
        Rect::new(
            left as i32,
            top as i32,
            right.saturating_sub(left) as i32,
            bottom.saturating_sub(top) as i32,
        )?,
    )?;
    resize_bgr_linear(
        &crop,
        Size::new(
            crop.size.width * spec.resize_scale,
            crop.size.height * spec.resize_scale,
        ),
    )
}

pub fn detect_grid_scroll_shift(
    previous: &BgrImage,
    next: &BgrImage,
    lower_threshold: f64,
    upper_threshold: f64,
    max_shift: i32,
) -> Result<GridScrollDecision> {
    if previous.size != next.size {
        return Err(VisionError::ImageSizeMismatch {
            expected: previous.size,
            actual: next.size,
        });
    }
    let previous_gray =
        convert_bgr_image(&previous.pixels, previous.size, ColorConversion::BgrToGray)?;
    let next_gray = convert_bgr_image(&next.pixels, next.size, ColorConversion::BgrToGray)?;
    let total_pixels = previous.size.width as usize * previous.size.height as usize;
    let mut best = GridScrollDecision {
        is_scrolling: false,
        shift_x: 0,
        shift_y: 0,
        response: 0.0,
    };

    for shift_y in -max_shift..=max_shift {
        for shift_x in -max_shift..=max_shift {
            let (correlation, overlap) = normalized_shift_correlation(
                &previous_gray.pixels,
                &next_gray.pixels,
                previous.size,
                shift_x,
                shift_y,
            );
            let response = correlation.max(0.0) * overlap as f64 / total_pixels as f64;
            if response > best.response {
                best.response = response;
                best.shift_x = shift_x;
                best.shift_y = shift_y;
            }
        }
    }
    best.is_scrolling =
        classify_grid_scroll_response(best.response, lower_threshold, upper_threshold);
    Ok(best)
}

pub fn classify_grid_scroll_response(
    response: f64,
    lower_threshold: f64,
    upper_threshold: f64,
) -> bool {
    response > lower_threshold && response < upper_threshold
}

fn average_col_spacing(cells: &[GridCell]) -> f64 {
    let mut count = 0;
    let mut sum = 0;
    let max_row = cells.iter().map(|cell| cell.row).max().unwrap_or(0);
    for row in 0..=max_row {
        let max_col = cells
            .iter()
            .filter(|cell| cell.row == row)
            .map(|cell| cell.col)
            .max()
            .unwrap_or(0);
        for col in 0..max_col {
            let left = cells.iter().find(|cell| cell.row == row && cell.col == col);
            let right = cells
                .iter()
                .find(|cell| cell.row == row && cell.col == col + 1);
            if let (Some(left), Some(right)) = (left, right) {
                sum += right.rect.x - left.rect.x - left.rect.width;
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}

fn round_ties_even_i32(value: f64) -> i32 {
    value.round_ties_even() as i32
}

fn round_away_from_zero_i32(value: f64) -> i32 {
    if value >= 0.0 {
        (value + 0.5).floor() as i32
    } else {
        (value - 0.5).ceil() as i32
    }
}

fn resize_bgr_linear(image: &BgrImage, size: Size) -> Result<BgrImage> {
    if size.width == 0 || size.height == 0 {
        return Err(VisionError::InvalidRect);
    }
    let mut pixels = Vec::with_capacity(size.width as usize * size.height as usize * 3);
    let scale_x = image.size.width as f64 / size.width as f64;
    let scale_y = image.size.height as f64 / size.height as f64;

    for y in 0..size.height {
        let source_y = (y as f64 + 0.5) * scale_y - 0.5;
        let (y0, y1, wy) = linear_sample_axis(source_y, image.size.height);
        for x in 0..size.width {
            let source_x = (x as f64 + 0.5) * scale_x - 0.5;
            let (x0, x1, wx) = linear_sample_axis(source_x, image.size.width);
            for channel in 0..3 {
                let top_left = bgr_channel(image, x0, y0, channel);
                let top_right = bgr_channel(image, x1, y0, channel);
                let bottom_left = bgr_channel(image, x0, y1, channel);
                let bottom_right = bgr_channel(image, x1, y1, channel);
                let top = top_left * (1.0 - wx) + top_right * wx;
                let bottom = bottom_left * (1.0 - wx) + bottom_right * wx;
                let value = top * (1.0 - wy) + bottom * wy;
                pixels.push(value.round().clamp(0.0, 255.0) as u8);
            }
        }
    }

    BgrImage::new(size, pixels)
}

fn linear_sample_axis(source: f64, upper: u32) -> (u32, u32, f64) {
    if upper <= 1 || source <= 0.0 {
        return (0, 0, 0.0);
    }
    let max = upper - 1;
    if source >= max as f64 {
        return (max, max, 0.0);
    }
    let lower = source.floor() as u32;
    (lower, lower + 1, source - lower as f64)
}

fn bgr_channel(image: &BgrImage, x: u32, y: u32, channel: usize) -> f64 {
    let index = ((y * image.size.width + x) as usize) * 3 + channel;
    image.pixels[index] as f64
}

fn validate_visible_grid_enumeration_spec(
    size: Size,
    spec: VisibleGridEnumerationSpec,
) -> Result<()> {
    if size.width == 0
        || size.height == 0
        || spec.roi.width <= 0
        || spec.roi.height <= 0
        || spec.columns == 0
        || spec.min_width_per_column_ratio <= 0.0
        || spec.shape_ratio_target <= 0.0
        || spec.shape_ratio_tolerance < 0.0
        || spec.close_kernel_width <= 0
        || spec.close_kernel_height <= 0
        || spec.fill_missing_threshold < 0
    {
        return Err(VisionError::InvalidRect);
    }
    Ok(())
}

fn visible_grid_candidate_rects(
    image: &BgrImage,
    spec: VisibleGridEnumerationSpec,
) -> Result<Vec<Rect>> {
    let edge_mask = visible_grid_edge_mask(image, spec)?;
    let closed_mask = close_binary_mask(
        &edge_mask,
        image.size,
        spec.close_kernel_width as u32,
        spec.close_kernel_height as u32,
    );
    let components = connected_component_rects(&closed_mask, image.size);
    let column_width = image.size.width as f64 / spec.columns as f64;
    let min_width = column_width * spec.min_width_per_column_ratio;
    let mut rects = Vec::new();
    for rect in components {
        if rect.width as f64 <= min_width {
            continue;
        }
        if rect.height <= 0 {
            continue;
        }
        let ratio = rect.width as f64 / rect.height as f64;
        if (ratio - spec.shape_ratio_target).abs() > spec.shape_ratio_tolerance {
            continue;
        }
        if is_top_right_excluded(rect, image.size, spec) {
            continue;
        }
        rects.push(rect);
    }
    rects.sort_by_key(|rect| (rect.y, rect.x));
    Ok(rects)
}

fn visible_grid_edge_mask(image: &BgrImage, spec: VisibleGridEnumerationSpec) -> Result<Vec<bool>> {
    let gray = convert_bgr_image(&image.pixels, image.size, ColorConversion::BgrToGray)?;
    let threshold = ((spec.canny_low_threshold + spec.canny_high_threshold) / 2.0).max(1.0);
    let width = image.size.width as usize;
    let height = image.size.height as usize;
    let mut mask = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let center = gray.pixels[y * width + x] as f64;
            let left = if x > 0 {
                gray.pixels[y * width + x - 1] as f64
            } else {
                center
            };
            let right = if x + 1 < width {
                gray.pixels[y * width + x + 1] as f64
            } else {
                center
            };
            let top = if y > 0 {
                gray.pixels[(y - 1) * width + x] as f64
            } else {
                center
            };
            let bottom = if y + 1 < height {
                gray.pixels[(y + 1) * width + x] as f64
            } else {
                center
            };
            let magnitude = (right - left).abs() + (bottom - top).abs();
            mask[y * width + x] = magnitude >= threshold;
        }
    }
    Ok(mask)
}

fn close_binary_mask(
    mask: &[bool],
    size: Size,
    kernel_width: u32,
    kernel_height: u32,
) -> Vec<bool> {
    let dilated = dilate_binary_mask(mask, size, kernel_width, kernel_height);
    erode_binary_mask(&dilated, size, kernel_width, kernel_height)
}

fn dilate_binary_mask(
    mask: &[bool],
    size: Size,
    kernel_width: u32,
    kernel_height: u32,
) -> Vec<bool> {
    let width = size.width as i32;
    let height = size.height as i32;
    let radius_x = (kernel_width as i32 - 1) / 2;
    let radius_y = (kernel_height as i32 - 1) / 2;
    let mut output = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut value = false;
            'window: for dy in -radius_y..=radius_y {
                for dx in -radius_x..=radius_x {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0
                        && ny >= 0
                        && nx < width
                        && ny < height
                        && mask[(ny as u32 * size.width + nx as u32) as usize]
                    {
                        value = true;
                        break 'window;
                    }
                }
            }
            output[(y as u32 * size.width + x as u32) as usize] = value;
        }
    }
    output
}

fn erode_binary_mask(
    mask: &[bool],
    size: Size,
    kernel_width: u32,
    kernel_height: u32,
) -> Vec<bool> {
    let width = size.width as i32;
    let height = size.height as i32;
    let radius_x = (kernel_width as i32 - 1) / 2;
    let radius_y = (kernel_height as i32 - 1) / 2;
    let mut output = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut value = true;
            'window: for dy in -radius_y..=radius_y {
                for dx in -radius_x..=radius_x {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx < 0
                        || ny < 0
                        || nx >= width
                        || ny >= height
                        || !mask[(ny as u32 * size.width + nx as u32) as usize]
                    {
                        value = false;
                        break 'window;
                    }
                }
            }
            output[(y as u32 * size.width + x as u32) as usize] = value;
        }
    }
    output
}

fn connected_component_rects(mask: &[bool], size: Size) -> Vec<Rect> {
    let mut visited = vec![false; mask.len()];
    let mut rects = Vec::new();
    let width = size.width as i32;
    let height = size.height as i32;
    let mut stack = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let start_index = (y as u32 * size.width + x as u32) as usize;
            if visited[start_index] || !mask[start_index] {
                continue;
            }

            visited[start_index] = true;
            stack.clear();
            stack.push((x, y));
            let mut min_x = x;
            let mut max_x = x;
            let mut min_y = y;
            let mut max_y = y;

            while let Some((cx, cy)) = stack.pop() {
                min_x = min_x.min(cx);
                max_x = max_x.max(cx);
                min_y = min_y.min(cy);
                max_y = max_y.max(cy);

                for (nx, ny) in [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)] {
                    if nx < 0 || ny < 0 || nx >= width || ny >= height {
                        continue;
                    }
                    let index = (ny as u32 * size.width + nx as u32) as usize;
                    if visited[index] || !mask[index] {
                        continue;
                    }
                    visited[index] = true;
                    stack.push((nx, ny));
                }
            }

            if let Ok(rect) = Rect::new(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1) {
                rects.push(rect);
            }
        }
    }

    rects
}

fn is_top_right_excluded(rect: Rect, size: Size, spec: VisibleGridEnumerationSpec) -> bool {
    let center = rect.center();
    center.x as f64 >= size.width as f64 * spec.top_right_exclusion_x_ratio
        && center.y as f64 <= size.height as f64 * spec.top_right_exclusion_y_ratio
}

fn average_row_spacing(cells: &[GridCell]) -> f64 {
    let mut count = 0;
    let mut sum = 0;
    let max_col = cells.iter().map(|cell| cell.col).max().unwrap_or(0);
    for col in 0..=max_col {
        let max_row = cells
            .iter()
            .filter(|cell| cell.col == col)
            .map(|cell| cell.row)
            .max()
            .unwrap_or(0);
        for row in 0..max_row {
            let top = cells.iter().find(|cell| cell.col == col && cell.row == row);
            let bottom = cells
                .iter()
                .find(|cell| cell.col == col && cell.row == row + 1);
            if let (Some(top), Some(bottom)) = (top, bottom) {
                sum += bottom.rect.y - top.rect.y - top.rect.height;
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}

fn rect_within_image(rect: Rect, size: Size) -> bool {
    rect.x >= 0
        && rect.y >= 0
        && rect.width > 0
        && rect.height > 0
        && rect.right() <= size.width as i32
        && rect.bottom() <= size.height as i32
}

fn normalized_shift_correlation(
    previous: &[u8],
    next: &[u8],
    size: Size,
    shift_x: i32,
    shift_y: i32,
) -> (f64, usize) {
    let x_start = 0.max(-shift_x) as u32;
    let x_end = (size.width as i32).min(size.width as i32 - shift_x) as u32;
    let y_start = 0.max(-shift_y) as u32;
    let y_end = (size.height as i32).min(size.height as i32 - shift_y) as u32;
    if x_start >= x_end || y_start >= y_end {
        return (0.0, 0);
    }

    let overlap = (x_end - x_start) as usize * (y_end - y_start) as usize;
    let mut previous_sum = 0.0;
    let mut next_sum = 0.0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let previous_index = (y * size.width + x) as usize;
            let next_index =
                ((y as i32 + shift_y) as u32 * size.width + (x as i32 + shift_x) as u32) as usize;
            previous_sum += previous[previous_index] as f64;
            next_sum += next[next_index] as f64;
        }
    }
    let previous_mean = previous_sum / overlap as f64;
    let next_mean = next_sum / overlap as f64;
    let mut numerator = 0.0;
    let mut previous_variance = 0.0;
    let mut next_variance = 0.0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let previous_index = (y * size.width + x) as usize;
            let next_index =
                ((y as i32 + shift_y) as u32 * size.width + (x as i32 + shift_x) as u32) as usize;
            let previous_delta = previous[previous_index] as f64 - previous_mean;
            let next_delta = next[next_index] as f64 - next_mean;
            numerator += previous_delta * next_delta;
            previous_variance += previous_delta * previous_delta;
            next_variance += next_delta * next_delta;
        }
    }
    if previous_variance <= f64::EPSILON || next_variance <= f64::EPSILON {
        return (0.0, overlap);
    }
    (
        numerator / (previous_variance.sqrt() * next_variance.sqrt()),
        overlap,
    )
}
