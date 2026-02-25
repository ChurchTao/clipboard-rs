#![cfg(feature = "async")]

use std::sync::{Arc, Mutex};

use crate::error::ClipboardError;
use crate::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat, Result};

/// Async clipboard facade built on top of blocking platform APIs.
///
/// Internally, each operation runs in `tokio::task::spawn_blocking` to avoid
/// blocking async executors.
#[derive(Clone)]
pub struct AsyncClipboardService {
	ctx: Arc<Mutex<ClipboardContext>>,
}

impl AsyncClipboardService {
	pub fn new() -> Result<Self> {
		Ok(Self {
			ctx: Arc::new(Mutex::new(ClipboardContext::new()?)),
		})
	}

	pub fn with_context(ctx: ClipboardContext) -> Self {
		Self {
			ctx: Arc::new(Mutex::new(ctx)),
		}
	}

	pub async fn formats(&self) -> Result<Vec<String>> {
		self.blocking_call(|ctx| ctx.available_formats()).await
	}

	pub async fn get_text(&self) -> Result<String> {
		self.blocking_call(|ctx| ctx.get_text()).await
	}

	pub async fn set_text(&self, text: impl Into<String> + Send + 'static) -> Result<()> {
		let text = text.into();
		self.blocking_call(move |ctx| ctx.set_text(text)).await
	}

	pub async fn get_html(&self) -> Result<String> {
		self.blocking_call(|ctx| ctx.get_html()).await
	}

	pub async fn set_html(&self, html: impl Into<String> + Send + 'static) -> Result<()> {
		let html = html.into();
		self.blocking_call(move |ctx| ctx.set_html(html)).await
	}

	pub async fn get_rich_text(&self) -> Result<String> {
		self.blocking_call(|ctx| ctx.get_rich_text()).await
	}

	pub async fn set_rich_text(&self, rich_text: impl Into<String> + Send + 'static) -> Result<()> {
		let rich_text = rich_text.into();
		self.blocking_call(move |ctx| ctx.set_rich_text(rich_text))
			.await
	}

	pub async fn get_files(&self) -> Result<Vec<String>> {
		self.blocking_call(|ctx| ctx.get_files()).await
	}

	pub async fn set_files(&self, files: Vec<String>) -> Result<()> {
		self.blocking_call(move |ctx| ctx.set_files(files)).await
	}

	pub async fn has(&self, format: ContentFormat) -> Result<bool> {
		self.blocking_call(move |ctx| Ok(ctx.has(format))).await
	}

	pub async fn get(&self, formats: Vec<ContentFormat>) -> Result<Vec<ClipboardContent>> {
		self.blocking_call(move |ctx| ctx.get(formats.as_slice()))
			.await
	}

	pub async fn set(&self, contents: Vec<ClipboardContent>) -> Result<()> {
		self.blocking_call(move |ctx| ctx.set(contents)).await
	}

	pub async fn clear(&self) -> Result<()> {
		self.blocking_call(|ctx| ctx.clear()).await
	}

	#[cfg(feature = "image")]
	pub async fn get_image(&self) -> Result<crate::RustImageData> {
		self.blocking_call(|ctx| ctx.get_image()).await
	}

	#[cfg(feature = "image")]
	pub async fn set_image(&self, image: crate::RustImageData) -> Result<()> {
		self.blocking_call(move |ctx| ctx.set_image(image)).await
	}

	async fn blocking_call<T, F>(&self, f: F) -> Result<T>
	where
		T: Send + 'static,
		F: FnOnce(&ClipboardContext) -> Result<T> + Send + 'static,
	{
		let ctx = Arc::clone(&self.ctx);
		tokio::task::spawn_blocking(move || {
			let guard = ctx
				.lock()
				.map_err(|_| ClipboardError::Message("clipboard mutex poisoned".to_string()))?;
			f(&guard)
		})
		.await
		.map_err(|e| ClipboardError::TaskJoin(e.to_string()))?
	}
}
