mod highlight;
mod rtp;
mod treesitter;

pub use highlight::nvim_highlight_into_text_style;
pub use rtp::find_file_in_runtime_path;
pub use treesitter::{load_ts_parser, load_ts_query};
