use std::{fmt::Debug, rc::Rc, sync::atomic::AtomicU64};

use crate::painting::{RectSize, RenderCtx, SpacePolicy};

use super::{BoxOptions, BuildContext};

static WIDGET_KEY: AtomicU64 = AtomicU64::new(0);

/// Unique key for each widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetKey(u64);

/// An element can be used in layout computation.
pub trait LayoutElement {
    /// Returns the style for initial layout tree registration.
    fn style(&self) -> BoxOptions;

    /// Returns the estimated size of this element.
    fn compute_layout(
        &self, _known_dimensions: RectSize<Option<f32>>,
        _available_space: RectSize<SpacePolicy>,
    ) -> RectSize<f32> {
        RectSize::default()
    }
}

pub trait Widget: LayoutElement + Debug {
    /// Get the unique key of the widget.
    fn key(&self) -> WidgetKey;

    /// Get the type name of the widget for debug.
    ///
    /// This method should not be overrided.
    fn type_name(&self) -> &'static str {
        let name = std::any::type_name::<Self>();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }

    /// Returns all children of this widget.
    fn children(&self) -> Vec<Rc<dyn Widget>> {
        vec![]
    }

    /// Returns is this widget need to build. Eg. [`StatefulWidget`] should
    /// return `true`.:
    fn need_build(&self) -> bool {
        false
    }

    /// If previous `need_build` returns `true`, the widget need build context
    /// to build itself.
    fn build(&self, _context: &BuildContext) -> anyhow::Result<()> {
        Ok(())
    }

    /// Paint widget.
    /// Each widget should only paint itself.
    fn paint(&self, context: &mut RenderCtx<'_>) -> anyhow::Result<()>;
}

impl WidgetKey {
    pub fn next() -> Self {
        Self(WIDGET_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}
