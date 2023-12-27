use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};
use taffy::{LengthPercentage, LengthPercentageAuto, Point, Rect};

use super::{FlexibleLength, FlexibleLengthAuto};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BoxConstraints {
    #[serde(default)]
    pub min_width: FlexibleLengthAuto,
    #[serde(default)]
    pub max_width: FlexibleLengthAuto,
    #[serde(default)]
    pub min_height: FlexibleLengthAuto,
    #[serde(default)]
    pub max_height: FlexibleLengthAuto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    #[serde(default)]
    pub left: FlexibleLength,
    #[serde(default)]
    pub right: FlexibleLength,
    #[serde(default)]
    pub top: FlexibleLength,
    #[serde(default)]
    pub bottom: FlexibleLength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margin {
    #[serde(default)]
    pub left: FlexibleLength,
    #[serde(default)]
    pub right: FlexibleLength,
    #[serde(default)]
    pub top: FlexibleLength,
    #[serde(default)]
    pub bottom: FlexibleLength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Axis {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpacePolicy {
    Fixed(f32),
    #[serde(rename = "shrink")]
    Shrink,
    #[serde(rename = "expand")]
    Expand,
}

impl Default for Axis {
    fn default() -> Self {
        Self::Horizontal
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            left: FlexibleLength::Fixed(0.),
            right: FlexibleLength::Fixed(0.),
            top: FlexibleLength::Fixed(0.),
            bottom: FlexibleLength::Fixed(0.),
        }
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            left: FlexibleLength::Fixed(0.),
            right: FlexibleLength::Fixed(0.),
            top: FlexibleLength::Fixed(0.),
            bottom: FlexibleLength::Fixed(0.),
        }
    }
}

impl Padding {
    pub const ZERO: Self = Padding {
        left: FlexibleLength::Fixed(0.),
        right: FlexibleLength::Fixed(0.),
        top: FlexibleLength::Fixed(0.),
        bottom: FlexibleLength::Fixed(0.),
    };

    pub fn all(size: FlexibleLength) -> Self {
        Self {
            left: size,
            right: size,
            top: size,
            bottom: size,
        }
    }

    pub fn horizontal(size: FlexibleLength) -> Self {
        Self {
            left: size,
            right: size,
            top: FlexibleLength::Fixed(0.),
            bottom: FlexibleLength::Fixed(0.),
        }
    }

    pub fn vertical(size: FlexibleLength) -> Self {
        Self {
            left: FlexibleLength::Fixed(0.),
            right: FlexibleLength::Fixed(0.),
            top: size,
            bottom: size,
        }
    }
}

impl Margin {
    pub const ZERO: Self = Margin {
        left: FlexibleLength::Fixed(0.),
        right: FlexibleLength::Fixed(0.),
        top: FlexibleLength::Fixed(0.),
        bottom: FlexibleLength::Fixed(0.),
    };

    pub fn all(size: FlexibleLength) -> Self {
        Self {
            left: size,
            right: size,
            top: size,
            bottom: size,
        }
    }

    pub fn horizontal(size: FlexibleLength) -> Self {
        Self {
            left: size,
            right: size,
            top: FlexibleLength::Fixed(0.),
            bottom: FlexibleLength::Fixed(0.),
        }
    }

    pub fn vertical(size: FlexibleLength) -> Self {
        Self {
            left: FlexibleLength::Fixed(0.),
            right: FlexibleLength::Fixed(0.),
            top: size,
            bottom: size,
        }
    }
}

impl From<Padding> for Rect<LengthPercentage> {
    fn from(padding: Padding) -> Self {
        Self {
            left: padding.left.into(),
            right: padding.right.into(),
            top: padding.top.into(),
            bottom: padding.bottom.into(),
        }
    }
}

impl From<Margin> for Rect<LengthPercentage> {
    fn from(margin: Margin) -> Self {
        Self {
            left: margin.left.into(),
            right: margin.right.into(),
            top: margin.top.into(),
            bottom: margin.bottom.into(),
        }
    }
}

impl From<Margin> for Rect<LengthPercentageAuto> {
    fn from(margin: Margin) -> Self {
        Self {
            left: margin.left.into(),
            right: margin.right.into(),
            top: margin.top.into(),
            bottom: margin.bottom.into(),
        }
    }
}

impl From<Axis> for taffy::FlexDirection {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => Self::Row,
            Axis::Vertical => Self::Column,
        }
    }
}

impl From<Point<f32>> for Location {
    fn from(value: Point<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl AddAssign for Location {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<Point<f32>> for Location {
    fn add_assign(&mut self, rhs: Point<f32>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Point<f32>> for Location {
    type Output = Self;

    fn add(self, rhs: Point<f32>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Location> for Location {
    type Output = Self;

    fn add(self, rhs: Location) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Location {
    pub const ZERO: Location = Location { x: 0.0, y: 0.0 };
}

impl From<Location> for skia_safe::Point {
    fn from(value: Location) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<taffy::AvailableSpace> for SpacePolicy {
    fn from(value: taffy::AvailableSpace) -> Self {
        match value {
            taffy::AvailableSpace::Definite(v) => Self::Fixed(v),
            taffy::AvailableSpace::MinContent => Self::Shrink,
            taffy::AvailableSpace::MaxContent => Self::Expand,
        }
    }
}
