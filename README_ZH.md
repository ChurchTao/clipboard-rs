# clipboard-rs

clipboard-rs 是一个用 Rust 语言编写的跨平台库，用于获取和设置操作系统级别的剪贴板内容。它支持 Linux、Windows 和 MacOS。

## 功能支持

- 纯文本
- Html
- 富文本
- 图片（以 `PNG` 格式）
- 文件（以 `file-uri-list` 形式）
- 任意类型（通过指定类型标识符）可以先通过 `available_formats` 方法获取支持的类型

## 开发计划

- [x] MacOS 支持
- [x] Linux 支持 (x11)
- [x] Windows 支持

## 使用方法

在 `Cargo.toml` 中添加如下内容：

```toml
[dependencies]
clipboard-rs = "0.1.2"
```

## [更新日志](CHANGELOG.md)

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

    img.save_to_path("/tmp/test.png").unwrap();

    let resize_img = img.thumbnail(300, 300).unwrap();

    resize_img.save_to_path("/tmp/test_thumbnail.png").unwrap();
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

## 贡献

欢迎提交 PR 和 issue，为项目贡献你的代码或者想法。由于本人水平有限，库也可能会有 bug，欢迎大家指出，我会第一时间修改。

## 感谢

- API 设计灵感来自于 [electron](https://www.electronjs.org/zh/docs/latest/api/clipboard)
- Linux 部分项目代码参考自 [x11-clipboard](https://github.com/quininer/x11-clipboard/tree/master)

## 许可证

本项目遵循 MIT 许可证。详情请参阅 [LICENSE](LICENSE) 文件。
