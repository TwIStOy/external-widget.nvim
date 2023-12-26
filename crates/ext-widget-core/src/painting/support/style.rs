#[derive(Debug, Clone, Copy)]
pub struct RectSize<T> {
    pub width: T,
    pub height: T,
}

impl<T: Default> Default for RectSize<T> {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl<T> From<taffy::Size<T>> for RectSize<T> {
    fn from(value: taffy::Size<T>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl<T> From<RectSize<T>> for taffy::Size<T> {
    fn from(value: RectSize<T>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<RectSize<f32>> for skia_safe::Size {
    fn from(value: RectSize<f32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}
