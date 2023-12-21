use std::num::ParseFloatError;

use nvim_oxi::{Object, ObjectKind};
use taffy::{Dimension, LengthPercentage, LengthPercentageAuto};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexibleLength {
    /// Fixed size
    Fixed(f32),
    /// Percent size, [0, 1]
    Percent(f32),
}

#[derive(Error, Debug)]
pub enum ParseFlexibleLengthError {
    #[error("invalid float")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("invalid percent")]
    InvalidPercent,
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid type, expected float or string, got {0:?}")]
    InvalidType(ObjectKind),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexibleLengthAuto {
    /// Fixed size
    Fixed(f32),
    /// Percent size, [0, 1]
    Percent(f32),
    /// Auto size
    Auto,
}

impl Default for FlexibleLengthAuto {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<FlexibleLength> for LengthPercentage {
    fn from(size: FlexibleLength) -> Self {
        match size {
            FlexibleLength::Fixed(size) => LengthPercentage::Length(size),
            FlexibleLength::Percent(percent) => {
                LengthPercentage::Percent(percent)
            }
        }
    }
}

impl From<FlexibleLength> for LengthPercentageAuto {
    fn from(size: FlexibleLength) -> Self {
        match size {
            FlexibleLength::Fixed(size) => LengthPercentageAuto::Length(size),
            FlexibleLength::Percent(percent) => {
                LengthPercentageAuto::Percent(percent)
            }
        }
    }
}

impl From<FlexibleLengthAuto> for LengthPercentageAuto {
    fn from(size: FlexibleLengthAuto) -> Self {
        match size {
            FlexibleLengthAuto::Fixed(size) => {
                LengthPercentageAuto::Length(size)
            }
            FlexibleLengthAuto::Percent(percent) => {
                LengthPercentageAuto::Percent(percent)
            }
            FlexibleLengthAuto::Auto => LengthPercentageAuto::Auto,
        }
    }
}

impl From<FlexibleLength> for FlexibleLengthAuto {
    fn from(size: FlexibleLength) -> Self {
        match size {
            FlexibleLength::Fixed(size) => FlexibleLengthAuto::Fixed(size),
            FlexibleLength::Percent(percent) => {
                FlexibleLengthAuto::Percent(percent)
            }
        }
    }
}

impl From<FlexibleLengthAuto> for Dimension {
    fn from(size: FlexibleLengthAuto) -> Self {
        match size {
            FlexibleLengthAuto::Fixed(size) => Dimension::Length(size),
            FlexibleLengthAuto::Percent(percent) => Dimension::Percent(percent),
            FlexibleLengthAuto::Auto => Dimension::Auto,
        }
    }
}

impl TryFrom<Object> for FlexibleLength {
    type Error = ParseFlexibleLengthError;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.kind() {
            ObjectKind::Integer => {
                // SAFETY: `obj` kind has been checked
                let i = unsafe { obj.as_integer_unchecked() };
                Ok(Self::Fixed(i as f32))
            }
            ObjectKind::Float => {
                // SAFETY: `obj` kind has been checked
                let f = unsafe { obj.as_float_unchecked() };
                Ok(Self::Fixed(f as f32))
            }
            ObjectKind::String => {
                // SAFETY: `obj` kind has been checked
                let _s = unsafe { obj.into_string_unchecked() };
                let s = _s.to_string_lossy();
                if let Some(p) = s.strip_suffix('%') {
                    let f = p.parse::<f32>()? / 100.0;
                    if !(0.0..=1.0).contains(&f) {
                        Err(ParseFlexibleLengthError::InvalidPercent)
                    } else {
                        Ok(Self::Percent(f))
                    }
                } else {
                    Err(ParseFlexibleLengthError::InvalidFormat)
                }
            }
            _ => Err(ParseFlexibleLengthError::InvalidType(obj.kind())),
        }
    }
}

impl TryFrom<Object> for FlexibleLengthAuto {
    type Error = ParseFlexibleLengthError;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.kind() {
            ObjectKind::Integer => {
                // SAFETY: `obj` kind has been checked
                let i = unsafe { obj.as_integer_unchecked() };
                Ok(Self::Fixed(i as f32))
            }
            ObjectKind::Float => {
                // SAFETY: `obj` kind has been checked
                let f = unsafe { obj.as_float_unchecked() };
                Ok(Self::Fixed(f as f32))
            }
            ObjectKind::String => {
                // SAFETY: `obj` kind has been checked
                let _s = unsafe { obj.into_string_unchecked() };
                let s = _s.to_string_lossy();
                if let Some(p) = s.strip_suffix('%') {
                    let f = p.parse::<f32>()? / 100.0;
                    if !(0.0..=1.0).contains(&f) {
                        Err(ParseFlexibleLengthError::InvalidPercent)
                    } else {
                        Ok(Self::Percent(f))
                    }
                } else if s == "auto" {
                    Ok(Self::Auto)
                } else {
                    Err(ParseFlexibleLengthError::InvalidFormat)
                }
            }
            _ => Err(ParseFlexibleLengthError::InvalidType(obj.kind())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RectSize<T> {
    pub width: T,
    pub height: T,
}

impl<T: Default> Default for RectSize<T> {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl<T> From<taffy::Size<T>> for RectSize<T> {
    fn from(value: taffy::Size<T>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl<T> From<RectSize<T>> for taffy::Size<T> {
    fn from(value: RectSize<T>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}
