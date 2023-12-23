use serde::{Deserialize, Serialize};

use crate::painting::Color;

use super::FlexibleLength;

/// An immutable description of how to paint a box.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoxDecoration {
    /// The color to fill in the background of the box.
    #[serde(default)]
    pub color: Color,
    /// The border of the box.
    #[serde(default)]
    pub border: BoxBorder,
}

// TODO(hawtian): support different border widths for different sides.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoxBorder {
    /// The color of the border.
    #[serde(default)]
    pub color: Color,
    /// The width for all sides.
    #[serde(default)]
    pub width: f32,
    /// The radius of the rounded corners of the box.
    #[serde(default)]
    pub radius: FlexibleLength,
}

impl BoxBorder {
    pub const NONE: Self = Self {
        color: Color::new(0),
        width: 0.0,
        radius: FlexibleLength::Fixed(0.0),
    };
}
