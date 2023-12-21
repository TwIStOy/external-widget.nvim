mod color;
mod renderer;
mod support;

// re-export
pub use color::{Color, ParseColorError};
pub use renderer::{RenderCtx, Renderer};
pub use support::*;
