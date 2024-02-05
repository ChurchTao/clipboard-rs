use clipboard_rs::{
    common::{RustImage, RustImageData},
    Clipboard, ClipboardContext, ContentFormat,
};
use image::{ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

#[test]
fn test_image() {
    let ctx = ClipboardContext::new().unwrap();

    // 创建一个 100x100 大小的纯红色图像
    let width = 100;
    let height = 100;
    let image_buffer: RgbaImage =
        ImageBuffer::from_fn(width, height, |_x, _y| Rgba([255u8, 0u8, 0u8, 255u8]));
    let mut buf = Cursor::new(Vec::new());
    image_buffer
        .write_to(&mut buf, image::ImageOutputFormat::Png)
        .expect("Failed to encode image as PNG");

    let rust_img = RustImageData::from_bytes(&buf.clone().into_inner()).unwrap();

    ctx.set_image(rust_img).unwrap();

    assert!(ctx.has(ContentFormat::Image));

    let clipboard_img = ctx.get_image().unwrap();

    assert_eq!(
        clipboard_img.to_png().unwrap().get_bytes(),
        &buf.into_inner()
    );
}
