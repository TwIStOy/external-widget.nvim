use std::rc::Rc;

use anyhow::bail;
use comrak::nodes::{AstNode, NodeCode, NodeValue};
use skia_safe::{
    font_style::{Slant as SkSlant, Weight as SkWeight, Width as SkWidth},
    textlayout::{ParagraphBuilder, TextDecoration, TextStyle},
    FontStyle,
};

use crate::{
    nvim::nvim_highlight_into_text_style,
    widgets::{widget::Widget, RichText},
};

pub struct ConverterOptions {
    pub mono_font: String,
}

pub(crate) struct Converter<'o> {
    paragraph_builder: ParagraphBuilder,
    opts: &'o ConverterOptions,
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

/// **Inline**
impl<'o> Converter<'o> {
    fn pack_markup_line(&mut self) -> anyhow::Result<Option<Rc<dyn Widget>>> {
        if self.paragraph_builder.get_text().is_empty() {
            return Ok(None);
        }
        let paragraph = self.paragraph_builder.build();

        // TODO(hawtian): create new paragraph builder
        todo!("Construct a new ParagraphBuilder");

        Ok(Some(Rc::new(RichText::new_with_paragraph(paragraph))))
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
        &mut self, style: TextStyle, node: &'a AstNode<'a>,
        block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        self.paragraph_builder.push_style(&style);
        node.children()
            .try_for_each(|x| self.visit_inline_node(x, block))?;
        self.paragraph_builder.pop();
        Ok(())
    }

    /// **Inline**
    fn visit_text(&mut self, text: &str) -> anyhow::Result<()> {
        self.paragraph_builder.add_text(text);
        Ok(())
    }

    /// **Inline**
    fn visit_emph<'a>(
        &mut self, node: &'a AstNode<'a>, block: &mut BlockContext,
    ) -> anyhow::Result<()> {
        let mut style = self.paragraph_builder.peek_style();
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
        let mut style = self.paragraph_builder.peek_style();
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
        let mut style = self.paragraph_builder.peek_style();
        style.set_decoration_type(
            style.decoration_type() | TextDecoration::LINE_THROUGH,
        );

        self.visit_simple_inline_node(style, node, block)
    }

    /// **Inline**
    fn visit_code(&mut self, c: &NodeCode) -> anyhow::Result<()> {
        let mut style = self.paragraph_builder.peek_style();
        let hl = nvim_oxi::api::get_hl_by_name(
            "@text.literal.markdown_inline",
            true,
        )?;
        nvim_highlight_into_text_style(&hl, &mut style);
        style.set_font_families(&[&self.opts.mono_font]);

        self.paragraph_builder.push_style(&style);
        self.paragraph_builder.add_text(&c.literal);
        self.paragraph_builder.pop();
        Ok(())
    }
}
