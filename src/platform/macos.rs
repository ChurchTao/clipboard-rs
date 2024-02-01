use crate::common::{CallBack, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardWatcher};
use cocoa::appkit::{
    NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypePNG, NSPasteboardTypeRTF,
    NSPasteboardTypeString,
};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSData, NSFastEnumeration, NSString};
use image::ImageFormat;
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

    fn get_shutdown_channel(&mut self) -> WatcherShutdown {
        WatcherShutdown {
            stop_signal: self.stop_signal.clone(),
        }
    }
}

impl ClipboardContext {
    pub fn new() -> Result<ClipboardContext> {
        let ns_pastboard = unsafe { NSPasteboard::generalPasteboard(nil) };
        let clipboard_ctx = ClipboardContext {
            clipboard: ns_pastboard,
        };
        Ok(clipboard_ctx)
    }
}

unsafe impl Send for ClipboardContext {}
unsafe impl Sync for ClipboardContext {}

impl Clipboard for ClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        let res = unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let types = self.clipboard.types().autorelease();
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
            let _pool = NSAutoreleasePool::new(nil);
            let ns_string: id = self
                .clipboard
                .stringForType(NSPasteboardTypeString)
                .autorelease();
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
            let _pool = NSAutoreleasePool::new(nil);
            let ns_string: id = self
                .clipboard
                .stringForType(NSPasteboardTypeRTF)
                .autorelease();
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
            let _pool = NSAutoreleasePool::new(nil);
            let ns_string: id = self
                .clipboard
                .stringForType(NSPasteboardTypeHTML)
                .autorelease();
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
            let _pool = NSAutoreleasePool::new(nil);
            let ns_data = self
                .clipboard
                .dataForType(NSPasteboardTypePNG)
                .autorelease();
            if ns_data.length() == 0 {
                return Ok(RustImageData::empty());
            }
            let length: usize = ns_data.length() as usize;
            let bytes = slice::from_raw_parts(ns_data.bytes() as *const u8, length).to_vec();
            RustImageData::from_bytes(&bytes)?
        };
        Ok(res)
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        let res = unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let ns_data = self
                .clipboard
                .dataForType(NSString::alloc(nil).init_str(format))
                .autorelease();
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
        let res = unsafe {
            let ns_string = NSString::alloc(nil).init_str(text.as_str());
            self.clipboard
                .setString_forType(ns_string, NSPasteboardTypeString)
        };
        if !res {
            return Err("set text failed".into());
        }
        Ok(())
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        let res = unsafe {
            let ns_string = NSString::alloc(nil).init_str(text.as_str());
            self.clipboard
                .setString_forType(ns_string, NSPasteboardTypeRTF)
        };
        if !res {
            return Err("set rich text failed".into());
        }
        Ok(())
    }

    fn set_html(&self, html: String) -> Result<()> {
        let res = unsafe {
            let ns_string = NSString::alloc(nil).init_str(html.as_str());
            self.clipboard
                .setString_forType(ns_string, NSPasteboardTypeHTML)
        };
        if !res {
            return Err("set html failed".into());
        }
        Ok(())
    }

    fn set_image(&self, image: RustImageData) -> Result<()> {
        match image.get_format() {
            Some(format) => {
                if format != ImageFormat::Png {
                    return Err("set image only support png format".into());
                }
            }
            None => return Err("image format unknow".into()),
        }
        let res = unsafe {
            let ns_data = NSData::dataWithBytes_length_(
                nil,
                image.get_bytes().as_ptr() as *const c_void,
                image.get_bytes().len() as u64,
            );
            self.clipboard.setData_forType(ns_data, NSPasteboardTypePNG)
        };
        if !res {
            return Err("set image failed".into());
        }
        Ok(())
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        let res = unsafe {
            let ns_data = NSData::dataWithBytes_length_(
                nil,
                buffer.as_ptr() as *const c_void,
                buffer.len() as u64,
            );
            self.clipboard
                .setData_forType(ns_data, NSString::alloc(nil).init_str(format))
        };
        if !res {
            return Err("set buffer failed".into());
        }
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        unsafe { self.clipboard.clearContents() };
        Ok(())
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
