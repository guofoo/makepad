/// System font provider trait and types for accessing operating system fonts.
///
/// This module provides a cross-platform interface for querying system fonts.
/// Each platform implements the `SystemFontProvider` trait to access fonts
/// installed on the operating system.

use std::fmt;

/// Data returned from a system font query.
#[derive(Clone)]
pub struct SystemFontData {
    /// The raw font file bytes (TTF/OTF format)
    pub data: Vec<u8>,
    /// Index within a font collection (TTC files), usually 0
    pub index: u32,
    /// The family name of the font
    pub family_name: String,
}

impl fmt::Debug for SystemFontData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemFontData")
            .field("data_len", &self.data.len())
            .field("index", &self.index)
            .field("family_name", &self.family_name)
            .finish()
    }
}

/// Errors that can occur when querying system fonts.
#[derive(Debug, Clone)]
pub enum SystemFontError {
    /// The requested font family was not found
    FontNotFound(String),
    /// Access to the font file was denied
    AccessDenied,
    /// System font access is not supported on this platform
    PlatformNotSupported,
    /// Failed to read the font file
    ReadError(String),
    /// The font file format is invalid or unsupported
    InvalidFontData,
}

impl fmt::Display for SystemFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemFontError::FontNotFound(name) => write!(f, "Font not found: {}", name),
            SystemFontError::AccessDenied => write!(f, "Access denied to font file"),
            SystemFontError::PlatformNotSupported => write!(f, "System fonts not supported on this platform"),
            SystemFontError::ReadError(msg) => write!(f, "Failed to read font: {}", msg),
            SystemFontError::InvalidFontData => write!(f, "Invalid or unsupported font data"),
        }
    }
}

impl std::error::Error for SystemFontError {}

/// Font style/weight hints for font queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Light,
    Medium,
    Thin,
}

impl FontStyle {
    /// Returns the style name as used by font APIs
    pub fn as_str(&self) -> &'static str {
        match self {
            FontStyle::Regular => "Regular",
            FontStyle::Bold => "Bold",
            FontStyle::Italic => "Italic",
            FontStyle::BoldItalic => "Bold Italic",
            FontStyle::Light => "Light",
            FontStyle::Medium => "Medium",
            FontStyle::Thin => "Thin",
        }
    }
}

/// Trait for platform-specific system font providers.
///
/// Implementations of this trait provide access to fonts installed on the
/// operating system. The font data returned is in a format suitable for
/// parsing with ttf-parser (raw TTF/OTF bytes).
pub trait SystemFontProvider: Send + Sync {
    /// Query a font by family name and optional style.
    ///
    /// # Arguments
    /// * `family` - The font family name (e.g., "Arial", "Helvetica")
    /// * `style` - Optional style hint (e.g., "Bold", "Italic")
    ///
    /// # Returns
    /// The font data if found, or an error if the font cannot be loaded.
    fn query_font(&self, family: &str, style: Option<FontStyle>) -> Result<SystemFontData, SystemFontError>;

    /// List all available font family names on the system.
    fn list_families(&self) -> Vec<String>;

    /// Get the system's default sans-serif font.
    fn default_sans(&self) -> Result<SystemFontData, SystemFontError>;

    /// Get the system's default monospace font.
    fn default_monospace(&self) -> Result<SystemFontData, SystemFontError>;

    /// Get the system's default serif font.
    fn default_serif(&self) -> Result<SystemFontData, SystemFontError>;

    /// Check if a font family is available on the system.
    fn has_family(&self, family: &str) -> bool {
        self.query_font(family, None).is_ok()
    }
}

/// Platform-specific fallback font chains.
pub mod fallbacks {
    /// macOS fallback chain for sans-serif fonts
    pub const MACOS_SANS: &[&str] = &["SF Pro", "SF Pro Text", "Helvetica Neue", "Helvetica", "Arial"];

    /// macOS fallback chain for monospace fonts
    pub const MACOS_MONO: &[&str] = &["SF Mono", "Menlo", "Monaco", "Courier New"];

    /// macOS fallback chain for serif fonts
    pub const MACOS_SERIF: &[&str] = &["New York", "Times New Roman", "Georgia"];

    /// Windows fallback chain for sans-serif fonts
    pub const WINDOWS_SANS: &[&str] = &["Segoe UI", "Arial", "Tahoma", "Verdana"];

    /// Windows fallback chain for monospace fonts
    pub const WINDOWS_MONO: &[&str] = &["Cascadia Mono", "Consolas", "Courier New"];

    /// Windows fallback chain for serif fonts
    pub const WINDOWS_SERIF: &[&str] = &["Cambria", "Times New Roman", "Georgia"];

    /// Linux fallback chain for sans-serif fonts
    pub const LINUX_SANS: &[&str] = &["Ubuntu", "DejaVu Sans", "Liberation Sans", "Noto Sans", "Arial"];

    /// Linux fallback chain for monospace fonts
    pub const LINUX_MONO: &[&str] = &["Ubuntu Mono", "DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"];

    /// Linux fallback chain for serif fonts
    pub const LINUX_SERIF: &[&str] = &["DejaVu Serif", "Liberation Serif", "Noto Serif", "Times New Roman"];
}

/// Helper function to try loading a font from a fallback chain.
pub fn try_fallback_chain<P: SystemFontProvider + ?Sized>(
    provider: &P,
    chain: &[&str],
    style: Option<FontStyle>,
) -> Result<SystemFontData, SystemFontError> {
    for family in chain {
        if let Ok(data) = provider.query_font(family, style) {
            return Ok(data);
        }
    }
    Err(SystemFontError::FontNotFound(
        chain.first().map(|s| s.to_string()).unwrap_or_default()
    ))
}
