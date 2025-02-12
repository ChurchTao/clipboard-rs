use crate::common::{Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardContent, ClipboardHandler, ClipboardWatcher, ContentFormat};
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2::{rc::autoreleasepool, runtime::ProtocolObject};
use objc2_app_kit::{
	NSFilenamesPboardType, NSImage, NSPasteboard, NSPasteboardItem, NSPasteboardType,
	NSPasteboardTypeHTML, NSPasteboardTypePNG, NSPasteboardTypeRTF, NSPasteboardTypeString,
	NSPasteboardTypeTIFF, NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSString};
use std::ffi::c_void;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::vec;

pub struct ClipboardContext {
	pasteboard: Retained<NSPasteboard>,
}

pub struct ClipboardWatcherContext<T: ClipboardHandler> {
	pasteboard: Retained<NSPasteboard>,
	handlers: Vec<T>,
	stop_signal: Sender<()>,
	stop_receiver: Receiver<()>,
	running: bool,
}

unsafe impl<T: ClipboardHandler> Send for ClipboardWatcherContext<T> {}

impl<T: ClipboardHandler> ClipboardWatcherContext<T> {
	pub fn new() -> Result<Self> {
		let ns_pasteboard = unsafe { NSPasteboard::generalPasteboard() };
		let (tx, rx) = mpsc::channel();
		Ok(ClipboardWatcherContext {
			pasteboard: ns_pasteboard,
			handlers: Vec::new(),
			stop_signal: tx,
			stop_receiver: rx,
			running: false,
		})
	}
}

impl<T: ClipboardHandler> ClipboardWatcher<T> for ClipboardWatcherContext<T> {
	fn add_handler(&mut self, handler: T) -> &mut Self {
		self.handlers.push(handler);
		self
	}

	fn start_watch(&mut self) {
		if self.running {
			println!("already start watch!");
			return;
		}
		if self.handlers.is_empty() {
			println!("no handler, no need to start watch!");
			return;
		}
		self.running = true;
		let mut last_change_count = unsafe { self.pasteboard.changeCount() };
		loop {
			// if receive stop signal, break loop
			if self
				.stop_receiver
				.recv_timeout(Duration::from_millis(500))
				.is_ok()
			{
				break;
			}
			let change_count = unsafe { self.pasteboard.changeCount() };
			if last_change_count == 0 {
				last_change_count = change_count;
			} else if change_count != last_change_count {
				self.handlers
					.iter_mut()
					.for_each(|handler| handler.on_clipboard_change());
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
		let ns_pasteboard = unsafe { NSPasteboard::generalPasteboard() };
		let clipboard_ctx = ClipboardContext {
			pasteboard: ns_pasteboard,
		};
		Ok(clipboard_ctx)
	}

	fn plain(&self, r#type: &NSPasteboardType) -> Result<String> {
		autoreleasepool(|_| {
			let contents = unsafe { self.pasteboard.pasteboardItems() }
				.ok_or("NSPasteboard#pasteboardItems errored")?;
			for item in contents {
				if let Some(string) = unsafe { item.stringForType(r#type) } {
					return Ok(string.to_string());
				}
			}
			Err("No string found".into())
		})
	}

	fn set_files(&self, files: &[String]) -> Result<()> {
		let ns_string_arr = files
			.iter()
			.map(|f| NSString::from_str(f))
			.collect::<Vec<_>>();
		let array: Retained<NSArray<NSString>> = NSArray::from_retained_slice(&ns_string_arr);
		unsafe {
			self.pasteboard
				.setPropertyList_forType(&array, NSFilenamesPboardType)
		};
		Ok(())
	}

	// learn from https://github.com/zed-industries/zed/blob/79c1003b344ee513cf97ee8313c38c7c3f02c916/crates/gpui/src/platform/mac/platform.rs#L793
	fn write_to_clipboard(&self, data: &[ClipboardContent], with_clear: bool) -> Result<()> {
		if with_clear {
			unsafe {
				self.pasteboard.clearContents();
			}
		}
		autoreleasepool(|_| unsafe {
			let mut write_objects: Vec<
				Retained<ProtocolObject<(dyn NSPasteboardWriting + 'static)>>,
			> = vec![];
			for d in data {
				match d {
					ClipboardContent::Text(text) => {
						let item = NSPasteboardItem::new();
						item.setString_forType(&NSString::from_str(text), NSPasteboardTypeString);
						write_objects.push(ProtocolObject::from_retained(item));
					}
					ClipboardContent::Rtf(rtf) => {
						let item = NSPasteboardItem::new();
						item.setString_forType(&NSString::from_str(rtf), NSPasteboardTypeRTF);
						write_objects.push(ProtocolObject::from_retained(item));
					}
					ClipboardContent::Html(html) => {
						let item = NSPasteboardItem::new();
						item.setString_forType(&NSString::from_str(html), NSPasteboardTypeHTML);
						write_objects.push(ProtocolObject::from_retained(item));
					}
					ClipboardContent::Image(image) => {
						let png_img = image.to_png();
						if let Ok(png_buffer) = png_img {
							let bytes = png_buffer.get_bytes();
							let ns_data = {
								NSData::initWithBytes_length(
									NSData::alloc(),
									bytes.as_ptr() as *mut c_void,
									bytes.len(),
								)
							};
							let item = NSPasteboardItem::new();
							item.setData_forType(&ns_data, NSPasteboardTypePNG);
							write_objects.push(ProtocolObject::from_retained(item));
						};
					}
					ClipboardContent::Files(files) => {
						let _ = self.set_files(files);
					}
					ClipboardContent::Other(format, buffer) => {
						let ns_data = {
							NSData::initWithBytes_length(
								NSData::alloc(),
								buffer.as_ptr() as *mut c_void,
								buffer.len(),
							)
						};
						self.pasteboard.declareTypes_owner(
							&NSArray::from_retained_slice(&[NSString::from_str(format)]),
							None,
						);
						let item = NSPasteboardItem::new();
						item.setData_forType(&ns_data, &NSString::from_str(format));
						write_objects.push(ProtocolObject::from_retained(item));
					}
				}
			}
			if !self
				.pasteboard
				.writeObjects(&NSArray::from_retained_slice(&write_objects))
			{
				return Err("writeObjects failed");
			}
			Ok(())
		})?;
		Ok(())
	}
}

unsafe impl Send for ClipboardContext {}

unsafe impl Sync for ClipboardContext {}

impl Clipboard for ClipboardContext {
	fn available_formats(&self) -> Result<Vec<String>> {
		let types = unsafe { self.pasteboard.types() }.ok_or("NSPasteboard#types errored")?;
		let res = types.iter().map(|t| t.to_string()).collect();
		Ok(res)
	}

	fn has(&self, format: ContentFormat) -> bool {
		match format {
			ContentFormat::Text => unsafe {
				let types = NSArray::arrayWithObject(NSPasteboardTypeString);
				// https://developer.apple.com/documentation/appkit/nspasteboard/1526078-availabletypefromarray?language=objc
				// The first pasteboard type in types that is available on the pasteboard, or nil if the receiver does not contain any of the types in types.
				// self.clipboard.availableTypeFromArray(types)
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
			ContentFormat::Rtf => unsafe {
				let types = NSArray::arrayWithObject(NSPasteboardTypeRTF);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
			ContentFormat::Html => unsafe {
				// Currently only judge whether there is a public.html format
				let types = NSArray::arrayWithObject(NSPasteboardTypeHTML);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
			ContentFormat::Image => unsafe {
				// Currently only judge whether there is a png format
				let types = NSArray::from_retained_slice(&[
					NSPasteboardTypePNG.to_owned(),
					NSPasteboardTypeTIFF.to_owned(),
				]);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
			ContentFormat::Files => unsafe {
				let types = NSArray::arrayWithObject(NSFilenamesPboardType);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
			ContentFormat::Other(format) => unsafe {
				let types = NSArray::from_retained_slice(&[NSString::from_str(&format)]);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			},
		}
	}

	fn clear(&self) -> Result<()> {
		unsafe { self.pasteboard.clearContents() };
		Ok(())
	}

	fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
		if let Some(data) = unsafe { self.pasteboard.dataForType(&NSString::from_str(format)) } {
			return Ok(data.to_vec());
		}
		Err("no data".into())
	}

	fn get_text(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeString })
	}

	fn get_rich_text(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeRTF })
	}

	fn get_html(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeHTML })
	}

	fn get_image(&self) -> Result<RustImageData> {
		autoreleasepool(|_| {
			let png_data = unsafe { self.pasteboard.dataForType(NSPasteboardTypePNG) };
			if let Some(data) = png_data {
				return RustImageData::from_bytes(&data.to_vec());
			};
			// if no png data, read NSImage;
			let ns_image =
				unsafe { NSImage::initWithPasteboard(NSImage::alloc(), &self.pasteboard) };
			if let Some(image) = ns_image {
				let tiff_data = unsafe { image.TIFFRepresentation() };
				if let Some(data) = tiff_data {
					return RustImageData::from_bytes(&data.to_vec());
				}
			};
			Err("no image data".into())
		})
	}

	fn get_files(&self) -> Result<Vec<String>> {
		let mut res = vec![];
		let ns_array = unsafe { self.pasteboard.propertyListForType(NSFilenamesPboardType) };
		unsafe {
			if let Some(array) = ns_array {
				// cast to NSArray<NSString>
				let array: Retained<NSArray<NSString>> = Retained::cast_unchecked(array);
				array.iter().for_each(|item| {
					res.push(item.to_string());
				});
			}
		}
		if res.is_empty() {
			return Err("no files".into());
		}
		Ok(res)
	}

	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		autoreleasepool(|_| {
			let contents = unsafe { self.pasteboard.pasteboardItems() }
				.ok_or("NSPasteboard#pasteboardItems errored")?;
			let mut results = Vec::new();
			for format in formats {
				for item in contents.iter() {
					match format {
						ContentFormat::Text => {
							if let Some(string) =
								unsafe { item.stringForType(NSPasteboardTypeString) }
							{
								results.push(ClipboardContent::Text(string.to_string()));
								break;
							}
						}
						ContentFormat::Rtf => {
							if let Some(string) = unsafe { item.stringForType(NSPasteboardTypeRTF) }
							{
								results.push(ClipboardContent::Rtf(string.to_string()));
								break;
							}
						}
						ContentFormat::Html => {
							if let Some(string) =
								unsafe { item.stringForType(NSPasteboardTypeHTML) }
							{
								results.push(ClipboardContent::Html(string.to_string()));
								break;
							}
						}
						ContentFormat::Image => {
							if let Ok(image) = self.get_image() {
								results.push(ClipboardContent::Image(image));
								break;
							}
						}
						ContentFormat::Files => {
							if let Ok(files) = self.get_files() {
								results.push(ClipboardContent::Files(files));
								break;
							}
						}
						ContentFormat::Other(format_name) => {
							if let Some(data) =
								unsafe { item.dataForType(&NSString::from_str(format_name)) }
							{
								results.push(ClipboardContent::Other(
									format_name.to_string(),
									data.to_vec(),
								));
								break;
							}
						}
					}
				}
			}
			Ok(results)
		})
	}

	fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Other(format.to_owned(), buffer)], true)
	}

	fn set_text(&self, text: String) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Text(text)], true)
	}

	fn set_rich_text(&self, text: String) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Rtf(text)], true)
	}

	fn set_html(&self, html: String) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Html(html)], true)
	}

	fn set_image(&self, image: RustImageData) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Image(image)], true)
	}

	fn set_files(&self, files: Vec<String>) -> Result<()> {
		if files.is_empty() {
			return Err("file list is empty".into());
		}
		let _ = self.clear();
		self.set_files(&files)
	}

	fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
		if contents.is_empty() {
			return Err(
				"contents is empty, if you want to clear clipboard, please use clear method".into(),
			);
		}
		self.write_to_clipboard(&contents, true)
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
