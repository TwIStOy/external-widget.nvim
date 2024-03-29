use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExtWidgetConfig {
    pub hover: HoverConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoverConfig {
    #[serde(default = "default_normal_font")]
    pub normal_font: Vec<String>,
    #[serde(default = "default_font_size")]
    pub normal_font_size: f32,
    #[serde(default = "default_mono_font")]
    pub mono_font: Vec<String>,
    #[serde(default = "default_font_size")]
    pub mono_font_size: f32,
    pub window: WindowConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_size_width")]
    pub max_width: f32,
    #[serde(default = "default_window_size_height")]
    pub max_height: f32,
    #[serde(default = "default_x_offset")]
    pub x_offset: i32,
    #[serde(default = "default_y_offset")]
    pub y_offset: i32,
}

impl Default for HoverConfig {
    fn default() -> Self {
        Self {
            normal_font: default_normal_font(),
            normal_font_size: default_font_size(),
            mono_font: default_mono_font(),
            mono_font_size: default_font_size(),
            window: default_hover_window_size(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            max_width: default_window_size_width(),
            max_height: default_window_size_height(),
            x_offset: default_x_offset(),
            y_offset: default_y_offset(),
        }
    }
}

fn default_window_size_width() -> f32 {
    1000.0
}

fn default_window_size_height() -> f32 {
    1000.0
}

fn default_x_offset() -> i32 {
    0
}

fn default_y_offset() -> i32 {
    1
}

fn default_hover_window_size() -> WindowConfig {
    WindowConfig {
        max_width: 1000.0,
        max_height: 1000.0,
        x_offset: 0,
        y_offset: 1,
    }
}

fn default_normal_font() -> Vec<String> {
    vec!["LXGW WenKai".to_string()]
}

fn default_mono_font() -> Vec<String> {
    vec!["MonoLisa".to_string()]
}

fn default_font_size() -> f32 {
    20.0
}

pub static CONFIG: Lazy<Mutex<ExtWidgetConfig>> =
    Lazy::new(|| Mutex::new(ExtWidgetConfig::default()));
