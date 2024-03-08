use image::imageops::FilterType;
use image::{self, DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

pub trait ContentData {
	fn get_format(&self) -> ContentFormat;

	fn as_bytes(&self) -> &[u8];

	fn as_str(&self) -> Result<&str>;
}

pub enum ClipboardContent {
	Text(String),
	Rtf(String),
	Html(String),
	Image(RustImageData),
	Files(Vec<String>),
	Other(String, Vec<u8>),
}

impl ContentData for ClipboardContent {
	fn get_format(&self) -> ContentFormat {
		match self {
			ClipboardContent::Text(_) => ContentFormat::Text,
			ClipboardContent::Rtf(_) => ContentFormat::Rtf,
			ClipboardContent::Html(_) => ContentFormat::Html,
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
			ClipboardContent::Image(data) => data.as_bytes(),
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
			ClipboardContent::Image(_) => Err("can't convert image to string".into()),
			ClipboardContent::Files(data) => {
				// use first file path as data
				if let Some(path) = data.first() {
					Ok(path)
				} else {
					Err("content is empty".into())
				}
			}
			ClipboardContent::Other(_, data) => std::str::from_utf8(data).map_err(|e| e.into()),
		}
	}
}

#[derive(Clone)]
pub enum ContentFormat {
	Text,
	Rtf,
	Html,
	Image,
	Files,
	Other(String),
}

pub struct RustImageData {
	width: u32,
	height: u32,
	data: Option<DynamicImage>,
}

macro_rules! handle_image_operation {
	($self:expr, $operation:expr) => {
		match &$self.data {
			Some(image) => {
				let mut buf = Cursor::new(Vec::new());
				image.write_to(&mut buf, $operation)?;
				Ok(RustImageBuffer(buf.into_inner()))
			}
			None => Err("image is empty".into()),
		}
	};
}

/// 此处的 RustImageBuffer 已经是带有图片格式的字节流，例如 png,jpeg;
pub struct RustImageBuffer(Vec<u8>);

impl RustImageData {
	pub fn as_bytes(&self) -> &[u8] {
		match &self.data {
			Some(image) => image.as_bytes(),
			None => &[],
		}
	}
}

pub trait RustImage: Sized {
	/// create an empty image
	fn empty() -> Self;

	fn is_empty(&self) -> bool;

	/// Read image from file path
	fn from_path(path: &str) -> Result<Self>;

	/// Create a new image from a byte slice
	fn from_bytes(bytes: &[u8]) -> Result<Self>;

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

	/// en: Convert image to jpeg format, quality is the quality, give a value of 0-100, 100 is the highest quality,
	/// the returned image is a new image, and the data itself will not be modified
	/// zh: 把图片转为 jpeg 格式，quality(0-100) 为质量，输出字节数组，可直接通过 io 写入文件
	fn to_jpeg(&self, quality: u8) -> Result<RustImageBuffer>;

	/// en: Convert to png format, the returned image is a new image, and the data itself will not be modified
	/// zh: 转为 png 格式,返回的为新的图片，本身数据不会修改
	fn to_png(&self) -> Result<RustImageBuffer>;

	fn to_bitmap(&self) -> Result<RustImageBuffer>;

	fn save_to_path(&self, path: &str) -> Result<()>;
}

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

	fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let image = image::load_from_memory(bytes)?;
		let (width, height) = image.dimensions();
		Ok(RustImageData {
			width,
			height,
			data: Some(image),
		})
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

	fn get_size(&self) -> (u32, u32) {
		(self.width, self.height)
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
			None => Err("image is empty".into()),
		}
	}

	fn save_to_path(&self, path: &str) -> Result<()> {
		match &self.data {
			Some(image) => {
				image.save(path)?;
				Ok(())
			}
			None => Err("image is empty".into()),
		}
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
			None => Err("image is empty".into()),
		}
	}

	fn to_jpeg(&self, quality: u8) -> Result<RustImageBuffer> {
		handle_image_operation!(self, image::ImageOutputFormat::Jpeg(quality))
	}

	fn to_png(&self) -> Result<RustImageBuffer> {
		handle_image_operation!(self, image::ImageOutputFormat::Png)
	}

	fn to_bitmap(&self) -> Result<RustImageBuffer> {
		handle_image_operation!(self, image::ImageOutputFormat::Bmp)
	}
}

impl RustImageBuffer {
	pub fn get_bytes(&self) -> &[u8] {
		&self.0
	}

	pub fn save_to_path(&self, path: &str) -> Result<()> {
		std::fs::write(path, &self.0)?;
		Ok(())
	}
}
