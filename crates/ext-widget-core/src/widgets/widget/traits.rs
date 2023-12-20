use std::sync::atomic::AtomicU64;

use crate::widgets::support::RectSize;

static WIDGET_KEY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgeKey(u64);

pub trait Widget {
    /// Get the unique key of the widget.
    fn key(&self) -> WidgeKey;

    /// Get the type name of the widget for debug.
    ///
    /// This method should not be overrided.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Measure the widget, returns the expected size of this widget.
    fn measure(&self) -> RectSize<f32> {
        RectSize {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Mount this widget into the widget tree. It's each widget's duty to also
    /// mount its children.
    ///
    /// Returns the widget id.
    fn mount(&mut self) -> u64;
}

impl WidgeKey {
    pub fn next() -> Self {
        Self(WIDGET_KEY.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}
