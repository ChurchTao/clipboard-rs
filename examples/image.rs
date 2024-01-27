use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
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
