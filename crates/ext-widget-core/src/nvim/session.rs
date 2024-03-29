use std::{
    collections::HashMap,
    fmt::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::{bail, Context};
use async_recursion::async_recursion;
use futures::AsyncWrite;
use libloading::Library;
use nvim_rs::Neovim;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use rmpv::ext::from_value;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tree_sitter::{Language, Parser};

use crate::{env::in_tmux, term::TermWriter};

use super::HighlightInfos;

/**
 * A session with Neovim. This will be saved in the `session` field in the
 * NvimHandler. Do some caching here.
 */
#[derive(Debug)]
pub struct NeovimSession {
    pub ts_libs: Mutex<HashMap<PathBuf, Arc<Library>>>,
    pub ts_queries: Mutex<HashMap<String, String>>,
    pub tty_writer: Mutex<Option<Arc<tokio::sync::Mutex<TermWriter>>>>,
}

static INHERITS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r";+\s*inherits\s*:?\s*([a-z_,()-]+)\s*").unwrap());

impl NeovimSession {
    pub fn new() -> Self {
        Self {
            ts_libs: Mutex::new(HashMap::new()),
            ts_queries: Mutex::new(HashMap::new()),
            tty_writer: Mutex::new(None),
        }
    }

    async fn get_highlight_info_impl<W>(
        &self, nvim: &Neovim<W>, name: &str,
    ) -> anyhow::Result<HighlightInfos>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let res = nvim.get_hl(0, vec![("name".into(), name.into())]).await?;
        let res = rmpv::Value::Map(res);
        let def: HighlightInfos = from_value(res)?;
        Ok(def)
    }

    pub async fn get_highlight_info<W>(
        &self, nvim: &Neovim<W>, name: &str,
    ) -> anyhow::Result<HighlightInfos>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let mut name = name.to_string();
        loop {
            let res = self.get_highlight_info_impl(nvim, &name).await?;
            if let Some(link) = res.link {
                name = link;
            } else {
                return Ok(res);
            }
        }
    }

    pub fn get_highlight_info_sync<W>(
        self: Arc<Self>, nvim: Neovim<W>, name: &str,
    ) -> anyhow::Result<HighlightInfos>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let name = name.to_string();
        tokio::spawn(async move {
            let res = self.get_highlight_info(&nvim, &name).await;
            let _ = sender.send(res);
        });
        futures::executor::block_on(receiver)
            .context("Failed to get highlight info")?
    }

    pub async fn load_ts_parser<W>(
        &self, nvim: &Neovim<W>, lang: &str,
    ) -> anyhow::Result<Option<Parser>>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let lang = self
            .load_language(nvim, lang)
            .await?
            .context("No language parser")?;
        let mut parser = Parser::new();
        parser.set_language(lang)?;
        Ok(Some(parser))
    }

    pub async fn load_ts_query<W>(
        &self, nvim: &Neovim<W>, lang: &str, query: &str,
    ) -> anyhow::Result<tree_sitter::Query>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let query = self.read_query(nvim, lang, query).await;
        let lang = self
            .load_language(nvim, lang)
            .await?
            .context("No language parser")?;
        let query = tree_sitter::Query::new(lang, &query)?;
        Ok(query)
    }

    async fn load_language<W>(
        &self, nvim: &Neovim<W>, lang: &str,
    ) -> anyhow::Result<Option<Language>>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let path = PathBuf::from_str(&format!("parser/{}.so", lang))?;
        let lib = self.load_lib(nvim, path).await?;
        if lib.is_none() {
            return Ok(None);
        }
        let lib = lib.unwrap();
        let func_name = format!("tree_sitter_{}", lang.replace('-', "_"));
        unsafe {
            let entry = lib.get::<unsafe extern "C" fn() -> Language>(
                func_name.as_bytes(),
            )?;
            let parser = entry();
            Ok(Some(parser))
        }
    }

    async fn load_lib<W>(
        &self, nvim: &Neovim<W>, path: PathBuf,
    ) -> anyhow::Result<Option<Arc<Library>>>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        {
            if let Some(lib) = self.ts_libs.lock().get(&path) {
                return Ok(Some(lib.clone()));
            }
        }

        let file = Self::find_file_in_runtime_path(nvim, &path).await?;
        if file.is_none() {
            return Ok(None);
        }
        let file = unsafe { file.unwrap_unchecked() };
        unsafe {
            let lib = Arc::new(libloading::Library::new(file)?);
            self.ts_libs.lock().insert(path, lib.clone());
            Ok(Some(lib))
        }
    }

    #[async_recursion]
    async fn read_query<W>(
        &self, nvim: &Neovim<W>, lang: &str, filename: &str,
    ) -> String
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let key = format!("{}/{}", lang, filename);
        if let Some(query) = self.ts_queries.lock().get(&key) {
            return query.clone();
        }

        let query = self
            .read_query_raw(nvim, lang, filename)
            .await
            .unwrap_or_default();

        // replaces all "; inherits <language>(,<language>)*" with the queries of the given language(s)
        let replaced = match INHERITS_REGEX.captures(&query) {
            Some(captures) => {
                let mut ret = String::new();
                let parts = captures.get(1).unwrap().as_str();
                for part in parts.split(',') {
                    let _ = write!(
                        &mut ret,
                        "\n{}\n",
                        self.read_query(nvim, part, filename).await
                    );
                }
                ret
            }
            None => "".to_string(),
        };
        let ret = INHERITS_REGEX
            .replace_all(&query, replaced.as_str())
            .to_string();

        {
            self.ts_queries.lock().insert(key, ret.clone());
        }

        ret
    }

    async fn read_query_raw<W>(
        &self, nvim: &Neovim<W>, lang: &str, filename: &str,
    ) -> anyhow::Result<String>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let path =
            PathBuf::from_str(&format!("queries/{}/{}.scm", lang, filename))
                .unwrap();
        let query_file = Self::find_file_in_runtime_path(nvim, path).await?;
        if query_file.is_none() {
            return Ok(String::new());
        }
        let query_file = query_file.unwrap();
        Ok(std::fs::read_to_string(query_file)?)
    }

    /**
     * Find a file in the runtime path of Neovim.
     *
     * NO CACHING
     */
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

    pub async fn get_tty<W>(nvim: &Neovim<W>) -> anyhow::Result<String>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let ret = nvim
            .exec_lua(
                r#"return io.popen("tty 2>/dev/null"):read("*a")"#,
                vec![],
            )
            .await?;
        match ret {
            rmpv::Value::String(ret) => {
                let ret =
                    ret.as_str().context("should str")?.trim().to_string();
                Ok(ret)
            }
            _ => bail!("unexpected return value, {}", ret),
        }
    }

    #[instrument(skip(self, nvim))]
    pub async fn post_instance<W>(&self, nvim: &Neovim<W>) -> anyhow::Result<()>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        {
            let tty_writer = self.tty_writer.lock();
            if tty_writer.is_some() {
                return Ok(());
            }
        }
        let tty = Self::get_tty(nvim).await?;
        let new_writer = TermWriter::new_tmux_tty(&tty, in_tmux()).await?;

        let mut tty_writer = self.tty_writer.lock();
        if tty_writer.is_some() {
            return Ok(());
        }
        *tty_writer = Some(Arc::new(tokio::sync::Mutex::new(new_writer)));
        Ok(())
    }

    pub async fn get_tty_writer<W>(
        &self, nvim: &Neovim<W>,
    ) -> anyhow::Result<Arc<tokio::sync::Mutex<TermWriter>>>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        self.post_instance(nvim).await?;
        let tty_writer = self.tty_writer.lock();
        Ok(tty_writer.clone().unwrap())
    }

    pub async fn cursor_position_to_client<W>(
        nvim: &Neovim<W>,
    ) -> anyhow::Result<(i32, i32)>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let cur_win = nvim.get_current_win().await?;
        let (row, col) = cur_win.get_cursor().await?;
        let win_pos = cur_win.get_position().await?;
        let line_start: i64 =
            from_value(nvim.call_function("line", vec!["w0".into()]).await?)?;

        let row = row - line_start + win_pos.0;
        let col = col + win_pos.1;

        Ok((row as i32, col as i32))
    }

    pub async fn get_term_size<W>(
        nvim: &Neovim<W>,
    ) -> anyhow::Result<NvimTermSize>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let ret = nvim
            .exec_lua(
                r#"return require("external-widget.utils").get_term_size()"#,
                vec![],
            )
            .await?;
        let ret: NvimTermSize = rmpv::ext::from_value(ret)?;
        Ok(ret)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvimTermSize {
    pub row: u32,
    pub col: u32,
    pub xpixel: u32,
    pub ypixel: u32,
}

impl Default for NeovimSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{nvim::NeovimSession, test_utils::EmbedNvim};

    #[tokio::test]
    async fn test_get_normal_highlight() -> anyhow::Result<()> {
        let embed_nvim = EmbedNvim::new().await?;
        let session = NeovimSession::new();
        let hl = session
            .get_highlight_info(&embed_nvim.neovim, "Normal")
            .await?;
        println!("{:?}", hl);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_tty() -> anyhow::Result<()> {
        let embed_nvim = EmbedNvim::new().await?;
        let tty = NeovimSession::get_tty(&embed_nvim.neovim).await?;
        println!("tty: {}", tty);
        Ok(())
    }
}
