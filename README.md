# clipboard-rs

[![Latest version](https://img.shields.io/crates/v/clipboard-rs?color=mediumvioletred)](https://crates.io/crates/clipboard-rs)
[![Documentation](https://docs.rs/clipboard-rs/badge.svg)](https://docs.rs/clipboard-rs)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ChurchTao/clipboard-rs/test.yml)
![MSRV](https://img.shields.io/badge/rustc-1.67+-blue.svg)
![GitHub License](https://img.shields.io/github/license/ChurchTao/clipboard-rs)

clipboard-rs is a cross-platform library written in Rust for getting and setting the system-level clipboard content. It supports Linux, Windows, and MacOS.

[简体中文](README_ZH.md)

## Function Support

- Plain text
- Html
- Rich text
- Image (In `PNG` format)
- File (In `file-uri-list` format)
- Any type (by specifying the type identifier) can be obtained through the `available_formats` method

## Features

- `default` - Enable text support (basic clipboard functionality)
- `text` - Enable text support (basic clipboard functionality)
- `image` - Enable basic image support (depends on text)
- `async` - Enable modern async API support

### Platform Support Type Comparison Table

| Type          | Windows              | macOS               | Linux(X11) | iOS(Beta) | Android(WIP) |
| ------------- | -------------------- | ------------------- | ---------- | --------- | ------------ |
| Plain Text    | ✅                   | ✅                  | ✅         | ✅        | 🚧           |
| HTML          | ✅                   | ✅                  | ✅         | ✅        | 🚧           |
| RTF           | ✅                   | ✅                  | ✅         | ✅        | 🚧           |
| Image         | PNG(preferred)/DIBV5 | PNG(preferred)/TIFF | PNG        | PNG       | 🚧           |
| File List     | ✅                   | ✅                  | ✅         | ❌        | 🚧           |
| Custom Type   | ✅                   | ✅                  | ✅         | ✅        | 🚧           |
| Watch Changes | ✅                   | ✅                  | ✅         | ✅        | 🚧           |

## Development Plan

- [x] MacOS Support
- [x] Linux Support (x11)
- [x] Windows Support
- [x] iOS Support (Beta)
- [ ] Android Support (🚧)

## Usage

Add the following content to your `Cargo.toml`:

```toml
[dependencies]
clipboard-rs = "0.4.0"
```

## [CHANGELOG](CHANGELOG.md)

## Examples

### All Usage Examples

[Examples](examples)

### Simple Read and Write (Synchronous API)

```rust
use clipboard_rs::{SyncClipboardManager, ContentFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new synchronous clipboard manager (requires "text" feature)
    let clipboard = SyncClipboardManager::new()?;

    let formats = clipboard.available_formats()?;
    println!("Available formats: {:?}", formats);

    let has_rtf = clipboard.has(ContentFormat::Rtf)?;
    println!("has_rtf={}", has_rtf);

    let rtf = clipboard.get_rtf().unwrap_or_default();
    println!("rtf={}", rtf);

    let has_html = clipboard.has(ContentFormat::Html)?;
    println!("has_html={}", has_html);

    let html = clipboard.get_html().unwrap_or_default();
    println!("html={}", html);

    let content = clipboard.get_text().unwrap_or_default();
    println!("txt={}", content);

    // Using the fluent builder API
    clipboard
        .set_with_builder(
            clipboard
                .build_content()
                .with_text("Hello, World!")
                .with_html("<h1>Hello, World!</h1>")
                .with_rtf(r"{\rtf1\ansi\b Hello, World!}")
        )?;

    Ok(())
}
```

### Simple Read and Write (Modern Async API)

> 💡 **提示**: 此API需要启用`async` feature。默认情况下，`text` feature已启用，因此您可以直接使用异步API。

```rust
use clipboard_rs::{AsyncClipboardManager, ContentFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = AsyncClipboardManager::new().await?;

    let formats = clipboard.available_formats().await?;
    println!("Available formats: {:?}", formats);

    let has_rtf = clipboard.has(ContentFormat::Rtf).await?;
    println!("has_rtf={}", has_rtf);

    let rtf = clipboard.get_rtf().await.unwrap_or_default();
    println!("rtf={}", rtf);

    let has_html = clipboard.has(ContentFormat::Html).await?;
    println!("has_html={}", has_html);

    let html = clipboard.get_html().await.unwrap_or_default();
    println!("html={}", html);

    let content = clipboard.get_text().await.unwrap_or_default();
    println!("txt={}", content);

    // Using the fluent builder API
    clipboard
        .set_with_builder(
            clipboard
                .build_content()
                .with_text("Hello, World!")
                .with_html("<h1>Hello, World!</h1>")
                .with_rtf(r"{\rtf1\ansi\b Hello, World!}")
        )
        .await?;

    Ok(())
}
```

### Reading Images (Unified Synchronous API)

> 💡 **提示**: 此API使用统一的ClipboardImage结构，提供同步方法处理图像，避免阻塞主线程。

```rust
use clipboard_rs::{SyncClipboardManager, ClipboardImage};

#[cfg(target_os = "macos")]
const TMP_PATH: &str = "/tmp/";
#[cfg(target_os = "windows")]
const TMP_PATH: &str = "C:\\Windows\\Temp\\";
#[cfg(all(
	unix,
	not(any(
		target_os = "macos",
		target_os = "ios",
		target_os = "android",
		target_os = "emscripten"
	))
))]
const TMP_PATH: &str = "/tmp/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new synchronous clipboard manager (requires "image" feature)
    let clipboard = SyncClipboardManager::new()?;

    // 从剪贴板获取图像
    match clipboard.get_image() {
        Ok(img) => {
            // 保存图像到文件（同步方法）
            img.save_to_path_sync(format!("{}test.png", TMP_PATH).as_str())?;
            println!("Image saved to {}test.png", TMP_PATH);

            // 创建缩略图（同步方法）
            let resize_img = img.thumbnail_sync(300, 300)?;
            resize_img.save_to_path_sync(format!("{}test_thumbnail.png", TMP_PATH).as_str())?;
            println!("Thumbnail saved to {}test_thumbnail.png", TMP_PATH);
        }
        Err(err) => {
            println!("Failed to get image from clipboard: {}", err);
        }
    }

    // 从文件创建图像并设置到剪贴板
    match ClipboardImage::from_path_sync("input.png") {
        Ok(image) => {
            clipboard.set_image(image)?;
            println!("Image set to clipboard successfully!");
        }
        Err(err) => {
            println!("Failed to load image from file: {}", err);
        }
    }

    Ok(())
}
```

### Reading Images (Modern Async API)

> 💡 **提示**: 此API需要启用`async-image` feature，它会自动启用`async`和`image` features。使用统一的ClipboardImage结构，提供异步方法处理图像。

To use the modern async image API, enable the `async-image` feature:

```bash
cargo run --example image_modern --features async-image
```

```rust
//! Modern image processing example
//! Requires async-image feature: `cargo run --example image_modern --features async-image`

#[cfg(feature = "async-image")]
use clipboard_rs::{ClipboardImage, AsyncClipboardManager};

#[cfg(feature = "async-image")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new clipboard manager
    let clipboard = AsyncClipboardManager::new().await?;

    // Create a simple image
    let mut image_buffer = image::RgbImage::new(100, 100);

    // Draw a red square
    for x in 0..50 {
        for y in 0..50 {
            image_buffer.put_pixel(x, y, image::Rgb([255, 0, 0]));
        }
    }

    // Convert to ClipboardImage
    let clipboard_image = clipboard_rs::ClipboardImage::from_dynamic_image(
        image::DynamicImage::ImageRgb8(image_buffer)
    );

    // Set image to clipboard (using unified API)
    clipboard.set_image(clipboard_image).await?;
    println!("Image set to clipboard successfully!");

    // Get image from clipboard (using unified API)
    match clipboard.get_image().await {
        Ok(image) => {
            println!("Got image from clipboard!");
            println!("Image dimensions: {:?}", image.dimensions());

            // Save image to file (async method)
            image.save_to_path("clipboard_image.png").await?;
            println!("Image saved to clipboard_image.png");
        }
        Err(e) => {
            println!("Failed to get image from clipboard: {}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "async-image"))]
fn main() {
    println!("This example requires the 'async-image' feature to be enabled.");
    println!("Run with: cargo run --example image_modern --features async-image");
}
```

### Reading Any Format

```rust
use clipboard_rs::{SyncClipboardManager, ContentFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new synchronous clipboard manager
    let clipboard = SyncClipboardManager::new()?;

    let formats = clipboard.available_formats()?;
    println!("Available formats: {:?}", formats);

    // Check if HTML format is available
    let has_html = clipboard.has(ContentFormat::Html)?;
    if has_html {
        let html_content = clipboard.get_html()?;
        println!("HTML content: {}", html_content);
    }

    // Read raw data with custom format
    if formats.contains(&"public.html".to_string()) {
        let buffer = clipboard.get_raw("public.html")?;
        let string = String::from_utf8(buffer)?;
        println!("Raw HTML: {}", string);
    }

    Ok(())
}
```

### Reading Any Format (Async API)

```rust
use clipboard_rs::{AsyncClipboardManager, ContentFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new async clipboard manager
    let clipboard = AsyncClipboardManager::new().await?;

    let formats = clipboard.available_formats().await?;
    println!("Available formats: {:?}", formats);

    // Check if HTML format is available
    let has_html = clipboard.has(ContentFormat::Html).await?;
    if has_html {
        let html_content = clipboard.get_html().await?;
        println!("HTML content: {}", html_content);
    }

    // Read raw data with custom format
    if formats.contains(&"public.html".to_string()) {
        let buffer = clipboard.get_raw("public.html").await?;
        let string = String::from_utf8(buffer)?;
        println!("Raw HTML: {}", string);
    }

    Ok(())
}

### Listening to Clipboard Changes (Legacy Synchronous API)

> ⚠️ **警告**: 此API使用同步阻塞方式监听剪贴板变化。推荐使用现代化的异步监听API。

```rust
use clipboard_rs::{
	Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
};
use std::{thread, time::Duration};

struct Manager {
	ctx: ClipboardContext,
}

impl Manager {
	pub fn new() -> Self {
		let ctx = ClipboardContext::new().unwrap();
		Manager { ctx }
	}
}

impl ClipboardHandler for Manager {
	fn on_clipboard_change(&mut self) {
		println!(
			"on_clipboard_change, txt = {}",
			self.ctx.get_text().unwrap()
		);
	}
}

fn main() {
	let manager = Manager::new();

	let mut watcher = ClipboardWatcherContext::new().unwrap();

	let watcher_shutdown = watcher.add_handler(manager).get_shutdown_channel();

	thread::spawn(move || {
		thread::sleep(Duration::from_secs(5));
		println!("stop watch!");
		watcher_shutdown.stop();
	});

	println!("start watch!");
	watcher.start_watch();
}
```

### Listening to Clipboard Changes (Modern Async API)

> 💡 **提示**: 此API使用现代化的异步流模式监听剪贴板变化，不会阻塞主线程。

```rust
use clipboard_rs::{AsyncClipboardWatcher, ClipboardEvent, AsyncClipboardManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new async clipboard manager
    let clipboard = AsyncClipboardManager::new().await?;

    // Start watching clipboard changes
    let mut event_stream = clipboard.watch().await?;

    println!("Clipboard watcher started. Try copying some text to see events!");

    // Handle events in a loop
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

    Ok(())
}
```

### Stopping Clipboard Watcher (Modern Async API)

> 💡 **提示**: 新版本的异步API支持主动停止监听器，避免资源泄露。

```rust
use clipboard_rs::{AsyncClipboardWatcher, ClipboardEvent, AsyncClipboardManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new async clipboard manager
    let clipboard = AsyncClipboardManager::new().await?;

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

```

## X11 - Clipboard Read Timeout

By default, in X11 clipboard-rs implements a read timeout of 500 ms. You can override or disable this timeout by creating **ClipboardContext** using `new_with_options`:

```rust
#[cfg(unix)]
fn setup_clipboard() -> ClipboardContext {
	ClipboardContext::new_with_options(ClipboardContextX11Options { read_timeout: None }).unwrap()
}

#[cfg(not(unix))]
fn setup_clipboard(ctx: &mut ClipboardContext) -> ClipboardContext{
	ClipboardContext::new().unwrap()
}
```

## Contributing

You are welcome to submit PRs and issues and contribute your code or ideas to the project. Due to my limited level, the library may also have bugs. You are welcome to point them out and I will modify them as soon as possible.

## Thanks

- API design is inspired by [electron](https://www.electronjs.org/zh/docs/latest/api/clipboard)
- Linux part of the project code is referenced from [x11-clipboard](https://github.com/quininer/x11-clipboard/tree/master)

## Contract

if you have any questions, you can contact me by email: `swkzymlyy@gmail.com`

Chinese users can also contact me by wechatNo: `uniq_idx_church_lynn`

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.