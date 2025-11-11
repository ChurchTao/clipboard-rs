# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a cross-platform clipboard library written in Rust that provides APIs for getting and setting system-level clipboard content. It supports Windows, macOS, Linux (X11), and iOS.

## Key Features

- Plain text clipboard operations
- HTML and Rich Text Format (RTF) support
- Image clipboard operations (PNG format)
- File list clipboard operations
- Custom format support
- Clipboard change monitoring
- Cross-platform support (Windows, macOS, Linux, iOS)

## Architecture

The library uses a platform-specific implementation approach:

1. **Core Interface**: Defined in `src/lib.rs` with `Clipboard` and `ClipboardWatcher` traits
2. **Platform Modules**:
   - `src/platform/macos.rs` - macOS implementation using objc2
   - `src/platform/win.rs` - Windows implementation using clipboard-win and windows crates
   - `src/platform/x11.rs` - Linux X11 implementation using x11rb
   - `src/platform/ios.rs` - iOS implementation
3. **Common Types**: Defined in `src/common.rs` including `ClipboardContent`, `ContentFormat`, and image handling utilities

## Common Development Tasks

### Building the Project

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features
```

### Running Tests

```bash
# Run all tests (note: some tests are platform-specific and may be ignored)
cargo test

# Run tests for specific platform
cargo test --target=x86_64-pc-windows-msvc  # Windows
cargo test --target=x86_64-apple-darwin     # macOS
```

### Running Examples

```bash
# Run basic example
cargo run --example helloworld

# Run image example (requires image feature)
cargo run --example image --features=image

# Run file example
cargo run --example files

# Run clipboard watcher example
cargo run --example watch_change
```

### Code Formatting and Linting

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy --all -- -D warnings
```

## Platform-Specific Considerations

### macOS
- Uses objc2 crates for Objective-C interop
- Implements NSPasteboard for clipboard operations
- Uses autoreleasepool for memory management

### Windows
- Uses clipboard-win and windows crates
- Handles multiple clipboard formats (CF_UNICODETEXT, CF_DIB, etc.)
- Supports PNG, BMP, and DIB image formats

### Linux (X11)
- Uses x11rb crate for X11 protocol implementation
- Implements X11 clipboard selection mechanism
- Has configurable read timeout (default 500ms)

### iOS
- Uses objc2-ui-kit for UIPasteboard integration
- Limited file support compared to other platforms

## Key Implementation Details

1. **Image Handling**: The library uses the `image` crate for image processing with optional feature flag
2. **Clipboard Content**: Content is represented as `ClipboardContent` enum with variants for different types
3. **Format Detection**: Each platform implements `available_formats()` and `has()` methods to detect clipboard content types
4. **Memory Safety**: Uses Rust's ownership system and platform-specific memory management patterns
5. **Error Handling**: Uses `Result<T>` type for all operations that can fail

## Testing Considerations

- Tests require an actual desktop environment to interact with the system clipboard
- Some platform-specific tests are marked with `#[ignore]` and need to be run manually
- Image tests require valid image files (see `tests/test.png`)
- Tests modify the actual system clipboard, so they should be run in isolation

## Dependencies

Key external dependencies include:
- `objc2` family of crates for macOS/iOS
- `windows` and `clipboard-win` for Windows
- `x11rb` for Linux X11
- `image` crate for image processing (optional feature)