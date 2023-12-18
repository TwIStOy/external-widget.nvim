use tree_sitter::{Language, Query};

const NVIM_TREE_SITTER_PATH: &str = "/.local/share/nvim/lazy/nvim-treesitter";

fn get_treesitter_dir(ty: &str) -> anyhow::Result<String> {
    let lib_path = format!(
        "{}/{}/{}",
        std::env::var("HOME")?,
        NVIM_TREE_SITTER_PATH,
        ty
    );
    Ok(lib_path)
}

pub fn load_lang(lang: &str) -> anyhow::Result<Language> {
    let lang = lang.to_lowercase();
    let lib_path = format!("{}/{}.so", get_treesitter_dir("parser")?, lang);
    let func_name = format!("tree_sitter_{}", lang.replace('-', "_"));
    unsafe {
        let lib = libloading::Library::new(lib_path)?;
        let entry = lib
            .get::<unsafe extern "C" fn() -> Language>(func_name.as_bytes())?;
        let language = entry();
        std::mem::forget(lib);
        Ok(language)
    }
}

pub fn get_parser(lang: &str) -> anyhow::Result<tree_sitter::Parser> {
    let lang = load_lang(lang)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(lang)?;
    Ok(parser)
}

pub fn get_query(lang: &str, query: &str) -> anyhow::Result<Query> {
    let query_path =
        format!("{}/{}/{}.scm", get_treesitter_dir("queries")?, lang, query);
    let lang = load_lang(lang)?;
    let query = std::fs::read_to_string(query_path)?;
    let query = tree_sitter::Query::new(lang, &query)?;
    Ok(query)
}
