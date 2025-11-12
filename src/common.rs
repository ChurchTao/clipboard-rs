#[cfg(feature = "image")]
use image::imageops::FilterType;
#[cfg(feature = "image")]
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipboardError {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[cfg(feature = "image")]
	#[error("Image error: {0}")]
	Image(#[from] image::ImageError),

	#[error("Clipboard is empty")]
	Empty,

	#[error("Unsupported format: {0}")]
	UnsupportedFormat(String),

	#[error("Platform specific error: {0}")]
	PlatformError(String),

	#[error("Invalid data: {0}")]
	InvalidData(String),

	#[error("Clipboard operation timeout")]
	Timeout,

	#[error("Permission denied")]
	PermissionDenied,
}

pub type Result<T> = std::result::Result<T, ClipboardError>;

pub trait ContentData {
	fn get_format(&self) -> ContentFormat;

	fn as_bytes(&self) -> &[u8];

	fn as_str(&self) -> Result<&str>;
}

pub trait ClipboardHandler {
	fn on_clipboard_change(&mut self);
}

pub enum ClipboardContent {
	Text(String),
	Rtf(String),
	Html(String),
	#[cfg(feature = "image")]
	Image(RustImageData),
	Files(Vec<String>),
	Other(String, Vec<u8>),
}

/// 剪贴板内容构建器，提供流畅的 API 来构建复杂的剪贴板内容
pub struct ClipboardContentBuilder {
	contents: Vec<ClipboardContent>,
}

impl ClipboardContentBuilder {
	/// 创建新的剪贴板内容构建器
	pub fn new() -> Self {
		Self {
			contents: Vec::new(),
		}
	}

	/// 添加纯文本内容
	pub fn with_text(mut self, text: impl AsRef<str>) -> Self {
		self.contents
			.push(ClipboardContent::Text(text.as_ref().to_string()));
		self
	}

	/// 添加 HTML 内容
	pub fn with_html(mut self, html: impl AsRef<str>) -> Self {
		self.contents
			.push(ClipboardContent::Html(html.as_ref().to_string()));
		self
	}

	/// 添加 RTF 内容
	pub fn with_rtf(mut self, rtf: impl AsRef<str>) -> Self {
		self.contents
			.push(ClipboardContent::Rtf(rtf.as_ref().to_string()));
		self
	}

	/// 添加图像内容
	#[cfg(feature = "image")]
	pub fn with_image(mut self, image: crate::RustImageData) -> Self {
		self.contents.push(ClipboardContent::Image(image));
		self
	}

	/// 添加文件列表
	pub fn with_files(mut self, files: &[impl AsRef<str>]) -> Self {
		let file_strings: Vec<String> = files.iter().map(|f| f.as_ref().to_string()).collect();
		self.contents.push(ClipboardContent::Files(file_strings));
		self
	}

	/// 添加自定义格式内容
	pub fn with_custom(mut self, format: impl AsRef<str>, data: Vec<u8>) -> Self {
		self.contents
			.push(ClipboardContent::Other(format.as_ref().to_string(), data));
		self
	}

	/// 构建剪贴板内容向量
	pub fn build(self) -> Vec<ClipboardContent> {
		self.contents
	}
}

impl ContentData for ClipboardContent {
	fn get_format(&self) -> ContentFormat {
		match self {
			ClipboardContent::Text(_) => ContentFormat::Text,
			ClipboardContent::Rtf(_) => ContentFormat::Rtf,
			ClipboardContent::Html(_) => ContentFormat::Html,
			#[cfg(feature = "image")]
			ClipboardContent::Image(_) => ContentFormat::Image,
			ClipboardContent::Files(_) => ContentFormat::Files,
			ClipboardContent::Other(format, _) => ContentFormat::Other(format.clone()),
		}
	}

	fn as_bytes(&self) -> &[u8] {
		match self {
			ClipboardContent::Text(data) => data.as_bytes(),
			ClipboardContent::Rtf(data) => data.as_bytes(),
			ClipboardContent::Html(data) => data.as_bytes(),
			// dynamic image is not supported to as bytes
			#[cfg(feature = "image")]
			ClipboardContent::Image(_) => &[],
			ClipboardContent::Files(data) => {
				// use first file path as data
				if let Some(path) = data.first() {
					path.as_bytes()
				} else {
					&[]
				}
			}
			ClipboardContent::Other(_, data) => data.as_slice(),
		}
	}

	fn as_str(&self) -> Result<&str> {
		match self {
			ClipboardContent::Text(data) => Ok(data),
			ClipboardContent::Rtf(data) => Ok(data),
			ClipboardContent::Html(data) => Ok(data),
			#[cfg(feature = "image")]
			ClipboardContent::Image(_) => Err(ClipboardError::InvalidData(
				"can't convert image to string".into(),
			)),
			ClipboardContent::Files(data) => {
				// use first file path as data
				if let Some(path) = data.first() {
					Ok(path)
				} else {
					Err(ClipboardError::Empty)
				}
			}
			ClipboardContent::Other(_, data) => {
				std::str::from_utf8(data).map_err(|e| ClipboardError::InvalidData(e.to_string()))
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum ContentFormat {
	Text,
	Rtf,
	Html,
	#[cfg(feature = "image")]
	Image,
	Files,
	Other(String),
}

#[cfg(feature = "image")]
pub struct RustImageData {
	width: u32,
	height: u32,
	data: Option<DynamicImage>,
}

/// 现代化的图像数据结构，提供异步支持和更多功能
#[cfg(feature = "async-image")]
pub struct ClipboardImage {
	inner: DynamicImage,
}

#[cfg(feature = "async-image")]
impl ClipboardImage {
	/// 从文件路径创建图像
	pub async fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self> {
		let path = path.as_ref().to_path_buf();
		// 在后台线程中加载图像以避免阻塞
		let image = tokio::task::spawn_blocking(move || {
			image::open(&path).map_err(|e| ClipboardError::Image(e))
		})
		.await
		.map_err(|e| ClipboardError::Io(e.into()))??;

		Ok(ClipboardImage { inner: image })
	}

	/// 从字节数组创建图像
	pub async fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let bytes = bytes.to_vec();
		// 在后台线程中加载图像以避免阻塞
		let image = tokio::task::spawn_blocking(move || {
			image::load_from_memory(&bytes).map_err(|e| ClipboardError::InvalidData(e.to_string()))
		})
		.await
		.map_err(|e| ClipboardError::Io(e.into()))??;

		Ok(ClipboardImage { inner: image })
	}

	/// 从 DynamicImage 创建
	pub fn from_dynamic_image(image: DynamicImage) -> Self {
		ClipboardImage { inner: image }
	}

	/// 获取图像尺寸
	pub fn dimensions(&self) -> (u32, u32) {
		self.inner.dimensions()
	}

	/// 获取图像宽度
	pub fn width(&self) -> u32 {
		self.inner.width()
	}

	/// 获取图像高度
	pub fn height(&self) -> u32 {
		self.inner.height()
	}

	/// 缩略图处理
	pub async fn thumbnail(&self, width: u32, height: u32) -> Result<Self> {
		let image = self.inner.clone();
		let thumbnail = tokio::task::spawn_blocking(move || image.thumbnail(width, height))
			.await
			.map_err(|e| ClipboardError::Io(e.into()))?;

		Ok(ClipboardImage { inner: thumbnail })
	}

	/// 调整图像大小
	pub async fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self> {
		let image = self.inner.clone();
		let resized =
			tokio::task::spawn_blocking(move || image.resize_exact(width, height, filter))
				.await
				.map_err(|e| ClipboardError::Io(e.into()))?;

		Ok(ClipboardImage { inner: resized })
	}

	/// 保存到文件路径
	pub async fn save_to_path(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
		let image = self.inner.clone();
		let path = path.as_ref().to_path_buf();
		tokio::task::spawn_blocking(move || {
			image.save(&path).map_err(|e| ClipboardError::Image(e))
		})
		.await
		.map_err(|e| ClipboardError::Io(e.into()))??;

		Ok(())
	}

	/// 转换为 PNG 格式
	pub async fn to_png(&self) -> Result<Vec<u8>> {
		self.encode(ImageFormat::Png).await
	}

	/// 转换为 JPEG 格式
	pub async fn to_jpeg(&self, quality: u8) -> Result<Vec<u8>> {
		self.encode_with_quality(ImageFormat::Jpeg, quality).await
	}

	/// 转换为 BMP 格式
	pub async fn to_bmp(&self) -> Result<Vec<u8>> {
		self.encode(ImageFormat::Bmp).await
	}

	/// 编码为指定格式
	pub async fn encode(&self, format: ImageFormat) -> Result<Vec<u8>> {
		self.encode_with_quality(format, 90).await
	}

	/// 编码为指定格式并设置质量
	pub async fn encode_with_quality(&self, format: ImageFormat, quality: u8) -> Result<Vec<u8>> {
		let image = self.inner.clone();
		let bytes = tokio::task::spawn_blocking(move || {
			let mut buffer = Vec::new();
			let mut cursor = std::io::Cursor::new(&mut buffer);

			match format {
				ImageFormat::Jpeg => {
					let mut encoder =
						image::codecs::jpeg::JpegEncoder::new_with_quality(cursor, quality);
					encoder
						.encode_image(&image)
						.map_err(|e| ClipboardError::Image(e))?;
				}
				_ => {
					image
						.write_to(&mut cursor, format)
						.map_err(|e| ClipboardError::Image(e))?;
				}
			}

			Ok::<Vec<u8>, ClipboardError>(buffer)
		})
		.await
		.map_err(|e| ClipboardError::Io(e.into()))??;

		Ok(bytes)
	}

	/// 获取 DynamicImage
	pub fn get_dynamic_image(&self) -> &DynamicImage {
		&self.inner
	}

	/// 转换为 RGBA8 格式
	pub fn to_rgba8(&self) -> RgbaImage {
		self.inner.to_rgba8()
	}
}

/// 此处的 `RustImageBuffer` 已经是带有图片格式的字节流，例如 png,jpeg;
#[cfg(feature = "image")]
pub struct RustImageBuffer(Vec<u8>);

#[cfg(feature = "image")]
pub trait RustImage: Sized {
	/// create an empty image
	fn empty() -> Self;

	fn is_empty(&self) -> bool;

	/// Read image from file path
	fn from_path(path: &str) -> Result<Self>;

	/// Create a new image from a byte slice
	fn from_bytes(bytes: &[u8]) -> Result<Self>;

	fn from_dynamic_image(image: DynamicImage) -> Self;

	/// width and height
	fn get_size(&self) -> (u32, u32);

	/// Scale this image down to fit within a specific size.
	/// Returns a new image. The image's aspect ratio is preserved.
	/// The image is scaled to the maximum possible size that fits
	/// within the bounds specified by `nwidth` and `nheight`.
	///
	/// This method uses a fast integer algorithm where each source
	/// pixel contributes to exactly one target pixel.
	/// May give aliasing artifacts if new size is close to old size.
	fn thumbnail(&self, width: u32, height: u32) -> Result<Self>;

	/// en: Adjust the size of the image without retaining the aspect ratio
	/// zh: 调整图片大小，不保留长宽比
	fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self>;

	fn encode_image(
		&self,
		target_color_type: ColorType,
		format: ImageFormat,
	) -> Result<RustImageBuffer>;

	fn to_jpeg(&self) -> Result<RustImageBuffer>;

	/// en: Convert to png format, the returned image is a new image, and the data itself will not be modified
	/// zh: 转为 png 格式,返回的为新的图片，本身数据不会修改
	fn to_png(&self) -> Result<RustImageBuffer>;

	#[cfg(target_os = "windows")]
	fn to_bitmap(&self) -> Result<RustImageBuffer>;

	fn save_to_path(&self, path: &str) -> Result<()>;

	fn get_dynamic_image(&self) -> Result<DynamicImage>;

	fn to_rgba8(&self) -> Result<RgbaImage>;
}

#[cfg(feature = "image")]
impl RustImage for RustImageData {
	fn empty() -> Self {
		RustImageData {
			width: 0,
			height: 0,
			data: None,
		}
	}

	fn is_empty(&self) -> bool {
		self.data.is_none()
	}

	fn from_path(path: &str) -> Result<Self> {
		let image = image::open(path)?;
		let (width, height) = image.dimensions();
		Ok(RustImageData {
			width,
			height,
			data: Some(image),
		})
	}

	fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let image = image::load_from_memory(bytes)?;
		let (width, height) = image.dimensions();
		Ok(RustImageData {
			width,
			height,
			data: Some(image),
		})
	}

	fn from_dynamic_image(image: DynamicImage) -> Self {
		let (width, height) = image.dimensions();
		RustImageData {
			width,
			height,
			data: Some(image),
		}
	}

	fn get_size(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	fn thumbnail(&self, width: u32, height: u32) -> Result<Self> {
		match &self.data {
			Some(image) => {
				let resized = image.thumbnail(width, height);
				Ok(RustImageData {
					width: resized.width(),
					height: resized.height(),
					data: Some(resized),
				})
			}
			None => Err(ClipboardError::Empty),
		}
	}

	fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self> {
		match &self.data {
			Some(image) => {
				let resized = image.resize_exact(width, height, filter);
				Ok(RustImageData {
					width: resized.width(),
					height: resized.height(),
					data: Some(resized),
				})
			}
			None => Err(ClipboardError::Empty),
		}
	}

	fn save_to_path(&self, path: &str) -> Result<()> {
		match &self.data {
			Some(image) => {
				image.save(path)?;
				Ok(())
			}
			None => Err(ClipboardError::Empty),
		}
	}

	fn get_dynamic_image(&self) -> Result<DynamicImage> {
		match &self.data {
			Some(image) => Ok(image.clone()),
			None => Err(ClipboardError::Empty),
		}
	}

	fn to_rgba8(&self) -> Result<RgbaImage> {
		match &self.data {
			Some(image) => Ok(image.to_rgba8()),
			None => Err(ClipboardError::Empty),
		}
	}

	// 私有辅助函数，处理图像格式转换和编码
	fn encode_image(
		&self,
		target_color_type: ColorType,
		format: ImageFormat,
	) -> Result<RustImageBuffer> {
		let image = self.data.as_ref().ok_or(ClipboardError::Empty)?;

		let mut bytes = Vec::new();
		match (image.color(), target_color_type) {
			(ColorType::Rgba8, ColorType::Rgb8) => image
				.to_rgb8()
				.write_to(&mut Cursor::new(&mut bytes), format)?,
			(_, ColorType::Rgba8) => image
				.to_rgba8()
				.write_to(&mut Cursor::new(&mut bytes), format)?,
			_ => image.write_to(&mut Cursor::new(&mut bytes), format)?,
		};
		Ok(RustImageBuffer(bytes))
	}

	fn to_jpeg(&self) -> Result<RustImageBuffer> {
		// JPEG 需要 RGB 格式（不支持 alpha 通道）
		self.encode_image(ColorType::Rgb8, ImageFormat::Jpeg)
	}

	fn to_png(&self) -> Result<RustImageBuffer> {
		// PNG 使用 RGBA 格式以支持透明度
		self.encode_image(ColorType::Rgba8, ImageFormat::Png)
	}

	#[cfg(target_os = "windows")]
	fn to_bitmap(&self) -> Result<RustImageBuffer> {
		// BMP 使用 RGBA 格式
		self.encode_image(ColorType::Rgba8, ImageFormat::Bmp)
	}
}

#[cfg(feature = "image")]
impl RustImageBuffer {
	pub fn get_bytes(&self) -> &[u8] {
		&self.0
	}

	pub fn save_to_path(&self, path: &str) -> Result<()> {
		std::fs::write(path, &self.0)?;
		Ok(())
	}
}
