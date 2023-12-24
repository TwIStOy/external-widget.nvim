use std::path::{Path, PathBuf};

use futures::AsyncWrite;
use nvim_rs::Neovim;

pub async fn find_file_in_runtime_path<W, P>(
    nvim: &Neovim<W>, path: P,
) -> anyhow::Result<Option<PathBuf>>
where
    P: AsRef<Path>,
    W: AsyncWrite + Send + Unpin + 'static,
{
    for rtp in nvim.list_runtime_paths().await? {
        let rtp = PathBuf::from(rtp);
        let p = rtp.join(path.as_ref());
        if p.exists() && p.is_file() {
            return Ok(Some(p));
        }
    }
    Ok(None)
}
