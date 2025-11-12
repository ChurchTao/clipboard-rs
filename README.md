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

- `default` - Enable image support
- `image` - Enable basic image support
- `async-image` - Enable modern async image processing with background thread support

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
clipboard-rs = "0.3.1"
```

## [CHANGELOG](CHANGELOG.md)

## Examples

### All Usage Examples

[Examples](examples)

### Simple Read and Write (Legacy API) ⚠️ *已废弃*

> ⚠️ **警告**: 此API已在v0.4.0版本中标记为废弃，将在未来版本中移除。请使用下方的现代化异步API。

```rust
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};

fn main() {
	let ctx = ClipboardContext::new().unwrap();
	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let has_rtf = ctx.has(ContentFormat::Rtf);
	println!("has_rtf={}", has_rtf);

	let rtf = ctx.get_rich_text().unwrap_or("".to_string());

	println!("rtf={}", rtf);

	let has_html = ctx.has(ContentFormat::Html);
	println!("has_html={}", has_html);

	let html = ctx.get_html().unwrap_or("".to_string());

	println!("html={}", html);

	let content = ctx.get_text().unwrap_or("".to_string());

	println!("txt={}", content);
}

```

### Simple Read and Write (Modern Async API)

```rust
use clipboard_rs::{ClipboardManager, ContentFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = ClipboardManager::new().await?;

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

### Reading Images

```rust
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};

const TMP_PATH: &str = "/tmp/";

fn main() {
	let ctx = ClipboardContext::new().unwrap();
	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let img = ctx.get_image();

	match img {
		Ok(img) => {
			img.save_to_path(format!("{}test.png", TMP_PATH).as_str())
				.unwrap();

			let resize_img = img.thumbnail(300, 300).unwrap();

			resize_img
				.save_to_path(format!("{}test_thumbnail.png", TMP_PATH).as_str())
				.unwrap();
		}
		Err(err) => {
			println!("err={}", err);
		}
	}
}
```

### Reading Images (Modern Async API)

To use the modern async image API, enable the `async-image` feature:

```bash
cargo run --example image_modern --features async-image
```

```rust
//! Modern image processing example
//! Requires async-image feature: `cargo run --example image_modern --features async-image`

#[cfg(feature = "async-image")]
use clipboard_rs::{ClipboardImage, ClipboardManager};

#[cfg(feature = "async-image")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new clipboard manager
    let clipboard = ClipboardManager::new().await?;

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

    // Set image to clipboard
    clipboard.set_image(clipboard_image).await?;
    println!("Image set to clipboard successfully!");

    // Get image from clipboard
    match clipboard.get_image().await {
        Ok(image) => {
            println!("Got image from clipboard!");
            println!("Image dimensions: {:?}", image.dimensions());

            // Save image to file
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
use clipboard_rs::{Clipboard, ClipboardContext};

fn main() {
    let ctx = ClipboardContext::new().unwrap();
    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let buffer = ctx.get_buffer("public.html").unwrap();

    let string = String::from_utf8(buffer).unwrap();

    println!("{}", string);
}

```

### Listening to Clipboard Changes

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
