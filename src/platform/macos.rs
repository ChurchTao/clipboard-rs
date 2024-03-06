use crate::common::{CallBack, ContentData, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardContent, ClipboardWatcher, ContentFormat};
use cocoa::appkit::{
    NSFilenamesPboardType, NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypePNG,
    NSPasteboardTypeRTF, NSPasteboardTypeString,
};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSData, NSFastEnumeration, NSString};
use std::ffi::{c_void, CStr};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::{slice, vec};

const NS_FILES: &str = "public.file-url";

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

    /// Read from clipboard return trait by NSPasteboardItem
    fn read_from_clipboard(&self) -> Result<Vec<id>> {
        let res = unsafe {
            let ns_array: id = self.clipboard.pasteboardItems();
            if ns_array.count() == 0 {
                return Ok(Vec::new());
            }
            ns_array.iter().collect::<Vec<id>>()
        };
        Ok(res)
    }

    // learn from https://github.com/zed-industries/zed/blob/79c1003b344ee513cf97ee8313c38c7c3f02c916/crates/gpui/src/platform/mac/platform.rs#L793
    fn write_to_clipboard(&self, data: &[WriteToClipboardData], with_clear: bool) -> Result<()> {
        if with_clear {
            unsafe {
                self.clipboard.clearContents();
            }
        }
        data.iter().for_each(|d| unsafe {
            let ns_type = match d.format.clone() {
                ContentFormat::Text => NSPasteboardTypeString,
                ContentFormat::Rtf => NSPasteboardTypeRTF,
                ContentFormat::Html => NSPasteboardTypeHTML,
                ContentFormat::Image => NSPasteboardTypePNG,
                ContentFormat::Files => NSFilenamesPboardType,
                ContentFormat::Other(other_format) => {
                    NSString::alloc(nil).init_str(other_format.as_str())
                }
            };
            if let ContentFormat::Other(_) | ContentFormat::Files = d.format {
                self.clipboard
                    .declareTypes_owner(NSArray::arrayWithObject(nil, ns_type), nil);
            }
            if d.is_multi {
                self.clipboard.setPropertyList_forType(
                    NSArray::arrayByAddingObjectsFromArray(nil, d.data),
                    ns_type,
                );
            } else {
                let ns_data = d.data;
                self.clipboard.setData_forType(ns_data, ns_type);
            }
        });
        Ok(())
    }
}

unsafe impl Send for ClipboardContext {}
unsafe impl Sync for ClipboardContext {}

struct WriteToClipboardData {
    data: id,
    format: ContentFormat,
    is_multi: bool,
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
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        text.as_ptr() as *const c_void,
                        text.len() as u64,
                    )
                },
                is_multi: false,
                format: ContentFormat::Text,
            }],
            true,
        )
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        text.as_ptr() as *const c_void,
                        text.len() as u64,
                    )
                },
                is_multi: false,
                format: ContentFormat::Rtf,
            }],
            true,
        )
    }

    fn set_html(&self, html: String) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        html.as_ptr() as *const c_void,
                        html.len() as u64,
                    )
                },
                is_multi: false,
                format: ContentFormat::Html,
            }],
            true,
        )
    }

    fn set_image(&self, image: RustImageData) -> Result<()> {
        let png = image.to_png()?;
        let res = self.write_to_clipboard(
            &[WriteToClipboardData {
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        png.get_bytes().as_ptr() as *const c_void,
                        png.get_bytes().len() as u64,
                    )
                },
                is_multi: false,
                format: ContentFormat::Image,
            }],
            true,
        );
        res
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        self.write_to_clipboard(
            &[WriteToClipboardData {
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        buffer.as_ptr() as *const c_void,
                        buffer.len() as u64,
                    )
                },
                is_multi: false,
                format: ContentFormat::Other(format.to_owned()),
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
                self.clipboard.availableTypeFromArray(types) != nil
            },
            ContentFormat::Files => unsafe {
                // Currently only judge whether there is a public.file-url format
                let types =
                    NSArray::arrayWithObjects(nil, &[NSString::alloc(nil).init_str(NS_FILES)]);
                self.clipboard.availableTypeFromArray(types) != nil
            },
            ContentFormat::Other(format) => unsafe {
                let types = NSArray::arrayWithObjects(
                    nil,
                    &[NSString::alloc(nil).init_str(format.as_str())],
                );
                self.clipboard.availableTypeFromArray(types) != nil
            },
        }
    }

    fn get_files(&self) -> Result<Vec<String>> {
        let res = unsafe {
            let ns_array: id = self.clipboard.pasteboardItems();
            if ns_array.count() == 0 {
                return Ok(vec![]);
            }
            ns_array
                .iter()
                .map(|ns_pastboard_item| {
                    let ns_string: id =
                        ns_pastboard_item.stringForType(NSString::alloc(nil).init_str(NS_FILES));
                    let bytes = ns_string.UTF8String();
                    let c_str = CStr::from_ptr(bytes);
                    let str_slice = c_str.to_str()?;
                    Ok(str_slice.to_owned())
                })
                .collect::<Result<Vec<String>>>()?
        };
        Ok(res)
    }

    fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
        let ns_pastboard_item_arr = self.read_from_clipboard()?;
        let mut res: Vec<ClipboardContent> = vec![];
        if ns_pastboard_item_arr.is_empty() {
            return Ok(res);
        }
        for format in formats {
            let content = convert_to_clipboard_content(&ns_pastboard_item_arr, format);
            res.push(content);
        }
        Ok(res)
    }

    fn set_files(&self, file: Vec<String>) -> Result<()> {
        unsafe {
            let ns_string_arr = file
                .iter()
                .map(|f| NSString::alloc(nil).init_str(f))
                .collect::<Vec<id>>();
            self.clipboard
                .declareTypes_owner(NSArray::arrayWithObject(nil, NSFilenamesPboardType), nil);
            self.clipboard.setPropertyList_forType(
                NSArray::arrayWithObjects(nil, ns_string_arr.as_ref()),
                NSFilenamesPboardType,
            );
        }
        Ok(())
    }

    fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
        let mut write_data_vec = vec![];
        for content in contents {
            let write_data = content.to_write_data()?;
            write_data_vec.push(write_data);
        }
        self.write_to_clipboard(&write_data_vec, true)
    }
}

impl ClipboardContent {
    fn to_write_data(&self) -> Result<WriteToClipboardData> {
        let write_data = match self {
            ClipboardContent::Files(file_list) => {
                let ns_string_arr = file_list
                    .iter()
                    .map(|f| unsafe { NSString::alloc(nil).init_str(f) })
                    .collect::<Vec<id>>();
                let ns_array = unsafe { NSArray::arrayWithObjects(nil, ns_string_arr.as_ref()) };
                WriteToClipboardData {
                    data: ns_array,
                    is_multi: true,
                    format: ContentFormat::Files,
                }
            }
            _ => WriteToClipboardData {
                data: unsafe {
                    NSData::dataWithBytes_length_(
                        nil,
                        self.as_bytes().as_ptr() as *const c_void,
                        self.as_bytes().len() as u64,
                    )
                },
                is_multi: false,
                format: self.get_format(),
            },
        };
        Ok(write_data)
    }
}

fn convert_to_clipboard_content(
    ns_pastboard_item_arr: &Vec<id>,
    format: &ContentFormat,
) -> ClipboardContent {
    unsafe {
        let ns_type = {
            match format {
                ContentFormat::Text => NSPasteboardTypeString,
                ContentFormat::Rtf => NSPasteboardTypeRTF,
                ContentFormat::Html => NSPasteboardTypeHTML,
                ContentFormat::Image => NSPasteboardTypePNG,
                ContentFormat::Files => NSString::alloc(nil).init_str(NS_FILES),
                ContentFormat::Other(other_format) => {
                    NSString::alloc(nil).init_str(other_format.as_str())
                }
            }
        };
        let content: ClipboardContent = match format {
            ContentFormat::Text | ContentFormat::Rtf | ContentFormat::Html => {
                let mut string_vec = Vec::new();
                for ns_pastboard_item in ns_pastboard_item_arr {
                    let ns_string: id = ns_pastboard_item.stringForType(ns_type);
                    if ns_string.len() == 0 {
                        continue;
                    }
                    let bytes = ns_string.UTF8String();
                    let c_str = CStr::from_ptr(bytes);
                    let str_slice = c_str.to_str().unwrap();
                    string_vec.push(str_slice);
                }
                match format {
                    ContentFormat::Text => ClipboardContent::Text(string_vec.join("\n")),
                    ContentFormat::Rtf => ClipboardContent::Rtf(string_vec.join("\n")),
                    ContentFormat::Html => ClipboardContent::Html(string_vec.join("\n")),
                    _ => panic!("unexpected format"),
                }
            }
            ContentFormat::Image => match ns_pastboard_item_arr.first() {
                Some(ns_pastboard_item) => {
                    let ns_data = ns_pastboard_item.dataForType(ns_type);
                    if ns_data.length() == 0 {
                        return ClipboardContent::Image(RustImageData::empty());
                    }
                    let length: usize = ns_data.length() as usize;
                    let bytes = slice::from_raw_parts(ns_data.bytes() as *const u8, length);
                    let image = RustImageData::from_bytes(bytes).unwrap();
                    ClipboardContent::Image(image)
                }
                None => ClipboardContent::Image(RustImageData::empty()),
            },
            ContentFormat::Files => {
                let mut string_vec = Vec::new();
                for ns_pastboard_item in ns_pastboard_item_arr {
                    let ns_string: id = ns_pastboard_item.stringForType(ns_type);
                    if ns_string.len() == 0 {
                        continue;
                    }
                    let bytes = ns_string.UTF8String();
                    let c_str = CStr::from_ptr(bytes);
                    let str_slice = c_str.to_str().unwrap();
                    string_vec.push(str_slice.to_owned());
                }
                ClipboardContent::Files(string_vec)
            }
            ContentFormat::Other(format) => match ns_pastboard_item_arr.first() {
                Some(ns_pastboard_item) => {
                    let ns_data = ns_pastboard_item.dataForType(ns_type);
                    if ns_data.length() == 0 {
                        return ClipboardContent::Other(format.clone(), Vec::new());
                    }
                    let length: usize = ns_data.length() as usize;
                    let bytes = slice::from_raw_parts(ns_data.bytes() as *const u8, length);
                    ClipboardContent::Other(format.to_string(), bytes.to_vec())
                }
                None => ClipboardContent::Other(format.clone(), Vec::new()),
            },
        };
        content
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
