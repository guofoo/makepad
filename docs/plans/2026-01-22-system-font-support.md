# System Font Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable Makepad to use operating system fonts instead of bundled fonts, reducing binary size from ~47MB to minimal.

**Architecture:** Platform-specific font providers (macOS/Windows/Linux) implement a common `SystemFontProvider` trait that returns raw font bytes. The draw crate's `Loader` gains a `FontSource` enum to distinguish embedded vs system fonts, with lazy loading on first use. Feature flags control bundled vs system font compilation.

**Tech Stack:** Rust, Core Text (macOS), DirectWrite (Windows), Fontconfig (Linux), ttf-parser

---

## Current State

- ✅ `platform/src/os/system_fonts.rs` - Trait definition exists
- ✅ `platform/src/os/apple/apple_sys.rs` - Core Text FFI link added (partial)
- ⬜ macOS implementation
- ⬜ Windows implementation
- ⬜ Linux implementation
- ⬜ Loader FontSource integration
- ⬜ Feature flags
- ⬜ Conditional builtins

---

## Task 1: Complete Core Text FFI

**Files:**
- Modify: `platform/src/os/apple/apple_sys.rs:1209-1220`

**Step 1: Verify current Core Text FFI**

Run: `grep -A 20 "CoreText" platform/src/os/apple/apple_sys.rs`

Expected: See `#[link(name = "CoreText"...)]` block

**Step 2: Ensure complete Core Text declarations**

The FFI block should contain:

```rust
// Core Text API for system fonts

#[link(name = "CoreText", kind = "framework")]
extern "C" {
    pub fn CTFontCreateWithName(name: CFStringRef, size: f64, matrix: *const ()) -> ObjcId;
    pub fn CTFontCreateCopyWithAttributes(font: ObjcId, size: f64, matrix: *const (), attributes: ObjcId) -> ObjcId;
    pub fn CTFontCopyAttribute(font: ObjcId, attribute: CFStringRef) -> ObjcId;
    pub fn CTFontCopyFamilyName(font: ObjcId) -> CFStringRef;
    pub fn CTFontCopyFullName(font: ObjcId) -> CFStringRef;
    pub fn CTFontManagerCopyAvailableFontFamilyNames() -> ObjcId;

    pub static kCTFontURLAttribute: CFStringRef;
    pub static kCTFontFamilyNameAttribute: CFStringRef;
    pub static kCTFontStyleNameAttribute: CFStringRef;
}
```

**Step 3: Build to verify FFI compiles**

Run: `cargo build -p makepad-platform --target aarch64-apple-darwin 2>&1 | head -30`

Expected: No errors related to CoreText symbols

**Step 4: Commit**

```bash
git add platform/src/os/apple/apple_sys.rs
git commit -m "feat(platform): add Core Text FFI for system font access"
```

---

## Task 2: Create macOS System Font Provider

**Files:**
- Create: `platform/src/os/apple/system_fonts.rs`
- Modify: `platform/src/os/apple/mod.rs`

**Step 2.1: Create the macOS provider file**

Create `platform/src/os/apple/system_fonts.rs`:

```rust
//! macOS system font provider using Core Text.

use crate::os::system_fonts::{
    fallbacks, try_fallback_chain, FontStyle, SystemFontData, SystemFontError, SystemFontProvider,
};
use super::apple_sys::*;
use std::ffi::CStr;
use std::fs;
use std::path::PathBuf;

/// macOS system font provider using Core Text framework.
pub struct MacOSFontProvider;

impl MacOSFontProvider {
    pub fn new() -> Self {
        Self
    }

    /// Convert CFString to Rust String
    unsafe fn cfstring_to_string(cf_string: CFStringRef) -> Option<String> {
        if cf_string.is_null() {
            return None;
        }
        let length = CFStringGetLength(cf_string);
        if length == 0 {
            return Some(String::new());
        }
        let mut buffer = vec![0u8; (length * 4) as usize];
        let mut used = 0u64;
        CFStringGetBytes(
            cf_string,
            CFRange { location: 0, length },
            kCFStringEncodingUTF8,
            0,
            false,
            buffer.as_mut_ptr(),
            buffer.len() as u64,
            &mut used,
        );
        buffer.truncate(used as usize);
        String::from_utf8(buffer).ok()
    }

    /// Create CFString from Rust &str
    unsafe fn string_to_cfstring(s: &str) -> CFStringRef {
        let cstr = std::ffi::CString::new(s).unwrap();
        __CFStringMakeConstantString(cstr.as_ptr())
    }

    /// Get font file path from CTFont
    unsafe fn get_font_path(font: ObjcId) -> Option<PathBuf> {
        if font.is_null() {
            return None;
        }
        let url: ObjcId = CTFontCopyAttribute(font, kCTFontURLAttribute);
        if url.is_null() {
            return None;
        }
        // Get path from NSURL
        let path: ObjcId = msg_send![url, path];
        if path.is_null() {
            let _: () = msg_send![url, release];
            return None;
        }
        let utf8: *const i8 = msg_send![path, UTF8String];
        let path_str = CStr::from_ptr(utf8).to_string_lossy().into_owned();
        let _: () = msg_send![url, release];
        Some(PathBuf::from(path_str))
    }
}

impl Default for MacOSFontProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemFontProvider for MacOSFontProvider {
    fn query_font(&self, family: &str, _style: Option<FontStyle>) -> Result<SystemFontData, SystemFontError> {
        unsafe {
            let name = Self::string_to_cfstring(family);
            let font = CTFontCreateWithName(name, 12.0, std::ptr::null());

            if font.is_null() {
                return Err(SystemFontError::FontNotFound(family.to_string()));
            }

            // Get the actual family name (may differ from requested)
            let actual_family = CTFontCopyFamilyName(font);
            let family_name = Self::cfstring_to_string(actual_family)
                .unwrap_or_else(|| family.to_string());
            if !actual_family.is_null() {
                // CFRelease equivalent - CFStringRef is toll-free bridged
                let _: () = msg_send![actual_family as ObjcId, release];
            }

            // Get font file path
            let path = Self::get_font_path(font).ok_or_else(|| {
                let _: () = msg_send![font, release];
                SystemFontError::AccessDenied
            })?;

            let _: () = msg_send![font, release];

            // Read font file
            let data = fs::read(&path).map_err(|e| {
                SystemFontError::ReadError(format!("{}: {}", path.display(), e))
            })?;

            Ok(SystemFontData {
                data,
                index: 0, // TODO: Handle TTC collections
                family_name,
            })
        }
    }

    fn list_families(&self) -> Vec<String> {
        unsafe {
            let families: ObjcId = CTFontManagerCopyAvailableFontFamilyNames();
            if families.is_null() {
                return Vec::new();
            }

            let count: u64 = msg_send![families, count];
            let mut result = Vec::with_capacity(count as usize);

            for i in 0..count {
                let name: ObjcId = msg_send![families, objectAtIndex: i];
                if let Some(s) = Self::cfstring_to_string(name as CFStringRef) {
                    result.push(s);
                }
            }

            let _: () = msg_send![families, release];
            result
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
```

**Step 2.2: Add module to apple/mod.rs**

Add to `platform/src/os/apple/mod.rs` after `pub mod apple_sys;`:

```rust
#[cfg(feature = "system-fonts")]
pub mod system_fonts;
```

**Step 2.3: Build to verify compilation**

Run: `cargo build -p makepad-platform --target aarch64-apple-darwin --features system-fonts 2>&1 | head -50`

Note: Will fail until feature flag is added. Continue to Task 5 for feature flags, then return.

**Step 2.4: Commit**

```bash
git add platform/src/os/apple/system_fonts.rs platform/src/os/apple/mod.rs
git commit -m "feat(platform): add macOS system font provider using Core Text"
```

---

## Task 3: Create Windows System Font Provider

**Files:**
- Create: `platform/src/os/windows/system_fonts.rs`
- Modify: `platform/src/os/windows/mod.rs`
- Modify: `platform/Cargo.toml` (add DirectWrite feature)

**Step 3.1: Add DirectWrite to Windows dependencies**

In `platform/Cargo.toml`, add to the windows features list (around line 82-125):

```toml
    "Win32_Graphics_DirectWrite",
```

**Step 3.2: Create Windows provider file**

Create `platform/src/os/windows/system_fonts.rs`:

```rust
//! Windows system font provider using DirectWrite.

use crate::os::system_fonts::{
    fallbacks, try_fallback_chain, FontStyle, SystemFontData, SystemFontError, SystemFontProvider,
};
use std::fs;
use std::path::PathBuf;
use windows::Win32::Graphics::DirectWrite::*;
use windows::core::*;

/// Windows system font provider using DirectWrite.
pub struct WindowsFontProvider {
    factory: IDWriteFactory,
}

impl WindowsFontProvider {
    pub fn new() -> Result<Self, SystemFontError> {
        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|_| SystemFontError::PlatformNotSupported)?;
            Ok(Self { factory })
        }
    }

    fn get_font_file_path(&self, font_face: &IDWriteFontFace) -> Result<PathBuf, SystemFontError> {
        unsafe {
            let mut file_count = 0u32;
            font_face.GetFiles(&mut file_count, None)
                .map_err(|_| SystemFontError::AccessDenied)?;

            if file_count == 0 {
                return Err(SystemFontError::AccessDenied);
            }

            let mut files: Vec<Option<IDWriteFontFile>> = vec![None; file_count as usize];
            font_face.GetFiles(&mut file_count, Some(files.as_mut_ptr()))
                .map_err(|_| SystemFontError::AccessDenied)?;

            let file = files.into_iter().next().flatten()
                .ok_or(SystemFontError::AccessDenied)?;

            let mut loader: Option<IDWriteFontFileLoader> = None;
            let mut key_ptr: *const std::ffi::c_void = std::ptr::null();
            let mut key_size = 0u32;
            file.GetReferenceKey(&mut key_ptr, &mut key_size)
                .map_err(|_| SystemFontError::AccessDenied)?;
            file.GetLoader(&mut loader)
                .map_err(|_| SystemFontError::AccessDenied)?;

            let loader = loader.ok_or(SystemFontError::AccessDenied)?;

            // Try to get as local file loader
            if let Ok(local_loader) = loader.cast::<IDWriteLocalFontFileLoader>() {
                let mut path_len = 0u32;
                local_loader.GetFilePathLengthFromKey(key_ptr, key_size, &mut path_len)
                    .map_err(|_| SystemFontError::AccessDenied)?;

                let mut path_buf: Vec<u16> = vec![0; (path_len + 1) as usize];
                local_loader.GetFilePathFromKey(key_ptr, key_size, &mut path_buf)
                    .map_err(|_| SystemFontError::AccessDenied)?;

                let path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                return Ok(PathBuf::from(path));
            }

            Err(SystemFontError::AccessDenied)
        }
    }
}

impl Default for WindowsFontProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create DirectWrite factory")
    }
}

impl SystemFontProvider for WindowsFontProvider {
    fn query_font(&self, family: &str, _style: Option<FontStyle>) -> Result<SystemFontData, SystemFontError> {
        unsafe {
            let mut collection: Option<IDWriteFontCollection> = None;
            self.factory.GetSystemFontCollection(&mut collection, false)
                .map_err(|_| SystemFontError::PlatformNotSupported)?;
            let collection = collection.ok_or(SystemFontError::PlatformNotSupported)?;

            let family_wide: Vec<u16> = family.encode_utf16().chain(std::iter::once(0)).collect();
            let mut index = 0u32;
            let mut exists = false.into();
            collection.FindFamilyName(PCWSTR(family_wide.as_ptr()), &mut index, &mut exists)
                .map_err(|_| SystemFontError::FontNotFound(family.to_string()))?;

            if !exists.as_bool() {
                return Err(SystemFontError::FontNotFound(family.to_string()));
            }

            let font_family = collection.GetFontFamily(index)
                .map_err(|_| SystemFontError::FontNotFound(family.to_string()))?;

            let font = font_family.GetFirstMatchingFont(
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
            ).map_err(|_| SystemFontError::FontNotFound(family.to_string()))?;

            let font_face = font.CreateFontFace()
                .map_err(|_| SystemFontError::AccessDenied)?;

            let path = self.get_font_file_path(&font_face)?;
            let data = fs::read(&path).map_err(|e| {
                SystemFontError::ReadError(format!("{}: {}", path.display(), e))
            })?;

            Ok(SystemFontData {
                data,
                index: font_face.GetIndex(),
                family_name: family.to_string(),
            })
        }
    }

    fn list_families(&self) -> Vec<String> {
        unsafe {
            let mut collection: Option<IDWriteFontCollection> = None;
            if self.factory.GetSystemFontCollection(&mut collection, false).is_err() {
                return Vec::new();
            }
            let collection = match collection {
                Some(c) => c,
                None => return Vec::new(),
            };

            let count = collection.GetFontFamilyCount();
            let mut result = Vec::with_capacity(count as usize);

            for i in 0..count {
                if let Ok(family) = collection.GetFontFamily(i) {
                    if let Ok(names) = family.GetFamilyNames() {
                        let mut len = 0u32;
                        if names.GetStringLength(0, &mut len).is_ok() {
                            let mut name: Vec<u16> = vec![0; (len + 1) as usize];
                            if names.GetString(0, &mut name).is_ok() {
                                let s = String::from_utf16_lossy(&name[..len as usize]);
                                result.push(s);
                            }
                        }
                    }
                }
            }

            result
        }
    }

    fn default_sans(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::WINDOWS_SANS, None)
    }

    fn default_monospace(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::WINDOWS_MONO, None)
    }

    fn default_serif(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::WINDOWS_SERIF, None)
    }
}

/// Get the system font provider for Windows
pub fn get_system_font_provider() -> WindowsFontProvider {
    WindowsFontProvider::new().expect("Failed to initialize DirectWrite")
}
```

**Step 3.3: Add module to windows/mod.rs**

Add to `platform/src/os/windows/mod.rs`:

```rust
#[cfg(feature = "system-fonts")]
pub mod system_fonts;
```

**Step 3.4: Commit**

```bash
git add platform/src/os/windows/system_fonts.rs platform/src/os/windows/mod.rs platform/Cargo.toml
git commit -m "feat(platform): add Windows system font provider using DirectWrite"
```

---

## Task 4: Create Linux System Font Provider

**Files:**
- Create: `platform/src/os/linux/fontconfig_sys.rs`
- Create: `platform/src/os/linux/system_fonts.rs`
- Modify: `platform/src/os/linux/mod.rs`

**Step 4.1: Create fontconfig FFI bindings**

Create `platform/src/os/linux/fontconfig_sys.rs`:

```rust
//! Fontconfig FFI bindings for Linux system font access.
//! Uses dynamic loading to avoid hard dependency on fontconfig.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_uchar, c_void};
use super::module_loader::ModuleLoader;

// Fontconfig types
pub type FcConfig = c_void;
pub type FcPattern = c_void;
pub type FcFontSet = c_void;
pub type FcResult = c_int;
pub type FcChar8 = c_uchar;
pub type FcBool = c_int;

// FcResult values
pub const FcResultMatch: FcResult = 0;

// Property names
pub const FC_FAMILY: &[u8] = b"family\0";
pub const FC_FILE: &[u8] = b"file\0";
pub const FC_INDEX: &[u8] = b"index\0";

/// Dynamically loaded fontconfig functions
pub struct Fontconfig {
    _lib: ModuleLoader,
    pub FcInitLoadConfigAndFonts: unsafe extern "C" fn() -> *mut FcConfig,
    pub FcPatternCreate: unsafe extern "C" fn() -> *mut FcPattern,
    pub FcPatternDestroy: unsafe extern "C" fn(*mut FcPattern),
    pub FcPatternAddString: unsafe extern "C" fn(*mut FcPattern, *const c_char, *const FcChar8) -> FcBool,
    pub FcConfigSubstitute: unsafe extern "C" fn(*mut FcConfig, *mut FcPattern, c_int) -> FcBool,
    pub FcDefaultSubstitute: unsafe extern "C" fn(*mut FcPattern),
    pub FcFontMatch: unsafe extern "C" fn(*mut FcConfig, *mut FcPattern, *mut FcResult) -> *mut FcPattern,
    pub FcPatternGetString: unsafe extern "C" fn(*const FcPattern, *const c_char, c_int, *mut *mut FcChar8) -> FcResult,
    pub FcPatternGetInteger: unsafe extern "C" fn(*const FcPattern, *const c_char, c_int, *mut c_int) -> FcResult,
    pub FcFontList: unsafe extern "C" fn(*mut FcConfig, *mut FcPattern, *mut c_void) -> *mut FcFontSet,
    pub FcFontSetDestroy: unsafe extern "C" fn(*mut FcFontSet),
    pub FcConfigDestroy: unsafe extern "C" fn(*mut FcConfig),
}

impl Fontconfig {
    pub fn load() -> Result<Self, ()> {
        let lib = ModuleLoader::load("libfontconfig.so.1")
            .or_else(|_| ModuleLoader::load("libfontconfig.so"))?;

        Ok(Self {
            FcInitLoadConfigAndFonts: lib.get_symbol("FcInitLoadConfigAndFonts")?,
            FcPatternCreate: lib.get_symbol("FcPatternCreate")?,
            FcPatternDestroy: lib.get_symbol("FcPatternDestroy")?,
            FcPatternAddString: lib.get_symbol("FcPatternAddString")?,
            FcConfigSubstitute: lib.get_symbol("FcConfigSubstitute")?,
            FcDefaultSubstitute: lib.get_symbol("FcDefaultSubstitute")?,
            FcFontMatch: lib.get_symbol("FcFontMatch")?,
            FcPatternGetString: lib.get_symbol("FcPatternGetString")?,
            FcPatternGetInteger: lib.get_symbol("FcPatternGetInteger")?,
            FcFontList: lib.get_symbol("FcFontList")?,
            FcFontSetDestroy: lib.get_symbol("FcFontSetDestroy")?,
            FcConfigDestroy: lib.get_symbol("FcConfigDestroy")?,
            _lib: lib,
        })
    }
}
```

**Step 4.2: Create Linux provider file**

Create `platform/src/os/linux/system_fonts.rs`:

```rust
//! Linux system font provider using Fontconfig.

use crate::os::system_fonts::{
    fallbacks, try_fallback_chain, FontStyle, SystemFontData, SystemFontError, SystemFontProvider,
};
use super::fontconfig_sys::*;
use std::ffi::CStr;
use std::fs;
use std::ptr;

/// Linux system font provider using Fontconfig.
pub struct LinuxFontProvider {
    fc: Fontconfig,
    config: *mut FcConfig,
}

// Safety: Fontconfig is thread-safe when properly initialized
unsafe impl Send for LinuxFontProvider {}
unsafe impl Sync for LinuxFontProvider {}

impl LinuxFontProvider {
    pub fn new() -> Result<Self, SystemFontError> {
        let fc = Fontconfig::load()
            .map_err(|_| SystemFontError::PlatformNotSupported)?;

        let config = unsafe { (fc.FcInitLoadConfigAndFonts)() };
        if config.is_null() {
            return Err(SystemFontError::PlatformNotSupported);
        }

        Ok(Self { fc, config })
    }
}

impl Drop for LinuxFontProvider {
    fn drop(&mut self) {
        if !self.config.is_null() {
            unsafe { (self.fc.FcConfigDestroy)(self.config) };
        }
    }
}

impl SystemFontProvider for LinuxFontProvider {
    fn query_font(&self, family: &str, _style: Option<FontStyle>) -> Result<SystemFontData, SystemFontError> {
        unsafe {
            let pattern = (self.fc.FcPatternCreate)();
            if pattern.is_null() {
                return Err(SystemFontError::PlatformNotSupported);
            }

            // Add family name to pattern
            let family_cstr = std::ffi::CString::new(family).unwrap();
            (self.fc.FcPatternAddString)(
                pattern,
                FC_FAMILY.as_ptr() as *const i8,
                family_cstr.as_ptr() as *const u8,
            );

            // Substitute defaults
            (self.fc.FcConfigSubstitute)(self.config, pattern, 0);
            (self.fc.FcDefaultSubstitute)(pattern);

            // Find matching font
            let mut result: FcResult = 0;
            let matched = (self.fc.FcFontMatch)(self.config, pattern, &mut result);
            (self.fc.FcPatternDestroy)(pattern);

            if matched.is_null() || result != FcResultMatch {
                return Err(SystemFontError::FontNotFound(family.to_string()));
            }

            // Get file path
            let mut file_ptr: *mut FcChar8 = ptr::null_mut();
            let file_result = (self.fc.FcPatternGetString)(
                matched,
                FC_FILE.as_ptr() as *const i8,
                0,
                &mut file_ptr,
            );

            if file_result != FcResultMatch || file_ptr.is_null() {
                (self.fc.FcPatternDestroy)(matched);
                return Err(SystemFontError::AccessDenied);
            }

            let path = CStr::from_ptr(file_ptr as *const i8)
                .to_string_lossy()
                .into_owned();

            // Get font index
            let mut index: i32 = 0;
            (self.fc.FcPatternGetInteger)(
                matched,
                FC_INDEX.as_ptr() as *const i8,
                0,
                &mut index,
            );

            // Get actual family name
            let mut family_ptr: *mut FcChar8 = ptr::null_mut();
            let actual_family = if (self.fc.FcPatternGetString)(
                matched,
                FC_FAMILY.as_ptr() as *const i8,
                0,
                &mut family_ptr,
            ) == FcResultMatch && !family_ptr.is_null() {
                CStr::from_ptr(family_ptr as *const i8)
                    .to_string_lossy()
                    .into_owned()
            } else {
                family.to_string()
            };

            (self.fc.FcPatternDestroy)(matched);

            // Read font file
            let data = fs::read(&path).map_err(|e| {
                SystemFontError::ReadError(format!("{}: {}", path, e))
            })?;

            Ok(SystemFontData {
                data,
                index: index as u32,
                family_name: actual_family,
            })
        }
    }

    fn list_families(&self) -> Vec<String> {
        // Simplified: return common font families
        // Full implementation would use FcFontList
        vec![
            "Ubuntu".to_string(),
            "DejaVu Sans".to_string(),
            "Liberation Sans".to_string(),
            "Noto Sans".to_string(),
        ]
    }

    fn default_sans(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::LINUX_SANS, None)
    }

    fn default_monospace(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::LINUX_MONO, None)
    }

    fn default_serif(&self) -> Result<SystemFontData, SystemFontError> {
        try_fallback_chain(self, fallbacks::LINUX_SERIF, None)
    }
}

/// Get the system font provider for Linux
pub fn get_system_font_provider() -> Result<LinuxFontProvider, SystemFontError> {
    LinuxFontProvider::new()
}
```

**Step 4.3: Add modules to linux/mod.rs**

Add to `platform/src/os/linux/mod.rs` (after existing modules):

```rust
#[cfg(all(feature = "system-fonts", not(any(target_env="ohos", target_os="android"))))]
pub mod fontconfig_sys;
#[cfg(all(feature = "system-fonts", not(any(target_env="ohos", target_os="android"))))]
pub mod system_fonts;
```

**Step 4.4: Commit**

```bash
git add platform/src/os/linux/fontconfig_sys.rs platform/src/os/linux/system_fonts.rs platform/src/os/linux/mod.rs
git commit -m "feat(platform): add Linux system font provider using Fontconfig"
```

---

## Task 5: Add Feature Flags

**Files:**
- Modify: `platform/Cargo.toml`
- Modify: `draw/Cargo.toml`

**Step 5.1: Add system-fonts feature to platform/Cargo.toml**

Add after `[dependencies]` section (around line 20):

```toml
[features]
default = []
system-fonts = []
```

**Step 5.2: Add features to draw/Cargo.toml**

Replace the `[features]` section (around line 31-34):

```toml
[features]
default = ["bundled-fonts"]
bundled-fonts = []
system-fonts = ["makepad-platform/system-fonts"]
## Enables certain public-facing types to derive serde serialization traits.
serde = ["dep:serde", "makepad-live-id/serde"]
```

**Step 5.3: Verify features compile**

Run: `cargo check -p makepad-draw --features bundled-fonts`

Expected: Compiles successfully

Run: `cargo check -p makepad-draw --no-default-features --features system-fonts`

Expected: Compiles (may have warnings about unused imports until integration)

**Step 5.4: Commit**

```bash
git add platform/Cargo.toml draw/Cargo.toml
git commit -m "feat: add bundled-fonts and system-fonts feature flags"
```

---

## Task 6: Wire Up Platform Module Exports

**Files:**
- Modify: `platform/src/os/mod.rs`

**Step 6.1: Add system_fonts module export**

Add to `platform/src/os/mod.rs` at the top (after line 8):

```rust
#[cfg(feature = "system-fonts")]
pub mod system_fonts;
```

**Step 6.2: Add platform-specific re-exports**

Add after each platform's existing exports:

After `pub use crate::os::apple::*;` (around line 14):
```rust
#[cfg(all(feature = "system-fonts", any(target_os = "macos", target_os="ios", target_os="tvos")))]
pub use crate::os::apple::system_fonts::get_system_font_provider;
```

After `pub use crate::os::windows::*;` (around line 23):
```rust
#[cfg(all(feature = "system-fonts", target_os = "windows"))]
pub use crate::os::windows::system_fonts::get_system_font_provider;
```

After `pub use crate::os::linux::*;` (around line 32):
```rust
#[cfg(all(feature = "system-fonts", any(target_os = "android", target_os = "linux")))]
pub use crate::os::linux::system_fonts::get_system_font_provider;
```

**Step 6.3: Commit**

```bash
git add platform/src/os/mod.rs
git commit -m "feat(platform): wire up system font provider exports"
```

---

## Task 7: Extend Font Loader with FontSource

**Files:**
- Modify: `draw/src/text/loader.rs`

**Step 7.1: Add FontSource enum**

Add after the imports (around line 13):

```rust
#[cfg(feature = "system-fonts")]
use makepad_platform::os::system_fonts::{SystemFontProvider, SystemFontError};

/// Source for font data - either embedded bytes or system font lookup.
#[derive(Clone, Debug)]
pub enum FontSource {
    /// Font data embedded in the binary
    Embedded {
        data: Rc<Vec<u8>>,
        index: u32,
    },
    /// Font loaded from the operating system
    #[cfg(feature = "system-fonts")]
    System {
        family: String,
        style: Option<String>,
    },
}
```

**Step 7.2: Update FontDefinition struct**

Replace `FontDefinition` (around line 138-144):

```rust
#[derive(Clone, Debug)]
pub struct FontDefinition {
    pub source: FontSource,
    pub ascender_fudge_in_ems: f32,
    pub descender_fudge_in_ems: f32,
}

impl FontDefinition {
    /// Create a font definition from embedded data (backwards compatible)
    pub fn from_data(data: Rc<Vec<u8>>, index: u32) -> Self {
        Self {
            source: FontSource::Embedded { data, index },
            ascender_fudge_in_ems: 0.0,
            descender_fudge_in_ems: 0.0,
        }
    }

    /// Create a font definition from embedded data with fudge factors
    pub fn from_data_with_fudge(
        data: Rc<Vec<u8>>,
        index: u32,
        ascender_fudge_in_ems: f32,
        descender_fudge_in_ems: f32,
    ) -> Self {
        Self {
            source: FontSource::Embedded { data, index },
            ascender_fudge_in_ems,
            descender_fudge_in_ems,
        }
    }

    /// Create a font definition for a system font
    #[cfg(feature = "system-fonts")]
    pub fn from_system(family: impl Into<String>) -> Self {
        Self {
            source: FontSource::System {
                family: family.into(),
                style: None,
            },
            ascender_fudge_in_ems: 0.0,
            descender_fudge_in_ems: 0.0,
        }
    }
}
```

**Step 7.3: Update load_font method**

Replace `load_font` method (around line 111-124):

```rust
    fn load_font(&mut self, id: FontId) -> Font {
        let definition = self
            .font_definitions
            .remove(&id)
            .expect("font is not defined");

        let (data, index) = match definition.source {
            FontSource::Embedded { data, index } => (data, index),
            #[cfg(feature = "system-fonts")]
            FontSource::System { family, style: _ } => {
                // Get system font provider and query font
                let provider = makepad_platform::os::get_system_font_provider();
                let font_data = provider.query_font(&family, None)
                    .unwrap_or_else(|e| panic!("Failed to load system font '{}': {}", family, e));
                (Rc::new(font_data.data), font_data.index)
            }
        };

        Font::new(
            id.clone(),
            self.rasterizer.clone(),
            FontFace::from_data_and_index(data, index)
                .expect("failed to load font from definition"),
            definition.ascender_fudge_in_ems,
            definition.descender_fudge_in_ems,
        )
    }
```

**Step 7.4: Build to verify**

Run: `cargo build -p makepad-draw --features bundled-fonts`

Expected: Compiles successfully

**Step 7.5: Commit**

```bash
git add draw/src/text/loader.rs
git commit -m "feat(draw): add FontSource enum for system font support"
```

---

## Task 8: Make Bundled Fonts Conditional

**Files:**
- Modify: `draw/src/text/builtins.rs`

**Step 8.1: Add conditional compilation for embedded fonts**

Replace the entire file content:

```rust
use super::loader::{FontDefinition, FontFamilyDefinition, FontSource, Loader};
use std::rc::Rc;

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
        FontDefinition::from_data(Rc::new(IBM_PLEX_SANS_TEXT.to_vec()), 0),
    );
    loader.define_font(
        "LXG WWen Kai Regular".into(),
        FontDefinition::from_data(Rc::new(LXG_WEN_KAI_REGULAR.to_vec()), 0),
    );
    loader.define_font(
        "Noto Color Emoji".into(),
        FontDefinition::from_data(Rc::new(NOTO_COLOR_EMOJI.to_vec()), 0),
    );
    loader.define_font(
        "Liberation Mono Regular".into(),
        FontDefinition::from_data(Rc::new(LIBERATION_MONO_REGULAR.to_vec()), 0),
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

    // System fonts will be resolved at load time
    #[cfg(target_os = "macos")]
    {
        loader.define_font("System Sans".into(), FontDefinition::from_system("SF Pro"));
        loader.define_font("System Mono".into(), FontDefinition::from_system("SF Mono"));
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
```

**Step 8.2: Build to verify both features**

Run: `cargo build -p makepad-draw --features bundled-fonts`

Expected: Compiles with bundled fonts

Run: `cargo build -p makepad-draw --no-default-features --features system-fonts`

Expected: Compiles with system fonts (on supported platforms)

**Step 8.3: Commit**

```bash
git add draw/src/text/builtins.rs
git commit -m "feat(draw): make bundled fonts conditional on feature flag"
```

---

## Task 9: Integration Test

**Files:**
- Create: `draw/tests/system_fonts.rs`

**Step 9.1: Create integration test**

Create `draw/tests/system_fonts.rs`:

```rust
//! Integration tests for system font support.

#[cfg(all(test, feature = "system-fonts"))]
mod tests {
    use makepad_platform::os::system_fonts::{SystemFontProvider, FontStyle};

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_query_font() {
        let provider = makepad_platform::os::get_system_font_provider();

        // Query a known system font
        let result = provider.query_font("Helvetica", None);
        assert!(result.is_ok(), "Should find Helvetica on macOS");

        let font_data = result.unwrap();
        assert!(!font_data.data.is_empty(), "Font data should not be empty");
        assert!(!font_data.family_name.is_empty(), "Family name should not be empty");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_default_fonts() {
        let provider = makepad_platform::os::get_system_font_provider();

        let sans = provider.default_sans();
        assert!(sans.is_ok(), "Should find default sans font");

        let mono = provider.default_monospace();
        assert!(mono.is_ok(), "Should find default monospace font");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_list_families() {
        let provider = makepad_platform::os::get_system_font_provider();
        let families = provider.list_families();

        assert!(!families.is_empty(), "Should list some font families");
        // Helvetica should be present on all macOS systems
        assert!(families.iter().any(|f| f.contains("Helvetica")),
            "Should include Helvetica");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_query_font() {
        let provider = makepad_platform::os::get_system_font_provider();

        let result = provider.query_font("Arial", None);
        assert!(result.is_ok(), "Should find Arial on Windows");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_query_font() {
        if let Ok(provider) = makepad_platform::os::get_system_font_provider() {
            let result = provider.query_font("DejaVu Sans", None);
            // DejaVu Sans is commonly installed but not guaranteed
            if result.is_ok() {
                let font_data = result.unwrap();
                assert!(!font_data.data.is_empty());
            }
        }
    }
}
```

**Step 9.2: Run tests**

Run: `cargo test -p makepad-draw --features system-fonts -- --nocapture`

Expected: Tests pass on current platform

**Step 9.3: Commit**

```bash
git add draw/tests/system_fonts.rs
git commit -m "test(draw): add system font integration tests"
```

---

## Task 10: Final Verification

**Step 10.1: Build with bundled fonts (default)**

Run: `cargo build -p makepad-draw --release`

Expected: Compiles, binary includes bundled fonts (~47MB contribution)

**Step 10.2: Build with system fonts only**

Run: `cargo build -p makepad-draw --release --no-default-features --features system-fonts`

Expected: Compiles, minimal binary size

**Step 10.3: Run example with system fonts**

Run: `cargo run -p makepad-example-simple --no-default-features --features system-fonts`

Expected: Application renders text using system fonts

**Step 10.4: Final commit with all changes**

```bash
git add -A
git commit -m "feat: complete system font support implementation

- Add SystemFontProvider trait for cross-platform font access
- Implement macOS provider using Core Text
- Implement Windows provider using DirectWrite
- Implement Linux provider using Fontconfig (dynamic loading)
- Add FontSource enum to support embedded and system fonts
- Add bundled-fonts and system-fonts feature flags
- Make bundled fonts conditional on feature flag
- Add integration tests for all platforms

Binary size reduction: ~47MB when using system-fonts feature
"
```

---

## Summary

| Task | Description | Est. Time |
|------|-------------|-----------|
| 1 | Complete Core Text FFI | 5 min |
| 2 | macOS Font Provider | 15 min |
| 3 | Windows Font Provider | 15 min |
| 4 | Linux Font Provider | 15 min |
| 5 | Feature Flags | 5 min |
| 6 | Platform Module Exports | 5 min |
| 7 | Extend Font Loader | 10 min |
| 8 | Conditional Builtins | 10 min |
| 9 | Integration Tests | 10 min |
| 10 | Final Verification | 5 min |

**Total: ~95 minutes**

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Font file access denied | Graceful fallback to bundled fonts + clear error |
| TTC collection index wrong | Parse font file to match family name |
| Variable fonts | Extract default instance; document limitation |
| Fontconfig not installed (Linux) | Dynamic loading with graceful error |
