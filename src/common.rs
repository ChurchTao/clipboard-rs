use image::imageops::FilterType;
use image::{self, DynamicImage, GenericImageView, ImageFormat};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

pub struct RustImageData {
    width: u32,
    height: u32,
    format: Option<ImageFormat>,
    data: Option<DynamicImage>,
}

pub trait RustImage: Sized {
    fn empty() -> Self;
    fn is_empty(&self) -> bool;
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

    /// /// Scale this image down to fit within a specific size.
    /// Returns a new image. The image's aspect ratio is preserved.
    /// The image is scaled to the maximum possible size that fits
    /// within the bounds specified by `nwidth` and `nheight`.
    ///
    /// This method uses a fast integer algorithm where each source
    /// pixel contributes to exactly one target pixel.
    /// May give aliasing artifacts if new size is close to old size.
    fn thumbnail(&self, width: u32, height: u32) -> Result<Self>;

    /// en: Adjust the size of the image, do not retain the aspect ratio, return a new image,
    /// will not modify the original image, the default return Png
    /// zh: 调整图片大小，不保留长宽比，返回新的图片，不会修改原图片，默认返回Png
    fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self>;

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
    fn empty() -> Self {
        RustImageData {
            width: 0,
            height: 0,
            format: None,
            data: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        let (width, height) = image.dimensions();
        let format = image::guess_format(image.as_bytes());
        let format = match format {
            Ok(f) => Some(f),
            Err(_) => None,
        };
        Ok(RustImageData {
            width,
            height,
            format,
            data: Some(image),
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
            format,
            data: Some(image),
        })
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_bytes(&self) -> &[u8] {
        match &self.data {
            Some(image) => image.as_bytes(),
            None => &[],
        }
    }

    fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self> {
        match &self.data {
            Some(image) => {
                let resized = image.resize_exact(width, height, filter);
                let mut buf = Cursor::new(Vec::new());
                resized.write_to(&mut buf, image::ImageOutputFormat::Png)?;
                let (width, height) = resized.dimensions();
                Ok(RustImageData {
                    width,
                    height,
                    format: Some(ImageFormat::Png),
                    data: Some(resized),
                })
            }
            None => Err("image is empty".into()),
        }
    }

    /// An Image in JPEG Format with specified quality, up to 100
    fn to_jpeg(&self, quality: u8) -> Result<Self> {
        match &self.data {
            Some(image) => {
                let mut buf = Cursor::new(Vec::new());
                image.write_to(&mut buf, image::ImageOutputFormat::Jpeg(quality))?;
                let data = image::load_from_memory(&buf.into_inner())?;
                let (width, height) = data.dimensions();
                Ok(RustImageData {
                    width,
                    height,
                    data: Some(data),
                    format: Some(ImageFormat::Jpeg),
                })
            }
            None => Err("image is empty".into()),
        }
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        match &self.data {
            Some(image) => {
                image.save(path)?;
                Ok(())
            }
            None => Err("image is empty".into()),
        }
    }

    fn get_format(&self) -> Option<ImageFormat> {
        self.format
    }

    fn to_png(&self) -> Result<Self> {
        match &self.data {
            Some(image) => {
                let mut buf = Cursor::new(Vec::new());
                image.write_to(&mut buf, image::ImageOutputFormat::Png)?;
                let data = image::load_from_memory(&buf.into_inner())?;
                let (width, height) = data.dimensions();
                Ok(RustImageData {
                    width,
                    height,
                    data: Some(data),
                    format: Some(ImageFormat::Png),
                })
            }
            None => Err("image is empty".into()),
        }
    }

    fn thumbnail(&self, width: u32, height: u32) -> Result<Self> {
        match &self.data {
            Some(image) => {
                let resized = image.thumbnail(width, height);
                let mut buf = Cursor::new(Vec::new());
                resized.write_to(&mut buf, image::ImageOutputFormat::Png)?;
                let (width, height) = resized.dimensions();
                Ok(RustImageData {
                    width,
                    height,
                    format: Some(ImageFormat::Png),
                    data: Some(resized),
                })
            }
            None => Err("image is empty".into()),
        }
    }
}
