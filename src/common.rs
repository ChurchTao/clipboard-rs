use image::imageops::FilterType;
use image::{self, DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

pub enum ContentFormat<'a> {
    Text,
    Rtf,
    Html,
    Image,
    Other(&'a str),
}

pub struct RustImageData {
    width: u32,
    height: u32,
    data: Option<DynamicImage>,
}

pub struct RustImageBuffer(Vec<u8>);

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

    fn to_jpeg(&self, quality: u8) -> Result<RustImageBuffer> {
        match &self.data {
            Some(image) => {
                let mut buf = Cursor::new(Vec::new());
                image.write_to(&mut buf, image::ImageOutputFormat::Jpeg(quality))?;
                Ok(RustImageBuffer(buf.into_inner()))
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

    fn to_png(&self) -> Result<RustImageBuffer> {
        match &self.data {
            Some(image) => {
                let mut buf = Cursor::new(Vec::new());
                image.write_to(&mut buf, image::ImageOutputFormat::Png)?;
                Ok(RustImageBuffer(buf.into_inner()))
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

    fn to_bitmap(&self) -> Result<RustImageBuffer> {
        match &self.data {
            Some(image) => {
                let mut buf = Cursor::new(Vec::new());
                image.write_to(&mut buf, image::ImageOutputFormat::Bmp)?;
                Ok(RustImageBuffer(buf.into_inner()))
            }
            None => Err("image is empty".into()),
        }
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
