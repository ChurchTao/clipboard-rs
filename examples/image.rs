#[cfg(target_os = "linux")]
use clipboard_rs::ClipboardContextX11Options;
use clipboard_rs::SyncClipboardManager;

#[cfg(target_os = "macos")]
const TMP_PATH: &str = "/tmp/";
#[cfg(target_os = "windows")]
const TMP_PATH: &str = "C:\\Windows\\Temp\\";
#[cfg(all(
	unix,
	not(any(
		target_os = "macos",
		target_os = "ios",
		target_os = "android",
		target_os = "emscripten"
	))
))]
const TMP_PATH: &str = "/tmp/";
// ios
#[cfg(any(target_os = "ios", target_os = "android"))]
const TMP_PATH: &str = "/tmp/";

#[cfg(target_os = "linux")]
fn setup_clipboard() -> clipboard_rs::ClipboardContext {
	clipboard_rs::ClipboardContext::new_with_options(clipboard_rs::ClipboardContextX11Options {
		read_timeout: None,
	})
	.unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Create new synchronous clipboard manager (requires "image" feature)
	let clipboard = SyncClipboardManager::new()?;

	match clipboard.get_image() {
		Ok(img) => {
			img.save_to_path_sync(format!("{}test.png", TMP_PATH).as_str())?;
			println!("Image saved to {}test.png", TMP_PATH);

			let resize_img = img.thumbnail_sync(300, 300)?;
			resize_img.save_to_path_sync(format!("{}test_thumbnail.png", TMP_PATH).as_str())?;
			println!("Thumbnail saved to {}test_thumbnail.png", TMP_PATH);
		}
		Err(err) => {
			println!("Failed to get image from clipboard: {}", err);
		}
	}

	Ok(())
}
