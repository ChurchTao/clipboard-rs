//! 文本相关功能的测试用例
//! 包括同步和异步API的文本、HTML、RTF操作测试

use clipboard_rs::common::ContentData;
use clipboard_rs::{ContentFormat, SyncClipboardManager};

/// 同步文本测试
#[test]
fn test_sync_string() {
	let clipboard = SyncClipboardManager::new().unwrap();
	clipboard.clear().unwrap();

	let test_plain_txt = "hell@$#%^&U都98好的😊o Rust!!!";
	clipboard.set_text(test_plain_txt).unwrap();
	assert!(clipboard.has(ContentFormat::Text).unwrap());
	assert_eq!(clipboard.get_text().unwrap(), test_plain_txt);

	let test_rich_txt = "{\\rtf1\\ansi\\b Hello, Rust!}";
	clipboard.set_rtf(test_rich_txt).unwrap();
	assert!(clipboard.has(ContentFormat::Rtf).unwrap());
	assert_eq!(clipboard.get_rtf().unwrap(), test_rich_txt);

	let test_html = "<html><body><h1>Hello, Rust!</h1></body></html>";
	clipboard.set_html(test_html).unwrap();
	assert!(clipboard.has(ContentFormat::Html).unwrap());
	assert_eq!(clipboard.get_html().unwrap(), test_html);
}

/// 同步多种格式测试
#[test]
fn test_sync_multiple_formats() {
	let clipboard = SyncClipboardManager::new().unwrap();
	clipboard.clear().unwrap();

	let test_plain_txt = "Hello Text";
	let test_rich_txt = "{\\rtf1 Hello RTF}";
	let test_html = "<h1>Hello HTML</h1>";

	let contents = clipboard
		.build_content()
		.with_text(test_plain_txt)
		.with_rtf(test_rich_txt)
		.with_html(test_html);

	clipboard.set(contents).unwrap();

	assert!(clipboard.has(ContentFormat::Text).unwrap());
	assert!(clipboard.has(ContentFormat::Rtf).unwrap());
	assert!(clipboard.has(ContentFormat::Html).unwrap());
	assert_eq!(clipboard.get_text().unwrap(), test_plain_txt);
	assert_eq!(clipboard.get_rtf().unwrap(), test_rich_txt);
	assert_eq!(clipboard.get_html().unwrap(), test_html);

	let content_arr = clipboard
		.get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])
		.unwrap();

	assert_eq!(content_arr.len(), 3);
	for c in content_arr {
		let content_str = c.as_str().unwrap();
		match c.get_format() {
			ContentFormat::Text => assert_eq!(content_str, test_plain_txt),
			ContentFormat::Rtf => assert_eq!(content_str, test_rich_txt),
			ContentFormat::Html => assert_eq!(content_str, test_html),
			_ => panic!("unexpected format"),
		}
	}
}
