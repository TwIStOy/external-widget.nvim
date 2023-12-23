use skia_safe::{
    font_style::{Slant as SkSlant, Weight as SkWeight, Width as SkWidth},
    textlayout::{TextDecoration, TextDecorationStyle, TextStyle},
    Color as SkColor, FontStyle as SkFontStyle, Paint,
};

use crate::painting::Color;

#[derive(Debug, Clone)]
pub(super) struct StyleFieldStack<T> {
    inner: Vec<T>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FontFeatureStack {
    inner: Vec<Vec<(String, i32)>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct StyleStack {
    pub foreground: StyleFieldStack<u32>,
    pub background: StyleFieldStack<u32>,
    pub special: StyleFieldStack<u32>,
    pub font_weight: StyleFieldStack<SkWeight>,
    pub font_width: StyleFieldStack<SkWidth>,
    pub font_slant: StyleFieldStack<SkSlant>,
    pub bold: StyleFieldStack<bool>,
    pub italic: StyleFieldStack<bool>,
    pub decoration_style: StyleFieldStack<TextDecorationStyle>,
    pub decoration_type: StyleFieldStack<TextDecoration>,
    pub font_family: StyleFieldStack<Vec<String>>,
    pub font_size: StyleFieldStack<f32>,
    pub font_features: FontFeatureStack,
    inner: StyleFieldStack<StyleStackFields>,
}

#[derive(Debug, Clone)]
enum StyleStackField {
    Foreground,
    Background,
    Special,
    FontWeight,
    FontWidth,
    FontSlant,
    Bold,
    Italic,
    DecorationStyle,
    DecorationType,
    FontFamily,
    FontSize,
    FontFeatures,
}

type StyleStackFields = Vec<StyleStackField>;

impl StyleStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, hl: &HighlightInfos) {
        let mut fields = StyleStackFields::new();
        if let Some(c) = hl.foreground {
            self.foreground.push(c);
            fields.push(StyleStackField::Foreground);
        }
        if let Some(c) = hl.background {
            self.background.push(c);
            fields.push(StyleStackField::Background);
        }
        if let Some(c) = hl.special {
            self.special.push(c);
            fields.push(StyleStackField::Special);
        }
        if let Some(v) = hl.bold {
            if v {
                self.font_weight.push(SkWeight::BOLD);
                fields.push(StyleStackField::FontWeight);
            }
        }
        if let Some(v) = hl.italic {
            if v {
                self.italic.push(true);
                fields.push(StyleStackField::Italic);
            }
        }
        let mut decoration_style = TextDecorationStyle::Solid;
        let mut decoration_type = TextDecoration::NO_DECORATION;
        if let Some(v) = hl.underline {
            if v {
                decoration_style = TextDecorationStyle::Solid;
                decoration_type |= TextDecoration::UNDERLINE;
            }
        }
        if let Some(v) = hl.undercurl {
            if v {
                decoration_style = TextDecorationStyle::Wavy;
                decoration_type |= TextDecoration::UNDERLINE;
            }
        }
        if let Some(v) = hl.strikethrough {
            if v {
                decoration_style = TextDecorationStyle::Solid;
                decoration_type |= TextDecoration::LINE_THROUGH;
            }
        }
        if let Some(v) = hl.underdash {
            if v {
                decoration_style = TextDecorationStyle::Dashed;
                decoration_type |= TextDecoration::UNDERLINE;
            }
        }
        if let Some(v) = hl.underdot {
            if v {
                decoration_style = TextDecorationStyle::Dotted;
                decoration_type |= TextDecoration::UNDERLINE;
            }
        }
        if let Some(v) = hl.underlineline {
            if v {
                decoration_style = TextDecorationStyle::Double;
                decoration_type |= TextDecoration::UNDERLINE;
            }
        }
        self.decoration_style.push(decoration_style);
        self.decoration_type.push(decoration_type);
        fields.push(StyleStackField::DecorationStyle);
        fields.push(StyleStackField::DecorationType);
        // TODO(hawtian): support blend
        // pub blend: Option<u32>,
        self.inner.push(fields);
    }

    pub fn push_font_family(&mut self, font_family: Vec<String>) {
        self.font_family.push(font_family);
        self.inner.push(vec![StyleStackField::FontFamily]);
    }

    pub fn push_font_features(&mut self, font_features: Vec<(String, i32)>) {
        self.font_features.push(font_features);
        self.inner.push(vec![StyleStackField::FontFeatures]);
    }

    pub fn pop(&mut self) {
        let fields = self.inner.pop();
        if fields.is_none() {
            return;
        }
        let fields = fields.unwrap();
        for field in fields {
            match field {
                StyleStackField::Foreground => {
                    self.foreground.pop();
                }
                StyleStackField::Background => {
                    self.background.pop();
                }
                StyleStackField::Special => {
                    self.special.pop();
                }
                StyleStackField::FontWeight => {
                    self.font_weight.pop();
                }
                StyleStackField::FontWidth => {
                    self.font_width.pop();
                }
                StyleStackField::FontSlant => {
                    self.font_slant.pop();
                }
                StyleStackField::Bold => {
                    self.bold.pop();
                }
                StyleStackField::Italic => {
                    self.italic.pop();
                }
                StyleStackField::DecorationStyle => {
                    self.decoration_style.pop();
                }
                StyleStackField::DecorationType => {
                    self.decoration_type.pop();
                }
                StyleStackField::FontFamily => {
                    self.font_family.pop();
                }
                StyleStackField::FontSize => {
                    self.font_size.pop();
                }
                StyleStackField::FontFeatures => {
                    self.font_features.pop();
                }
            }
        }
    }

    pub fn top(&self) -> TextStyle {
        let mut ret = TextStyle::new();
        if let Some(c) = self.foreground.top() {
            let mut paint = Paint::default();
            let c: SkColor = Color::new(*c).into();
            paint.set_color(c);
            ret.set_foreground_paint(&paint);
        }
        if let Some(c) = self.background.top() {
            let mut paint = Paint::default();
            let c: SkColor = Color::new(*c).into();
            paint.set_color(c);
            ret.set_background_paint(&paint);
        }
        if let Some(c) = self.special.top() {
            let c: SkColor = Color::new(*c).into();
            ret.set_decoration_color(c);
        }
        let font_weight = if let Some(v) = self.bold.top() {
            if *v {
                SkWeight::BOLD
            } else {
                SkWeight::NORMAL
            }
        } else {
            SkWeight::NORMAL
        };
        let font_width: SkWidth =
            self.font_width.top().copied().unwrap_or(SkWidth::NORMAL);
        let font_slant = if let Some(v) = self.italic.top() {
            if *v {
                SkSlant::Italic
            } else {
                SkSlant::Upright
            }
        } else {
            SkSlant::Upright
        };
        ret.set_font_style(SkFontStyle::new(
            font_weight,
            font_width,
            font_slant,
        ));
        ret.set_decoration_style(
            self.decoration_style
                .top()
                .copied()
                .unwrap_or(TextDecorationStyle::default()),
        );
        ret.set_decoration_type(
            self.decoration_type
                .top()
                .copied()
                .unwrap_or(TextDecoration::default()),
        );
        if let Some(font_family) = self.font_family.top() {
            ret.set_font_families(&font_family);
        }
        if let Some(font_size) = self.font_size.top() {
            ret.set_font_size((*font_size).into());
        }
        for (k, v) in self.font_features.top() {
            ret.add_font_feature(k, v);
        }
        ret
    }
}

impl<T> StyleFieldStack<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn top(&self) -> Option<&T> {
        self.inner.last()
    }
}

impl FontFeatureStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, value: Vec<(String, i32)>) {
        self.inner.push(value);
    }

    pub fn pop(&mut self) {
        self.inner.pop();
    }

    pub fn top(&self) -> Vec<(String, i32)> {
        let mut ret = Vec::new();
        for v in self.inner.iter().rev() {
            ret.extend(v.iter().cloned());
        }
        ret
    }
}

impl<T> Default for StyleFieldStack<T> {
    fn default() -> Self {
        Self::new()
    }
}
