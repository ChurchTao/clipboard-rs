use clipboard_rs::{
    common::ContentData, Clipboard, ClipboardContent, ClipboardContext, ContentFormat,
};

fn main() {
    let ctx = ClipboardContext::new().unwrap();

    let contents: Vec<ClipboardContent> = vec![
        ClipboardContent::Text("hell@$#%^&U都98好的😊o Rust!!!".to_string()),
        ClipboardContent::Rtf("\x1b[1m\x1b[4m\x1b[31mHello, Rust!\x1b[0m".to_string()),
        ClipboardContent::Html("<html><body><h1>Hello, Rust!</h1></body></html>".to_string()),
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
