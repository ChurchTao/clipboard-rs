# clipboard-rs

clipboard-rs is a cross-platform library written in Rust for getting and setting the system-level clipboard content. It supports Linux, Windows, and MacOS.

Currently, the logic for MacOS has been completed, and we are continuing to develop the logic for Linux and Windows.

[简体中文](README_ZH.md)

## Development Plan

- [x] MacOS Support
- [ ] Linux Support
- [ ] Windows Support

## Usage

It's not ready for use yet, as it hasn't been released. Please check back after the first release version is published.
Add the following content to your `Cargo.toml`:

```toml
[dependencies]
clipboard-rs = "0.0.1"
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

    println!(
        "size={:?},byte len={}",
        img.get_size(),
        img.get_bytes().len()
    );

    img.save_to_file("/tmp/test.png").unwrap();
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

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
