use clipboard_rs::{Clipboard, ClipboardContext};

fn main() {
    let ctx = ClipboardContext::new().unwrap();
    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let has_image = ctx.has_image();
    let has_text = ctx.has_text();
    let has_rtf = ctx.has_rtf();
    let has_html = ctx.has_html();
    println!("has_image={}, has_text={}, has_rtf={}, has_html={}", has_image, has_text, has_rtf, has_html);

    let rtf = ctx.get_rich_text().unwrap();

    println!("rtf={}", rtf);

    let html = ctx.get_html().unwrap();

    println!("html={}", html);

    let content = ctx.get_text().unwrap();

    println!("txt={}", content);
}
