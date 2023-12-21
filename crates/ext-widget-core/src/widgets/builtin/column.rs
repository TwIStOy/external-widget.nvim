use std::rc::Rc;

use crate::{
    painting::{Axis, RenderCtx},
    widgets::{
        widget::{LayoutElement, Widget, WidgetKey},
        BoxOptions,
    },
};

#[derive(Debug, Clone)]
pub struct Column {
    key: WidgetKey,
    children: Vec<Rc<dyn Widget>>,
}

impl Column {
    pub fn new_with_children(children: Vec<Rc<dyn Widget>>) -> Self {
        let key = WidgetKey::next();
        Self { key, children }
    }

    pub fn add_child(&mut self, child: Rc<dyn Widget>) {
        self.children.push(child);
    }
}

impl LayoutElement for Column {
    fn style(&self) -> BoxOptions {
        BoxOptions {
            axis: Axis::Vertical,
            ..Default::default()
        }
    }
}

impl Widget for Column {
    fn key(&self) -> WidgetKey {
        self.key
    }

    fn children(&self) -> Vec<Rc<dyn Widget>> {
        self.children.clone()
    }

    fn paint(&self, _render: &mut RenderCtx<'_>) -> anyhow::Result<()> {
        // skip paint
        Ok(())
    }
}
