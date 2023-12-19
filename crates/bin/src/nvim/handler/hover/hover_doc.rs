use std::{collections::HashMap, rc::Rc};

use external_widget_core::{
    nvim::{hl_props_from_group, HighlightDefinition, Nvim},
    treesitter::TREE_SITTER,
};
use external_widget_widgets::{render_widget_tree_to_buf, MdDoc, MdDocOpts};

const BUILTIN_GROUPS: [&str; 1] = ["Normal"];

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

    let mut ret = HashMap::new();

    for name in highlights {
        let hl = hl_props_from_group(name.clone(), nvim).await;
        if let Ok(hl) = hl {
            ret.insert(name, hl);
        }
    }
    Ok(ret)
}

pub async fn build_hover_doc_image(
    nvim: &Nvim, md: String, lang: &str,
) -> anyhow::Result<Vec<u8>> {
    let highlights = prepare_highlights(lang, nvim).await.unwrap_or_default();
    let opts = MdDocOpts {
        md,
        highlights,
        normal_font: "MonoLisa".to_string(),
        mono_font: "MonoLisa".to_string(),
        font_size: "14pt".to_string(),
    };
    let widget = Rc::new(MdDoc::new(opts)?);
    let buffer = render_widget_tree_to_buf(widget, 1000, 1000)?;
    Ok(buffer)
}
