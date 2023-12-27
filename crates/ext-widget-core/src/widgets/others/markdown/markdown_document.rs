use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use comrak::nodes::AstNode;
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

const DEFAULT_HIGHLIGHS: &[&str] = &["Normal", "NormalNC"];

impl<W> MarkdownDocumentBuilder<W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    pub async fn build(self, text: &str) -> anyhow::Result<Rc<dyn Widget>> {
        let langs = {
            let arena = comrak::Arena::new();
            let root = comrak::parse_document(
                &arena,
                text,
                &comrak::ComrakOptions::default(),
            );
            get_codeblock_infos(root)
        };
        let mut parsers = HashMap::new();
        let mut quries = HashMap::new();
        let mut highlights = HashMap::new();
        for name in langs {
            let info = self.session.load_ts_parser(&self.nvim, &name).await;
            if let Ok(Some(info)) = info {
                parsers.insert(name.clone(), RefCell::new(info));
            }
            let query = self
                .session
                .load_ts_query(&self.nvim, &name, "highlights")
                .await;
            if let Ok(query) = query {
                quries.insert(name.clone(), query);
            }
        }
        for (_, query) in &quries {
            let groups = query.capture_names();
            for group in groups {
                let hl =
                    self.session.get_highlight_info(&self.nvim, &group).await;
                if let Ok(hl) = hl {
                    highlights.insert(group.to_string(), hl);
                }
            }
        }
        for name in DEFAULT_HIGHLIGHS {
            let hl = self.session.get_highlight_info(&self.nvim, &name).await;
            if let Ok(hl) = hl {
                highlights.insert(name.to_string(), hl);
            }
        }

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
            parsers,
            highlight_queries: quries,
            highlight_infos: highlights,
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

fn visit_node_children<'a>(node: &'a AstNode<'a>, infos: &mut Vec<String>) {
    let children = node.children();
    for child in children {
        visit_node(child, infos);
    }
}

fn visit_node<'a>(node: &'a AstNode<'a>, infos: &mut Vec<String>) {
    let data = &node.data.borrow().value;
    match data {
        comrak::nodes::NodeValue::CodeBlock(codeblock) => {
            infos.push(codeblock.info.clone());
        }
        _ => visit_node_children(node, infos),
    }
}

fn get_codeblock_infos<'a>(root: &'a AstNode<'a>) -> Vec<String> {
    let mut infos = Vec::new();
    visit_node_children(root, &mut infos);
    infos
}
