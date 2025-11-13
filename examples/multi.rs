use clipboard_rs::common::ContentData;
use clipboard_rs::{ContentFormat, SyncClipboardManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create new synchronous clipboard manager (requires "text" feature)
	let clipboard = SyncClipboardManager::new()?;

	clipboard.set_with_builder(
		clipboard
			.build_content()
			.with_text("Hello, Rust!")
			.with_html("<h1>Hello, Rust!</h1>")
			.with_rtf("{\\rtf1\\ansi\\b Hello, Rust!}"),
	)?;

	let formats = clipboard.available_formats()?;
	println!("Available formats: {:?}", formats);

	let read = clipboard.get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])?;

	for c in read {
		println!("{}", c.as_str()?);
	}

	Ok(())
}
