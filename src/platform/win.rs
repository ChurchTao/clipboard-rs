use crate::common::{ContentData, Result, RustImage, RustImageData};
use crate::{Clipboard, ClipboardContent, ClipboardHandler, ClipboardWatcher, ContentFormat};
use clipboard_win::formats::{CF_DIB, CF_DIBV5};
use clipboard_win::raw::{set_file_list, set_file_list_with, set_string_with, set_without_clear};
use clipboard_win::types::c_uint;
use clipboard_win::{
	formats, get, get_clipboard, options, raw, set_clipboard, Clipboard as ClipboardWin, Monitor,
	Setter, SysResult,
};
use image::codecs::bmp::BmpDecoder;
use image::{DynamicImage, EncodableLayout};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

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
		Ok(String::from_utf8(rtf_raw_data)?)
	}

	fn get_html(&self) -> Result<String> {
		let res: SysResult<String> = get_clipboard(self.html_format);
		match res {
			Ok(html) => Ok(html),
			Err(e) => Err(format!("Get html error, code = {}", e).into()),
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
							let rtf = String::from_utf8(buffer)?;
							res.push(ClipboardContent::Rtf(rtf));
						}
						Err(_) => continue,
					}
				}
				ContentFormat::Html => {
					let html_res: SysResult<String> = get(self.html_format);
					match html_res {
						Ok(html) => {
							res.push(ClipboardContent::Html(html));
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
			let write_png_res = raw::set_without_clear(*cf_png_format.unwrap(), png.get_bytes());
			if let Err(e) = write_png_res {
				return Err(format!("set png image error, code = {}", e).into());
			}
		}
		let bmp = image.to_bitmap()?;
		let bytes = bmp.get_bytes();
		let no_file_header_bytes = &bytes[14..];
		// ignore error of write_dib_res
		// some applications may not support CF_DIBV5 format,so we also set CF_DIB format. e.g. Feishu
		let _write_dib_res = raw::set_without_clear(CF_DIB, no_file_header_bytes);
		let res = raw::set_without_clear(CF_DIBV5, no_file_header_bytes);
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
fn utf8_to_utf16(input: &str) -> Vec<u16> {
	let mut vec: Vec<u16> = input.encode_utf16().collect();
	vec.push(0);
	vec
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
