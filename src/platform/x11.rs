use crate::{
	common::{Result, RustImage},
	ClipboardContent, ClipboardHandler, ContentFormat, RustImageData,
};
use crate::{Clipboard, ClipboardWatcher};
use std::sync::mpsc::{self, Receiver, Sender};
use std::{
	sync::{Arc, RwLock},
	thread,
	time::{Duration, Instant},
};
use x11rb::{
	connection::Connection,
	protocol::{
		xfixes,
		xproto::{
			Atom, AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, Property,
			SelectionNotifyEvent, SelectionRequestEvent, WindowClass, SELECTION_NOTIFY_EVENT,
		},
		Event,
	},
	rust_connection::RustConnection,
	wrapper::ConnectionExt as _,
	COPY_DEPTH_FROM_PARENT, CURRENT_TIME,
};

x11rb::atom_manager! {
	pub Atoms: AtomCookies {
		CLIPBOARD,
		CLIPBOARD_MANAGER,
		PROPERTY,
		SAVE_TARGETS,
		TARGETS,
		ATOM,
		INCR,
		TIMESTAMP,
		MULTIPLE,

		UTF8_STRING,
		UTF8_MIME_0: b"text/plain;charset=utf-8",
		UTF8_MIME_1: b"text/plain;charset=UTF-8",
		// Text in ISO Latin-1 encoding
		// See: https://tronche.com/gui/x/icccm/sec-2.html#s-2.6.2
		STRING,
		// Text in unknown encoding
		// See: https://tronche.com/gui/x/icccm/sec-2.html#s-2.6.2
		TEXT,
		TEXT_MIME_UNKNOWN: b"text/plain",
		// Rich Text Format
		RTF: b"text/rtf",
		RTF_1: b"text/richtext",
		HTML: b"text/html",
		PNG_MIME: b"image/png",
		FILE_LIST: b"text/uri-list",
		GNOME_COPY_FILES: b"x-special/gnome-copied-files",
		NAUTILUS_FILE_LIST: b"x-special/nautilus-clipboard",
	}
}

const FILE_PATH_PREFIX: &str = "file://";
pub struct ClipboardContext {
	inner: Arc<InnerContext>,
}

struct ClipboardData {
	format: Atom,
	data: Vec<u8>,
}

struct InnerContext {
	server: XServerContext,
	server_for_write: XServerContext,
	ignore_formats: Vec<Atom>,
	// 此刻待写入的剪贴板内容
	wait_write_data: RwLock<Vec<ClipboardData>>,
}

impl InnerContext {
	pub fn new() -> Result<Self> {
		let server = XServerContext::new()?;
		let server_for_write = XServerContext::new()?;
		let wait_write_data = RwLock::new(Vec::new());

		let ignore_formats = vec![
			server.atoms.TIMESTAMP,
			server.atoms.MULTIPLE,
			server.atoms.TARGETS,
			server.atoms.SAVE_TARGETS,
		];

		Ok(Self {
			server,
			server_for_write,
			ignore_formats,
			wait_write_data,
		})
	}

	pub fn handle_selection_request(&self, event: SelectionRequestEvent) -> Result<()> {
		let success;
		let ctx = &self.server_for_write;
		let atoms = ctx.atoms;
		// we are asked for a list of supported conversion targets
		if event.target == atoms.TARGETS {
			let reader = self.wait_write_data.read();
			match reader {
				Ok(data_list) => {
					let mut targets = Vec::with_capacity(10);
					targets.push(atoms.TARGETS);
					targets.push(atoms.SAVE_TARGETS);
					if data_list.len() > 0 {
						data_list.iter().for_each(|data| {
							targets.push(data.format);
						});
					}
					ctx.conn.change_property32(
						PropMode::REPLACE,
						event.requestor,
						event.property,
						AtomEnum::ATOM,
						&targets,
					)?;
					success = true;
				}
				Err(_) => return Err("Failed to read clipboard data".into()),
			}
		} else {
			let reader = self.wait_write_data.read();
			match reader {
				Ok(data_list) => {
					success = match data_list.iter().find(|d| d.format == event.target) {
						Some(data) => {
							ctx.conn.change_property8(
								PropMode::REPLACE,
								event.requestor,
								event.property,
								event.target,
								&data.data,
							)?;
							true
						}
						None => false,
					};
				}
				Err(_) => return Err("Failed to read clipboard data".into()),
			}
		}
		// on failure, we notify the requester of it
		let property = if success {
			event.property
		} else {
			AtomEnum::NONE.into()
		};
		// tell the requester that we finished sending data
		ctx.conn.send_event(
			false,
			event.requestor,
			EventMask::NO_EVENT,
			SelectionNotifyEvent {
				response_type: SELECTION_NOTIFY_EVENT,
				sequence: event.sequence,
				time: event.time,
				requestor: event.requestor,
				selection: event.selection,
				target: event.target,
				property,
			},
		)?;
		ctx.conn.flush()?;
		Ok(())
	}

	pub fn process_event(
		&self,
		buff: &mut Vec<u8>,
		selection: Atom,
		target: Atom,
		property: Atom,
		timeout: Option<Duration>,
		sequence_number: u64,
	) -> Result<()> {
		let mut is_incr = false;
		let start_time = if timeout.is_some() {
			Some(Instant::now())
		} else {
			None
		};
		let ctx = &self.server;
		let atoms = ctx.atoms;
		loop {
			if timeout
				.into_iter()
				.zip(start_time)
				.next()
				.map(|(timeout, time)| (Instant::now() - time) >= timeout)
				.unwrap_or(false)
			{
				return Err("Timeout while waiting for clipboard data".into());
			}

			let (event, seq) = match ctx.conn.poll_for_event_with_sequence()? {
				Some(event) => event,
				None => {
					thread::park_timeout(Duration::from_millis(50));
					continue;
				}
			};

			if seq < sequence_number {
				continue;
			}

			match event {
				Event::SelectionNotify(event) => {
					if event.selection != selection {
						continue;
					};

					let target_type = {
						if target == atoms.TARGETS {
							atoms.ATOM
						} else {
							target
						}
					};

					let reply = ctx
						.conn
						.get_property(
							false,
							event.requestor,
							event.property,
							target_type,
							buff.len() as u32,
							u32::MAX,
						)?
						.reply()?;

					if reply.type_ == atoms.INCR {
						if let Some(mut value) = reply.value32() {
							if let Some(size) = value.next() {
								buff.reserve(size as usize);
							}
						}
						ctx.conn.delete_property(ctx.win_id, property)?.check()?;
						is_incr = true;
						continue;
					} else if reply.type_ != target && reply.type_ != atoms.ATOM {
						return Err("Clipboard data type mismatch".into());
					}
					buff.extend_from_slice(&reply.value);
					break;
				}

				Event::PropertyNotify(event) if is_incr => {
					if event.state != Property::NEW_VALUE {
						continue;
					};

					let cookie =
						ctx.conn
							.get_property(false, ctx.win_id, property, AtomEnum::ATOM, 0, 0)?;

					let length = cookie.reply()?.bytes_after;

					let cookie = ctx.conn.get_property(
						true,
						ctx.win_id,
						property,
						AtomEnum::NONE,
						0,
						length,
					)?;
					let reply = cookie.reply()?;
					if reply.type_ != target {
						continue;
					};

					let value = reply.value;

					if !value.is_empty() {
						buff.extend_from_slice(&value);
					} else {
						break;
					}
				}
				_ => (),
			}
		}
		Ok(())
	}
}

impl ClipboardContext {
	pub fn new() -> Result<Self> {
		// build connection to X server
		let ctx = InnerContext::new()?;
		let ctx_arc = Arc::new(ctx);
		let ctx_clone = ctx_arc.clone();

		thread::spawn(move || {
			let res = process_server_req(&ctx_clone);
			if let Err(e) = res {
				println!("process_server_req error: {:?}", e);
			}
		});
		Ok(Self { inner: ctx_arc })
	}

	fn read(&self, format: &Atom) -> Result<Vec<u8>> {
		let ctx = &self.inner.server;
		let atoms = ctx.atoms;
		let clipboard = atoms.CLIPBOARD;
		let win_id = ctx.win_id;
		let cookie =
			ctx.conn
				.convert_selection(win_id, clipboard, *format, atoms.PROPERTY, CURRENT_TIME)?;
		let sequence_num = cookie.sequence_number();
		cookie.check()?;
		let mut buff = Vec::new();

		self.inner.process_event(
			&mut buff,
			clipboard,
			*format,
			atoms.PROPERTY,
			None,
			sequence_num,
		)?;

		ctx.conn.delete_property(win_id, atoms.PROPERTY)?.check()?;

		Ok(buff)
	}

	fn write(&self, data: Vec<ClipboardData>) -> Result<()> {
		let writer = self.inner.wait_write_data.write();
		match writer {
			Ok(mut writer) => {
				writer.clear();
				writer.extend(data);
			}
			Err(_) => return Err("Failed to write clipboard data".into()),
		}
		let ctx = &self.inner.server_for_write;
		let atoms = ctx.atoms;

		let win_id = ctx.win_id;
		let clipboard = atoms.CLIPBOARD;
		ctx.conn
			.set_selection_owner(win_id, clipboard, CURRENT_TIME)?
			.check()?;

		if ctx
			.conn
			.get_selection_owner(clipboard)?
			.reply()
			.map(|reply| reply.owner == win_id)
			.unwrap_or(false)
		{
			Ok(())
		} else {
			Err("Failed to take ownership of the clipboard".into())
		}
	}
}

fn process_server_req(context: &InnerContext) -> Result<()> {
	let atoms = context.server_for_write.atoms;
	loop {
		match context
			.server_for_write
			.conn
			.wait_for_event()
			.map_err(|e| format!("wait_for_event error: {:?}", e))?
		{
			Event::DestroyNotify(_) => {
				// This window is being destroyed.
				println!("Clipboard server window is being destroyed x_x");
				break;
			}
			Event::SelectionClear(event) => {
				// Someone else has new content in the clipboard, so it is
				// notifying us that we should delete our data now.
				println!("Somebody else owns the clipboard now");
				if event.selection == atoms.CLIPBOARD {
					// Clear the clipboard contents
					context
						.wait_write_data
						.write()
						.map(|mut writer| writer.clear())
						.map_err(|e| format!("write clipboard data error: {:?}", e))?;
				}
			}
			Event::SelectionRequest(event) => {
				// Someone is requesting the clipboard content from us.
				context
					.handle_selection_request(event)
					.map_err(|e| format!("handle_selection_request error: {:?}", e))?;
			}
			Event::SelectionNotify(event) => {
				// We've requested the clipboard content and this is the answer.
				// Considering that this thread is not responsible for reading
				// clipboard contents, this must come from the clipboard manager
				// signaling that the data was handed over successfully.
				if event.selection != atoms.CLIPBOARD_MANAGER {
					println!("Received a `SelectionNotify` from a selection other than the CLIPBOARD_MANAGER. This is unexpected in this thread.");
					continue;
				}
			}
			_event => {
				// May be useful for debugging but nothing else really.
				// trace!("Received unwanted event: {:?}", event);
			}
		}
	}
	Ok(())
}

impl Clipboard for ClipboardContext {
	fn available_formats(&self) -> Result<Vec<String>> {
		let ctx = &self.inner.server;
		let atoms = ctx.atoms;
		self.read(&atoms.TARGETS).map(|data| {
			let mut formats = Vec::new();
			// 解析原子标识符列表
			let atom_list: Vec<Atom> = parse_atom_list(&data);
			for atom in atom_list {
				if self.inner.ignore_formats.contains(&atom) {
					continue;
				}
				let atom_name = ctx.get_atom_name(atom).unwrap_or("Unknown".to_string());
				formats.push(atom_name);
			}
			formats
		})
	}

	fn has(&self, format: crate::ContentFormat) -> bool {
		let ctx = &self.inner.server;
		let atoms = ctx.atoms;
		let atom_list = self.read(&atoms.TARGETS).map(|data| parse_atom_list(&data));
		match atom_list {
			Ok(formats) => match format {
				ContentFormat::Text => formats.contains(&atoms.UTF8_STRING),
				ContentFormat::Rtf => formats.contains(&atoms.RTF),
				ContentFormat::Html => formats.contains(&atoms.HTML),
				ContentFormat::Image => formats.contains(&atoms.PNG_MIME),
				ContentFormat::Files => formats.contains(&atoms.FILE_LIST),
				ContentFormat::Other(format_name) => {
					let atom = ctx.get_atom(format_name.as_str());
					match atom {
						Ok(atom) => formats.contains(&atom),
						Err(_) => false,
					}
				}
			},
			Err(_) => false,
		}
	}

	fn clear(&self) -> Result<()> {
		self.write(vec![])
	}

	fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
		let atom = self.inner.server.get_atom(format);
		match atom {
			Ok(atom) => self.read(&atom),
			Err(_) => Err("Invalid format".into()),
		}
	}

	fn get_text(&self) -> Result<String> {
		let atoms = self.inner.server.atoms;
		let text_data = self.read(&atoms.UTF8_STRING);
		text_data.map_or_else(
			|_| Ok("".to_string()),
			|data| Ok(String::from_utf8_lossy(&data).to_string()),
		)
	}

	fn get_rich_text(&self) -> Result<String> {
		let atoms = self.inner.server.atoms;
		let rtf_data = self.read(&atoms.RTF);
		rtf_data.map_or_else(
			|_| Ok("".to_string()),
			|data| Ok(String::from_utf8_lossy(&data).to_string()),
		)
	}

	fn get_html(&self) -> Result<String> {
		let atoms = self.inner.server.atoms;
		let html_data = self.read(&atoms.HTML);
		html_data.map_or_else(
			|_| Ok("".to_string()),
			|data| Ok(String::from_utf8_lossy(&data).to_string()),
		)
	}

	fn get_image(&self) -> Result<crate::RustImageData> {
		let atoms = self.inner.server.atoms;
		let image_bytes = self.read(&atoms.PNG_MIME);
		match image_bytes {
			Ok(bytes) => {
				let image = RustImageData::from_bytes(&bytes);
				match image {
					Ok(image) => Ok(image),
					Err(_) => Err("Invalid image data".into()),
				}
			}
			Err(_) => Err("No image data found".into()),
		}
	}

	fn get_files(&self) -> Result<Vec<String>> {
		let atoms = self.inner.server.atoms;
		let file_list_data = self.read(&atoms.FILE_LIST);
		file_list_data.map_or_else(
			|_| Ok(vec![]),
			|data| {
				let file_list_str = String::from_utf8_lossy(&data).to_string();
				let mut list = Vec::new();
				for line in file_list_str.lines() {
					if !line.starts_with(FILE_PATH_PREFIX) {
						continue;
					}
					list.push(line.to_string())
				}
				Ok(list)
			},
		)
	}

	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		let mut contents = Vec::new();
		for format in formats {
			match format {
				ContentFormat::Text => match self.get_text() {
					Ok(text) => contents.push(ClipboardContent::Text(text)),
					Err(_) => continue,
				},
				ContentFormat::Rtf => match self.get_rich_text() {
					Ok(rtf) => contents.push(ClipboardContent::Rtf(rtf)),
					Err(_) => continue,
				},
				ContentFormat::Html => match self.get_html() {
					Ok(html) => contents.push(ClipboardContent::Html(html)),
					Err(_) => continue,
				},
				ContentFormat::Image => match self.get_image() {
					Ok(image) => contents.push(ClipboardContent::Image(image)),
					Err(_) => continue,
				},
				ContentFormat::Files => match self.get_files() {
					Ok(files) => contents.push(ClipboardContent::Files(files)),
					Err(_) => continue,
				},
				ContentFormat::Other(format_name) => match self.get_buffer(format_name) {
					Ok(buffer) => {
						contents.push(ClipboardContent::Other(format_name.clone(), buffer))
					}
					Err(_) => continue,
				},
			}
		}
		Ok(contents)
	}

	fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
		let atom = self.inner.server_for_write.get_atom(format)?;
		let data = ClipboardData {
			format: atom,
			data: buffer,
		};
		self.write(vec![data])
	}

	fn set_text(&self, text: String) -> Result<()> {
		let atoms = self.inner.server_for_write.atoms;
		let text_bytes = text.as_bytes().to_vec();

		let data = ClipboardData {
			format: atoms.UTF8_STRING,
			data: text_bytes,
		};
		self.write(vec![data])
	}

	fn set_rich_text(&self, text: String) -> Result<()> {
		let atoms = self.inner.server_for_write.atoms;
		let text_bytes = text.as_bytes().to_vec();

		let data = ClipboardData {
			format: atoms.RTF,
			data: text_bytes,
		};
		self.write(vec![data])
	}

	fn set_html(&self, html: String) -> Result<()> {
		let atoms = self.inner.server_for_write.atoms;
		let html_bytes = html.as_bytes().to_vec();

		let data = ClipboardData {
			format: atoms.HTML,
			data: html_bytes,
		};
		self.write(vec![data])
	}

	fn set_image(&self, image: RustImageData) -> Result<()> {
		let atoms = self.inner.server_for_write.atoms;
		let image_png = image.to_png()?;
		let data = ClipboardData {
			format: atoms.PNG_MIME,
			data: image_png.get_bytes().to_vec(),
		};
		self.write(vec![data])
	}

	fn set_files(&self, files: Vec<String>) -> Result<()> {
		let atoms = self.inner.server_for_write.atoms;
		let data = file_uri_list_to_clipboard_data(files, atoms);
		self.write(data)
	}

	fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
		let mut data = Vec::new();
		let atoms = self.inner.server_for_write.atoms;
		for content in contents {
			match content {
				ClipboardContent::Text(text) => {
					data.push(ClipboardData {
						format: atoms.UTF8_STRING,
						data: text.as_bytes().to_vec(),
					});
				}
				ClipboardContent::Rtf(rtf) => {
					data.push(ClipboardData {
						format: atoms.RTF,
						data: rtf.as_bytes().to_vec(),
					});
				}
				ClipboardContent::Html(html) => {
					data.push(ClipboardData {
						format: atoms.HTML,
						data: html.as_bytes().to_vec(),
					});
				}
				ClipboardContent::Image(image) => {
					let image_png = image.to_png()?;
					data.push(ClipboardData {
						format: atoms.PNG_MIME,
						data: image_png.get_bytes().to_vec(),
					});
				}
				ClipboardContent::Files(files) => {
					let data_arr = file_uri_list_to_clipboard_data(files, atoms);
					data.extend(data_arr);
				}
				ClipboardContent::Other(format_name, buffer) => {
					let atom = self.inner.server_for_write.get_atom(&format_name)?;
					data.push(ClipboardData {
						format: atom,
						data: buffer,
					});
				}
			}
		}
		self.write(data)
	}
}

pub struct ClipboardWatcherContext<T: ClipboardHandler> {
	handlers: Vec<T>,
	stop_signal: Sender<()>,
	stop_receiver: Receiver<()>,
}

impl<T: ClipboardHandler> ClipboardWatcherContext<T> {
	pub fn new() -> Result<Self> {
		let (tx, rx) = mpsc::channel();
		Ok(Self {
			handlers: Vec::new(),
			stop_signal: tx,
			stop_receiver: rx,
		})
	}
}

impl<T: ClipboardHandler> ClipboardWatcher<T> for ClipboardWatcherContext<T> {
	fn add_handler(&mut self, f: T) -> &mut Self {
		self.handlers.push(f);
		self
	}

	fn start_watch(&mut self) {
		let watch_server = XServerContext::new().expect("Failed to create X server context");
		let screen = watch_server
			.conn
			.setup()
			.roots
			.get(watch_server._screen)
			.expect("Failed to get screen");

		xfixes::query_version(&watch_server.conn, 5, 0)
			.expect("Failed to query version xfixes is not available");
		let cookie = xfixes::select_selection_input(
			&watch_server.conn,
			screen.root,
			watch_server.atoms.CLIPBOARD,
			xfixes::SelectionEventMask::SET_SELECTION_OWNER
				| xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE
				| xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY,
		)
		.expect("Failed to select selection input");

		cookie.check().unwrap();

		loop {
			if self
				.stop_receiver
				.recv_timeout(Duration::from_millis(500))
				.is_ok()
			{
				break;
			}
			let event = match watch_server
				.conn
				.poll_for_event()
				.expect("Failed to poll for event")
			{
				Some(event) => event,
				None => {
					continue;
				}
			};
			if let Event::XfixesSelectionNotify(_) = event {
				self.handlers
					.iter_mut()
					.for_each(|handler| handler.on_clipboard_change());
			}
		}
	}

	fn get_shutdown_channel(&self) -> WatcherShutdown {
		WatcherShutdown {
			sender: self.stop_signal.clone(),
		}
	}
}

pub struct WatcherShutdown {
	sender: Sender<()>,
}

impl Drop for WatcherShutdown {
	fn drop(&mut self) {
		let _ = self.sender.send(());
	}
}

struct XServerContext {
	conn: RustConnection,
	win_id: u32,
	_screen: usize,
	atoms: Atoms,
}

impl XServerContext {
	fn new() -> Result<Self> {
		let (conn, screen) = x11rb::connect(None)?;
		let win_id = conn.generate_id()?;
		{
			let screen = conn.setup().roots.get(screen).unwrap();
			conn.create_window(
				COPY_DEPTH_FROM_PARENT,
				win_id,
				screen.root,
				0,
				0,
				1,
				1,
				0,
				WindowClass::INPUT_OUTPUT,
				screen.root_visual,
				&CreateWindowAux::new()
					.event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE),
			)?
			.check()?;
		}
		let atoms = Atoms::new(&conn)?.reply()?;
		Ok(Self {
			conn,
			win_id,
			_screen: screen,
			atoms,
		})
	}

	fn get_atom(&self, format: &str) -> Result<Atom> {
		let cookie = self.conn.intern_atom(false, format.as_bytes())?;
		Ok(cookie.reply()?.atom)
	}

	fn get_atom_name(&self, atom: Atom) -> Result<String> {
		let cookie = self.conn.get_atom_name(atom)?;
		Ok(String::from_utf8_lossy(&cookie.reply()?.name).to_string())
	}
}

// 解析原子标识符列表
fn parse_atom_list(data: &[u8]) -> Vec<Atom> {
	data.chunks(4)
		.map(|chunk| {
			let mut bytes = [0u8; 4];
			bytes.copy_from_slice(chunk);
			u32::from_ne_bytes(bytes)
		})
		.collect()
}

fn file_uri_list_to_clipboard_data(file_list: Vec<String>, atoms: Atoms) -> Vec<ClipboardData> {
	let uri_list: Vec<String> = file_list
		.iter()
		.map(|f| {
			if f.starts_with(FILE_PATH_PREFIX) {
				f.to_owned()
			} else {
				format!("{}{}", FILE_PATH_PREFIX, f)
			}
		})
		.collect();
	let uri_list = uri_list.join("\n");
	let text_uri_list_data = uri_list.as_bytes().to_vec();
	let gnome_copied_files_data = ["copy\n".as_bytes(), uri_list.as_bytes()].concat();

	vec![
		ClipboardData {
			format: atoms.FILE_LIST,
			data: text_uri_list_data,
		},
		ClipboardData {
			format: atoms.GNOME_COPY_FILES,
			data: gnome_copied_files_data.clone(),
		},
		ClipboardData {
			format: atoms.NAUTILUS_FILE_LIST,
			data: gnome_copied_files_data,
		},
	]
}
