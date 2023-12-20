use std::{collections::HashMap, rc::Rc};

use external_widget_core::{
    nvim::{hl_props_from_group, HighlightDefinition, Nvim},
    treesitter::TREE_SITTER,
    Color,
};
use external_widget_widgets::{
    render_widget_tree_to_buf, Container, ContainerBorderStyle, MdDoc,
    MdDocOpts,
};
use tracing::info;

const BUILTIN_GROUPS: [&str; 3] = ["Normal", "NormalNC", "NormalFloat"];

#[tracing::instrument(skip(nvim))]
async fn prepare_highlights(
    lang: &str, nvim: &Nvim,
) -> anyhow::Result<HashMap<String, HighlightDefinition>> {
    let mut highlights = TREE_SITTER
        .lock()
        .unwrap()
        .get_query("markdown", "highlights")
        .map(|x| x.capture_names().to_vec())
        .unwrap_or_default();
    if lang != "markdown" {
        let lang_hl = TREE_SITTER
            .lock()
            .unwrap()
            .get_query(lang, "highlights")
            .map(|x| x.capture_names().to_vec())
            .unwrap_or_default();
        highlights.extend(lang_hl.into_iter());
    }
    highlights.extend(BUILTIN_GROUPS.iter().map(|x| x.to_string()));

    info!("highlights: {:?}", highlights);

    let mut ret = HashMap::new();

    for name in highlights {
        let hl = hl_props_from_group(name.clone(), nvim).await;
        if let Ok(hl) = hl {
            ret.insert(name, hl);
        }
    }
    Ok(ret)
}

#[tracing::instrument(skip(nvim, md))]
pub async fn build_hover_doc_image(
    nvim: &Nvim, md: String, lang: &str,
) -> anyhow::Result<Vec<u8>> {
    let highlights = prepare_highlights(lang, nvim).await.unwrap_or_default();
    let background = highlights
        .get("Normal")
        .map(|x| x.bg)
        .or_else(|| highlights.get("NormalFloat").map(|x| x.bg))
        .flatten()
        .map(Color::from_u32);
    let opts = MdDocOpts {
        md,
        highlights,
        normal_font: "MonoLisa".to_string(),
        mono_font: "MonoLisa".to_string(),
        font_size: "14pt".to_string(),
    };
    let doc = Rc::new(MdDoc::new(opts)?);
    let mut root = Container::new(doc);
    root.background = background;
    root.constraints.padding = {
        use taffy::{LengthPercentage, Rect};
        Rect {
            left: LengthPercentage::Length(10.),
            right: LengthPercentage::Length(10.),
            top: LengthPercentage::Length(10.),
            bottom: LengthPercentage::Length(10.),
        }
    };
    root.border = Some(ContainerBorderStyle {
        color: Color::from_u32(0x000000),
        width: 1.0,
    });
    let widget = Rc::new(root);
    let buffer = render_widget_tree_to_buf(widget, 1000, 1000)?;
    Ok(buffer)
}
