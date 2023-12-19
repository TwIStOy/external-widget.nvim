use std::collections::HashMap;
use std::rc::Rc;

use anyhow::bail;
use comrak::nodes::{
    AstNode, NodeCode, NodeCodeBlock, NodeHeading, NodeList, NodeValue,
};
use external_widget_core::{
    nvim::HighlightDefinition,
    pango::{MarkupProperties, MarkupSpan, MarkupSpanStack},
    Widget,
};
use glib::markup_escape_text;
use taffy::LengthPercentage;

use crate::{Bar, Column, MarkupParagraph, Row};

use super::{
    codeblock::{get_all_captures, HighlightMarkerType},
    MdDocOpts,
};

pub(crate) struct Converter<'a> {
    pending_markup_line: String,
    stack: MarkupSpanStack,
    opts: &'a MdDocOpts,
}

struct BlockContext {
    block_widgets: Vec<Rc<dyn Widget>>,
}

impl BlockContext {
    fn new() -> Self {
        Self {
            block_widgets: Vec::new(),
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
}

impl<'c> Converter<'c> {
    pub(crate) fn new(opts: &'c MdDocOpts) -> Self {
        let mut ret = Self {
            pending_markup_line: String::new(),
            stack: MarkupSpanStack::new(),
            opts,
        };
        let mut props = MarkupProperties::new();
        props.insert("font".into(), opts.normal_font.clone());
        props.insert("size".into(), opts.font_size.to_string());
        if let Some(hl) = opts.highlights.get("Normal") {
            let p: MarkupProperties = hl.into();
            props.0.extend(p.0);
        }
        ret.push_span(MarkupSpan::new_with_properties(props))
            .unwrap();
        ret
    }

    pub(crate) fn visit_node<'a>(
        &mut self, node: &'a AstNode<'a>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let value = &node.data.borrow().value;
        if value.block() {
            self.visit_block_node(node)
        } else {
            bail!("Unsupported inline node: {:?}", value)
        }
    }

    fn push_span(&mut self, span: MarkupSpan) -> anyhow::Result<()> {
        span.to_markup_open(&mut self.pending_markup_line)?;
        self.stack.push(span);
        Ok(())
    }

    fn pop_span(&mut self) -> anyhow::Result<()> {
        let span = self.stack.pop();
        if let Some(span) = span {
            span.to_markup_close(&mut self.pending_markup_line);
        }
        Ok(())
    }
}

/// **Inline**
impl<'c> Converter<'c> {
    fn pack_markup_line(&mut self) -> anyhow::Result<Option<Rc<dyn Widget>>> {
        let markup_start = self.stack.to_markup_open_owned()?;
        if self.pending_markup_line.is_empty()
            || self.pending_markup_line == markup_start
        {
            return Ok(None);
        }
        self.stack.to_markup_close(&mut self.pending_markup_line);
        let mut ret = markup_start;
        std::mem::swap(&mut self.pending_markup_line, &mut ret);
        Ok(Some(Rc::new(MarkupParagraph::new(ret))))
    }

    fn visit_inline_node<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let data = &node.data.borrow().value;
        match data {
            NodeValue::Text(s) => self.visit_text(s),
            NodeValue::Emph => self.visit_emph(node, block),
            NodeValue::Strong => self.visit_strong(node, block),
            NodeValue::Strikethrough => self.visit_strike_strough(node, block),
            NodeValue::Code(c) => self.visit_code(c),
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                let w = self.pack_markup_line()?;
                block.push_or(w);
                Ok(())
            }
            _ => bail!("Unsupported inline node: {:?}", data),
        }
    }

    fn visit_simple_inline_node<'a>(
        &mut self, props: MarkupProperties, node: &'a AstNode<'a>,
        block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let span = MarkupSpan::new_with_properties(props);
        self.push_span(span)?;
        node.children()
            .try_for_each(|x| self.visit_inline_node(x, block))?;
        self.pop_span()?;
        Ok(())
    }

    /// **Inline**
    fn visit_text(&mut self, text: &str) -> anyhow::Result<()> {
        let escaped = glib::markup_escape_text(text).to_string();
        self.pending_markup_line.push_str(&escaped);
        Ok(())
    }

    /// **Inline**
    fn visit_emph<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut props = MarkupProperties::new();
        props.insert("style".into(), "italic".into());
        self.visit_simple_inline_node(props, node, block)
    }

    /// **Inline**
    fn visit_strong<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut props = MarkupProperties::new();
        props.insert("weight".into(), "bold".into());
        self.visit_simple_inline_node(props, node, block)
    }

    /// **Inline**
    fn visit_strike_strough<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut props = MarkupProperties::new();
        props.insert("strikethrough".into(), "true".into());
        self.visit_simple_inline_node(props, node, block)
    }

    /// **Inline**
    fn visit_code(&mut self, c: &NodeCode) -> anyhow::Result<()> {
        let mut props = MarkupProperties::new();
        props.insert("font".into(), self.opts.mono_font.clone());
        let span = MarkupSpan::new_with_properties(props);
        let code = glib::markup_escape_text(&c.literal).to_string();
        self.pending_markup_line
            .push_str(&span.wrap_text_owned(code)?);
        Ok(())
    }
}

enum VirtialBlockWrapper {
    Column,
    Row,
}

static HEADING_FONT_SIZES: [&str; 3] = ["200%", "150%", "120%"];

/// **Block**
impl<'c> Converter<'c> {
    fn visit_block_node<'a>(
        &mut self, node: &'a AstNode<'a>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let value = &node.data.borrow().value;
        assert!(value.block());
        match value {
            NodeValue::Document => self.visit_virtual_block(
                node,
                VirtialBlockWrapper::Column,
                None,
            ),
            NodeValue::FrontMatter(_) => todo!(),
            NodeValue::BlockQuote => todo!(),
            NodeValue::List(node_list) => self.visit_list(node, node_list),
            NodeValue::Item(node_list) => self.visit_list_item(node, node_list),
            NodeValue::CodeBlock(codeblock) => self.visit_code_block(codeblock),
            NodeValue::Paragraph => self.visit_virtual_block(
                node,
                VirtialBlockWrapper::Column,
                None,
            ),
            NodeValue::Heading(heading) => self.visit_heading(node, heading),
            NodeValue::ThematicBreak => {
                Ok(Rc::new(Bar::new().height(LengthPercentage::Length(2.))))
            }
            _ => bail!("Unsupported block node: {:?}", value),
        }
    }

    fn visit_code_block(
        &mut self, codeblock: &NodeCodeBlock,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block = BlockContext::new();
        let mut props = MarkupProperties::new();
        props.insert("font".into(), self.opts.mono_font.clone());
        let span = MarkupSpan::new_with_properties(props);
        self.push_span(span)?;

        let code = codeblock.literal.trim();
        let mut highlights = get_all_captures(code, &codeblock.info).unwrap();
        highlights.sort();

        let mut m = 0usize;
        let mut offset = 0usize;
        for ch in code.chars() {
            while m < highlights.len() && highlights[m].offset <= offset {
                let marker = &highlights[m];
                match marker.kind {
                    HighlightMarkerType::Start => {
                        let hl_def = self.opts.highlights.get(&marker.group);
                        let props: MarkupProperties =
                            if let Some(hl_def) = hl_def {
                                hl_def.clone().into()
                            } else {
                                MarkupProperties::new()
                            };
                        self.push_span(MarkupSpan::new_with_properties(props))?;
                    }
                    HighlightMarkerType::End => {
                        self.pop_span()?;
                    }
                }
                m += 1;
            }

            match ch {
                '\n' | '\r' => {
                    let line = self.pack_markup_line()?;
                    block.push_or(line);
                }
                _ => {
                    let ch = ch;
                    self.pending_markup_line
                        .push_str(markup_escape_text(&ch.to_string()).as_str());
                }
            }

            offset += ch.len_utf8();
        }
        while m < highlights.len() {
            let marker = &highlights[m];
            // let hl_def = self.highlights.get(&marker.group);
            match marker.kind {
                HighlightMarkerType::Start => {
                    let hl_def = self.opts.highlights.get(&marker.group);
                    let props: MarkupProperties = if let Some(hl_def) = hl_def {
                        hl_def.clone().into()
                    } else {
                        MarkupProperties::new()
                    };
                    self.push_span(MarkupSpan::new_with_properties(props))?;
                }
                HighlightMarkerType::End => {
                    self.pop_span()?;
                }
            }
            m += 1;
        }
        self.pop_span()?;
        block.push_or(self.pack_markup_line()?);
        Ok(Rc::new(Column::new(block.block_widgets)))
    }

    fn visit_heading<'a>(
        &mut self, node: &'a AstNode<'a>, heading: &NodeHeading,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut props = MarkupProperties::new();
        if heading.level <= 3 {
            props.insert(
                "font_size".into(),
                HEADING_FONT_SIZES[(heading.level as usize) - 1].into(),
            );
        }
        self.visit_virtual_block(node, VirtialBlockWrapper::Row, Some(props))
    }

    fn visit_list<'a>(
        &mut self, node: &'a AstNode<'a>, node_list: &NodeList,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut lines: Vec<Rc<dyn Widget>> = Vec::new();
        for c in node.children() {
            let w = self.visit_node(c)?;
            lines.push(w);
        }
        Ok(Rc::new(Column::new(lines)))
    }

    fn visit_list_item<'a>(
        &mut self, node: &'a AstNode<'a>, node_list: &NodeList,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block = BlockContext::new();
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
        block.block_widgets.push(Rc::new(MarkupParagraph::new(
            self.stack.wrap_text_owned(marker)?,
        )));
        for c in node.children() {
            if c.data.borrow().value.block() {
                let w = self.visit_block_node(c)?;
                block.push(w);
            } else {
                self.visit_inline_node(c, &mut block)?;
                let remaining = self.pack_markup_line()?;
                block.push_or(remaining);
            }
        }
        Ok(Rc::new(Row::new(block.block_widgets)))
    }

    fn visit_virtual_block<'a>(
        &mut self, node: &'a AstNode<'a>, wrapper: VirtialBlockWrapper,
        props: Option<MarkupProperties>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block = BlockContext::new();
        if let Some(props) = &props {
            self.push_span(MarkupSpan::new_with_properties(props.clone()))?;
        }
        // visit childrens
        for child in node.children() {
            if child.data.borrow().value.block() {
                let w = self.visit_block_node(child)?;
                block.push(w);
            } else {
                self.visit_inline_node(child, &mut block)?;
                let remaining = self.pack_markup_line()?;
                block.push_or(remaining);
            }
        }
        if props.is_some() {
            self.pop_span()?;
        }
        let ret: Rc<dyn Widget> = match wrapper {
            VirtialBlockWrapper::Column => {
                Rc::new(Column::new(block.block_widgets))
            }
            VirtialBlockWrapper::Row => Rc::new(Row::new(block.block_widgets)),
        };
        Ok(ret)
    }
}
