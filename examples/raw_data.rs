use clipboard_rs::{ContentFormat, SyncClipboardManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create new synchronous clipboard manager (requires "text" feature)
	let clipboard = SyncClipboardManager::new()?;

	let formats = clipboard.available_formats()?;
	println!("Available formats: {:?}", formats);

	// Check if HTML format is available
	let has_html = clipboard.has(ContentFormat::Html)?;
	if has_html {
		let html_content = clipboard.get_html()?;
		println!("HTML content: {}", html_content);
	}

	// Read raw data with custom format
	if formats.contains(&"public.html".to_string()) {
		let buffer = clipboard.get_raw("public.html")?;
		let string = String::from_utf8(buffer)?;
		println!("Raw HTML: {}", string);
	}

	Ok(())
}
