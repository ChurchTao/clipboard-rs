use clipboard_rs::{
    common::ContentData, Clipboard, ClipboardContent, ClipboardContext, ContentFormat,
};

#[test]
fn test_string() {
    let ctx = ClipboardContext::new().unwrap();

    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let test_plain_txt = "hell@$#%^&U都98好的😊o Rust!!!";
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

    let content_arr = ctx
        .get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])
        .unwrap();

    assert_eq!(content_arr.len(), 3);
    for c in content_arr {
        let content_str = c.as_str().unwrap();
        match c.get_format() {
            ContentFormat::Text => assert_eq!(content_str, test_plain_txt),
            ContentFormat::Rtf => assert_eq!(content_str, test_rich_txt),
            ContentFormat::Html => assert_eq!(content_str, test_html),
            _ => panic!("unexpected format"),
        }
    }
}
