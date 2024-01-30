use image::{self, GenericImageView};
use std::error::Error;
use std::io::Cursor;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
pub type CallBack = Box<dyn Fn() + Send + Sync>;

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
