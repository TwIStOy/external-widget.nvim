use std::fs;

use anyhow::Result;

use headless_chrome::{
    protocol::cdp::Page::CaptureScreenshotFormatOption, Browser, LaunchOptions,
};

fn main() -> Result<()> {
    // Create a headless browser, navigate to wikipedia.org, wait for the page
    // to render completely, take a screenshot of the entire page
    // in JPEG-format using 75% quality.
    let options = LaunchOptions::default_builder()
        .build()
        .expect("Couldn't find appropriate Chrome binary.");
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    let tab = tab
        .navigate_to("https://www.wikipedia.org")?
        .wait_until_navigated()?;
    // current timestamp
    let st = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let jpeg_data = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Jpeg,
        Some(75),
        None,
        true,
    )?;
    let ed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    println!("time: {}", ed - st);
    fs::write("screenshot.jpg", jpeg_data)?;

    // Browse to the WebKit-Page and take a screenshot of the infobox.
    let element = tab
        .navigate_to("https://en.wikipedia.org/wiki/WebKit")?
        .wait_for_element("#mw-content-text > div > table.infobox.vevent")?;
    let st = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let png_data =
        element.capture_screenshot(CaptureScreenshotFormatOption::Png)?;
    let ed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    println!("time: {}", ed - st);
    fs::write("screenshot.png", png_data)?;

    println!("Screenshots successfully created.");
    Ok(())
}
