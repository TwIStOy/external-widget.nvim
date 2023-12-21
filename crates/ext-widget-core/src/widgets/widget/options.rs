use nvim_oxi::{Object, ObjectKind};
use std::fmt::Debug;
use thiserror::Error;

use crate::painting::{
    Axis, BoxConstraints, Margin, Padding, ParseAxisError,
    ParseBoxConstraintsError, ParsePaddingError,
};

#[derive(Debug, Default, Clone)]
pub struct BoxOptions {
    pub constraints: BoxConstraints,
    pub padding: Padding,
    pub margin: Margin,
    pub axis: Axis,
}

#[derive(Debug, Error)]
pub enum ParseBoxOptionsError {
    #[error("invalid type, expected object, got {0:?}")]
    Type(ObjectKind),
    #[error("invalid field: {0:?}")]
    Constraits(#[from] ParseBoxConstraintsError),
    #[error("invalid padding/margin: {0:?}")]
    PaddingOrMargin(#[from] ParsePaddingError),
    #[error("invalid axis: {0:?}")]
    Axis(#[from] ParseAxisError),
}

impl TryFrom<Object> for BoxOptions {
    type Error = ParseBoxOptionsError;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.kind() {
            ObjectKind::Nil => Ok(Self::default()),
            ObjectKind::Dictionary => {
                let mut ret = Self::default();
                let dict = unsafe { obj.into_dict_unchecked() };
                for (key, value) in dict.into_iter() {
                    match &*key.to_string_lossy() {
                        "constraints" => {
                            ret.constraints = value.try_into()?;
                        }
                        "padding" => {
                            ret.padding = value.try_into()?;
                        }
                        "margin" => {
                            ret.margin = value.try_into()?;
                        }
                        "axis" => {
                            ret.axis = value.try_into()?;
                        }
                        _ => {}
                    }
                }
                Ok(ret)
            }
            _ => Err(ParseBoxOptionsError::Type(obj.kind())),
        }
    }
}

impl From<BoxOptions> for taffy::Style {
    fn from(value: BoxOptions) -> Self {
        Self {
            margin: value.margin.into(),
            padding: value.padding.into(),
            min_size: taffy::Size {
                width: value.constraints.min_width.into(),
                height: value.constraints.min_height.into(),
            },
            max_size: taffy::Size {
                width: value.constraints.max_width.into(),
                height: value.constraints.max_height.into(),
            },
            flex_direction: value.axis.into(),
            ..Default::default()
        }
    }
}
