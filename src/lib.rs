pub mod common;
mod platform;
pub use common::{CallBack, ContentFormat, Result, RustImageData};
pub use image::imageops::FilterType;
pub use platform::{ClipboardContext, ClipboardWatcherContext, WatcherShutdown};
pub trait Clipboard: Send {
    /// zh: 获得剪切板当前内容的所有格式
    /// en: Get all formats of the current content in the clipboard
    fn available_formats(&self) -> Result<Vec<String>>;

    fn has(&self, format: ContentFormat) -> bool;

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

    fn get_image(&self) -> Result<RustImageData>;

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()>;

    fn set_text(&self, text: String) -> Result<()>;

    fn set_rich_text(&self, text: String) -> Result<()>;

    fn set_html(&self, html: String) -> Result<()>;

    fn set_image(&self, image: RustImageData) -> Result<()>;
}

pub trait ClipboardWatcher: Send {
    fn add_handler(&mut self, f: CallBack) -> &mut Self;

    fn start_watch(&mut self);

    fn get_shutdown_channel(&self) -> WatcherShutdown;
}

impl WatcherShutdown {
    ///Signals shutdown
    pub fn stop(self) {
        drop(self);
    }
}
