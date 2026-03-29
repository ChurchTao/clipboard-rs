use crate::{
    common::Result,
    ClipboardContent, ClipboardHandler, ContentFormat,
};

#[cfg(feature = "image")]
use crate::{common::RustImage, RustImageData};

use crate::Clipboard;
use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use wl_clipboard_rs::{
    copy::{self, MimeSource, MimeType, Options, Source},
    paste::{self, get_contents, Seat},
    utils::is_primary_selection_supported,
};

const MIME_TEXT: &str = "text/plain;charset=utf-8";
const MIME_HTML: &str = "text/html";
const MIME_RTF: &str = "text/rtf";
const MIME_PNG: &str = "image/png";
const MIME_URI_LIST: &str = "text/uri-list";

fn read_wayland_clipboard(mime: paste::MimeType) -> Result<Vec<u8>> {
    let result = get_contents(
        paste::ClipboardType::Regular,
        Seat::Unspecified,
        mime,
    );
    match result {
        Ok((mut pipe, _)) => {
            let mut buffer = vec![];
            pipe.read_to_end(&mut buffer)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            Ok(buffer)
        }
        Err(paste::Error::ClipboardEmpty) | Err(paste::Error::NoMimeType) => {
            Err("Clipboard is empty or content type not available".into())
        }
        Err(e) => Err(e.to_string().into()),
    }
}

fn write_wayland_clipboard(sources: Vec<MimeSource>) -> Result<()> {
    let mut opts = Options::new();
    opts.foreground(false);
    opts.clipboard(copy::ClipboardType::Regular);
    opts.copy_multi(sources)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
    Ok(())
}

pub struct ClipboardContext;

impl ClipboardContext {
    pub fn new() -> Result<Self> {
        match is_primary_selection_supported() {
            Ok(_) => Ok(Self),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

impl Clipboard for ClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        // wl-clipboard-rs doesn't provide a way to list formats directly
        // Try known formats and report which ones are available
        let mut formats = Vec::new();
        if self.has(ContentFormat::Text) {
            formats.push("text/plain".to_string());
        }
        if self.has(ContentFormat::Html) {
            formats.push(MIME_HTML.to_string());
        }
        if self.has(ContentFormat::Rtf) {
            formats.push(MIME_RTF.to_string());
        }
        if self.has(ContentFormat::Image) {
            formats.push(MIME_PNG.to_string());
        }
        if self.has(ContentFormat::Files) {
            formats.push(MIME_URI_LIST.to_string());
        }
        Ok(formats)
    }

    fn has(&self, format: ContentFormat) -> bool {
        let mime = match format {
            ContentFormat::Text => paste::MimeType::Text,
            ContentFormat::Html => paste::MimeType::Specific(MIME_HTML),
            ContentFormat::Rtf => paste::MimeType::Specific(MIME_RTF),
            ContentFormat::Image => paste::MimeType::Specific(MIME_PNG),
            ContentFormat::Files => paste::MimeType::Specific(MIME_URI_LIST),
            ContentFormat::Other(ref s) => paste::MimeType::Specific(s.as_str()),
        };
        read_wayland_clipboard(mime).is_ok()
    }

    fn clear(&self) -> Result<()> {
        copy::clear(copy::ClipboardType::Regular, copy::Seat::All)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
        Ok(())
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        read_wayland_clipboard(paste::MimeType::Specific(format))
    }

    fn get_text(&self) -> Result<String> {
        let bytes = read_wayland_clipboard(paste::MimeType::Text)?;
        String::from_utf8(bytes)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
    }

    fn get_rich_text(&self) -> Result<String> {
        let bytes = read_wayland_clipboard(paste::MimeType::Specific(MIME_RTF))?;
        String::from_utf8(bytes)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
    }

    fn get_html(&self) -> Result<String> {
        let bytes = read_wayland_clipboard(paste::MimeType::Specific(MIME_HTML))?;
        String::from_utf8(bytes)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
    }

    #[cfg(feature = "image")]
    fn get_image(&self) -> Result<RustImageData> {
        let bytes = read_wayland_clipboard(paste::MimeType::Specific(MIME_PNG))?;
        RustImageData::from_bytes(&bytes)
    }

    fn get_files(&self) -> Result<Vec<String>> {
        let bytes = read_wayland_clipboard(paste::MimeType::Specific(MIME_URI_LIST))?;
        let text = String::from_utf8(bytes)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
        Ok(text
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| line.to_string())
            .collect())
    }

    fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
        let mut contents = Vec::new();
        for format in formats {
            match format {
                ContentFormat::Text => {
                    if let Ok(text) = self.get_text() {
                        contents.push(ClipboardContent::Text(text));
                    }
                }
                ContentFormat::Html => {
                    if let Ok(html) = self.get_html() {
                        contents.push(ClipboardContent::Html(html));
                    }
                }
                ContentFormat::Rtf => {
                    if let Ok(rtf) = self.get_rich_text() {
                        contents.push(ClipboardContent::Rtf(rtf));
                    }
                }
                #[cfg(feature = "image")]
                ContentFormat::Image => {
                    if let Ok(image) = self.get_image() {
                        contents.push(ClipboardContent::Image(image));
                    }
                }
                ContentFormat::Files => {
                    if let Ok(files) = self.get_files() {
                        contents.push(ClipboardContent::Files(files));
                    }
                }
                ContentFormat::Other(mime) => {
                    if let Ok(data) = self.get_buffer(mime) {
                        contents.push(ClipboardContent::Other(mime.clone(), data));
                    }
                }
                #[cfg(not(feature = "image"))]
                _ => {}
            }
        }
        Ok(contents)
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        write_wayland_clipboard(vec![MimeSource {
            source: Source::Bytes(buffer.into_boxed_slice()),
            mime_type: MimeType::Specific(format.to_string()),
        }])
    }

    fn set_text(&self, text: String) -> Result<()> {
        write_wayland_clipboard(vec![MimeSource {
            source: Source::Bytes(text.into_bytes().into_boxed_slice()),
            mime_type: MimeType::Text,
        }])
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        write_wayland_clipboard(vec![MimeSource {
            source: Source::Bytes(text.into_bytes().into_boxed_slice()),
            mime_type: MimeType::Specific(MIME_RTF.to_string()),
        }])
    }

    fn set_html(&self, html: String) -> Result<()> {
        write_wayland_clipboard(vec![
            MimeSource {
                source: Source::Bytes(html.into_bytes().into_boxed_slice()),
                mime_type: MimeType::Specific(MIME_HTML.to_string()),
            },
        ])
    }

    #[cfg(feature = "image")]
    fn set_image(&self, image: RustImageData) -> Result<()> {
        let bytes = image.to_png()?;
        write_wayland_clipboard(vec![MimeSource {
            source: Source::Bytes(bytes.get_bytes().into()),
            mime_type: MimeType::Specific(MIME_PNG.to_string()),
        }])
    }

    fn set_files(&self, files: Vec<String>) -> Result<()> {
        let uri_list = files.join("\n");
        write_wayland_clipboard(vec![MimeSource {
            source: Source::Bytes(uri_list.into_bytes().into_boxed_slice()),
            mime_type: MimeType::Specific(MIME_URI_LIST.to_string()),
        }])
    }

    fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
        let mut sources = Vec::new();
        for content in contents {
            match content {
                ClipboardContent::Text(text) => {
                    sources.push(MimeSource {
                        source: Source::Bytes(text.into_bytes().into_boxed_slice()),
                        mime_type: MimeType::Text,
                    });
                }
                ClipboardContent::Html(html) => {
                    sources.push(MimeSource {
                        source: Source::Bytes(html.into_bytes().into_boxed_slice()),
                        mime_type: MimeType::Specific(MIME_HTML.to_string()),
                    });
                }
                ClipboardContent::Rtf(rtf) => {
                    sources.push(MimeSource {
                        source: Source::Bytes(rtf.into_bytes().into_boxed_slice()),
                        mime_type: MimeType::Specific(MIME_RTF.to_string()),
                    });
                }
                #[cfg(feature = "image")]
                ClipboardContent::Image(image) => {
                    let bytes = image.to_png()?;
                    sources.push(MimeSource {
                        source: Source::Bytes(bytes.get_bytes().into()),
                        mime_type: MimeType::Specific(MIME_PNG.to_string()),
                    });
                }
                ClipboardContent::Files(files) => {
                    let uri_list = files.join("\n");
                    sources.push(MimeSource {
                        source: Source::Bytes(uri_list.into_bytes().into_boxed_slice()),
                        mime_type: MimeType::Specific(MIME_URI_LIST.to_string()),
                    });
                }
                ClipboardContent::Other(mime, data) => {
                    sources.push(MimeSource {
                        source: Source::Bytes(data.into_boxed_slice()),
                        mime_type: MimeType::Specific(mime),
                    });
                }
                #[cfg(not(feature = "image"))]
                _ => {}
            }
        }
        write_wayland_clipboard(sources)
    }
}

unsafe impl Send for ClipboardContext {}

// Polling-based clipboard watcher for Wayland
pub struct ClipboardWatcherContext<T: ClipboardHandler> {
    pub(crate) handlers: Vec<T>,
    pub(crate) stop_signal: Sender<()>,
    stop_receiver: Receiver<()>,
}

unsafe impl<T: ClipboardHandler> Send for ClipboardWatcherContext<T> {}

impl<T: ClipboardHandler> ClipboardWatcherContext<T> {
    pub fn new() -> Result<Self> {
        // Verify data-control protocol is available (same check as ClipboardContext)
        is_primary_selection_supported()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            handlers: Vec::new(),
            stop_signal: tx,
            stop_receiver: rx,
        })
    }
}

impl<T: ClipboardHandler> ClipboardWatcherContext<T> {
    pub(crate) fn start_watch_inner(&mut self) {
        let mut last_text = String::new();

        // Get initial clipboard content
        if let Ok(text) = read_wayland_clipboard(paste::MimeType::Text) {
            if let Ok(s) = String::from_utf8(text) {
                last_text = s;
            }
        }

        loop {
            if self
                .stop_receiver
                .recv_timeout(Duration::from_millis(500))
                .is_ok()
            {
                break;
            }

            let changed = if let Ok(bytes) = read_wayland_clipboard(paste::MimeType::Text) {
                if let Ok(current) = String::from_utf8(bytes) {
                    if current != last_text {
                        last_text = current;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                // Clipboard might be empty or have non-text content
                if !last_text.is_empty() {
                    last_text.clear();
                    true
                } else {
                    false
                }
            };

            if changed {
                self.handlers
                    .iter_mut()
                    .for_each(|handler| handler.on_clipboard_change());
            }
        }
    }

}

// get_shutdown_channel is handled by the linux_clipboard wrapper in mod.rs

pub struct WatcherShutdown {
    pub(crate) sender: Sender<()>,
}

impl Drop for WatcherShutdown {
    fn drop(&mut self) {
        let _ = self.sender.send(());
    }
}
