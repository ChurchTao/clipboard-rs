pub mod common;
mod platform;
#[cfg(feature = "image")]
pub use common::RustImageData;
#[cfg(feature = "async-image")]
pub use common::ClipboardImage;
use crate::common::{ClipboardContent, ClipboardContentBuilder, ClipboardHandler, ContentFormat, Result};
#[cfg(feature = "image")]
pub use image::imageops::FilterType;
#[cfg(target_os = "linux")]
pub use platform::ClipboardContextX11Options;
pub use platform::{ClipboardContext, ClipboardWatcherContext, WatcherShutdown};

// 重新导出 async_trait 以便使用者可以直接使用
pub use async_trait;

#[cfg(feature = "async-image")]
use tokio::sync::mpsc;

/// 高级别的剪贴板管理器，提供简化的 API
pub struct ClipboardManager {
	inner: Box<dyn AsyncClipboard>,
}

impl ClipboardManager {
	/// 创建新的剪贴板管理器
	pub async fn new() -> Result<Self> {
		let ctx = ClipboardContext::new()?;
		Ok(Self {
			inner: Box::new(ctx),
		})
	}

	/// 获取剪贴板中所有可用的格式
	pub async fn available_formats(&self) -> Result<Vec<String>> {
		self.inner.available_formats().await
	}

	/// 检查剪贴板是否包含特定格式的内容
	pub async fn has(&self, format: ContentFormat) -> Result<bool> {
		self.inner.has(format).await
	}

	/// 清空剪贴板
	pub async fn clear(&self) -> Result<()> {
		self.inner.clear().await
	}

	/// 获取纯文本内容
	pub async fn get_text(&self) -> Result<String> {
		self.inner.get_text().await
	}

	/// 设置纯文本内容
	pub async fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
		self.inner.set_text(text.as_ref()).await
	}

	/// 获取 HTML 内容
	pub async fn get_html(&self) -> Result<String> {
		self.inner.get_html().await
	}

	/// 设置 HTML 内容
	pub async fn set_html(&self, html: impl AsRef<str>) -> Result<()> {
		self.inner.set_html(html.as_ref()).await
	}

	/// 获取 RTF 内容
	pub async fn get_rtf(&self) -> Result<String> {
		self.inner.get_rtf().await
	}

	/// 设置 RTF 内容
	pub async fn set_rtf(&self, rtf: impl AsRef<str>) -> Result<()> {
		self.inner.set_rtf(rtf.as_ref()).await
	}

	/// 获取文件列表
	pub async fn get_files(&self) -> Result<Vec<String>> {
		self.inner.get_files().await
	}

	/// 设置文件列表
	pub async fn set_files(&self, files: &[impl AsRef<str>]) -> Result<()> {
		let string_files: Vec<String> = files.iter().map(|f| f.as_ref().to_string()).collect();
		self.inner.set_files(&string_files).await
	}

	/// 创建剪贴板内容构建器
	pub fn build_content(&self) -> ClipboardContentBuilder {
		ClipboardContentBuilder::new()
	}

	/// 使用构建器设置多种内容
	pub async fn set_with_builder(&self, builder: ClipboardContentBuilder) -> Result<()> {
		self.inner.set_with_builder(builder).await
	}

	/// 获取原始数据
	pub async fn get_raw(&self, format: &str) -> Result<Vec<u8>> {
		self.inner.get_raw(format).await
	}

	/// 设置原始数据
	pub async fn set_raw(&self, format: &str, data: &[u8]) -> Result<()> {
		self.inner.set_raw(format, data).await
	}

	/// 获取图像内容
	#[cfg(feature = "async-image")]
	pub async fn get_image(&self) -> Result<ClipboardImage> {
		let image_data = self.inner.get_image().await?;
		Ok(ClipboardImage::from_dynamic_image(image_data.get_dynamic_image()?))
	}

	/// 设置图像内容
	#[cfg(feature = "async-image")]
	pub async fn set_image(&self, image: ClipboardImage) -> Result<()> {
		let image_data = RustImageData::from_dynamic_image(image.get_dynamic_image().clone());
		self.inner.set_image(image_data).await
	}
}

#[cfg(feature = "async-image")]
#[async_trait::async_trait]
impl AsyncClipboardWatcher for ClipboardManager {
	async fn watch(&self) -> Result<ClipboardEventStream> {
		// 创建一个通道用于发送剪贴板事件
		let (_sender, receiver) = mpsc::channel(100);

		// 创建事件流
		let event_stream = ClipboardEventStream { receiver };

		// TODO: 实现实际的监视逻辑
		// 这里需要在后台任务中实现剪贴板监视

		Ok(event_stream)
	}

	async fn add_handler<F>(&self, _handler: F) -> Result<()>
	where
		F: Fn(ClipboardEvent) + Send + Sync + 'static,
	{
		// TODO: 实现事件处理器的添加逻辑
		Ok(())
	}
}

pub trait Clipboard: Send {
	/// zh: 获得剪切板当前内容的所有格式
	/// en: Get all formats of the current content in the clipboard
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.available_formats().await")]
	fn available_formats(&self) -> Result<Vec<String>>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.has(format).await")]
	fn has(&self, format: ContentFormat) -> bool;

	/// zh: 清空剪切板
	/// en: clear clipboard
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.clear().await")]
	fn clear(&self) -> Result<()>;

	/// zh: 获得指定格式的数据，以字节数组形式返回
	/// en: Get the data in the specified format in the clipboard as a byte array
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_buffer(format).await")]
	fn get_buffer(&self, format: &str) -> Result<Vec<u8>>;

	/// zh: 仅获得无格式纯文本，以字符串形式返回
	/// en: Get plain text content in the clipboard as string
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_text().await")]
	fn get_text(&self) -> Result<String>;

	/// zh: 获得剪贴板中的富文本内容，以字符串形式返回
	/// en: Get the rich text content in the clipboard as string
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_rtf().await")]
	fn get_rich_text(&self) -> Result<String>;

	/// zh: 获得剪贴板中的html内容，以字符串形式返回
	/// en: Get the html format content in the clipboard as string
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_html().await")]
	fn get_html(&self) -> Result<String>;

	#[cfg(feature = "image")]
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_image().await")]
	fn get_image(&self) -> Result<RustImageData>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get_files().await")]
	fn get_files(&self) -> Result<Vec<String>>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.get(formats).await")]
	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_buffer(format, buffer).await")]
	fn set_buffer(&self, format: &str, buffer: Vec<u8>) -> Result<()>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_text(text).await")]
	fn set_text(&self, text: String) -> Result<()>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_rtf(text).await")]
	fn set_rich_text(&self, text: String) -> Result<()>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_html(html).await")]
	fn set_html(&self, html: String) -> Result<()>;

	#[cfg(feature = "image")]
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_image(image).await")]
	fn set_image(&self, image: RustImageData) -> Result<()>;

	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set_files(files).await")]
	fn set_files(&self, files: Vec<String>) -> Result<()>;

	/// set image will clear clipboard
	#[deprecated(since = "0.4.0", note = "请使用新的异步API: clipboard.set(contents).await")]
	fn set(&self, contents: Vec<ClipboardContent>) -> Result<()>;
}

/// 现代化的异步 Clipboard trait，提供更现代化的 API
#[async_trait::async_trait]
pub trait AsyncClipboard: Send + Sync {
	/// 获取剪贴板中所有可用的格式
	async fn available_formats(&self) -> Result<Vec<String>>;

	/// 检查剪贴板是否包含特定格式的内容
	async fn has(&self, format: ContentFormat) -> Result<bool>;

	/// 清空剪贴板
	async fn clear(&self) -> Result<()>;

	/// 获取指定格式的原始数据
	async fn get_raw(&self, format: &str) -> Result<Vec<u8>>;

	/// 获取纯文本内容
	async fn get_text(&self) -> Result<String>;

	/// 获取富文本内容（RTF）
	async fn get_rtf(&self) -> Result<String>;

	/// 获取 HTML 内容
	async fn get_html(&self) -> Result<String>;

	/// 获取图像内容
	#[cfg(feature = "image")]
	async fn get_image(&self) -> Result<RustImageData>;

	/// 获取文件列表
	async fn get_files(&self) -> Result<Vec<String>>;

	/// 获取多种格式的内容
	async fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>>;

	/// 设置原始数据
	async fn set_raw(&self, format: &str, data: &[u8]) -> Result<()>;

	/// 设置纯文本内容
	async fn set_text(&self, text: &str) -> Result<()>;

	/// 设置富文本内容
	async fn set_rtf(&self, rtf: &str) -> Result<()>;

	/// 设置 HTML 内容
	async fn set_html(&self, html: &str) -> Result<()>;

	/// 设置图像内容
	#[cfg(feature = "image")]
	async fn set_image(&self, image: RustImageData) -> Result<()>;

	/// 设置文件列表
	async fn set_files(&self, files: &[String]) -> Result<()>;

	/// 设置多种内容
	async fn set(&self, contents: Vec<ClipboardContent>) -> Result<()>;

	/// 使用构建器设置多种内容
	async fn set_with_builder(&self, builder: ClipboardContentBuilder) -> Result<()> {
		self.set(builder.build()).await
	}
}

pub trait ClipboardWatcher<T: ClipboardHandler>: Send {
	/// zh: 添加一个剪切板变化处理器，可以添加多个处理器，处理器需要实现 [`ClipboardHandler`] 这个trait
	/// en: Add a clipboard change handler, you can add multiple handlers, the handler needs to implement the trait [`ClipboardHandler`]
	#[deprecated(since = "0.4.0", note = "请使用新的异步API，未来版本将提供更好的异步监听方案")]
	fn add_handler(&mut self, handler: T) -> &mut Self;

	/// zh: 开始监视剪切板变化，这是一个阻塞方法，直到监视结束，或者调用了stop方法，所以建议在单独的线程中调用
	/// en: Start monitoring clipboard changes, this is a blocking method, until the monitoring ends, or the stop method is called, so it is recommended to call it in a separate thread
	#[deprecated(since = "0.4.0", note = "请使用新的异步API，未来版本将提供更好的异步监听方案")]
	fn start_watch(&mut self);

	/// zh: 获得停止监视的通道，可以通过这个通道停止监视
	/// en: Get the channel to stop monitoring, you can stop monitoring through this channel
	#[deprecated(since = "0.4.0", note = "请使用新的异步API，未来版本将提供更好的异步监听方案")]
	fn get_shutdown_channel(&self) -> WatcherShutdown;
}

/// 剪贴板事件流
#[cfg(feature = "async-image")]
pub struct ClipboardEventStream {
	receiver: mpsc::Receiver<ClipboardEvent>,
}

#[cfg(feature = "async-image")]
impl ClipboardEventStream {
	pub async fn next(&mut self) -> Option<ClipboardEvent> {
		self.receiver.recv().await
	}
}

/// 现代化的异步 ClipboardWatcher trait，提供更现代化的 API
#[cfg(feature = "async-image")]
#[async_trait::async_trait]
pub trait AsyncClipboardWatcher: Send + Sync {
	/// 启动监视器并返回一个事件流
	async fn watch(&self) -> Result<ClipboardEventStream>;

	/// 添加事件处理器
	async fn add_handler<F>(&self, handler: F) -> Result<()>
	where
		F: Fn(ClipboardEvent) + Send + Sync + 'static;
}


/// 剪贴板事件
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
	/// 剪贴板内容已更改
	Changed {
		formats: Vec<ContentFormat>,
	},
	/// 剪贴板已清空
	Cleared,
	/// 错误事件
	Error(String),
}

impl WatcherShutdown {
	/// zh: 停止监视
	///
	/// en: stop watching
	pub fn stop(self) {
		drop(self);
	}
}
