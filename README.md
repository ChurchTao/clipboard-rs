# clipboard-rs

clipboard-rs is a cross-platform library written in Rust for getting and setting the system-level clipboard content. It supports Linux, Windows, and MacOS.

[简体中文](README_ZH.md)

## Function Support

- Plain text
- Html
- Rich text
- Image (In `PNG` format)
- File (In `file-uri-list` format)
- Any type (by specifying the type identifier) can be obtained through the `available_formats` method

## Development Plan

- [x] MacOS Support
- [x] Linux Support (x11)
- [x] Windows Support

## Usage

Add the following content to your `Cargo.toml`:

```toml
[dependencies]
clipboard-rs = "0.1.1"
```

## Examples

### Simple Read and Write

```rust
use clipboard_rs::{Clipboard, ClipboardContext};

fn main() {
    let ctx = ClipboardContext::new().unwrap();
    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let rtf = ctx.get_rich_text().unwrap();

    println!("rtf={}", rtf);

    let html = ctx.get_html().unwrap();

    println!("html={}", html);

    let content = ctx.get_text().unwrap();

    println!("txt={}", content);
}

```

### Reading Images

```rust
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};

fn main() {
    let ctx = ClipboardContext::new().unwrap();
    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let img = ctx.get_image().unwrap();

    img.save_to_path("/tmp/test.png").unwrap();

    let resize_img = img.thumbnail(300, 300).unwrap();

    resize_img.save_to_path("/tmp/test_thumbnail.png").unwrap();
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
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardWatcher, ClipboardWatcherContext};
use std::{thread, time::Duration};

fn main() {
    let ctx = ClipboardContext::new().unwrap();
    let mut watcher = ClipboardWatcherContext::new().unwrap();

    watcher.add_handler(Box::new(move || {
        let content = ctx.get_text().unwrap();
        println!("read:{}", content);
    }));

    let watcher_shutdown = watcher.get_shutdown_channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        println!("stop watch!");
        watcher_shutdown.stop();
    });

    println!("start watch!");
    watcher.start_watch();
}

```

## Contributing

You are welcome to submit PRs and issues and contribute your code or ideas to the project. Due to my limited level, the library may also have bugs. You are welcome to point them out and I will modify them as soon as possible.

## Thanks

- API design is inspired by [electron](https://www.electronjs.org/zh/docs/latest/api/clipboard)
- Linux part of the project code is referenced from [x11-clipboard](https://github.com/quininer/x11-clipboard/tree/master)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
