use image::{self, GenericImageView};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

pub trait Clipboard: Send {
    /// zh: 获得剪切板当前内容的所有格式
    /// en: Get all formats of the current content in the clipboard
    fn available_formats(&self) -> Result<Vec<String>>;

    /// zh: 清空剪切板
    /// en: clear clipboard
    fn clear(&self) -> Result<()>;

    /// zh: 获得指定格式的数据，以字节数组形式返回
    /// en: Get the data in the specified format in the clipboard as a byte array
    fn get_buffer(&self, format: &str) -> Result<Vec<u8>>;

    /// zh: 仅获得无格式纯文本，以字符串形式返回
    /// en: Get plain text content in the clipboard as string
    fn get_text(&self) -> Result<String>;

    /// zh: 获得剪贴板中的富文本内容，以字符串形式返回
    /// en: Get the rich text content in the clipboard as string
    fn get_rich_text(&self) -> Result<String>;

    /// zh: 获得剪贴板中的html内容，以字符串形式返回
    /// en: Get the html format content in the clipboard as string
    fn get_html(&self) -> Result<String>;

    /// zh: 统一获得 png 格式的图片
    /// en: get image in png format
    fn get_image(&self) -> Result<RustImageData>;

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()>;

    fn set_text(&self, text: String) -> Result<()>;

    fn set_rich_text(&self, text: String) -> Result<()>;

    fn set_html(&self, html: String) -> Result<()>;

    fn set_image(&self, image: Vec<u8>) -> Result<()>;

    fn add_listener(&mut self, f: Box<dyn Fn(&Self) + Send + Sync>);

    /// zh: 开始监听剪切板内容变化,这是一个无限循环，你需要在另一个线程中调用
    /// en: Start listening for clipboard content changes, this is an infinite loop, you need to call it in another thread
    fn start_listen_change(&mut self);
}

pub struct RustImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

pub struct ResizeOptions {
    pub width: u32,
    pub height: u32,
    pub quality: u8,
}

pub trait RustImage: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    fn get_size(&self) -> (u32, u32);

    fn resize(&mut self, options: ResizeOptions) -> Result<Self>;

    fn get_bytes(&self) -> &[u8];

    fn to_jpeg(&self, quality: u8) -> Result<Self>;

    fn save_to_file(&self, path: &str) -> Result<()>;
}

impl RustImage for RustImageData {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        let (width, height) = image.dimensions();
        let data = bytes.as_ref().to_vec();
        Ok(RustImageData {
            width,
            height,
            data,
        })
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn resize(&mut self, options: ResizeOptions) -> Result<Self> {
        let image = image::load_from_memory(&self.data)?;
        let resized = image.resize_exact(
            options.width,
            options.height,
            image::imageops::FilterType::Nearest,
        );

        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageOutputFormat::Png)?;
        let data: Vec<u8> = buf.into_inner();
        let (width, height) = resized.dimensions();
        Ok(RustImageData {
            width,
            height,
            data,
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
        })
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        let image = image::load_from_memory(self.get_bytes())?;
        image.save(path)?;
        Ok(())
    }
}
