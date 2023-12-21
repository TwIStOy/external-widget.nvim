use std::{rc::Rc, sync::atomic::AtomicU64};

use crate::widgets::support::RectSize;

use super::BoxOptions;

static WIDGET_KEY: AtomicU64 = AtomicU64::new(0);

/// Unique key for each widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgeKey(u64);

/// An element can be used in layout computation.
pub trait LayoutElement {
    /// Returns the style for initial layout tree registration.
    fn style(&self) -> BoxOptions;

    /// Returns the estimated size of this element.
    fn compute_layout(
        &self, _known_dimensions: RectSize<Option<f32>>,
        _available_space: RectSize<f32>,
    ) -> RectSize<f32> {
        RectSize::default()
    }
}

pub trait Widget: LayoutElement {
    /// Get the unique key of the widget.
    fn key(&self) -> WidgeKey;

    /// Get the type name of the widget for debug.
    ///
    /// This method should not be overrided.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Returns all children of this widget.
    fn children(&self) -> Vec<Rc<dyn Widget>> {
        vec![]
    }
}

impl WidgeKey {
    pub fn next() -> Self {
        Self(WIDGET_KEY.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}
