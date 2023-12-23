use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(try_from = "ColorRepr", into = "ColorRepr")]
pub struct Color {
    // range: [0, 255]
    pub red: u8,
    // range: [0, 255]
    pub green: u8,
    // range: [0, 255]
    pub blue: u8,
    // range: [0, 255]
    pub alpha: u8,
}

impl Color {
    /// Construct a color from the 24-bit or 32-bit integer.
    pub const fn new(value: u32) -> Self {
        if value <= 0xffffff {
            // 24-bit, no alpha in this
            Self {
                red: ((value >> 16) & 0xff) as u8,
                green: ((value >> 8) & 0xff) as u8,
                blue: (value & 0xff) as u8,
                alpha: 0xff,
            }
        } else {
            Self {
                red: ((value >> 24) & 0xff) as u8,
                green: ((value >> 16) & 0xff) as u8,
                blue: ((value >> 8) & 0xff) as u8,
                alpha: (value & 0xff) as u8,
            }
        }
    }

    pub fn new_from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn new_from_rgbo(red: u8, green: u8, blue: u8, opacity: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: (opacity * 255.0) as u8,
        }
    }

    pub fn new_from_str<T>(s: T) -> Result<Self, ParseColorError>
    where
        T: AsRef<str>,
    {
        if let Some(v) = s.as_ref().strip_prefix('#') {
            let r = u8::from_str_radix(&v[0..2], 16)?;
            let g = u8::from_str_radix(&v[2..4], 16)?;
            let b = u8::from_str_radix(&v[4..6], 16)?;
            let a = if v.len() == 6 {
                // 24-bit, no alpha in this
                0xff
            } else {
                u8::from_str_radix(&v[6..8], 16)?
            };
            Ok(Color {
                red: r,
                green: g,
                blue: b,
                alpha: a,
            })
        } else {
            match s.as_ref() {
                "black" => Ok(Color::new(0x000000)),
                "silver" => Ok(Color::new(0xc0c0c0)),
                "gray" => Ok(Color::new(0x808080)),
                "white" => Ok(Color::new(0xffffff)),
                "maroon" => Ok(Color::new(0x800000)),
                "red" => Ok(Color::new(0xff0000)),
                "purple" => Ok(Color::new(0x800080)),
                "fuchsia" => Ok(Color::new(0xff00ff)),
                "green" => Ok(Color::new(0x008000)),
                "lime" => Ok(Color::new(0x00ff00)),
                "olive" => Ok(Color::new(0x808000)),
                "yellow" => Ok(Color::new(0xffff00)),
                "navy" => Ok(Color::new(0x000080)),
                "blue" => Ok(Color::new(0x0000ff)),
                "teal" => Ok(Color::new(0x008080)),
                "aqua" => Ok(Color::new(0x00ffff)),
                _ => Err(ParseColorError::InvalidColorFormat(
                    s.as_ref().to_string(),
                )),
            }
        }
    }

    /// The alpha channel of this color as a double. In [0, 1]
    pub fn opacity(&self) -> f32 {
        self.alpha as f32 / 255.0
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Error)]
pub enum ParseColorError {
    #[error("invalid type, expected string or integer, got {0:?}")]
    Type(String),
    #[error("invalid color: {0:?}")]
    InvalidColorPart(#[from] std::num::ParseIntError),
    #[error("invalid color: {0:?}")]
    InvalidColorFormat(String),
}

impl From<Color> for skia_safe::Color {
    fn from(value: Color) -> Self {
        Self::from_argb(value.alpha, value.red, value.green, value.blue)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ColorRepr {
    /// 24-bit or 32-bit integer
    Integer(u32),
    /// String
    String(String),
}

impl TryFrom<ColorRepr> for Color {
    type Error = ParseColorError;

    fn try_from(value: ColorRepr) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&ColorRepr> for Color {
    type Error = ParseColorError;

    fn try_from(value: &ColorRepr) -> Result<Self, Self::Error> {
        match value {
            ColorRepr::Integer(i) => Ok(Color::new(*i)),
            ColorRepr::String(s) => Self::new_from_str(s),
        }
    }
}

impl From<Color> for ColorRepr {
    fn from(value: Color) -> Self {
        Self::String(value.to_string())
    }
}
