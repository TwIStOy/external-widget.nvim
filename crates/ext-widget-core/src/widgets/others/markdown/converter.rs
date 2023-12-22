use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use anyhow::bail;
use comrak::nodes::{
    AstNode, NodeCode, NodeCodeBlock, NodeHeading, NodeList, NodeValue,
};
use skia_safe::{
    font_style::{Slant as SkSlant, Weight as SkWeight, Width as SkWidth},
    textlayout::{
        FontCollection, ParagraphBuilder, ParagraphStyle, TextDecoration,
        TextStyle,
    },
    FontMgr, FontStyle,
};

use crate::{
    nvim::nvim_highlight_into_text_style,
    widgets::{widget::Widget, Column, RichText, Row},
};

use super::codeblock::{get_all_captures, HighlightMarkerType};

pub struct ConverterOptions {
    pub mono_font: String,
}

pub(crate) struct Converter<'o, 'f> {
    opts: &'o ConverterOptions,
    font_collection: &'f FontCollection,
}

struct BlockContext {
    block_widgets: Vec<Rc<dyn Widget>>,
    last_paragraph: ParagraphBuilder,
}

impl BlockContext {
    fn new(
        paragraph_style: &ParagraphStyle, font_collection: &FontCollection,
    ) -> Self {
        Self {
            block_widgets: Vec::new(),
            last_paragraph: ParagraphBuilder::new(
                paragraph_style,
                font_collection,
            ),
        }
    }

    fn new_from_other(
        other: &mut Self, font_collection: &FontCollection,
    ) -> Self {
        let paragraph_style = other.last_paragraph.get_paragraph_style();
        let mut paragraph_builder =
            ParagraphBuilder::new(&paragraph_style, font_collection);
        paragraph_builder.push_style(&other.last_paragraph.peek_style());
        Self {
            block_widgets: Vec::new(),
            last_paragraph: paragraph_builder,
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
        let paragraph = self.last_paragraph.build();
        // TODO(hawtian): clear text
        Ok(Some(Rc::new(RichText::new_with_paragraph(paragraph))))
    }
}

impl<'o, 'f> Converter<'o, 'f> {
    fn new(
        opts: &'o ConverterOptions, font_collection: &'f FontCollection,
    ) -> Self {
        Self {
            opts,
            font_collection,
        }
    }
}

/// **Inline**
impl<'o, 'f> Converter<'o, 'f> {
    fn visit_inline_node<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let data = &node.data.borrow().value;
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

    fn visit_simple_inline_node<'a>(
        &mut self, style: TextStyle, node: &'a AstNode<'a>,
        block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        block.last_paragraph.push_style(&style);
        node.children()
            .try_for_each(|x| self.visit_inline_node(x, block))?;
        block.last_paragraph.pop();
        Ok(())
    }

    /// **Inline**
    fn visit_text(
        &mut self, text: &str, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        block.last_paragraph.add_text(text);
        Ok(())
    }

    /// **Inline**
    fn visit_emph<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
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
    fn visit_strong<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
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
    fn visit_strike_strough<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        style.set_decoration_type(
            style.decoration_type() | TextDecoration::LINE_THROUGH,
        );

        self.visit_simple_inline_node(style, node, block)
    }

    /// **Inline**
    fn visit_code(
        &mut self, c: &NodeCode, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut style = block.last_paragraph.peek_style();
        let hl = nvim_oxi::api::get_hl_by_name(
            "@text.literal.markdown_inline",
            true,
        )?;
        nvim_highlight_into_text_style(&hl, &mut style);
        style.set_font_families(&[&self.opts.mono_font]);

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
impl<'c, 'f> Converter<'c, 'f> {
    fn visit_block_node<'a>(
        &mut self, node: &'a AstNode<'a>, block: Option<&RefCell<BlockContext>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let value = &node.data.borrow().value;
        assert!(value.block());
        match value {
            NodeValue::Document => self.visit_block_common(
                node,
                VirtialBlockWrapper::Column,
                block,
            ),
            NodeValue::FrontMatter(_) => todo!(),
            NodeValue::BlockQuote => todo!(),
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
            // NodeValue::ThematicBreak => {
            //     Ok(Rc::new(Bar::new().height(LengthPercentage::Length(2.))))
            // }
            _ => bail!("Unsupported block node: {:?}", value),
        }
    }

    fn visit_code_block(
        &mut self, codeblock: &NodeCodeBlock,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut block =
            BlockContext::new(&Default::default(), self.font_collection);
        let mut style = block.last_paragraph.peek_style();
        style.set_font_families(&[&self.opts.mono_font]);
        block.last_paragraph.push_style(&style);

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
                        let hl =
                            nvim_oxi::api::get_hl_by_name(&marker.group, true)?;
                        let mut style = block.last_paragraph.peek_style();
                        nvim_highlight_into_text_style(&hl, &mut style);
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
                    let hl =
                        nvim_oxi::api::get_hl_by_name(&marker.group, true)?;
                    let mut style = block.last_paragraph.peek_style();
                    nvim_highlight_into_text_style(&hl, &mut style);
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
        let base_font_size = block
            .last_paragraph
            .get_paragraph_style()
            .text_style()
            .font_size();
        style.set_font_families(&[&self.opts.mono_font]);
        style.set_font_size(base_font_size * scale);
        block.last_paragraph.push_style(&style);
        for child in node.children() {
            assert!(!child.data.borrow().value.block());
            self.visit_inline_node(child, &mut block)?;
        }
        Ok(block.pack_content()?.unwrap())
    }

    fn visit_list<'a>(
        &mut self, node: &'a AstNode<'a>, _node_list: &NodeList,
        block: Option<&RefCell<BlockContext>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let mut lines: Vec<Rc<dyn Widget>> = Vec::new();
        for c in node.children() {
            let w = self.visit_block_node(c, block)?;
            lines.push(w);
        }
        Ok(Rc::new(Column::new_with_children(lines)))
    }

    fn visit_list_item<'a>(
        &mut self, node: &'a AstNode<'a>, node_list: &NodeList,
        block: Option<&RefCell<BlockContext>>,
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
        Ok(Rc::new(Column::new_with_children(block.block_widgets)))
    }

    fn visit_block_common<'a>(
        &mut self, node: &'a AstNode<'a>, wrapper: VirtialBlockWrapper,
        block: Option<&RefCell<BlockContext>>,
    ) -> anyhow::Result<Rc<dyn Widget>> {
        let block = match block {
            Some(block) => BlockContext::new_from_other(
                &mut block.borrow_mut(),
                self.font_collection,
            ),
            None => {
                BlockContext::new(&Default::default(), self.font_collection)
            }
        };
        let block = RefCell::new(block);
        // visit childrens
        for child in node.children() {
            if child.data.borrow().value.block() {
                let w = self.visit_block_node(child, Some(&block))?;
                block.borrow_mut().push(w);
            } else {
                self.visit_inline_node(child, &mut block.borrow_mut())?;
                let content = block.borrow_mut().pack_content()?;
                block.borrow_mut().push_or(content);
            }
        }
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
