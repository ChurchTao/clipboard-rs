use clipboard_rs::{SyncClipboardManager, ContentFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new synchronous clipboard manager (requires "text" feature)
    let clipboard = SyncClipboardManager::new()?;

    // change the file paths to your own
    // let files = vec![
    //     "file:///home/parallels/clipboard-rs/Cargo.toml".to_string(),
    //     "file:///home/parallels/clipboard-rs/CHANGELOG.md".to_string(),
    // ];

    // clipboard.set_files(&files)?;

    let formats = clipboard.available_formats()?;
    println!("Available formats: {:?}", formats);

    let has_files = clipboard.has(ContentFormat::Files)?;
    println!("has_files={}", has_files);

    let files = clipboard.get_files().unwrap_or_default();
    println!("files: {:?}", files);

    Ok(())
}
