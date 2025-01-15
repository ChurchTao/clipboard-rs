use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::common::{ContentData, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardContent, ClipboardHandler, ClipboardWatcher, ContentFormat};
use clipboard_win::raw::{set_bitmap_with, set_file_list_with, set_string_with, set_without_clear};
use clipboard_win::types::c_uint;
use clipboard_win::{
	formats, get, get_clipboard, options, raw, set_clipboard, Clipboard as ClipboardWin, Monitor,
	SysResult,
};
use image::codecs::bmp::BmpDecoder;
use image::DynamicImage;

pub struct WatcherShutdown {
	stop_signal: Sender<()>,
}

static UNKNOWN_FORMAT: &str = "unknown format";
static CF_RTF: &str = "Rich Text Format";
static CF_HTML: &str = "HTML Format";
static CF_PNG: &str = "PNG";

pub struct ClipboardContext {
	format_map: HashMap<&'static str, c_uint>,
	html_format: formats::Html,
}

pub struct ClipboardWatcherContext<T: ClipboardHandler> {
	handlers: Vec<T>,
	stop_signal: Sender<()>,
	stop_receiver: Receiver<()>,
	running: bool,
}

unsafe impl Send for ClipboardContext {}
unsafe impl Sync for ClipboardContext {}
unsafe impl<T: ClipboardHandler> Send for ClipboardWatcherContext<T> {}
unsafe impl<T: ClipboardHandler> Sync for ClipboardWatcherContext<T> {}

impl ClipboardContext {
	pub fn new() -> Result<ClipboardContext> {
		let (format_map, html_format) = {
			let cf_html_format = formats::Html::new();
			let cf_rtf_uint = clipboard_win::register_format(CF_RTF);
			let cf_png_uint = clipboard_win::register_format(CF_PNG);
			let mut m: HashMap<&str, c_uint> = HashMap::new();
			if let Some(cf_html) = cf_html_format {
				m.insert(CF_HTML, cf_html.code());
			}
			if let Some(cf_rtf) = cf_rtf_uint {
				m.insert(CF_RTF, cf_rtf.get());
			}
			if let Some(cf_png) = cf_png_uint {
				m.insert(CF_PNG, cf_png.get());
			}
			(m, cf_html_format)
		};
		Ok(ClipboardContext {
			format_map,
			html_format: html_format.ok_or("register html format error")?,
		})
	}

	fn get_format(&self, format: &ContentFormat) -> c_uint {
		match format {
			ContentFormat::Text => formats::CF_UNICODETEXT,
			ContentFormat::Rtf => *self.format_map.get(CF_RTF).unwrap(),
			ContentFormat::Html => *self.format_map.get(CF_HTML).unwrap(),
			ContentFormat::Image => formats::CF_DIB,
			ContentFormat::Files => formats::CF_HDROP,
			ContentFormat::Other(format) => clipboard_win::register_format(format).unwrap().get(),
		}
	}
}

impl<T: ClipboardHandler> ClipboardWatcherContext<T> {
	pub fn new() -> Result<Self> {
		let (tx, rx) = std::sync::mpsc::channel();
		Ok(Self {
			handlers: Vec::new(),
			stop_signal: tx,
			stop_receiver: rx,
			running: false,
		})
	}
}

impl Clipboard for ClipboardContext {
	fn available_formats(&self) -> Result<Vec<String>> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
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
					res.push(UNKNOWN_FORMAT.to_string());
				}
			}
		});
		Ok(res)
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
					|| clipboard_win::is_format_avail(formats::CF_DIB)
			}
			ContentFormat::Files => clipboard_win::is_format_avail(formats::CF_HDROP),
			ContentFormat::Other(format) => {
				let format_uint = clipboard_win::register_format(format.as_str());
				if let Some(format_uint) = format_uint {
					return clipboard_win::is_format_avail(format_uint.get());
				}
				false
			}
		}
	}

	fn clear(&self) -> Result<()> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let res = clipboard_win::empty();
		if let Err(e) = res {
			return Err(format!("Empty clipboard error, code = {}", e).into());
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
			Err(e) => Err(format!("Get buffer error, code = {}", e).into()),
		}
	}

	fn get_text(&self) -> Result<String> {
		let string: SysResult<String> = get_clipboard(formats::Unicode);
		match string {
			Ok(s) => Ok(s),
			Err(e) => Err(format!("Get text error, code = {}", e).into()),
		}
	}

	fn get_rich_text(&self) -> Result<String> {
		let rtf_raw_data = self.get_buffer(CF_RTF)?;
		Ok(String::from_utf8_lossy(&rtf_raw_data).to_string())
	}

	fn get_html(&self) -> Result<String> {
		let buffer = get_clipboard(formats::RawData(self.html_format.code()));
		match buffer {
			Ok(data) => {
				let html_res = String::from_utf8(data);
				if let Ok(html_full_str) = html_res {
					let html = extract_html_from_clipboard_data(html_full_str.as_str());
					if let Ok(html) = html {
						return Ok(html);
					}
				}
				Err("Get html error".into())
			}
			Err(e) => Err(format!("Get buffer error, code = {}", e).into()),
		}
	}

	fn get_image(&self) -> Result<RustImageData> {
		let cf_png_format = self.format_map.get(CF_PNG);
		if cf_png_format.is_some() && clipboard_win::is_format_avail(*cf_png_format.unwrap()) {
			let image_raw_data = self.get_buffer(CF_PNG)?;
			RustImageData::from_bytes(&image_raw_data)
		} else if clipboard_win::is_format_avail(formats::CF_DIBV5) {
			let res = get_clipboard(formats::RawData(formats::CF_DIBV5));
			match res {
				Ok(data) => {
					let decoder = {
						// if data.as_slice().starts_with(b"BM") {
						// 	BmpDecoder::new(Cursor::new(data.as_slice()))
						// } else {
						BmpDecoder::new_without_file_header(Cursor::new(data.as_slice()))
						// }
					};
					let decoder = decoder.map_err(|e| format!("{}", e))?;
					let dynamic_image =
						DynamicImage::from_decoder(decoder).map_err(|e| format!("{}", e))?;
					Ok(RustImageData::from_dynamic_image(dynamic_image))
				}
				Err(e) => Err(format!("Get image error, code = {}", e).into()),
			}
		} else if clipboard_win::is_format_avail(formats::CF_DIB) {
			let res = get_clipboard(formats::Bitmap);
			match res {
				Ok(data) => RustImageData::from_bytes(&data),
				Err(e) => Err(format!("Get image error, code = {}", e).into()),
			}
		} else {
			Err("No image data in clipboard".into())
		}
	}

	fn get_files(&self) -> Result<Vec<String>> {
		let files: SysResult<Vec<String>> = get_clipboard(formats::FileList);
		match files {
			Ok(f) => Ok(f),
			Err(e) => Err(format!("Get files error, code = {}", e).into()),
		}
	}

	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let mut res = Vec::new();
		for format in formats {
			match format {
				ContentFormat::Text => {
					let r = get(formats::Unicode);
					match r {
						Ok(txt) => {
							res.push(ClipboardContent::Text(txt));
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Rtf => {
					let format_uint = self.get_format(format);
					let buffer = get(formats::RawData(format_uint));
					match buffer {
						Ok(buffer) => {
							let rtf = String::from_utf8_lossy(&buffer);
							res.push(ClipboardContent::Rtf(rtf.to_string()));
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Html => {
					let html_buffer = get(formats::RawData(self.html_format.code()));
					match html_buffer {
						Ok(html) => {
							let html_res = String::from_utf8(html);
							if let Ok(html_full_str) = html_res {
								let html = extract_html_from_clipboard_data(html_full_str.as_str());
								if let Ok(html) = html {
									res.push(ClipboardContent::Html(html));
								}
							}
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Image => {
					let img = self.get_image();
					match img {
						Ok(img) => {
							res.push(ClipboardContent::Image(img));
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Other(fmt) => {
					let format_uint = self.get_format(format);
					let buffer = get(formats::RawData(format_uint));
					match buffer {
						Ok(buffer) => {
							res.push(ClipboardContent::Other(fmt.clone(), buffer));
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Files => {
					let files = self.get_files();
					match files {
						Ok(files) => {
							res.push(ClipboardContent::Files(files));
						}
						Err(_) => continue,
					}
				}
			}
		}
		Ok(res)
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
		res.map_err(|e| format!("set text error, code = {}", e).into())
	}

	fn set_rich_text(&self, text: String) -> Result<()> {
		let res = self.set_buffer(CF_RTF, text.as_bytes().to_vec());
		res.map_err(|e| format!("set rich text error, code = {}", e).into())
	}

	fn set_html(&self, html: String) -> Result<()> {
		let cf_html = plain_html_to_cf_html(&html);
		let res = set_clipboard(
			formats::RawData(self.html_format.code()),
			cf_html.as_bytes(),
		);
		res.map_err(|e| format!("set html error, code = {}", e).into())
	}

	fn set_image(&self, image: RustImageData) -> Result<()> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let res = clipboard_win::empty();
		if let Err(e) = res {
			return Err(format!("Empty clipboard error, code = {}", e).into());
		}
		// chromium source code
		// @link {https://source.chromium.org/chromium/chromium/src/+/main:ui/base/clipboard/clipboard_win.cc;l=771;drc=2a5aaed0ff3a0895c8551495c2656ed49baf742c;bpv=0;bpt=1}
		let cf_png_format = self.format_map.get(CF_PNG);
		if cf_png_format.is_some() {
			let png = image.to_png()?;
			let write_png_res = set_without_clear(*cf_png_format.unwrap(), png.get_bytes());
			if let Err(e) = write_png_res {
				return Err(format!("set png image error, code = {}", e).into());
			}
		}
		let bmp = image
			.to_bitmap()
			.map_err(|e| format!("to bitmap error, code = {}", e))?;
		let res = set_bitmap_with(bmp.get_bytes(), options::NoClear);
		res.map_err(|e| format!("set image error, code = {}", e).into())
	}

	fn set_files(&self, files: Vec<String>) -> Result<()> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let res = set_file_list_with(&files, options::DoClear);
		res.map_err(|e| format!("set files error, code = {}", e).into())
	}

	fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let res = clipboard_win::empty();
		if let Err(e) = res {
			return Err(format!("Empty clipboard error, code = {}", e).into());
		}
		for content in contents {
			match content {
				ClipboardContent::Text(txt) => {
					let res = set_string_with(txt.as_str(), options::NoClear);
					if res.is_err() {
						continue;
					}
				}
				ClipboardContent::Html(html) => {
					let format_uint_html = self.html_format.code();
					let res = set_without_clear(format_uint_html, html.as_bytes());
					if res.is_err() {
						continue;
					}
				}
				ClipboardContent::Image(img) => {
					// set image will clear clipboard
					let res = self.set_image(img);
					if res.is_err() {
						continue;
					}
				}
				ClipboardContent::Rtf(_) | ClipboardContent::Other(_, _) => {
					let format_uint = self.get_format(&content.get_format());
					let res = set_without_clear(format_uint, content.as_bytes());
					if res.is_err() {
						continue;
					}
				}
				ClipboardContent::Files(file_list) => {
					let res = set_file_list_with(&file_list, options::NoClear);
					if res.is_err() {
						continue;
					}
				}
			}
		}
		Ok(())
	}
}

impl ClipboardContext {
	pub fn set_png_image(&self, png_image: &DynamicImage) -> Result<()> {
		let _clip = ClipboardWin::new_attempts(10)
			.map_err(|code| format!("Open clipboard error, code = {}", code));
		let res = clipboard_win::empty();
		if let Err(e) = res {
			return Err(format!("Empty clipboard error, code = {}", e).into());
		}

		image_data::add_png_image(png_image)?;
		image_data::add_cf_dibv5(png_image)?;
		Ok(())
	}
}

impl<T: ClipboardHandler> ClipboardWatcher<T> for ClipboardWatcherContext<T> {
	fn add_handler(&mut self, f: T) -> &mut Self {
		self.handlers.push(f);
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
		let mut monitor = Monitor::new().expect("create monitor error");
		let shutdown = monitor.shutdown_channel();
		loop {
			if self.stop_receiver.try_recv().is_ok() {
				break;
			}
			let msg = monitor.try_recv();
			match msg {
				Ok(true) => {
					self.handlers.iter_mut().for_each(|f| {
						f.on_clipboard_change();
					});
				}
				Ok(false) => {
					// no change
					thread::park_timeout(Duration::from_millis(200));
					continue;
				}
				Err(e) => {
					eprintln!("watch error, code = {}", e);
					break;
				}
			}
		}
		drop(shutdown);
		self.running = false;
	}

	fn get_shutdown_channel(&self) -> WatcherShutdown {
		WatcherShutdown {
			stop_signal: self.stop_signal.clone(),
		}
	}
}

impl Drop for WatcherShutdown {
	fn drop(&mut self) {
		let _ = self.stop_signal.send(());
	}
}

/// 将输入的 UTF-8 字符串转换为宽字符（UTF-16）字符串
// fn utf8_to_utf16(input: &str) -> Vec<u16> {
// 	let mut vec: Vec<u16> = input.encode_utf16().collect();
// 	vec.push(0);
// 	vec
// }

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
	if !fragment.starts_with("<html>") {
		buffer.push_str("<html>\r\n<body>\r\n<!--StartFragment-->");
	}

	let start_fragment_pos = buffer.len();
	buffer.push_str(fragment);

	let end_fragment_pos = buffer.len();
	if !fragment.ends_with("</html>") {
		buffer.push_str("<!--EndFragment-->\r\n</body>\r\n</html>");
	}

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

const SEP: char = ':';
const START_HTML: &str = "StartHTML";
const END_HTML: &str = "EndHTML";

fn extract_html_from_clipboard_data(data: &str) -> Result<String> {
	let mut start_idx = 0usize;
	let mut end_idx = data.len();
	for line in data.lines() {
		let mut split = line.split(SEP);
		let key = match split.next() {
			Some(key) => key,
			None => break,
		};
		let value = match split.next() {
			Some(value) => value,
			//Reached HTML
			None => break,
		};
		match key {
			START_HTML => match value.trim_start_matches('0').parse() {
				Ok(value) => {
					start_idx = value;
					continue;
				}
				//Should not really happen
				Err(_) => break,
			},
			END_HTML => match value.trim_start_matches('0').parse() {
				Ok(value) => {
					end_idx = value;
					continue;
				}
				//Should not really happen
				Err(_) => break,
			},
			_ => continue,
		}
	}
	//Make sure HTML writer didn't screw up offsets of fragment
	let size = match end_idx.checked_sub(start_idx) {
		Some(size) => size,
		None => return Err("Invalid HTML offsets".into()),
	};
	if size > data.len() {
		return Err("Invalid HTML offsets".into());
	};
	Ok(data[start_idx..end_idx].to_string())
}

mod image_data {
	use super::Result;
	use image::{DynamicImage, GenericImageView as _};
	use std::{borrow::Cow, io, ptr::copy_nonoverlapping};
	use windows::Win32::{
		Foundation::{HANDLE, HGLOBAL},
		Graphics::Gdi::{DeleteObject, BITMAPV5HEADER, BI_BITFIELDS, HGDIOBJ, LCS_GM_IMAGES},
		System::{
			DataExchange::SetClipboardData,
			Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND},
			Ole::CF_DIBV5,
		},
	};

	#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
	pub(crate) struct ScopeGuard<F: FnOnce()> {
		callback: Option<F>,
	}

	#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
	impl<F: FnOnce()> ScopeGuard<F> {
		#[cfg_attr(all(windows), allow(dead_code))]
		pub(crate) fn new(callback: F) -> Self {
			ScopeGuard {
				callback: Some(callback),
			}
		}
	}

	#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
	impl<F: FnOnce()> Drop for ScopeGuard<F> {
		fn drop(&mut self) {
			if let Some(callback) = self.callback.take() {
				(callback)();
			}
		}
	}

	fn last_error(message: &str) -> String {
		let os_error = io::Error::last_os_error();
		format!("{}: {}", message, os_error)
	}

	unsafe fn global_unlock_checked(hdata: HGLOBAL) -> Result<()> {
		// If the memory object is unlocked after decrementing the lock count, the function
		// returns zero and GetLastError returns NO_ERROR. If it fails, the return value is
		// zero and GetLastError returns a value other than NO_ERROR.
		GlobalUnlock(hdata)?;
		Ok(())
	}

	pub(super) fn add_cf_dibv5(image: &DynamicImage) -> Result<()> {
		// This constant is missing in windows-rs
		// https://github.com/microsoft/windows-rs/issues/2711
		#[allow(non_upper_case_globals)]
		const LCS_sRGB: u32 = 0x7352_4742;

		let header_size = size_of::<BITMAPV5HEADER>();
		let header = BITMAPV5HEADER {
			bV5Size: header_size as u32,
			bV5Width: image.width() as i32,
			bV5Height: image.height() as i32,
			bV5Planes: 1,
			bV5BitCount: 32,
			bV5Compression: BI_BITFIELDS,
			bV5SizeImage: (4 * image.width() * image.height()),
			bV5XPelsPerMeter: 0,
			bV5YPelsPerMeter: 0,
			bV5ClrUsed: 0,
			bV5ClrImportant: 0,
			bV5RedMask: 0x00ff0000,
			bV5GreenMask: 0x0000ff00,
			bV5BlueMask: 0x000000ff,
			bV5AlphaMask: 0xff000000,
			bV5CSType: LCS_sRGB,
			// SAFETY: Windows ignores this field because `bV5CSType` is not set to `LCS_CALIBRATED_RGB`.
			bV5Endpoints: unsafe { std::mem::zeroed() },
			bV5GammaRed: 0,
			bV5GammaGreen: 0,
			bV5GammaBlue: 0,
			bV5Intent: LCS_GM_IMAGES as u32, // I'm not sure about this.
			bV5ProfileData: 0,
			bV5ProfileSize: 0,
			bV5Reserved: 0,
		};

		// In theory we don't need to flip the image because we could just specify
		// a negative height in the header, which according to the documentation, indicates that the
		// image rows are in top-to-bottom order. HOWEVER: MS Word (and WordPad) cannot paste an image
		// that has a negative height in its header.
		let image = flip_v(image);

		let data_size = header_size + image.2.len();
		let hdata = unsafe { global_alloc(data_size)? };
		unsafe {
			let data_ptr = global_lock(hdata)?;
			let _unlock = ScopeGuard::new(|| {
				let _ = global_unlock_checked(hdata);
			});

			copy_nonoverlapping::<u8>((&header) as *const _ as *const u8, data_ptr, header_size);

			// Not using the `add` function, because that has a restriction, that the result cannot overflow isize
			let pixels_dst = (data_ptr as usize + header_size) as *mut u8;
			copy_nonoverlapping::<u8>(image.2.as_ptr(), pixels_dst, image.2.len());

			let dst_pixels_slice = std::slice::from_raw_parts_mut(pixels_dst, image.2.len());

			// If the non-allocating version of the function failed, we need to assign the new bytes to
			// the global allocation.
			if let Cow::Owned(new_pixels) = rgba_to_win(dst_pixels_slice) {
				// SAFETY: `data_ptr` is valid to write to and has no outstanding mutable borrows, and
				// `new_pixels` will be the same length as the original bytes.
				copy_nonoverlapping::<u8>(new_pixels.as_ptr(), data_ptr, new_pixels.len())
			}
		}

		if let Err(err) = unsafe { SetClipboardData(CF_DIBV5.0 as u32, Some(HANDLE(hdata.0))) } {
			let _ = unsafe { DeleteObject(HGDIOBJ(hdata.0)) };
			Err(err.into())
		} else {
			Ok(())
		}
	}

	pub(super) fn add_png_image(image: &DynamicImage) -> Result<()> {
		let buf = image.as_bytes();

		// Register PNG format.
		let format_id = match clipboard_win::register_format("PNG") {
			Some(format_id) => format_id.into(),
			None => return Err(last_error("Cannot register PNG clipboard format.").into()),
		};

		let data_size = buf.len();
		let hdata = unsafe { global_alloc(data_size)? };
		unsafe {
			let pixels_dst = global_lock(hdata)?;
			copy_nonoverlapping::<u8>(buf.as_ptr(), pixels_dst, data_size);
			let _ = global_unlock_checked(hdata);
		}

		if let Err(err) = unsafe { SetClipboardData(format_id, Some(HANDLE(hdata.0))) } {
			let _ = unsafe { DeleteObject(HGDIOBJ(hdata.0)) };
			return Err(format!("SetClipboardData Error {}", err).into());
		}
		Ok(())
	}

	unsafe fn global_alloc(bytes: usize) -> Result<HGLOBAL> {
		let hdata = GlobalAlloc(GHND, bytes)?;
		if hdata.is_invalid() {
			Err(last_error("Could not allocate global memory object").into())
		} else {
			Ok(hdata)
		}
	}

	unsafe fn global_lock(hmem: HGLOBAL) -> Result<*mut u8> {
		let data_ptr = GlobalLock(hmem) as *mut u8;
		if data_ptr.is_null() {
			Err(last_error("Could not lock the global memory object").into())
		} else {
			Ok(data_ptr)
		}
	}

	/// Vertically flips the image pixels in memory
	fn flip_v(image: &DynamicImage) -> (i32, i32, Vec<u8>) {
		let w = image.width() as usize;
		let h = image.height() as usize;

		let mut bytes = to_bgr_bytes(image);

		let rowsize = w * 4; // each pixel is 4 bytes
		let mut tmp_a = vec![0; rowsize];
		// I believe this could be done safely with `as_chunks_mut`, but that's not stable yet
		for a_row_id in 0..(h / 2) {
			let b_row_id = h - a_row_id - 1;

			// swap rows `first_id` and `second_id`
			let a_byte_start = a_row_id * rowsize;
			let a_byte_end = a_byte_start + rowsize;
			let b_byte_start = b_row_id * rowsize;
			let b_byte_end = b_byte_start + rowsize;
			tmp_a.copy_from_slice(&bytes[a_byte_start..a_byte_end]);
			bytes.copy_within(b_byte_start..b_byte_end, a_byte_start);
			bytes[b_byte_start..b_byte_end].copy_from_slice(&tmp_a);
		}

		(h as i32, w as i32, bytes)
	}

	fn to_bgr_bytes(image: &DynamicImage) -> Vec<u8> {
		let mut byte_vec = Vec::with_capacity((image.width() * image.height() * 4) as usize);
		for (_, _, pixel) in image.pixels() {
			//Setting the pixels, one by one

			let pixel_bytes = pixel.0;
			//One pixel is 4 bytes, BGR and unused
			byte_vec.push(pixel_bytes[0]);
			byte_vec.push(pixel_bytes[1]);
			byte_vec.push(pixel_bytes[2]);
			byte_vec.push(pixel_bytes[3]); //This is unused based on the specifications
		}

		byte_vec
	}

	/// Converts the RGBA (u8) pixel data into the bitmap-native ARGB (u32)
	/// format in-place.
	///
	/// Safety: the `bytes` slice must have a length that's a multiple of 4
	#[allow(clippy::identity_op, clippy::erasing_op)]
	#[must_use]
	unsafe fn rgba_to_win(bytes: &mut [u8]) -> Cow<'_, [u8]> {
		// Check safety invariants to catch obvious bugs.
		debug_assert_eq!(bytes.len() % 4, 0);

		let mut u32pixels_buffer = convert_bytes_to_u32s(bytes);
		let u32pixels = match u32pixels_buffer {
			ImageDataCow::Borrowed(ref mut b) => b,
			ImageDataCow::Owned(ref mut b) => b.as_mut_slice(),
		};

		for p in u32pixels.iter_mut() {
			let [mut r, mut g, mut b, mut a] = p.to_ne_bytes().map(u32::from);
			r <<= 2 * 8;
			g <<= 1 * 8;
			b <<= 0 * 8;
			a <<= 3 * 8;

			*p = r | g | b | a;
		}

		match u32pixels_buffer {
			ImageDataCow::Borrowed(_) => Cow::Borrowed(bytes),
			ImageDataCow::Owned(bytes) => {
				Cow::Owned(bytes.into_iter().flat_map(|b| b.to_ne_bytes()).collect())
			}
		}
	}

	// XXX: std's Cow is not usable here because it does not allow mutably
	// borrowing data.
	enum ImageDataCow<'a> {
		Borrowed(&'a mut [u32]),
		Owned(Vec<u32>),
	}

	/// Safety: the `bytes` slice must have a length that's a multiple of 4
	unsafe fn convert_bytes_to_u32s(bytes: &mut [u8]) -> ImageDataCow<'_> {
		// When the correct conditions are upheld, `std` should return everything in the well-aligned slice.
		let (prefix, _, suffix) = bytes.align_to::<u32>();

		// Check if `align_to` gave us the optimal result.
		//
		// If it didn't, use the slow path with more allocations
		if prefix.is_empty() && suffix.is_empty() {
			// We know that the newly-aligned slice will contain all the values
			ImageDataCow::Borrowed(bytes.align_to_mut::<u32>().1)
		} else {
			// XXX: Use `as_chunks` when it stabilizes.
			let u32pixels_buffer = bytes
				.chunks(4)
				.map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
				.collect();
			ImageDataCow::Owned(u32pixels_buffer)
		}
	}
}
