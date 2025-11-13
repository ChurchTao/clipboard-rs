use clipboard_rs::{AsyncClipboardWatcher, ClipboardEvent, AsyncClipboardManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// 创建新的异步剪贴板管理器
	let clipboard = AsyncClipboardManager::new().await?;

	// 启动监视器
	let mut event_stream = clipboard.watch().await?;

	println!("Clipboard watcher started. Try copying some text to see events!");
	println!("This example will run for 30 seconds...");

	// 处理事件的异步任务
	let handle_events = tokio::spawn(async move {
		loop {
			match event_stream.next().await {
				Some(ClipboardEvent::Changed { formats }) => {
					println!("Clipboard changed! Available formats: {:?}", formats);
				}
				Some(ClipboardEvent::Cleared) => {
					println!("Clipboard cleared!");
				}
				Some(ClipboardEvent::Error(e)) => {
					eprintln!("Clipboard error: {:?}", e);
				}
				None => {
					println!("Event stream ended");
					break;
				}
			}
		}
	});

	// 在另一个任务中设置一些剪贴板内容来测试
	let test_clipboard = AsyncClipboardManager::new().await?;
	tokio::spawn(async move {
		tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
		test_clipboard
			.set_text("Hello from async watcher!")
			.await
			.unwrap();
		println!("Set text to clipboard: 'Hello from async watcher!'");

		tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
		test_clipboard
			.set_text("Another test message")
			.await
			.unwrap();
		println!("Set text to clipboard: 'Another test message'");
	});

	// 运行30秒后停止
	tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
	println!("Stopping clipboard watcher...");

	handle_events.abort();

	Ok(())
}
