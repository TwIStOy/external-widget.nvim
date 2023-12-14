use cosmic_text::FontSystem;

/// Create a new font system with the given font families.
pub fn new_font_system<RS, MS>(
    regular_font_family: RS, monospace_family: MS,
) -> FontSystem
where
    RS: Into<String>,
    MS: Into<String>,
{
    let mut font_system = FontSystem::new();

    font_system
        .db_mut()
        .set_sans_serif_family(regular_font_family);
    font_system.db_mut().set_monospace_family(monospace_family);

    font_system
}
