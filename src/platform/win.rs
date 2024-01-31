use crate::common::{CallBack, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardWatcher};
use clipboard_win::{formats, get_clipboard, raw, set_clipboard, Clipboard as ClipboardWin};

static UNKNOW_FORMAT: &str = "unknow format";

pub struct ClipboardContext {
    clipboard: ClipboardWin,
}

pub struct ClipboardWatcherContext {}

pub struct WatcherShutdown {}

impl ClipboardContext {
    pub fn new() -> Result<ClipboardContext> {
        let clipboard = ClipboardWin::new_attempts(10).expect("Open clipboard");

        Ok(ClipboardContext { clipboard })
    }
}

impl ClipboardWatcherContext {
    pub fn new() -> Result<ClipboardWatcherContext> {
        Ok(ClipboardWatcherContext {})
    }
}

impl Clipboard for ClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        let format_count = clipboard_win::count_formats();
        if format_count.is_none() {
            return Ok(Vec::new());
        }
        let mut res = Vec::new();
        let enum_formats = clipboard_win::raw::EnumFormats::new();
        enum_formats.into_iter().for_each(|format| {
            let f_name = raw::format_name_big(format);
            match f_name {
                Some(name) => res.push(name),
                None => {
                    res.push(UNKNOW_FORMAT.to_string());
                }
            }
        });
        Ok(res)
    }

    fn clear(&self) -> Result<()> {
        todo!()
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        todo!()
    }

    fn get_text(&self) -> Result<String> {
        todo!()
    }

    fn get_rich_text(&self) -> Result<String> {
        todo!()
    }

    fn get_html(&self) -> Result<String> {
        todo!()
    }

    fn get_image(&self) -> Result<RustImageData> {
        todo!()
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        todo!()
    }

    fn set_text(&self, text: String) -> Result<()> {
        todo!()
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        todo!()
    }

    fn set_html(&self, html: String) -> Result<()> {
        todo!()
    }

    fn set_image(&self, image: Vec<u8>) -> Result<()> {
        todo!()
    }
}

impl ClipboardWatcher for ClipboardWatcherContext {
    fn add_handler(&mut self, f: CallBack) -> &mut Self {
        todo!()
    }

    fn start_watch(&mut self) {
        todo!()
    }

    fn get_shutdown_channel(&mut self) -> WatcherShutdown {
        todo!()
    }
}
