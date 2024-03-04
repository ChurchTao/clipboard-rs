use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};

fn main() {
    let ctx = ClipboardContext::new().unwrap();

    let contents: Vec<ClipboardContent> = vec![
        ClipboardContent::new_with_data(ContentFormat::Text, "hello Rust".as_bytes().to_vec()),
        ClipboardContent::new_with_data(
            ContentFormat::Rtf,
            "\x1b[1m\x1b[4m\x1b[31mHello, Rust!\x1b[0m"
                .as_bytes()
                .to_vec(),
        ),
        ClipboardContent::new_with_data(
            ContentFormat::Html,
            "<html><body><h1>Hello, Rust!</h1></body></html>"
                .as_bytes()
                .to_vec(),
        ),
    ];

    ctx.set(contents).unwrap();

    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let has_rtf = ctx.has(ContentFormat::Rtf);
    println!("has_rtf={}", has_rtf);

    let rtf = ctx.get_rich_text().unwrap();

    println!("rtf={}", rtf);

    let has_html = ctx.has(ContentFormat::Html);
    println!("has_html={}", has_html);

    let html = ctx.get_html().unwrap();

    println!("html={}", html);

    let content = ctx.get_text().unwrap();

    println!("txt={}", content);
}
