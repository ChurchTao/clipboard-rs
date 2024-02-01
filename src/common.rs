use image::imageops::FilterType;
use image::{self, GenericImageView, ImageFormat};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

pub struct RustImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
    format: Option<ImageFormat>,
}

impl RustImageData {
    pub fn empty() -> Self {
        RustImageData {
            width: 0,
            height: 0,
            data: Vec::new(),
            format: None,
        }
    }
}

pub trait RustImage: Sized {
    /// en: Read image from file path
    /// zh: 从文件路径读取图片
    fn from_file_path(path: &str) -> Result<Self>;

    /// en: Read image from bytes
    /// zh: 从字节数组读取图片
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    /// en: Get image width and height
    /// zh: 获得图片宽高
    fn get_size(&self) -> (u32, u32);

    fn get_bytes(&self) -> &[u8];

    /// en: Resize image, return new image, will not modify the original image, default return Png
    /// zh: 调整图片大小，返回新的图片，不会修改原图片，默认返回Png
    fn resize(&mut self, width: u32, height: u32, filter: FilterType) -> Result<Self>;

    fn get_format(&self) -> Option<ImageFormat>;

    /// en: Convert image to jpeg format, quality is the quality, give a value of 0-100, 100 is the highest quality,
    /// the returned image is a new image, and the data itself will not be modified
    /// zh: 把图片转为 jpeg 格式，quality 为质量，给0-100的值，100为最高质量,返回的为新的图片，本身数据不会修改
    fn to_jpeg(&self, quality: u8) -> Result<Self>;

    /// en: Convert to png format, the returned image is a new image, and the data itself will not be modified
    /// zh: 转为 png 格式,返回的为新的图片，本身数据不会修改
    fn to_png(&self) -> Result<Self>;

    fn save_to_file(&self, path: &str) -> Result<()>;
}

impl RustImage for RustImageData {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        let (width, height) = image.dimensions();
        let data = bytes.as_ref().to_vec();
        let format = image::guess_format(&data);
        let format = match format {
            Ok(f) => Some(f),
            Err(_) => None,
        };
        Ok(RustImageData {
            width,
            height,
            data,
            format,
        })
    }

    fn from_file_path(path: &str) -> Result<Self> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        let format = image::guess_format(image.as_bytes());
        let format = match format {
            Ok(f) => Some(f),
            Err(_) => None,
        };
        Ok(RustImageData {
            width,
            height,
            data: image.into_bytes(),
            format,
        })
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize(&mut self, width: u32, height: u32, filter: FilterType) -> Result<Self> {
        let image = image::load_from_memory(&self.data)?;
        let resized = image.resize_exact(width, height, filter);

        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageOutputFormat::Png)?;
        let data: Vec<u8> = buf.into_inner();
        let (width, height) = resized.dimensions();
        Ok(RustImageData {
            width,
            height,
            data,
            format: Some(ImageFormat::Png),
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.data
    }

    /// An Image in JPEG Format with specified quality, up to 100
    fn to_jpeg(&self, quality: u8) -> Result<Self> {
        let image = image::load_from_memory(&self.data)?;
        let mut buf = Cursor::new(Vec::new());
        image.write_to(&mut buf, image::ImageOutputFormat::Jpeg(quality))?;
        let data: Vec<u8> = buf.into_inner();
        let (width, height) = image.dimensions();
        Ok(RustImageData {
            width,
            height,
            data,
            format: Some(ImageFormat::Jpeg),
        })
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        if let Some(format) = self.format {
            let image = image::load_from_memory(&self.data)?;
            image.save_with_format(path, format)?;
            return Ok(());
        } else {
            Err("image format unknow".into())
        }
    }

    fn get_format(&self) -> Option<ImageFormat> {
        self.format
    }

    fn to_png(&self) -> Result<Self> {
        let image = image::load_from_memory(&self.data)?;
        let mut buf = Cursor::new(Vec::new());
        image.write_to(&mut buf, image::ImageOutputFormat::Png)?;
        let data: Vec<u8> = buf.into_inner();
        let (width, height) = image.dimensions();
        Ok(RustImageData {
            width,
            height,
            data,
            format: Some(ImageFormat::Png),
        })
    }
}
