mod hl;

pub use hl::{hl_props_from_group, HighlightDefinition};
use nvim_rs::Neovim;
use tokio::{io::WriteHalf, net::TcpStream};
use tokio_util::compat::Compat;

pub type NvimWriter = Compat<WriteHalf<TcpStream>>;
pub type Nvim = Neovim<NvimWriter>;
