#[cfg(feature = "image")]
use crate::common::ClipboardImage;
use crate::common::{ClipboardError, Result};
#[cfg(feature = "async")]
use crate::AsyncClipboard;
use crate::{Clipboard, ClipboardContent, ClipboardContentBuilder, ContentFormat};
use objc2::rc::Retained;
#[cfg(feature = "image")]
use objc2::AllocAnyThread;
use objc2::{rc::autoreleasepool, runtime::ProtocolObject, ClassType};
#[cfg(feature = "image")]
use objc2_app_kit::{NSImage, NSPasteboardTypePNG, NSPasteboardTypeTIFF};
use objc2_app_kit::{
	NSPasteboard, NSPasteboardItem, NSPasteboardType, NSPasteboardTypeFileURL,
	NSPasteboardTypeHTML, NSPasteboardTypeRTF, NSPasteboardTypeString,
};
use objc2_foundation::{NSArray, NSData, NSString, NSURL};
use std::ffi::c_void;

pub struct ClipboardContext {
	pasteboard: Retained<NSPasteboard>,
}

impl ClipboardContext {
	pub fn new() -> Result<ClipboardContext> {
		let ns_pasteboard = NSPasteboard::generalPasteboard();
		let clipboard_ctx = ClipboardContext {
			pasteboard: ns_pasteboard,
		};
		Ok(clipboard_ctx)
	}

	fn plain(&self, r#type: &NSPasteboardType) -> Result<String> {
		autoreleasepool(|_| {
			let contents =
				self.pasteboard
					.pasteboardItems()
					.ok_or(ClipboardError::PlatformError(
						"NSPasteboard#pasteboardItems errored".into(),
					))?;
			for item in contents {
				if let Some(string) = item.stringForType(r#type) {
					return Ok(string.to_string());
				}
			}
			Err(ClipboardError::Empty)
		})
	}

	fn set_files(&self, files: &[&str]) -> Result<()> {
		autoreleasepool(|_| {
			// Build NSArray<NSURL> and write via writeObjects for better compatibility
			let urls: Vec<Retained<NSURL>> = files
				.iter()
				.filter_map(|file_path| {
					// Normalize to local filesystem path, and verify it exists
					let local_path = if file_path.starts_with("file://") {
						file_path.trim_start_matches("file://")
					} else {
						file_path
					};
					if !std::path::Path::new(local_path).exists() {
						return None;
					}
					let ns_path = NSString::from_str(local_path);
					// NSURL::fileURLWithPath returns a retained NSURL
					let url = NSURL::fileURLWithPath(&ns_path);
					Some(url)
				})
				.collect();
			if urls.is_empty() {
				return Err(ClipboardError::InvalidData("no valid files".into()));
			}
			let write_objects = NSArray::from_retained_slice(
				&urls
					.iter()
					.map(|u| ProtocolObject::from_retained(u.clone()))
					.collect::<Vec<_>>(),
			);
			if !self.pasteboard.writeObjects(&write_objects) {
				return Err(ClipboardError::PlatformError(
					"writeObjects failed for files".into(),
				));
			}
			Ok(())
		})
	}

	// learn from https://github.com/zed-industries/zed/blob/79c1003b344ee513cf97ee8313c38c7c3f02c916/crates/gpui/src/platform/mac/platform.rs#L793
	fn write_to_clipboard(&self, data: &[ClipboardContent], with_clear: bool) -> Result<()> {
		if with_clear {
			self.pasteboard.clearContents();
		}
		autoreleasepool(|_| {
			// we create one NSPasteboardItem for all representations of the same content
			let item = NSPasteboardItem::new();
			let mut has_content_other_than_files = false;

			for d in data {
				match d {
					ClipboardContent::Text(text) => {
						item.setString_forType(&NSString::from_str(text), unsafe {
							NSPasteboardTypeString
						});
						has_content_other_than_files = true;
					}
					ClipboardContent::Rtf(rtf) => {
						let rtf_data = unsafe {
							NSData::dataWithBytes_length(rtf.as_ptr() as *const c_void, rtf.len())
						};
						item.setData_forType(&rtf_data, unsafe { NSPasteboardTypeRTF });
						has_content_other_than_files = true;
					}
					ClipboardContent::Html(html) => {
						item.setString_forType(&NSString::from_str(html), unsafe {
							NSPasteboardTypeHTML
						});
						has_content_other_than_files = true;
					}
					#[cfg(feature = "image")]
					ClipboardContent::Image(image) => {
						if let Ok(png_buffer) = image.to_png_sync() {
							let bytes = &png_buffer;
							let ns_data = unsafe {
								NSData::dataWithBytes_length(
									bytes.as_ptr() as *mut c_void,
									bytes.len(),
								)
							};
							item.setData_forType(&ns_data, unsafe { NSPasteboardTypePNG });
							has_content_other_than_files = true;
						};
					}
					ClipboardContent::Files(files) => {
						let string_files: Vec<&str> = files.iter().map(|f| f.as_str()).collect();
						let _ = self.set_files(&string_files);
					}
					ClipboardContent::Other(format, buffer) => {
						let ns_data = unsafe {
							NSData::dataWithBytes_length(
								buffer.as_ptr() as *mut c_void,
								buffer.len(),
							)
						};
						item.setData_forType(&ns_data, &NSString::from_str(format));
						has_content_other_than_files = true;
					}
				}
			}
			if has_content_other_than_files {
				let write_objects =
					NSArray::from_retained_slice(&[ProtocolObject::from_retained(item)]);
				if !self.pasteboard.writeObjects(&write_objects) {
					return Err(ClipboardError::PlatformError("writeObjects failed".into()));
				}
			}
			Ok(())
		})?;
		Ok(())
	}
}

unsafe impl Send for ClipboardContext {}

#[async_trait::async_trait]
#[cfg(feature = "async")]
impl AsyncClipboard for ClipboardContext {
	async fn available_formats(&self) -> Result<Vec<String>> {
		Clipboard::available_formats(self)
	}

	async fn has(&self, format: ContentFormat) -> Result<bool> {
		Ok(Clipboard::has(self, format))
	}

	async fn clear(&self) -> Result<()> {
		Clipboard::clear(self)
	}

	async fn get_raw(&self, format: &str) -> Result<Vec<u8>> {
		Clipboard::get_raw(self, format)
	}

	async fn get_text(&self) -> Result<String> {
		Clipboard::get_text(self)
	}

	async fn get_rtf(&self) -> Result<String> {
		Clipboard::get_rtf(self)
	}

	async fn get_html(&self) -> Result<String> {
		Clipboard::get_html(self)
	}

	#[cfg(feature = "image")]
	async fn get_image(&self) -> Result<ClipboardImage> {
		Clipboard::get_image(self)
	}

	async fn get_files(&self) -> Result<Vec<String>> {
		Clipboard::get_files(self)
	}

	async fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		Clipboard::get(self, formats)
	}

	async fn set_raw(&self, format: &str, data: &[u8]) -> Result<()> {
		Clipboard::set_raw(self, format, data)
	}

	async fn set_text(&self, text: &str) -> Result<()> {
		Clipboard::set_text(self, text)
	}

	async fn set_rtf(&self, rtf: &str) -> Result<()> {
		Clipboard::set_rtf(self, rtf)
	}

	async fn set_html(&self, html: &str) -> Result<()> {
		Clipboard::set_html(self, html)
	}

	#[cfg(feature = "image")]
	async fn set_image(&self, image: ClipboardImage) -> Result<()> {
		Clipboard::set_image(self, image)
	}

	async fn set_files(&self, files: &[&str]) -> Result<()> {
		Clipboard::set_files(self, files)
	}

	async fn set(&self, builder: ClipboardContentBuilder) -> Result<()> {
		Clipboard::set(self, builder)
	}
}

unsafe impl Sync for ClipboardContext {}

impl Clipboard for ClipboardContext {
	fn available_formats(&self) -> Result<Vec<String>> {
		let types = self
			.pasteboard
			.types()
			.ok_or(ClipboardError::PlatformError(
				"NSPasteboard#types errored".into(),
			))?;
		let res = types.iter().map(|t| t.to_string()).collect();
		Ok(res)
	}

	fn has(&self, format: ContentFormat) -> bool {
		match format {
			ContentFormat::Text => {
				let types = NSArray::arrayWithObject(unsafe { NSPasteboardTypeString });
				// https://developer.apple.com/documentation/appkit/nspasteboard/1526078-availabletypefromarray?language=objc
				// The first pasteboard type in types that is available on the pasteboard, or nil if the receiver does not contain any of the types in types.
				// self.clipboard.availableTypeFromArray(types)
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
			ContentFormat::Rtf => {
				let types = NSArray::arrayWithObject(unsafe { NSPasteboardTypeRTF });
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
			ContentFormat::Html => {
				// Currently only judge whether there is a public.html format
				let types = NSArray::arrayWithObject(unsafe { NSPasteboardTypeHTML });
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
			#[cfg(feature = "image")]
			ContentFormat::Image => {
				// Currently only judge whether there is a png format
				let types = NSArray::from_retained_slice(&[
					unsafe { NSPasteboardTypePNG }.to_owned(),
					unsafe { NSPasteboardTypeTIFF }.to_owned(),
				]);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
			ContentFormat::Files => {
				let types = NSArray::arrayWithObject(unsafe { NSPasteboardTypeFileURL });
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
			ContentFormat::Other(format) => {
				let types = NSArray::from_retained_slice(&[NSString::from_str(&format)]);
				self.pasteboard.availableTypeFromArray(&types).is_some()
			}
		}
	}

	fn clear(&self) -> Result<()> {
		self.pasteboard.clearContents();
		Ok(())
	}

	fn get_raw(&self, format: &str) -> Result<Vec<u8>> {
		if let Some(data) = self.pasteboard.dataForType(&NSString::from_str(format)) {
			return Ok(data.to_vec());
		}
		Err(ClipboardError::Empty)
	}

	fn get_text(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeString })
	}

	fn get_rtf(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeRTF })
	}

	fn get_html(&self) -> Result<String> {
		self.plain(unsafe { NSPasteboardTypeHTML })
	}

	fn get_files(&self) -> Result<Vec<String>> {
		autoreleasepool(|_| {
			let mut res = vec![];
			// 使用 readObjectsForClasses 读取 NSURL（文件 URL）
			// 相当于 Objective-C: [pasteboard readObjectsForClasses:@[[NSURL class]] options:nil]
			let classes = NSArray::arrayWithObject(NSURL::class());
			let objects = unsafe {
				self.pasteboard
					.readObjectsForClasses_options(&classes, None)
			};
			if let Some(objects) = objects {
				for any_obj in objects {
					// 尝试将返回对象视为 NSURL
					if let Ok(url) = any_obj.downcast::<NSURL>() {
						// 只接受文件 URL
						if url.isFileURL() {
							// 优先使用 path（去掉 file:// 前缀后的本地路径）
							if let Some(path) = url.path() {
								res.push(path.to_string());
							} else {
								// 兜底使用 absoluteString，再去掉前缀
								let abs = if let Some(s) = url.absoluteString() {
									s.to_string()
								} else {
									String::new()
								};
								let file_path = if abs.starts_with("file://") {
									abs.strip_prefix("file://").unwrap_or(&abs).to_string()
								} else {
									abs
								};
								res.push(file_path);
							}
						}
					}
				}
			}
			if res.is_empty() {
				return Err(ClipboardError::Empty);
			}
			Ok(res)
		})
	}

	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		autoreleasepool(|_| {
			let contents =
				self.pasteboard
					.pasteboardItems()
					.ok_or(ClipboardError::PlatformError(
						"NSPasteboard#pasteboardItems errored".into(),
					))?;
			let mut results = Vec::new();
			for format in formats {
				for item in contents.iter() {
					match format {
						ContentFormat::Text => {
							if let Some(string) =
								item.stringForType(unsafe { NSPasteboardTypeString })
							{
								results.push(ClipboardContent::Text(string.to_string()));
								break;
							}
						}
						ContentFormat::Rtf => {
							if let Some(string) = item.stringForType(unsafe { NSPasteboardTypeRTF })
							{
								results.push(ClipboardContent::Rtf(string.to_string()));
								break;
							}
						}
						ContentFormat::Html => {
							if let Some(string) =
								item.stringForType(unsafe { NSPasteboardTypeHTML })
							{
								results.push(ClipboardContent::Html(string.to_string()));
								break;
							}
						}
						#[cfg(feature = "image")]
						ContentFormat::Image => {
							if let Ok(image) = Clipboard::get_image(self) {
								results.push(ClipboardContent::Image(image));
								break;
							}
						}
						ContentFormat::Files => {
							if let Ok(files) = Clipboard::get_files(self) {
								results.push(ClipboardContent::Files(files));
								break;
							}
						}
						ContentFormat::Other(format_name) => {
							if let Some(data) = item.dataForType(&NSString::from_str(format_name)) {
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

	fn set_raw(&self, format: &str, data: &[u8]) -> Result<()> {
		self.write_to_clipboard(
			&[ClipboardContent::Other(format.to_owned(), data.to_vec())],
			true,
		)
	}

	fn set_text(&self, text: &str) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Text(text.to_string())], true)
	}

	fn set_rtf(&self, rtf: &str) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Rtf(rtf.to_string())], true)
	}

	fn set_html(&self, html: &str) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Html(html.to_string())], true)
	}

	fn set_files(&self, files: &[&str]) -> Result<()> {
		if files.is_empty() {
			return Err(ClipboardError::InvalidData("file list is empty".into()));
		}
		let _ = Clipboard::clear(self);
		self.set_files(files)
	}

	fn set(&self, builder: ClipboardContentBuilder) -> Result<()> {
		let contents = builder.build();
		if contents.is_empty() {
			return Err(ClipboardError::InvalidData(
				"contents is empty, if you want to clear clipboard, please use clear method".into(),
			));
		}
		self.write_to_clipboard(&contents, true)
	}

	#[cfg(feature = "image")]
	fn get_image(&self) -> Result<ClipboardImage> {
		autoreleasepool(|_| {
			let png_data = self.pasteboard.dataForType(unsafe { NSPasteboardTypePNG });
			if let Some(data) = png_data {
				return ClipboardImage::from_bytes_sync(&data.to_vec());
			};
			// if no png data, read NSImage;
			let ns_image = NSImage::initWithPasteboard(NSImage::alloc(), &self.pasteboard);
			if let Some(image) = ns_image {
				let tiff_data = image.TIFFRepresentation();
				if let Some(data) = tiff_data {
					return ClipboardImage::from_bytes_sync(&data.to_vec());
				}
			};
			Err(ClipboardError::Empty)
		})
	}

	#[cfg(feature = "image")]
	fn set_image(&self, image: ClipboardImage) -> Result<()> {
		self.write_to_clipboard(&[ClipboardContent::Image(image)], true)
	}
}

/// 启动 macOS 平台的异步剪贴板监听
#[cfg(feature = "async")]
pub fn start_async_watch(
	sender: tokio::sync::mpsc::Sender<crate::ClipboardEvent>,
	shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
	// 克隆sender用于在线程中移动
	let sender_clone = sender.clone();

	tokio::task::spawn_blocking(move || {
		// 在阻塞任务中运行监听循环，避免在异步任务中使用非 Send/Sync 对象
		let pasteboard = NSPasteboard::generalPasteboard();
		let mut last_change_count = pasteboard.changeCount();

		loop {
			// 检查是否收到停止信号
			if *shutdown_rx.borrow() {
				// 收到停止信号，退出循环
				println!("Shutting down clipboard watcher");
				break;
			}

			// 使用 std::thread::sleep 而不是 tokio::time::sleep
			std::thread::sleep(std::time::Duration::from_millis(500));

			let change_count = pasteboard.changeCount();
			if last_change_count == 0 {
				last_change_count = change_count;
			} else if change_count != last_change_count {
				// 获取可用格式
				let formats = if let Some(types) = pasteboard.types() {
					types
						.iter()
						.map(|t| {
							let type_str = t.to_string();
							// 将常见的格式映射到ContentFormat枚举
							match type_str.as_str() {
								"public.utf8-plain-text" => crate::ContentFormat::Text,
								"public.html" => crate::ContentFormat::Html,
								"public.rtf" => crate::ContentFormat::Rtf,
								"public.png" | "public.tiff" => {
									#[cfg(feature = "image")]
									{
										crate::ContentFormat::Image
									}
									#[cfg(not(feature = "image"))]
									{
										crate::ContentFormat::Other(type_str.clone())
									}
								}
								"public.file-url" => crate::ContentFormat::Files,
								_ => crate::ContentFormat::Other(type_str),
							}
						})
						.collect()
				} else {
					vec![]
				};

				// 发送事件到异步运行时
				if let Err(_) =
					sender_clone.blocking_send(crate::ClipboardEvent::Changed { formats })
				{
					// 接收端已关闭，退出循环
					println!("Failed to send clipboard change event, then stop watching");
					break;
				}
				last_change_count = change_count;
			}
		}
	});
}
