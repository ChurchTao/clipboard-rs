pub mod common;
mod platform;
pub use crate::common::{
	ClipboardContent, ClipboardContentBuilder, ClipboardHandler, ContentFormat, Result,
};
#[cfg(feature = "image")]
pub use common::ClipboardImage;
#[cfg(target_os = "linux")]
pub use platform::ClipboardContextX11Options;
pub use platform::{start_async_watch, ClipboardContext};

/// 剪贴板事件
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub enum ClipboardEvent {
	/// 剪贴板内容已更改
	Changed { formats: Vec<ContentFormat> },
	/// 剪贴板已清空
	Cleared,
	/// 错误事件
	Error(String),
}

/// 剪贴板事件流
#[cfg(feature = "async")]
pub struct ClipboardEventStream {
	receiver: mpsc::Receiver<ClipboardEvent>,
	/// 停止信号发送器，当 drop 时会自动关闭通道
	_shutdown_tx: tokio::sync::watch::Sender<bool>,
}

#[cfg(feature = "async")]
impl ClipboardEventStream {
	/// 接收下一个剪贴板事件
	pub async fn next(&mut self) -> Option<ClipboardEvent> {
		self.receiver.recv().await
	}

	/// 主动停止监听器
	pub fn stop(&self) {
		let _ = self._shutdown_tx.send(true);
	}
}

#[cfg(feature = "async")]
impl Drop for ClipboardEventStream {
	fn drop(&mut self) {
		// 发送停止信号，监听循环会收到信号并退出
		let _ = self._shutdown_tx.send(true);
	}
}

// 重新导出 async_trait 以便使用者可以直接使用
#[cfg(feature = "async")]
pub use async_trait;
use objc2::sel;
#[cfg(feature = "async")]
use tokio::sync::mpsc;

/// 高级别的同步剪贴板管理器，提供简化的 API
pub struct SyncClipboardManager {
	inner: Box<dyn Clipboard>,
}

/// 高级别的异步剪贴板管理器，提供简化的 API
#[cfg(feature = "async")]
pub struct AsyncClipboardManager {
	inner: Box<dyn AsyncClipboard>,
}

#[cfg(feature = "text")]
impl SyncClipboardManager {
	/// 创建新的同步剪贴板管理器
	pub fn new() -> Result<Self> {
		let ctx = ClipboardContext::new()?;
		Ok(Self {
			inner: Box::new(ctx),
		})
	}

	/// 获取剪贴板中所有可用的格式
	pub fn available_formats(&self) -> Result<Vec<String>> {
		self.inner.available_formats()
	}

	/// 检查剪贴板是否包含特定格式的内容
	pub fn has(&self, format: ContentFormat) -> Result<bool> {
		Ok(self.inner.has(format))
	}

	/// 清空剪贴板
	pub fn clear(&self) -> Result<()> {
		self.inner.clear()
	}

	/// 获取纯文本内容
	pub fn get_text(&self) -> Result<String> {
		self.inner.get_text()
	}

	/// 设置纯文本内容
	pub fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
		self.inner.set_text(text.as_ref())
	}

	/// 获取 HTML 内容
	pub fn get_html(&self) -> Result<String> {
		self.inner.get_html()
	}

	/// 设置 HTML 内容
	pub fn set_html(&self, html: impl AsRef<str>) -> Result<()> {
		self.inner.set_html(html.as_ref())
	}

	/// 获取 RTF 内容
	pub fn get_rtf(&self) -> Result<String> {
		self.inner.get_rtf()
	}

	/// 设置 RTF 内容
	pub fn set_rtf(&self, rtf: impl AsRef<str>) -> Result<()> {
		self.inner.set_rtf(rtf.as_ref())
	}

	/// 获取文件列表
	pub fn get_files(&self) -> Result<Vec<String>> {
		self.inner.get_files()
	}

	/// 设置文件列表
	pub fn set_files(&self, files: &[impl AsRef<str>]) -> Result<()> {
		let string_files: Vec<&str> = files.iter().map(|f| f.as_ref()).collect();
		self.inner.set_files(&string_files)
	}

	/// 创建剪贴板内容构建器
	pub fn build_content(&self) -> ClipboardContentBuilder {
		ClipboardContentBuilder::new()
	}

	/// 使用构建器设置多种内容
	pub fn set_with_builder(&self, builder: ClipboardContentBuilder) -> Result<()> {
		self.inner.set(builder)
	}

	/// 获取原始数据
	pub fn get_raw(&self, format: &str) -> Result<Vec<u8>> {
		self.inner.get_raw(format)
	}

	/// 设置原始数据
	pub fn set_raw(&self, format: &str, data: &[u8]) -> Result<()> {
		self.inner.set_raw(format, data)
	}

	pub fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		self.inner.get(formats)
	}

	/// 获取图像内容
	#[cfg(feature = "image")]
	pub fn get_image(&self) -> Result<ClipboardImage> {
		self.inner.get_image()
	}

	/// 设置图像内容
	#[cfg(feature = "image")]
	pub fn set_image(&self, image: ClipboardImage) -> Result<()> {
		self.inner.set_image(image)
	}
}

#[cfg(feature = "async")]
impl AsyncClipboardManager {
	/// 创建新的异步剪贴板管理器
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
		let string_files: Vec<&str> = files.iter().map(|f| f.as_ref()).collect();
		self.inner.set_files(&string_files).await
	}

	/// 创建剪贴板内容构建器
	pub fn build_content(&self) -> ClipboardContentBuilder {
		ClipboardContentBuilder::new()
	}

	/// 使用构建器设置多种内容
	pub async fn set(&self, builder: ClipboardContentBuilder) -> Result<()> {
		self.inner.set(builder).await
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
	#[cfg(feature = "image")]
	pub async fn get_image(&self) -> Result<ClipboardImage> {
		self.inner.get_image().await
	}

	/// 设置图像内容
	#[cfg(feature = "image")]
	pub async fn set_image(&self, image: ClipboardImage) -> Result<()> {
		self.inner.set_image(image).await
	}

	pub async fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>> {
		self.inner.get(formats).await
	}
}

/// 现代化的异步 ClipboardWatcher trait，提供更现代化的 API
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncClipboardWatcher: Send + Sync {
	/// 启动监视器并返回一个事件流
	async fn watch(&self) -> Result<ClipboardEventStream>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncClipboardWatcher for AsyncClipboardManager {
	async fn watch(&self) -> Result<ClipboardEventStream> {
		// 创建一个通道用于发送剪贴板事件
		let (sender, receiver) = mpsc::channel(100);
		// 创建一个停止信号通道
		let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
		// 启动监听器，传递停止信号接收器
		start_async_watch(sender, shutdown_rx);
		// 创建事件流
		let event_stream = ClipboardEventStream {
			receiver,
			_shutdown_tx: shutdown_tx,
		};
		Ok(event_stream)
	}
}

/// 同步剪贴板 API trait，提供基础的同步剪贴板操作
pub trait Clipboard: Send {
	/// 获取剪贴板中所有可用的格式
	fn available_formats(&self) -> Result<Vec<String>>;

	/// 检查剪贴板是否包含特定格式的内容
	fn has(&self, format: ContentFormat) -> bool;

	/// 清空剪贴板
	fn clear(&self) -> Result<()>;

	/// 获取指定格式的原始数据
	fn get_raw(&self, format: &str) -> Result<Vec<u8>>;

	/// 获取纯文本内容
	fn get_text(&self) -> Result<String>;

	/// 获取富文本内容（RTF）
	fn get_rtf(&self) -> Result<String>;

	/// 获取 HTML 内容
	fn get_html(&self) -> Result<String>;

	/// 获取文件列表
	fn get_files(&self) -> Result<Vec<String>>;

	/// 获取多种格式的内容
	fn get(&self, formats: &[ContentFormat]) -> Result<Vec<ClipboardContent>>;

	/// 设置原始数据
	fn set_raw(&self, format: &str, data: &[u8]) -> Result<()>;

	/// 设置纯文本内容
	fn set_text(&self, text: &str) -> Result<()>;

	/// 设置富文本内容
	fn set_rtf(&self, rtf: &str) -> Result<()>;

	/// 设置 HTML 内容
	fn set_html(&self, html: &str) -> Result<()>;

	/// 设置文件列表
	fn set_files(&self, files: &[&str]) -> Result<()>;

	/// 设置多种内容
	fn set(&self, builder: ClipboardContentBuilder) -> Result<()>;

	/// 获取图像内容
	#[cfg(feature = "image")]
	fn get_image(&self) -> Result<ClipboardImage>;

	/// 设置图像内容
	#[cfg(feature = "image")]
	fn set_image(&self, image: ClipboardImage) -> Result<()>;
}

/// 异步剪贴板 API trait，提供现代化的异步剪贴板操作
#[cfg(feature = "async")]
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
	async fn get_image(&self) -> Result<ClipboardImage>;

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
	async fn set_image(&self, image: ClipboardImage) -> Result<()>;

	/// 设置文件列表
	async fn set_files(&self, files: &[&str]) -> Result<()>;

	/// 设置多种内容
	async fn set(&self, builder: ClipboardContentBuilder) -> Result<()>;
}
