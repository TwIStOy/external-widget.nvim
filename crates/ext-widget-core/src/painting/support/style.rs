use std::ops::Sub;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RectSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> Sub<RectSize<T>> for RectSize<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: RectSize<T>) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
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

impl<T, U> From<RectSize<T>> for taffy::Size<U>
where
    T: Into<U>,
{
    fn from(value: RectSize<T>) -> Self {
        Self {
            width: value.width.into(),
            height: value.height.into(),
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
