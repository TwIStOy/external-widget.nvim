use std::{fmt::Debug, rc::Rc};

use crate::{
    painting::{Axis, RenderCtx},
    widgets::{
        widget::{LayoutElement, Widget, WidgetKey},
        BoxOptions,
    },
};

#[derive(Clone)]
pub struct Row {
    key: WidgetKey,
    children: Vec<Rc<dyn Widget>>,
}

impl Row {
    pub fn new_with_children(children: Vec<Rc<dyn Widget>>) -> Self {
        let key = WidgetKey::next();
        Self { key, children }
    }

    pub fn add_child(&mut self, child: Rc<dyn Widget>) {
        self.children.push(child);
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Row")
            .field("children", &self.children.len())
            .finish()
    }
}

impl LayoutElement for Row {
    fn style(&self) -> BoxOptions {
        BoxOptions {
            axis: Axis::Horizontal,
            ..Default::default()
        }
    }
}

impl Widget for Row {
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
