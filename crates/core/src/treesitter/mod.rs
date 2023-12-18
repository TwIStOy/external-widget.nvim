use regex::Regex;
use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;
use tree_sitter::Language;

const DEFAULT_NVIM_TREEISTTER_DIR: &str =
    "/.local/share/nvim/lazy/nvim-treesitter";

pub struct TreeSitter {
    parsers: HashMap<String, Language>,
    runtime_dir: String,
}

static INHERITS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r";+\s*inherits\s*:?\s*([a-z_,()-]+)\s*").unwrap());

pub static TREE_SITTER: Lazy<Mutex<TreeSitter>> =
    Lazy::new(|| Mutex::new(TreeSitter::new()));

impl TreeSitter {
    pub fn new() -> Self {
        // use nvim-treesitter installed in lazy.nvim
        Self {
            parsers: HashMap::new(),
            runtime_dir: format!(
                "{}/{}",
                std::env::var("HOME").unwrap(),
                DEFAULT_NVIM_TREEISTTER_DIR
            ),
        }
    }

    pub fn read_query(&self, language: &str, filename: &str) -> String {
        let query = self
            .load_runtime_query(language, filename)
            .unwrap_or_default();

        // replaces all "; inherits <language>(,<language>)*" with the queries of the given language(s)
        INHERITS_REGEX
            .replace_all(&query, |captures: &regex::Captures| {
                captures[1]
                    .split(',')
                    .map(|language| {
                        format!("\n{}\n", self.read_query(language, filename))
                    })
                    .collect::<String>()
            })
            .to_string()
    }

    fn load_runtime_query(
        &self, language: &str, filename: &str,
    ) -> anyhow::Result<String> {
        let path = format!(
            "{}/{}/{}.scm",
            self.get_dir("queries"),
            language,
            filename
        );
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn get_lang(&mut self, lang: &str) -> anyhow::Result<Language> {
        if let Some(lang) = self.parsers.get(lang) {
            return Ok(*lang);
        }
        let name = lang;
        let lang = self.load_lang(lang)?;
        self.parsers.insert(name.to_string(), lang);
        Ok(lang)
    }

    pub fn get_parser(
        &mut self, lang: &str,
    ) -> anyhow::Result<tree_sitter::Parser> {
        let lang = self.get_lang(lang)?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(lang)?;
        Ok(parser)
    }

    pub fn get_query(
        &mut self, lang: &str, query: &str,
    ) -> anyhow::Result<tree_sitter::Query> {
        let query = self.read_query(lang, query);
        let lang = self.get_lang(lang)?;
        let query = tree_sitter::Query::new(lang, &query)?;
        Ok(query)
    }

    fn get_dir(&self, ty: &str) -> String {
        let lib_path = format!("{}/{}", self.runtime_dir, ty);
        lib_path
    }

    fn load_lang(&self, lang: &str) -> anyhow::Result<Language> {
        let lang = lang.to_lowercase();
        let lib_path = format!("{}/{}.so", self.get_dir("parser"), lang);
        let func_name = format!("tree_sitter_{}", lang.replace('-', "_"));
        unsafe {
            let lib = libloading::Library::new(lib_path)?;
            let entry = lib.get::<unsafe extern "C" fn() -> Language>(
                func_name.as_bytes(),
            )?;
            let language = entry();
            std::mem::forget(lib);
            Ok(language)
        }
    }
}

impl Default for TreeSitter {
    fn default() -> Self {
        Self::new()
    }
}
