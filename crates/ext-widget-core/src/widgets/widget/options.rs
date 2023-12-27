use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
            ..Default::default()
        }
    }
}
