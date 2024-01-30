# clipboard-rs

clipboard-rs 是一个用 Rust 语言编写的跨平台库，用于获取和设置操作系统级别的剪贴板内容。它支持 Linux、Windows 和 MacOS。

目前，MacOS 的逻辑已经完成编写，我们正在继续开发 Linux 和 Windows 的逻辑。

[简体中文](README_ZH.md)

## 开发计划

- [x] MacOS 支持
- [ ] Linux 支持
- [ ] Windows 支持

## 使用方法

暂时还没用哦，还没发布，等发布第一个 release 版本后再来看看吧。
在 `Cargo.toml` 中添加如下内容：

```toml
[dependencies]
clipboard-rs = "0.0.1"
```

## 示例

### 简单读写

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

### 读取图片

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

### 读取任意类型

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

### 监听剪贴板变化

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

## 许可证

本项目遵循 MIT 许可证。详情请参阅 [LICENSE](LICENSE) 文件。
