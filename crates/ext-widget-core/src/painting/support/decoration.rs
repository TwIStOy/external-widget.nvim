use crate::painting::Color;

/// An immutable description of how to paint a box.
pub struct BoxDecoration {
    /// The color to fill in the background of the box.
    pub color: Color,
    /// The border of the box.
    pub border: BoxBorder,
}

// TODO(hawtian): support different border widths for different sides.
pub struct BoxBorder {
    /// The color of the border.
    pub color: Color,
    /// The width for all sides.
    pub width: f32,
}
