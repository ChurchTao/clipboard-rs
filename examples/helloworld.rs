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
