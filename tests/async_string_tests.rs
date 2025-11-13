//! 异步文本相关功能的测试用例
//! 包括异步API的文本、HTML、RTF操作测试

#[cfg(feature = "async")]
use clipboard_rs::common::ContentData;
#[cfg(feature = "async")]
use clipboard_rs::{AsyncClipboardManager, ContentFormat};

/// 异步文本测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_string() {
	let clipboard = AsyncClipboardManager::new().await.unwrap();
	clipboard.clear().await.unwrap();

	let test_plain_txt = "Hello Async Rust!!!";
	clipboard.set_text(test_plain_txt).await.unwrap();
	assert!(clipboard.has(ContentFormat::Text).await.unwrap());
	assert_eq!(clipboard.get_text().await.unwrap(), test_plain_txt);

	let test_rich_txt = "{\\rtf1\\ansi\\b Hello, Async Rust!}";
	clipboard.set_rtf(test_rich_txt).await.unwrap();
	assert!(clipboard.has(ContentFormat::Rtf).await.unwrap());
	assert_eq!(clipboard.get_rtf().await.unwrap(), test_rich_txt);

	let test_html = "<h1>Hello, Async Rust!</h1>";
	clipboard.set_html(test_html).await.unwrap();
	assert!(clipboard.has(ContentFormat::Html).await.unwrap());
	assert_eq!(clipboard.get_html().await.unwrap(), test_html);
}

/// 异步构建器模式测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_builder() {
	let clipboard = AsyncClipboardManager::new().await.unwrap();
	clipboard.clear().await.unwrap();

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
		.await
		.unwrap();

	assert!(clipboard.has(ContentFormat::Text).await.unwrap());
	assert!(clipboard.has(ContentFormat::Html).await.unwrap());
	assert!(clipboard.has(ContentFormat::Rtf).await.unwrap());

	assert_eq!(clipboard.get_text().await.unwrap(), test_plain_txt);
	assert_eq!(clipboard.get_html().await.unwrap(), test_html);
	assert_eq!(clipboard.get_rtf().await.unwrap(), test_rtf);
}

/// 异步多种格式测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_multiple_formats() {
	let clipboard = AsyncClipboardManager::new().await.unwrap();
	clipboard.clear().await.unwrap();

	let test_plain_txt = "Hello Async Text";
	let test_rich_txt = "{\\rtf1 Hello Async RTF}";
	let test_html = "<h1>Hello Async HTML</h1>";

	let contents = clipboard
		.build_content()
		.with_text(test_plain_txt)
		.with_rtf(test_rich_txt)
		.with_html(test_html);

	clipboard.set(contents).await.unwrap();

	assert!(clipboard.has(ContentFormat::Text).await.unwrap());
	assert!(clipboard.has(ContentFormat::Rtf).await.unwrap());
	assert!(clipboard.has(ContentFormat::Html).await.unwrap());
	assert_eq!(clipboard.get_text().await.unwrap(), test_plain_txt);
	assert_eq!(clipboard.get_rtf().await.unwrap(), test_rich_txt);
	assert_eq!(clipboard.get_html().await.unwrap(), test_html);

	let content_arr = clipboard
		.get(&[ContentFormat::Text, ContentFormat::Rtf, ContentFormat::Html])
		.await
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
