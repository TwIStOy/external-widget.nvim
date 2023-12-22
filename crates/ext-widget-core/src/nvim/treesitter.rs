use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Context;
use libloading::Library;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use std::fmt::Write;
use tree_sitter::{Language, Parser};

use super::find_file_in_runtime_path;

static LIB_CACHE: Lazy<Mutex<TreeSitterLibraries>> =
    Lazy::new(|| Mutex::new(TreeSitterLibraries::new()));

static INHERITS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r";+\s*inherits\s*:?\s*([a-z_,()-]+)\s*").unwrap());

#[derive(Debug)]
struct TreeSitterLibraries {
    lib_cache: HashMap<PathBuf, Arc<Library>>,
    query_cache: HashMap<String, String>,
}

impl TreeSitterLibraries {
    fn new() -> Self {
        Self {
            lib_cache: HashMap::new(),
            query_cache: HashMap::new(),
        }
    }

    fn load_lib(
        &mut self, path: PathBuf,
    ) -> anyhow::Result<Option<Arc<Library>>> {
        if let Some(lib) = self.lib_cache.get(&path) {
            return Ok(Some(lib.clone()));
        }

        let file = find_file_in_runtime_path(&path)?;
        if file.is_none() {
            return Ok(None);
        }
        let file = unsafe { file.unwrap_unchecked() };
        unsafe {
            let lib = Arc::new(libloading::Library::new(file)?);
            self.lib_cache.insert(path, lib.clone());
            Ok(Some(lib))
        }
    }

    fn load_language(
        &mut self, lang: &str,
    ) -> anyhow::Result<Option<Language>> {
        let path = PathBuf::from_str(&format!("parser/{}.so", lang))?;
        let lib = self.load_lib(path)?;
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

    fn read_query_raw(
        &self, lang: &str, filename: &str,
    ) -> anyhow::Result<String> {
        let path =
            PathBuf::from_str(&format!("queries/{}/{}.scm", lang, filename))
                .unwrap();
        let query_file = find_file_in_runtime_path(path)?;
        if query_file.is_none() {
            return Ok(String::new());
        }
        let query_file = query_file.unwrap();
        Ok(std::fs::read_to_string(query_file)?)
    }

    fn read_query(&mut self, lang: &str, filename: &str) -> String {
        let key = format!("{}/{}", lang, filename);
        if let Some(query) = self.query_cache.get(&key) {
            return query.clone();
        }

        let query = self.read_query_raw(lang, filename).unwrap_or_default();

        // replaces all "; inherits <language>(,<language>)*" with the queries of the given language(s)
        let ret = INHERITS_REGEX
            .replace_all(&query, |captures: &regex::Captures| {
                captures[1].split(',').fold(
                    String::new(),
                    |mut output, language| {
                        let _ = write!(
                            output,
                            "\n{}\n",
                            self.read_query(language, filename)
                        );
                        output
                    },
                )
            })
            .to_string();

        self.query_cache.insert(key, ret.clone());

        ret
    }
}

/// Load a language parser.
pub fn load_ts_parser(lang: &str) -> anyhow::Result<Parser> {
    let lang = LIB_CACHE
        .lock()
        .load_language(lang)?
        .context("No language parser")?;
    let mut parser = Parser::new();
    parser.set_language(lang)?;
    Ok(parser)
}

pub fn load_ts_query(
    lang: &str, query: &str,
) -> anyhow::Result<tree_sitter::Query> {
    let mut cache = LIB_CACHE.lock();
    let query = cache.read_query(lang, query);
    let lang = cache.load_language(lang)?.context("No language parser")?;
    let query = tree_sitter::Query::new(lang, &query)?;
    Ok(query)
}
