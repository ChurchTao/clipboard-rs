use clipboard_rs::{
	common::ContentData, Clipboard, ClipboardContent, ClipboardContext, ContentFormat,
};

#[test]
fn test_string() {
	let ctx = ClipboardContext::new().unwrap();
	ctx.clear().unwrap();

	let test_plain_txt = "hell@$#%^&U都98好的😊o Rust!!!";
	ctx.set_text(test_plain_txt.to_string()).unwrap();
	assert!(ctx.has(ContentFormat::Text));
	assert_eq!(ctx.get_text().unwrap(), test_plain_txt);

	let test_rich_txt = "\x1b[1m\x1b[4m\x1b[31mHello, Rust!\x1b[0m";
	ctx.set_rich_text(test_rich_txt.to_string()).unwrap();
	assert!(ctx.has(ContentFormat::Rtf));
	assert_eq!(ctx.get_rich_text().unwrap(), test_rich_txt);

	let test_html = "<html><body><h1>Hello, Rust!</h1></body></html>";
	ctx.set_html(test_html.to_string()).unwrap();
	assert!(ctx.has(ContentFormat::Html));
	assert_eq!(ctx.get_html().unwrap(), test_html);

	let contents: Vec<ClipboardContent> = vec![
		ClipboardContent::Text(test_plain_txt.to_string()),
		ClipboardContent::Rtf(test_rich_txt.to_string()),
		ClipboardContent::Html(test_html.to_string()),
	];
	ctx.set(contents).unwrap();
	assert!(ctx.has(ContentFormat::Text));
	assert!(ctx.has(ContentFormat::Rtf));
	assert!(ctx.has(ContentFormat::Html));
	assert_eq!(ctx.get_text().unwrap(), test_plain_txt);
	assert_eq!(ctx.get_rich_text().unwrap(), test_rich_txt);
	assert_eq!(ctx.get_html().unwrap(), test_html);

	let content_arr = ctx
		.get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])
		.unwrap();

	assert_eq!(content_arr.len(), 3);
	for c in content_arr {
		let content_str = c.as_str().unwrap();
		match c.get_format() {
			ContentFormat::Text => assert_eq!(content_str, test_plain_txt),
			ContentFormat::Rtf => assert_eq!(content_str, test_rich_txt),
			ContentFormat::Html => assert_eq!(content_str, test_html),
			_ => panic!("unexpected format"),
		}
	}
}

#[test]
#[ignore]
#[cfg(target_os = "macos")]
fn test_set_multiple_formats_is_one_item_macos() {
	// Import macOS-specific types needed for verification
	use objc2::rc::autoreleasepool;
	use objc2_app_kit::{
		NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypeRTF, NSPasteboardTypeString,
	};

	let ctx = ClipboardContext::new().unwrap();

	ctx.clear().unwrap();

	let test_plain_txt = "Hello Text";
	let test_rich_txt = "{\\rtf1 Hello RTF}";
	let test_html = "<h1>Hello HTML</h1>";

	let contents: Vec<ClipboardContent> = vec![
		ClipboardContent::Text(test_plain_txt.to_string()),
		ClipboardContent::Rtf(test_rich_txt.to_string()),
		ClipboardContent::Html(test_html.to_string()),
	];

	// Action: Set the clipboard with multiple content types
	ctx.set(contents).unwrap();

	// Verification: Directly inspect the NSPasteboard to check the number of items.
	// The correct behavior is to have ONE item with multiple representations.
	// The buggy behavior creates THREE separate items.
	autoreleasepool(|_| {
		let pasteboard = unsafe { NSPasteboard::generalPasteboard() };
		let items = unsafe { pasteboard.pasteboardItems() }
			.expect("Failed to get pasteboard items for verification");

		// [THIS IS THE KEY ASSERTION]
		// It will fail on the original code because `items.count()` will be 3.
		// It will pass on the fixed code because `items.count()` will be 1.
		assert_eq!(
			items.count(),
			1,
			"Setting multiple formats should create a single pasteboard item, but it created {}",
			items.count()
		);

		// [BONUS ASSERTIONS]
		// We can also verify that the single item contains all the correct types.
		let item = items.objectAtIndex(0);
		let types = unsafe { item.types() };

		assert!(
			unsafe { types.containsObject(NSPasteboardTypeString) },
			"The single pasteboard item should contain the String type"
		);
		assert!(
			unsafe { types.containsObject(NSPasteboardTypeRTF) },
			"The single pasteboard item should contain the RTF type"
		);
		assert!(
			unsafe { types.containsObject(NSPasteboardTypeHTML) },
			"The single pasteboard item should contain the HTML type"
		);
	});
}
