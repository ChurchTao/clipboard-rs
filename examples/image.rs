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

    let resize_img = img.thumbnail(300, 300).unwrap();

    println!(
        "size={:?},byte len={}",
        resize_img.get_size(),
        resize_img.get_bytes().len()
    );

    resize_img.save_to_file("/tmp/test_resize.png").unwrap();
}
