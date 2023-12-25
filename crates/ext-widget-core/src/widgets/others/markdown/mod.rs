mod codeblock;
mod converter;
mod markdown_document;

use std::{rc::Rc, sync::Arc};

pub use converter::ConverterOptions;
use futures::AsyncWrite;
use nvim_rs::Neovim;
use skia_safe::{textlayout::FontCollection, FontMgr};

use crate::{nvim::NeovimSession, widgets::widget::Widget};

use converter::Converter;

pub use markdown_document::MarkdownDocumentBuilder;

/**
 * Render markdown text into a widget.
 */
pub fn render_markdown<W>(
    nvim: &Neovim<W>, session: Arc<NeovimSession>, text: &str,
    options: &ConverterOptions,
) -> anyhow::Result<Rc<dyn Widget>>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::new(), None);
    let mut converter = Converter {
        nvim: nvim.clone(),
        session: session.clone(),
        opts: options,
        font_collection: &font_collection,
    };
    let arena = comrak::Arena::new();
    let root =
        comrak::parse_document(&arena, text, &comrak::ComrakOptions::default());
    converter.visit_block_node(root, None)
}
