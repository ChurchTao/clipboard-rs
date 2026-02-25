use clipboard_rs::ClipboardWatcherBuilder;

#[test]
fn watcher_builder_can_register_callback() {
	let builder = ClipboardWatcherBuilder::new().on_change(|| {
		let _ = 1 + 1;
	});
	let _ = builder;
}
