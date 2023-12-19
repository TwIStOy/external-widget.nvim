mod hl;

use futures::AsyncWrite;
pub use hl::{hl_props_from_group, HighlightDefinition};
use nvim_rs::Neovim;

pub type NvimWriter = Box<dyn AsyncWrite + Send + Unpin + 'static>;
pub type Nvim = Neovim<NvimWriter>;
