use std::fmt::{Debug, Display};

use nvim_oxi::{Object, ObjectKind};
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn new(value: u32) -> Self {
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
    Type(ObjectKind),
    #[error("invalid color: {0:?}")]
    InvalidColorPart(#[from] std::num::ParseIntError),
    #[error("invalid color: {0:?}")]
    InvalidColorFormat(String),
}

impl TryFrom<Object> for Color {
    type Error = ParseColorError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value.kind() {
            ObjectKind::Integer => {
                let v = unsafe { value.as_integer_unchecked() };
                Ok(Color::new(v as u32))
            }
            ObjectKind::String => {
                let _v = unsafe { value.into_string_unchecked() };
                let v = _v.to_string_lossy();
                if let Some(v) = v.strip_prefix('#') {
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
                    Err(ParseColorError::InvalidColorFormat(v.to_string()))
                }
            }
            _ => Err(ParseColorError::Type(value.kind())),
        }
    }
}

impl From<Color> for skia_safe::Color {
    fn from(value: Color) -> Self {
        Self::from_argb(value.alpha, value.red, value.green, value.blue)
    }
}
