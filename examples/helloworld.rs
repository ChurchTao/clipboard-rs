use clipboard_rs::{ContentFormat, SyncClipboardManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create new synchronous clipboard manager (requires "text" feature)
	let clipboard = SyncClipboardManager::new()?;

	let formats = clipboard.available_formats()?;
	println!("Available formats: {:?}", formats);

	let has_rtf = clipboard.has(ContentFormat::Rtf)?;
	println!("has_rtf={}", has_rtf);

	let rtf = clipboard.get_rtf().unwrap_or_default();
	println!("rtf={}", rtf);

	let has_html = clipboard.has(ContentFormat::Html)?;
	println!("has_html={}", has_html);

	let html = clipboard.get_html().unwrap_or_default();
	println!("html={}", html);

	let content = clipboard.get_text().unwrap_or_default();
	println!("txt={}", content);

	// Using the fluent builder API
	clipboard.set(
		clipboard
			.build_content()
			.with_text("Hello, World!")
			.with_html("<h1>Hello, World!</h1>")
			.with_rtf(r"{\rtf1\ansi\b Hello, World!}"),
	)?;

	Ok(())
}
