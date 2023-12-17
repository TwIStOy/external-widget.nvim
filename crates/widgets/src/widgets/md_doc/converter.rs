use std::sync::Arc;

use anyhow::bail;
use comrak::nodes::{AstNode, NodeCode, NodeCodeBlock, NodeHeading, NodeValue};
use external_widget_core::{
    pango::{MarkupProperties, MarkupSpan, MarkupSpanStack},
    Widget,
};
use taffy::LengthPercentage;

use crate::{Bar, Column, MarkupParagraph, Row};

pub(crate) struct Converter {
    pending_markup_line: String,
    stack: MarkupSpanStack,
}

struct BlockContext {
    block_widgets: Vec<Arc<dyn Widget>>,
}

impl BlockContext {
    fn new() -> Self {
        Self {
            block_widgets: Vec::new(),
        }
    }

    fn push(&mut self, w: Arc<dyn Widget>) {
        self.block_widgets.push(w);
    }

    fn push_or(&mut self, w: Option<Arc<dyn Widget>>) {
        if let Some(w) = w {
            self.push(w);
        }
    }
}

impl Converter {
    pub(crate) fn new() -> Self {
        Self {
            pending_markup_line: String::new(),
            stack: MarkupSpanStack::new(),
        }
    }

    pub(crate) fn visit_node<'a>(
        &mut self, node: &'a AstNode<'a>,
    ) -> anyhow::Result<Arc<dyn Widget>> {
        let value = &node.data.borrow().value;
        if value.block() {
            self.visit_block_node(node)
        } else {
            bail!("Unsupported inline node: {:?}", value)
        }
    }
}

/// **Inline**
impl Converter {
    fn pack_markup_line(&mut self) -> anyhow::Result<Option<Arc<dyn Widget>>> {
        let markup_start = self.stack.to_markup_open_owned()?;
        if self.pending_markup_line.is_empty()
            || self.pending_markup_line == markup_start
        {
            return Ok(None);
        }
        self.stack.to_markup_close(&mut self.pending_markup_line);
        let mut ret = String::new();
        std::mem::swap(&mut self.pending_markup_line, &mut ret);
        Ok(Some(Arc::new(MarkupParagraph::new(ret))))
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
        self.stack.push(span);
        node.children()
            .try_for_each(|x| self.visit_inline_node(x, block))?;
        self.stack.pop();
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
        props.insert("font".into(), "monospace".into());
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
impl Converter {
    fn visit_block_node<'a>(
        &mut self, node: &'a AstNode<'a>,
    ) -> anyhow::Result<Arc<dyn Widget>> {
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
            NodeValue::List(_) => todo!(),
            NodeValue::Item(_) => todo!(),
            NodeValue::CodeBlock(codeblock) => self.visit_code_block(codeblock),
            NodeValue::Paragraph => self.visit_virtual_block(
                node,
                VirtialBlockWrapper::Column,
                None,
            ),
            NodeValue::Heading(heading) => self.visit_heading(node, heading),
            NodeValue::ThematicBreak => {
                Ok(Arc::new(Bar::new().height(LengthPercentage::Length(2.))))
            }
            _ => bail!("Unsupported block node: {:?}", value),
        }
    }

    fn visit_code_block(
        &mut self, codeblock: &NodeCodeBlock,
    ) -> anyhow::Result<Arc<dyn Widget>> {
        let mut props = MarkupProperties::new();
        props.insert("font".into(), "monospace".into());
        let span = MarkupSpan::new_with_properties(props);
        self.stack.push(span);
        let lines: Vec<String> = codeblock
            .literal
            .split('\n')
            .map(|x| x.trim_end())
            .map(|x| {
                let mut ret = String::new();
                self.stack.to_markup_open(&mut ret)?;
                ret.push_str(glib::markup_escape_text(x).as_str());
                self.stack.to_markup_close(&mut ret);
                Ok(ret)
            })
            .collect::<anyhow::Result<_>>()?;
        let widgets = lines
            .into_iter()
            .map(|x| Arc::new(MarkupParagraph::new(x)) as Arc<dyn Widget>)
            .collect::<Vec<_>>();
        self.stack.pop();
        Ok(Arc::new(Column::new(widgets)))
    }

    fn visit_heading<'a>(
        &mut self, node: &'a AstNode<'a>, heading: &NodeHeading,
    ) -> anyhow::Result<Arc<dyn Widget>> {
        let mut props = MarkupProperties::new();
        if !heading.setext && heading.level <= 3 {
            props.insert(
                "font_size".into(),
                HEADING_FONT_SIZES[(heading.level as usize) - 1].into(),
            );
        }
        self.visit_virtual_block(node, VirtialBlockWrapper::Row, Some(props))
    }

    fn visit_virtual_block<'a>(
        &mut self, node: &'a AstNode<'a>, wrapper: VirtialBlockWrapper,
        props: Option<MarkupProperties>,
    ) -> anyhow::Result<Arc<dyn Widget>> {
        let mut block = BlockContext::new();
        if let Some(props) = &props {
            self.stack
                .push(MarkupSpan::new_with_properties(props.clone()));
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
            self.stack.pop();
        }
        let ret: Arc<dyn Widget> = match wrapper {
            VirtialBlockWrapper::Column => {
                Arc::new(Column::new(block.block_widgets))
            }
            VirtialBlockWrapper::Row => Arc::new(Row::new(block.block_widgets)),
        };
        Ok(ret)
    }
}
