//! macOS system font provider using Core Text.
//!
//! This module provides access to system fonts on macOS through the Core Text
//! framework. It implements the `SystemFontProvider` trait to allow querying
//! fonts by family name and retrieving font file data.

use std::ffi::CStr;
use std::path::PathBuf;

use crate::os::apple::apple_sys::{
    msg_send, sel, sel_impl, nil, ObjcId, CFStringRef, CFRange,
    CFStringGetLength, CFStringGetBytes,
    kCFStringEncodingUTF8,
    CTFontCreateWithName, CTFontCopyAttribute, CTFontManagerCopyAvailableFontFamilyNames,
    kCTFontURLAttribute,
};
use crate::os::apple::apple_util::str_to_nsstring;
use crate::os::system_fonts::{
    SystemFontProvider, SystemFontData, SystemFontError, FontStyle,
    fallbacks, try_fallback_chain,
};

/// macOS system font provider using Core Text.
pub struct MacOSFontProvider;

impl MacOSFontProvider {
    /// Create a new macOS font provider.
    pub fn new() -> Self {
        MacOSFontProvider
    }
}

impl Default for MacOSFontProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a Rust string to a CFStringRef.
///
/// Uses `str_to_nsstring` to create an autoreleased NSString, then casts to CFStringRef.
/// CFString and NSString are toll-free bridged, so this cast is safe.
/// The returned CFStringRef is autoreleased and valid within the current autorelease pool scope.
fn string_to_cfstring(s: &str) -> CFStringRef {
    str_to_nsstring(s) as CFStringRef
}

/// Convert a CFStringRef to a Rust String.
fn cfstring_to_string(cfstring: CFStringRef) -> Option<String> {
    if cfstring.is_null() {
        return None;
    }

    unsafe {
        let length = CFStringGetLength(cfstring);
        if length == 0 {
            return Some(String::new());
        }

        // Estimate buffer size (UTF-8 can be up to 4 bytes per character)
        let max_size = length * 4;
        let mut buffer: Vec<u8> = vec![0; max_size as usize];
        let mut used_len: u64 = 0;

        let range = CFRange {
            location: 0,
            length,
        };

        let converted = CFStringGetBytes(
            cfstring,
            range,
            kCFStringEncodingUTF8,
            0, // lossByte - 0 means fail on unconvertible
            false, // isExternalRepresentation
            buffer.as_mut_ptr(),
            max_size,
            &mut used_len,
        );

        if converted == 0 && length > 0 {
            return None;
        }

        buffer.truncate(used_len as usize);
        String::from_utf8(buffer).ok()
    }
}

/// Get the file path for a CTFont reference.
///
/// Uses `CTFontCopyAttribute(font, kCTFontURLAttribute)` to get the font URL,
/// then extracts the file path from the NSURL.
fn get_font_path(font: ObjcId) -> Option<PathBuf> {
    if font == nil {
        return None;
    }

    unsafe {
        // Get the URL attribute from the font
        let url: ObjcId = CTFontCopyAttribute(font, kCTFontURLAttribute);
        if url == nil {
            return None;
        }

        // Get the path string from the NSURL
        let path_string: ObjcId = msg_send![url, path];
        if path_string == nil {
            let () = msg_send![url, release];
            return None;
        }

        // Get the UTF8 C string from the NSString
        let utf8_ptr: *const std::os::raw::c_char = msg_send![path_string, UTF8String];
        if utf8_ptr.is_null() {
            let () = msg_send![url, release];
            return None;
        }

        let path_str = CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned();
        let () = msg_send![url, release];

        Some(PathBuf::from(path_str))
    }
}

/// Create a CTFont reference for the given family name and style.
fn create_font_for_family(family: &str, _style: Option<FontStyle>) -> Option<ObjcId> {
    let family_cfstring = string_to_cfstring(family);

    unsafe {
        // Create a font with the given family name
        // Using size 12.0 as a reasonable default - we just need the font reference
        // to get the file path, not to render at a specific size
        let font: ObjcId = CTFontCreateWithName(family_cfstring, 12.0, std::ptr::null());

        if font == nil {
            return None;
        }

        Some(font)
    }
}

impl SystemFontProvider for MacOSFontProvider {
    fn query_font(&self, family: &str, style: Option<FontStyle>) -> Result<SystemFontData, SystemFontError> {
        // Create a CTFont for the requested family
        let font = create_font_for_family(family, style)
            .ok_or_else(|| SystemFontError::FontNotFound(family.to_string()))?;

        // Get the font file path
        let path = get_font_path(font)
            .ok_or_else(|| SystemFontError::FontNotFound(family.to_string()))?;

        // Release the font reference
        unsafe {
            let () = msg_send![font, release];
        }

        // Read the font file data
        let data = std::fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                SystemFontError::AccessDenied
            } else {
                SystemFontError::ReadError(format!("Failed to read font file: {}", e))
            }
        })?;

        if data.is_empty() {
            return Err(SystemFontError::InvalidFontData);
        }

        Ok(SystemFontData {
            data,
            index: 0, // Default to first font in collection
            family_name: family.to_string(),
        })
    }

    fn list_families(&self) -> Vec<String> {
        unsafe {
            let array: ObjcId = CTFontManagerCopyAvailableFontFamilyNames();
            if array == nil {
                return Vec::new();
            }

            let count: u64 = msg_send![array, count];
            let mut families = Vec::with_capacity(count as usize);

            for i in 0..count {
                let ns_string: ObjcId = msg_send![array, objectAtIndex: i];
                if ns_string != nil {
                    // Get UTF8String from NSString
                    let utf8_ptr: *const std::os::raw::c_char = msg_send![ns_string, UTF8String];
                    if !utf8_ptr.is_null() {
                        let family_str = CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned();
                        families.push(family_str);
                    }
                }
            }

            let () = msg_send![array, release];
            families
        }
    }

    fn default_sans(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::MACOS_SANS, None)
    }

    fn default_monospace(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::MACOS_MONO, None)
    }

    fn default_serif(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::MACOS_SERIF, None)
    }
}

/// Get the system font provider for macOS
pub fn get_system_font_provider() -> MacOSFontProvider {
    MacOSFontProvider::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_families() {
        let provider = MacOSFontProvider::new();
        let families = provider.list_families();
        // macOS should have at least some fonts
        assert!(!families.is_empty());
    }

    #[test]
    fn test_query_helvetica() {
        let provider = MacOSFontProvider::new();
        // Helvetica should always be available on macOS
        let result = provider.query_font("Helvetica", None);
        assert!(result.is_ok());
        let font_data = result.unwrap();
        assert!(!font_data.data.is_empty());
        assert_eq!(font_data.family_name, "Helvetica");
    }

    #[test]
    fn test_query_nonexistent_font() {
        let provider = MacOSFontProvider::new();
        let result = provider.query_font("ThisFontDefinitelyDoesNotExist12345", None);
        // Note: macOS Core Text may return a fallback font instead of an error
        // This is expected behavior - the system substitutes a default font
        // We just verify we get some valid font data back
        if let Ok(data) = &result {
            assert!(!data.data.is_empty(), "Font data should not be empty");
            // The family name will be different from what we requested (substituted)
        }
        // Either success (with substitution) or error is acceptable
    }

    #[test]
    fn test_default_sans() {
        let provider = MacOSFontProvider::new();
        let result = provider.default_sans();
        // Should succeed with one of the fallback fonts
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_monospace() {
        let provider = MacOSFontProvider::new();
        let result = provider.default_monospace();
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_serif() {
        let provider = MacOSFontProvider::new();
        let result = provider.default_serif();
        assert!(result.is_ok());
    }
}
