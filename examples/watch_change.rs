use clipboard_rs::{Clipboard, ClipboardContext, ClipboardWatcher, ClipboardWatcherContext};
use std::{thread, time::Duration};

fn main() {
	let ctx = ClipboardContext::new().unwrap();
	let mut watcher = ClipboardWatcherContext::new().unwrap();

	watcher.add_handler(Box::new(move || {
		let content = ctx.get_text().unwrap();
		println!("read:{}", content);
	}));

	let watcher_shutdown = watcher.get_shutdown_channel();

	thread::spawn(move || {
		thread::sleep(Duration::from_secs(5));
		println!("stop watch!");
		watcher_shutdown.stop();
	});

	println!("start watch!");
	watcher.start_watch();
}
