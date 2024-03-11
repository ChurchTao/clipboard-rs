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

fn main() {
	let ctx = ClipboardContext::new().unwrap();
	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let img = ctx.get_image();

	match img {
		Ok(img) => {
			img.save_to_path(format!("{}test.png", TMP_PATH).as_str())
				.unwrap();

			let resize_img = img.thumbnail(300, 300).unwrap();

			resize_img
				.save_to_path(format!("{}test_thumbnail.png", TMP_PATH).as_str())
				.unwrap();
		}
		Err(err) => {
			println!("err={}", err);
		}
	}
}
