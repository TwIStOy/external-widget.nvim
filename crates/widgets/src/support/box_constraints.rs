use taffy::{Dimension, LengthPercentage, Rect, LengthPercentageAuto};

#[derive(Debug, Clone)]
pub struct BoxConstraints {
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,
    pub padding: Rect<LengthPercentage>,
    pub margin: Rect<LengthPercentageAuto>,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        Self {
            min_width: Dimension::Auto,
            max_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_height: Dimension::Auto,
            padding: Rect {
                left: LengthPercentage::Length(0.),
                right: LengthPercentage::Length(0.),
                top: LengthPercentage::Length(0.),
                bottom: LengthPercentage::Length(0.),
            },
            margin: Rect {
                left: LengthPercentageAuto::Auto,
                right: LengthPercentageAuto::Auto,
                top: LengthPercentageAuto::Auto,
                bottom: LengthPercentageAuto::Auto,
            },
        }
    }
}
