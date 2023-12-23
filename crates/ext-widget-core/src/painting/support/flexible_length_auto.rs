use serde::{Deserialize, Serialize};
use taffy::{Dimension, LengthPercentageAuto};

use super::flexible_length::FlexibleLength;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum FlexibleLengthAutoRepr {
    /// Fixed size
    Fixed(f32),
    /// Percent size, [0, 1]
    Percent(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FlexibleLengthAutoRepr", into = "FlexibleLengthAutoRepr")]
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

impl From<FlexibleLengthAuto> for FlexibleLengthAutoRepr {
    fn from(value: FlexibleLengthAuto) -> Self {
        match value {
            FlexibleLengthAuto::Fixed(f) => Self::Fixed(f),
            FlexibleLengthAuto::Percent(f) => {
                Self::Percent(format!("{:.2}%", f))
            }
            FlexibleLengthAuto::Auto => Self::Percent("auto".to_string()),
        }
    }
}

impl From<FlexibleLengthAutoRepr> for FlexibleLengthAuto {
    fn from(s: FlexibleLengthAutoRepr) -> Self {
        match s {
            FlexibleLengthAutoRepr::Fixed(f) => Self::Fixed(f),
            FlexibleLengthAutoRepr::Percent(s) => {
                if s == "auto" {
                    Self::Auto
                } else if let Some(p) = s.strip_suffix('%') {
                    let f = p.parse::<f32>().unwrap() / 100.0;
                    if !(0.0..=1.0).contains(&f) {
                        panic!("invalid percent");
                    } else {
                        Self::Percent(f)
                    }
                } else {
                    let f = s.parse::<f32>().unwrap();
                    Self::Fixed(f)
                }
            }
        }
    }
}
