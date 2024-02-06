use std::{thread, time::{Duration, Instant}};

use crate::{common::{Result, RustImage}, ContentFormat, RustImageData};
use crate::Clipboard;
use x11rb::{
	connection::Connection, protocol::{
		xproto::{
			Atom, AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, Property,
			PropertyNotifyEvent, SelectionNotifyEvent, SelectionRequestEvent, Time, WindowClass,
			SELECTION_NOTIFY_EVENT,
		},
		Event,
	}, rust_connection::RustConnection, wrapper::ConnectionExt as _, COPY_DEPTH_FROM_PARENT, COPY_FROM_PARENT, CURRENT_TIME, NONE
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
	}
}

pub struct ClipboardContext {
	server: XServerContext,
	atoms: Atoms,
    ignore_formats: Vec<Atom>,
}

impl ClipboardContext {
    pub fn new() -> Result<Self> {
        let server = XServerContext::new()?;
        let atoms = Atoms::new(&server.conn)?.reply()?;
        Ok(Self {
            server,
            atoms,
            ignore_formats: vec![atoms.TIMESTAMP,atoms.MULTIPLE,atoms.TARGETS,atoms.SAVE_TARGETS],
        })
    }

    fn read(&self, format: &Atom) -> Result<Vec<u8>> {
        let clipboard = self.atoms.CLIPBOARD;
        let win_id = self.server.win_id;
        let cookie = self.server.conn.convert_selection(
            win_id, 
            clipboard,
            *format,
            self.atoms.PROPERTY,
            CURRENT_TIME
        )?;
        let sequence_num = cookie.sequence_number();
        cookie.check()?;
        let mut buff = Vec::new();

        self.process_event(
            &mut buff,
            clipboard,
            *format,
            self.atoms.PROPERTY,
            None,
            sequence_num
        )?;

        self.server.conn.delete_property(
            win_id,
            self.atoms.PROPERTY
        )?.check()?;

        Ok(buff)
    }

    fn process_event(&self, buff: &mut Vec<u8>, selection: Atom, target: Atom, property: Atom, timeout: Option<Duration>, sequence_number: u64)
        -> Result<()>
    {
        let mut is_incr = false;
        let start_time =
            if timeout.is_some() { Some(Instant::now()) }
            else { None };

        loop {
            if timeout.into_iter()
                .zip(start_time)
                .next()
                .map(|(timeout, time)| (Instant::now() - time) >= timeout)
                .unwrap_or(false)
            {
                return Err("Timeout while waiting for clipboard data".into());
            }

            let (event, seq) =match self.server.conn.poll_for_event_with_sequence()? {
                        Some(event) => event,
                        None => {
                            thread::park_timeout(Duration::from_millis(50));
                            continue
                        }
                    };

            if seq < sequence_number {
                continue;
            }

            match event {
                Event::SelectionNotify(event) => {
                    if event.selection != selection { continue };

                    let target_type = {
                        if target == self.atoms.TARGETS {
                            self.atoms.ATOM
                        } else {
                            target
                        }
                    };

                    let reply = self.server.conn.get_property(
                        false,
                        event.requestor,
                        event.property,
                        target_type,
                        buff.len() as u32,
                        u32::MAX
                    )?.reply()?;

                    if reply.type_ == self.atoms.INCR {
                        if let Some(mut value) = reply.value32() {
                            if let Some(size) = value.next() {
                                buff.reserve(size as usize);
                            }
                        }
                        self.server.conn.delete_property(
                            self.server.win_id,
                            property
                        )?.check()?;
                        is_incr = true;
                        continue
                    } else if reply.type_ != target && reply.type_ != self.atoms.ATOM{
                        return Err("Clipboard data type mismatch".into());
                    }
                    buff.extend_from_slice(&reply.value);
                    break
                }

                Event::PropertyNotify(event) if is_incr => {
                    if event.state != Property::NEW_VALUE { continue };


                    let cookie = self.server.conn.get_property(
                        false,
                        self.server.win_id,
                        property,
                        AtomEnum::ATOM,
                        0,
                        0
                    )?;

                    let length = cookie.reply()?.bytes_after;

                    let cookie = self.server.conn.get_property(
                        true,
                        self.server.win_id,
                        property,
                        AtomEnum::NONE,
                        0, length
                    )?;
                    let reply = cookie.reply()?;
                    if reply.type_ != target { continue };

                    let value = reply.value;

                    if !value.is_empty() {
                        buff.extend_from_slice(&value);
                    } else {
                        break
                    }
                },
                _ => ()
            }
        }
        Ok(())
    }
    
    fn get_atom(&self, format: &str) -> Result<Atom> {
        let cookie = self.server.conn.intern_atom(false, format.as_bytes())?;  
        Ok(cookie.reply()?.atom)
    }
}

impl Clipboard for ClipboardContext {

    fn available_formats(&self) -> Result<Vec<String>> {
        self.read(&self.atoms.TARGETS).map(|data| {
            let mut formats = Vec::new();
                // 解析原子标识符列表
                let atom_list: Vec<Atom> = data.chunks(4).map(|chunk| {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(chunk);
                    u32::from_ne_bytes(bytes)
                }).collect();
                for atom in atom_list {
                    if self.ignore_formats.contains(&atom) {
                        continue;
                    }
                    let atom_name = self.server.conn.get_atom_name(atom).unwrap().reply().unwrap().name;
                    formats.push(String::from_utf8_lossy(&atom_name).to_string());
                }  
            formats
        })
    }

    fn has(&self, format: crate::ContentFormat) -> bool {
        let atom_list = self.read(&self.atoms.TARGETS).map(|data| {
            let atom_list: Vec<Atom> = data.chunks(4).map(|chunk| {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(chunk);
                u32::from_ne_bytes(bytes)
            }).collect();
            atom_list
        });
        match atom_list {
            Ok(formats) => {
                match format {
                    ContentFormat::Text => formats.contains(&self.atoms.UTF8_STRING),
                    ContentFormat::Rtf => formats.contains(&self.atoms.RTF),
                    ContentFormat::Html => formats.contains(&self.atoms.HTML),
                    ContentFormat::Image => formats.contains(&self.atoms.PNG_MIME),
                    ContentFormat::Other(format_name) => {
                        let atom = self.get_atom(format_name);
                        match atom {
                            Ok(atom) => formats.contains(&atom),
                            Err(_) => false
                        }
                    }
                }
            },
            Err(_) => false
        }
    }

    fn clear(&self) -> Result<()> {
        todo!()
    }

    fn get_buffer(&self, format: &str) -> Result<Vec<u8>> {
        let atom = self.get_atom(format);
        match atom {
            Ok(atom) => self.read(&atom),
            Err(_) => Err("Invalid format".into())
        }
    }

    fn get_text(&self) -> Result<String> {
        let text_data = self.read(&self.atoms.UTF8_STRING);
        match text_data {
            Ok(data) => {
                let text = String::from_utf8_lossy(&data).to_string();
                Ok(text)
            },
            Err(_) => Ok("".to_string())
        }
    }

    fn get_rich_text(&self) -> Result<String> {
        let rtf_data = self.read(&self.atoms.RTF);
        match rtf_data {
            Ok(data) => {
                let rtf = String::from_utf8_lossy(&data).to_string();
                Ok(rtf)
            },
            Err(_) => Ok("".to_string())
        }
    }

    fn get_html(&self) -> Result<String> {
        let html_data = self.read(&self.atoms.HTML);
        match html_data {
            Ok(data) => {
                let html = String::from_utf8_lossy(&data).to_string();
                Ok(html)
            },
            Err(_) => Ok("".to_string())
        }
    }

    fn get_image(&self) -> Result<crate::RustImageData> {
        let image_bytes = self.read(&self.atoms.PNG_MIME);
        match image_bytes {
            Ok(bytes) => {
                let image = RustImageData::from_bytes(&bytes);
                match image {
                    Ok(image) => Ok(image),
                    Err(_) => Err("Invalid image data".into())
                }
            },
            Err(_) => Err("No image data found".into())
        }
    }

    fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()> {
        todo!()
    }

    fn set_text(&self, text: String) -> Result<()> {
        todo!()
    }

    fn set_rich_text(&self, text: String) -> Result<()> {
        todo!()
    }

    fn set_html(&self, html: String) -> Result<()> {
        todo!()
    }

    fn set_image(&self, image: crate::RustImageData) -> Result<()> {
        todo!()
    }
}

pub struct ClipboardWatcherContext {}

pub struct WatcherShutdown {}

struct XServerContext {
	conn: RustConnection,
	win_id: u32,
    _screen: usize,
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
                    .event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE)
            )?.check()?;
        }
        Ok(Self {
            conn,
            win_id,
            _screen: screen
        })
    }
}