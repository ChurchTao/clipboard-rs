use image::imageops::FilterType;
use image::{self, DynamicImage, GenericImageView};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

pub trait ContentData {
    fn get_format(&self) -> &ContentFormat;

    fn as_bytes(&self) -> &[u8];

    fn as_array(&self) -> &[ClipboardContent];

    fn as_str(&self) -> Result<&str>;

    fn as_image(&self) -> Result<RustImageData>;
}

pub struct ClipboardContent {
    format: ContentFormat,
    data: Option<Vec<u8>>,
    // maybe there is multiple data like files
    multi_data: Option<Vec<ClipboardContent>>,
}

impl ClipboardContent {
    pub fn new(format: ContentFormat) -> Self {
        ClipboardContent {
            format,
            data: None,
            multi_data: None,
        }
    }

    pub fn new_with_data(format: ContentFormat, data: Vec<u8>) -> Self {
        ClipboardContent {
            format,
            data: Some(data),
            multi_data: None,
        }
    }

    pub fn new_with_multi_data(format: ContentFormat, data: Vec<ClipboardContent>) -> Self {
        ClipboardContent {
            format,
            data: None,
            multi_data: Some(data),
        }
    }

    pub fn is_multi(&self) -> bool {
        self.multi_data.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn put_data(&mut self, data: Vec<u8>) {
        self.data = Some(data);
    }

    pub fn put_multi_data(&mut self, data: ClipboardContent) {
        match &mut self.multi_data {
            Some(multi) => multi.push(data),
            None => {
                let multi = vec![data];
                self.multi_data = Some(multi);
            }
        }
    }
}

impl ContentData for ClipboardContent {
    fn get_format(&self) -> &ContentFormat {
        &self.format
    }

    fn as_bytes(&self) -> &[u8] {
        match &self.data {
            Some(data) => data.as_slice(),
            None => &[],
        }
    }

    fn as_str(&self) -> Result<&str> {
        if let Some(data) = &self.data {
            return match self.format {
                ContentFormat::Image => Err("can't convert image to string".into()),
                ContentFormat::Other(_) => std::str::from_utf8(data).map_err(|e| e.into()),
                _ => std::str::from_utf8(data).map_err(|e| e.into()),
            };
        }
        Err("content is empty".into())
    }

    fn as_image(&self) -> Result<RustImageData> {
        if let ContentFormat::Image = self.format {
            if let Some(data) = &self.data {
                return RustImageData::from_bytes(data);
            }
            Err("image data is empty".into())
        } else {
            Err("content is not image".into())
        }
    }

    fn as_array(&self) -> &[ClipboardContent] {
        match &self.multi_data {
            Some(data) => data.as_slice(),
            None => &[],
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

/// 此处的 RustImageBuffer 已经是带有图片格式的字节流，例如 png,jpeg;
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
