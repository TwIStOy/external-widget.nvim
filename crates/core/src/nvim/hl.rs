use crate::{color::Color, pango::MarkupProperties};
use futures::AsyncWrite;
use nvim_rs::Neovim;
use rmpv::ext::from_value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighlightDefinition {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub sp: Option<u32>,
    pub guifg: Option<u32>,
    pub guibg: Option<u32>,
    pub guisp: Option<u32>,
    pub blend: Option<u32>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub undercurl: bool,
    #[serde(default)]
    pub underdouble: bool,
    #[serde(default)]
    pub underdotted: bool,
    #[serde(default)]
    pub underdashed: bool,
    #[serde(default)]
    pub strikethrough: bool,
    pub link: Option<String>,
}

impl From<&HighlightDefinition> for MarkupProperties {
    fn from(value: &HighlightDefinition) -> Self {
        let mut ret = Self::new();
        if let Some(v) = value.fg {
            ret.insert("foreground".into(), Color::from_u32(v).to_string());
        }
        if let Some(v) = value.bg {
            ret.insert("background".into(), Color::from_u32(v).to_string());
        }
        if let Some(v) = value.sp {
            ret.insert(
                "underline_color".into(),
                Color::from_u32(v).to_string(),
            );
        }
        if let Some(v) = value.guifg {
            ret.insert("foreground".into(), Color::from_u32(v).to_string());
        }
        if let Some(v) = value.guibg {
            ret.insert("background".into(), Color::from_u32(v).to_string());
        }
        if let Some(v) = value.guisp {
            ret.insert(
                "underline_color".into(),
                Color::from_u32(v).to_string(),
            );
        }
        if let Some(v) = value.blend {
            ret.insert(
                "alpha".into(),
                format!("{}%", (v as f32 / 100.0).ceil() as u32),
            );
        }
        if value.bold {
            ret.insert("weight".into(), "bold".into());
        }
        if value.underline {
            ret.insert("underline".into(), "single".into());
        }
        if value.undercurl {
            ret.insert("underline".into(), "error".into());
        }
        if value.underdouble {
            ret.insert("underline".into(), "double".into());
        }
        if value.underdotted {
            ret.insert("underline".into(), "single".into());
        }
        if value.underdashed {
            ret.insert("underline".into(), "single".into());
        }
        if value.strikethrough {
            ret.insert("strikethrough".into(), "true".into());
        }
        ret
    }
}

impl From<HighlightDefinition> for MarkupProperties {
    fn from(value: HighlightDefinition) -> Self {
        Self::from(&value)
    }
}

async fn hl_props_from_group_impl<W>(
    group: &str, nvim: &Neovim<W>,
) -> anyhow::Result<HighlightDefinition>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let res = nvim.get_hl(0, vec![("name".into(), group.into())]).await?;
    let res = rmpv::Value::Map(res);
    let def: HighlightDefinition = from_value(res)?;
    Ok(def)
}

pub async fn hl_props_from_group<W>(
    group: String, nvim: &Neovim<W>,
) -> anyhow::Result<HighlightDefinition>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let mut name = group;
    loop {
        let res = hl_props_from_group_impl(&name, nvim).await?;
        if let Some(link) = res.link {
            name = link;
        } else {
            return Ok(res);
        }
    }
}

impl HighlightDefinition {
    pub async fn new_from_group<W>(
        group: String, nvim: &Neovim<W>,
    ) -> anyhow::Result<Self>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        hl_props_from_group(group, nvim).await
    }
}
