use serde::{Deserialize, Serialize};

use super::{Result, VisionError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Result<Self> {
        if width < 0 || height < 0 {
            return Err(VisionError::InvalidRect);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn right(self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(self) -> i32 {
        self.y + self.height
    }

    pub fn center(self) -> Point {
        Point {
            x: self.x + self.width / 2,
            y: self.y + self.height / 2,
        }
    }

    pub fn is_empty(self) -> bool {
        self.x == 0 && self.y == 0 && self.width == 0 && self.height == 0
    }

    pub fn clamp_to(self, size: Size) -> Result<Self> {
        if self.width < 0 || self.height < 0 {
            return Err(VisionError::InvalidRect);
        }

        let max_width = size.width as i32;
        let max_height = size.height as i32;
        let x = self.x.clamp(0, max_width);
        let y = self.y.clamp(0, max_height);
        let right = self.right().clamp(x, max_width);
        let bottom = self.bottom().clamp(y, max_height);
        Self::new(x, y, right - x, bottom - y)
    }
}
