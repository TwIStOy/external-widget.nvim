mod support;
mod widgets;

use std::{io::Write, rc::Rc};

use external_widget_core::{MeasureCtx, RenderCtx, Widget, WidgetTree};
pub use widgets::*;

const MARKUPS: &[&str] = &[
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span font_size="130%" foreground="#eed4a0"><span>function <span font_family="MonoLisa" foreground="#8bd5cb">main</span></span></span></span></span></span>"##,
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span>→ <span font_family="MonoLisa" foreground="#8bd5cb">int</span></span></span></span></span>"##,
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span>Parameters<span>:</span></span></span></span></span>"##,
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span> - <span><span font_family="MonoLisa" foreground="#8bd5cb">int argc</span></span></span></span></span></span>"##,
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span> - <span><span font_family="MonoLisa" foreground="#8bd5cb">const char * argv</span></span></span></span></span></span>"##,
    r##"<span foreground="#cad3f6" background="#1e2031" size="15pt" font_family="MonoLisa"><span><span><span font_family="MonoLisa"><span foreground="#7dc4e5">public</span><span foreground="#939ab8">:</span> <span foreground="#eed4a0">int</span> <span foreground="#8aadf5"><span foreground="#cad3f6">main</span></span><span foreground="#939ab8">(</span><span foreground="#eed4a0">int</span> <span foreground="#cad3f6">argc</span><span foreground="#939ab8">,</span> <span foreground="#cad3f6">const</span> <span foreground="#cad3f6">char</span> <span foreground="#91d7e4">*</span><span foreground="#cad3f6">argv</span><span foreground="#939ab8">)</span></span></span></span></span>"##,
];

pub fn render_widget_tree_to_buf(
    root: Rc<dyn Widget>, canvas_width: i32, canvas_height: i32,
) -> anyhow::Result<Vec<u8>> {
    let surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        canvas_width,
        canvas_height,
    )?;
    let ctx = Rc::new(cairo::Context::new(&surface)?);
    let render_ctx = RenderCtx::new(ctx.clone());
    let measure_ctx = MeasureCtx::new(ctx.clone());

    root.print_element();

    let mut tree = WidgetTree::new();
    let root = root.register(&mut tree)?;

    tree.set_root(root);
    tree.print_tree(&measure_ctx);

    tree.draw_from_root(&render_ctx)?;

    let mut writer = std::io::BufWriter::new(Vec::new());
    surface.write_to_png(&mut writer)?;
    writer.flush()?;

    Ok(writer.into_inner()?)
}

pub fn render_widget_tree(
    root: Rc<dyn Widget>, canvas_width: i32, canvas_height: i32,
) -> anyhow::Result<()> {
    let surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        canvas_width,
        canvas_height,
    )?;
    let ctx = Rc::new(cairo::Context::new(&surface)?);
    let render_ctx = RenderCtx::new(ctx.clone());
    let measure_ctx = MeasureCtx::new(ctx.clone());

    let mut tree = WidgetTree::new();
    let root = root.register(&mut tree)?;

    tree.set_root(root);
    tree.print_tree(&measure_ctx);

    tree.draw_from_root(&render_ctx)?;

    // write to /tmp/test.png
    let mut file = std::fs::File::create("/tmp/test.png")?;
    let mut stream = std::io::BufWriter::new(&mut file);
    surface.write_to_png(&mut stream)?;
    stream.flush()?;

    Ok(())
}

pub fn taffy_test() -> anyhow::Result<()> {
    let title = Rc::new(Column::new(
        MARKUPS
            .iter()
            .map(|markup| {
                Rc::new(MarkupParagraph::new(markup.to_string()))
                    as Rc<dyn Widget>
            })
            .collect(),
    ));

    render_widget_tree(title, 1000, 1000)?;

    Ok(())
}
