use nvim_oxi::{Object, ObjectKind};
use taffy::{LengthPercentage, LengthPercentageAuto, Rect};
use thiserror::Error;

use super::{
    style::ParseFlexibleLengthError, FlexibleLength, FlexibleLengthAuto,
};

#[derive(Debug, Default, Clone)]
pub struct BoxConstraints {
    pub min_width: FlexibleLengthAuto,
    pub max_width: FlexibleLengthAuto,
    pub min_height: FlexibleLengthAuto,
    pub max_height: FlexibleLengthAuto,
}

#[derive(Debug, Clone)]
pub struct Padding {
    pub left: FlexibleLength,
    pub right: FlexibleLength,
    pub top: FlexibleLength,
    pub bottom: FlexibleLength,
}

#[derive(Debug, Clone)]
pub struct Margin {
    pub left: FlexibleLength,
    pub right: FlexibleLength,
    pub top: FlexibleLength,
    pub bottom: FlexibleLength,
}

#[derive(Debug, Clone)]
pub enum Axis {
    Horizontal,
    Vertical,
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

#[derive(Error, Debug)]
pub enum ParseBoxConstraintsError {
    #[error("invalid type, expect dictionary, got {0:?}")]
    InvalidType(ObjectKind),
    #[error("invalid value, {0}")]
    BadString(String),
    #[error("invalid size value, {0:?}")]
    InvalidSizeValue(#[from] ParseFlexibleLengthError),
}

impl TryFrom<Object> for BoxConstraints {
    type Error = ParseBoxConstraintsError;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object.kind() {
            ObjectKind::Nil => Ok(Self::default()),
            ObjectKind::String => {
                let _s = unsafe { object.into_string_unchecked() };
                let s = _s.to_string_lossy();
                if s == "auto" {
                    Ok(Self::default())
                } else {
                    Err(ParseBoxConstraintsError::BadString(s.to_string()))
                }
            }
            ObjectKind::Dictionary => {
                let mut min_width = FlexibleLengthAuto::Auto;
                let mut max_width = FlexibleLengthAuto::Auto;
                let mut min_height = FlexibleLengthAuto::Auto;
                let mut max_height = FlexibleLengthAuto::Auto;
                let dict = unsafe { object.into_dict_unchecked() };
                for (_key, value) in dict.into_iter() {
                    let key = _key.to_string_lossy();
                    match &*key {
                        "min_width" => {
                            min_width = value.try_into()?;
                        }
                        "max_width" => {
                            max_width = value.try_into()?;
                        }
                        "min_height" => {
                            min_height = value.try_into()?;
                        }
                        "max_height" => {
                            max_height = value.try_into()?;
                        }
                        _ => (),
                    }
                }
                Ok(Self {
                    min_width,
                    max_width,
                    min_height,
                    max_height,
                })
            }
            _ => Err(ParseBoxConstraintsError::InvalidType(object.kind())),
        }
    }
}

#[derive(Error, Debug)]
pub enum ParsePaddingError {
    #[error("invalid type, expect dictionary, got {0:?}")]
    InvalidType(ObjectKind),
    #[error("invalid value, {0}")]
    BadString(String),
    #[error("invalid size value, {0:?}")]
    InvalidSizeValue(#[from] ParseFlexibleLengthError),
}

impl TryFrom<Object> for Padding {
    type Error = ParsePaddingError;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object.kind() {
            ObjectKind::Nil => Ok(Self::default()),
            ObjectKind::String => {
                let _s = unsafe { object.into_string_unchecked() };
                let s = _s.to_string_lossy();
                if s == "auto" {
                    Ok(Self::default())
                } else {
                    Err(ParsePaddingError::BadString(s.to_string()))
                }
            }
            ObjectKind::Dictionary => {
                let mut left = FlexibleLength::Fixed(0.);
                let mut right = FlexibleLength::Fixed(0.);
                let mut top = FlexibleLength::Fixed(0.);
                let mut bottom = FlexibleLength::Fixed(0.);
                let dict = unsafe { object.into_dict_unchecked() };
                for (_key, value) in dict.into_iter() {
                    let key = _key.to_string_lossy();
                    match &*key {
                        "left" => {
                            left = value.try_into()?;
                        }
                        "right" => {
                            right = value.try_into()?;
                        }
                        "top" => {
                            top = value.try_into()?;
                        }
                        "bottom" => {
                            bottom = value.try_into()?;
                        }
                        _ => (),
                    }
                }
                Ok(Self {
                    left,
                    right,
                    top,
                    bottom,
                })
            }
            _ => Err(ParsePaddingError::InvalidType(object.kind())),
        }
    }
}

impl TryFrom<Object> for Margin {
    type Error = ParsePaddingError;

    fn try_from(object: Object) -> Result<Self, Self::Error> {
        match object.kind() {
            ObjectKind::Nil => Ok(Self::default()),
            ObjectKind::String => {
                let _s = unsafe { object.into_string_unchecked() };
                let s = _s.to_string_lossy();
                if s == "auto" {
                    Ok(Self::default())
                } else {
                    Err(ParsePaddingError::BadString(s.to_string()))
                }
            }
            ObjectKind::Dictionary => {
                let mut left = FlexibleLength::Fixed(0.);
                let mut right = FlexibleLength::Fixed(0.);
                let mut top = FlexibleLength::Fixed(0.);
                let mut bottom = FlexibleLength::Fixed(0.);
                let dict = unsafe { object.into_dict_unchecked() };
                for (_key, value) in dict.into_iter() {
                    let key = _key.to_string_lossy();
                    match &*key {
                        "left" => {
                            left = value.try_into()?;
                        }
                        "right" => {
                            right = value.try_into()?;
                        }
                        "top" => {
                            top = value.try_into()?;
                        }
                        "bottom" => {
                            bottom = value.try_into()?;
                        }
                        _ => (),
                    }
                }
                Ok(Self {
                    left,
                    right,
                    top,
                    bottom,
                })
            }
            _ => Err(ParsePaddingError::InvalidType(object.kind())),
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseAxisError {
    #[error("invalid type, expect string, got {0:?}")]
    InvalidType(ObjectKind),
    #[error("invalid value, {0}")]
    BadString(String),
}

impl TryFrom<Object> for Axis {
    type Error = ParseAxisError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value.kind() {
            ObjectKind::String => {
                let s = unsafe { value.into_string_unchecked() };
                let s = s.to_string_lossy();
                match &*s {
                    "horizontal" => Ok(Self::Horizontal),
                    "vertical" => Ok(Self::Vertical),
                    _ => Err(ParseAxisError::BadString(s.to_string())),
                }
            }
            _ => Err(ParseAxisError::InvalidType(value.kind())),
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
