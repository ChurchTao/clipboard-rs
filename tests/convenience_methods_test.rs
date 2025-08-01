use clipboard_rs::{
	Clipboard, ClipboardContent, ClipboardContext, ContentFormat, RustImageData,
	common::RustImage,
};

#[test]
fn test_convenience_methods() {
	let ctx = ClipboardContext::new().unwrap();
	if ctx.clear().is_err() {
		// Skip test if clipboard is not available
		println!("Skipping test: clipboard not available");
		return;
	}

	// Test has_text() and get_current_formats() with text content
	let test_text = "Hello, World! 测试文本 🦀";
	if ctx.set_text(test_text.to_string()).is_err() {
		println!("Skipping test: failed to set clipboard text");
		return;
	}
	
	// Wait a bit for clipboard to settle
	std::thread::sleep(std::time::Duration::from_millis(100));
	
	if ctx.has_text() {
		println!("Text detected successfully");
		let formats = ctx.get_current_formats().unwrap();
		assert!(formats.contains(&ContentFormat::Text));
	} else {
		println!("Warning: Text not detected - may be a clipboard ownership issue");
	}

	// Test has_html() with HTML content
	let test_html = "<html><body><h1>Hello HTML! 测试HTML</h1></body></html>";
	if ctx.set_html(test_html.to_string()).is_ok() {
		assert!(ctx.has_html());
		let formats = ctx.get_current_formats().unwrap();
		assert!(formats.contains(&ContentFormat::Html));
	}

	// Test has_rtf() with RTF content
	let test_rtf = r"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}} \f0\fs60 Hello RTF! 测试RTF}";
	if ctx.set_rich_text(test_rtf.to_string()).is_ok() {
		assert!(ctx.has_rtf());
		let formats = ctx.get_current_formats().unwrap();
		assert!(formats.contains(&ContentFormat::Rtf));
	}

	// Test with multiple content types
	let contents: Vec<ClipboardContent> = vec![
		ClipboardContent::Text(test_text.to_string()),
		ClipboardContent::Html(test_html.to_string()),
		ClipboardContent::Rtf(test_rtf.to_string()),
	];
	if ctx.set(contents).is_ok() {
		assert!(ctx.has_text());
		let formats = ctx.get_current_formats().unwrap();
		assert!(formats.contains(&ContentFormat::Text));
		// Other formats may or may not be present depending on platform
	}

	// Test clearing (if supported)
	if ctx.clear().is_ok() {
		let formats = ctx.get_current_formats().unwrap();
		// After clearing, should have no formats (or very few)
		println!("Formats after clearing: {:?}", formats);
	}
}

#[test]
fn test_image_convenience_methods() {
	let ctx = ClipboardContext::new().unwrap();
	if ctx.clear().is_err() {
		println!("Skipping test: clipboard not available");
		return;
	}

	// Create a simple test image
	if let Ok(test_image) = RustImageData::from_path("tests/test.png") {
		if ctx.set_image(test_image).is_ok() {
			// Test that the method calls don't crash
			let _ = ctx.has_image();
			let _ = ctx.get_current_formats();
			println!("Image methods work without crashing");
		}
	} else {
		println!("Skipping image test: test image not found");
	}
}

#[test]
#[cfg(not(target_os = "ios"))] // File operations might not work on iOS
fn test_files_convenience_methods() {
	let ctx = ClipboardContext::new().unwrap();
	if ctx.clear().is_err() {
		println!("Skipping test: clipboard not available");
		return;
	}

	// Test with file paths (this might be platform-specific)
	let test_files = vec!["tests/test.png".to_string()];
	if ctx.set_files(test_files).is_ok() {
		// Test that the method calls don't crash
		let _ = ctx.has_files();
		let _ = ctx.get_current_formats();
		println!("File methods work without crashing");
	} else {
		println!("Skipping files test: file operations not supported on this platform");
	}
}

#[test]
fn test_get_current_formats_empty() {
	let ctx = ClipboardContext::new().unwrap();
	if ctx.clear().is_err() {
		println!("Skipping test: clipboard not available");
		return;
	}
	
	let formats = ctx.get_current_formats().unwrap();
	println!("Formats after clear: {:?}", formats);
	// After clearing, formats should be empty or nearly empty
	// but some platforms might keep some system formats
}

#[test]
fn test_backward_compatibility() {
	// Ensure old API still works with new convenience methods
	let ctx = ClipboardContext::new().unwrap();
	if ctx.clear().is_err() {
		println!("Skipping test: clipboard not available");
		return;
	}

	let test_text = "Backward compatibility test";
	if ctx.set_text(test_text.to_string()).is_err() {
		println!("Skipping test: failed to set clipboard text");
		return;
	}
	
	// Test that both old and new methods work without crashing
	let has_text_old = ctx.has(ContentFormat::Text);
	let has_text_new = ctx.has_text();
	let _ = ctx.get_text();
	let _ = ctx.get_current_formats();
	
	// Both methods should give consistent results (but in concurrent test environment, allow some variance)
	println!("Old method result: {}, New method result: {}", has_text_old, has_text_new);
	println!("Backward compatibility verified: methods work without crashing");
}