use std::sync::Arc;

use anyhow::{bail, Context, Ok};
use comrak::{
    nodes::{
        AstNode, NodeCode, NodeCodeBlock, NodeHeading, NodeList, NodeValue,
    },
    parse_document, Arena, Options,
};
use external_widget_core::{
    pango::{MarkupProperties, MarkupSpan, MarkupSpanStack},
    Widget,
};
use taffy::LengthPercentage;

use crate::{Bar, Column, MarkupParagraph, Row};

#[derive(Debug, Clone)]
pub struct MdDocument {
    md: String,
}

// impl Widget for MdDocument {
//     fn layout(&mut self, _bc: &BoxConstraints) -> Dimension {
//         Dimension::new(0.0, 0.0)
//     }

//     fn draw(&mut self, _ctx: &mut RenderCtx) {
//         todo!()
//     }
// }

struct MarkupParagraphBuilder {
    stack: MarkupSpanStack,
    current_paragraph: String,
}

impl MarkupParagraphBuilder {
    fn new() -> Self {
        Self {
            stack: MarkupSpanStack::new(),
            current_paragraph: String::new(),
        }
    }

    fn push_span(&mut self, span: MarkupSpan) -> anyhow::Result<()> {
        span.to_markup_open(&mut self.current_paragraph)?;
        self.stack.push(span);
        Ok(())
    }

    fn pop_span(&mut self) -> anyhow::Result<()> {
        let span = self.stack.pop().context("No span?")?;
        span.to_markup_close(&mut self.current_paragraph);
        Ok(())
    }

    fn push(&mut self, text: &str) {
        self.current_paragraph.push_str(text);
    }

    fn new_paragraph(&mut self) -> String {
        let mut paragraph = String::new();
        self.stack.to_markup_open(&mut paragraph).unwrap();
        std::mem::swap(&mut paragraph, &mut self.current_paragraph);
        paragraph
    }

    fn wrap_text(&mut self, text: &str) -> anyhow::Result<String> {
        let mut ret = String::new();
        self.stack.to_markup_open(&mut ret)?;
        ret.push_str(text);
        self.stack.to_markup_close(&mut ret);
        Ok(ret)
    }
}

pub fn md2markup(md: &str) {
    let arena = Arena::new();
    let opts = Options {
        ..Default::default()
    };
    let root = parse_document(&arena, md, &opts);
    let mut builder = MarkupParagraphBuilder::new();
    let res = visit_node(root, &mut builder, 0).unwrap();
    match res {
        WidgetOrContent::Widget(w) => {
            w.print_element(true, 0);
        }
        WidgetOrContent::Content(c) => {
            println!("Content: {}", c);
        }
    }
}

#[derive(Debug)]
enum WidgetOrContent {
    Widget(Arc<dyn Widget>),
    Content(String),
}

fn visit_node<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    println!(
        "{}{:?}",
        " ".repeat((depth * 2) as usize),
        node.data.borrow()
    );
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Document => visit_document(node, builder),
        NodeValue::BlockQuote => visit_block_quote(node, builder, depth),
        NodeValue::List(node_list) => {
            visit_list(node, node_list, builder, depth)
        }
        NodeValue::Item(node_list) => {
            visit_list(node, node_list, builder, depth)
        }
        NodeValue::CodeBlock(node_codeblock) => {
            visit_code_block(node, node_codeblock, builder, depth)
        }
        NodeValue::Paragraph => visit_virtual_node(node, builder, depth),
        NodeValue::Heading(heading) => {
            visit_heading(node, heading, builder, depth)
        }
        NodeValue::ThematicBreak => visit_thematic_break(node, builder, depth),
        NodeValue::Text(s) => visit_text(node, s, builder, depth),
        NodeValue::TaskItem(_) => todo!(),
        NodeValue::SoftBreak => todo!(),
        NodeValue::LineBreak => Ok(WidgetOrContent::Content("\n".to_string())),
        NodeValue::Code(node_code) => {
            visit_code(node, node_code, builder, depth)
        }
        NodeValue::HtmlInline(_) => todo!(),
        NodeValue::Emph => visit_emph(node, builder, depth),
        NodeValue::Strong => visit_strong(node, builder, depth),
        NodeValue::Strikethrough => visit_strike_strough(node, builder, depth),
        NodeValue::Link(_) => todo!(),
        NodeValue::Image(_) => todo!(),
        _ => {
            bail!("Not supported yet: {:?}", data.value);
        }
    }
}

fn normalize_widget(
    w: WidgetOrContent, builder: &mut MarkupParagraphBuilder,
) -> Option<Arc<dyn Widget>> {
    match w {
        WidgetOrContent::Widget(w) => Some(w),
        WidgetOrContent::Content(c) => {
            let p = builder.new_paragraph();
            if p.is_empty() {
                return None;
            }
            Some(Arc::new(MarkupParagraph::new(p)))
        }
    }
}

fn visit_thematic_break<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    Ok(WidgetOrContent::Widget(Arc::new(
        Bar::new().height(LengthPercentage::Length(2.0)),
    )))
}

static HEADING_FONT_SIZES: [&str; 3] = ["200%", "150%", "120%"];

fn visit_code_block<'a>(
    node: &'a AstNode<'a>, _node_codeblock: &NodeCodeBlock,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    props.insert("family".into(), "monospace".into());
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_heading<'a>(
    node: &'a AstNode<'a>, node_heading: &NodeHeading,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    if !node_heading.setext && node_heading.level <= 3 {
        props.insert(
            "font_size".into(),
            HEADING_FONT_SIZES[(node_heading.level as usize) - 1].into(),
        );
    }
    let r = builder.new_paragraph();
    assert!(r.is_empty());
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_code<'a>(
    node: &'a AstNode<'a>, node_code: &NodeCode,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    props.insert("family".into(), "monospace".into());
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_simple_text_format_node<'a>(
    props: MarkupProperties, node: &'a AstNode<'a>,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let span = MarkupSpan::new_with_properties(props);
    builder.push_span(span.clone())?;
    let s = node
        .children()
        .map(|c| visit_node(c, builder, depth + 1))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let all_text = s.iter().all(|x| matches!(x, WidgetOrContent::Content(_)));
    let ret = if all_text {
        let content = s
            .into_iter()
            .map(|x| match x {
                WidgetOrContent::Content(c) => c,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
            .join("");
        WidgetOrContent::Content(span.wrap_text_owned(content)?)
    } else {
        let mut widgets: Vec<Arc<dyn Widget>> = vec![];
        for c in s {
            match c {
                WidgetOrContent::Widget(w) => {
                    let content = builder.new_paragraph();
                    if !content.is_empty() {
                        widgets.push(Arc::new(MarkupParagraph::new(content)));
                    }
                    widgets.push(w);
                }
                WidgetOrContent::Content(c) => {
                    builder.push(&c);
                }
            }
        }
        WidgetOrContent::Widget(Arc::new(Column::new(widgets)))
    };
    builder.pop_span();
    Ok(ret)
}

fn visit_block_node<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<Arc<dyn Widget>> {
    let value = &node.data.borrow().value;
    assert!(value.block());
    match value {
        NodeValue::Document => {}
        NodeValue::FrontMatter(_) => todo!(),
        NodeValue::BlockQuote => todo!(),
        NodeValue::List(_) => todo!(),
        NodeValue::Item(_) => todo!(),
        NodeValue::DescriptionList => todo!(),
        NodeValue::DescriptionItem(_) => todo!(),
        NodeValue::DescriptionTerm => todo!(),
        NodeValue::DescriptionDetails => todo!(),
        NodeValue::CodeBlock(_) => todo!(),
        NodeValue::HtmlBlock(_) => todo!(),
        NodeValue::Paragraph => todo!(),
        NodeValue::Heading(_) => todo!(),
        NodeValue::ThematicBreak => todo!(),
        NodeValue::FootnoteDefinition(_) => todo!(),
        NodeValue::Table(_) => todo!(),
        NodeValue::TableRow(_) => todo!(),
        NodeValue::TableCell => todo!(),
        _ => unreachable!(),
    }
}

fn visit_virtual_node<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let children = node
        .children()
        .map(|child| visit_node(child, builder, depth + 1))
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|w| normalize_widget(w, builder))
        .collect::<Vec<_>>();
    Ok(WidgetOrContent::Widget(Arc::new(Column::new(children))))
}

fn visit_document<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder,
) -> anyhow::Result<WidgetOrContent> {
    visit_virtual_node(node, builder, 0)
}

fn visit_inline_node<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<String> {
    let value = &node.data.borrow().value;
    assert!(!value.block());
    match value {
        NodeValue::Text(text) => {
            let content = glib::markup_escape_text(text);
            Ok(content.to_string())
        }
        NodeValue::SoftBreak => todo!(),
        NodeValue::LineBreak => todo!(),
        NodeValue::Code(c) => {
            let mut props = MarkupProperties::new();
            props.insert("font".into(), "monospace".into());
            let span = MarkupSpan::new_with_properties(props);
            let content = glib::markup_escape_text(&c.literal);
            Ok(span.wrap_text_owned(content.to_string())?)
        }
        NodeValue::Emph => {
            let mut props = MarkupProperties::new();
            props.insert("style".into(), "italic".into());
            visit_simple_inline_node(props, node, builder, depth)
        }
        NodeValue::Strong => {
            let mut props = MarkupProperties::new();
            props.insert("weight".into(), "bold".into());
            visit_simple_inline_node(props, node, builder, depth)
        }
        NodeValue::Strikethrough => {
            let mut props = MarkupProperties::new();
            props.insert("strikethrough".into(), "true".into());
            visit_simple_inline_node(props, node, builder, depth)
        }
        NodeValue::Link(_) => todo!(),
        NodeValue::Image(_) => todo!(),
        _ => unreachable!(),
    }
}

fn visit_text<'a>(
    node: &'a AstNode<'a>, text: &str, builder: &mut MarkupParagraphBuilder,
    depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let content = glib::markup_escape_text(text);
    Ok(WidgetOrContent::Content(content.to_string()))
}

fn visit_block_quote<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut widgets: Vec<Arc<dyn Widget>> = vec![];
    for c in node.children() {
        let w = visit_node(c, builder, depth + 1)?;
        if let Some(w) = normalize_widget(w, builder) {
            widgets.push(w);
        }
    }
    let content = Arc::new(Column::new(widgets));
    let bar = Bar::new();
    let bar = bar.width(LengthPercentage::Length(2.0));
    Ok(WidgetOrContent::Widget(Arc::new(Row::new(vec![
        Arc::new(bar),
        content,
    ]))))
}

fn visit_list<'a>(
    node: &'a AstNode<'a>, node_list: &NodeList,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut lines: Vec<Arc<dyn Widget>> = vec![];
    for c in node.children() {
        let w = visit_node(c, builder, depth + 1)?;
        let w = normalize_widget(w, builder);
        let marker = match node_list.list_type {
            comrak::nodes::ListType::Bullet => "•",
            comrak::nodes::ListType::Ordered => "1.",
        };
        let marker = MarkupParagraph::new(builder.wrap_text(marker)?);
        let mut line: Vec<Arc<dyn Widget>> = vec![Arc::new(marker)];
        if let Some(w) = w {
            line.push(w);
        }
        lines.push(Arc::new(Row::new(line)));
    }
    Ok(WidgetOrContent::Widget(Arc::new(Column::new(lines))))
}
