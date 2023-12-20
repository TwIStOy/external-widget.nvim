mod hl;

use std::collections::HashMap;

use futures::AsyncWrite;
pub use hl::{hl_props_from_group, HighlightDefinition};
use nvim_rs::Neovim;
use parking_lot::Mutex;

pub type NvimWriter = Box<dyn AsyncWrite + Send + Unpin + 'static>;
pub type Nvim = Neovim<NvimWriter>;

pub struct NvimSession {
    inner: Nvim,
    highlights: Mutex<HashMap<String, HighlightDefinition>>,
}

impl NvimSession {
    /// Create a new NvimSession.
    pub fn new(nvim: Nvim) -> Self {
        Self {
            inner: nvim,
            highlights: Mutex::new(HashMap::new()),
        }
    }

    /// Get the highlight definition for a group.
    pub async fn get_highlight(
        &self, group: &str,
    ) -> Option<HighlightDefinition> {
        {
            let hl = self.highlights.lock();
            let ret = hl.get(group);
            if ret.is_some() {
                return ret.cloned();
            }
        }

        let color = hl_props_from_group(group.to_owned(), &self.inner).await;
        match color {
            Ok(color) => {
                let mut hl = self.highlights.lock();
                hl.insert(group.to_string(), color);
                hl.get(group).cloned()
            }
            Err(_) => None,
        }
    }

    // pub async fn get_tty(&self) -> anyhow::Result<String> {}
}
