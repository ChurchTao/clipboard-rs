use std::collections::HashMap;

use crate::common::{CallBack, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardWatcher, ContentFormat};
use clipboard_win::types::c_uint;
use clipboard_win::{
    formats, get_clipboard, raw, set_clipboard, Clipboard as ClipboardWin, SysResult,
};
use windows_win::sys::{
    AddClipboardFormatListener, PostMessageW, RemoveClipboardFormatListener, HWND,
    WM_CLIPBOARDUPDATE,
};
use windows_win::{Messages, Window};

static UNKNOW_FORMAT: &str = "unknow format";
static CF_RTF: &str = "Rich Text Format";
static CF_HTML: &str = "HTML Format";
static CF_PNG: &str = "PNG";

pub struct ClipboardContext {
    format_map: HashMap<&'static str, c_uint>,
}

pub struct ClipboardWatcherContext {
    handlers: Vec<CallBack>,
    window: Window,
}

unsafe impl Send for ClipboardContext {}
unsafe impl Sync for ClipboardContext {}
unsafe impl Send for ClipboardWatcherContext {}

pub struct WatcherShutdown {
    window: HWND,
}

impl Drop for WatcherShutdown {
    fn drop(&mut self) {
        unsafe { PostMessageW(self.window, WM_CLIPBOARDUPDATE, 0, -1) };
    }
}

unsafe impl Send for WatcherShutdown {}

pub struct ClipboardListener(HWND);

impl ClipboardListener {
    pub fn new(window: HWND) -> Result<Self> {
        unsafe {
            if AddClipboardFormatListener(window) != 1 {
                Err("AddClipboardFormatListener failed".into())
            } else {
                Ok(ClipboardListener(window))
            }
        }
    }
}

impl Drop for ClipboardListener {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.0);
        }
    }
}

impl ClipboardContext {
    pub fn new() -> Result<ClipboardContext> {
        let window = core::ptr::null_mut();
        let _ = ClipboardWin::new_attempts_for(window, 10).expect("Open clipboard");
        let format_map = {
            let cf_html_uint = clipboard_win::register_format(CF_HTML);
            let cf_rtf_uint = clipboard_win::register_format(CF_RTF);
            let cf_png_uint = clipboard_win::register_format(CF_PNG);
            let mut m: HashMap<&str, c_uint> = HashMap::new();
            if let Some(cf_html) = cf_html_uint {
                m.insert(CF_HTML, cf_html.get());
            }
            if let Some(cf_rtf) = cf_rtf_uint {
                m.insert(CF_RTF, cf_rtf.get());
            }
            if let Some(cf_png) = cf_png_uint {
                m.insert(CF_PNG, cf_png.get());
            }
            m
        };
        Ok(ClipboardContext { format_map })
    }
}

impl ClipboardWatcherContext {
    pub fn new() -> Result<ClipboardWatcherContext> {
        let window = match Window::from_builder(
            windows_win::raw::window::Builder::new()
                .class_name("STATIC")
                .parent_message(),
        ) {
            Ok(window) => window,
            Err(_) => return Err("create window error".into()),
        };
        Ok(Self {
            handlers: Vec::new(),
            window,
        })
    }
}

impl Clipboard for ClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        let _clip = ClipboardWin::new_attempts(10).expect("Open clipboard");
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
        let _clip = ClipboardWin::new_attempts(10).expect("Open clipboard");
        let res = clipboard_win::empty();
        if res.is_err() {
            return Err("clear clipboard error".into());
        }
        Ok(())
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        let format_uint = clipboard_win::register_format(format);
        if format_uint.is_none() {
            return Err("register format error".into());
        }
        let format_uint = format_uint.unwrap().get();
        let buffer = get_clipboard(formats::RawData(format_uint));
        match buffer {
            Ok(data) => Ok(data),
            Err(_) => Err("get buffer error".into()),
        }
    }

    fn get_text(&self) -> Result<String> {
        let string: SysResult<String> = get_clipboard(formats::Unicode);
        match string {
            Ok(s) => Ok(s),
            Err(_) => Ok("".to_string()),
        }
    }

    fn get_rich_text(&self) -> Result<String> {
        let rtf_raw_data = self.get_buffer(CF_RTF);
        match rtf_raw_data {
            Ok(data) => {
                let rtf = String::from_utf8(data);
                match rtf {
                    Ok(s) => Ok(s),
                    Err(_) => Ok("".to_string()),
                }
            }
            Err(_) => Ok("".to_string()),
        }
    }

    fn get_html(&self) -> Result<String> {
        let html_raw_data = self.get_buffer(CF_HTML);
        match html_raw_data {
            Ok(data) => cf_html_to_plain_html(data),
            Err(_) => Ok("".to_string()),
        }
    }

    fn get_image(&self) -> Result<RustImageData> {
        let image_raw_data = self.get_buffer(CF_PNG);
        match image_raw_data {
            Ok(data) => RustImageData::from_bytes(&data),
            Err(_) => Ok(RustImageData::empty()),
        }
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        let format_uint = clipboard_win::register_format(format);
        if format_uint.is_none() {
            return Err("register format error".into());
        }
        let format_uint = format_uint.unwrap().get();
        let res = set_clipboard(formats::RawData(format_uint), buffer);
        if res.is_err() {
            return Err("set buffer error".into());
        }
        Ok(())
    }

    fn set_text(&self, text: String) -> Result<()> {
        let res = set_clipboard(formats::Unicode, text);
        if res.is_err() {
            return Err("set text error".into());
        }
        Ok(())
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        let res = self.set_buffer(CF_RTF, text.as_bytes().to_vec());
        if res.is_err() {
            return Err("set rich text error".into());
        }
        Ok(())
    }

    fn set_html(&self, html: String) -> Result<()> {
        let cf_html = plain_html_to_cf_html(&html);
        let res = self.set_buffer(CF_HTML, cf_html.as_bytes().to_vec());
        if res.is_err() {
            return Err("set html error".into());
        }
        Ok(())
    }

    fn set_image(&self, image: RustImageData) -> Result<()> {
        let png = image.to_png()?;
        let res = self.set_buffer(CF_PNG, png.get_bytes().to_vec());
        if res.is_err() {
            return Err("set image error".into());
        }
        Ok(())
    }

    fn has(&self, format: ContentFormat) -> bool {
        match format {
            ContentFormat::Text => clipboard_win::is_format_avail(formats::CF_UNICODETEXT),
            ContentFormat::Rtf => {
                let cf_rtf_uint = self.format_map.get(CF_RTF).unwrap();
                clipboard_win::is_format_avail(*cf_rtf_uint)
            }
            ContentFormat::Html => {
                let cf_html_uint = self.format_map.get(CF_HTML).unwrap();
                clipboard_win::is_format_avail(*cf_html_uint)
            }
            ContentFormat::Image => {
                // Currently only judge whether there is a png format
                let cf_png_uint = self.format_map.get(CF_PNG).unwrap();
                clipboard_win::is_format_avail(*cf_png_uint)
            }
            ContentFormat::Other(format) => {
                let format_uint = clipboard_win::register_format(format);
                if let Some(format_uint) = format_uint {
                    return clipboard_win::is_format_avail(format_uint.get());
                }
                false
            }
        }
    }
}

impl ClipboardWatcher for ClipboardWatcherContext {
    fn add_handler(&mut self, f: CallBack) -> &mut Self {
        self.handlers.push(f);
        self
    }

    fn start_watch(&mut self) {
        let _guard = ClipboardListener::new(self.window.inner()).unwrap();
        for msg in Messages::new()
            .window(Some(self.window.inner()))
            .low(Some(WM_CLIPBOARDUPDATE))
            .high(Some(WM_CLIPBOARDUPDATE))
        {
            match msg {
                Ok(msg) => match msg.id() {
                    WM_CLIPBOARDUPDATE => {
                        let msg = msg.inner();

                        //Shutdown requested
                        if msg.lParam == -1 {
                            break;
                        }
                        self.handlers.iter().for_each(|handler| {
                            handler();
                        });
                    }
                    _ => panic!("Unexpected message"),
                },
                Err(e) => {
                    println!("msg: error: {:?}", e);
                }
            }
        }
    }

    fn get_shutdown_channel(&self) -> WatcherShutdown {
        WatcherShutdown {
            window: self.window.inner(),
        }
    }
}

// https://learn.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format
// The description header includes the clipboard version number and offsets, indicating where the context and the fragment start and end. The description is a list of ASCII text keywords followed by a string and separated by a colon (:).
// Version: vv version number of the clipboard. Starting version is . As of Windows 10 20H2 this is now .Version:0.9Version:1.0
// StartHTML: Offset (in bytes) from the beginning of the clipboard to the start of the context, or if no context.-1
// EndHTML: Offset (in bytes) from the beginning of the clipboard to the end of the context, or if no context.-1
// StartFragment: Offset (in bytes) from the beginning of the clipboard to the start of the fragment.
// EndFragment: Offset (in bytes) from the beginning of the clipboard to the end of the fragment.
// StartSelection: Optional. Offset (in bytes) from the beginning of the clipboard to the start of the selection.
// EndSelection: Optional. Offset (in bytes) from the beginning of the clipboard to the end of the selection.
// The and keywords are optional and must both be omitted if you do not want the application to generate this information.StartSelectionEndSelection
// Future revisions of the clipboard format may extend the header, for example, since the HTML starts at the offset then multiple and pairs could be added later to support noncontiguous selection of fragments.CF_HTMLStartHTMLStartFragmentEndFragment
// example:
// html=Version:1.0
// StartHTML:000000096
// EndHTML:000000375
// StartFragment:000000096
// EndFragment:000000375
// <html><head><meta http-equiv="content-type" content="text/html; charset=UTF-8"></head><body><div style="background-color:#2b2b2b;color:#a9b7c6;font-family:'JetBrains Mono',monospace;font-size:9.8pt;"><pre><span style="color:#9876aa;">sellChannel</span></pre></div></body></html>
fn cf_html_to_plain_html(cf_html: Vec<u8>) -> Result<String> {
    let cf_html_str = String::from_utf8(cf_html)?;
    let cf_html_bytes = cf_html_str.as_bytes();
    let mut start_fragment_offset_in_bytes = 0;
    let mut end_fragment_offset_in_bytes = 0;
    for line in cf_html_str.lines() {
        match line.split_once(':') {
            Some((k, v)) => match k {
                "StartFragment" => {
                    start_fragment_offset_in_bytes = v.trim_start_matches('0').parse::<usize>()?;
                }
                "EndFragment" => {
                    end_fragment_offset_in_bytes = v.trim_start_matches('0').parse::<usize>()?;
                }
                _ => {}
            },
            None => {
                if start_fragment_offset_in_bytes != 0 && end_fragment_offset_in_bytes != 0 {
                    return Ok(String::from_utf8(
                        cf_html_bytes[start_fragment_offset_in_bytes..end_fragment_offset_in_bytes]
                            .to_vec(),
                    )?);
                }
            }
        }
    }
    // if no StartFragment and EndFragment, return the whole html
    Ok(cf_html_str)
}

// cp from https://github.com/Devolutions/IronRDP/blob/37aa6426dba3272f38a2bb46a513144a326854ee/crates/ironrdp-cliprdr-format/src/html.rs#L91
fn plain_html_to_cf_html(fragment: &str) -> String {
    const POS_PLACEHOLDER: &str = "0000000000";

    let mut buffer = String::new();

    let mut write_header = |key: &str, value: &str| {
        let size = key.len() + value.len() + ":\r\n".len();
        buffer.reserve(size);

        buffer.push_str(key);
        buffer.push(':');
        let value_pos = buffer.len();
        buffer.push_str(value);
        buffer.push_str("\r\n");

        value_pos
    };

    write_header("Version", "0.9");

    let start_html_header_value_pos = write_header("StartHTML", POS_PLACEHOLDER);
    let end_html_header_value_pos = write_header("EndHTML", POS_PLACEHOLDER);
    let start_fragment_header_value_pos = write_header("StartFragment", POS_PLACEHOLDER);
    let end_fragment_header_value_pos = write_header("EndFragment", POS_PLACEHOLDER);

    let start_html_pos = buffer.len();
    buffer.push_str("<html>\r\n<body>\r\n<!--StartFragment-->");

    let start_fragment_pos = buffer.len();
    buffer.push_str(fragment);

    let end_fragment_pos = buffer.len();
    buffer.push_str("<!--EndFragment-->\r\n</body>\r\n</html>");

    let end_html_pos = buffer.len();

    let start_html_pos_value = format!("{:0>10}", start_html_pos);
    let end_html_pos_value = format!("{:0>10}", end_html_pos);
    let start_fragment_pos_value = format!("{:0>10}", start_fragment_pos);
    let end_fragment_pos_value = format!("{:0>10}", end_fragment_pos);

    let mut replace_placeholder = |value_begin_idx: usize, header_value: &str| {
        let value_end_idx = value_begin_idx + POS_PLACEHOLDER.len();
        buffer.replace_range(value_begin_idx..value_end_idx, header_value);
    };

    replace_placeholder(start_html_header_value_pos, &start_html_pos_value);
    replace_placeholder(end_html_header_value_pos, &end_html_pos_value);
    replace_placeholder(start_fragment_header_value_pos, &start_fragment_pos_value);
    replace_placeholder(end_fragment_header_value_pos, &end_fragment_pos_value);

    buffer
}
