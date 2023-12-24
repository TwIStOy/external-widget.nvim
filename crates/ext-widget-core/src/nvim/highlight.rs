use serde::{Deserialize, Serialize};
use skia_safe::font_style::{
    Slant as SkSlant, Weight as SkWeight, Width as SkWidth,
};
use skia_safe::textlayout::{TextDecoration, TextDecorationStyle};
use skia_safe::Color as SkColor;
use skia_safe::{textlayout::TextStyle, FontStyle as SkFontStyle, Paint};

use crate::painting::Color;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighlightInfos {
    #[serde(alias = "fg", alias = "foreground")]
    pub fg: Option<Color>,
    #[serde(alias = "bg", alias = "background")]
    pub bg: Option<Color>,
    #[serde(alias = "sp", alias = "special")]
    pub sp: Option<Color>,
    pub guifg: Option<Color>,
    pub guibg: Option<Color>,
    pub guisp: Option<Color>,
    pub blend: Option<Color>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub undercurl: Option<bool>,
    #[serde(alias = "underdouble", alias = "underlineline")]
    pub underdouble: Option<bool>,
    #[serde(alias = "underdot", alias = "underdotted")]
    pub underdotted: Option<bool>,
    #[serde(alias = "underdash", alias = "underdashed")]
    pub underdashed: Option<bool>,
    pub strikethrough: Option<bool>,
    pub italic: Option<bool>,
    pub link: Option<String>,
}

impl HighlightInfos {
    pub fn update_text_tyle(&self, style: &mut TextStyle) {
        if let Some(c) = self.fg {
            let mut paint = Paint::default();
            let c: SkColor = c.into();
            paint.set_color(c);
            style.set_foreground_paint(&paint);
        }
        if let Some(c) = self.bg {
            let mut paint = Paint::default();
            let c: SkColor = c.into();
            paint.set_color(c);
            style.set_background_paint(&paint);
        }
        if let Some(c) = self.sp {
            let c: SkColor = c.into();
            style.set_decoration_color(c);
        }
        let mut font_weight = SkWeight::NORMAL;
        let mut font_slant = SkSlant::Upright;
        if let Some(v) = self.bold {
            if v {
                font_weight = SkWeight::BOLD;
            }
        }
        if let Some(v) = self.italic {
            if v {
                font_slant = SkSlant::Italic;
            }
        }
        style.set_font_style(SkFontStyle::new(
            font_weight,
            SkWidth::NORMAL,
            font_slant,
        ));
        if let Some(v) = self.undercurl {
            if v {
                style.set_decoration_style(TextDecorationStyle::Wavy);
                style.set_decoration_type(
                    style.decoration_type() | TextDecoration::UNDERLINE,
                );
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::UNDERLINE);
                style.set_decoration_type(ty);
            }
        }
        if let Some(v) = self.underline {
            if v {
                style.set_decoration_style(TextDecorationStyle::Solid);
                style.set_decoration_type(
                    style.decoration_type() | TextDecoration::UNDERLINE,
                );
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::UNDERLINE);
                style.set_decoration_type(ty);
            }
        }
        if let Some(v) = self.strikethrough {
            if v {
                style.set_decoration_style(TextDecorationStyle::Solid);
                style.set_decoration_type(
                    style.decoration_type() | TextDecoration::LINE_THROUGH,
                );
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::LINE_THROUGH);
                style.set_decoration_type(ty);
            }
        }
        if let Some(v) = self.underdashed {
            if v {
                style.set_decoration_style(TextDecorationStyle::Dashed);
                style.set_decoration_type(
                    style.decoration_type() | TextDecoration::UNDERLINE,
                );
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::UNDERLINE);
                style.set_decoration_type(ty);
            }
        }
        if let Some(v) = self.underdotted {
            if v {
                style.set_decoration_style(TextDecorationStyle::Dotted);
                style.set_decoration_type(
                    style.decoration_type() | TextDecoration::UNDERLINE,
                );
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::UNDERLINE);
                style.set_decoration_type(ty);
            }
        }
        if let Some(v) = self.underdouble {
            if v {
                style.set_decoration_style(TextDecorationStyle::Double);
                style.set_decoration_type(TextDecoration::UNDERLINE);
            } else {
                let mut ty = style.decoration_type();
                ty.remove(TextDecoration::UNDERLINE);
                style.set_decoration_type(ty);
            }
        }
        // TODO(hawtian): support blend
        // pub blend: Option<u32>,
    }
}
