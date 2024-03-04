use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};

fn main() {
    let ctx = ClipboardContext::new().unwrap();

    // change the file paths to your own
    let files = vec![
        "file:///Users/churcht/Downloads/IMG_2343.JPG".to_string(),
        "file:///Users/churcht/Downloads/cmdk-main.zip".to_string(),
    ];

    ctx.set_files(files).unwrap();

    let types = ctx.available_formats().unwrap();
    println!("{:?}", types);

    let has = ctx.has(ContentFormat::Files);
    println!("has_files={}", has);

    let files = ctx.get_files().unwrap();
    println!("{:?}", files);
}
