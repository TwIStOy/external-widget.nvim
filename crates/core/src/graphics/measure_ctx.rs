use std::rc::Rc;

pub struct MeasureCtx {
    ctx: Rc<cairo::Context>,
}

impl MeasureCtx {
    pub fn new(ctx: Rc<cairo::Context>) -> Self {
        Self { ctx }
    }

    pub fn inner(&self) -> &cairo::Context {
        &self.ctx
    }
}
