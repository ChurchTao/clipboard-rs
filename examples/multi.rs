use clipboard_rs::common::ContentData;
use clipboard_rs::{ClipboardContent, ContentFormat, SyncClipboardManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create new synchronous clipboard manager (requires "text" feature)
	let clipboard = SyncClipboardManager::new()?;

	let contents: Vec<ClipboardContent> = vec![
		ClipboardContent::Text("hell@$#%^&U都98好的😊o Rust!!!".to_string()),
		ClipboardContent::Rtf("{\\rtf1\\ansi\\b Hello, Rust!}".to_string()),
		ClipboardContent::Html("<html><body><h1>Hello, Rust!</h1></body></html>".to_string()),
	];

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
