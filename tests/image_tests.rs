//! 图像相关功能的测试用例
//! 包括同步和异步API的图像操作测试

#[cfg(feature = "image")]
use clipboard_rs::{SyncClipboardManager, ContentFormat, ClipboardImage};

/// 同步图像测试
#[test]
#[cfg(feature = "image")]
fn test_sync_image() {
    let clipboard = SyncClipboardManager::new().unwrap();
    clipboard.clear().unwrap();

    let image = ClipboardImage::from_path_sync("tests/test.png").unwrap();
    let image_bytes = image.to_png_sync().unwrap();

    clipboard.set_image(image).unwrap();
    assert!(clipboard.has(ContentFormat::Image).unwrap());

    let clipboard_img = clipboard.get_image().unwrap();
    assert_eq!(
        clipboard_img.to_png_sync().unwrap().len(),
        image_bytes.len()
    );
}

/// 同步图像处理测试
#[test]
#[cfg(feature = "image")]
fn test_sync_image_processing() {
    let clipboard = SyncClipboardManager::new().unwrap();
    clipboard.clear().unwrap();

    let image = ClipboardImage::from_path_sync("tests/test.png").unwrap();

    // 测试缩略图
    let thumbnail = image.thumbnail_sync(100, 100).unwrap();
    assert_ne!(thumbnail.width(), image.width());
    assert_ne!(thumbnail.height(), image.height());

    // 测试保存
    // 注意：这个测试会创建实际文件，可能需要清理
    // thumbnail.save_to_path_sync("/tmp/test_thumbnail.png").unwrap();

    // 测试编码
    let png_data = image.to_png_sync().unwrap();
    let jpeg_data = image.to_jpeg_sync(80).unwrap();
    assert!(!png_data.is_empty());
    assert!(!jpeg_data.is_empty());
    assert_ne!(png_data.len(), jpeg_data.len());
}