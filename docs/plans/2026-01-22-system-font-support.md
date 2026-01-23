# System Font Support

**Status:** Complete

**Goal:** Enable Makepad to use operating system fonts instead of bundled fonts.

## Architecture

Platform-specific font providers implement `SystemFontProvider` trait:

```rust
pub trait SystemFontProvider: Send + Sync {
    fn query_font(&self, family: &str) -> Result<SystemFontData, SystemFontError>;
    fn list_families(&self) -> Vec<String>;
}
```

The draw crate's `FontSource` enum distinguishes embedded vs system fonts:

```rust
pub enum FontSource {
    Embedded { data: Rc<Vec<u8>>, index: u32 },
    System { family: String },  // requires system-fonts feature
}
```

## Files

| File | Description |
|------|-------------|
| `platform/src/os/system_fonts.rs` | Trait and types |
| `platform/src/os/apple/system_fonts.rs` | macOS (Core Text) |
| `platform/src/os/windows/system_fonts.rs` | Windows (DirectWrite) |
| `platform/src/os/linux/system_fonts.rs` | Linux (Fontconfig) |
| `platform/src/os/linux/fontconfig_sys.rs` | Fontconfig FFI |
| `draw/src/text/loader.rs` | FontSource integration |
| `draw/src/text/builtins.rs` | Conditional font definitions |
| `platform/tests/system_fonts.rs` | Integration tests |

## Usage

```bash
# Default (bundled fonts)
cargo run -p makepad-fonts --release

# System fonts only
cargo run -p makepad-fonts --no-default-features --features system-fonts --release
```

## Feature Flags

| Crate | Feature | Description |
|-------|---------|-------------|
| `draw` | `bundled-fonts` (default) | Include embedded fonts |
| `draw` | `system-fonts` | Enable system font loading |
| `widgets` | Same as draw | Pass through to draw |

## Limitations

- Widgets crate has unconditional dependencies on Chinese/emoji font crates
- Full binary size reduction requires refactoring `widgets/src/theme_*.rs`
