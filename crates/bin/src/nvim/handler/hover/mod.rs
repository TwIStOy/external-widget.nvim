mod hover_doc;

use anyhow::bail;
use external_widget_core::{
    kitty::transmit_image, nvim::Nvim, Image, TermWriter,
};
pub use hover_doc::build_hover_doc_image;
use rmpv::Value;

pub async fn handle_hover_notify(
    args: Vec<Value>, nvim: Nvim,
) -> anyhow::Result<()> {
    if args.len() != 2 {
        bail!("hover expects 2 arguments, got {}", args.len());
    }
    let md = args[0].as_str().unwrap_or_default();
    let lang = args[1].as_str().unwrap_or_default();
    println!("md: {}, lang: {}", md, lang);
    let image = build_hover_doc_image(&nvim, md.to_string(), lang).await?;
    let image = Image::from_buffer(image);
    Ok(())
}
