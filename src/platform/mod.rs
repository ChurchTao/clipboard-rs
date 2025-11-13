#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::{start_async_watch, ClipboardContext};
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{start_async_watch, ClipboardContext};
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::{start_async_watch, ClipboardContext};
#[cfg(all(
	unix,
	not(any(
		target_os = "macos",
		target_os = "ios",
		target_os = "android",
		target_os = "emscripten"
	))
))]
mod x11;
#[cfg(all(
	unix,
	not(any(
		target_os = "macos",
		target_os = "ios",
		target_os = "android",
		target_os = "emscripten"
	))
))]
pub use x11::{start_async_watch, ClipboardContext, ClipboardContextX11Options};
