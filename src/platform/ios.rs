use crate::{
	common::{Result, RustImage},
	Clipboard, ContentFormat, RustImageData,
};
use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSData, NSString};
use objc2_ui_kit::{UIImage, UIImagePNGRepresentation, UIPasteboard};

pub struct ClipboardContext {
	clipboard: Retained<UIPasteboard>,
}

impl ClipboardContext {
	pub fn new() -> Result<Self> {
		let clipboard = unsafe { UIPasteboard::generalPasteboard() };
		Ok(Self { clipboard })
	}
}

impl Clipboard for ClipboardContext {
	fn available_formats(&self) -> Result<Vec<String>> {
		let formats = unsafe { self.clipboard.pasteboardTypes() };
		Ok(formats.iter().map(|f| f.to_string()).collect())
	}

	fn has(&self, format: ContentFormat) -> bool {
		match format {
			ContentFormat::Text => unsafe { self.clipboard.hasStrings() },
			ContentFormat::Image => unsafe { self.clipboard.hasImages() },
			_ => false,
		}
	}

	fn clear(&self) -> Result<()> {
		unsafe { self.clipboard.setItems(&NSArray::from_slice(&[])) };
		Ok(())
	}

	fn get_buffer(&self, _format: &str) -> Result<Vec<u8>> {
		Err("Not supported".into())
	}

	fn get_text(&self) -> Result<String> {
		let text = unsafe { self.clipboard.string() };
		if let Some(text) = text {
			Ok(text.to_string())
		} else {
			Err("No text found".into())
		}
	}

	fn get_rich_text(&self) -> Result<String> {
		Err("Not supported".into())
	}

	fn get_html(&self) -> Result<String> {
		Err("Not supported".into())
	}

	fn get_image(&self) -> Result<RustImageData> {
		let image = unsafe { self.clipboard.image() };
		if let Some(image) = image {
			let data = unsafe { UIImagePNGRepresentation(&image) };
			if let Some(data) = data {
				let bytes = unsafe { data.as_bytes_unchecked() };
				Ok(RustImageData::from_bytes(bytes)?)
			} else {
				Err("No image data found".into())
			}
		} else {
			Err("No image found".into())
		}
	}

	fn get_files(&self) -> Result<Vec<String>> {
		Err("Not supported".into())
	}

	fn get(&self, _formats: &[crate::ContentFormat]) -> Result<Vec<crate::ClipboardContent>> {
		Err("Not supported".into())
	}

	fn set_buffer(&self, _format: &str, _buffer: Vec<u8>) -> Result<()> {
		Err("Not supported".into())
	}

	fn set_text(&self, text: String) -> Result<()> {
		unsafe {
			self.clipboard
				.setString(Some(&NSString::from_str(text.as_str())))
		};
		Ok(())
	}

	fn set_rich_text(&self, _text: String) -> Result<()> {
		Err("Not supported".into())
	}

	fn set_html(&self, _html: String) -> Result<()> {
		Err("Not supported".into())
	}

	fn set_image(&self, image: RustImageData) -> Result<()> {
		if image.is_empty() {
			Err("Image is empty".into())
		} else {
			let png = image.to_png()?;
			let ns_data = NSData::with_bytes(png.get_bytes());
			let image = unsafe { UIImage::imageWithData(&ns_data) };
			if let Some(image) = image {
				unsafe { self.clipboard.setImage(Some(&image)) };
				Ok(())
			} else {
				Err("Failed to create image".into())
			}
		}
	}

	fn set_files(&self, _files: Vec<String>) -> Result<()> {
		Err("Not supported".into())
	}

	fn set(&self, _contents: Vec<crate::ClipboardContent>) -> Result<()> {
		Err("Not supported".into())
	}
}
