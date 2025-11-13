//! 异步图像相关功能的测试用例
//! 包括异步API的图像操作测试

#[cfg(all(feature = "image", feature = "async"))]
use clipboard_rs::{AsyncClipboardManager, ContentFormat, ClipboardImage};

/// 异步图像测试
#[tokio::test]
#[cfg(all(feature = "image", feature = "async"))]
async fn test_async_image() {
    let clipboard = AsyncClipboardManager::new().await.unwrap();
    clipboard.clear().await.unwrap();

    let image = ClipboardImage::from_path("tests/test.png").await.unwrap();
    let image_bytes = image.to_png().await.unwrap();

    clipboard.set_image(image).await.unwrap();
    assert!(clipboard.has(ContentFormat::Image).await.unwrap());

    let clipboard_img = clipboard.get_image().await.unwrap();
    assert_eq!(
        clipboard_img.to_png().await.unwrap().len(),
        image_bytes.len()
    );
}

/// 异步图像处理测试
#[tokio::test]
#[cfg(all(feature = "image", feature = "async"))]
async fn test_async_image_processing() {
    let clipboard = AsyncClipboardManager::new().await.unwrap();
    clipboard.clear().await.unwrap();

    let image = ClipboardImage::from_path("tests/test.png").await.unwrap();

    // 测试缩略图
    let thumbnail = image.thumbnail(100, 100).await.unwrap();
    assert_ne!(thumbnail.width(), image.width());
    assert_ne!(thumbnail.height(), image.height());

    // 测试编码
    let png_data = image.to_png().await.unwrap();
    let jpeg_data = image.to_jpeg(80).await.unwrap();
    assert!(!png_data.is_empty());
    assert!(!jpeg_data.is_empty());
    assert_ne!(png_data.len(), jpeg_data.len());
}