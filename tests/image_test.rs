use clipboard_rs::{
	common::{RustImage, RustImageData},
	Clipboard, ClipboardContext, ContentFormat,
};

#[test]
fn test_image() {
	let ctx = ClipboardContext::new().unwrap();

	let rust_img = RustImageData::from_path("tests/test.png").unwrap();

	let binding = RustImageData::from_path("tests/test.png").unwrap();

	let rust_img_bytes = binding.to_png().unwrap();

	ctx.set_image(rust_img).unwrap();

	assert!(ctx.has(ContentFormat::Image));

	let clipboard_img = ctx.get_image().unwrap();

	assert_eq!(clipboard_img.to_png().unwrap().get_bytes(), rust_img_bytes.get_bytes());
}
