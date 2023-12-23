use thiserror::Error;

use crate::painting::{Color, ParseColorError};

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

impl BoxBorder {
    pub const NONE: Self = Self {
        color: Color::new(0),
        width: 0.0,
        radius: FlexibleLength::Fixed(0.0),
    };
}

#[derive(Error, Debug)]
pub enum ParseBoxBorderError {
    #[error("invalid border color: {0:?}")]
    Color(#[from] ParseColorError),
    #[error("invalid border radius")]
    Radius,
    #[error("invalid border width")]
    Width,
}

// impl TryFrom<rmpv::Value> for BoxBorder {
//     type Error = ParseBoxBorderError;
//     fn try_from(value: rmpv::Value) -> Result<Self, Self::Error> {
//         let mut color = Color::new(0);
//         let mut width = 0.0;
//         let mut radius = 0.0;
//         if let rmpv::Value::Map(map) = value {
//             for (k, v) in map.iter() {
//                 if let rmpv::Value::String(s) = k {
//                     match s.as_str().unwrap() {
//                         "color" => {
//                             color = v.try_into();
//                         }
//                         "width" => {
//                             width = v.as_f64().unwrap_or(0.0) as f32;
//                         }
//                         "radius" => {
//                             radius = v.as_f64().unwrap_or(0.0) as f32;
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//         }
//         Self {
//             color,
//             width,
//             radius: FlexibleLength::Fixed(radius),
//         }
//     }
// }
