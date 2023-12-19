use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    pub normal_font: String,
    pub mono_font: String,
    pub font_size: u32,
}

impl Configuration {
    /// Creates a new [`Configuration`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            normal_font: "Sans".to_string(),
            mono_font: "monospace".to_string(),
            font_size: 16,
        }
    }
}
