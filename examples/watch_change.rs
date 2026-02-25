use clipboard_rs::{Clipboard, ClipboardContext, ClipboardWatcherBuilder};
use std::{thread, time::Duration};

fn main() {
	let ctx = ClipboardContext::new().unwrap();

	let watcher = ClipboardWatcherBuilder::new()
		.on_change(move || {
			println!(
				"on_clipboard_change, txt = {}",
				ctx.get_text().unwrap_or_default()
			);
		})
		.spawn()
		.unwrap();

	thread::sleep(Duration::from_secs(5));
	println!("stop watch!");
	watcher.stop().unwrap();
}
