# System Font Support

**Status:** Complete
**Date:** 2026-01-22

## Overview

This document describes the changes made to enable Makepad applications to use operating system fonts instead of bundled fonts. This reduces distribution size and allows apps to use native system fonts for non-Latin text rendering.

## Architecture

### Font Loading Modes

| Mode | Feature Flag | Description |
|------|--------------|-------------|
| Bundled (default) | `bundled-fonts` | Embeds Chinese and emoji fonts from separate crates |
| System | `system-fonts` | Uses OS font providers for fallback |

### Platform-Specific Font Providers

Platform-specific implementations of `SystemFontProvider` trait:

| Platform | Backend | File |
|----------|---------|------|
| macOS/iOS | Core Text | `platform/src/os/apple/system_fonts.rs` |
| Windows | DirectWrite | `platform/src/os/windows/system_fonts.rs` |
| Linux | Fontconfig | `platform/src/os/linux/system_fonts.rs` |

```rust
pub trait SystemFontProvider: Send + Sync {
    fn query_font(&self, family: &str) -> Result<SystemFontData, SystemFontError>;
    fn list_families(&self) -> Vec<String>;
}
```

## Changes Made

### 1. widgets/Cargo.toml

Made Chinese and emoji font crates optional, gated behind `bundled-fonts` feature:

```toml
[dependencies]
makepad-draw = { path = "../draw", version = "1.0.0", default-features = false }

makepad-fonts-emoji = {path = "./fonts/emoji", version = "1.0.0", optional = true}
makepad-fonts-chinese-regular = {path = "./fonts/chinese_regular", version = "1.0.1", optional = true}
makepad-fonts-chinese-regular-2 = {path = "./fonts/chinese_regular_2", version = "1.0.1", optional = true}
makepad-fonts-chinese-bold = {path = "./fonts/chinese_bold", version = "1.0.1", optional = true}
makepad-fonts-chinese-bold-2 = {path = "./fonts/chinese_bold_2", version = "1.0.1", optional = true}

[features]
default = ["bundled-fonts"]
bundled-fonts = [
    "makepad-draw/bundled-fonts",
    "dep:makepad-fonts-emoji",
    "dep:makepad-fonts-chinese-regular",
    "dep:makepad-fonts-chinese-regular-2",
    "dep:makepad-fonts-chinese-bold",
    "dep:makepad-fonts-chinese-bold-2",
]
system-fonts = ["makepad-draw/system-fonts"]
```

### 2. widgets/src/lib.rs

Conditional font crate initialization:

```rust
#[cfg(feature = "bundled-fonts")]
{
    makepad_fonts_emoji::live_design(cx);
    makepad_fonts_chinese_regular::live_design(cx);
    makepad_fonts_chinese_regular_2::live_design(cx);
    makepad_fonts_chinese_bold::live_design(cx);
    makepad_fonts_chinese_bold_2::live_design(cx);
}
```

### 3. code_editor/Cargo.toml

Added feature flag propagation:

```toml
[dependencies]
makepad-widgets = { path = "../widgets", version="1.0.0", default-features = false}

[features]
default = ["bundled-fonts"]
bundled-fonts = ["makepad-widgets/bundled-fonts"]
system-fonts = ["makepad-widgets/system-fonts"]
```

### 4. libs/math_widget/Cargo.toml

Added feature flag propagation:

```toml
[dependencies]
makepad-widgets = { path = "../../widgets", default-features = false }

[features]
default = ["bundled-fonts"]
bundled-fonts = ["makepad-widgets/bundled-fonts"]
system-fonts = ["makepad-widgets/system-fonts"]
```

### 5. Theme Files (Desktop)

Removed Chinese and emoji font references from `font_family` definitions in:
- `widgets/src/theme_desktop_dark.rs`
- `widgets/src/theme_desktop_light.rs`
- `widgets/src/theme_desktop_skeleton.rs`

Before:
```rust
pub THEME_FONT_REGULAR = {
    font_family: {
        latin = font("crate://self/resources/IBMPlexSans-Text.ttf", -0.1, 0.0),
        chinese = font(
            "crate://makepad_fonts_chinese_regular/resources/LXGWWenKaiRegular.ttf",
            "crate://makepad_fonts_chinese_regular_2/resources/LXGWWenKaiRegular.ttf.2",
            0.0, 0.0)
        emoji = font("crate://makepad_fonts_emoji/resources/NotoColorEmoji.ttf", 0.0, 0.0)
    },
    line_spacing: 1.2
}
```

After:
```rust
pub THEME_FONT_REGULAR = {
    font_family: {
        latin = font("crate://self/resources/IBMPlexSans-Text.ttf", -0.1, 0.0),
    },
    line_spacing: 1.2
}
```

## Usage

### Using System Fonts in Your App

```toml
# In your Cargo.toml
[dependencies]
makepad-widgets = { version = "1.0", default-features = false, features = ["system-fonts"] }
makepad-code-editor = { version = "1.0", default-features = false, features = ["system-fonts"] }
```

### Using Bundled Fonts (Default)

```toml
# In your Cargo.toml (default behavior)
[dependencies]
makepad-widgets = "1.0"
```

### Running Examples

```bash
# Default (bundled fonts)
cargo run -p makepad-example-ui-zoo --release

# System fonts only
cargo run -p makepad-example-ui-zoo --no-default-features --features system-fonts --release
```

## Trade-offs

### Bundled Fonts

**Pros:**
- Consistent appearance across all platforms
- Guaranteed font availability
- Predictable text layout and metrics
- Works offline/embedded

**Cons:**
- Larger distribution size (~80MB for Chinese + emoji fonts)
- Memory overhead for font data
- Limited to bundled scripts only

### System Fonts

**Pros:**
- Smaller distribution size
- Native platform appearance
- Full Unicode coverage via OS fallback
- Lower memory usage

**Cons:**
- Platform-dependent rendering
- Potential layout differences across platforms
- Dependent on OS font availability

## Files Modified

| File | Change |
|------|--------|
| `widgets/Cargo.toml` | Made font crates optional |
| `widgets/src/lib.rs` | Conditional font initialization |
| `code_editor/Cargo.toml` | Feature flag propagation |
| `libs/math_widget/Cargo.toml` | Feature flag propagation |
| `widgets/src/theme_desktop_dark.rs` | Removed chinese/emoji font refs |
| `widgets/src/theme_desktop_light.rs` | Removed chinese/emoji font refs |
| `widgets/src/theme_desktop_skeleton.rs` | Removed chinese/emoji font refs |

## Integration Example: moly-ai

The moly-ai project uses makepad-fonts with system fonts:

```toml
# moly-ai/Cargo.toml
[dependencies]
makepad-widgets = { path = "../makepad-fonts/widgets", features = ["system-fonts"], default-features = false }
makepad-code-editor = { path = "../makepad-fonts/code_editor", features = ["system-fonts"], default-features = false }

# Patch other dependencies that use makepad
[patch.'https://github.com/wyeworks/makepad']
makepad-widgets = { path = "../makepad-fonts/widgets" }
makepad-code-editor = { path = "../makepad-fonts/code_editor" }
makepad-draw = { path = "../makepad-fonts/draw" }
makepad-platform = { path = "../makepad-fonts/platform" }
math_widget = { path = "../makepad-fonts/libs/math_widget" }
```
