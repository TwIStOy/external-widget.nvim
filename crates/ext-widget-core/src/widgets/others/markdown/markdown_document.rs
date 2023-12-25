use std::{rc::Rc, sync::Arc};

use futures::AsyncWrite;
use nvim_rs::Neovim;
use skia_safe::{textlayout::FontCollection, FontMgr};

use crate::{nvim::NeovimSession, widgets::Widget};

use super::{converter::Converter, ConverterOptions};

pub struct MarkdownDocumentBuilder<W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    pub nvim: Neovim<W>,
    pub session: Arc<NeovimSession>,
    pub normal_font: Vec<String>,
    pub normal_font_size: f32,
    pub mono_font: Vec<String>,
    pub mono_font_size: f32,
}

impl<W> MarkdownDocumentBuilder<W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    pub fn build(self, text: &str) -> anyhow::Result<Rc<dyn Widget>> {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let opts = ConverterOptions {
            normal_font: self.normal_font,
            normal_font_size: self.normal_font_size,
            mono_font: self.mono_font,
            mono_font_size: self.mono_font_size,
        };
        let mut converter = Converter {
            nvim: self.nvim.clone(),
            session: self.session.clone(),
            opts: &opts,
            font_collection: &font_collection,
        };
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            text,
            &comrak::ComrakOptions::default(),
        );
        converter.visit_block_node(root, None)
    }
}
