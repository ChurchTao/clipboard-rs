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

1. **Core Interface**: Defined in `src/lib.rs` with `Clipboard` and `AsyncClipboard` traits
2. **High-Level API**: `ClipboardManager` in `src/lib.rs` provides a unified interface for both synchronous and asynchronous operations
3. **Platform Modules**:
   - `src/platform/macos.rs` - macOS implementation using objc2
   - `src/platform/win.rs` - Windows implementation using clipboard-win and windows crates
   - `src/platform/x11.rs` - Linux X11 implementation using x11rb
   - `src/platform/ios.rs` - iOS implementation
4. **Common Types**: Defined in `src/common.rs` including `ClipboardContent`, `ContentFormat`, `ClipboardContentBuilder`, and image handling utilities

## Common Development Tasks

### Building the Project

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features
```

### Using the New API

The library now provides a unified `ClipboardManager` API that supports both synchronous and asynchronous operations:

```rust
// Synchronous usage
use clipboard_rs::{ClipboardManager, ContentFormat};

fn sync_example() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new_sync()?;

    // Get text
    let text = clipboard.get_text_sync()?;

    // Set text
    clipboard.set_text_sync("Hello, World!")?;

    // Using the fluent builder API
    clipboard.set_with_builder_sync(
        clipboard
            .build_content()
            .with_text("Hello, World!")
            .with_html("<h1>Hello, World!</h1>")
    )?;

    Ok(())
}

// Asynchronous usage
use clipboard_rs::{ClipboardManager, ContentFormat};

#[tokio::main]
async fn async_example() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new().await?;

    // Get text
    let text = clipboard.get_text().await?;

    // Set text
    clipboard.set_text("Hello, World!").await?;

    // Using the fluent builder API
    clipboard
        .set_with_builder(
            clipboard
                .build_content()
                .with_text("Hello, World!")
                .with_html("<h1>Hello, World!</h1>")
        )
        .await?;

    Ok(())
}

// Synchronous image usage
#[cfg(feature = "image")]
fn sync_image_example() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new_sync()?;

    // Load image from file and set to clipboard
    let image = clipboard_rs::ClipboardImage::from_path_sync("input.png")?;
    clipboard.set_image_sync(image)?;

    // Get image from clipboard
    let image = clipboard.get_image_sync()?;

    // Save image to file
    image.save_to_path_sync("output.png")?;

    // Create thumbnail
    let thumbnail = image.thumbnail_sync(100, 100)?;
    thumbnail.save_to_path_sync("thumbnail.png")?;

    Ok(())
}

// Asynchronous image usage
#[cfg(feature = "async-image")]
async fn async_image_example() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new().await?;

    // Load image from file and set to clipboard
    let image = clipboard_rs::ClipboardImage::from_path("input.png").await?;
    clipboard.set_image(image).await?;

    // Get image from clipboard
    let image = clipboard.get_image().await?;

    // Save image to file
    image.save_to_path("output.png").await?;

    // Create thumbnail
    let thumbnail = image.thumbnail(100, 100).await?;
    thumbnail.save_to_path("thumbnail.png").await?;

    Ok(())
}

// Asynchronous clipboard watcher with stop functionality
#[cfg(feature = "async")]
async fn async_watcher_example() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new().await?;

    // Start watching clipboard changes
    let mut event_stream = clipboard.watch().await?;

    println!("Clipboard watcher started. Try copying some text to see events!");
    println!("Press Enter to stop the watcher...");

    // Handle events in a separate task
    let handle_events = tokio::spawn(async move {
        loop {
            match event_stream.next().await {
                Some(ClipboardEvent::Changed { formats }) => {
                    println!("Clipboard changed! Available formats: {:?}", formats);
                }
                Some(ClipboardEvent::Cleared) => {
                    println!("Clipboard cleared!");
                }
                Some(ClipboardEvent::Error(e)) => {
                    eprintln!("Clipboard error: {:?}", e);
                }
                None => {
                    println!("Event stream ended");
                    break;
                }
            }
        }
    });

    // Wait for user input to stop the watcher
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop the watcher
    event_stream.stop();

    // Wait for the event handling task to complete
    let _ = handle_events.await;

    Ok(())
}
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

# Run sync image example (requires image feature)
cargo run --example sync_image --features=image

# Run file example
cargo run --example files

# Run clipboard watcher example (new async API)
cargo run --example watch_change --features=async

# Run modern async example
cargo run --example modern --features=async

# Run modern image example
cargo run --example image_modern --features=async-image

# Run clipboard watcher with stop example
cargo run --example async_watcher_with_stop --features=async
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

1. **Image Handling**: The library uses the `image` crate for image processing with optional feature flag. The new unified `ClipboardImage` API provides both synchronous and asynchronous methods for image operations.
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