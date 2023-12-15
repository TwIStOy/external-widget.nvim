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

pub async fn hl_props_from_group<W>(
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
