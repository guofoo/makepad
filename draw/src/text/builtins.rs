use super::loader::{FontDefinition, FontFamilyDefinition, Loader};

#[cfg(feature = "bundled-fonts")]
pub const IBM_PLEX_SANS_TEXT: &[u8] =
    include_bytes!("../../../widgets/resources/IBMPlexSans-Text.ttf");
#[cfg(feature = "bundled-fonts")]
pub const LXG_WEN_KAI_REGULAR: &[u8] =
    include_bytes!("../../../widgets/resources/LXGWWenKaiRegular.ttf");
#[cfg(feature = "bundled-fonts")]
pub const NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../../../widgets/resources/NotoColorEmoji.ttf");
#[cfg(feature = "bundled-fonts")]
pub const LIBERATION_MONO_REGULAR: &[u8] =
    include_bytes!("../../../widgets/resources/LiberationMono-Regular.ttf");

#[cfg(feature = "bundled-fonts")]
pub fn define(loader: &mut Loader) {
    loader.define_font_family(
        "Sans".into(),
        FontFamilyDefinition {
            font_ids: [
                "IBM Plex Sans Text".into(),
                "LXG WWen Kai Regular".into(),
                "Noto Color Emoji".into(),
            ]
            .into(),
        },
    );
    loader.define_font_family(
        "Monospace".into(),
        FontFamilyDefinition {
            font_ids: ["Liberation Mono Regular".into()].into(),
        },
    );
    loader.define_font(
        "IBM Plex Sans Text".into(),
        FontDefinition::from_data(IBM_PLEX_SANS_TEXT.to_vec().into(), 0),
    );
    loader.define_font(
        "LXG WWen Kai Regular".into(),
        FontDefinition::from_data(LXG_WEN_KAI_REGULAR.to_vec().into(), 0),
    );
    loader.define_font(
        "Noto Color Emoji".into(),
        FontDefinition::from_data(NOTO_COLOR_EMOJI.to_vec().into(), 0),
    );
    loader.define_font(
        "Liberation Mono Regular".into(),
        FontDefinition::from_data(LIBERATION_MONO_REGULAR.to_vec().into(), 0),
    );
}

#[cfg(all(feature = "system-fonts", not(feature = "bundled-fonts")))]
pub fn define(loader: &mut Loader) {
    // Define font families using system fonts
    loader.define_font_family(
        "Sans".into(),
        FontFamilyDefinition {
            font_ids: ["System Sans".into()].into(),
        },
    );
    loader.define_font_family(
        "Monospace".into(),
        FontFamilyDefinition {
            font_ids: ["System Mono".into()].into(),
        },
    );

    // Platform-specific system font definitions
    #[cfg(target_os = "macos")]
    {
        loader.define_font("System Sans".into(), FontDefinition::from_system("Helvetica Neue"));
        loader.define_font("System Mono".into(), FontDefinition::from_system("Menlo"));
    }

    #[cfg(target_os = "windows")]
    {
        loader.define_font("System Sans".into(), FontDefinition::from_system("Segoe UI"));
        loader.define_font("System Mono".into(), FontDefinition::from_system("Consolas"));
    }

    #[cfg(target_os = "linux")]
    {
        loader.define_font("System Sans".into(), FontDefinition::from_system("DejaVu Sans"));
        loader.define_font("System Mono".into(), FontDefinition::from_system("DejaVu Sans Mono"));
    }
}

#[cfg(not(any(feature = "bundled-fonts", feature = "system-fonts")))]
pub fn define(_loader: &mut Loader) {
    // No fonts defined - user must define their own
}
