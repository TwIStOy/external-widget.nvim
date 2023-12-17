use std::sync::Arc;

pub struct MeasureCtx {
    ctx: Arc<cairo::Context>,
}

impl MeasureCtx {
    pub fn new(ctx: Arc<cairo::Context>) -> Self {
        Self { ctx }
    }

    pub fn inner(&self) -> &cairo::Context {
        &self.ctx
    }
}
