//! 构建器模式相关功能的测试用例
//! 包括同步和异步API的构建器模式操作测试

use clipboard_rs::common::ContentData;
use clipboard_rs::{ContentFormat, SyncClipboardManager};

/// 同步构建器模式测试
#[test]
fn test_sync_builder() {
	let clipboard = SyncClipboardManager::new().unwrap();
	clipboard.clear().unwrap();

	let test_plain_txt = "Hello Builder API!";
	let test_html = "<h1>Hello Builder API!</h1>";
	let test_rtf = "{\\rtf1\\ansi\\b Hello Builder API!}";

	clipboard
		.set(
			clipboard
				.build_content()
				.with_text(test_plain_txt)
				.with_html(test_html)
				.with_rtf(test_rtf),
		)
		.unwrap();

	assert!(clipboard.has(ContentFormat::Text).unwrap());
	assert!(clipboard.has(ContentFormat::Html).unwrap());
	assert!(clipboard.has(ContentFormat::Rtf).unwrap());

	assert_eq!(clipboard.get_text().unwrap(), test_plain_txt);
	assert_eq!(clipboard.get_html().unwrap(), test_html);
	assert_eq!(clipboard.get_rtf().unwrap(), test_rtf);
}

/// 同步构建器模式复杂测试
#[test]
fn test_sync_builder_complex() {
	let clipboard = SyncClipboardManager::new().unwrap();
	clipboard.clear().unwrap();

	// 构建包含多种内容的剪贴板数据
	let builder = clipboard
		.build_content()
		.with_text("Sample text content")
		.with_html("<p>Sample HTML content</p>")
		.with_rtf("{\\rtf1\\ansi Sample RTF content}")
		.with_custom("custom.format", b"Custom data".to_vec());

	clipboard.set(builder).unwrap();

	// 验证所有格式都已设置
	assert!(clipboard.has(ContentFormat::Text).unwrap());
	assert!(clipboard.has(ContentFormat::Html).unwrap());
	assert!(clipboard.has(ContentFormat::Rtf).unwrap());
	// 注意：自定义格式可能无法通过has()方法检测，因为这取决于平台实现

	// 获取并验证内容
	let contents = clipboard
		.get(&[ContentFormat::Text, ContentFormat::Html, ContentFormat::Rtf])
		.unwrap();

	assert_eq!(contents.len(), 3);

	let mut found_text = false;
	let mut found_html = false;
	let mut found_rtf = false;

	for content in contents {
		match content.get_format() {
			ContentFormat::Text => {
				assert_eq!(content.as_str().unwrap(), "Sample text content");
				found_text = true;
			}
			ContentFormat::Html => {
				assert_eq!(content.as_str().unwrap(), "<p>Sample HTML content</p>");
				found_html = true;
			}
			ContentFormat::Rtf => {
				assert_eq!(
					content.as_str().unwrap(),
					"{\\rtf1\\ansi Sample RTF content}"
				);
				found_rtf = true;
			}
			_ => panic!("Unexpected content format"),
		}
	}

	assert!(found_text);
	assert!(found_html);
	assert!(found_rtf);
}
