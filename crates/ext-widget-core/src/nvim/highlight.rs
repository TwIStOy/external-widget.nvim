use skia_safe::font_style::{
    Slant as SkSlant, Weight as SkWeight, Width as SkWidth,
};
use skia_safe::textlayout::{TextDecoration, TextDecorationStyle};
use skia_safe::Color as SkColor;
use skia_safe::{textlayout::TextStyle, FontStyle as SkFontStyle, Paint};

use crate::painting::Color;

pub fn nvim_highlight_into_text_style(
    hl: &HighlightInfos, ret: &mut TextStyle,
) {
    if let Some(c) = hl.foreground {
        let mut paint = Paint::default();
        let c: SkColor = Color::new(c).into();
        paint.set_color(c);
        ret.set_foreground_paint(&paint);
    }
    if let Some(c) = hl.background {
        let mut paint = Paint::default();
        let c: SkColor = Color::new(c).into();
        paint.set_color(c);
        ret.set_background_paint(&paint);
    }
    if let Some(c) = hl.special {
        let c: SkColor = Color::new(c).into();
        ret.set_decoration_color(c);
    }
    let mut font_weight = SkWeight::NORMAL;
    let mut font_slant = SkSlant::Upright;
    if let Some(v) = hl.bold {
        if v {
            font_weight = SkWeight::BOLD;
        }
    }
    if let Some(v) = hl.italic {
        if v {
            font_slant = SkSlant::Italic;
        }
    }
    ret.set_font_style(SkFontStyle::new(
        font_weight,
        SkWidth::NORMAL,
        font_slant,
    ));
    if let Some(v) = hl.undercurl {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Wavy);
            ret.set_decoration_type(
                ret.decoration_type() | TextDecoration::UNDERLINE,
            );
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::UNDERLINE);
            ret.set_decoration_type(ty);
        }
    }
    if let Some(v) = hl.underline {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Solid);
            ret.set_decoration_type(
                ret.decoration_type() | TextDecoration::UNDERLINE,
            );
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::UNDERLINE);
            ret.set_decoration_type(ty);
        }
    }
    if let Some(v) = hl.strikethrough {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Solid);
            ret.set_decoration_type(
                ret.decoration_type() | TextDecoration::LINE_THROUGH,
            );
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::LINE_THROUGH);
            ret.set_decoration_type(ty);
        }
    }
    if let Some(v) = hl.underdash {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Dashed);
            ret.set_decoration_type(
                ret.decoration_type() | TextDecoration::UNDERLINE,
            );
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::UNDERLINE);
            ret.set_decoration_type(ty);
        }
    }
    if let Some(v) = hl.underdot {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Dotted);
            ret.set_decoration_type(
                ret.decoration_type() | TextDecoration::UNDERLINE,
            );
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::UNDERLINE);
            ret.set_decoration_type(ty);
        }
    }
    if let Some(v) = hl.underlineline {
        if v {
            ret.set_decoration_style(TextDecorationStyle::Double);
            ret.set_decoration_type(TextDecoration::UNDERLINE);
        } else {
            let mut ty = ret.decoration_type();
            ty.remove(TextDecoration::UNDERLINE);
            ret.set_decoration_type(ty);
        }
    }
    // TODO(hawtian): support blend
    // pub blend: Option<u32>,
}
