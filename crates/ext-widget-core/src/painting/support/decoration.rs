use crate::painting::Color;

use super::FlexibleLength;

/// An immutable description of how to paint a box.
#[derive(Debug, Clone)]
pub struct BoxDecoration {
    /// The color to fill in the background of the box.
    pub color: Color,
    /// The border of the box.
    pub border: BoxBorder,
}

// TODO(hawtian): support different border widths for different sides.
#[derive(Debug, Clone)]
pub struct BoxBorder {
    /// The color of the border.
    pub color: Color,
    /// The width for all sides.
    pub width: f32,
    /// The radius of the rounded corners of the box.
    pub radius: FlexibleLength,
}
