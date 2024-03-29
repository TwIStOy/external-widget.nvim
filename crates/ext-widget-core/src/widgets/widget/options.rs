use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use taffy::Rect;

use crate::painting::{
    Axis, BoxConstraints, FlexibleLength, Margin, Padding, RectSize,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BoxOptions {
    #[serde(default)]
    pub constraints: BoxConstraints,
    #[serde(default)]
    pub padding: Padding,
    #[serde(default)]
    pub margin: Margin,
    #[serde(default)]
    pub axis: Axis,
    #[serde(default)]
    pub gap: RectSize<FlexibleLength>,
    #[serde(default)]
    pub border: f32,
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
            gap: value.gap.into(),
            border: Rect {
                left: taffy::LengthPercentage::Length(value.border),
                right: taffy::LengthPercentage::Length(value.border),
                top: taffy::LengthPercentage::Length(value.border),
                bottom: taffy::LengthPercentage::Length(value.border),
            },
            ..Default::default()
        }
    }
}
