use clipboard_rs::{
    common::ContentData, Clipboard, ClipboardContent, ClipboardContext, ContentFormat,
};

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

    let read = ctx
        .get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])
        .unwrap();

    for c in read {
        println!("{}", c.as_str().unwrap());
    }
}
