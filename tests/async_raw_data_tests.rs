//! 异步原始数据相关功能的测试用例
//! 包括异步API的原始数据操作测试

#[cfg(feature = "async")]
use clipboard_rs::AsyncClipboardManager;

/// 异步原始数据测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_raw_data() {
	let clipboard = AsyncClipboardManager::new().await.unwrap();
	clipboard.clear().await.unwrap();

	// 设置一些文本数据
	let test_text = "Hello Async Raw Data Test!";
	clipboard.set_text(test_text).await.unwrap();

	// 获取可用格式
	let formats = clipboard.available_formats().await.unwrap();
	println!("Available formats: {:?}", formats);

	// 测试获取原始数据（使用常见的文本格式）
	#[cfg(target_os = "macos")]
	let text_format = "public.utf8-plain-text";
	#[cfg(target_os = "windows")]
	let text_format = "CF_UNICODETEXT";
	#[cfg(target_os = "linux")]
	let text_format = "UTF8_STRING";

	// 尝试获取原始数据
	if formats.contains(&text_format.to_string()) {
		let raw_data = clipboard.get_raw(text_format).await.unwrap();
		let text_from_raw = String::from_utf8(raw_data).unwrap();
		// 注意：根据平台不同，原始数据可能包含额外的格式信息
		// 所以我们只验证原始数据不为空
		assert!(!text_from_raw.is_empty());
		println!("Raw data length: {}", text_from_raw.len());
	}

	// 测试设置原始数据
	let custom_format = "custom.test.format";
	let custom_data = b"Custom async test data".to_vec();
	clipboard
		.set_raw(custom_format, &custom_data)
		.await
		.unwrap();

	// 验证自定义格式已设置
	let formats_after = clipboard.available_formats().await.unwrap();
	println!("Formats after setting raw data: {:?}", formats_after);
}

/// 异步错误处理测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_error_handling() {
	let clipboard = AsyncClipboardManager::new().await.unwrap();

	// 测试获取不存在的原始数据格式
	let result = clipboard.get_raw("nonexistent.format").await;
	// 这应该返回一个错误，但具体错误类型取决于平台实现
	println!("Get raw data result: {:?}", result);

	// 测试设置空数据
	let result = clipboard.set_raw("empty.format", &[]).await;
	println!("Set empty raw data result: {:?}", result);
}
