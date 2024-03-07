use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};

fn main() {
	let ctx = ClipboardContext::new().unwrap();
	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let img = ctx.get_image().unwrap();

	img.save_to_path("/tmp/test.png").unwrap();

	let resize_img = img.thumbnail(300, 300).unwrap();

	resize_img.save_to_path("/tmp/test_thumbnail.png").unwrap();
}
