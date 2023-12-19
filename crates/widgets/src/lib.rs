mod support;
mod widgets;

use std::{io::Write, rc::Rc};

use external_widget_core::{MeasureCtx, RenderCtx, Widget, WidgetTree};
pub use widgets::*;

fn render_widget_tree<W>(
    root: Rc<dyn Widget>, canvas_width: i32, canvas_height: i32, output: &mut W,
) -> anyhow::Result<()>
where
    W: Write,
{
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

    surface.write_to_png(output)?;

    Ok(())
}

pub fn render_widget_tree_to_buf(
    root: Rc<dyn Widget>, canvas_width: i32, canvas_height: i32,
) -> anyhow::Result<Vec<u8>> {
    let mut writer = std::io::BufWriter::new(Vec::new());
    render_widget_tree(root, canvas_width, canvas_height, &mut writer)?;
    writer.flush()?;

    Ok(writer.into_inner()?)
}

pub fn render_widget_tree_to_file(
    root: Rc<dyn Widget>, canvas_width: i32, canvas_height: i32,
) -> anyhow::Result<()> {
    // write to /tmp/test.png
    let mut file = std::fs::File::create("/tmp/test.png")?;
    let mut stream = std::io::BufWriter::new(&mut file);
    render_widget_tree(root, canvas_width, canvas_height, &mut stream)?;
    stream.flush()?;
    Ok(())
}
