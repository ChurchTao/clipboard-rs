use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext, RustImageData};

const PNG_IMAGE_PATH: &str = "C:\\Users\\78663\\Documents\\GitHub\\clipboard-rs\\tests\\test.png";
const BMP_IMAGE_PATH: &str = "C:\\Users\\78663\\Documents\\GitHub\\clipboard-rs\\tests\\test.bmp";

fn main() {
	let img = RustImageData::from_path(PNG_IMAGE_PATH).unwrap();

	// let resized_img = img.to_bitmap().unwrap();

	// // let _ = resized_img.save_to_path(BMP_IMAGE_PATH).unwrap();
	let ctx = ClipboardContext::new().unwrap();
	// let _ = ctx.set_image(img);

	let formats = ctx.available_formats().unwrap();
	println!("{:?}", formats);
}
