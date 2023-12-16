use taffy::Dimension;

#[derive(Debug, Clone)]
pub struct BoxConstraints {
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        Self {
            min_width: Dimension::Auto,
            max_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_height: Dimension::Auto,
        }
    }
}
