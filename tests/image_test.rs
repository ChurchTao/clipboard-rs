use clipboard_rs::{
	common::{RustImage, RustImageData},
	Clipboard, ClipboardContext, ContentFormat,
};

#[test]
fn test_image() {
	let ctx = ClipboardContext::new().unwrap();

	let rust_img = RustImageData::from_path("tests/test.png").unwrap();

	let binding = RustImageData::from_path("tests/test.png").unwrap();

	let rust_img_bytes = binding.as_bytes();

	ctx.set_image(rust_img).unwrap();

	assert!(ctx.has(ContentFormat::Image));

	let clipboard_img = ctx.get_image().unwrap();

	assert_eq!(
		clipboard_img.to_png().unwrap().get_bytes().len(),
		rust_img_bytes.len()
	);
}
