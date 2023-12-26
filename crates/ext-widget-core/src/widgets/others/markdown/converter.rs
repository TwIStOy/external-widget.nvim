use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use anyhow::{bail, Context};
use comrak::nodes::{
    AstNode, NodeCode, NodeCodeBlock, NodeHeading, NodeList, NodeValue,
};
use futures::AsyncWrite;
use nvim_rs::Neovim;
use skia_safe::{
    font_style::{Slant as SkSlant, Weight as SkWeight},
    textlayout::{
        FontCollection, ParagraphBuilder, ParagraphStyle, TextDecoration,
        TextStyle,
    },
    FontStyle,
};
use tracing::{info, instrument, trace};
use tree_sitter::{Parser, Query};

use crate::{
    nvim::{HighlightInfos, NeovimSession},
    painting::{BoxBorder, BoxConstraints, BoxDecoration, FlexibleLengthAuto},
    widgets::{widget::Widget, BoxOptions, Column, Container, RichText, Row},
};

use super::codeblock::{HighlightMarker, HighlightMarkerType};

pub struct ConverterOptions {
    pub mono_font: Vec<String>,
    pub mono_font_size: f32,
    pub normal_font: Vec<String>,
    pub normal_font_size: f32,
}

pub(crate) struct Converter<'o, 'f, W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    pub(super) opts: &'o ConverterOptions,
    pub(super) font_collection: &'f FontCollection,
    pub(super) nvim: Neovim<W>,
    pub(super) session: Arc<NeovimSession>,

    pub(super) parsers: HashMap<String, RefCell<Parser>>,
    pub(super) highlight_queries: HashMap<String, Query>,
    pub(super) highlight_infos: HashMap<String, HighlightInfos>,
}

pub(super) struct BlockContext<'a> {
    block_widgets: Vec<Rc<dyn Widget>>,
    font_collection: &'a FontCollection,
    last_paragraph: ParagraphBuilder,
    font_size_scale: f32,
}

impl<'a> BlockContext<'a> {
    fn new(
        paragraph_style: &ParagraphStyle, font_collection: &'a FontCollection,
    ) -> Self {
        Self {
            block_widgets: Vec::new(),
            font_collection,
            last_paragraph: ParagraphBuilder::new(
                paragraph_style,
                font_collection,
            ),
            font_size_scale: 1.0,
        }
    }

    fn new_from_other(
        other: &mut Self, font_collection: &'a FontCollection,
    ) -> Self {
        let paragraph_style = other.last_paragraph.get_paragraph_style();
        let mut paragraph_builder =
            ParagraphBuilder::new(&paragraph_style, font_collection);
        paragraph_builder.push_style(&other.last_paragraph.peek_style());
        Self {
            block_widgets: Vec::new(),
            font_collection,
            last_paragraph: paragraph_builder,
            font_size_scale: other.font_size_scale,
        }
    }

    fn push(&mut self, w: Rc<dyn Widget>) {
        self.block_widgets.push(w);
    }

    fn push_or(&mut self, w: Option<Rc<dyn Widget>>) {
        if let Some(w) = w {
            self.push(w);
        }
    }

    fn pack_content(&mut self) -> anyhow::Result<Option<Rc<dyn Widget>>> {
        let text = self.last_paragraph.get_text().to_string();
        if text.is_empty() {
            return Ok(None);
        }
        let paragraph = self.last_paragraph.build();
        let mut new_paragraph = ParagraphBuilder::new(
            &self.last_paragraph.get_paragraph_style(),
            self.font_collection,
        );
        new_paragraph.push_style(&self.last_paragraph.peek_style());
        self.last_paragraph = new_paragraph;
        Ok(Some(Rc::new(RichText::new_with_paragraph(
            paragraph,
            Some(text),
        ))))
    }
}

impl<'o, 'f, W> Converter<'o, 'f, W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    fn update_text_tyle(&self, group: &str, style: &mut TextStyle) {
        if let Some(hl) = self.highlight_infos.get(group) {
            hl.update_text_tyle(style)
        }
    }

    fn default_paragraph_style(&self) -> ParagraphStyle {
        let mut style = ParagraphStyle::default();
        let mut text_style = style.text_style().clone();
        text_style.set_font_families(&self.opts.normal_font);
        text_style.set_font_size(self.opts.normal_font_size);
        style.set_text_style(&text_style);
        style
    }
}

/// **Inline**
impl<'o, 'f, W> Converter<'o, 'f, W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    #[instrument(level = "trace", skip_all)]
    fn visit_inline_node<'a, 'b>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()>
    where
        'f: 'b,
    {
        let data = &node.data.borrow().value;
        info!("visit_inline_node: {:?}", data);
        match data {
            NodeValue::Text(s) => self.visit_text(s, block),
            NodeValue::Emph => self.visit_emph(node, block),
            NodeValue::Strong => self.visit_strong(node, block),
            NodeValue::Strikethrough => self.visit_strike_strough(node, block),
            NodeValue::Code(c) => self.visit_code(c, block),
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                self.visit_text("\n", block)
            }
            _ => bail!("Unsupported inline node: {:?}", data),
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_simple_inline_node<'a>(
        &mut self, style: TextStyle, node: &'a AstNode<'a>,
        block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()> {
        block.last_paragraph.push_style(&style);
        for child in node.children() {
            self.visit_inline_node(child, block)?;
        }
        block.last_paragraph.pop();
        Ok(())
    }

    /// **Inline**
    #[instrument(level = "trace", skip_all)]
    fn visit_text(
        &mut self, text: &str, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        block.last_paragraph.add_text(text);
        Ok(())
    }

    /// **Inline**
    #[instrument(level = "trace", skip_all)]
    fn visit_emph<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        let font_style = style.font_style();
        let new_font_style = FontStyle::new(
            font_style.weight(),
            font_style.width(),
            SkSlant::Italic,
        );
        style.set_font_style(new_font_style);

        self.visit_simple_inline_node(style, node, block)
    }

    /// **Inline**
    #[instrument(level = "trace", skip_all)]
    fn visit_strong<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        let font_style = style.font_style();
        let new_font_style = FontStyle::new(
            SkWeight::BOLD,
            font_style.width(),
            font_style.slant(),
        );
        style.set_font_style(new_font_style);

        self.visit_simple_inline_node(style, node, block)
    }

    /// **Inline**
    #[instrument(level = "trace", skip_all)]
    fn visit_strike_strough<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        style.set_decoration_type(
            style.decoration_type() | TextDecoration::LINE_THROUGH,
        );

        self.visit_simple_inline_node(style, node, block)
    }

    /// **Inline**
    #[instrument(level = "trace", skip_all)]
    fn visit_code(
        &mut self, c: &NodeCode, block: &mut BlockContext<'f>,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        self.update_text_tyle("@text.literal.markdown_inline", &mut style);
        style.set_font_families(&self.opts.mono_font);
        style.set_font_size(self.opts.mono_font_size * block.font_size_scale);

        block.last_paragraph.push_style(&style);
        block.last_paragraph.add_text(&c.literal);
        block.last_paragraph.pop();
        Ok(())
    }
}

enum VirtialBlockWrapper {
    Column,
    Row,
}

static HEADING_FONT_SIZES: [f32; 3] = [2.0, 1.5, 1.2];

/// **Block**
impl<'c, 'f, W> Converter<'c, 'f, W>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    #[instrument(level = "trace", skip_all)]
    pub(super) fn visit_block_node<'a>(
        &mut self, node: &'a AstNode<'a>,
        block: Option<&RefCell<BlockContext<'f>>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let value = &node.data.borrow().value;
        assert!(value.block());
        trace!("{:?}", node.data.borrow().value);
        match value {
            NodeValue::Document => self.visit_block_common(
                node,
                VirtialBlockWrapper::Column,
                block,
            ),
            NodeValue::List(node_list) => {
                self.visit_list(node, node_list, block)
            }
            NodeValue::Item(node_list) => {
                self.visit_list_item(node, node_list, block)
            }
            NodeValue::CodeBlock(codeblock) => self.visit_code_block(codeblock),
            NodeValue::Paragraph => self.visit_block_common(
                node,
                VirtialBlockWrapper::Column,
                block,
            ),
            NodeValue::Heading(heading) => self.visit_heading(node, heading),
            NodeValue::ThematicBreak => {
                trace!("visit_block_node: ThematicBreak");
                let hl = self.highlight_infos.get("Normal");
                Ok(Rc::new(Container::new(
                    BoxDecoration {
                        color: hl.and_then(|x| x.fg).unwrap_or_default(),
                        border: BoxBorder::NONE,
                    },
                    BoxOptions {
                        constraints: BoxConstraints {
                            min_height: FlexibleLengthAuto::Fixed(2.0),
                            max_height: FlexibleLengthAuto::Fixed(2.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )))
            }
            _ => bail!("Unsupported block node: {:?}", value),
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_code_block(
        &mut self, codeblock: &NodeCodeBlock,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block =
            BlockContext::new(&Default::default(), self.font_collection);
        let mut style = block.last_paragraph.peek_style();
        style.set_font_families(&self.opts.mono_font);
        style.set_font_size(self.opts.mono_font_size);
        block.last_paragraph.push_style(&style);
        let code = &codeblock.literal;
        let mut highlights =
            get_all_captures(code, &codeblock.info, &self).unwrap_or_default();
        highlights.sort();

        let mut m = 0usize;
        let mut offset = 0usize;
        for ch in code.chars() {
            while m < highlights.len() && highlights[m].offset <= offset {
                let marker = &highlights[m];
                match marker.kind {
                    HighlightMarkerType::Start => {
                        let mut style = block.last_paragraph.peek_style();
                        self.update_text_tyle(&marker.group, &mut style);
                        block.last_paragraph.push_style(&style);
                    }
                    HighlightMarkerType::End => {
                        block.last_paragraph.pop();
                    }
                }
                m += 1;
            }
            block.last_paragraph.add_text(&ch.to_string());
            offset += ch.len_utf8();
        }
        while m < highlights.len() {
            let marker = &highlights[m];
            match marker.kind {
                HighlightMarkerType::Start => {
                    let mut style = block.last_paragraph.peek_style();
                    self.update_text_tyle(&marker.group, &mut style);
                    block.last_paragraph.push_style(&style);
                }
                HighlightMarkerType::End => {
                    block.last_paragraph.pop();
                }
            }
            m += 1;
        }

        Ok(block.pack_content()?.unwrap())
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_heading<'a>(
        &mut self, node: &'a AstNode<'a>, heading: &NodeHeading,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut scale = 1.0;
        if heading.level <= 3 {
            scale = HEADING_FONT_SIZES[(heading.level as usize) - 1];
        }

        let mut block =
            BlockContext::new(&Default::default(), self.font_collection);
        let mut style = block.last_paragraph.peek_style();
        style.set_font_families(&self.opts.normal_font);
        style.set_font_size(self.opts.normal_font_size * scale);
        block.font_size_scale = scale;
        block.last_paragraph.push_style(&style);
        for child in node.children() {
            assert!(!child.data.borrow().value.block());
            self.visit_inline_node(child, &mut block)?;
        }
        Ok(block.pack_content()?.unwrap())
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_list<'a>(
        &mut self, node: &'a AstNode<'a>, _node_list: &NodeList,
        block: Option<&RefCell<BlockContext<'f>>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut lines: Vec<Rc<dyn Widget>> = Vec::new();
        for c in node.children() {
            let w = self.visit_block_node(c, block)?;
            lines.push(w);
        }
        Ok(Rc::new(Column::new_with_children(lines)))
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_list_item<'a>(
        &mut self, node: &'a AstNode<'a>, node_list: &NodeList,
        block: Option<&RefCell<BlockContext<'f>>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block = match block {
            Some(block) => BlockContext::new_from_other(
                &mut block.borrow_mut(),
                self.font_collection,
            ),
            None => {
                BlockContext::new(&Default::default(), self.font_collection)
            }
        };
        let marker = match node_list.list_type {
            comrak::nodes::ListType::Bullet => {
                format!("{} ", node_list.bullet_char as char)
            }
            comrak::nodes::ListType::Ordered => {
                let start = node_list.start;
                let offset = node_list.marker_offset;
                let delimitor = match node_list.delimiter {
                    comrak::nodes::ListDelimType::Period => ". ",
                    comrak::nodes::ListDelimType::Paren => ") ",
                };
                format!("{}{}", start + offset, delimitor)
            }
        };
        block.last_paragraph.add_text(&marker);
        let marker = block.pack_content()?.unwrap();
        let block = RefCell::new(block);
        for c in node.children() {
            if c.data.borrow().value.block() {
                let w = self.visit_block_node(c, Some(&block))?;
                block.borrow_mut().push(w);
            } else {
                self.visit_inline_node(c, &mut block.borrow_mut())?;
                let content = block.borrow_mut().pack_content()?;
                block.borrow_mut().push_or(content);
            }
        }
        let block = block.into_inner();
        let content = Column::new_with_children(block.block_widgets);
        Ok(Rc::new(Row::new_with_children(vec![
            marker,
            Rc::new(content),
        ])))
    }

    #[instrument(level = "trace", skip_all)]
    fn visit_block_common<'a>(
        &mut self, node: &'a AstNode<'a>, wrapper: VirtialBlockWrapper,
        block: Option<&RefCell<BlockContext<'f>>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        trace!("{:?}", node.data.borrow().value);
        let block = match block {
            Some(block) => BlockContext::<'f>::new_from_other(
                &mut block.borrow_mut(),
                self.font_collection,
            ),
            None => BlockContext::new(
                &self.default_paragraph_style(),
                self.font_collection,
            ),
        };
        let block = RefCell::new(block);
        // visit childrens
        for child in node.children() {
            if child.data.borrow().value.block() {
                let content = block.borrow_mut().pack_content()?;
                block.borrow_mut().push_or(content);

                let w = self.visit_block_node(child, Some(&block))?;
                block.borrow_mut().push(w);
            } else {
                self.visit_inline_node(child, &mut block.borrow_mut())?;
            }
        }
        let content = block.borrow_mut().pack_content()?;
        block.borrow_mut().push_or(content);

        let block = block.into_inner();
        let ret: Rc<dyn Widget> = match wrapper {
            VirtialBlockWrapper::Column => {
                Rc::new(Column::new_with_children(block.block_widgets))
            }
            VirtialBlockWrapper::Row => {
                Rc::new(Row::new_with_children(block.block_widgets))
            }
        };
        Ok(ret)
    }
}

fn get_all_captures<'o, 'f, W>(
    code: &str, lang: &str, converter: &Converter<'o, 'f, W>,
) -> anyhow::Result<Vec<HighlightMarker>>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let mut parser = converter
        .parsers
        .get(lang)
        .context("No parser")?
        .borrow_mut();
    let query = converter
        .highlight_queries
        .get(lang)
        .context("No highlight query")?;

    let tree = parser.parse(code, None).context("Parse tree failed")?;
    let mut cursor = tree_sitter::QueryCursor::new();

    let all_captures =
        cursor.captures(&query, tree.root_node(), code.as_bytes());

    let mut ret = vec![];
    for (m, _) in all_captures {
        for (_, capture) in m.captures.iter().enumerate() {
            let start_byte = capture.node.start_byte();
            let end_byte = capture.node.end_byte();
            if start_byte < end_byte {
                ret.push(HighlightMarker {
                    group: query.capture_names()[capture.index as usize]
                        .to_string(),
                    kind: HighlightMarkerType::Start,
                    offset: start_byte,
                });
                ret.push(HighlightMarker {
                    group: query.capture_names()[capture.index as usize]
                        .to_string(),
                    offset: end_byte,
                    kind: HighlightMarkerType::End,
                })
            }
        }
    }
    Ok(ret)
}
