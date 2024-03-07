#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{ClipboardContext, ClipboardWatcherContext, WatcherShutdown};
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::{ClipboardContext, ClipboardWatcherContext, WatcherShutdown};
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
pub use x11::{ClipboardContext, ClipboardWatcherContext, WatcherShutdown};
