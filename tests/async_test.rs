#[cfg(test)]
mod tests {
	use clipboard_rs::{ClipboardManager, ContentFormat};

	#[tokio::test]
	async fn test_async_string() {
		let clipboard = ClipboardManager::new().await.unwrap();
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

	#[tokio::test]
	async fn test_async_builder() {
		let clipboard = ClipboardManager::new().await.unwrap();
		clipboard.clear().await.unwrap();

		let test_plain_txt = "Hello Builder API!";
		let test_html = "<h1>Hello Builder API!</h1>";
		let test_rtf = "{\\rtf1\\ansi\\b Hello Builder API!}";

		clipboard
			.set_with_builder(
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
}
