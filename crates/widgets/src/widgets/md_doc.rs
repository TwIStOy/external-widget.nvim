use std::sync::Arc;

use anyhow::{bail, Context, Ok};
use comrak::{
    nodes::{AstNode, NodeHeading, NodeList, NodeValue},
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
        self.stack.to_markup_open(&mut paragraph);
        std::mem::swap(&mut paragraph, &mut self.current_paragraph);
        paragraph
    }
}

pub fn md2markup(md: &str) {
    let arena = Arena::new();
    let opts = Options {
        ..Default::default()
    };
    let root = parse_document(&arena, md, &opts);
    let mut stack = MarkupSpanStack::new();
    visit_node(root, &mut stack, 0);
}

enum WidgetOrContent {
    Widget(Arc<dyn Widget>),
    Content(String),
}

pub fn visit_node<'a>(
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
        NodeValue::Item(_) => todo!(),
        NodeValue::CodeBlock(_) => todo!(),
        NodeValue::HtmlBlock(_) => todo!(),
        NodeValue::Paragraph => visit_virtual_node(node, builder, depth),
        NodeValue::Heading(heading) => {
            visit_heading(node, heading, builder, depth)
        }
        NodeValue::ThematicBreak => todo!(),
        NodeValue::FootnoteDefinition(_) => todo!(),
        NodeValue::Table(_) => todo!(),
        NodeValue::TableRow(_) => todo!(),
        NodeValue::TableCell => todo!(),
        NodeValue::Text(s) => visit_text(node, &s, builder, depth),
        NodeValue::TaskItem(_) => todo!(),
        NodeValue::SoftBreak => todo!(),
        NodeValue::LineBreak => todo!(),
        NodeValue::Code(_) => todo!(),
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

fn visit_document(
    node: &AstNode, builder: &mut MarkupParagraphBuilder,
) -> anyhow::Result<WidgetOrContent> {
    visit_virtual_node(node, builder, 0)
}

static HEADING_FONT_SIZES: [&str; 3] = ["200%", "150%", "120%"];

fn visit_heading<'a>(
    node: &'a AstNode<'a>, node_heading: &NodeHeading,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    if !node_heading.setext && node_heading.level < 3 {
        props.insert(
            "font_size".into(),
            HEADING_FONT_SIZES[node_heading.level as usize].into(),
        );
    }
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_simple_text_format_node<'a>(
    props: MarkupProperties, node: &'a AstNode<'a>,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let span = MarkupSpan::new_with_properties(props);
    builder.push_span(span.clone());
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
                    if content.len() > 0 {
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

fn visit_emph<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    props.insert("style".into(), "italic".into());
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_strong<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    props.insert("weight".into(), "bold".into());
    visit_simple_text_format_node(props, node, builder, depth)
}

fn visit_strike_strough<'a>(
    node: &'a AstNode<'a>, builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let mut props = MarkupProperties::new();
    props.insert("strikethrough".into(), "true".into());
    visit_simple_text_format_node(props, node, builder, depth)
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
    bar.width(LengthPercentage::Length(2.0));
    Ok(WidgetOrContent::Widget(Arc::new(Row::new(vec![
        Arc::new(bar),
        content,
    ]))))
}

fn visit_list<'a>(
    node: &'a AstNode<'a>, node_list: &NodeList,
    builder: &mut MarkupParagraphBuilder, depth: u32,
) -> anyhow::Result<WidgetOrContent> {
    let lines = vec![];
    for c in node.children() {
        let w = visit_node(c, builder, depth + 1)?;
        let w = normalize_widget(w, builder);
    }

    // let mut widgets: Vec<Arc<dyn Widget>> = vec![];
    // for c in node.children() {
    //     let w = visit_node(c, builder, depth + 1)?;
    //     if let Some(w) = normalize_widget(w, builder) {
    //         widgets.push(w);
    //     }
    // }
    // Ok(WidgetOrContent::Widget(Arc::new(Column::new(widgets))))
    Ok(())
}
