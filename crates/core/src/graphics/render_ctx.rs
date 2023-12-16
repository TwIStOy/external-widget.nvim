use crate::Color;

pub struct RenderCtx {
    ctx: cairo::Context,
}

impl RenderCtx {
    pub fn new(ctx: cairo::Context) -> Self {
        Self { ctx }
    }

    pub fn inner(&self) -> &cairo::Context {
        &self.ctx
    }

    pub fn set_color(&self, color: &Color) {
        self.ctx.set_source_rgba(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
            color.a as f64 / 255.0,
        );
    }

    pub fn move_to<X, Y>(&self, x: X, y: Y)
    where
        X: Into<f64>,
        Y: Into<f64>,
    {
        self.ctx.move_to(x.into(), y.into());
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.ctx.save()?;
        Ok(())
    }

    pub fn restore(&self) -> anyhow::Result<()> {
        self.ctx.restore()?;
        Ok(())
    }

    pub fn new_sub_path(&self) {
        self.ctx.new_sub_path();
    }

    pub fn close_path(&self) {
        self.ctx.close_path();
    }

    pub fn arc<X, Y, R, A1, A2>(
        &self, x: X, y: Y, radius: R, angle1: A1, angle2: A2,
    ) where
        X: Into<f64>,
        Y: Into<f64>,
        R: Into<f64>,
        A1: Into<f64>,
        A2: Into<f64>,
    {
        self.ctx.arc(
            x.into(),
            y.into(),
            radius.into(),
            angle1.into(),
            angle2.into(),
        );
    }

    pub fn set_line_width(&self, width: impl Into<f64>) {
        self.ctx.set_line_width(width.into());
    }

    pub fn stroke(&self) -> anyhow::Result<()> {
        self.ctx.stroke()?;
        Ok(())
    }

    pub fn fill(&self) -> anyhow::Result<()> {
        self.ctx.fill()?;
        Ok(())
    }

    pub fn fill_preserve(&self) -> anyhow::Result<()> {
        self.ctx.fill_preserve()?;
        Ok(())
    }
}
