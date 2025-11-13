//! 文本相关功能的测试用例
//! 包括同步和异步API的文本、HTML、RTF操作测试

use clipboard_rs::{SyncClipboardManager, ContentFormat};
use clipboard_rs::common::ContentData;

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

    clipboard.set_with_builder(contents).unwrap();

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

/// macOS平台特定测试：验证设置多种格式时应该创建单个项目
#[test]
#[ignore]
#[cfg(target_os = "macos")]
fn test_set_multiple_formats_is_one_item_macos() {
    // Import macOS-specific types needed for verification
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::{
        NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypeRTF, NSPasteboardTypeString,
    };

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

    // Action: Set the clipboard with multiple content types
    clipboard.set_with_builder(contents).unwrap();

    // Verification: Directly inspect the NSPasteboard to check the number of items.
    // The correct behavior is to have ONE item with multiple representations.
    // The buggy behavior creates THREE separate items.
    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();
        let items = pasteboard
            .pasteboardItems()
            .expect("Failed to get pasteboard items for verification");

        // [THIS IS THE KEY ASSERTION]
        // It will fail on the original code because `items.count()` will be 3.
        // It will pass on the fixed code because `items.count()` will be 1.
        assert_eq!(
            items.count(),
            1,
            "Setting multiple formats should create a single pasteboard item, but it created {}",
            items.count()
        );

        // [BONUS ASSERTIONS]
        // We can also verify that the single item contains all the correct types.
        let item = items.objectAtIndex(0);
        let types = item.types();

        assert!(
            unsafe { types.containsObject(NSPasteboardTypeString) },
            "The single pasteboard item should contain the String type"
        );
        assert!(
            unsafe { types.containsObject(NSPasteboardTypeRTF) },
            "The single pasteboard item should contain the RTF type"
        );
        assert!(
            unsafe { types.containsObject(NSPasteboardTypeHTML) },
            "The single pasteboard item should contain the HTML type"
        );
    });
}