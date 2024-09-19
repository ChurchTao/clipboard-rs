#[cfg(target_os = "linux")]
use clipboard_rs::ClipboardContextX11Options;
use clipboard_rs::{common::RustImage, Clipboard, ClipboardContext};

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

#[cfg(unix)]
fn setup_clipboard() -> ClipboardContext {
	ClipboardContext::new_with_options(ClipboardContextX11Options { read_timeout: None }).unwrap()
}

#[cfg(not(unix))]
fn setup_clipboard(ctx: &mut ClipboardContext) -> ClipboardContext {
	ClipboardContext::new().unwrap()
}

fn main() {
	let ctx = setup_clipboard();

	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let img = ctx.get_image();

	match img {
		Ok(img) => {
			let _ = img
				.save_to_path(format!("{}test.png", TMP_PATH).as_str())
				.map_err(|e| println!("save test.png err={}", e));

			let resize_img = img
				.thumbnail(300, 300)
				.map_err(|e| println!("thumbnail err={}", e))
				.unwrap();

			let _ = resize_img
				.save_to_path(format!("{}test_thumbnail.png", TMP_PATH).as_str())
				.map_err(|e| println!("save test_thumbnail.png err={}", e));
		}
		Err(err) => {
			println!("err={}", err);
		}
	}
}
