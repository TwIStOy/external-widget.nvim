use nvim_oxi::api::list_runtime_paths;
use std::path::{Path, PathBuf};

pub fn find_file_in_runtime_path<P>(path: P) -> anyhow::Result<Option<PathBuf>>
where
    P: AsRef<Path>,
{
    for rtp in list_runtime_paths()? {
        let p = rtp.join(path.as_ref());
        if p.exists() && p.is_file() {
            return Ok(Some(p));
        }
    }

    Ok(None)
}
