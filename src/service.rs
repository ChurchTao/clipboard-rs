use crate::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat, Result};

/// High-level clipboard facade. Encapsulates the platform context and exposes
/// focused methods for common clipboard workflows.
pub struct ClipboardService {
	ctx: ClipboardContext,
}

impl ClipboardService {
	pub fn new() -> Result<Self> {
		Ok(Self {
			ctx: ClipboardContext::new()?,
		})
	}

	pub fn with_context(ctx: ClipboardContext) -> Self {
		Self { ctx }
	}

	pub fn formats(&self) -> Result<Vec<String>> {
		self.ctx.available_formats()
	}

	pub fn get_text(&self) -> Result<String> {
		self.ctx.get_text()
	}

	pub fn set_text(&self, text: impl Into<String>) -> Result<()> {
		self.ctx.set_text(text.into())
	}

	pub fn get_html(&self) -> Result<String> {
		self.ctx.get_html()
	}

	pub fn set_html(&self, html: impl Into<String>) -> Result<()> {
		self.ctx.set_html(html.into())
	}

	pub fn get_rich_text(&self) -> Result<String> {
		self.ctx.get_rich_text()
	}

	pub fn set_rich_text(&self, rich_text: impl Into<String>) -> Result<()> {
		self.ctx.set_rich_text(rich_text.into())
	}

	pub fn get_files(&self) -> Result<Vec<String>> {
		self.ctx.get_files()
	}

	pub fn set_files(&self, files: Vec<String>) -> Result<()> {
		self.ctx.set_files(files)
	}

	pub fn has(&self, format: ContentFormat) -> bool {
		self.ctx.has(format)
	}

	pub fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		self.ctx.get(formats)
	}

	pub fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
		self.ctx.set(contents)
	}

	pub fn clear(&self) -> Result<()> {
		self.ctx.clear()
	}

	#[cfg(feature = "image")]
	pub fn get_image(&self) -> Result<crate::RustImageData> {
		self.ctx.get_image()
	}

	#[cfg(feature = "image")]
	pub fn set_image(&self, image: crate::RustImageData) -> Result<()> {
		self.ctx.set_image(image)
	}
}
