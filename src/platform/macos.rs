use crate::common::{CallBack, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardWatcher, ContentFormat};
use cocoa::appkit::{
    NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypePNG, NSPasteboardTypeRTF,
    NSPasteboardTypeString,
};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSData, NSFastEnumeration, NSString};
use std::ffi::{c_void, CStr};
use std::slice;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

// required for Send trait because *mut runtime::Object; cannot be sent between threads safely
pub struct SafeId(id);
unsafe impl Send for SafeId {}
unsafe impl Sync for SafeId {}

pub struct ClipboardContext {
    clipboard: id,
}

pub struct ClipboardWatcherContext {
    clipboard: id,
    handlers: Vec<CallBack>,
    stop_signal: Sender<()>,
    stop_receiver: Receiver<()>,
    running: bool,
}

unsafe impl Send for ClipboardWatcherContext {}
impl ClipboardWatcherContext {
    pub fn new() -> Result<ClipboardWatcherContext> {
        let ns_pastboard = unsafe { NSPasteboard::generalPasteboard(nil) };
        let (tx, rx) = mpsc::channel();
        Ok(ClipboardWatcherContext {
            clipboard: ns_pastboard,
            handlers: Vec::new(),
            stop_signal: tx,
            stop_receiver: rx,
            running: false,
        })
    }
}

impl ClipboardWatcher for ClipboardWatcherContext {
    fn add_handler(&mut self, f: CallBack) -> &mut Self {
        self.handlers.push(f);
        self
    }

    fn start_watch(&mut self) {
        if self.running {
            println!("already start watch!");
            return;
        }
        self.running = true;
        let mut last_change_count: i64 = unsafe { self.clipboard.changeCount() };
        loop {
            // if receive stop signal, break loop
            if self
                .stop_receiver
                .recv_timeout(Duration::from_millis(500))
                .is_ok()
            {
                break;
            }
            let change_count = unsafe { self.clipboard.changeCount() };
            if last_change_count == 0 {
                last_change_count = change_count;
            } else if change_count != last_change_count {
                self.handlers.iter().for_each(|handler| {
                    handler();
                });
                last_change_count = change_count;
            }
        }
        self.running = false;
    }

    fn get_shutdown_channel(&self) -> WatcherShutdown {
        WatcherShutdown {
            stop_signal: self.stop_signal.clone(),
        }
    }
}

impl ClipboardContext {
    pub fn new() -> Result<ClipboardContext> {
        let ns_pastboard = unsafe {
            NSPasteboard::generalPasteboard(nil)
            // let format_ns_array = NSArray::arrayWithObjects(
            //     nil,
            //     vec![
            //         NSPasteboardTypeString,
            //         NSPasteboardTypeRTF,
            //         NSPasteboardTypeHTML,
            //         NSPasteboardTypePNG,
            //     ]
            //     .as_ref(),
            // );
            // np.declareTypes_owner(format_ns_array, nil);
        };
        let clipboard_ctx = ClipboardContext {
            clipboard: ns_pastboard,
        };
        Ok(clipboard_ctx)
    }

    // learn from https://github.com/zed-industries/zed/blob/79c1003b344ee513cf97ee8313c38c7c3f02c916/crates/gpui/src/platform/mac/platform.rs#L793
    fn write_to_clipboard(&self, data: &[WriteToClipboardData], with_clear: bool) -> Result<()> {
        if with_clear {
            unsafe {
                self.clipboard.clearContents();
            }
        }
        data.iter().for_each(|d| unsafe {
            let ns_data = NSData::dataWithBytes_length_(nil, d.data, d.len);
            if let Some(format) = d.format {
                self.clipboard.setData_forType(ns_data, format);
            } else {
                let custom_format = NSString::alloc(nil).init_str(d.custom_format);
                self.clipboard
                    .declareTypes_owner(NSArray::arrayWithObject(nil, custom_format), nil);
                self.clipboard.setData_forType(ns_data, custom_format);
            }
        });
        Ok(())
    }
}

unsafe impl Send for ClipboardContext {}
unsafe impl Sync for ClipboardContext {}

struct WriteToClipboardData<'a> {
    data: *const c_void,
    len: u64,
    format: Option<id>,
    custom_format: &'a str,
}

impl Clipboard for ClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        let res = unsafe {
            // let _pool = NSAutoreleasePool::new(nil);
            // let types = self.clipboard.types().autorelease();
            let types = self.clipboard.types();
            if types.count() == 0 {
                return Ok(Vec::new());
            }
            types
                .iter()
                .map(|t| {
                    let bytes = t.UTF8String();
                    let c_str = CStr::from_ptr(bytes);
                    let str_slice = c_str.to_str()?;
                    Ok(str_slice.to_owned())
                })
                .collect::<Result<Vec<String>>>()?
        };
        Ok(res)
    }

    fn get_text(&self) -> Result<String> {
        let res = unsafe {
            let ns_string: id = self.clipboard.stringForType(NSPasteboardTypeString);
            if ns_string.len() == 0 {
                return Ok("".to_owned());
            }
            let bytes = ns_string.UTF8String();
            let c_str = CStr::from_ptr(bytes);
            let str_slice = c_str.to_str()?;
            str_slice.to_owned()
        };
        Ok(res)
    }

    fn get_rich_text(&self) -> Result<String> {
        let res = unsafe {
            let ns_string: id = self.clipboard.stringForType(NSPasteboardTypeRTF);
            if ns_string.len() == 0 {
                return Ok("".to_owned());
            }
            let bytes = ns_string.UTF8String();
            let c_str = CStr::from_ptr(bytes);
            let str_slice = c_str.to_str()?;
            str_slice.to_owned()
        };
        Ok(res)
    }

    fn get_html(&self) -> Result<String> {
        let res = unsafe {
            let ns_string: id = self.clipboard.stringForType(NSPasteboardTypeHTML);
            if ns_string.len() == 0 {
                return Ok("".to_owned());
            }
            let bytes = ns_string.UTF8String();
            let c_str = CStr::from_ptr(bytes);
            let str_slice = c_str.to_str()?;
            str_slice.to_owned()
        };
        Ok(res)
    }

    fn get_image(&self) -> Result<RustImageData> {
        let res = unsafe {
            let ns_data = self.clipboard.dataForType(NSPasteboardTypePNG);
            if ns_data.length() == 0 {
                return Ok(RustImageData::empty());
            }
            let length: usize = ns_data.length() as usize;
            let bytes = slice::from_raw_parts(ns_data.bytes() as *const u8, length);
            RustImageData::from_bytes(bytes)?
        };
        Ok(res)
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        let res = unsafe {
            let ns_data = self
                .clipboard
                .dataForType(NSString::alloc(nil).init_str(format));
            if ns_data.length() == 0 {
                return Ok(Vec::new());
            }
            let length: usize = ns_data.length() as usize;
            let bytes = slice::from_raw_parts(ns_data.bytes() as *const u8, length).to_vec();
            bytes
        };
        Ok(res)
    }

    fn set_text(&self, text: String) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: text.as_ptr() as *const c_void,
                len: text.len() as u64,
                format: Some(unsafe { NSPasteboardTypeString }),
                custom_format: "",
            }],
            true,
        )
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: text.as_ptr() as *const c_void,
                len: text.len() as u64,
                format: Some(unsafe { NSPasteboardTypeRTF }),
                custom_format: "",
            }],
            true,
        )
    }

    fn set_html(&self, html: String) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: html.as_ptr() as *const c_void,
                len: html.len() as u64,
                format: Some(unsafe { NSPasteboardTypeHTML }),
                custom_format: "",
            }],
            true,
        )
    }

    fn set_image(&self, image: RustImageData) -> Result<()> {
        let png = image.to_png()?;
        let res = self.write_to_clipboard(
            &[WriteToClipboardData {
                data: png.get_bytes().as_ptr() as *const c_void,
                len: png.get_bytes().len() as u64,
                format: Some(unsafe { NSPasteboardTypePNG }),
                custom_format: "",
            }],
            true,
        );
        res
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: buffer.as_ptr() as *const c_void,
                len: buffer.len() as u64,
                format: None,
                custom_format: format,
            }],
            true,
        )
    }

    fn clear(&self) -> Result<()> {
        unsafe { self.clipboard.clearContents() };
        Ok(())
    }

    fn has(&self, format: ContentFormat) -> bool {
        match format {
            ContentFormat::Text => unsafe {
                let types = NSArray::arrayWithObject(nil, NSPasteboardTypeString);
                // https://developer.apple.com/documentation/appkit/nspasteboard/1526078-availabletypefromarray?language=objc
                // The first pasteboard type in types that is available on the pasteboard, or nil if the receiver does not contain any of the types in types.
                // self.clipboard.availableTypeFromArray(types)
                self.clipboard.availableTypeFromArray(types) != nil
            },
            ContentFormat::Rtf => unsafe {
                let types = NSArray::arrayWithObject(nil, NSPasteboardTypeRTF);
                self.clipboard.availableTypeFromArray(types) != nil
            },
            ContentFormat::Html => unsafe {
                // Currently only judge whether there is a public.html format
                let types = NSArray::arrayWithObjects(nil, &[NSPasteboardTypeHTML]);
                self.clipboard.availableTypeFromArray(types) != nil
            },
            ContentFormat::Image => unsafe {
                // Currently only judge whether there is a png format
                let types = NSArray::arrayWithObjects(nil, &[NSPasteboardTypePNG]);
                self.clipboard.availableTypeFromArray(types).is_null()
            },
            ContentFormat::Other(format) => unsafe {
                let types =
                    NSArray::arrayWithObjects(nil, &[NSString::alloc(nil).init_str(format)]);
                self.clipboard.availableTypeFromArray(types).is_null()
            },
        }
    }
}

pub struct WatcherShutdown {
    stop_signal: Sender<()>,
}

impl Drop for WatcherShutdown {
    fn drop(&mut self) {
        let _ = self.stop_signal.send(());
    }
}
