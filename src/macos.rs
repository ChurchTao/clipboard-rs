extern crate cocoa;
use crate::common::{Clipboard, Result, RustImage, RustImageData};
use std::ffi::{c_void, CStr};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{slice, thread};

use cocoa::appkit::{
    NSPasteboard, NSPasteboardTypeHTML, NSPasteboardTypePNG, NSPasteboardTypeRTF,
    NSPasteboardTypeString,
};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSData, NSFastEnumeration, NSString};

// required for Send trait because *mut runtime::Object; cannot be sent between threads safely
pub struct SafeId(id);
unsafe impl Send for SafeId {}

pub struct MacOSClipboardContext {
    clipboard: Arc<Mutex<SafeId>>,
    listeners: Vec<Box<dyn Fn(&Self) + Send + Sync>>,
    consumer: Option<Receiver<()>>,
    listen_thread: Option<thread::JoinHandle<()>>,
}

impl MacOSClipboardContext {
    pub fn new() -> Result<MacOSClipboardContext> {
        let ns_pastboard = unsafe { NSPasteboard::generalPasteboard(nil) };
        let mut clipboard_ctx = MacOSClipboardContext {
            clipboard: Arc::new(Mutex::new(SafeId(ns_pastboard))),
            listeners: Vec::new(),
            consumer: None,
            listen_thread: None,
        };
        clipboard_ctx.start_listen();
        Ok(clipboard_ctx)
    }

    fn start_listen(&mut self) {
        let shared_clipboard = self.clipboard.clone();
        let (sender, receiver) = mpsc::channel();
        self.consumer = Some(receiver);
        let thread = thread::spawn(move || {
            let clipboard = shared_clipboard.lock().unwrap().0;
            let mut last_change_count: i64 = unsafe { clipboard.changeCount() };
            loop {
                let change_count = unsafe { clipboard.changeCount() };
                if last_change_count == 0 {
                    last_change_count = change_count;
                } else if change_count != last_change_count {
                    sender.send(()).unwrap();
                    last_change_count = change_count;
                }
                thread::sleep(Duration::from_millis(500));
            }
        });
        self.listen_thread = Some(thread);
    }
}

unsafe impl Send for MacOSClipboardContext {}
unsafe impl Sync for MacOSClipboardContext {}

impl Clipboard for MacOSClipboardContext {
    fn available_formats(&self) -> Result<Vec<String>> {
        let res = unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let types = self.clipboard.lock().unwrap().0.types().autorelease();
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
                .lock()
                .unwrap()
                .0
                .stringForType(NSPasteboardTypeString)
                .autorelease();

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
                .lock()
                .unwrap()
                .0
                .stringForType(NSPasteboardTypeRTF)
                .autorelease();

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
                .lock()
                .unwrap()
                .0
                .stringForType(NSPasteboardTypeHTML)
                .autorelease();

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
                .lock()
                .unwrap()
                .0
                .dataForType(NSPasteboardTypePNG)
                .autorelease();
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
                .lock()
                .unwrap()
                .0
                .dataForType(NSString::alloc(nil).init_str(format))
                .autorelease();
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
                .lock()
                .unwrap()
                .0
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
                .lock()
                .unwrap()
                .0
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
                .lock()
                .unwrap()
                .0
                .setString_forType(ns_string, NSPasteboardTypeHTML)
        };
        if !res {
            return Err("set html failed".into());
        }
        Ok(())
    }

    fn set_image(&self, image: Vec<u8>) -> Result<()> {
        let res = unsafe {
            let ns_data = NSData::dataWithBytes_length_(
                nil,
                image.as_ptr() as *const c_void,
                image.len() as u64,
            );
            self.clipboard
                .lock()
                .unwrap()
                .0
                .setData_forType(ns_data, NSPasteboardTypePNG)
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
                .lock()
                .unwrap()
                .0
                .setData_forType(ns_data, NSString::alloc(nil).init_str(format))
        };
        if !res {
            return Err("set buffer failed".into());
        }
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        unsafe { self.clipboard.lock().unwrap().0.clearContents() };
        Ok(())
    }

    fn start_listen_change(&mut self) {
        match self.listen_thread {
            Some(_) => {}
            None => {
                self.start_listen();
            }
        }
        match self.consumer.take() {
            Some(receiver) => loop {
                let _ = receiver.recv();
                self.listeners.iter().for_each(|f| f(&self));
            },
            _ => {}
        }
    }

    fn add_listener(&mut self, f: Box<dyn Fn(&Self) + Send + Sync>) {
        self.listeners.push(f);
    }
}
