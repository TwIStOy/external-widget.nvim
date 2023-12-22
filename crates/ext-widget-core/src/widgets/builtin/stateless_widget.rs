use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    painting::SpacePolicy,
    widgets::widget::{BuildContext, LayoutElement, Widget, WidgetKey},
};

pub trait StatelessWidget: Debug {
    fn build(&self, context: &BuildContext) -> Rc<dyn Widget>;
}

pub struct StatelessWidgetPod<T: StatelessWidget> {
    pub key: WidgetKey,
    pub widget: T,
    pub child: RefCell<Option<Rc<dyn Widget>>>,
}

impl<T: StatelessWidget> Debug for StatelessWidgetPod<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatelessWidget")
            .field("widget", &self.widget)
            .finish()
    }
}

impl<T: StatelessWidget> StatelessWidgetPod<T> {
    pub fn new(widget: T) -> Self {
        Self {
            key: WidgetKey::next(),
            widget,
            child: RefCell::new(None),
        }
    }

    pub fn build_inner(&self, context: &BuildContext) -> Rc<dyn Widget> {
        let mut child = self.child.borrow_mut();
        if child.is_none() || context.is_dirty(self.key) {
            let c = self.widget.build(context);
            child.replace(c.clone());
            c
        } else {
            child.as_ref().unwrap().clone()
        }
    }
}

impl<T: StatelessWidget> LayoutElement for StatelessWidgetPod<T> {
    fn style(&self) -> crate::widgets::BoxOptions {
        self.child.borrow().as_ref().unwrap().style()
    }

    fn compute_layout(
        &self, known_dimensions: crate::painting::RectSize<Option<f32>>,
        available_space: crate::painting::RectSize<SpacePolicy>,
    ) -> crate::painting::RectSize<f32> {
        self.child
            .borrow()
            .as_ref()
            .unwrap()
            .compute_layout(known_dimensions, available_space)
    }
}

impl<T: StatelessWidget> Widget for StatelessWidgetPod<T> {
    fn key(&self) -> WidgetKey {
        self.key
    }

    fn children(&self) -> Vec<Rc<dyn Widget>> {
        self.child.borrow().as_ref().unwrap().children()
    }

    fn paint(
        &self, render: &mut crate::painting::RenderCtx<'_>,
    ) -> anyhow::Result<()> {
        self.child.borrow().as_ref().unwrap().paint(render)
    }

    fn type_name(&self) -> &'static str {
        self.child.borrow().as_ref().unwrap().type_name()
    }

    fn need_build(&self) -> bool {
        true
    }

    fn build(&self, context: &BuildContext) -> anyhow::Result<()> {
        self.build_inner(context);
        Ok(())
    }
}
