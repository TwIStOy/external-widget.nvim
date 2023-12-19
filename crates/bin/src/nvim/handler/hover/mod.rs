mod hover_doc;

use anyhow::bail;
use external_widget_core::{nvim::Nvim, TermWriter, kitty::transmit_image};
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
    let mut writer = TermWriter::new().await?;
    let id = ID(10.try_into().unwrap());
    transmit_image(&image, &mut writer, id).await?;
    sleep(Duration::from_millis(1000)).await;
    display_image(&mut writer, id).await?;
    Ok(())
}
