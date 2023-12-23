use serde::{Deserialize, Serialize};
use std::num::ParseFloatError;
use taffy::{LengthPercentage, LengthPercentageAuto};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum FlexibleLengthRepr {
    /// Fixed size
    Fixed(f32),
    /// Percent size, [0, 1]
    Percent(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FlexibleLengthRepr", into = "FlexibleLengthRepr")]
pub enum FlexibleLength {
    /// Fixed size
    Fixed(f32),
    /// Percent size, [0, 1]
    Percent(f32),
}

impl Default for FlexibleLength {
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

#[derive(Error, Debug)]
pub enum ParseFlexibleLengthError {
    #[error("invalid float")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("invalid percent")]
    InvalidPercent,
    #[error("invalid format")]
    InvalidFormat,
}

impl From<FlexibleLength> for LengthPercentage {
    fn from(value: FlexibleLength) -> Self {
        match value {
            FlexibleLength::Fixed(f) => Self::Length(f),
            FlexibleLength::Percent(f) => Self::Percent(f),
        }
    }
}

impl From<FlexibleLength> for LengthPercentageAuto {
    fn from(value: FlexibleLength) -> Self {
        match value {
            FlexibleLength::Fixed(f) => Self::Length(f),
            FlexibleLength::Percent(f) => Self::Percent(f),
        }
    }
}

impl TryFrom<FlexibleLengthRepr> for FlexibleLength {
    type Error = ParseFlexibleLengthError;

    fn try_from(s: FlexibleLengthRepr) -> Result<Self, Self::Error> {
        match s {
            FlexibleLengthRepr::Fixed(f) => Ok(Self::Fixed(f)),
            FlexibleLengthRepr::Percent(s) => {
                if let Some(p) = s.strip_suffix('%') {
                    let f = p.parse::<f32>()? / 100.0;
                    if !(0.0..=1.0).contains(&f) {
                        Err(ParseFlexibleLengthError::InvalidPercent)
                    } else {
                        Ok(Self::Percent(f))
                    }
                } else {
                    let f = s.parse::<f32>()?;
                    Ok(Self::Fixed(f))
                }
            }
        }
    }
}

impl From<FlexibleLength> for FlexibleLengthRepr {
    fn from(value: FlexibleLength) -> Self {
        match value {
            FlexibleLength::Fixed(f) => Self::Fixed(f),
            FlexibleLength::Percent(f) => {
                Self::Percent(format!("{:.2}%", f * 100.0))
            }
        }
    }
}
