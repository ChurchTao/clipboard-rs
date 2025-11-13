//! 现代化图像处理示例
//! 需要启用 async-image feature: `cargo run --example image_modern --features async-image`

#[cfg(feature = "async-image")]
use clipboard_rs::{ClipboardImage, AsyncClipboardManager};

#[cfg(feature = "async-image")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// 创建新的异步剪贴板管理器
	let clipboard = AsyncClipboardManager::new().await?;

	// 创建一个简单的图像
	let mut image_buffer = image::RgbImage::new(100, 100);

	// 绘制一个红色的正方形
	for x in 0..50 {
		for y in 0..50 {
			image_buffer.put_pixel(x, y, image::Rgb([255, 0, 0]));
		}
	}

	// 转换为 ClipboardImage
	let clipboard_image = clipboard_rs::ClipboardImage::from_dynamic_image(
		image::DynamicImage::ImageRgb8(image_buffer),
	);

	// 设置图像到剪贴板
	clipboard.set_clipboard_image(clipboard_image).await?;
	println!("Image set to clipboard successfully!");

	// 从剪贴板获取图像
	match clipboard.get_clipboard_image().await {
		Ok(image) => {
			println!("Got image from clipboard!");
			println!("Image dimensions: {:?}", image.dimensions());

			// 保存图像到文件
			image.save_to_path("clipboard_image.png").await?;
			println!("Image saved to clipboard_image.png");
		}
		Err(e) => {
			println!("Failed to get image from clipboard: {}", e);
		}
	}

	Ok(())
}

#[cfg(not(feature = "async-image"))]
fn main() {
	println!("This example requires the 'async-image' feature to be enabled.");
	println!("Run with: cargo run --example image_modern --features async-image");
}
