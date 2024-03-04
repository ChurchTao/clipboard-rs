use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};

#[test]
fn test_string() {
    let ctx = ClipboardContext::new().unwrap();

    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let test_plain_txt = "hello world";
    ctx.set_text(test_plain_txt.to_string()).unwrap();
    assert!(ctx.has(ContentFormat::Text));
    assert_eq!(ctx.get_text().unwrap(), test_plain_txt);

    let test_rich_txt = "\x1b[1m\x1b[4m\x1b[31mHello, Rust!\x1b[0m";
    ctx.set_rich_text(test_rich_txt.to_string()).unwrap();
    assert!(ctx.has(ContentFormat::Rtf));
    assert_eq!(ctx.get_rich_text().unwrap(), test_rich_txt);

    let test_html = "<html><body><h1>Hello, Rust!</h1></body></html>";
    ctx.set_html(test_html.to_string()).unwrap();
    assert!(ctx.has(ContentFormat::Html));
    assert_eq!(ctx.get_html().unwrap(), test_html);

    let contents: Vec<ClipboardContent> = vec![
        ClipboardContent::new_with_data(ContentFormat::Text, test_plain_txt.as_bytes().to_vec()),
        ClipboardContent::new_with_data(ContentFormat::Rtf, test_rich_txt.as_bytes().to_vec()),
        ClipboardContent::new_with_data(ContentFormat::Html, test_html.as_bytes().to_vec()),
    ];
    ctx.set(contents).unwrap();
    assert!(ctx.has(ContentFormat::Text));
    assert!(ctx.has(ContentFormat::Rtf));
    assert!(ctx.has(ContentFormat::Html));
    assert_eq!(ctx.get_text().unwrap(), test_plain_txt);
    assert_eq!(ctx.get_rich_text().unwrap(), test_rich_txt);
    assert_eq!(ctx.get_html().unwrap(), test_html);
}
