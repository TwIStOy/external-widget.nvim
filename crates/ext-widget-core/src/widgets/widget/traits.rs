use taffy::Size;

pub trait Widget {
    /// Get the type name of the widget for debug.
    ///
    /// This method should not be overrided.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Measure the widget, returns the expected size of this widget.
    fn measure(&self) -> Size<f32> {
    }
}
